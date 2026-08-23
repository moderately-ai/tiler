//! Lowering and addressing tests for the data-dependent gather read.
//!
//! The property under test is **which element the body reads**, not that a body
//! exists. A gather's address is a sum of direct coordinates and one coordinate
//! loaded from an index operand, and every plausible way of getting that sum
//! wrong — exchanging the strides, dropping a wrap, scaling the loaded
//! coordinate by the wrong axis — produces an address that is still inside the
//! source buffer and still names an element. No bounds check, no verifier, and
//! no "a kernel was produced" assertion can separate those from the right one.
//!
//! So the emitted body is executed and its recorded memory offsets are compared
//! against a derivation written from **coordinates**: the result domain is
//! enumerated as coordinate tuples, each tuple is split into the source prefix,
//! the index run, and the source suffix by the composition
//! [`crate::semantic::gather_result_shape`] performs, and the source position is
//! the row-major offset of `prefix ++ [loaded] ++ suffix`. Nothing in
//! [`expected_addresses`] reads an `OffsetTerm`, a divisor, or a stride from the
//! lowering; it shares only the definition of row-major order.
//!
//! # The fixtures reach the inhabited closed argument
//!
//! Only a statically proved gather reaches schedule formation, and the closed
//! arguments are an empty result domain and a gathered extent containing the
//! whole U32 space. An empty domain would make every address vacuous and could
//! hide any arithmetic defect at all, so every fixture here takes the inhabited
//! argument: the gathered axis has extent `2^32`. Its buffer is never
//! materialized — the machine records offsets rather than reading values — so
//! the extent costs nothing.

use std::collections::BTreeMap;

use super::super::{
    AddressSpace, BinaryOp, BlockRef, BufferAccess, BufferParameter, Builtin, CompareOp, ConvertOp,
    KernelBuilder, KernelConstant, KernelDiagnostic, KernelType, OperationView, VerifiedBufferId,
    VerifiedKernel, VerifiedValueId, lower_scheduled_region,
};
use super::support::{diagnostics, guard, linear_schedule, numerical};
use crate::index::{
    DomainRole, GatherIndexBoundsProof, GatherIndexBoundsProofKind, IndexRegionBuilder,
    TensorAccessView,
};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PointwiseF32ExpressionBuilder, RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder,
    TensorRole, VerifiedScheduledRegion, element_count, gather_index_read_map,
};
use crate::semantic::{F32, gather_index_resolved_type};
use crate::shape::{Axis, Shape};

/// The gathered extent every fixture takes, chosen to reach `2^32`.
const GATHERED_EXTENT: u64 = 1 << 32;

/// One gather occurrence, stated once and read by the fixture and the oracle.
#[derive(Clone, Debug)]
struct Occurrence {
    source: Shape,
    index: Shape,
    axis: Axis,
    /// The index operand's contents, in row-major order over `index`.
    ///
    /// Deliberately not a permutation of `0..n` and deliberately not ascending:
    /// an oracle and a body that disagree about which coordinate an address
    /// carries agree anyway on an identity index, and an ascending one hides an
    /// exchange of the gathered axis with a neighbouring one.
    values: Vec<u32>,
}

impl Occurrence {
    fn result(&self) -> Shape {
        crate::semantic::gather_result_shape(self.axis, &self.source, &self.index)
            .expect("the fixture is a well-formed gather")
            .1
    }

    fn gathered_axis(&self) -> usize {
        usize::try_from(self.axis.get()).expect("a bounded axis")
    }
}

/// A rank-two source gathered on its leading axis by a rank-one index.
///
/// The narrow case: one direct source axis, which trails the index run.
fn leading_axis() -> Occurrence {
    Occurrence {
        source: Shape::from_dims([GATHERED_EXTENT, 3]),
        index: Shape::from_dims([2]),
        axis: Axis::new(0),
        values: vec![7, 1],
    }
}

/// A rank-three source gathered on its middle axis by a rank-two index.
///
/// The case that separates the halves of the composition: one source axis sits
/// *before* the gathered one and keeps its result position, one sits after it
/// and is displaced by the whole index run, and the index run itself is two
/// axes rather than one. A body that placed the displacement on the wrong side,
/// or that decoded the index run as a single axis, addresses inside the source
/// and reads a different element.
fn middle_axis() -> Occurrence {
    Occurrence {
        source: Shape::from_dims([2, GATHERED_EXTENT, 3]),
        index: Shape::from_dims([2, 5]),
        axis: Axis::new(1),
        values: vec![9, 3, 0, 4_000_000_000, 11, 2, 8, 1, 5, 6],
    }
}

/// A rank-zero index: one address, read by every invocation.
///
/// The third relation [`gather_index_read_map`] derives, and the one whose
/// address is *not* a function of the invocation at all. A body that addressed
/// it by the invocation index would read a different element of a one-element
/// tensor per invocation.
fn scalar_index() -> Occurrence {
    Occurrence {
        source: Shape::from_dims([GATHERED_EXTENT, 3]),
        index: Shape::from_dims([]),
        axis: Axis::new(0),
        values: vec![2_500_000_000],
    }
}

/// Enumerates a shape's coordinate tuples in row-major order.
fn coordinates(shape: &Shape) -> Vec<Vec<u64>> {
    let extents: Vec<u64> = shape.extents().iter().map(|extent| extent.get()).collect();
    let mut tuples = vec![Vec::new()];
    for extent in extents {
        tuples = tuples
            .into_iter()
            .flat_map(|prefix| {
                (0..extent).map(move |coordinate| {
                    let mut next = prefix.clone();
                    next.push(coordinate);
                    next
                })
            })
            .collect();
    }
    tuples
}

/// Returns the row-major element offset of one coordinate tuple.
fn row_major(shape: &Shape, coordinates: &[u64]) -> u64 {
    let extents: Vec<u64> = shape.extents().iter().map(|extent| extent.get()).collect();
    assert_eq!(coordinates.len(), extents.len());
    let mut offset = 0_u64;
    for (coordinate, extent) in coordinates.iter().zip(&extents) {
        offset = offset
            .checked_mul(*extent)
            .and_then(|scaled| scaled.checked_add(*coordinate))
            .expect("a bounded offset");
    }
    offset
}

/// The addresses a correct gather body reads, derived from coordinates alone.
///
/// One entry per invocation, in invocation order: the index-operand offset the
/// address is loaded from, and the source offset the gathered value is read at.
/// Nothing here consults the lowering.
fn expected_addresses(occurrence: &Occurrence) -> Vec<(u64, u64)> {
    let gathered = occurrence.gathered_axis();
    let index_rank = occurrence.index.rank();
    coordinates(&occurrence.result())
        .into_iter()
        .map(|result| {
            // The result domain is `source[..axis] ++ index ++ source[axis + 1..]`,
            // split back into the three runs it was composed from.
            let (prefix, rest) = result.split_at(gathered);
            let (index, suffix) = rest.split_at(index_rank);
            let index_offset = row_major(&occurrence.index, index);
            let loaded = u64::from(
                occurrence.values[usize::try_from(index_offset).expect("a bounded index")],
            );
            let mut source: Vec<u64> = prefix.to_vec();
            source.push(loaded);
            source.extend_from_slice(suffix);
            (index_offset, row_major(&occurrence.source, &source))
        })
        .collect()
}

/// Reads one index-region dimension as a coordinate expression.
///
/// A free function rather than a closure: it borrows the builder for exactly
/// one call, which is what lets the whole result coordinate run below be built
/// against the same builder before any of it is sliced.
fn coordinate(
    builder: &mut IndexRegionBuilder,
    dimension: crate::index::DimensionId,
) -> crate::index::IndexExprId {
    builder
        .dimension_expr(dimension)
        .expect("a dimension coordinate is admitted")
}

/// Mints one real static gather proof for this occurrence through the index layer.
///
/// There is no other way to obtain one: the retained proof has no public
/// constructor and is minted solely by the index layer's verifier-private
/// deriver, so a fixture holding one is evidence that the closed static
/// argument actually ran over these exact shapes.
fn static_proof(occurrence: &Occurrence) -> GatherIndexBoundsProof {
    let registry =
        crate::index::FrozenScalarRegistry::standard().expect("the scalar profile composes");
    let mut builder = IndexRegionBuilder::new(registry).expect("a builder is admitted");
    let result = occurrence.result();
    let dimensions: Vec<_> = result
        .extents()
        .iter()
        .map(|extent| {
            builder
                .dimension(DomainRole::Parallel, *extent)
                .expect("a parallel dimension is admitted")
        })
        .collect();
    let source = builder
        .tensor(
            crate::index::TensorRole::Input,
            F32::resolved_type().clone(),
            occurrence.source.clone(),
        )
        .expect("the source boundary is admitted");
    let index = builder
        .tensor(
            crate::index::TensorRole::Input,
            gather_index_resolved_type(),
            occurrence.index.clone(),
        )
        .expect("the index boundary is admitted");
    let output = builder
        .tensor(
            crate::index::TensorRole::Output,
            F32::resolved_type().clone(),
            result.clone(),
        )
        .expect("the output boundary is admitted");
    let mut result_coordinates = Vec::with_capacity(dimensions.len());
    for dimension in &dimensions {
        result_coordinates.push(coordinate(&mut builder, *dimension));
    }
    let gathered = occurrence.gathered_axis();
    let index_rank = occurrence.index.rank();
    // The same split the oracle performs: the result dimensions before the
    // gathered axis and after the index run address the source's other axes,
    // and the index run addresses the index tensor.
    let source_coordinates: Vec<_> = result_coordinates[..gathered]
        .iter()
        .chain(&result_coordinates[gathered + index_rank..])
        .copied()
        .collect();
    let index_coordinates = result_coordinates[gathered..gathered + index_rank].to_vec();
    let value = builder
        .gather_read(
            source,
            index,
            &dimensions,
            &source_coordinates,
            &index_coordinates,
            occurrence.axis,
        )
        .expect("the gather is admitted");
    let write = builder
        .write(output, &dimensions, &result_coordinates)
        .expect("the write is admitted");
    builder
        .output(write, value)
        .expect("the output is admitted");
    let region = builder.build().expect("the index region verifies");
    let proof = region
        .accesses()
        .find_map(|access| match access.view() {
            TensorAccessView::GatherRead(gather) => gather.bounds_resolution().statically_proved(),
            TensorAccessView::Direct(_) => None,
        })
        .expect("the gathered extent contains every U32 value, so the obligation is proved")
        .clone();
    assert_eq!(
        proof.kind(),
        GatherIndexBoundsProofKind::U32RangeContainedBySourceExtent,
        "every fixture must rest on the inhabited argument, not on vacuity",
    );
    proof
}

/// Builds the verified scheduled region of one gather occurrence.
///
/// The canonical access order the accepted surface fixes: the source read at
/// local access 0, the address-only U32 read it owns at access 1, the owning
/// write at access 2.
fn gather_region(occurrence: &Occurrence) -> VerifiedScheduledRegion {
    let result = occurrence.result();
    let result_elements = element_count(&result).expect("a bounded result");
    let owner = OwnershipWitnessId::new(0);
    let address = gather_index_read_map(&occurrence.source, occurrence.axis, &occurrence.index)
        .expect("the fixture is a well-formed gather");
    let address_elements = element_count(&occurrence.index).expect("a bounded index");
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(3));
    builder
        .iteration_shape(result.clone())
        .expect("the iteration shape is admitted");
    let read = |map: LogicalAccess, witness: u32| Access {
        tensor: TensorRole::Input,
        component_role: None,
        mode: AccessMode::Read,
        map,
        bounds: BoundsWitnessId::new(witness),
        ownership: None,
    };
    builder
        .push_access(read(
            LogicalAccess::GatherSource {
                source_shape: occurrence.source.clone(),
                result_shape: result.clone(),
                axis: occurrence.axis,
                index_access: AccessOrdinal::new(1),
                index_shape: occurrence.index.clone(),
            },
            0,
        ))
        .expect("the gather source is admitted");
    builder
        .push_access(read(address, 1))
        .expect("the address operand is admitted");
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(owner),
        })
        .expect("the write is admitted");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::GatherSource {
                source_shape: occurrence.source.clone(),
                result_shape: result.clone(),
                axis: occurrence.axis,
                index_access: AccessOrdinal::new(1),
                index_shape: occurrence.index.clone(),
                proof: Box::new(static_proof(occurrence)),
            },
        })
        .expect("the gather proof is admitted");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: address_elements,
            },
        })
        .expect("the address proof is admitted");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: result_elements,
            },
        })
        .expect("the write proof is admitted");
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: result_elements,
            },
        })
        .expect("the ownership proof is admitted");
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let leaf = expression
        .input(AccessOrdinal::FIRST)
        .expect("the gathered value is the sole leaf");
    let expression = expression.build(leaf).expect("the identity composes");
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(expression),
            numerical: numerical(),
        })
        .expect("the program is admitted");
    builder
        .schedule(linear_schedule(result_elements, owner))
        .expect("the schedule is admitted");
    builder.build().expect("the gather region verifies")
}

/// One structured-kernel SSA value, at the four types a gather body carries.
#[derive(Clone, Copy, Debug, PartialEq)]
enum KirValue {
    Bool(bool),
    Index(u64),
    U32(u32),
    F32(f32),
}

impl KirValue {
    fn index(self) -> u64 {
        match self {
            Self::Index(value) => value,
            other => panic!("expected an index value, got {other:?}"),
        }
    }

    fn boolean(self) -> bool {
        match self {
            Self::Bool(flag) => flag,
            other => panic!("expected a predicate, got {other:?}"),
        }
    }
}

/// A backend-shaped machine that executes one invocation of a gather body.
///
/// It reads the structured kernel IR and nothing else — no schedule, no
/// relation, no shape — so what it records is what a backend translating this
/// kernel would emit. A load returns the value the buffer's *declared element
/// type* says it holds, which is what makes an index operand declared at the
/// wrong type a failure here rather than a plausible number.
struct GatherMachine<'a> {
    /// Each buffer's signature ordinal and declared element type.
    buffers: BTreeMap<VerifiedBufferId, (usize, KernelType)>,
    /// The index operand's contents, addressed by the offsets the body computes.
    index_values: &'a [u32],
    values: BTreeMap<VerifiedValueId, KirValue>,
    /// Element offsets loaded, paired with the buffer ordinal, in program order.
    loads: Vec<(usize, u64)>,
    /// Element offsets stored to, in program order.
    stores: Vec<u64>,
}

impl<'a> GatherMachine<'a> {
    fn run(
        kernel: &'a VerifiedKernel,
        index_values: &'a [u32],
        invocation: u64,
    ) -> (Vec<(usize, u64)>, Vec<u64>) {
        let buffers = kernel
            .declared_buffers()
            .enumerate()
            .map(|(ordinal, (id, parameter))| (id, (ordinal, parameter.element_type)))
            .collect();
        let mut machine = GatherMachine {
            buffers,
            index_values,
            values: BTreeMap::new(),
            loads: Vec::new(),
            stores: Vec::new(),
        };
        machine.run_block(kernel.body(), invocation);
        (machine.loads, machine.stores)
    }

    fn get(&self, value: VerifiedValueId) -> KirValue {
        *self.values.get(&value).expect("a defined SSA value")
    }

    fn run_block(&mut self, block: BlockRef<'a>, invocation: u64) {
        for operation in block.operations() {
            let mut results = operation.results();
            let mut define = |machine: &mut Self, value: KirValue| {
                let id = results.next().expect("a single-result operation");
                machine.values.insert(id, value);
            };
            match operation.view() {
                OperationView::Builtin {
                    builtin: Builtin::GlobalInvocationIndex,
                } => define(self, KirValue::Index(invocation)),
                OperationView::Constant {
                    value: KernelConstant::Index(constant),
                } => define(self, KirValue::Index(constant)),
                OperationView::Binary { op, lhs, rhs } => {
                    let (lhs, rhs) = (self.get(lhs).index(), self.get(rhs).index());
                    let value = match op {
                        BinaryOp::IndexAdd => lhs + rhs,
                        BinaryOp::IndexMultiply => lhs * rhs,
                        BinaryOp::IndexDivide => lhs / rhs,
                        BinaryOp::IndexModulo => lhs % rhs,
                        other => panic!("a gather body emitted {other:?}"),
                    };
                    define(self, KirValue::Index(value));
                }
                OperationView::Compare {
                    op: CompareOp::IndexLessThan,
                    lhs,
                    rhs,
                } => {
                    let value = self.get(lhs).index() < self.get(rhs).index();
                    define(self, KirValue::Bool(value));
                }
                OperationView::Convert {
                    op: ConvertOp::U32ToIndex,
                    source,
                } => {
                    let KirValue::U32(loaded) = self.get(source) else {
                        panic!("the widening conversion took a value that is not U32");
                    };
                    define(self, KirValue::Index(u64::from(loaded)));
                }
                OperationView::Load { buffer, offset, .. } => {
                    let (ordinal, element_type) = self.buffers[&buffer];
                    let offset = self.get(offset).index();
                    self.loads.push((ordinal, offset));
                    let value = match element_type {
                        KernelType::U32 => KirValue::U32(
                            usize::try_from(offset)
                                .ok()
                                .and_then(|position| self.index_values.get(position))
                                .copied()
                                .unwrap_or_else(|| {
                                    panic!(
                                        "the body addressed element {offset} of a {}-element index operand",
                                        self.index_values.len(),
                                    )
                                }),
                        ),
                        // The gathered element itself. Its payload is never
                        // asserted on: what this machine is here to record is
                        // the offset it was read from.
                        KernelType::F32 => KirValue::F32(0.0),
                        other => panic!("a gather body loaded from a {other:?} buffer"),
                    };
                    define(self, value);
                }
                OperationView::Store { offset, .. } => {
                    let offset = self.get(offset).index();
                    self.stores.push(offset);
                }
                OperationView::Predicated { predicate, body } => {
                    if self.get(predicate).boolean() {
                        self.run_block(body, invocation);
                    }
                }
                other => panic!("a gather body emitted {other:?}"),
            }
        }
    }
}

/// Runs every invocation of a lowered gather and returns its recorded addresses.
///
/// One entry per invocation: the offset read from the index operand and the
/// offset read from the source. The buffer *ordinals* are asserted rather than
/// assumed, so a body that read its address out of the source tensor — or its
/// value out of the index tensor — fails here instead of comparing two numbers
/// that happen to agree.
fn observed_addresses(occurrence: &Occurrence) -> Vec<(u64, u64)> {
    let kernel = lower_scheduled_region(&gather_region(occurrence))
        .expect("a statically proved gather lowers to a verified kernel");
    let invocations = element_count(&occurrence.result()).expect("a bounded result");
    (0..invocations)
        .map(|invocation| {
            let (loads, stores) = GatherMachine::run(&kernel, &occurrence.values, invocation);
            assert_eq!(
                stores,
                vec![invocation],
                "one owning store, at this invocation's own output position",
            );
            let [
                (address_buffer, address_offset),
                (source_buffer, source_offset),
            ] = loads[..]
            else {
                panic!("a gather invocation performs exactly two loads, got {loads:?}");
            };
            assert_eq!(
                address_buffer, 1,
                "the address is read from the index operand"
            );
            assert_eq!(
                source_buffer, 0,
                "the value is read from the gathered source"
            );
            (address_offset, source_offset)
        })
        .collect()
}

/// The emitted addresses are the ones the relation names, on every invocation.
///
/// **Watched failing under four separate subject perturbations**, each on the
/// lowering rather than on the assertion, and each quoted from the run:
///
/// - `emit_gather_offset` scaling the loaded coordinate by `1` instead of by
///   the gathered axis's own stride reports, on the leading-axis occurrence,
///   `left: [(0, 7), (0, 8), (0, 9), (1, 1), (1, 2), (1, 3)]` against
///   `right: [(0, 21), (0, 22), (0, 23), (1, 3), (1, 4), (1, 5)]`;
/// - `gather_direct_terms` taking the *result's* stride for a direct term
///   instead of the source's leaves the leading-axis occurrence passing — the
///   two strides coincide there — and reddens the middle-axis one, where the
///   thirty-first pair reads `(0, 57)` against `(0, 12884901915)`. That is
///   what the middle-axis fixture is for: one occurrence is not a population;
/// - `gather_direct_terms` emitting `modulus: None` for every term reports
///   `left: [(0, 21), (0, 22), (0, 23), (1, 6), (1, 7), (1, 8)]` against
///   `right: [(0, 21), (0, 22), (0, 23), (1, 3), (1, 4), (1, 5)]`;
/// - `gather_address_addressing` answering `Identity` for the scalar broadcast
///   instead of the empty term list reports
///   `the body addressed element 1 of a 1-element index operand`.
///
/// Every source address above is inside a buffer of 12,884,901,888 elements,
/// which is exactly why this compares addresses instead of asserting that a
/// kernel was produced: a bounds check cannot separate any of them.
#[test]
fn a_lowered_gather_reads_the_element_its_relation_names() {
    for occurrence in [leading_axis(), middle_axis(), scalar_index()] {
        assert_eq!(
            observed_addresses(&occurrence),
            expected_addresses(&occurrence),
            "{occurrence:?}",
        );
    }
}

/// The oracle and the body disagree when the subject moves.
///
/// A comparison that cannot say *no* proves nothing, and the perturbations
/// quoted above are the demonstration for the body. This is the demonstration
/// for the **oracle**: it is driven with an index operand the fixture does not
/// carry, and the two disagree. Without it, an oracle that had quietly stopped
/// reading its input would agree with every body forever.
#[test]
fn the_address_oracle_separates_two_different_index_operands() {
    let occurrence = middle_axis();
    let mut moved = occurrence.clone();
    moved.values[0] += 1;
    assert_ne!(
        expected_addresses(&occurrence),
        expected_addresses(&moved),
        "a changed index value must move the address it supplies",
    );
    assert_eq!(
        observed_addresses(&occurrence),
        expected_addresses(&occurrence),
    );
    assert_ne!(
        observed_addresses(&occurrence),
        expected_addresses(&moved),
        "the body reads the operand it was given, not a neighbouring one",
    );
}

/// The address operand is a U32 buffer beside the region's `f32` boundary.
///
/// The signature is the fact a backend binds against, and it is the one place
/// the four-byte unsigned input could be silently reinterpreted. Element counts
/// are asserted with it because a buffer declared at the right type over the
/// wrong population is the same class of defect.
///
/// **Watched failing under two subject perturbations on opposite sides of the
/// one derivation**, which is what shows both readers of `gather_address_reads`
/// are load-bearing rather than one of them agreeing with itself. Declaring
/// every read at the region element type makes the widening conversion
/// unstatable, and the *builder* refuses before verification with
/// `the gather lowers: Construction(TypeMismatch { expected: U32, actual: F32 })`.
/// Leaving the declaration correct and reverting `verify_signature` to
/// `vec![element_type; reads.len()]` reddens instead with
/// `the gather lowers: Verification(BufferContract)`.
#[test]
fn a_gather_signature_declares_its_index_operand_at_the_exact_unsigned_width() {
    let occurrence = middle_axis();
    let kernel = lower_scheduled_region(&gather_region(&occurrence)).expect("the gather lowers");
    let signature: Vec<_> = kernel
        .buffers()
        .map(|buffer| (buffer.element_type, buffer.element_count))
        .collect();
    assert_eq!(
        signature,
        vec![
            (KernelType::F32, element_count(&occurrence.source).unwrap()),
            (KernelType::U32, element_count(&occurrence.index).unwrap()),
            (
                KernelType::F32,
                element_count(&occurrence.result()).unwrap()
            ),
        ],
    );
}

/// A lowered gather is the canonical body its own refinement gate re-derives.
///
/// The gate is what makes a producer-authored kernel a proven refinement rather
/// than a trusted one, and it runs `derive_canonical` over the same region. A
/// body reaching it is therefore evidence that the gather arm is reached from
/// both directions rather than only from `lower_scheduled_region`.
#[test]
fn a_lowered_gather_survives_its_own_refinement_gate() {
    for occurrence in [leading_axis(), middle_axis(), scalar_index()] {
        let region = gather_region(&occurrence);
        let kernel = lower_scheduled_region(&region).expect("the gather lowers");
        assert_eq!(
            kernel.scheduled_region_identity(),
            region.canonical_identity(),
        );
    }
}

/// A producer that ignores the index operand is refused as a non-refinement.
///
/// **The failure mode this lane exists to keep out.** The body below is
/// structurally impeccable — its signature matches the region, its one load is
/// bounds-witnessed and predicate-dominated, its one store commits last and
/// carries the ownership witness — and it reads the source at the invocation
/// index instead of at the coordinate the index operand holds. Every address it
/// computes is inside a source of 12,884,901,888 elements, so no bounds
/// argument separates it from the right body. The refinement gate does, because
/// it re-derives the canonical body and compares.
///
/// **Watched failing by perturbing the fixture's own subject**: setting
/// `GATHERED_EXTENT` to `4` — an extent no closed argument can discharge over
/// an inhabited domain — reddens this at
/// `the gathered extent contains every U32 value, so the obligation is proved`,
/// which is the index layer's deriver returning no proof at all. That is the
/// separation this lane rests on and does not weaken: an undischarged gather
/// has no retained proof, a region without one cannot verify, and this layer
/// only ever sees a `VerifiedScheduledRegion`.
#[test]
fn a_producer_body_that_reads_the_source_directly_is_not_a_refinement() {
    let occurrence = leading_axis();
    let region = gather_region(&occurrence);
    let invocations = element_count(&occurrence.result()).expect("a bounded result");
    let mut builder = KernelBuilder::new(&region).expect("a builder opens against the region");
    let mut declare = |element_type, elements, access| {
        builder
            .declare_buffer(BufferParameter {
                tensor: if matches!(access, BufferAccess::Write) {
                    TensorRole::Output
                } else {
                    TensorRole::Input
                },
                component_role: None,
                element_type,
                address_space: AddressSpace::Device,
                access,
                element_count: elements,
            })
            .expect("a buffer is declared")
    };
    // The signature the region states, so the body below is the only thing
    // wrong: an operand declared at the wrong type would be refused by
    // `BufferContract` and would prove nothing about the addressing.
    let source = declare(
        KernelType::F32,
        element_count(&occurrence.source).unwrap(),
        BufferAccess::Read,
    );
    let _operand = declare(
        KernelType::U32,
        element_count(&occurrence.index).unwrap(),
        BufferAccess::Read,
    );
    let write = declare(KernelType::F32, invocations, BufferAccess::Write);
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .expect("the launch builtin is admitted");
    builder
        .numerical(numerical())
        .expect("the realization is admitted");
    builder
        .requirements(region.requirements())
        .expect("the requirements are admitted");
    let (invocation, active) = guard(&mut builder, invocations);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(source, invocation, BoundsWitnessId::new(0))?;
            builder.store(
                write,
                invocation,
                loaded,
                BoundsWitnessId::new(2),
                OwnershipWitnessId::new(0),
            )
        })
        .expect("the body is structurally admitted");
    assert_eq!(diagnostics(builder), [KernelDiagnostic::BodyRefinement]);
}
