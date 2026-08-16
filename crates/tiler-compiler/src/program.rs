//! Compiler-owned program layers over the shared target-neutral kernel program.
//!
//! The stage DAG, the exact selected scheduled/KIR refinements, the checked
//! materialized values, views, allocations, lifetimes and handoffs, the typed
//! dependencies, the named outputs, complete semantic coverage, the host
//! preflight expression arena, the entry ABI, the applicability guard, and the
//! routing-commit contract all live in [`tiler_ir::program`] (ADR 0070), where
//! they are constructed through the ADR 0071 checked builder and carry a
//! canonical identity folding the ADR 0072 layers.
//!
//! `complete-program-identity-with-abi-guards-and-routing` moved the last four
//! of those down. This module previously held a second copy of each, verified
//! against the shared core; the copies are gone rather than re-checked, because
//! two representations of one ABI that nothing keeps in agreement is the drift
//! ADR 0068 exists to prevent.
//!
//! What remains here is what only a *compilation* can decide: the target
//! binding, the request budgets, the compile-time truth of the applicability
//! guard, the agreement between each stage's declared launch and the scheduled
//! region it was planned from, and the artifact construction plan that binds
//! all of it to one compilation request.

use std::error::Error;
use std::fmt;

use tiler_ir::kernel::KernelType;
use tiler_ir::program::{
    AbiExprId, AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec,
    CoveredOccurrence, KernelProgramBuildError, KernelProgramBuilder, KernelProgramDiagnostic,
    MaterializedOrigin, MaterializedValueSpec, MemorySpace, RoutingCommitState,
    RoutingCommitTransition, StageAccess, StageAccessMode, StageLaunch, StageRef, StorageEncoding,
    StorageScalar, ValueRole, VerifiedKernelProgram, ViewId,
};
use tiler_ir::schedule::ArithmeticType;
use tiler_ir::semantic::{InputKey, OutputKey, SemanticIdentity, SemanticProgram};
use tiler_ir::shape::{Axis, Shape, SourcedExtent};

use tiler_ir::program::abi::{
    AbiBinaryOp, AbiEvaluationError, AbiFacts, AbiRoot, AbiValue, AvailabilityPhase, ExprNode,
    evaluate as abi_evaluate,
};

use crate::cover::{CoverRegion, RegionCover};
use crate::lowering::ResolvedLowering;
use crate::physical::{
    AccessMode, AccessOrdinal, ContributorPartition, NumericalRealization, RegionId, TensorRole,
    VerifiedKernel, VerifiedScheduledRegion, lower_structured_kernel,
};
use crate::region::{RegionOccurrenceIdentity, SemanticStage, SemanticValueId, value_ordinal};
use crate::request::{LoweringProviderIdentity, TargetProfile, VerifiedTargetRequest};
use crate::selection::SelectedPlan;
use crate::target::feasibility::DeferredPredicate;
use crate::target::feasibility::{FeasibilityRuleSetIdentity, GOVERNED_FEASIBILITY_RULE_SET};

/// The physical storage carrier and kernel element type of one program's values.
///
/// **It was the constant `StorageScalar::F32`, and it is now derived from the
/// program's own arithmetic.** Every byte width and alignment below derives from
/// it, which is what the constant already bought — a second carrier had to be
/// found by changing one binding rather than by reading every arithmetic site.
/// What the constant could not express is that the binding is now per *program*:
/// a `bf16` program's values are two bytes wide, and declaring them four wide
/// gave `KernelProgramBuilder::build` a stage whose kernel expected `Bf16` and
/// whose value declared `F32` — the refusal was `StageElementType`, and it is the
/// downstream `f32` assumption the recognizer's own wall had been hiding.
///
/// The two halves travel together because they are one fact: the storage scalar
/// sizes and aligns the buffer, the kernel type is what a stage's accesses are
/// checked against, and a carrier paired with the wrong element type is exactly
/// the disagreement that refusal names.
#[derive(Clone, Copy)]
struct BoundedCarrier {
    storage: StorageScalar,
    element_type: KernelType,
}

impl BoundedCarrier {
    /// The carrier one recognized arithmetic type materializes through.
    ///
    /// `None` for a width this profile materializes no carrier for, which is a
    /// refusal rather than a default: sizing a program's buffers by a width
    /// nobody stated is how a caller's tensor is read past its end.
    const fn of(arithmetic: ArithmeticType) -> Option<Self> {
        match arithmetic {
            ArithmeticType::F32 => Some(Self {
                storage: StorageScalar::F32,
                element_type: KernelType::F32,
            }),
            ArithmeticType::Bf16 => Some(Self {
                storage: StorageScalar::Bf16,
                element_type: KernelType::Bf16,
            }),
            ArithmeticType::F16 | ArithmeticType::F64 => None,
        }
    }

    /// The byte width of one element, unpacked.
    ///
    /// `StorageScalar::byte_width` is the authority; this names the carrier's
    /// choice, not a width of its own.
    fn element_bytes(self) -> u64 {
        self.storage.byte_width()
    }

    /// The natural requirement every value of this carrier states.
    fn element_requirement(self) -> AlignmentRequirement {
        AlignmentRequirement::natural_for(self.storage)
    }

    /// The natural guarantee every allocation of this carrier states.
    fn element_guarantee(self) -> AlignmentGuarantee {
        AlignmentGuarantee::natural_for(self.storage)
    }
}

/// Arena position of one node of the program's ABI expression arena.
///
/// This is a reference into the [`ExprNode`] arena
/// [`VerifiedKernelProgram::abi_expressions`] retains, not a second expression
/// vocabulary. `relocate-abi-expressions-into-tiler-ir` replaced the compiler's
/// own `HostExpr` — a nine-node table of `U64`/`Bool`/`CheckedMultiply` — with
/// the shared `AbiExpr` domain, because the two covered the same three facts
/// (guards, accessible byte counts, launch geometry) with two vocabularies that
/// nothing kept in agreement. That is the drift hazard ADR 0068 exists to
/// prevent, and it is why the width widened from `u8` to the arena's own `u32`.
///
/// It survives `complete-program-identity-with-abi-guards-and-routing` — which
/// moved the arena itself into the program — only as the spelling this crate's
/// typed errors and explain subjects use to name a rejected node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct HostExprId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct StageId(u8);

impl HostExprId {
    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

impl StageId {
    pub(crate) const fn index(self) -> u8 {
        self.0
    }
}

/// One target-bound executable program: a verified shared kernel program and
/// the target profile whose feasibility it was assessed under.
///
/// The ABI arena, the applicability guard, the entry ABI, and the
/// routing-commit contract are all inside `core` and inside its canonical
/// identity. This wrapper adds the one fact the target-neutral program
/// deliberately does not carry: which target profile it was planned for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelProgram {
    target_profile: TargetProfile,
    core: VerifiedKernelProgram,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactConstructionPlan {
    semantic_identity: SemanticIdentity,
    numerical_contract_key: &'static str,
    numerical_realizations: Vec<NumericalRealization>,
    /// The immutable profile the plan was assessed against. Its key and exact
    /// descriptor remain inseparable throughout artifact construction.
    target_profile: TargetProfile,
    /// Feasibility rules the plan's candidates were assessed under.
    ///
    /// A second, independent identity beside the profile rather than a field of
    /// it: one profile can be re-assessed under new rules and one rule set
    /// applies across profiles, so neither determines the other. The artifact
    /// layer records them as two references for exactly that reason.
    feasibility_rule_set: FeasibilityRuleSetIdentity,
    entry_regions: Vec<RegionId>,
    entry_deferred_predicates: Vec<EntryDeferredPredicate>,
    /// Arena position of the guard deciding whether this plan may be routed to.
    ///
    /// Named for what it decides rather than for "routing": the portfolio-level
    /// sense of that word orders variants against each other, and this guard
    /// orders nothing.
    applicability_guard: u32,
    lowering_providers: Vec<LoweringProviderIdentity>,
    request_subject: crate::request::VerifiedRequestSubject,
    verified_program: KernelProgram,
    verified_schedules: Vec<VerifiedScheduledRegion>,
    verified_kernels: Vec<VerifiedKernel>,
}

/// One compiler-minted deferred predicate bound to its exact program entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EntryDeferredPredicate {
    entry: u32,
    predicate: DeferredPredicate,
}

impl EntryDeferredPredicate {
    /// The zero-based program-entry ordinal whose prepared subject is queried.
    pub(crate) const fn entry(&self) -> u32 {
        self.entry
    }

    /// The typed predicate and executable query contract.
    pub(crate) const fn predicate(&self) -> &DeferredPredicate {
        &self.predicate
    }
}

impl KernelProgram {
    /// Returns the verified target-neutral program this target binding wraps.
    ///
    /// No longer `#[cfg(test)]`: `crate::session` is the reviewed public facade
    /// this accessor's previous comment was waiting for, and an artifact
    /// assembler outside this crate cannot package a variant without the
    /// program `push_variant` binds against.
    pub(crate) const fn core(&self) -> &VerifiedKernelProgram {
        &self.core
    }

    pub(crate) fn stage_count(&self) -> usize {
        self.core.stages().len()
    }

    #[allow(
        dead_code,
        reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
    )]
    pub(crate) fn dependency_count(&self) -> usize {
        self.core.dependencies().len()
    }
}

impl ArtifactConstructionPlan {
    pub(crate) fn lowering_providers(&self) -> &[LoweringProviderIdentity] {
        &self.lowering_providers
    }

    /// Returns compiler-minted deferred predicates in entry then predicate order.
    pub(crate) fn entry_deferred_predicates(&self) -> &[EntryDeferredPredicate] {
        &self.entry_deferred_predicates
    }

    /// Returns the target-bound program whose ABI contract this plan packages.
    pub(crate) const fn verified_program(&self) -> &KernelProgram {
        &self.verified_program
    }

    /// Returns the canonical descriptor bytes of the assessed target profile.
    pub(crate) fn target_profile_descriptor(&self) -> &[u8] {
        self.target_profile.canonical_descriptor()
    }

    /// Returns the feasibility rules this plan's candidates were assessed under.
    ///
    /// Minted by the feasibility authority and handed over whole, like a
    /// capability key: the pair enters artifact identity, and a consumer
    /// composing a key and a revision of its own would be a second derivation of
    /// one identity.
    pub(crate) const fn feasibility_rule_set(&self) -> FeasibilityRuleSetIdentity {
        self.feasibility_rule_set
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProgramError {
    HostExpression {
        rule: &'static str,
        expression: HostExprId,
    },
    Structure {
        rule: &'static str,
    },
    Storage {
        rule: &'static str,
    },
    Abi {
        rule: &'static str,
        stage: StageId,
    },
    Routing {
        rule: &'static str,
    },
    /// The shared kernel-program builder rejected a locally malformed insertion.
    CoreConstruction(KernelProgramBuildError),
    /// The shared whole-program verifier rejected the assembled program.
    CoreVerification(KernelProgramDiagnostic),
}

impl ProgramError {
    /// Returns the stable rule identifier a rejected program reports.
    pub(crate) fn rule(&self) -> &str {
        match self {
            Self::HostExpression { rule, .. }
            | Self::Structure { rule }
            | Self::Storage { rule }
            | Self::Abi { rule, .. }
            | Self::Routing { rule } => rule,
            Self::CoreConstruction(_) => "core-construction",
            Self::CoreVerification(diagnostic) => diagnostic.rule(),
        }
    }
}

impl fmt::Display for ProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HostExpression { rule, expression } => write!(
                formatter,
                "program.host-expression.{rule}: expression {} rejected",
                expression.0
            ),
            Self::Structure { rule } => write!(formatter, "program.structure.{rule}: rejected"),
            Self::Storage { rule } => write!(formatter, "program.storage.{rule}: rejected"),
            Self::Abi { rule, stage } => {
                write!(formatter, "program.abi.{rule}: stage {} rejected", stage.0)
            }
            Self::Routing { rule } => write!(formatter, "program.routing.{rule}: rejected"),
            Self::CoreConstruction(_) => {
                write!(formatter, "program.core.core-construction: rejected")
            }
            Self::CoreVerification(diagnostic) => {
                write!(formatter, "program.core.{}: rejected", diagnostic.rule())
            }
        }
    }
}

impl Error for ProgramError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HostExpression { .. }
            | Self::Structure { .. }
            | Self::Storage { .. }
            | Self::Abi { .. }
            | Self::Routing { .. } => None,
            Self::CoreConstruction(source) => Some(source),
            Self::CoreVerification(source) => Some(source),
        }
    }
}

impl From<KernelProgramBuildError> for ProgramError {
    fn from(value: KernelProgramBuildError) -> Self {
        Self::CoreConstruction(value)
    }
}

// ---------------------------------------------------------------------------
// What one cover assembles into
// ---------------------------------------------------------------------------

/// One value an assembled program owns beyond the tensors its caller binds.
///
/// Its extents are the iteration extents of the stage whose owning write
/// defines it, because every region of this profile's schedule vocabulary
/// writes one element per iteration point. The element count is derived from
/// those extents rather than carried beside them, so the allocation capacity,
/// the value's required bytes, the view window, and the ABI byte expression are
/// four readings of one fact instead of four numbers something has to keep in
/// agreement.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AssemblyValue {
    shape: Shape,
    /// `Temporary` for a value another stage reads, `Output` for one a named
    /// program output publishes.
    role: ValueRole,
}

/// Which program value one stage access binds.
///
/// A stage's accesses realize its kernel's buffer parameters positionally, so
/// there is one of these per access, in access order. Nothing here decides
/// whether the binding is *legal*: [`KernelProgramBuilder::push_stage`] compares
/// each bound value's role, component role, element type, and addressed extent
/// against the buffer it fills, and this only says which value is named.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AssemblyBinding {
    /// The semantic program's declared input at this ordinal.
    Input(usize),
    /// The internal value at this position of [`CoverAssembly::internals`].
    Internal(usize),
}

/// One dispatch of the program a cover assembles into.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AssemblyStage {
    /// The attribution atoms this stage claims, taken from the verified region
    /// the stage dispatches rather than from the cover region as a whole.
    ///
    /// **Per pass, because the passes of one region do not claim it uniformly.**
    /// A split reduction's partial pass claims the reduction occurrence's first
    /// stage and its combine claims the stage after it, so reading the cover
    /// region's atoms for every pass would have both passes claim the same work.
    /// A publishing copy claims nothing at all — it computes no occurrence, it
    /// moves a value — and that is the one empty claim this profile mints.
    ///
    /// It projects onto [`tiler_ir::program::CoveredOccurrence`] through
    /// [`covered`], which keeps only first-stage atoms: whole-program coverage
    /// is an obligation of the *occurrence*, discharged once, and the IR admits
    /// the resulting uncovering pass exactly when a declaration accounts for it
    /// — the split [`KernelProgramBuilder::push_partial_reduction`] states, or
    /// the copy [`KernelProgramBuilder::push_publishing_copy`] states. Without
    /// either, it is a stage computing nothing and `UncoveringStage` rejects it.
    pub(crate) coverage: Vec<SemanticStage>,
    pub(crate) bindings: Vec<AssemblyBinding>,
}

/// One producer-to-consumer data edge between two stages of one program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AssemblyDependency {
    producer: usize,
    consumer: usize,
    value: usize,
}

/// One split-reduction contract two consecutive passes of a subprogram declare.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AssemblySplit {
    pub(crate) producer: usize,
    pub(crate) combiner: usize,
    pub(crate) partial: usize,
    pub(crate) result: usize,
    pub(crate) partition: ContributorPartition,
}

/// One publishing-copy contract a two-dispatch publishing region declares.
///
/// Distinct from [`AssemblySplit`] rather than a widening of it, because the two
/// declare different facts: a split partitions a fold's contributors and
/// combines partials, while a copy moves one value into the buffer the interface
/// publishes. Collapsing them would make `contributors_per_partition` a number a
/// copy has to invent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AssemblyPublishingCopy {
    pub(crate) source_stage: usize,
    pub(crate) publisher: usize,
    pub(crate) source: usize,
    pub(crate) published: usize,
}

/// One staged-realization contract two dispatches of one occurrence declare.
///
/// Distinct from both declarations above, and for the reason the IR states: a
/// split partitions a fold's contributors, a copy moves a value into the buffer
/// the interface publishes, and this continues one occurrence's realization
/// across a stage boundary. Its two dispatches belong to *different cover
/// regions* — a staged family's stages are separate placements, unlike a split's
/// two passes of one region — which is why it is derived over the flattened
/// stage list rather than inside the per-region loop that mints the other two.
///
/// The occurrence travels as the planner's own [`SemanticMemberId`] because the
/// assembler holds no lowering; [`build_cover_core`] projects it onto the IR's
/// [`tiler_ir::program::SemanticOccurrence`] through the same
/// `OccurrenceLowering` that mints the stage's coverage records, so the
/// declaration and the coverage it continues name one receipt's occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AssemblyStagedRealization {
    pub(crate) producer: usize,
    pub(crate) consumer: usize,
    pub(crate) handed: usize,
    pub(crate) occurrence: crate::region::SemanticMemberId,
}

/// The complete structural description one retained plan's cover assembles into.
///
/// **Every quantity here is read from the cover or from the semantic program,
/// and none from a whole-program strategy recognizer.** Program inputs and keys
/// are the semantic program's declared inputs; one internal value and one
/// program-owned allocation exist per materialization edge, sized by the edge's
/// own element count; one output value exists per ordered named program output;
/// one stage exists per scheduled region, ordered so producers precede
/// consumers; and one data dependency exists per edge, from the producing stage
/// to each consuming stage.
///
/// It proves nothing. [`KernelProgramBuilder::build`] remains the whole-program
/// authority for complete disjoint coverage of the semantic graph, a unique
/// writer per materialized value, boundary-contract satisfaction, temporary
/// initialization and lifetimes, aliasing, ordered opaque effects, ABI and
/// launch references, and named-output coverage. The claim this type carries is
/// only that it constructs the same obligations for N regions that the retired
/// three enumerated shapes constructed for one, two, and three.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverAssembly {
    /// The scheduled regions this program dispatches, in execution order and
    /// parallel to [`Self::stages`].
    regions: Vec<VerifiedScheduledRegion>,
    internals: Vec<AssemblyValue>,
    stages: Vec<AssemblyStage>,
    dependencies: Vec<AssemblyDependency>,
    splits: Vec<AssemblySplit>,
    copies: Vec<AssemblyPublishingCopy>,
    staged: Vec<AssemblyStagedRealization>,
    /// The ordered named program outputs, each naming the internal value that
    /// publishes it, in the semantic program's declaration order.
    outputs: Vec<(OutputKey, usize)>,
}

/// Which failure class one assembly refusal belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AssemblyRefusalClass {
    /// A **missing compilation capability**: the plan is valid, its cover is an
    /// already-verified authority, and the absent authority is this assembler.
    ///
    /// This is the correction the retired `"unsupported-plan-shape"` rule makes
    /// necessary. Reporting a cover the assembler cannot express as invalid
    /// compiler output claims the compiler produced something wrong when it
    /// produced nothing at all, and the two classes are not interchangeable:
    /// one tells a caller their installed authority is incomplete, the other
    /// says the compiler has a bug.
    MissingCapability,
    /// A selected body this compiler did not schedule and cannot lower.
    ///
    /// Keeps the classification it already had rather than being reclassified
    /// alongside the cover shapes: lowering an opaque call is a separate
    /// capability with a separate owner, and moving its class here would change
    /// a decision this ticket does not own.
    UnlowerableBody,
}

/// Why one retained plan's cover cannot be assembled into a kernel program.
///
/// Names the region occurrence the refusal is about, so a caller learns which
/// part of its program the assembler had no expression for rather than that
/// "the shape" was unsupported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AssemblyRefusal {
    region: String,
    rule: &'static str,
    class: AssemblyRefusalClass,
}

impl AssemblyRefusal {
    /// States one refusal directly, so a test can drive the reporting path.
    #[cfg(test)]
    pub(crate) fn stated(
        region: impl Into<String>,
        rule: &'static str,
        class: AssemblyRefusalClass,
    ) -> Self {
        Self {
            region: region.into(),
            rule,
            class,
        }
    }

    fn missing(region: impl Into<String>, rule: &'static str) -> Self {
        Self {
            region: region.into(),
            rule,
            class: AssemblyRefusalClass::MissingCapability,
        }
    }

    /// Returns the bounded explain label of the region the refusal is about.
    pub(crate) fn region(&self) -> &str {
        &self.region
    }

    /// Returns the stable rule identifier naming the missing capability.
    pub(crate) const fn rule(&self) -> &'static str {
        self.rule
    }

    /// Returns the failure class a caller reports this refusal under.
    pub(crate) const fn class(&self) -> AssemblyRefusalClass {
        self.class
    }
}

impl fmt::Display for AssemblyRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "program.assembly.{}: region {} rejected",
            self.rule, self.region
        )
    }
}

impl Error for AssemblyRefusal {}

impl CoverAssembly {
    /// Returns the scheduled regions this program dispatches, in stage order.
    pub(crate) fn regions(&self) -> &[VerifiedScheduledRegion] {
        &self.regions
    }

    /// Derives the whole program description one retained plan assembles into.
    ///
    /// **The cover is consumed, never re-derived.** `verify_cover` already
    /// proved that each placed region is an authoritative candidate, that every
    /// operation is covered, that each ordered named output is produced by
    /// exactly one region, and that the materialization edges recompute exactly
    /// — so reading region membership, materialization edges, and write roles
    /// off it is reading a checked value rather than trusting an unchecked one.
    ///
    /// # Errors
    ///
    /// Returns an [`AssemblyRefusal`] naming the region and the missing
    /// capability. Every refusal is a statement about a shape this assembler
    /// has no expression for; none of them asserts the plan is malformed.
    pub(crate) fn from_plan(
        semantic: &SemanticProgram,
        plan: &SelectedPlan,
    ) -> Result<Self, AssemblyRefusal> {
        let cover = plan.cover();
        let regions = cover.regions();
        if plan.selections().len() != regions.len() {
            return Err(AssemblyRefusal::missing(
                COVER_SUBJECT,
                "cover-selection-arity",
            ));
        }
        // One selection per placed region, and the stages that selection
        // dispatches. A body with no scheduled region -- an opaque call -- has
        // nothing to bind a stage to, and saying so is what stops the plan being
        // assembled as though the call were not in it.
        let mut selected: Vec<&[VerifiedScheduledRegion]> = Vec::with_capacity(regions.len());
        for region in regions {
            let selection = plan
                .selections()
                .iter()
                .find(|selection| selection.occurrence() == region.occurrence())
                .ok_or_else(|| {
                    AssemblyRefusal::missing(region.label(), "cover-region-unselected")
                })?;
            let stages = selection
                .implementation()
                .scheduled_stages()
                .ok_or_else(|| AssemblyRefusal {
                    region: region.label().to_owned(),
                    rule: "unlowerable-opaque-body",
                    class: AssemblyRefusalClass::UnlowerableBody,
                })?;
            if stages.is_empty() {
                return Err(AssemblyRefusal::missing(
                    region.label(),
                    "cover-region-undispatched",
                ));
            }
            selected.push(stages);
        }

        let order = execution_order(cover)?;
        // The flattened stage ordinal each region's first and last dispatch
        // occupies, which is what a data edge between two regions names.
        let mut stage_regions: Vec<VerifiedScheduledRegion> = Vec::new();
        let mut span: Vec<(usize, usize)> = vec![(0, 0); regions.len()];
        for position in &order {
            let stages = selected[*position];
            let first = stage_regions.len();
            stage_regions.extend(stages.iter().cloned());
            span[*position] = (first, stage_regions.len() - 1);
        }

        // Internal values, in the order the assembler mints them: one per
        // materialization edge in the cover's own canonical edge order, then one
        // per pass a subprogram stages between its dispatches, then one per
        // ordered named program output.
        let mut internals: Vec<AssemblyValue> = Vec::new();
        let mut edge_value: Vec<usize> = Vec::with_capacity(cover.materializations().len());
        for edge in cover.materializations() {
            let producer = region_position(regions, edge.producer())?;
            let shape = stage_regions[span[producer].1]
                .region()
                .index
                .iteration_shape
                .clone();
            // The edge states the element count and the producing region states
            // the extents it writes. A disagreement is refused rather than
            // silently resized, because resizing would publish a buffer neither
            // authority asked for.
            if shape_elements(&shape).map_err(|_| {
                AssemblyRefusal::missing(regions[producer].label(), "materialized-extent-overflow")
            })? != edge.element_count()
            {
                return Err(AssemblyRefusal::missing(
                    regions[producer].label(),
                    "materialized-extent-disagreement",
                ));
            }
            edge_value.push(internals.len());
            internals.push(AssemblyValue {
                shape,
                role: ValueRole::Temporary,
            });
        }
        // Which regions materialize an edge, and which of those also retain a
        // declared named result. Derived before the staged values below because
        // a publishing region's first dispatch writes the *edge* rather than a
        // staged value of its own, so how many staged values it mints depends on
        // this answer.
        let materializing: Vec<bool> = regions
            .iter()
            .map(|region| {
                cover
                    .materializations()
                    .iter()
                    .any(|edge| edge.producer() == region.occurrence())
            })
            .collect();
        let named_results: Vec<&[SemanticValueId]> =
            regions.iter().map(CoverRegion::named_results).collect();
        let attribution = attribute_named_outputs(semantic, &named_results, &materializing)
            .map_err(|failure| {
                AssemblyRefusal::missing(
                    failure
                        .region()
                        .map_or(COVER_SUBJECT, |position| regions[position].label()),
                    "cover-named-output-attribution",
                )
            })?;
        // A region whose value is published *and* consumed: its first dispatch
        // stages the value the edge carries and its last publishes a copy.
        let publishing: Vec<bool> = (0..regions.len())
            .map(|region| materializing[region] && attribution.contains(&region))
            .collect();
        // Exactly two dispatches, because that is what the physical layer offers
        // for this shape and what the program-scope declaration below accounts
        // for. A longer subprogram would be a split *and* a publication, whose
        // middle passes have no declared account, and refusing it by name is
        // what keeps an unaccounted-for dispatch from reaching the verifier.
        for position in &order {
            let (first, last) = span[*position];
            if publishing[*position] && last != first + 1 {
                return Err(AssemblyRefusal::missing(
                    regions[*position].label(),
                    "publishing-copy-pass-count",
                ));
            }
        }
        // One staged value per dispatch of a subprogram other than its last: the
        // pass that writes it and the pass that reads it are two dispatches, and
        // the dispatch boundary is what makes the staged value visible. A
        // publishing region mints none — its first dispatch's owning write goes
        // to the materialization edge, which the cover already minted above.
        let mut pass_values: Vec<Vec<usize>> = vec![Vec::new(); regions.len()];
        for position in &order {
            let (first, last) = span[*position];
            if publishing[*position] {
                continue;
            }
            for staged in &stage_regions[first..last] {
                pass_values[*position].push(internals.len());
                internals.push(AssemblyValue {
                    shape: staged.region().index.iteration_shape.clone(),
                    role: ValueRole::Temporary,
                });
            }
        }
        // One value per ordered named program output, in declaration order,
        // attributed to the region the *cover* says retains that named result.
        //
        // Deliberately by value rather than by the position a publishing region
        // occupies in execution order. Execution order is the cover's canonical
        // occurrence order, which has nothing to do with the order the caller
        // declared its interface in — so pairing the two lists positionally
        // publishes the right buffers under the wrong keys whenever they
        // disagree, which is the interchangeable-outputs interface the
        // architectural contract forbids. With one declared output the two
        // agree, which is why the guess was invisible while `output-arity`
        // stood.
        let mut outputs: Vec<(OutputKey, usize)> = Vec::new();
        let mut output_value: Vec<Option<usize>> = vec![None; regions.len()];
        for (output, position) in semantic.outputs().zip(&attribution) {
            let shape = semantic
                .shape(output.value())
                .map_err(|_| {
                    AssemblyRefusal::missing(regions[*position].label(), "named-output-unshaped")
                })?
                // Physical assembly sizes every allocation and view from this
                // shape, so a symbolic one has nothing to size. Refused by its
                // own rule rather than folded into the handle failure above,
                // because the program is well formed and only unschedulable.
                .as_static()
                .ok_or_else(|| {
                    AssemblyRefusal::missing(regions[*position].label(), "named-output-symbolic")
                })?
                .clone();
            output_value[*position] = Some(internals.len());
            outputs.push((output.key().clone(), internals.len()));
            internals.push(AssemblyValue {
                shape,
                role: ValueRole::Output,
            });
        }

        let inputs = semantic.input_count();
        let mut stages: Vec<AssemblyStage> = Vec::with_capacity(stage_regions.len());
        // The cover region each stage belongs to, so a refusal names the region
        // rather than a flattened dispatch ordinal no reader can resolve.
        let mut stage_region: Vec<usize> = Vec::with_capacity(stage_regions.len());
        let mut splits: Vec<AssemblySplit> = Vec::new();
        let mut copies: Vec<AssemblyPublishingCopy> = Vec::new();
        for position in &order {
            let region = &regions[*position];
            let (first, last) = span[*position];
            // Which edge this region reads across, and which it writes across.
            // A region reading or producing several is refused: `TensorRole`
            // separates reads of *declared inputs* by ordinal and carries none
            // for an intermediate, so two of either leave nothing to say which
            // access binds which edge.
            let consumed: Vec<usize> = cover
                .materializations()
                .iter()
                .enumerate()
                .filter(|(_, edge)| {
                    edge.consumers()
                        .iter()
                        .any(|consumer| consumer == region.occurrence())
                })
                .map(|(edge, _)| edge_value[edge])
                .collect();
            let produced: Vec<usize> = cover
                .materializations()
                .iter()
                .enumerate()
                .filter(|(_, edge)| edge.producer() == region.occurrence())
                .map(|(edge, _)| edge_value[edge])
                .collect();
            if produced.len() > 1 {
                return Err(AssemblyRefusal::missing(
                    region.label(),
                    "cover-region-multiple-materializations",
                ));
            }
            for stage in first..=last {
                let accesses = &stage_regions[stage].region().index.accesses;
                let Some((write, reads)) = accesses.split_last() else {
                    return Err(AssemblyRefusal::missing(
                        region.label(),
                        "region-without-accesses",
                    ));
                };
                let mut bindings = Vec::with_capacity(accesses.len());
                let mut intermediate_reads = 0_usize;
                for (access_position, read) in reads.iter().enumerate() {
                    if read.mode != AccessMode::Read {
                        return Err(AssemblyRefusal::missing(
                            region.label(),
                            "region-access-order",
                        ));
                    }
                    match read.tensor {
                        TensorRole::Input => {
                            let access = u32::try_from(access_position)
                                .ok()
                                .map(AccessOrdinal::new)
                                .ok_or_else(|| {
                                    AssemblyRefusal::missing(
                                        region.label(),
                                        "region-access-ordinal",
                                    )
                                })?;
                            let ordinal = stage_regions[stage]
                                .declared_input_at(access)
                                .and_then(|ordinal| usize::try_from(ordinal.get()).ok())
                                .ok_or_else(|| {
                                    AssemblyRefusal::missing(
                                        region.label(),
                                        "region-input-association",
                                    )
                                })?;
                            if ordinal >= inputs {
                                return Err(AssemblyRefusal::missing(
                                    region.label(),
                                    "region-input-ordinal",
                                ));
                            }
                            bindings.push(AssemblyBinding::Input(ordinal));
                        }
                        TensorRole::Intermediate => {
                            intermediate_reads += 1;
                            // One intermediate read per dispatch, and exactly one
                            // edge for the dispatch that reads across the region
                            // boundary. The fieldless role carries no edge
                            // coordinate, so a second one leaves nothing to say
                            // which edge it binds, and guessing would bind a
                            // stage to the wrong buffer.
                            if intermediate_reads > 1 || (stage == first && consumed.len() != 1) {
                                return Err(AssemblyRefusal::missing(
                                    region.label(),
                                    "cover-intermediate-read-attribution",
                                ));
                            }
                            // The first dispatch of a region reads what the cover
                            // hands the region; every later one reads what the
                            // dispatch before it staged — except a publishing
                            // copy, whose source is the materialization edge the
                            // dispatch before it wrote rather than a staged value
                            // of its own.
                            bindings.push(AssemblyBinding::Internal(if stage == first {
                                consumed[0]
                            } else if publishing[*position] {
                                *produced.first().ok_or_else(|| {
                                    AssemblyRefusal::missing(
                                        region.label(),
                                        "cover-materialization-unnamed",
                                    )
                                })?
                            } else {
                                pass_values[*position][stage - first - 1]
                            }));
                        }
                        TensorRole::Output => {
                            return Err(AssemblyRefusal::missing(
                                region.label(),
                                "region-reads-program-output",
                            ));
                        }
                    }
                }
                if write.mode != AccessMode::Write {
                    return Err(AssemblyRefusal::missing(
                        region.label(),
                        "region-access-order",
                    ));
                }
                let written = match (write.tensor, stage == last) {
                    // The region's owning write, whose target the cover decided:
                    // the edge it materializes, or the output it publishes.
                    (TensorRole::Intermediate, true) => *produced.first().ok_or_else(|| {
                        AssemblyRefusal::missing(region.label(), "cover-materialization-unnamed")
                    })?,
                    (TensorRole::Output, true) => output_value[*position].ok_or_else(|| {
                        AssemblyRefusal::missing(region.label(), "cover-named-output-unnamed")
                    })?,
                    // A pass that is not the region's last stages its result for
                    // the pass after it, so it writes an intermediate whatever
                    // the region as a whole writes — except a publishing region's
                    // first dispatch, whose owning write *is* the materialization
                    // edge, because the publication it also owes is the next
                    // dispatch's write rather than this one's.
                    (TensorRole::Intermediate, false) if publishing[*position] => {
                        *produced.first().ok_or_else(|| {
                            AssemblyRefusal::missing(
                                region.label(),
                                "cover-materialization-unnamed",
                            )
                        })?
                    }
                    (TensorRole::Intermediate, false) => pass_values[*position][stage - first],
                    (TensorRole::Output | TensorRole::Input, false) | (TensorRole::Input, true) => {
                        return Err(AssemblyRefusal::missing(
                            region.label(),
                            "region-write-role",
                        ));
                    }
                };
                bindings.push(AssemblyBinding::Internal(written));
                stage_region.push(*position);
                stages.push(AssemblyStage {
                    coverage: stage_regions[stage].semantic_members().to_vec(),
                    bindings,
                });
            }
            // The publishing-copy contract a two-dispatch publishing region
            // declares. It is stated instead of the split contracts below rather
            // than beside them: the two dispatches are not a split — nothing is
            // partitioned and no partial is combined — and the value the second
            // reads is the edge the first wrote, which is the fact the
            // declaration names.
            if publishing[*position] {
                copies.push(AssemblyPublishingCopy {
                    source_stage: first,
                    publisher: last,
                    source: *produced.first().ok_or_else(|| {
                        AssemblyRefusal::missing(region.label(), "cover-materialization-unnamed")
                    })?,
                    published: output_value[*position].ok_or_else(|| {
                        AssemblyRefusal::missing(region.label(), "cover-named-output-unnamed")
                    })?,
                });
                continue;
            }
            // The split contract each consecutive pair of passes declares. The
            // partition is read back from the producing pass's own topology, so
            // the program-scope declaration agrees with the schedule that
            // produced it by construction rather than by a second derivation.
            for stage in first..last {
                let partition =
                    crate::physical::declared_partial_partition(stage_regions[stage].region())
                        .ok_or_else(|| {
                            AssemblyRefusal::missing(region.label(), "split-partition-undeclared")
                        })?;
                let combiner = stage + 1;
                let result = match stages[combiner].bindings.last() {
                    Some(AssemblyBinding::Internal(value)) => *value,
                    _ => {
                        return Err(AssemblyRefusal::missing(
                            region.label(),
                            "split-result-unnamed",
                        ));
                    }
                };
                splits.push(AssemblySplit {
                    producer: stage,
                    combiner,
                    partial: pass_values[*position][stage - first],
                    result,
                    partition,
                });
            }
        }

        // **Every dispatch that covers no occurrence must be accounted for, and
        // this profile names three accounts.** `tiler_ir::program` refuses an
        // uncovering stage under `UncoveringStage` unless a declaration explains
        // it: the final pass of a declared split, the publisher of a declared
        // publishing copy, or the consumer of a declared staged realization. A
        // stage covers an occurrence exactly when it claims some occurrence's
        // *first* attribution atom, because that is what whole-program coverage
        // is keyed on — a later stage continues a realization the first stage
        // already claimed.
        //
        // The third account is derived here rather than in the per-region loop
        // above, and the region boundary is why: a staged elementary family's
        // stages are separate cover regions, so neither region can see the other
        // and the producer of a chain is found over the flattened stage list.
        // The chain obligation itself is proven before any of it, by the one
        // authority that owns the rule.
        let mut all_claims: Vec<SemanticStage> = stages
            .iter()
            .flat_map(|stage| stage.coverage.iter().copied())
            .collect();
        let subject: Vec<SemanticStage> = (0..semantic.operation_count())
            .map(|member| {
                u32::try_from(member)
                    .map(|member| SemanticStage::first(crate::region::SemanticMemberId(member)))
                    .map_err(|_| AssemblyRefusal::missing(COVER_SUBJECT, "cover-member-ordinal"))
            })
            .collect::<Result<_, _>>()?;
        // **The dispatches' claims partition the program's occurrences, with
        // every later stage directly behind the stage it continues.** This is
        // what the derivation below rests on: it searches for the dispatch that
        // claims each continued atom's predecessor and takes the answer as
        // existing and unique, which is a claim about the whole chain rather
        // than about any one dispatch. `chain_realizes_subject` is the one place
        // that rule is written — the physical frontier and the whole-program
        // schedule binding owe the identical obligation, and two spellings of
        // one rule are two rules to keep in agreement.
        //
        // Stated rather than presented as a live gate: `verify_cover` proves the
        // placed regions' atoms partition the graph and every dispatch claims a
        // subset of its own region's atoms, so no cover the boundary currently
        // admits reaches this refusal. It is the condition rather than its
        // consequence — a region realizing its atoms across a different number
        // of dispatches than it claims makes it reachable — and meanwhile it is
        // what keeps the search below a proved fact instead of an assumption.
        if !crate::region::chain_realizes_subject(&mut all_claims, &subject) {
            return Err(AssemblyRefusal::missing(
                COVER_SUBJECT,
                "dispatch-chain-unrealizing",
            ));
        }
        let mut realizations: Vec<AssemblyStagedRealization> = Vec::new();
        for (stage, claimed) in stages.iter().enumerate() {
            if splits.iter().any(|split| split.combiner == stage)
                || copies.iter().any(|copy| copy.publisher == stage)
            {
                continue;
            }
            for atom in claimed.coverage.iter().filter(|atom| !atom.is_first()) {
                // The dispatch that computed the stage before this one. The
                // chain check above already proved exactly one claims it, so
                // this searches a proven-unique answer rather than choosing
                // among candidates; the refusal is unreachable while that proof
                // holds and is stated rather than asserted, so a weakened chain
                // rule refuses by name instead of panicking or silently taking
                // whichever dispatch happened to be first.
                let producer = stages
                    .iter()
                    .position(|candidate| {
                        candidate
                            .coverage
                            .iter()
                            .any(|earlier| earlier.next_stage() == *atom)
                    })
                    .ok_or_else(|| {
                        AssemblyRefusal::missing(
                            regions[stage_region[stage]].label(),
                            "realization-stage-unbegun",
                        )
                    })?;
                // A dispatch computing both stages itself hands nothing across a
                // boundary and needs no account: it claims the occurrence's
                // first atom, so program-scope coverage already names it.
                if producer == stage {
                    continue;
                }
                // The value the producer hands on: the one internal value this
                // dispatch reads that the producer's owning write defines. A
                // dispatch reads at most one intermediate — `TensorRole`
                // carries no ordinal for one — so there is nothing to choose
                // between, and a dispatch that reads none continues a
                // realization through no value at all.
                let handed = claimed
                    .bindings
                    .split_last()
                    .and_then(|(_, reads)| {
                        reads.iter().find_map(|binding| match binding {
                            AssemblyBinding::Internal(value)
                                if stages[producer].bindings.last()
                                    == Some(&AssemblyBinding::Internal(*value)) =>
                            {
                                Some(*value)
                            }
                            AssemblyBinding::Internal(_) | AssemblyBinding::Input(_) => None,
                        })
                    })
                    .ok_or_else(|| {
                        AssemblyRefusal::missing(
                            regions[stage_region[stage]].label(),
                            "realization-stage-unhanded",
                        )
                    })?;
                realizations.push(AssemblyStagedRealization {
                    producer,
                    consumer: stage,
                    handed,
                    occurrence: atom.member(),
                });
            }
        }

        let dependencies = derive_dependencies(&stages, internals.len())?;
        check_materialized_values_are_read(&internals, &dependencies)?;
        Ok(Self {
            regions: stage_regions,
            internals,
            stages,
            dependencies,
            splits,
            copies,
            staged: realizations,
            outputs,
        })
    }

    /// Spells one assembly from stated structural facts.
    ///
    /// **The compile path never reaches this.** [`Self::from_plan`] is its only
    /// derivation, and every program this crate compiles goes through it. This
    /// exists so a test can state a cover's facts directly — which values are
    /// materialized, which stage writes each, which named output publishes which
    /// — and check what the assembler builds from them, including shapes no
    /// cover the current region vocabulary admits can state.
    ///
    /// The data dependencies are *derived* rather than stated, by the same
    /// function [`Self::from_plan`] uses, so a stated assembly cannot describe an
    /// edge set the derived one would not have produced.
    #[cfg(test)]
    pub(crate) fn stated(
        regions: Vec<VerifiedScheduledRegion>,
        internals: Vec<(Shape, ValueRole)>,
        stages: Vec<AssemblyStage>,
        splits: Vec<AssemblySplit>,
        copies: Vec<AssemblyPublishingCopy>,
        realizations: Vec<AssemblyStagedRealization>,
        outputs: Vec<(OutputKey, usize)>,
    ) -> Result<Self, AssemblyRefusal> {
        let internals: Vec<AssemblyValue> = internals
            .into_iter()
            .map(|(shape, role)| AssemblyValue { shape, role })
            .collect();
        let dependencies = derive_dependencies(&stages, internals.len())?;
        check_materialized_values_are_read(&internals, &dependencies)?;
        Ok(Self {
            regions,
            internals,
            stages,
            dependencies,
            splits,
            copies,
            staged: realizations,
            outputs,
        })
    }
}

/// Why a cover's regions cannot be paired with a program's declared outputs.
///
/// Every variant is a statement about the *pairing*, never about either side
/// alone: the cover is legal and the program is admitted, and what is missing is
/// a one-to-one correspondence between the ordered named outputs and the regions
/// that write them. They are separate variants rather than one flag because each
/// names a different thing a caller or a later widening would have to change,
/// and because a check that cannot distinguish them cannot be driven against
/// each case that must fail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttributionFailure {
    /// No placed region retains this declared output as a named result.
    ///
    /// Also the answer for a declared output the program holds no value ordinal
    /// for. That is the fail-closed reading of an impossible state rather than a
    /// variant of its own: [`SemanticProgram::outputs`] yields values of the
    /// same program [`value_ordinal`] searches, so no input distinguishes the
    /// two and a separate variant would be an arm nothing could drive.
    Unattributed { output: usize },
    /// Several placed regions retain one declared output.
    Ambiguous { output: usize, region: usize },
    /// One region retains two declared outputs, so its one owning write would
    /// have to publish both.
    Shared { region: usize },
    /// A region that materializes nothing is attributed no declared output, so
    /// nothing names what its owning write produces.
    Unpublished { region: usize },
}

impl AttributionFailure {
    /// Returns the region the failure is about, when it is about one.
    const fn region(self) -> Option<usize> {
        match self {
            Self::Unattributed { .. } => None,
            Self::Ambiguous { region, .. }
            | Self::Shared { region }
            | Self::Unpublished { region } => Some(region),
        }
    }
}

/// Pairs a program's ordered named outputs with the regions that publish them.
///
/// Returns, for each declared output in declaration order, the position of its
/// publishing region — which is what lets [`CoverAssembly::from_plan`] mint one
/// output value per declared output and bind it to the write that produces it.
///
/// **The pairing is a proved bijection, not a zip.** Each declared output is
/// attributed to the region whose retained named results contain its value
/// ordinal, and the two directions are checked separately: no output may be
/// attributed to zero or several regions, and no region may be attributed
/// several outputs or none while writing an output rather than a
/// materialization edge. A region's one owning write goes to exactly one place,
/// so anything else is a description the schedule cannot realize.
///
/// **A region that materializes an edge *and* publishes is admitted, and used to
/// be the arm nearest the surface.** `MaterializesAndPublishes` refused it on
/// the reading that one owning write would have to serve both — true of one
/// dispatch, and the reason the arm stood while every region was one dispatch.
/// A published-and-consumed region is now two: the first stages the edge, the
/// second publishes a copy, and [`CoverAssembly::from_plan`] binds each write to
/// its own value. The variant is gone rather than left unreachable, because a
/// refusal whose premise has been replaced is a claim the next reader would take
/// as current.
///
/// **Every refusal below is defence in depth against the search as it stands,
/// and that is stated rather than presented as a live gate.** `verify_cover`
/// already proved each ordered named output is produced by exactly one placed
/// region, region formation refuses a duplicated named-result producer, and
/// `physical::spell_region` declines a region straddling two outputs' recognized
/// partitions before any of them can be proposed — so no cover the boundary
/// currently admits reaches any arm. They are the conditions rather than their
/// consequences: a profile that lets one write serve two consumers, or a
/// duplication policy that admits a named-result producer, makes each reachable,
/// and `named_output_attribution_can_say_no_in_every_direction` is what proves
/// meanwhile that they can say no.
///
/// # Errors
///
/// Returns the [`AttributionFailure`] naming which pairing obligation failed and
/// the region it is about.
fn attribute_named_outputs(
    semantic: &SemanticProgram,
    named_results: &[&[SemanticValueId]],
    materializing: &[bool],
) -> Result<Vec<usize>, AttributionFailure> {
    let mut attribution = Vec::with_capacity(semantic.output_count());
    let mut attributed: Vec<Option<usize>> = vec![None; named_results.len()];
    for (output, declared) in semantic.outputs().enumerate() {
        let value = value_ordinal(semantic, declared.value())
            .ok_or(AttributionFailure::Unattributed { output })?;
        let mut retaining = named_results
            .iter()
            .enumerate()
            .filter(|(_, retained)| retained.contains(&value))
            .map(|(position, _)| position);
        let position = retaining
            .next()
            .ok_or(AttributionFailure::Unattributed { output })?;
        if let Some(region) = retaining.next() {
            return Err(AttributionFailure::Ambiguous { output, region });
        }
        if attributed[position].is_some() {
            return Err(AttributionFailure::Shared { region: position });
        }
        attributed[position] = Some(output);
        attribution.push(position);
    }
    // The converse direction. A region materializing nothing has an owning write
    // whose only remaining destination is a declared output, so one that no
    // output claims describes a buffer the program's interface never names.
    for (region, claim) in attributed.iter().enumerate() {
        if claim.is_none() && !materializing.get(region).copied().unwrap_or(true) {
            return Err(AttributionFailure::Unpublished { region });
        }
    }
    Ok(attribution)
}

/// Refuses an assembly that materializes a value no stage reads.
///
/// A materialization edge exists because the cover decided a value crosses a
/// region boundary, so a described program where nothing reads it is one whose
/// description and whose cover disagree. Left unrefused it would assemble: the
/// whole-program verifier requires a *writer* for every value and a dependency
/// behind every cross-stage *read*, and a temporary nobody reads violates
/// neither. The result would be a program that allocates and fills a buffer for
/// no consumer, which is the "silently wrong shape" this ticket exists to
/// prevent rather than a cost.
///
/// A value published as a named output is deliberately exempt: publishing *is*
/// its consumer, and it leaves the program through the interface rather than
/// through a stage.
fn check_materialized_values_are_read(
    internals: &[AssemblyValue],
    dependencies: &[AssemblyDependency],
) -> Result<(), AssemblyRefusal> {
    for (value, internal) in internals.iter().enumerate() {
        if internal.role == ValueRole::Temporary
            && !dependencies
                .iter()
                .any(|dependency| dependency.value == value)
        {
            return Err(AssemblyRefusal::missing(
                COVER_SUBJECT,
                "materialized-value-unread",
            ));
        }
    }
    Ok(())
}

/// Derives one data dependency per internal value, in the order values were
/// minted, from the stage that defines it to each stage that reads it.
///
/// **The bindings already say this.** A stage's last binding names the value its
/// owning write defines and its earlier bindings name what it reads, so the edge
/// set is a reading of the stage list rather than a second description of it.
/// For a materialization edge the derived edge is the cover's own
/// producer-to-consumer edge; for a value one subprogram pass stages for the
/// next it is the dispatch boundary that makes the partials visible, which is
/// why a split needs no barrier and declares none.
fn derive_dependencies(
    stages: &[AssemblyStage],
    internals: usize,
) -> Result<Vec<AssemblyDependency>, AssemblyRefusal> {
    let mut producers: Vec<Option<usize>> = vec![None; internals];
    let mut consumers: Vec<Vec<usize>> = vec![Vec::new(); internals];
    for (position, stage) in stages.iter().enumerate() {
        let Some((written, read_bindings)) = stage.bindings.split_last() else {
            return Err(AssemblyRefusal::missing(
                COVER_SUBJECT,
                "stage-without-write",
            ));
        };
        for binding in read_bindings {
            if let AssemblyBinding::Internal(value) = binding {
                consumers
                    .get_mut(*value)
                    .ok_or_else(|| {
                        AssemblyRefusal::missing(COVER_SUBJECT, "binding-names-no-value")
                    })?
                    .push(position);
            }
        }
        let AssemblyBinding::Internal(value) = written else {
            return Err(AssemblyRefusal::missing(
                COVER_SUBJECT,
                "stage-writes-an-input",
            ));
        };
        let slot = producers
            .get_mut(*value)
            .ok_or_else(|| AssemblyRefusal::missing(COVER_SUBJECT, "binding-names-no-value"))?;
        if slot.is_some() {
            // Two writers of one value is what `KernelProgramBuilder::build`
            // refuses as an aliasing violation, and describing it here would
            // hand it a program it must then reject. Refusing at the description
            // keeps the refusal attributable to the cover rather than to the
            // assembly it produced.
            return Err(AssemblyRefusal::missing(
                COVER_SUBJECT,
                "value-written-twice",
            ));
        }
        *slot = Some(position);
    }
    let mut dependencies = Vec::new();
    for (value, readers) in consumers.iter().enumerate() {
        let Some(producer) = producers[value] else {
            return Err(AssemblyRefusal::missing(
                COVER_SUBJECT,
                "internal-unwritten",
            ));
        };
        for consumer in readers {
            dependencies.push(AssemblyDependency {
                producer,
                consumer: *consumer,
                value,
            });
        }
    }
    Ok(dependencies)
}

/// The explain subject a refusal about the cover as a whole is attributed to.
const COVER_SUBJECT: &str = "region-cover";

/// Orders a cover's regions so every producer precedes each of its consumers.
///
/// **Region identifiers cannot order this.** They are constants of the schedule
/// vocabulary — every elementwise region carries `RegionId::new(0)` whichever
/// occurrences it covers — so sorting by them returns an arbitrary order the
/// moment a cover places two regions the same builder produced. The cover's
/// materialization edges are the authority for what must precede what, and the
/// cover's own canonical occurrence order breaks every remaining tie, so one
/// cover has exactly one execution order.
pub(crate) fn execution_order(cover: &RegionCover) -> Result<Vec<usize>, AssemblyRefusal> {
    let regions = cover.regions();
    let mut indegree = vec![0_usize; regions.len()];
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); regions.len()];
    for edge in cover.materializations() {
        let producer = region_position(regions, edge.producer())?;
        for consumer in edge.consumers() {
            let consumer = region_position(regions, consumer)?;
            if consumer == producer {
                return Err(AssemblyRefusal::missing(
                    regions[producer].label(),
                    "cover-self-materialization",
                ));
            }
            successors[producer].push(consumer);
            indegree[consumer] = indegree[consumer].saturating_add(1);
        }
    }
    let mut ready: std::collections::BTreeSet<usize> = (0..regions.len())
        .filter(|position| indegree[*position] == 0)
        .collect();
    let mut order = Vec::with_capacity(regions.len());
    while let Some(next) = ready.pop_first() {
        order.push(next);
        for successor in &successors[next] {
            indegree[*successor] = indegree[*successor].saturating_sub(1);
            if indegree[*successor] == 0 {
                ready.insert(*successor);
            }
        }
    }
    if order.len() != regions.len() {
        return Err(AssemblyRefusal::missing(
            COVER_SUBJECT,
            "cover-materialization-cycle",
        ));
    }
    Ok(order)
}

/// Returns the position one occurrence occupies in a cover's placed regions.
fn region_position(
    regions: &[crate::cover::CoverRegion],
    occurrence: &RegionOccurrenceIdentity,
) -> Result<usize, AssemblyRefusal> {
    regions
        .iter()
        .position(|region| region.occurrence() == occurrence)
        .ok_or_else(|| AssemblyRefusal::missing(COVER_SUBJECT, "cover-edge-region-unknown"))
}

/// Builds and target-binds the program one assembly describes, resolving the
/// request's own lowering first.
///
/// The compile path resolves lowering once per portfolio and hands it down, so
/// this convenience exists for tests of the assembler alone.
#[cfg(test)]
pub(crate) fn build_kernel_program(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    assembly: &CoverAssembly,
) -> Result<KernelProgram, ProgramError> {
    let lowering = resolve_program_lowering(semantic, request)?;
    build_cover_kernel_program_with_lowering(semantic, request, assembly, &lowering)
}

/// Builds the verified kernel program one cover assembles into.
pub(crate) fn build_cover_kernel_program_with_lowering(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    assembly: &CoverAssembly,
    lowering: &ResolvedLowering,
) -> Result<KernelProgram, ProgramError> {
    let core = build_cover_core(semantic, request, assembly, lowering)?;
    let program = KernelProgram {
        target_profile: request.target_profile().clone(),
        core,
    };
    verify_kernel_program_layers(&program, request, assembly.regions())?;
    Ok(program)
}

/// Assembles the shared verified program of one cover, of any region count.
///
/// Every structural obligation — complete disjoint coverage of the semantic
/// graph, a unique writer per materialized value, the data dependency behind
/// each cross-stage read, temporary initialization and lifetimes, the aliasing
/// contract, the split-reduction contributor coverage, and named-output
/// coverage — is proven by [`KernelProgramBuilder::build`], not re-implemented
/// here. This declares them; it does not decide them.
fn build_cover_core(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    assembly: &CoverAssembly,
    lowering: &ResolvedLowering,
) -> Result<VerifiedKernelProgram, ProgramError> {
    if assembly.regions.len() != assembly.stages.len() {
        return Err(ProgramError::Structure {
            rule: "assembly-stage-cardinality",
        });
    }
    // The declared program interface, in declaration order. Checked request
    // subjects retain the association from each regional access to this
    // interface, and stage accesses bind to kernel buffers positionally, so
    // reordering here would silently bind buffers to the wrong tensors.
    let inputs: Vec<(InputKey, Shape, Vec<SourcedExtent>)> = semantic
        .inputs()
        .map(|input| {
            let sourced = semantic
                .shape(input.value())
                .map_err(|_| ProgramError::Structure {
                    rule: "program-input-unshaped",
                })?;
            let extents: Vec<SourcedExtent> = sourced.extents().collect();
            // Symbolic axes occupy a zero static extent so the program value
            // does not bake a live binding. The ABI byte formula below names
            // `InputExtent` for those axes instead.
            let dims: Vec<u64> = extents
                .iter()
                .map(|extent| extent.as_static().map_or(0, tiler_ir::shape::Extent::get))
                .collect();
            let shape = Shape::try_from_dims(dims).map_err(|_| ProgramError::Structure {
                rule: "program-input-rank",
            })?;
            Ok((input.key().clone(), shape, extents))
        })
        .collect::<Result<_, ProgramError>>()?;
    let internal_elements = assembly
        .internals
        .iter()
        .map(|value| shape_elements(&value.shape))
        .collect::<Result<Vec<_>, ProgramError>>()?;

    // The program's own carrier, taken from the contract this target compiles
    // under. The request boundary requires that contract's arithmetic to be the
    // program's own, so this is the width every declared value carries — and a
    // width with no carrier is refused here rather than sized as some other.
    let Some(carrier) = BoundedCarrier::of(request.numerical_contract().arithmetic) else {
        return Err(ProgramError::Structure {
            rule: "program-carrier-arithmetic",
        });
    };
    let mut builder = open_core_builder(semantic, request)?;
    let abi = declare_host_abi(
        &mut builder,
        carrier,
        &inputs
            .iter()
            .map(|(key, _, extents)| (key.clone(), extents.as_slice()))
            .collect::<Vec<_>>(),
        &internal_elements,
    )?;
    // Program-owned storage for every value the program materializes for
    // itself, then the externally bound storage of each declared input.
    let mut internal_storage = Vec::with_capacity(assembly.internals.len());
    for elements in &internal_elements {
        internal_storage.push(builder.push_allocation(storage(
            carrier,
            byte_count(carrier, *elements)?,
            AllocationOwnership::Program,
        ))?);
    }
    let mut input_views = Vec::with_capacity(inputs.len());
    for (key, shape, extents) in &inputs {
        let elements =
            extents
                .iter()
                .try_fold(1_u64, |product, extent| match extent.as_static() {
                    Some(static_extent) => {
                        product
                            .checked_mul(static_extent.get())
                            .ok_or(ProgramError::Storage {
                                rule: "element-count-overflow",
                            })
                    }
                    None => Ok(0),
                })?;
        let external = builder.push_allocation(storage(
            carrier,
            byte_count(carrier, elements)?,
            AllocationOwnership::External,
        ))?;
        let input =
            builder.push_value(program_input(carrier, key.clone(), shape.clone()), external)?;
        input_views.push(builder.push_whole_view(input)?);
    }
    let mut internal_values = Vec::with_capacity(assembly.internals.len());
    for (value, allocation) in assembly.internals.iter().zip(&internal_storage) {
        internal_values.push(builder.push_value(
            internal(carrier, value.role, value.shape.clone()),
            *allocation,
        )?);
    }
    let mut internal_views = Vec::with_capacity(internal_values.len());
    for value in &internal_values {
        internal_views.push(builder.push_whole_view(*value)?);
    }

    let mut pushed = Vec::with_capacity(assembly.stages.len());
    for (stage, region) in assembly.stages.iter().zip(&assembly.regions) {
        let mut accesses = Vec::with_capacity(stage.bindings.len());
        let Some((written, read_bindings)) = stage.bindings.split_last() else {
            return Err(ProgramError::Structure {
                rule: "assembly-stage-bindings",
            });
        };
        for binding in read_bindings {
            accesses.push(read(
                view_of(*binding, &input_views, &internal_views)?,
                bytes_of(*binding, &abi)?,
            ));
        }
        accesses.push(write(
            view_of(*written, &input_views, &internal_views)?,
            bytes_of(*written, &abi)?,
        ));
        let launch = HostAbi::launch(&mut builder, region)?;
        pushed.push(builder.push_stage(
            &lower(region)?,
            &covered(&stage.coverage, lowering)?,
            &accesses,
            launch,
        )?);
    }
    for dependency in &assembly.dependencies {
        builder.push_data_dependency(
            *pushed
                .get(dependency.producer)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-dependency-stage",
                })?,
            *pushed
                .get(dependency.consumer)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-dependency-stage",
                })?,
            *internal_values
                .get(dependency.value)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-dependency-value",
                })?,
        )?;
    }
    for split in &assembly.splits {
        builder.push_partial_reduction(tiler_ir::program::PartialReduction {
            producer: *pushed.get(split.producer).ok_or(ProgramError::Structure {
                rule: "assembly-split-stage",
            })?,
            combiner: *pushed.get(split.combiner).ok_or(ProgramError::Structure {
                rule: "assembly-split-stage",
            })?,
            partial: *internal_values
                .get(split.partial)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-split-value",
                })?,
            result: *internal_values
                .get(split.result)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-split-value",
                })?,
            partitions: split.partition.partitions,
            contributors_per_partition: split.partition.contributors_per_partition,
        })?;
    }
    for copy in &assembly.copies {
        builder.push_publishing_copy(tiler_ir::program::PublishingCopy {
            source_stage: *pushed
                .get(copy.source_stage)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-publishing-copy-stage",
                })?,
            publisher: *pushed.get(copy.publisher).ok_or(ProgramError::Structure {
                rule: "assembly-publishing-copy-stage",
            })?,
            source: *internal_values
                .get(copy.source)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-publishing-copy-value",
                })?,
            published: *internal_values
                .get(copy.published)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-publishing-copy-value",
                })?,
        })?;
    }
    for realization in &assembly.staged {
        builder.push_staged_realization(tiler_ir::program::StagedRealization {
            producer: *pushed
                .get(realization.producer)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-staged-realization-stage",
                })?,
            consumer: *pushed
                .get(realization.consumer)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-staged-realization-stage",
                })?,
            handed: *internal_values
                .get(realization.handed)
                .ok_or(ProgramError::Structure {
                    rule: "assembly-staged-realization-value",
                })?,
            // The occurrence the coverage records name, read through the same
            // `OccurrenceLowering` that mints them, so a declaration can never
            // continue an occurrence some other receipt is about.
            occurrence: lowering
                .occurrence(realization.occurrence)
                .map(crate::lowering::OccurrenceLowering::covered_occurrence)
                .ok_or(ProgramError::Structure {
                    rule: "refinement-coverage-missing",
                })?
                .occurrence(),
        })?;
    }
    for (key, value) in &assembly.outputs {
        builder.push_output(
            key.clone(),
            *internal_values.get(*value).ok_or(ProgramError::Structure {
                rule: "assembly-output-value",
            })?,
        )?;
    }
    declare_routing_commit(&mut builder)?;
    finish_core(builder)
}

/// Returns the declared view one binding addresses.
fn view_of(
    binding: AssemblyBinding,
    inputs: &[ViewId],
    internals: &[ViewId],
) -> Result<ViewId, ProgramError> {
    match binding {
        AssemblyBinding::Input(ordinal) => inputs.get(ordinal).copied(),
        AssemblyBinding::Internal(value) => internals.get(value).copied(),
    }
    .ok_or(ProgramError::Structure {
        rule: "assembly-binding-value",
    })
}

/// Returns the ABI byte expression of the value one binding addresses.
fn bytes_of(binding: AssemblyBinding, abi: &HostAbi) -> Result<AbiExprId, ProgramError> {
    match binding {
        AssemblyBinding::Input(ordinal) => abi.input_bytes.get(ordinal).copied(),
        AssemblyBinding::Internal(value) => abi.internal_bytes.get(value).copied(),
    }
    .ok_or(ProgramError::Structure {
        rule: "assembly-binding-bytes",
    })
}

/// The ABI quantities named by programs in the bounded governed profile.
///
/// A static axis is an `UnsignedLiteral`. A symbolic input axis is an
/// `InputExtent` root that resolves at `LiveDevicePreflight` from the same
/// bound fact range and launch evaluation already consume. The live value is
/// never folded into a literal here: that would specialize the program on a
/// binding.
/// The `input_bytes` run is per declared input, in declaration order, because a
/// contraction's two operands have different extents and therefore different
/// accessible ranges; `internal_bytes` is per value the program materializes for
/// itself, in the order [`CoverAssembly`] mints them. Both are indexed by the
/// binding that names them rather than searched, so a program that grew a value
/// nothing declared bytes for fails at the lookup instead of sizing a stage by
/// whichever entry happened to be first.
#[derive(Clone, Debug)]
struct HostAbi {
    input_bytes: Vec<AbiExprId>,
    internal_bytes: Vec<AbiExprId>,
}

impl HostAbi {
    /// Returns the launch one stage declares, read from the schedule it lowers.
    ///
    /// **Both quantities come from the schedule, and both used to come from
    /// somewhere else.** The width was one literal `1` shared by every stage of a
    /// program, and the grid was whichever declared element count happened to
    /// equal the region's work items — the output count for a reduction, the
    /// contributor count for a pointwise stage. Both were true of every region
    /// that runs one independent invocation per result element, and both are
    /// false for a cooperative one: a single-workgroup tree launches one
    /// invocation *per participant* inside one workgroup, so its work items are
    /// the participant count and its width is too, while its output count is one.
    ///
    /// The effect was not a wrong dispatch — `verify_entry` below and the
    /// shared kernel-program builder each prove the declared launch against the
    /// schedule, so the whole compilation failed as invalid compiler output the
    /// first time a tree reached a kernel program. Deriving the declaration from
    /// the same schedule those proofs compare against is what makes the agreement
    /// structural instead of coincidental.
    ///
    /// **What this gives up, stated rather than hidden.** The grid was previously
    /// an ABI *expression* over a declared element count, which is the shape a
    /// dynamic-shape subject would need — the count would become an `InputExtent`
    /// root resolving at live-device preflight and the launch would follow it.
    /// Nothing is lost today, because this profile's shapes are static and every
    /// declared extent is already an `UnsignedLiteral`; a dynamic subject will
    /// need the launch to be a formula over the *schedule's* own extents rather
    /// than a re-use of an operand's, which is the same derivation this reads.
    ///
    /// The arena deduplicates by content, so stages sharing a launch declare one
    /// node for each of its quantities.
    fn launch(
        builder: &mut KernelProgramBuilder,
        scheduled: &VerifiedScheduledRegion,
    ) -> Result<StageLaunch, ProgramError> {
        let schedule = &scheduled.region().schedule;
        Ok(StageLaunch {
            grid_threads: builder.push_abi_root(AbiRoot::UnsignedLiteral(schedule.work_items))?,
            threads_per_workgroup: builder.push_abi_root(AbiRoot::UnsignedLiteral(u64::from(
                schedule.threads_per_workgroup,
            )))?,
        })
    }
}

/// Declares the ABI arena and applicability guard of one bounded-profile program.
///
/// The arena is deduplicated by content inside the builder, so declaring the
/// same formula at several use sites yields one node — two values of equal
/// extent therefore share one byte expression rather than declaring a second
/// that nothing keeps in agreement with the first. Operands always precede their
/// use, which is the arena's acyclicity invariant.
fn declare_host_abi(
    builder: &mut KernelProgramBuilder,
    carrier: BoundedCarrier,
    inputs: &[(InputKey, &[SourcedExtent])],
    internal_elements: &[u64],
) -> Result<HostAbi, ProgramError> {
    // The element byte width every accessible range scales by, taken from the
    // program's own carrier: a `bf16` program's accessible byte counts are half
    // an `f32` program's, and scaling by the wrong width is a range the runtime
    // would bind past the end of the caller's buffer.
    let element_bytes = builder.push_abi_root(AbiRoot::UnsignedLiteral(carrier.element_bytes()))?;
    let declare_static = |builder: &mut KernelProgramBuilder,
                          counts: &[u64]|
     -> Result<Vec<AbiExprId>, ProgramError> {
        let mut declared = Vec::with_capacity(counts.len());
        for elements in counts {
            let elements = builder.push_abi_root(AbiRoot::UnsignedLiteral(*elements))?;
            declared.push(builder.push_abi_binary(
                AbiBinaryOp::CheckedMultiply,
                element_bytes,
                elements,
            )?);
        }
        Ok(declared)
    };
    let mut input_bytes = Vec::with_capacity(inputs.len());
    for (key, extents) in inputs {
        let count = if let Some(product) = static_extent_product(extents) {
            // The historical static spelling: one literal product, not a
            // chain of per-axis ones. Changing it would move every existing
            // program identity.
            builder.push_abi_root(AbiRoot::UnsignedLiteral(product))?
        } else {
            let mut count = builder.push_abi_root(AbiRoot::UnsignedLiteral(1))?;
            for (axis, extent) in extents.iter().enumerate() {
                let axis = Axis::new(u32::try_from(axis).map_err(|_| ProgramError::Structure {
                    rule: "program-input-rank",
                })?);
                let factor = match extent {
                    SourcedExtent::Static(static_extent) => {
                        builder.push_abi_root(AbiRoot::UnsignedLiteral(static_extent.get()))?
                    }
                    SourcedExtent::Symbol(_) => builder.push_abi_root(AbiRoot::InputExtent {
                        key: key.clone(),
                        axis,
                    })?,
                    _ => {
                        return Err(ProgramError::Structure {
                            rule: "program-input-extent-kind",
                        });
                    }
                };
                count = builder.push_abi_binary(AbiBinaryOp::CheckedMultiply, count, factor)?;
            }
            count
        };
        input_bytes.push(builder.push_abi_binary(
            AbiBinaryOp::CheckedMultiply,
            element_bytes,
            count,
        )?);
    }
    let internal_bytes = declare_static(builder, internal_elements)?;
    // The bounded profile admits every governed target unconditionally, so the
    // guard is a constant. It is still declared rather than assumed, because a
    // program identity blind to its guard is the hazard ADR 0072 names.
    let guard = builder.push_abi_root(AbiRoot::BooleanLiteral(true))?;
    builder.applicability_guard(guard)?;
    Ok(HostAbi {
        input_bytes,
        internal_bytes,
    })
}

/// Declares the routing-commit lifecycle every compiled program shares.
///
/// Fallback to another plan is admitted only while nothing is committed. The
/// shared builder proves that rule rather than trusting it, so this states an
/// intent instead of re-deriving a policy.
fn declare_routing_commit(builder: &mut KernelProgramBuilder) -> Result<(), ProgramError> {
    for (from, to, fallback_permitted) in [
        (
            RoutingCommitState::Preflight,
            RoutingCommitState::Committed,
            true,
        ),
        (
            RoutingCommitState::Committed,
            RoutingCommitState::Executing,
            false,
        ),
        (
            RoutingCommitState::Executing,
            RoutingCommitState::Published,
            false,
        ),
    ] {
        builder.push_routing_commit_transition(RoutingCommitTransition {
            from,
            to,
            fallback_permitted,
        })?;
    }
    Ok(())
}

/// Opens a shared program builder bound to the request's exact semantic program.
fn open_core_builder(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
) -> Result<KernelProgramBuilder, ProgramError> {
    if semantic.semantic_identity() != request.semantic_identity() {
        return Err(ProgramError::Structure {
            rule: "semantic-request-binding",
        });
    }
    Ok(KernelProgramBuilder::new(semantic)?)
}

#[cfg(test)]
fn resolve_program_lowering(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
) -> Result<ResolvedLowering, ProgramError> {
    if semantic.semantic_identity() != request.semantic_identity() {
        return Err(ProgramError::Structure {
            rule: "semantic-request-binding",
        });
    }
    crate::lowering::resolve_lowering(semantic, request).map_err(|_| ProgramError::Structure {
        rule: "refinement-coverage-resolution",
    })
}

fn finish_core(builder: KernelProgramBuilder) -> Result<VerifiedKernelProgram, ProgramError> {
    builder.build().map_err(|error| {
        error.diagnostics().first().copied().map_or(
            ProgramError::Structure {
                rule: "core-verification",
            },
            ProgramError::CoreVerification,
        )
    })
}

/// Lowers one verified scheduled region to the kernel its stage dispatches.
fn lower(scheduled: &VerifiedScheduledRegion) -> Result<VerifiedKernel, ProgramError> {
    lower_structured_kernel(scheduled).map_err(|_| ProgramError::Structure {
        rule: "schedule-verification",
    })
}

/// Projects the coverage a stage declares onto the receipts that prove it.
///
/// The occurrence and its evidence are read from one `OccurrenceLowering`, so
/// they cannot be paired wrongly here: a member with no resolved lowering has no
/// receipt and produces a refusal rather than a bare occurrence. That is the
/// compiler half of the fail-closed rule — a proof gap never reaches
/// `push_stage` in a form the IR could accept.
///
/// **Only first-stage atoms yield a record, and that is a projection rather than
/// a filter that discards evidence.** `tiler_ir::program`'s coverage is keyed on
/// [`tiler_ir::program::SemanticOccurrence`] and refuses the same occurrence
/// twice across a program, because what it proves is that every operation is
/// computed exactly once with a refinement receipt behind it — a property of the
/// occurrence, not of whichever dispatch continues it. A later stage's work is
/// accounted for at program scope by the declaration that admits it as an
/// uncovering pass, which is where the chain, not the coverage, is proven.
fn covered(
    members: &[SemanticStage],
    lowering: &ResolvedLowering,
) -> Result<Vec<CoveredOccurrence>, ProgramError> {
    members
        .iter()
        .filter(|atom| atom.is_first())
        .map(|atom| {
            lowering
                .occurrence(atom.member())
                .map(crate::lowering::OccurrenceLowering::covered_occurrence)
                .ok_or(ProgramError::Structure {
                    rule: "refinement-coverage-missing",
                })
        })
        .collect()
}

/// Declares an allocation backing values of one program's carrier.
///
/// The alignment is the carrier's rather than a constant of its own: an
/// allocation has to be at least as aligned as every value placed in it, and
/// deriving both from one carrier is what keeps that true without a check.
fn storage(
    carrier: BoundedCarrier,
    capacity_bytes: u64,
    ownership: AllocationOwnership,
) -> AllocationSpec {
    AllocationSpec {
        capacity_bytes,
        alignment: carrier.element_guarantee(),
        memory_space: MemorySpace::Device,
        ownership,
    }
}

fn program_input(
    carrier: BoundedCarrier,
    key: tiler_ir::semantic::InputKey,
    shape: Shape,
) -> MaterializedValueSpec {
    MaterializedValueSpec {
        origin: MaterializedOrigin::ProgramInput { key },
        role: ValueRole::Input,
        shape,
        storage_scalar: carrier.storage,
        encoding: StorageEncoding::Unpacked,
        element_type: carrier.element_type,
        alignment: carrier.element_requirement(),
        memory_space: MemorySpace::Device,
    }
}

fn internal(carrier: BoundedCarrier, role: ValueRole, shape: Shape) -> MaterializedValueSpec {
    MaterializedValueSpec {
        origin: MaterializedOrigin::Internal,
        role,
        shape,
        storage_scalar: carrier.storage,
        encoding: StorageEncoding::Unpacked,
        element_type: carrier.element_type,
        alignment: carrier.element_requirement(),
        memory_space: MemorySpace::Device,
    }
}

const fn read(view: ViewId, accessible_bytes: AbiExprId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Read,
        accessible_bytes,
    }
}

const fn write(view: ViewId, accessible_bytes: AbiExprId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Write,
        accessible_bytes,
    }
}

fn byte_count(carrier: BoundedCarrier, elements: u64) -> Result<u64, ProgramError> {
    elements
        .checked_mul(carrier.element_bytes())
        .ok_or(ProgramError::Storage {
            rule: "required-byte-overflow",
        })
}

/// Returns how many elements one declared shape holds.
///
/// Routed through the shared shape authority rather than multiplied here, so an
/// extent product that leaves the 64-bit domain is the same refusal the schedule
/// and program layers already report rather than a wrapped count this module
/// invented.
fn static_extent_product(extents: &[SourcedExtent]) -> Option<u64> {
    extents.iter().try_fold(1_u64, |product, extent| {
        product.checked_mul(extent.as_static()?.get())
    })
}

fn shape_elements(shape: &Shape) -> Result<u64, ProgramError> {
    tiler_ir::schedule::element_count(shape).map_err(|_| ProgramError::Storage {
        rule: "element-count-overflow",
    })
}

/// Verifies the compiler-owned layers of one program against its shared core.
///
/// The shared core is already verified by construction — including its ABI
/// arena, guard, entry ABI and routing-commit contract — so this proves only
/// what a target-neutral program cannot: the request and target binding, the
/// request's budgets, the compile-time truth of the guard, and the agreement
/// between each stage's declared launch and the scheduled region it was planned
/// from.
pub(crate) fn verify_kernel_program_layers(
    program: &KernelProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<(), ProgramError> {
    if scheduled.is_empty() || program.core.stages().len() != scheduled.len() {
        return Err(ProgramError::Structure {
            rule: "cardinality",
        });
    }
    if scheduled
        .iter()
        .any(|region| !region.matches_request(request))
    {
        return Err(ProgramError::Structure {
            rule: "request-subject",
        });
    }
    if &program.target_profile != request.target_profile()
        || scheduled
            .iter()
            .any(|region| region.target_profile() != &program.target_profile)
    {
        return Err(ProgramError::Structure {
            rule: "target-profile",
        });
    }
    let values = verify_host_contract(program, request)?;
    for (position, (stage, region)) in program.core.stages().zip(scheduled).enumerate() {
        verify_entry(stage, region, position, &values)?;
    }
    // Fallback before commit is what makes preflight rejection recoverable, so
    // a governed compilation states it. The complementary rule — that no later
    // step permits fallback — is proven by the shared builder, which rejects
    // such a step as `RoutingCommitFallbackAfterCommit`.
    if !program
        .core
        .routing_commit_contract()
        .first()
        .is_some_and(|first| first.fallback_permitted)
    {
        return Err(ProgramError::Routing {
            rule: "pre-commit-fallback",
        });
    }
    Ok(())
}

fn verify_host_contract(
    program: &KernelProgram,
    request: &VerifiedTargetRequest,
) -> Result<Vec<AbiValue>, ProgramError> {
    let expressions = program.core.abi_expressions();
    if expressions.len()
        > usize::try_from(request.budgets().host_expression_nodes).map_err(|_| {
            ProgramError::Structure {
                rule: "host-expression-budget",
            }
        })?
    {
        return Err(ProgramError::Structure {
            rule: "host-expression-budget",
        });
    }
    if program.core.values().len()
        > usize::try_from(request.budgets().buffers).map_err(|_| ProgramError::Storage {
            rule: "buffer-budget",
        })?
    {
        return Err(ProgramError::Storage {
            rule: "buffer-budget",
        });
    }
    // The assembled region count, checked here beside its two siblings because
    // the three bound the same object — the plan this request produced — while
    // `check_program_budgets` can bound only what the *declaration* implies.
    //
    // **Defence in depth rather than a live gate, exactly as `buffer-budget`
    // above is.** The boundary's actual is four dispatches per declared output,
    // which is the widest chain the recognizer can spell for one output, and the
    // outputs' chains are disjoint; so no request this boundary admits can
    // assemble past it on a plan of this build's own making, and the check
    // firing would mean the recognizer gained a wider chain than the derivation
    // enumerates. That is precisely the state worth failing closed on, because
    // this refusal reaches the caller as `InvalidCompilerOutput` — the compiler
    // reporting its own defect, which is the right report for exactly this cause
    // and the wrong one for an over-large request, which is why the boundary and
    // not this site is what bounds a declaration.
    //
    // Proven able to refuse rather than assumed: with the boundary's `regions`
    // check disabled and a stated budget of three, the four-dispatch chain
    // reaches here and reports
    // `InvalidCompilerOutput(Program(Structure { rule: "region-budget" }))`.
    //
    // A stage and a scheduled region are the same count by
    // `verify_kernel_program_layers`' `cardinality` rule, which is what lets a
    // budget named for regions be enforced against the stage list here. It is
    // deliberately *not* the cover's `region_count()`: a split fold is one cover
    // region and three dispatches, and it is the dispatches the budget's own
    // derivation enumerates.
    if program.core.stages().len()
        > usize::try_from(request.budgets().regions).map_err(|_| ProgramError::Structure {
            rule: "region-budget",
        })?
    {
        return Err(ProgramError::Structure {
            rule: "region-budget",
        });
    }
    let values = evaluate_expressions(expressions)?;
    if values.get(position(program.core.applicability_guard())) != Some(&AbiValue::Boolean(true)) {
        return Err(ProgramError::Structure {
            rule: "applicability-guard",
        });
    }
    Ok(values)
}

/// Proves one stage's entry ABI realizes the region it was planned from.
///
/// The shared program already proves the structural half — that each access
/// binds the view its kernel buffer names, that its accessible range equals
/// that view's window, and that the declared workgroup width is the bound
/// kernel's. What only a compilation can add is the *planning* half: the region
/// this stage was scheduled from, whose launch extent and numerical realization
/// the entry must not contradict.
fn verify_entry(
    stage: StageRef<'_>,
    scheduled: &VerifiedScheduledRegion,
    position_of_stage: usize,
    values: &[AbiValue],
) -> Result<(), ProgramError> {
    let index = u8::try_from(position_of_stage).map_err(|_| ProgramError::Structure {
        rule: "stage-id-overflow",
    })?;
    let stage_id = StageId(index);
    if stage.kernel().requirements() != scheduled.requirements()
        || stage.kernel().numerical() != scheduled.region().index.numerical
    {
        return Err(ProgramError::Abi {
            rule: "entry-contract",
            stage: stage_id,
        });
    }
    let launch = stage.launch();
    if values.get(position(launch.grid_threads))
        != Some(&AbiValue::Unsigned(scheduled.region().schedule.work_items))
        || values.get(position(launch.threads_per_workgroup))
            != Some(&AbiValue::Unsigned(u64::from(
                scheduled.region().schedule.threads_per_workgroup,
            )))
    {
        return Err(ProgramError::Abi {
            rule: "launch-expression",
            stage: stage_id,
        });
    }
    // The shared layer permits a partial view; a bounded-profile entry binds a
    // whole materialized value, so its accessible range is that value's bytes.
    for access in stage.accesses() {
        let expected = AbiValue::Unsigned(access.view().value().required_bytes());
        if values.get(position(access.accessible_bytes())) != Some(&expected) {
            return Err(ProgramError::Abi {
                rule: "binding",
                stage: stage_id,
            });
        }
    }
    Ok(())
}

/// Converts a checked ABI arena ordinal into a host index.
fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

#[cfg(test)]
pub(crate) fn build_artifact_plan(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    assembly: &CoverAssembly,
    kernels: &[VerifiedKernel],
    program: &KernelProgram,
    providers: Vec<LoweringProviderIdentity>,
) -> Result<ArtifactConstructionPlan, ProgramError> {
    let lowering = resolve_program_lowering(semantic, request)?;
    if providers != lowering.providers() {
        return Err(ProgramError::Structure {
            rule: "artifact-provider-coverage",
        });
    }
    drop(providers);
    build_artifact_plan_with_lowering(semantic, request, assembly, kernels, program, &lowering)
}

pub(crate) fn build_artifact_plan_with_lowering(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    assembly: &CoverAssembly,
    kernels: &[VerifiedKernel],
    program: &KernelProgram,
    lowering: &ResolvedLowering,
) -> Result<ArtifactConstructionPlan, ProgramError> {
    let scheduled = assembly.regions();
    verify_artifact_refinements(semantic, request, assembly, kernels, program, lowering)?;
    // Lowering provenance is re-derived from the request's own installed
    // registry rather than trusted from the caller, so a plan cannot claim a
    // provider the registry never resolved for this program.
    let expected_providers =
        crate::lowering::resolve_capabilities(semantic, request).map_err(|_| {
            ProgramError::Structure {
                rule: "artifact-provider-resolution",
            }
        })?;
    let providers = lowering.providers();
    if providers.is_empty() || providers != expected_providers {
        return Err(ProgramError::Structure {
            rule: "artifact-provider-coverage",
        });
    }
    Ok(ArtifactConstructionPlan {
        semantic_identity: request.semantic_identity().clone(),
        numerical_contract_key: request.numerical_contract().key,
        numerical_realizations: program
            .core
            .stages()
            .map(|stage| stage.kernel().numerical())
            .collect(),
        target_profile: program.target_profile.clone(),
        // Read from the authority that decides feasibility rather than composed
        // here. It is a constant and not a function of the request because the
        // rules do not vary by target: `CheckedTargetProfile::assess` applies
        // exactly these rules to every profile, so a per-target derivation would
        // imply a variation that cannot occur.
        feasibility_rule_set: GOVERNED_FEASIBILITY_RULE_SET,
        entry_regions: program
            .core
            .stages()
            .map(|stage| stage.kernel().scheduled_region())
            .collect(),
        entry_deferred_predicates: scheduled
            .iter()
            .enumerate()
            .flat_map(|(entry, region)| {
                region
                    .admission()
                    .deferred()
                    .into_iter()
                    .flat_map(move |deferred| {
                        deferred.predicates().iter().cloned().map(move |predicate| {
                            EntryDeferredPredicate {
                                entry: u32::try_from(entry)
                                    .expect("program stage counts are bounded below u32"),
                                predicate,
                            }
                        })
                    })
            })
            .collect(),
        applicability_guard: program.core.applicability_guard(),
        lowering_providers: providers,
        request_subject: request.subject().clone(),
        verified_program: program.clone(),
        verified_schedules: scheduled.to_vec(),
        verified_kernels: kernels.to_vec(),
    })
}

/// Proves the artifact's inputs are the exact refinements of one request.
///
/// The expected program is re-derived through the **same route the build path
/// took** — one call to [`build_cover_kernel_program_with_lowering`] over the
/// same [`CoverAssembly`] — rather than through a second description of what a
/// cover assembles into. The retired three-way match over the scheduled slice
/// was exactly that second description, and it embodied the duplicate-derivation
/// hazard twice: a change to either assembler had to be mirrored in the receipt
/// path, and nothing kept the two in agreement.
fn verify_artifact_refinements(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    assembly: &CoverAssembly,
    kernels: &[VerifiedKernel],
    program: &KernelProgram,
    lowering: &ResolvedLowering,
) -> Result<(), ProgramError> {
    let scheduled = assembly.regions();
    if semantic.semantic_identity() != request.semantic_identity() {
        return Err(ProgramError::Structure {
            rule: "semantic-request-binding",
        });
    }
    if scheduled.is_empty()
        || scheduled
            .iter()
            .any(|region| !region.matches_request(request))
        || kernels.len() != scheduled.len()
    {
        return Err(ProgramError::Structure {
            rule: "artifact-refinement-cardinality",
        });
    }
    for (region, kernel) in scheduled.iter().zip(kernels) {
        let expected = lower(region).map_err(|_| ProgramError::Structure {
            rule: "artifact-schedule-refinement",
        })?;
        if kernel != &expected {
            return Err(ProgramError::Structure {
                rule: "artifact-kernel-refinement",
            });
        }
    }
    let expected_program =
        build_cover_kernel_program_with_lowering(semantic, request, assembly, lowering)?;
    if program != &expected_program {
        return Err(ProgramError::Structure {
            rule: "artifact-program-refinement",
        });
    }
    assert_kernels_match_program(request, scheduled, program, kernels)?;
    // The published interface is the declared one, key for key and in order.
    //
    // Both halves matter and they are different claims. The count catches an
    // assembly that published a subset or invented an entry; the ordered
    // key-by-key comparison catches one that published the right set under a
    // permuted interface, which is the failure a program declaring several
    // ordered named outputs makes possible and which no count can see. This
    // check used to additionally require both counts to be one — the second of
    // the two arity guards `admit-ordered-multi-output-programs-at-the-compiler-
    // request-boundary` relaxed — and widening it here is what keeps the receipt
    // path checking the interface rather than merely its size.
    if program.core.outputs().len() != semantic.output_count()
        || semantic.output_count() == 0
        || program
            .core
            .outputs()
            .zip(semantic.outputs())
            .any(|(named, declared)| named.key() != declared.key())
    {
        return Err(ProgramError::Structure {
            rule: "semantic-output-coverage",
        });
    }
    if &program.target_profile != request.target_profile()
        || program
            .core
            .stages()
            .any(|stage| stage.kernel().numerical() != request.numerical_contract().realization())
    {
        return Err(ProgramError::Structure {
            rule: "artifact-numerical-realization",
        });
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn verify_artifact_plan(
    plan: &ArtifactConstructionPlan,
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    assembly: &CoverAssembly,
    kernels: &[VerifiedKernel],
    program: &KernelProgram,
    providers: Vec<LoweringProviderIdentity>,
) -> Result<(), ProgramError> {
    let lowering = resolve_program_lowering(semantic, request)?;
    if providers != lowering.providers() {
        return Err(ProgramError::Structure {
            rule: "artifact-provider-coverage",
        });
    }
    drop(providers);
    verify_artifact_plan_with_lowering(
        plan, semantic, request, assembly, kernels, program, &lowering,
    )
}

pub(crate) fn verify_artifact_plan_with_lowering(
    plan: &ArtifactConstructionPlan,
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    assembly: &CoverAssembly,
    kernels: &[VerifiedKernel],
    program: &KernelProgram,
    lowering: &ResolvedLowering,
) -> Result<(), ProgramError> {
    let expected =
        build_artifact_plan_with_lowering(semantic, request, assembly, kernels, program, lowering)?;
    if plan != &expected {
        return Err(ProgramError::Structure {
            rule: "artifact-receipt",
        });
    }
    Ok(())
}

/// Evaluates every node of the program's ABI arena in arena order.
///
/// The shared evaluator is the authority for what each node means, so this
/// function owns only the mapping from its typed failures onto this crate's
/// rules. Every node is evaluated rather than only the roots the entries name:
/// the program layer proves every node is *reachable* from a use site, and this
/// proves every node is *evaluable* at compile time, which the bounded profile
/// requires but a program in general does not.
///
/// The bound environment is empty and reaches only `CompileProfile`: the
/// bounded profile's graph is entirely literals, so binding a device fact here
/// would be claiming an availability this stage does not have.
fn evaluate_expressions(expressions: &[ExprNode]) -> Result<Vec<AbiValue>, ProgramError> {
    let facts = AbiFacts::new(AvailabilityPhase::CompileProfile, Vec::new(), Vec::new());
    let mut values = Vec::with_capacity(expressions.len());
    for position in 0..expressions.len() {
        let root = u32::try_from(position).map_err(|_| ProgramError::Structure {
            rule: "host-expression-budget",
        })?;
        values.push(
            abi_evaluate(expressions, root, &facts)
                .map_err(|error| host_expression_error(&error, HostExprId(root)))?,
        );
    }
    Ok(values)
}

/// Maps one shared evaluation failure onto this crate's stable rule vocabulary.
///
/// The match is exhaustive over the wildcard-free arms it can name; the shared
/// error is `#[non_exhaustive]`, so a variant added upstream reaches the final
/// arm and reports `evaluation` rather than being silently reclassified as one
/// of the specific rules.
fn host_expression_error(error: &AbiEvaluationError, at: HostExprId) -> ProgramError {
    let rule = match error {
        AbiEvaluationError::Overflow { .. } => "overflow",
        AbiEvaluationError::UnboundInputExtent { .. }
        | AbiEvaluationError::UnboundTargetProperty { .. } => "operand",
        _ => "evaluation",
    };
    ProgramError::HostExpression {
        rule,
        expression: at,
    }
}

/// Proves the separately retained kernels are exactly the ones the program binds.
///
/// The shared program already holds each stage's verified kernel, so this
/// checks the compilation product's parallel kernel list against that binding
/// and against the schedules it claims to refine.
pub(crate) fn assert_kernels_match_program(
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
    program: &KernelProgram,
    kernels: &[VerifiedKernel],
) -> Result<(), ProgramError> {
    if kernels.len() != scheduled.len()
        || kernels.len() != program.core.stages().len()
        || scheduled
            .iter()
            .any(|region| !region.matches_request(request))
    {
        return Err(ProgramError::Structure {
            rule: "kernel-entry-cardinality",
        });
    }
    for ((region, kernel), stage) in scheduled.iter().zip(kernels).zip(program.core.stages()) {
        if lower(region)? != *kernel || stage.kernel() != kernel {
            return Err(ProgramError::Structure {
                rule: "kernel-schedule-refinement",
            });
        }
    }
    Ok(())
}

/// Requires every declared output to carry a width this build can plan.
///
/// **It was `f32` exactly, and the admitted set is now the recognizer's own.**
/// The check states an invariant of the *recognized* program — the compiler is
/// about to build regions for these outputs, so a width no scalar program spells
/// is invalid compiler output rather than a caller error — and while recognition
/// admitted one width the two statements coincided. They no longer do, so the
/// set is asked of [`crate::request::recognized_arithmetic`], which is the one
/// place it is stated. Restating `f32` here would refuse, as a compiler defect,
/// exactly the programs recognition had just admitted; it did, and this is the
/// downstream `f32` assumption the recognizer's own wall had been hiding.
///
/// What it deliberately does *not* check is uniformity across the program. That
/// is recognition's, which refuses a mixed-width program under `dtype-uniform`
/// before any output reaches here.
pub(crate) fn verify_semantic_output_type(program: &SemanticProgram) -> Result<(), ProgramError> {
    if program.output_count() == 0
        || program.outputs().any(|output| {
            program.value(output.value()).map_or(true, |value| {
                crate::request::recognized_arithmetic(value.resolved_type()).is_none()
            })
        })
    {
        return Err(ProgramError::Structure {
            rule: "semantic-output-type",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
