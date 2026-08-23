use super::super::{
    AddressSpace, BinaryOp, BlockRef, Builtin, CompareOp, ConvertOp, KernelBuilder, KernelConstant,
    KernelDiagnostic, KernelType, OperationRef, OperationView, SerialLoopRef, StagingParameter,
    VerifiedKernel, VerifiedValueId, lower_scheduled_region,
};
use super::support::{
    COOPERATIVE_STAGING, cooperative_diagnostic, cooperative_region, cooperative_signature, guard,
    multi_round_cooperative_region, pointwise_region, pointwise_signature, scale_bias,
};
use crate::schedule::{BoundsWitnessId, OwnershipWitnessId, RegionId, StagingId};
use crate::shape::Shape;
use std::collections::BTreeMap;

// ---- Executing a cooperative body ------------------------------------------
//
// A verifier proves the body is the canonical refinement of its schedule; it
// does not prove the canonical body computes the declared order. Running it is
// what does, and the machine below reads *only* the structured kernel IR — no
// schedule, no semantic graph — so agreeing with the reference is also the
// evidence that a backend needs nothing else.

/// One typed value produced while interpreting a structured kernel.
#[derive(Clone, Copy, Debug)]
enum KirValue {
    Bool(bool),
    Index(u64),
    F32(f32),
}

impl KirValue {
    fn index(self) -> u64 {
        match self {
            Self::Index(value) => value,
            other => panic!("expected an index-typed value, found {other:?}"),
        }
    }
    fn float(self) -> f32 {
        match self {
            Self::F32(value) => value,
            other => panic!("expected an f32-typed value, found {other:?}"),
        }
    }
    fn boolean(self) -> bool {
        match self {
            Self::Bool(flag) => flag,
            other => panic!("expected a predicate value, found {other:?}"),
        }
    }
}

/// One step of a workgroup's execution, flattened past every rendezvous.
///
/// A barrier separates the lanes' execution, so every construct that *contains*
/// one has to be unrolled into this stream: a lane cannot be advanced through
/// half a round loop by an interpreter that recurses into it. Loops that contain
/// no barrier stay a single [`Self::Operation`] and are interpreted recursively,
/// which is why the staged and contributor folds cost nothing here.
#[derive(Clone, Copy, Debug)]
enum Step<'a> {
    /// An operation one lane executes in place.
    Operation(OperationRef<'a>),
    /// Every lane reaches this point before any lane passes it.
    Rendezvous,
    /// Carry a barrier-containing loop's initial values into its state.
    Seed(SerialLoopRef<'a>),
    /// Bind one iteration's induction variable and accumulator parameters.
    Iterate(SerialLoopRef<'a>, u64),
    /// Read one iteration's yields back into the carried state.
    Yield(SerialLoopRef<'a>),
    /// Publish the carried state as the loop's results.
    Exit(SerialLoopRef<'a>),
}

/// Returns whether a block, or anything nested inside it, contains a barrier.
fn contains_barrier(block: BlockRef<'_>) -> bool {
    block.operations().any(|operation| match operation.view() {
        OperationView::Barrier { .. } => true,
        OperationView::Predicated { body, .. } => contains_barrier(body),
        OperationView::SerialLoop(loops) => contains_barrier(loops.body()),
        _ => false,
    })
}

/// Flattens one block into the step stream a workgroup executes.
fn flatten<'a>(block: BlockRef<'a>, steps: &mut Vec<Step<'a>>) {
    for operation in block.operations() {
        match operation.view() {
            OperationView::Barrier { .. } => steps.push(Step::Rendezvous),
            OperationView::SerialLoop(loops) if contains_barrier(loops.body()) => {
                steps.push(Step::Seed(loops));
                for iteration in loops.start()..loops.end() {
                    steps.push(Step::Iterate(loops, iteration));
                    flatten(loops.body(), steps);
                    steps.push(Step::Yield(loops));
                }
                steps.push(Step::Exit(loops));
            }
            _ => steps.push(Step::Operation(operation)),
        }
    }
}

/// One lane's private interpreter state, carried across every rendezvous.
#[derive(Clone, Debug, Default)]
struct Lane {
    values: BTreeMap<VerifiedValueId, KirValue>,
    /// Each barrier-containing loop's carried accumulators, keyed by its own
    /// induction variable — the one value that names a loop uniquely.
    carried: BTreeMap<VerifiedValueId, Vec<KirValue>>,
}

/// A backend-shaped interpreter that reads only the structured kernel IR.
///
/// **Lanes advance one segment at a time, and each lane runs a whole segment
/// before the next lane starts it.** That is the faithful model of a control
/// barrier and it is deliberately unforgiving: a body that read a staged slot in
/// the same segment as another lane's write to it reads whatever that lane had
/// not yet stored, and a body that rewrote a slot in the same segment as another
/// lane's read of it destroys the value the reader was about to take. Both are
/// exactly the races the two synchronization evidence classes exist to prevent,
/// and both surface here as a wrong result rather than as a passing test.
struct KirMachine<'a> {
    kernel: &'a VerifiedKernel,
    input: &'a [f32],
    output: Vec<f32>,
    lane: Lane,
    local: u64,
    staged: Vec<f32>,
}

impl<'a> KirMachine<'a> {
    fn run(kernel: &'a VerifiedKernel, input: &'a [f32]) -> Vec<f32> {
        let mut buffers = kernel.buffers();
        let read = buffers.next().expect("a read buffer parameter");
        let write = buffers.next().expect("a write buffer parameter");
        assert_eq!(input.len(), usize::try_from(read.element_count).unwrap());
        let outputs = usize::try_from(write.element_count).unwrap();
        // Read from the kernel's own staging declaration, so the machine still
        // resolves nothing from the schedule or the graph.
        let slots = kernel
            .staging()
            .next()
            .map_or(1, |staging| staging.element_count.max(1));
        let participants = usize::try_from(slots).unwrap();
        let mut steps = Vec::new();
        flatten(kernel.body(), &mut steps);
        let mut machine = KirMachine {
            kernel,
            input,
            output: vec![f32::NAN; outputs],
            lane: Lane::default(),
            local: 0,
            staged: vec![f32::NAN; participants],
        };
        for workgroup in 0..outputs {
            let mut lanes = vec![Lane::default(); participants];
            machine.staged.fill(f32::NAN);
            for segment in steps.split(|step| matches!(step, Step::Rendezvous)) {
                for (lane, state) in lanes.iter_mut().enumerate() {
                    let lane = u64::try_from(lane).unwrap();
                    machine.lane = std::mem::take(state);
                    machine.local = lane;
                    let invocation = u64::try_from(workgroup).unwrap() * slots + lane;
                    for step in segment {
                        machine.run_step(*step, invocation);
                    }
                    *state = std::mem::take(&mut machine.lane);
                }
            }
        }
        machine.output
    }

    fn run_step(&mut self, step: Step<'a>, invocation: u64) {
        match step {
            Step::Operation(operation) => self.run_operation(operation, invocation),
            // Consumed by the segment split above; a lane never executes one.
            Step::Rendezvous => unreachable!("a rendezvous is a segment boundary"),
            Step::Seed(loops) => {
                let initial: Vec<KirValue> = loops.initial().map(|value| self.get(value)).collect();
                self.lane.carried.insert(Self::loop_key(loops), initial);
            }
            Step::Iterate(loops, iteration) => {
                let key = Self::loop_key(loops);
                let carried = self.lane.carried.get(&key).cloned().expect("a seeded loop");
                self.lane.values.insert(key, KirValue::Index(iteration));
                for (parameter, value) in loops.accumulators().zip(carried) {
                    self.lane.values.insert(parameter, value);
                }
            }
            Step::Yield(loops) => {
                let yielded: Vec<KirValue> = loops.yields().map(|value| self.get(value)).collect();
                self.lane.carried.insert(Self::loop_key(loops), yielded);
            }
            Step::Exit(loops) => {
                let key = Self::loop_key(loops);
                let carried = self.lane.carried.get(&key).cloned().expect("a seeded loop");
                let results: Vec<VerifiedValueId> = self.loop_results(loops);
                for (result, value) in results.into_iter().zip(carried) {
                    self.lane.values.insert(result, value);
                }
            }
        }
    }

    /// Names one barrier-containing loop by its own induction variable.
    fn loop_key(loops: SerialLoopRef<'a>) -> VerifiedValueId {
        loops.induction().expect("an induction variable")
    }

    /// Returns the values a flattened loop defines in its enclosing block.
    ///
    /// Recovered by searching the top-level operations for the loop whose
    /// induction variable matches, because a [`SerialLoopRef`] views the loop's
    /// inputs and body and not the operation that owns it.
    fn loop_results(&self, loops: SerialLoopRef<'a>) -> Vec<VerifiedValueId> {
        let key = Self::loop_key(loops);
        self.kernel
            .body()
            .operations()
            .find(|operation| match operation.view() {
                OperationView::SerialLoop(candidate) => candidate.induction() == Some(key),
                _ => false,
            })
            .expect("a flattened loop is a top-level operation")
            .results()
            .collect()
    }

    fn run_block(&mut self, block: BlockRef<'a>, invocation: u64) {
        for operation in block.operations() {
            self.run_operation(operation, invocation);
        }
    }

    fn run_operation(&mut self, operation: OperationRef<'a>, invocation: u64) {
        let mut results = operation.results();
        match operation.view() {
            OperationView::Builtin { builtin } => {
                let value = match builtin {
                    Builtin::GlobalInvocationIndex => invocation,
                    Builtin::LocalInvocationIndex => self.local,
                };
                self.define(&mut results, KirValue::Index(value));
            }
            OperationView::Constant { value } => {
                let value = match value {
                    KernelConstant::Bool(flag) => KirValue::Bool(flag),
                    KernelConstant::Index(index) => KirValue::Index(index),
                    KernelConstant::F32Bits(bits) => KirValue::F32(f32::from_bits(bits)),
                    // This machine executes cooperative reductions, and every
                    // reduction family in this vocabulary is `f32`; a `bf16`
                    // constant reaching it would mean the fixture had drifted
                    // into a program it cannot model, which is a defect to
                    // report rather than a value to guess at.
                    KernelConstant::Bf16Bits(bits) => {
                        panic!("no cooperative fixture carries the bf16 constant {bits:#06x}")
                    }
                };
                self.define(&mut results, value);
            }
            OperationView::Binary { op, lhs, rhs } => {
                let value = match op {
                    BinaryOp::IndexAdd => {
                        KirValue::Index(self.get(lhs).index() + self.get(rhs).index())
                    }
                    BinaryOp::IndexMultiply => {
                        KirValue::Index(self.get(lhs).index() * self.get(rhs).index())
                    }
                    BinaryOp::IndexDivide => {
                        KirValue::Index(self.get(lhs).index() / self.get(rhs).index())
                    }
                    BinaryOp::IndexModulo => {
                        KirValue::Index(self.get(lhs).index() % self.get(rhs).index())
                    }
                    BinaryOp::F32Add => {
                        KirValue::F32(self.get(lhs).float() + self.get(rhs).float())
                    }
                    BinaryOp::F32Multiply => {
                        KirValue::F32(self.get(lhs).float() * self.get(rhs).float())
                    }
                    other => panic!("unsupported binary operation {other:?}"),
                };
                self.define(&mut results, value);
            }
            OperationView::Compare { op, lhs, rhs } => {
                let value = match op {
                    CompareOp::IndexLessThan => {
                        KirValue::Bool(self.get(lhs).index() < self.get(rhs).index())
                    }
                };
                self.define(&mut results, value);
            }
            OperationView::Convert { op, source } => {
                let value = self.get(source).float();
                let value = match op {
                    ConvertOp::CanonicalizeF32Nan => {
                        if value.is_nan() {
                            f32::from_bits(self.kernel.numerical().canonical_arithmetic_nan_bits)
                        } else {
                            value
                        }
                    }
                    other => panic!("unsupported conversion {other:?}"),
                };
                self.define(&mut results, KirValue::F32(value));
            }
            OperationView::Load { offset, .. } => {
                let offset = usize::try_from(self.get(offset).index()).unwrap();
                let value = KirValue::F32(self.input[offset]);
                self.define(&mut results, value);
            }
            OperationView::GuardedLoad {
                predicate,
                offset,
                inactive,
                ..
            } => {
                let value = if self.get(predicate).boolean() {
                    let offset = usize::try_from(self.get(offset).index()).unwrap();
                    KirValue::F32(self.input[offset])
                } else {
                    self.get(inactive)
                };
                self.define(&mut results, value);
            }
            OperationView::Store { offset, value, .. } => {
                let offset = usize::try_from(self.get(offset).index()).unwrap();
                self.output[offset] = self.get(value).float();
            }
            OperationView::Predicated { predicate, body } => {
                if self.get(predicate).boolean() {
                    self.run_block(body, invocation);
                }
            }
            OperationView::SerialLoop(loops) => {
                let mut carried: Vec<KirValue> =
                    loops.initial().map(|value| self.get(value)).collect();
                let induction = loops.induction().expect("an induction variable");
                let parameters: Vec<_> = loops.accumulators().collect();
                for iteration in loops.start()..loops.end() {
                    self.lane
                        .values
                        .insert(induction, KirValue::Index(iteration));
                    for (parameter, value) in parameters.iter().zip(&carried) {
                        self.lane.values.insert(*parameter, *value);
                    }
                    self.run_block(loops.body(), invocation);
                    carried = loops.yields().map(|value| self.get(value)).collect();
                }
                for (result, value) in results.zip(carried) {
                    self.lane.values.insert(result, value);
                }
            }
            // Flattened into a segment boundary before any lane runs, so a
            // barrier reaching here is one nested below a construct this machine
            // descends into — which the verifier refuses.
            OperationView::Barrier { .. } => panic!("a nested barrier reached the machine"),
            OperationView::StagedStore { offset, value, .. } => {
                let offset = usize::try_from(self.get(offset).index()).unwrap();
                self.staged[offset] = self.get(value).float();
            }
            OperationView::StagedLoad { offset, .. } => {
                let offset = usize::try_from(self.get(offset).index()).unwrap();
                let value = KirValue::F32(self.staged[offset]);
                self.define(&mut results, value);
            }
            other => panic!("unsupported structured operation {other:?}"),
        }
    }

    fn define(&mut self, results: &mut impl Iterator<Item = VerifiedValueId>, value: KirValue) {
        let result = results.next().expect("one defined result");
        self.lane.values.insert(result, value);
    }

    fn get(&self, id: VerifiedValueId) -> KirValue {
        *self
            .lane
            .values
            .get(&id)
            .expect("a value defined before its use")
    }
}

/// Two rows whose sum depends on where the round boundaries fall.
///
/// `5e19` is far enough above the unit ulp that adding one to it is the
/// identity, so a grouping that puts the cancelling pair in one round absorbs
/// the small value beside it and a grouping that splits them does not. The two
/// rows are sensitive in opposite directions, so neither the round-major nor the
/// participant-major grouping can agree with the other by luck on both.
/// Each row also carries a small value in a round the other's cancellation does
/// not reach, so a body that folded round zero's range twice — the shape a
/// dropped round term produces — disagrees on both rows rather than on one.
const REGROUPING_SENSITIVE_ROWS: [[f32; 6]; 2] = [
    [5.0e19, 1.0, -5.0e19, 3.0, 0.0, 0.0],
    [0.0, 5.0e19, 0.0, -5.0e19, 2.0, 0.0],
];

/// The exact value a cooperative tile's declared order computes for one row.
///
/// Written from the declared arithmetic rather than from the emitted body:
/// participant `p` of round `r` folds the contiguous range at index
/// `r * participants + p` seeded at its own first contributor, the staged set is
/// folded in ascending participant order, and the round totals accumulate in
/// ascending round order. Every fold seeds at its first contributor, which is
/// what makes this a reassociation of the declared sequence rather than a sum
/// against an identity element.
fn cooperative_reference(
    row: &[f32],
    participants: usize,
    contributors: usize,
    rounds: usize,
) -> f32 {
    let mut total: Option<f32> = None;
    for round in 0..rounds {
        let mut staged: Option<f32> = None;
        for participant in 0..participants {
            let base = (round * participants + participant) * contributors;
            let mut range = row[base];
            for step in 1..contributors {
                range += row[base + step];
            }
            staged = Some(staged.map_or(range, |value| value + range));
        }
        let round_total = staged.expect("a tile has at least one participant");
        total = Some(total.map_or(round_total, |value| value + round_total));
    }
    total.expect("a tile runs at least one round")
}

/// The same fold with the rounds and participants exchanged.
///
/// Participant `p` of round `r` owning the range at `p * rounds + r` is the
/// other natural reading of a two-dimensional split, and it is the one the
/// contributor arithmetic must *not* compute.
fn participant_major_reference(
    row: &[f32],
    participants: usize,
    contributors: usize,
    rounds: usize,
) -> f32 {
    let mut total: Option<f32> = None;
    for round in 0..rounds {
        let mut staged: Option<f32> = None;
        for participant in 0..participants {
            let base = (participant * rounds + round) * contributors;
            let mut range = row[base];
            for step in 1..contributors {
                range += row[base + step];
            }
            staged = Some(staged.map_or(range, |value| value + range));
        }
        let round_total = staged.expect("a tile has at least one participant");
        total = Some(total.map_or(round_total, |value| value + round_total));
    }
    total.expect("a tile runs at least one round")
}

fn bit_patterns(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

/// The neighbouring round grouping really does compute something else.
///
/// The guard on the conformance test below: an executed kernel agreeing with its
/// declared order is only evidence if some *other* order would have disagreed.
/// This pins that, so an input that made the comparison vacuous fails here rather
/// than silently weakening the claim next door.
#[test]
fn the_declared_round_grouping_is_what_the_agreement_is_evidence_about() {
    for row in &REGROUPING_SENSITIVE_ROWS {
        assert_ne!(
            cooperative_reference(row, 3, 1, 2).to_bits(),
            participant_major_reference(row, 3, 1, 2).to_bits(),
            "the conformance input cannot tell two round groupings apart"
        );
    }
}

/// The single-round body executes to the reference's bits at its declared order.
///
/// Run first and reported separately, because it is what anchors the machine:
/// the single-round shape is already verified, already has a checked-in Metal
/// golden, and its order is not in question — so a disagreement here is a defect
/// in the interpreter rather than in the body under test.
#[test]
fn the_cooperative_body_matches_the_reference_at_its_declared_order() {
    let scheduled = cooperative_region();
    let kernel = lower_scheduled_region(&scheduled).expect("the cooperative region lowers");
    let input: Vec<f32> = REGROUPING_SENSITIVE_ROWS.concat();
    let expected: Vec<f32> = REGROUPING_SENSITIVE_ROWS
        .iter()
        .map(|row| cooperative_reference(row, 3, 2, 1))
        .collect();
    assert_eq!(
        bit_patterns(&KirMachine::run(&kernel, &input)),
        bit_patterns(&expected)
    );
}

/// The loop-carried body executes to the reference's bits at its declared order.
///
/// The ticket's closing evidence. The kernel is *run* rather than inspected:
/// every lane is advanced to each barrier before any lane crosses it, so a body
/// that read a staged slot before its writer produced it, or rewrote one before
/// its readers were finished, would carry a `NaN` or a next-round partial into
/// the fold and fail here rather than pass by accident.
#[test]
fn the_loop_carried_body_matches_the_reference_at_its_declared_order() {
    let scheduled = multi_round_cooperative_region();
    let kernel = lower_scheduled_region(&scheduled).expect("the loop-carried region lowers");
    let input: Vec<f32> = REGROUPING_SENSITIVE_ROWS.concat();
    let expected: Vec<f32> = REGROUPING_SENSITIVE_ROWS
        .iter()
        .map(|row| cooperative_reference(row, 3, 1, 2))
        .collect();
    assert_eq!(
        bit_patterns(&KirMachine::run(&kernel, &input)),
        bit_patterns(&expected)
    );
}

/// A kernel that stages without a cooperative region is refused by name.
#[test]
fn a_staged_access_without_a_tile_is_refused() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    // A region with no tile may declare no workgroup storage at all, so the
    // declaration is refused before a staged operation could even name one.
    builder.declare_staging(COOPERATIVE_STAGING).unwrap();
    assert_eq!(
        cooperative_diagnostic(builder),
        KernelDiagnostic::StagingContract
    );
}

/// Staging that does not realize the region's tile is refused.
#[test]
fn staging_that_contradicts_the_region_tile_is_refused() {
    for staging in [
        StagingParameter {
            element_count: 4,
            ..COOPERATIVE_STAGING
        },
        StagingParameter {
            address_space: AddressSpace::Device,
            ..COOPERATIVE_STAGING
        },
        StagingParameter {
            element_type: KernelType::U8,
            ..COOPERATIVE_STAGING
        },
        StagingParameter {
            staging: StagingId::new(1),
            ..COOPERATIVE_STAGING
        },
    ] {
        let scheduled = cooperative_region();
        let mut builder = KernelBuilder::new(&scheduled).unwrap();
        cooperative_signature(&mut builder, &scheduled);
        builder.declare_staging(staging).unwrap();
        assert_eq!(
            cooperative_diagnostic(builder),
            KernelDiagnostic::StagingContract,
            "{staging:?} was admitted against the region's tile"
        );
    }

    // The count itself, in both directions.
    let scheduled = cooperative_region();
    let mut missing = KernelBuilder::new(&scheduled).unwrap();
    cooperative_signature(&mut missing, &scheduled);
    assert_eq!(
        cooperative_diagnostic(missing),
        KernelDiagnostic::StagingContract
    );
    let mut extra = KernelBuilder::new(&scheduled).unwrap();
    cooperative_signature(&mut extra, &scheduled);
    extra.declare_staging(COOPERATIVE_STAGING).unwrap();
    extra
        .declare_staging(StagingParameter {
            staging: StagingId::new(1),
            ..COOPERATIVE_STAGING
        })
        .unwrap();
    assert_eq!(
        cooperative_diagnostic(extra),
        KernelDiagnostic::StagingContract
    );
}
