//! The elementwise expression walk: planning, leaf ordering, and minting.
//!
//! One validated plan is produced by a single classifier and then replayed into
//! whichever per-point vocabulary the program's arithmetic names, because two
//! callers number their leaves differently and a second walk written for the
//! second numbering would be free to disagree with the first about what the
//! program says. The two shapes this walk produces — a whole-program expression
//! and an epilogue over a staged value — are built here from that one plan.

use super::*;

/// One recognized elementwise expression and the occurrences it covers.
pub(super) struct RecognizedElementwise {
    pub(super) expression: RecognizedPointwise,
    pub(super) members: Vec<SemanticStage>,
    /// One entry per expression input leaf, in access order.
    ///
    /// Parallel to the region's reads: leaf `i` is served by entry `i`, which
    /// names the declared input ordinal it binds and the relation it addresses
    /// that tensor with. An ordinal appears twice when one declared input is
    /// read both densely and through a relation.
    pub(super) reads: Vec<(DeclaredInputOrdinal, LogicalAccess)>,
}

/// Which values one elementwise walk *reads* rather than computes.
///
/// **The leaf set and the leaf *order* are separate facts, and separating them
/// is what makes an epilogue expressible.** A whole-program or prologue walk
/// reads exactly the declared program inputs and numbers its expression leaves
/// by declaration position, because its region binds one buffer per declared
/// input in that order. An epilogue additionally reads the value an earlier
/// region staged, reads only *some* of the declared inputs, and numbers its
/// leaves by the position of the read that serves them — which is not the
/// declaration ordinal. [`plan_elementwise`] decides the set and the validation;
/// [`mint_elementwise`] is handed the order.
pub(super) struct ElementwiseLeaves<'a> {
    /// The program's declared input values, in declaration order.
    pub(super) declared: &'a [ValueId],
    /// The producer result an epilogue reads as a materialized value.
    ///
    /// `None` for every walk that reads only declared inputs, which keeps the
    /// classification below one rule rather than two.
    pub(super) staged: Option<ValueId>,
}

impl ElementwiseLeaves<'_> {
    /// Returns whether one value is read rather than computed by this walk.
    pub(super) fn is_leaf(&self, value: ValueId) -> bool {
        self.staged == Some(value) || self.declared.contains(&value)
    }
}

/// One tensor a walk reads, and the relation it addresses it with.
///
/// **The relation is part of the leaf's identity, not an annotation on it.** One
/// expression may name a tensor twice meaning two different things — `a *
/// permute(a)` reads declared input `0` densely *and* through a transposition —
/// and those two leaves need two reads with two relations. Keying leaves by
/// value alone made them one leaf, so the region bound one access for both and
/// `a * permute(a)` compiled as `permute(a) * permute(a)`. Two reads of one
/// tensor under the *same* relation address identically and stay one leaf, which
/// is what keeps `(a * a) + b` one read of `a`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LeafRead {
    pub(super) value: ValueId,
    pub(super) map: LogicalAccess,
}

/// What one step of a planned elementwise walk mints.
///
/// The steps are in mint order, so a node's operands are always already minted
/// when it is reached — which is the property that lets [`mint_elementwise`]
/// replay a plan against any leaf ordering without re-deciding anything.
pub(super) enum ElementwiseMint {
    /// A read of one leaf tensor value under one relation.
    ///
    /// The leaf's value is the step's own for a direct read, and the structural
    /// occurrence's *operand* when one interposed: a structural occurrence
    /// computes nothing, so the value it produces is minted as the leaf that
    /// reads the tensor behind it under the relation the family denotes.
    Read { leaf: LeafRead },
    /// An exact `f32` constant leaf.
    Constant(u32),
    /// One node of the recognized vocabulary over already-minted operands.
    Node(ElementwiseFamily, Vec<ValueId>),
}

/// Why one elementwise walk did not complete.
///
/// **The second variant is the epilogue's discovery, and it is a variant rather
/// than a rule code because the caller acts on it.** A walk that reaches a value
/// produced by a folding family has not found an unrecognizable program; it has
/// found the *boundary* between two regions, and the value it names is the one a
/// cover materializes. Reporting it as `operation-set` — which is what a caller
/// that only wants a whole-program expression still does — would throw away the
/// one fact the epilogue recognizer needs.
pub(super) enum ElementwiseRefusal {
    /// The typed refusal to report.
    Refused(RequestError),
    /// The walk reached a value a folding family produces.
    ///
    /// Raised only for a walk that has no staged value yet: a walk that already
    /// reads one and reaches a second has nothing to attribute the second read
    /// to, so it is refused rather than reported as another boundary. That is a
    /// bound on how many edges reach *one* region and not on how deep the chain
    /// is; [`StagedOperandAdmission`] states the depth rule and separates the
    /// two.
    Folded(ValueId),
}

impl From<ElementwiseRefusal> for RequestError {
    /// Flattens a discovered materialization boundary into the rule a caller
    /// with no epilogue to build reports for it.
    ///
    /// **This is where a fold's chained prologue is refused, and it is a third
    /// wall rather than either of the two above.** [`recognize_reduction`]'s
    /// contributor walk is the only caller that reaches it with a `Folded`
    /// finding, and it discards the finding because [`NormalizedSerialSum`]
    /// carries no producer field to hang the boundary on — so `sum(sum(x) * 2.0)`
    /// reports `reduction-contributor-materialization` here rather than reaching
    /// [`StagedOperandAdmission`]'s guard, which never runs for it.
    /// [`name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`](../../../tickets/name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set.md)
    /// owns the rule name.
    fn from(refusal: ElementwiseRefusal) -> Self {
        match refusal {
            ElementwiseRefusal::Refused(error) => error,
            ElementwiseRefusal::Folded(_) => Self::UnsupportedCapability {
                phase: "strategy",
                rule: "reduction-contributor-materialization",
            },
        }
    }
}

/// A validated elementwise expression, linearized in mint order.
///
/// **Every rule the recognizer states is discharged here**, so the only thing
/// left for minting is the arithmetic of node identifiers. That split exists
/// because two callers need the same validation under different leaf numbering,
/// and a second walk written for the second numbering would be a second
/// classifier that could drift from this one.
pub(super) struct ElementwisePlan {
    /// Each value the walk mints, in mint order.
    pub(super) steps: Vec<(ValueId, ElementwiseMint)>,
    /// The distinct leaf reads, in first-mint order.
    pub(super) leaves: Vec<LeafRead>,
    pub(super) members: Vec<SemanticStage>,
    pub(super) root: ValueId,
}

/// The elementwise operation families this recognizer projects.
///
/// Exactly the families whose per-point body the physical expression vocabulary
/// can express. Two are single nodes of that vocabulary; the third is a
/// composition, and the distinction between "one node" and "expressible" is
/// where this set used to stop.
///
/// **`tiler::silu-f32@1` is projected rather than restated, and the difference
/// is the whole reason it is admissible here.** No `PointwiseF32Node` spells a
/// sigmoid-weighted linear unit, so the projection is a subtree — but the
/// subtree is not written in this module. [`crate::elementary::silu_point_body`]
/// is the one statement of the composition in this crate, and the governed
/// index-access lowering emits the *same* function into the scalar vocabulary
/// its regions are built from. So the boundary is not re-deriving a provider's
/// lowering; both realizations are driven from one authority, and occurrence
/// refinement independently proves that the resolved provider's emitted region
/// realizes the occurrence.
#[derive(Clone, Copy)]
pub(super) enum ElementwiseFamily {
    Add,
    Multiply,
    /// The activation, projected through [`crate::elementary::silu_point_body`].
    Silu,
}

impl ElementwiseFamily {
    /// The operand count this family's occurrences declare.
    ///
    /// Read from the family rather than from the occurrence, so an occurrence
    /// whose arity disagrees with its registered family is refused under
    /// `elementwise-arity` instead of being projected against whichever operands
    /// happened to be present.
    const fn operand_count(self) -> usize {
        match self {
            Self::Add | Self::Multiply => 2,
            Self::Silu => 1,
        }
    }
}

/// Classifies one operation as a recognized elementwise family, or declines.
///
/// **Keyed by the program's arithmetic rather than by trying both vocabularies.**
/// A family's key already names its width, so the two lists are disjoint and a
/// union would classify the same operations; keying on the arithmetic the caller
/// derived is what keeps a `bf16` program from ever being offered an `f32`
/// projection to fail on later. The exhaustive match is what makes a third
/// admitted width a build error here rather than a program silently declining
/// every family.
///
/// The `bf16` row is deliberately shorter. There is no `tiler::silu-bf16@1`
/// registered to classify, and [`PointwiseBf16Node`] has no division or
/// exponential for a projection to land in, so the activation is absent because
/// the vocabulary cannot state it rather than because this list forgot it.
///
/// [`PointwiseBf16Node`]: tiler_ir::schedule::PointwiseBf16Node
fn elementwise_family(
    operation: &tiler_ir::semantic::OperationRef<'_>,
    arithmetic: ArithmeticType,
) -> Option<ElementwiseFamily> {
    match arithmetic {
        ArithmeticType::F32 => {
            if operation.key() == &add_f32_op() {
                Some(ElementwiseFamily::Add)
            } else if operation.key() == &multiply_f32_op() {
                Some(ElementwiseFamily::Multiply)
            } else if operation.key() == &silu_f32_op() {
                Some(ElementwiseFamily::Silu)
            } else {
                None
            }
        }
        ArithmeticType::Bf16 => {
            if operation.key() == &add_bf16_op() {
                Some(ElementwiseFamily::Add)
            } else if operation.key() == &multiply_bf16_op() {
                Some(ElementwiseFamily::Multiply)
            } else {
                None
            }
        }
        // No program of either width reaches this function:
        // [`recognized_program_arithmetic`] refuses every value type that is not
        // one of the two above under `dtype-recognized`. Declining is the
        // fail-closed answer rather than a wildcard that would silently offer one
        // width's families to another.
        ArithmeticType::F16 | ArithmeticType::F64 => None,
    }
}

/// The nullary constant family of one recognized arithmetic type.
///
/// `None` for a width this recognizer states no constant family for, which is
/// the same fail-closed answer [`elementwise_family`] gives and for the same
/// reason.
pub(super) fn constant_family(arithmetic: ArithmeticType) -> Option<OpKey> {
    match arithmetic {
        ArithmeticType::F32 => Some(constant_f32_op()),
        ArithmeticType::Bf16 => Some(constant_bf16_op()),
        ArithmeticType::F16 | ArithmeticType::F64 => None,
    }
}

/// Recognizes the elementwise expression rooted at one value.
///
/// **General over the graph rather than over a taught shape.** Each operand is
/// classified independently — a declared input tensor becomes the leaf that
/// reads it, a `tiler.constant-f32` occurrence becomes an exact constant leaf,
/// and a recognized elementwise occurrence is walked in turn — so depth, arity,
/// family mixing, and shared subexpressions are properties of the caller's
/// program rather than of a template. Two operands naming one value share the
/// node already minted, which is what makes `(a * a) + b` one read of `a`.
///
/// `shape` is the region's iteration domain, and every tensor read must carry
/// it: the region binds one linear-identity access per read, so an operand at a
/// different shape would be sized by a domain it does not have. A constant is
/// rank-zero and is a literal node rather than a read, so it is deliberately not
/// held to it.
///
/// The walk is iterative over an explicit worklist rather than recursive. The
/// depth is the caller's own longest elementwise chain, and a recognizer that
/// consumed host stack proportional to it would turn an input property into a
/// crash rather than a refusal.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the exact property
/// that was not recognized: `operation-set` for a family the expression
/// vocabulary cannot spell, `elementwise-shape` for a read at another domain,
/// `elementwise-attributes` for an attribute this projection would drop,
/// `elementwise-arity` for an operand count the vocabulary has no node for, and
/// every rule [`resolve_elementwise`] reports for the numbering that follows —
/// which is where `elementwise-expression` comes from, along with
/// `elementwise-reads`, `input-ordinal`, `elementwise-operand`, and
/// `elementwise-node-limit`.
pub(super) fn recognize_elementwise(
    program: &SemanticProgram,
    root: ValueId,
    declared: &[ValueId],
    shape: &Shape,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<RecognizedElementwise, RequestError> {
    let sourced = SourcedShape::from(shape.clone());
    let plan = plan_elementwise(
        program,
        root,
        &ElementwiseLeaves {
            declared,
            staged: None,
        },
        &sourced,
        laws,
        arithmetic,
    )
    .map_err(RequestError::from)?;
    resolve_elementwise(plan, declared, arithmetic)
}

/// Resolves one planned whole-program or prologue expression against the
/// declared inputs.
///
/// Declaration order is the *group* order here: the region's reads walk the
/// declared inputs in the order the ABI binds them. It is not a one-to-one
/// correspondence with the leaves in either direction — one declared input may
/// be read twice, and one this walk does not reach is read not at all, so the
/// list is a *map* from the expression's dense leaf ordinals to the program's
/// input ordinals. [`canonical_input_reads`] states both orders.
///
/// An epilogue additionally reads a staged value, and [`recognize_epilogue`]
/// states its own order rather than relaxing this one.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads` for
/// a leaf that is not a declared input, `input-ordinal` for a declaration
/// position no expression ordinal can hold, and every rule
/// [`mint_elementwise`] reports.
fn resolve_elementwise(
    plan: ElementwisePlan,
    declared: &[ValueId],
    arithmetic: ArithmeticType,
) -> Result<RecognizedElementwise, RequestError> {
    let order = canonical_input_reads(&plan.leaves, declared)?;
    let expression = mint_elementwise(&plan, &order, arithmetic)?;
    let reads = order
        .iter()
        .map(|leaf| Ok((declared_ordinal(declared, leaf.value)?, leaf.map.clone())))
        .collect::<Result<Vec<_>, RequestError>>()?;
    Ok(RecognizedElementwise {
        expression,
        members: plan.members,
        reads,
    })
}

/// Orders one walk's leaf reads into the read list a whole-program or prologue
/// region binds.
///
/// **Declared inputs in declaration order, and each input's reads dense-first.**
/// The group order is the ABI's, and the order *within* a group is this
/// compiler normalization's canonical spelling. It has to be decided here
/// rather than left to the walk: the two
/// reads of `a` in `a * permute(a)` are popped in whichever order the operands
/// happened to be visited, and a read list in walk order would give one program
/// two spellings. The fieldless region verifier checks boundary categories, not
/// declared-input grouping, so it cannot supply this order later.
///
/// **A declared input this walk never reads contributes no read, and the
/// ordinals stay the program's.** An output whose expression names two of three
/// declared inputs binds those two, carrying the ordinals the program declared
/// them at rather than a region-local renumbering. The physical region itself
/// carries only the ordered accesses; program assembly projects each exact
/// [`AccessOrdinal`] through the retained checked request subject to recover
/// these declared ordinals. This walk therefore skips an unread group instead
/// of refusing it: the obligation
/// that no declared input goes unread by *every* output is a program-scoped
/// property and lives in [`check_output_cover`](super::recognize::check_output_cover), where the other program-scoped
/// obligations moved when several ordered outputs became statable.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads`
/// when some leaf reads a value that is not a declared input. That is
/// unreachable for these walks, whose leaf set is the declared inputs by
/// construction, and is refused rather than assumed away.
pub(super) fn canonical_input_reads(
    leaves: &[LeafRead],
    declared: &[ValueId],
) -> Result<Vec<LeafRead>, RequestError> {
    let mut order: Vec<LeafRead> = Vec::with_capacity(leaves.len());
    for input in declared {
        for dense in [true, false] {
            order.extend(
                leaves
                    .iter()
                    .filter(|leaf| {
                        leaf.value == *input && (leaf.map == LogicalAccess::LinearIdentity) == dense
                    })
                    .cloned(),
            );
        }
    }
    if order.len() != leaves.len() {
        return mismatch("elementwise-reads");
    }
    Ok(order)
}

/// Returns the expression input ordinal one declared input value occupies.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads` for
/// a value the declaration does not name and `input-ordinal` for a declaration
/// position no expression ordinal can hold.
pub(super) fn declared_ordinal(
    declared: &[ValueId],
    value: ValueId,
) -> Result<DeclaredInputOrdinal, RequestError> {
    let position = declared.iter().position(|input| *input == value).ok_or(
        RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "elementwise-reads",
        },
    )?;
    u32::try_from(position)
        .map(DeclaredInputOrdinal::new)
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "input-ordinal",
        })
}

/// Records one leaf read at its first sighting, or refuses a second read of one
/// tensor that nothing could attribute.
///
/// **Two reads of one tensor are admitted exactly when the compiler can tell
/// them apart and order them.** A dense read and a mapped read of one declared
/// input are two different tensors as far as the expression is concerned, and
/// [`canonical_input_reads`] binds the pair in one canonical order — dense first
/// — before exact access positions carry that spelling into the region. The
/// intrinsic schedule verifier sees only their fieldless boundary categories.
///
/// The two refusals are that admission's own boundary rather than separate
/// rules. A *staged* value read twice would need two `TensorRole::Intermediate`
/// accesses, and that role carries no ordinal, so nothing says which
/// materialization edge each binds — the very attribution that makes the input
/// pair unambiguous is what it lacks. Two *structural* relations on one tensor
/// have no canonical order between them, so the pair would have two encodings
/// and the region two identities.
pub(super) fn record_leaf(
    leaves: &mut Vec<LeafRead>,
    staged: Option<ValueId>,
    read: LeafRead,
) -> Result<(), ElementwiseRefusal> {
    if leaves.contains(&read) {
        return Ok(());
    }
    let already_read = |mapped_only: bool| {
        leaves.iter().any(|leaf| {
            leaf.value == read.value && (!mapped_only || leaf.map != LogicalAccess::LinearIdentity)
        })
    };
    let unattributable = staged == Some(read.value) && already_read(false);
    let unordered = read.map != LogicalAccess::LinearIdentity && already_read(true);
    if unattributable || unordered {
        return refused("structural-access-conflict");
    }
    leaves.push(read);
    Ok(())
}

/// Validates and linearizes the elementwise expression rooted at one value.
///
/// This is the whole of the recognition stated in [`recognize_elementwise`]'s
/// documentation; what it deliberately does not do is choose expression input
/// ordinals, because two callers number their leaves differently and a walk that
/// decided the numbering would have to be written twice.
///
/// # Errors
///
/// Returns every [`RequestError::UnsupportedCapability`]
/// [`recognize_elementwise`] documents except `elementwise-reads`,
/// `elementwise-node-limit`, and `elementwise-expression`, which are properties
/// of a *numbering* and are reported by [`mint_elementwise`], each wrapped in
/// [`ElementwiseRefusal::Refused`] — or [`ElementwiseRefusal::Folded`] naming
/// the value a folding family produced.
pub(super) fn plan_elementwise(
    program: &SemanticProgram,
    root: ValueId,
    leaves: &ElementwiseLeaves<'_>,
    shape: &SourcedShape,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<ElementwisePlan, ElementwiseRefusal> {
    let mut steps: Vec<(ValueId, ElementwiseMint)> = Vec::new();
    let mut minted: Vec<ValueId> = Vec::new();
    let mut members: Vec<SemanticStage> = Vec::new();
    let mut leaf_reads: Vec<LeafRead> = Vec::new();
    let mut pending = vec![(root, false)];
    while let Some((value, operands_visited)) = pending.pop() {
        if minted.contains(&value) {
            continue;
        }
        if leaves.is_leaf(value) {
            if sourced_shape_ref(program, value) != Some(shape) {
                return refused("elementwise-shape");
            }
            let leaf = LeafRead {
                value,
                map: LogicalAccess::LinearIdentity,
            };
            record_leaf(&mut leaf_reads, leaves.staged, leaf.clone())?;
            steps.push((value, ElementwiseMint::Read { leaf }));
            minted.push(value);
            continue;
        }
        let (member, operation) =
            producer_for_value(program, value).map_err(ElementwiseRefusal::Refused)?;
        if operation.results().collect::<Vec<_>>() != [value] {
            return refused("elementwise-result-arity");
        }
        if constant_family(arithmetic).is_some_and(|constant| operation.key() == &constant) {
            let (bits, _) =
                constant_bits(program, value, arithmetic).map_err(ElementwiseRefusal::Refused)?;
            members.push(SemanticStage::first(SemanticMemberId(member)));
            steps.push((value, ElementwiseMint::Constant(bits)));
            minted.push(value);
            continue;
        }
        // A structural occurrence contributes an *access relation*, not a node.
        // It computes nothing — the value it produces is the value it read — so
        // it becomes the leaf that reads its operand, carrying the coordinate
        // map the family denotes. That is what makes a fused region the
        // deliverable rather than a materializing copy kernel: the arithmetic
        // still comes from the neighbour, and only the addressing changes.
        //
        // A sourced broadcast is recognized as the parametric carrier over the
        // authored domain. Reindex still needs a static domain for its axis
        // decodes; that refusal is inside [`recognize_structural_read`].
        if is_structural_family(operation.key())
            && let Some((operand, map)) =
                recognize_structural_read(program, &operation, leaves, shape)
                    .map_err(ElementwiseRefusal::Refused)?
        {
            let leaf = LeafRead {
                value: operand,
                map,
            };
            record_leaf(&mut leaf_reads, leaves.staged, leaf.clone())?;
            members.push(SemanticStage::first(SemanticMemberId(member)));
            steps.push((value, ElementwiseMint::Read { leaf }));
            minted.push(value);
            continue;
        }
        let Some(family) = elementwise_family(&operation, arithmetic) else {
            // A folding family is the *boundary* between two regions rather than
            // an unrecognizable operation: no `PointwiseF32Node` spells a sum
            // over a contributor sequence, and none ever will, because the
            // expression is a per-point body. Naming the value lets the epilogue
            // recognizer read it as the tensor an earlier region staged.
            //
            // **A walk that already reads one staged value reports the ordinary
            // rule instead, and that is a rule about chain *width* rather than
            // depth.** Naming a second, *different* folded value —
            // `sum(a, 1) * sum(b, 1)` — would give this one region two
            // `TensorRole::Intermediate` reads, and that role carries no ordinal,
            // so nothing would say which edge each access binds. The walk is
            // still one materialization boundary deep. It is the same
            // unordinalled-role fact `record_leaf` refuses for one staged value
            // read *twice*, and its region-vocabulary owner is
            // `admit-a-scheduled-region-that-reads-two-materialization-edges`.
            // The depth rule is `StagedOperandAdmission`'s, which states where it
            // sits and what it is not.
            if leaves.staged.is_none() && materializes_its_result(&operation, laws) {
                return Err(ElementwiseRefusal::Folded(value));
            }
            return refused("operation-set");
        };
        // A recognized elementwise operation of this profile is attribute-free.
        // An attribute is a semantic fact the expression does not carry forward,
        // so admitting one would silently drop it.
        if !operation.attributes().fields().is_empty() {
            return refused("elementwise-attributes");
        }
        // The region's domain, or rank zero. The second arm is not a relaxation:
        // the expression's nodes are *per-point values*, so a subexpression over
        // constants alone — which the semantic inferencer types as rank zero —
        // is evaluated exactly like a constant leaf and reads no tensor. It is
        // also reachable rather than defensive: reassociating `(a * 2.0) * 3.0`
        // into `a * (2.0 * 3.0)` is an alternative the algebraic exploration
        // proposes under a contract that permits it, and its inner product is
        // rank zero. Refusing every rank would have lost that alternative to a
        // check with no correctness content behind it.
        let value_shape = sourced_shape_ref(program, value);
        if value_shape != Some(shape) && value_shape.map(SourcedShape::rank) != Some(0) {
            return refused("elementwise-shape");
        }
        let operands: Vec<ValueId> = operation.operands().collect();
        if operands.len() != family.operand_count() {
            return refused("elementwise-arity");
        }
        if !operands_visited {
            pending.push((value, true));
            // Pushed in reverse so the first operand is popped first, which is
            // what keeps a deterministic walk order across arities.
            for operand in operands.iter().rev() {
                pending.push((*operand, false));
            }
            continue;
        }
        // Every operand is already minted, so the node's own step records the
        // operand *values* and the numbering is left to the mint pass.
        if !operands.iter().all(|operand| minted.contains(operand)) {
            return refused("elementwise-operand");
        }
        members.push(SemanticStage::first(SemanticMemberId(member)));
        steps.push((value, ElementwiseMint::Node(family, operands)));
        minted.push(value);
    }
    members.sort_unstable();
    members.dedup();
    Ok(ElementwisePlan {
        steps,
        leaves: leaf_reads,
        members,
        root,
    })
}

/// One per-point expression vocabulary a planned walk can be minted into.
///
/// **One walk, one mint loop, two vocabularies.** The plan
/// [`plan_elementwise`] produces is arithmetic-neutral — it is a linearized run
/// of reads, constants, and classified families — and the only thing that
/// differs between the two widths is which builder the run is replayed against.
/// Writing the replay twice would be two accounts of one numbering, free to
/// disagree about which leaf serves which read, which is the drift a single
/// authority exists to prevent.
///
/// The error is a rule name rather than a unit, because the two vocabularies
/// refuse for different reasons and a shared `()` would report a node-count bound
/// for a family the width has no node for at all.
pub(super) trait PointwiseMintSink {
    /// The sink's handle to one minted per-point value.
    type Value: Clone;
    /// The verified expression this sink builds.
    type Expression;

    /// Mints a read of the expression input at one dense leaf ordinal.
    fn input(&mut self, ordinal: AccessOrdinal) -> Result<Self::Value, &'static str>;
    /// Mints an exact constant leaf from its canonical bit pattern.
    fn constant(&mut self, bits: u32) -> Result<Self::Value, &'static str>;
    /// Mints one ordered addition.
    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Result<Self::Value, &'static str>;
    /// Mints one ordered multiplication.
    fn multiply(&mut self, lhs: Self::Value, rhs: Self::Value)
    -> Result<Self::Value, &'static str>;
    /// Mints the sigmoid-weighted linear unit's projected body.
    fn silu(&mut self, argument: &Self::Value) -> Result<Self::Value, &'static str>;
    /// Builds the verified expression rooted at one minted value.
    fn build(self, root: Self::Value) -> Result<Self::Expression, &'static str>;
}

/// The `f32` per-point vocabulary.
struct F32Mint(PointwiseF32ExpressionBuilder);

impl PointwiseMintSink for F32Mint {
    type Value = PointwiseF32Value;
    type Expression = PointwiseF32Expression;

    fn input(&mut self, ordinal: AccessOrdinal) -> Result<Self::Value, &'static str> {
        self.0.input(ordinal).map_err(|_| "elementwise-node-limit")
    }

    fn constant(&mut self, bits: u32) -> Result<Self::Value, &'static str> {
        self.0.constant(bits).map_err(|_| "elementwise-node-limit")
    }

    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Result<Self::Value, &'static str> {
        self.0.add(lhs, rhs).map_err(|_| "elementwise-node-limit")
    }

    fn multiply(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
    ) -> Result<Self::Value, &'static str> {
        self.0
            .multiply(lhs, rhs)
            .map_err(|_| "elementwise-node-limit")
    }

    // The composition is emitted by the shared authority rather than spelled
    // here; see [`ElementwiseFamily::Silu`].
    fn silu(&mut self, argument: &Self::Value) -> Result<Self::Value, &'static str> {
        let mut sink = PointwiseExpressionSink::new(&mut self.0);
        silu_point_body(&mut sink, argument).map_err(|_| "elementwise-node-limit")
    }

    fn build(self, root: Self::Value) -> Result<Self::Expression, &'static str> {
        self.0.build(root).map_err(|_| "elementwise-expression")
    }
}

/// The `bf16` per-point vocabulary.
pub(super) struct Bf16Mint(PointwiseBf16ExpressionBuilder);

impl PointwiseMintSink for Bf16Mint {
    type Value = PointwiseBf16Value;
    type Expression = PointwiseBf16Expression;

    fn input(&mut self, ordinal: AccessOrdinal) -> Result<Self::Value, &'static str> {
        self.0.input(ordinal).map_err(|_| "elementwise-node-limit")
    }

    /// The payload is narrowed rather than truncated.
    ///
    /// [`constant_bits`] reads a `bf16` constant's exactly two declared payload
    /// bytes, so every value reaching here fits; a wider one is a mismatch
    /// between the two and is refused by name instead of silently losing the
    /// upper half of a pattern that would then be a different number.
    fn constant(&mut self, bits: u32) -> Result<Self::Value, &'static str> {
        let bits = u16::try_from(bits).map_err(|_| "constant-bits")?;
        self.0.constant(bits).map_err(|_| "elementwise-node-limit")
    }

    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Result<Self::Value, &'static str> {
        self.0.add(lhs, rhs).map_err(|_| "elementwise-node-limit")
    }

    fn multiply(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
    ) -> Result<Self::Value, &'static str> {
        self.0
            .multiply(lhs, rhs)
            .map_err(|_| "elementwise-node-limit")
    }

    /// Unreachable, and refused by its own name rather than by a bound it did
    /// not exceed: [`elementwise_family`] classifies no activation for this
    /// width, because no `bf16` activation family is registered and the `bf16`
    /// node vocabulary has neither the division nor the exponential its body
    /// composes.
    fn silu(&mut self, _argument: &Self::Value) -> Result<Self::Value, &'static str> {
        Err("elementwise-family-arithmetic")
    }

    fn build(self, root: Self::Value) -> Result<Self::Expression, &'static str> {
        self.0.build(root).map_err(|_| "elementwise-expression")
    }
}

/// Replays one planned walk into one per-point vocabulary.
///
/// `order` is the read list the region will bind, in access order: leaf `i` of
/// the built expression is served by read `i`, which is the correspondence
/// `emit_pointwise` relies on. Intrinsic region verification checks that this
/// same ordered access has a permissible fieldless boundary category; the
/// compiler's retained request subject supplies any declared-input association.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads` for
/// a leaf the order does not name, `input-ordinal` for a position no expression
/// ordinal can hold, `elementwise-operand` for an operand no earlier step of the
/// plan minted, `elementwise-arity` for a family and operand count this
/// projection has no node for, and every rule the sink itself reports —
/// `elementwise-node-limit` for an expression exceeding its vocabulary's node
/// bound, `elementwise-expression` for an assembled expression no region can
/// bind, and the sink's own refusal for a family its width cannot state.
fn mint_into<S: PointwiseMintSink>(
    plan: &ElementwisePlan,
    order: &[LeafRead],
    mut sink: S,
) -> Result<S::Expression, RequestError> {
    let mut minted: Vec<(ValueId, S::Value)> = Vec::new();
    for (value, mint) in &plan.steps {
        let node = match mint {
            ElementwiseMint::Read { leaf } => {
                let position = order.iter().position(|named| named == leaf).ok_or(
                    RequestError::UnsupportedCapability {
                        phase: "strategy",
                        rule: "elementwise-reads",
                    },
                )?;
                let ordinal =
                    u32::try_from(position).map_err(|_| RequestError::UnsupportedCapability {
                        phase: "strategy",
                        rule: "input-ordinal",
                    })?;
                sink.input(AccessOrdinal::new(ordinal))
            }
            ElementwiseMint::Constant(bits) => sink.constant(*bits),
            ElementwiseMint::Node(family, operands) => {
                let projected: Vec<S::Value> = operands
                    .iter()
                    .map(|operand| minted_value(&minted, *operand))
                    .collect::<Result<_, _>>()?;
                match (family, projected.as_slice()) {
                    (ElementwiseFamily::Add, [lhs, rhs]) => sink.add(lhs.clone(), rhs.clone()),
                    (ElementwiseFamily::Multiply, [lhs, rhs]) => {
                        sink.multiply(lhs.clone(), rhs.clone())
                    }
                    (ElementwiseFamily::Silu, [argument]) => sink.silu(argument),
                    // Unreachable through the planner's arity check, and refused
                    // rather than assumed away: an arity this projection has no
                    // case for is a vocabulary gap, not a node to invent.
                    _ => return mismatch("elementwise-arity"),
                }
            }
        }
        .map_err(mismatch_rule)?;
        minted.push((*value, node));
    }
    let root = minted_value(&minted, plan.root)?;
    sink.build(root).map_err(mismatch_rule)
}

/// Mints one planned elementwise expression in the program's own arithmetic.
///
/// # Errors
///
/// Returns every rule [`mint_into`] reports, and `dtype-recognized` for an
/// arithmetic type this recognizer states no per-point vocabulary for — the same
/// rule [`recognized_program_arithmetic`] refuses that width under, because it is
/// the same finding reached from the other end.
pub(super) fn mint_elementwise(
    plan: &ElementwisePlan,
    order: &[LeafRead],
    arithmetic: ArithmeticType,
) -> Result<RecognizedPointwise, RequestError> {
    match arithmetic {
        ArithmeticType::F32 => {
            mint_into(plan, order, F32Mint(PointwiseF32ExpressionBuilder::new()))
                .map(RecognizedPointwise::F32)
        }
        ArithmeticType::Bf16 => {
            mint_into(plan, order, Bf16Mint(PointwiseBf16ExpressionBuilder::new()))
                .map(RecognizedPointwise::Bf16)
        }
        ArithmeticType::F16 | ArithmeticType::F64 => mismatch("dtype-recognized"),
    }
}

/// Mints one planned expression into the `f32` vocabulary specifically.
///
/// The fold's prologue and the elementwise epilogue call this rather than
/// [`mint_elementwise`], because both shapes are reachable only for an `f32`
/// program: each is entered from a folding family, and the three families that
/// discover one — the strict serial sum, the strict tensor contraction, and any
/// registered family whose realization law spans a region sequence — are `f32`
/// throughout. Asking for the `f32` vocabulary directly is what keeps
/// [`NormalizedSerialSum::prologue`] and [`NormalizedEpilogue::expression`] typed
/// as the one vocabulary they can hold, instead of carrying a width neither
/// shape can reach.
fn mint_elementwise_f32(
    plan: &ElementwisePlan,
    order: &[LeafRead],
) -> Result<PointwiseF32Expression, RequestError> {
    mint_into(plan, order, F32Mint(PointwiseF32ExpressionBuilder::new()))
}

/// Wraps one sink's rule name in the recognizer's typed refusal.
const fn mismatch_rule(rule: &'static str) -> RequestError {
    RequestError::UnsupportedCapability {
        phase: "strategy",
        rule,
    }
}

/// Returns the expression node already minted for one recognized value.
///
/// Generic over the sink's handle rather than over one vocabulary's value type,
/// because the lookup is by planned `ValueId` and says nothing about what the
/// handle denotes; a second copy per width would be one rule stated twice.
fn minted_value<V: Clone>(minted: &[(ValueId, V)], value: ValueId) -> Result<V, RequestError> {
    minted
        .iter()
        .find(|(seen, _)| *seen == value)
        .map(|(_, node)| node.clone())
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "elementwise-operand",
        })
}

/// Recognizes a whole-program elementwise expression.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the unrecognized
/// property — every rule [`resolve_elementwise`] reports, the planning half
/// having already run in the caller — or [`RequestError::ShapeProductOverflow`]
/// under `input` for a domain whose extents do not multiply into a `u64`. The
/// rank and occurrence-coverage obligations are the caller's:
/// [`recognize_elementwise_output`](super::recognize::recognize_elementwise_output) reports `elementwise-rank` and
/// [`check_output_cover`](super::recognize::check_output_cover) reports `operation-set`.
pub(super) fn recognize_pointwise(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    declared: &[ValueId],
    shape: SourcedShape,
    plan: ElementwisePlan,
    arithmetic: ArithmeticType,
) -> Result<NormalizedPointwise, RequestError> {
    let recognized = resolve_elementwise(plan, declared, arithmetic)?;
    let elements = match shape.as_static() {
        Some(static_shape) => element_count_u64(static_shape, "input")?,
        None => 0,
    };
    Ok(NormalizedPointwise {
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key: output.key().clone(),
        shape,
        expression: recognized.expression,
        members: recognized.members,
        inputs: declared.to_vec(),
        output: output.value(),
        elements,
        reads: recognized.reads,
    })
}

/// Recognizes an elementwise epilogue over one staged producer result.
///
/// **The read order is compiler normalization rather than the order the walk
/// minted leaves in, and that is a correctness requirement rather than
/// tidiness.** A read list in walk order would give `staged * (b + a)` and
/// `staged * (a + b)` different spellings solely because their operands were
/// popped in a different order. The staged read leads because exactly one read
/// binds it and it carries no declared association to interleave with;
/// [`canonical_input_reads`]'s rule supplies the rest — declaration groups in
/// order and each group's distinguishable reads dense-first. Intrinsic schedule
/// verification sees only the resulting local access positions and fieldless
/// boundary categories.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the property that was
/// not recognized: every rule [`plan_elementwise`], [`mint_elementwise`], and
/// [`declared_ordinal`] report for the epilogue's own walk, and every rule
/// [`recognize_epilogue_producer`] reports for the staged half — `operation-set`
/// for a folded family it has no producer recognizer for,
/// [`producer_for_value`]'s `missing-producer` and `operation-ordinal`, and the
/// producing family's own rules. Returns [`RequestError::ShapeProductOverflow`]
/// under `output` for a domain whose extents do not multiply into a `u64`.
pub(super) fn recognize_epilogue(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    declared: &[ValueId],
    shape: Shape,
    staged: ValueId,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<NormalizedEpilogue, RequestError> {
    let leaves = ElementwiseLeaves {
        declared,
        staged: Some(staged),
    };
    let sourced = SourcedShape::from(shape.clone());
    let plan = plan_elementwise(program, output.value(), &leaves, &sourced, laws, arithmetic)
        .map_err(RequestError::from)?;
    // The staged read, then whichever declared inputs the expression names. The
    // *declared* half is now the same rule `canonical_input_reads` states —
    // groups in declaration order, dense before mapped, an unread input
    // contributing nothing — and what keeps the walk spelled here is the staged
    // read: it leads, it binds no declared input, and that function refuses a
    // leaf which is not one. Reusing it would mean handing it a leaf set it is
    // defined to reject.
    let mut order: Vec<LeafRead> = plan
        .leaves
        .iter()
        .filter(|leaf| leaf.value == staged)
        .cloned()
        .collect();
    for input in declared {
        for dense in [true, false] {
            order.extend(
                plan.leaves
                    .iter()
                    .filter(|leaf| {
                        leaf.value == *input && (leaf.map == LogicalAccess::LinearIdentity) == dense
                    })
                    .cloned(),
            );
        }
    }
    let expression = mint_elementwise_f32(&plan, &order)?;
    let reads = order
        .iter()
        .map(|leaf| {
            let read = if leaf.value == staged {
                BoundaryRead::Staged
            } else {
                BoundaryRead::Input(declared_ordinal(declared, leaf.value)?)
            };
            Ok((read, leaf.map.clone()))
        })
        .collect::<Result<Vec<_>, RequestError>>()?;
    let producer = recognize_epilogue_producer(program, staged, output.key().clone(), laws)?;
    let elements = element_count_u64(&shape, "output")?;
    Ok(NormalizedEpilogue {
        producer: Box::new(producer),
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key: output.key().clone(),
        shape,
        expression,
        reads,
        members: plan.members,
        inputs: declared.to_vec(),
        output: output.value(),
        elements,
    })
}

/// The elementwise planner's spelling of [`mismatch`].
///
/// Separate because the planner's error type additionally carries a discovered
/// materialization boundary, which is a finding rather than a rule.
fn refused<T>(rule: &'static str) -> Result<T, ElementwiseRefusal> {
    Err(ElementwiseRefusal::Refused(
        RequestError::UnsupportedCapability {
            phase: "strategy",
            rule,
        },
    ))
}
