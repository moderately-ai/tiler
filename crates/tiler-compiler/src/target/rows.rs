//! The per-family declaration vocabularies and the canonical rows they mint.
//!
//! One family per fact a profile may state — quantitative capability axes,
//! numerical honourability, dtype dispatchability, evaluation-order
//! preservation, measured cost, and the workgroup-tree width policy — each with
//! its public vocabulary, its stored row, that row's validity rule, and the
//! row's own canonical byte encoding. The order the rows are written in and the
//! separators between families belong to [`super::descriptor`]; a row knows
//! only how to encode itself.

use std::sync::Arc;

use tiler_ir::identity::push_slice;
use tiler_ir::program::abi::{AvailabilityPhase, TargetPropertyQuery};
use tiler_ir::semantic::ResolvedValueType;

use crate::target::ScalarArithmetic;
use crate::target::descriptor::encode_compact_index;
use crate::target::error::TargetProfileBuildError;
use crate::target::feasibility::{CapabilityAxis, SubgroupRealization, SynchronizationRealization};
use crate::target::honourability::{
    DeclaredBehaviour, DimensionBehaviour, FactSourceProvenance, HonouringMeans,
    NumericalDimension, governed_profile_source,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ScalarHonourabilityDeclaration {
    pub(super) subject: ScalarArithmetic,
    pub(super) dimension: NumericalDimension,
    pub(super) behaviour: DimensionBehaviour,
    pub(super) means: HonouringMeans,
    pub(super) source: Arc<FactSourceProvenance>,
}

impl ScalarHonourabilityDeclaration {
    pub(super) fn governed_exact(
        dimension: NumericalDimension,
        behaviour: DimensionBehaviour,
    ) -> Self {
        Self {
            subject: ScalarArithmetic::f32(),
            dimension,
            behaviour,
            means: HonouringMeans::SupportedExactly,
            source: governed_profile_source(),
        }
    }

    pub(super) fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.dimension.admits(self.behaviour) {
            return Err(TargetProfileBuildError::InvalidDimensionBehaviour);
        }
        match &self.means {
            HonouringMeans::SupportedExactly | HonouringMeans::Unsupported => {}
            HonouringMeans::SupportedWithExactEmulation => {
                return Err(TargetProfileBuildError::UnverifiedExactEmulation);
            }
            HonouringMeans::SupportedOnlyUnderDeclaredRelaxation { relaxation } => {
                // The relaxation names a subject rather than a loose
                // (arithmetic, type) pair, so the whole subject is compared in
                // one step. A profile may only condition a declaration on an
                // authorization stated for the same subject it is declaring
                // about; a relaxation naming another subject would make the
                // condition unresolvable against the caller's contract.
                if !relaxation.dimension().admits(relaxation.behaviour())
                    || relaxation.subject() != &self.subject.identity()
                {
                    return Err(TargetProfileBuildError::InvalidRelaxation);
                }
            }
        }
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    pub(super) fn declared(&self) -> DeclaredBehaviour {
        DeclaredBehaviour::new(
            self.dimension,
            self.subject.arithmetic(),
            self.subject.resolved_type().clone(),
            self.behaviour,
            self.means.clone(),
            Arc::clone(&self.source),
        )
    }

    pub(super) fn encode(&self, bytes: &mut Vec<u8>) {
        self.subject.encode(bytes);
        bytes.push(self.dimension.tag());
        self.behaviour.encode(bytes);
        self.means.encode(bytes);
        push_slice(bytes, self.source.canonical_bytes().as_slice());
    }
}

/// Support for the governed KIR index-arithmetic family.
///
/// This is deliberately not a raw integer width. `CompleteU64` means the target
/// supports every unsigned-64 operation that [`tiler_ir::kernel::KernelType::Index`]
/// may emit, rather than merely storing a 64-bit scalar.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexArithmeticSupport {
    /// The governed unsigned-64 index operation family is unsupported.
    Unsupported,
    /// The governed unsigned-64 index operation family is supported completely.
    CompleteU64,
}

impl IndexArithmeticSupport {
    pub(super) const fn bound(self) -> u64 {
        match self {
            Self::Unsupported => 0,
            Self::CompleteU64 => 1,
        }
    }
}

/// Width of a target's device address model.
///
/// This fact does not describe integer arithmetic, buffer length, or launch
/// coordinate delivery. A profile omits it when no applicable authority has
/// established the address model.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeviceAddressWidth {
    /// A 32-bit device address model.
    Bits32,
    /// A 64-bit device address model.
    Bits64,
}

impl DeviceAddressWidth {
    /// Returns the width in bits.
    #[must_use]
    pub const fn bits(self) -> u8 {
        match self {
            Self::Bits32 => 32,
            Self::Bits64 => 64,
        }
    }
}

/// Qualitative ability of a target family to dispatch one exact dtype.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DTypeDispatchability {
    /// The exact dtype can be dispatched.
    Dispatchable,
    /// The exact dtype is explicitly unsupported.
    Unsupported,
}

impl DTypeDispatchability {
    const fn tag(self) -> u8 {
        match self {
            Self::Dispatchable => 0x01,
            Self::Unsupported => 0x02,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DTypeDispatchabilityFact {
    pub(super) resolved_type: ResolvedValueType,
    pub(super) verdict: DTypeDispatchability,
    pub(super) source: Arc<FactSourceProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct QuantitativeCapabilityDeclaration {
    pub(super) axis: CapabilityAxis,
    pub(super) bound: u64,
    pub(super) source: Arc<FactSourceProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct QuantitativeCapabilityQueryDeclaration {
    pub(super) axis: CapabilityAxis,
    pub(super) query: TargetPropertyQuery,
}

impl QuantitativeCapabilityDeclaration {
    pub(super) fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    pub(super) fn encode_source_index(bytes: &mut Vec<u8>, source_index: usize) {
        encode_compact_index(bytes, source_index);
    }
}

impl DTypeDispatchabilityFact {
    pub(super) fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    pub(super) fn encode(&self, bytes: &mut Vec<u8>, source_index: usize) {
        push_slice(bytes, self.resolved_type.canonical_encoding().as_bytes());
        bytes.push(self.verdict.tag());
        encode_compact_index(bytes, source_index);
    }
}

/// Result of an exact dtype-dispatchability lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DTypeDispatchabilityResolution {
    /// An exact declaration admits dispatch.
    Dispatchable,
    /// An exact declaration refuses dispatch.
    Unsupported,
    /// An exact declaration exists, but only from this later phase.
    Deferred {
        /// Earliest phase at which an exact declaration can resolve.
        available_at: AvailabilityPhase,
    },
    /// No exact declaration exists.
    Unknown,
}

/// The arithmetic-rewriting licence a backend translation is granted.
///
/// **Accepted public surface.** The key half of the evaluation-order fact, which
/// Tom accepted on 2026-08-06 under
/// `accept-the-evaluation-order-preservation-target-fact`.
///
/// # Why the key is a licence rather than a backend flag spelling
///
/// The measurement this vocabulary exists to carry is indexed by Metal's
/// `-fmetal-math-mode`, whose three values are `safe`, `relaxed`, and `fast`.
/// Those are one backend driver's option tokens, and a consumer-agnostic profile
/// that named them would have learnt a Metal flag. What the measurement actually
/// attributes the behaviour to is the *licence set the emitted operations carry*:
/// [finding 34](../../../docs/research/apple-targets/numerical-behaviour.md)
/// records the reordering firing exactly where the emitted set carries LLVM's
/// `reassoc`, and names `reassoc` as "the licence that authorizes regrouping".
/// `safe` withholds it; `relaxed` and `fast` both grant it, differing only in
/// `nnan`/`ninf`, which no measured cell attributes an order change to.
///
/// So the two values below cover all three modes without inheriting between
/// them, and a row that separated `relaxed` from `fast` would be a third value
/// here — a build error at every match, never a silent reading of a neighbour's
/// fact.
///
/// This is **not** a caller permission. [`tiler_ir::schedule::NumericalPermission`]
/// states what a caller's contract allows *Tiler* to do; this states what Tiler's
/// emission allows the *backend translator* to do, and ADR 0011's rule that one
/// permission never implies another applies across the two vocabularies too.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BackendArithmeticLicence {
    /// The backend translation is granted no licence to rewrite floating-point
    /// arithmetic.
    Withheld,
    /// The backend translation is licensed to rewrite floating-point arithmetic,
    /// including regrouping a same-operation operand sequence.
    Granted,
}

impl BackendArithmeticLicence {
    /// Returns the stable governed key naming this licence.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Withheld => "arithmetic-rewriting-withheld",
            Self::Granted => "arithmetic-rewriting-granted",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Withheld => 0x01,
            Self::Granted => 0x02,
        }
    }
}

/// Whether a backend translation preserves the evaluation order the emitted
/// program pins.
///
/// **Accepted public surface**, with the licence key above.
///
/// Two valued, and the negative is *statable*, exactly as
/// [`SynchronizationSupport`] is: a target measured to re-serialize a written
/// grouping records that, and a target nobody asked records nothing. Those are
/// different states — a typed refusal and an `Unknown` — and a vocabulary with
/// only a positive spelling would collapse them into one silence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvaluationOrderPreservation {
    /// The backend executes the evaluation order the emitted program names.
    Preserved,
    /// The backend may execute some other legal order than the one emitted.
    NotPreserved,
}

impl EvaluationOrderPreservation {
    /// Returns the stable governed key naming this verdict.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Preserved => "evaluation-order-preserved",
            Self::NotPreserved => "evaluation-order-not-preserved",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Preserved => 0x01,
            Self::NotPreserved => 0x02,
        }
    }
}

/// Result of an evaluation-order-preservation lookup.
///
/// **Accepted public surface**, with the two vocabularies above.
///
/// [`Self::Unknown`] is the fail-closed answer and the overwhelmingly common
/// one: a profile that declares nothing about the property answers it, and a
/// consumer may not read a neighbouring subject's or a neighbouring licence's
/// row in its place. The oracle's refusal class 3 is what consumes it — a plan
/// whose pinned order the backend is permitted to change is refused rather than
/// qualified — so an `Unknown` never becomes an admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationOrderResolution {
    /// An exact declaration states the order is preserved.
    Preserved,
    /// An exact declaration states the order may be changed.
    NotPreserved,
    /// An exact declaration exists, but only from this later phase.
    Deferred {
        /// Earliest phase at which an exact declaration can resolve.
        available_at: AvailabilityPhase,
    },
    /// No exact declaration exists for this subject and licence.
    Unknown,
}

/// One target's evaluation-order-preservation row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EvaluationOrderFact {
    pub(super) subject: ScalarArithmetic,
    pub(super) licence: BackendArithmeticLicence,
    pub(super) preservation: EvaluationOrderPreservation,
    pub(super) source: Arc<FactSourceProvenance>,
}

impl EvaluationOrderFact {
    pub(super) fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    /// The canonical key this row is unique under: the exact scalar subject, the
    /// licence, and the phase. The verdict is deliberately excluded, for the
    /// reason [`TargetProfileBuildError::DuplicateSynchronizationRealization`]
    /// excludes it — a profile stating one subject both preserved and not
    /// preserved has stated a contradiction, and admitting both rows would leave
    /// whichever the sort put first deciding.
    pub(super) fn subject_key(&self) -> (Vec<u8>, u8, AvailabilityPhase) {
        let mut subject = Vec::new();
        self.subject.encode(&mut subject);
        (subject, self.licence.tag(), self.source.phase())
    }

    pub(super) fn encode(&self, bytes: &mut Vec<u8>, source_index: usize) {
        self.subject.encode(bytes);
        bytes.push(self.licence.tag());
        bytes.push(self.preservation.tag());
        encode_compact_index(bytes, source_index);
    }
}

/// One measured machine quantity a target may state a *preference* on.
///
/// **Deliberately not a [`CapabilityAxis`], and the distinction is the whole
/// point of the type.** Every capability axis is a hard bound: silence about one
/// resolves `Unknown`, and an `Unknown` never reaches an executable frontier.
/// `docs/research/program-planning/flash-class-capability-set.md` already
/// eliminated that shape for a bandwidth number and the argument transfers
/// unchanged — a cost row declared as a capability axis would make silence render
/// a profile **unexecutable for a quantity no feasibility predicate reads**,
/// which is the wrong failure direction. Silence about a cost row means *no
/// preference*, never *no plan*, and [`TargetCostRowResolution`] is where that is
/// written down.
///
/// Private, and the public surface is one `declare_*` / `declare_measured_*` pair
/// plus one reader per row, exactly as the quantitative axes are spelled. A
/// second row lands as a variant here plus its own pair, additively.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum CostRow {
    /// Fold steps the device retires at once when it is saturated.
    SaturatedParallelFoldSteps,
}

impl CostRow {
    /// The stable governed key naming this row.
    pub(super) const fn key(self) -> &'static str {
        match self {
            Self::SaturatedParallelFoldSteps => "cost.saturated-parallel-fold-steps",
        }
    }
}

/// One declared cost row, its value, and who vouches for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CostRowFact {
    pub(super) row: CostRow,
    pub(super) value: u64,
    pub(super) source: Arc<FactSourceProvenance>,
}

impl CostRowFact {
    pub(super) fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    pub(super) fn encode(&self, bytes: &mut Vec<u8>, source_index: usize) {
        push_slice(bytes, self.row.key().as_bytes());
        bytes.extend_from_slice(&self.value.to_le_bytes());
        encode_compact_index(bytes, source_index);
    }
}

/// Result of a cost-row lookup.
///
/// **Accepted public surface**, accepted by Tom on 2026-08-07 under
/// `accept-the-measured-cost-row-public-surface`, with the declaration pair and
/// reader.
///
/// [`Self::Unknown`] is the common answer and it means **no preference**, not no
/// plan. A consumer must treat it, and [`Self::Deferred`], as evidence it does not
/// have — never as a refusal, and never as a zero. Nothing is inherited from a
/// neighbouring row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetCostRowResolution {
    /// An exact declaration states this value.
    Declared {
        /// The declared quantity, in the row's own unit.
        value: u64,
    },
    /// An exact declaration exists, but only from this later phase.
    Deferred {
        /// Earliest phase at which an exact declaration can resolve.
        available_at: AvailabilityPhase,
    },
    /// No declaration exists, which is a stated absence of preference.
    Unknown,
}

/// The closed tree-width policy a target must declare to offer the
/// single-workgroup tree.
///
/// **Accepted public surface.** Tom delegated the choice to the coordinator on
/// 2026-08-11 under `gate-the-workgroup-tree-on-an-explicit-qualified-width-policy`.
///
/// One variant, and there is deliberately no omitted/default case and no public
/// numeric cap. A profile that does not declare an accepted policy makes the
/// tree unavailable with a typed reason. The fixed internal `256` stays private
/// to the partition rule this variant names.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorkgroupTreeWidthPolicy {
    /// The existing nearest-admissible-width rule around the fixed internal
    /// value `256`, ties going to the narrower. Qualified by the retained
    /// 2026-08-07 Apple9 partition calibration.
    MeasuredNearestCap256V1,
}

impl WorkgroupTreeWidthPolicy {
    /// Returns the stable governed key naming this policy.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::MeasuredNearestCap256V1 => "workgroup-tree-width.measured-nearest-cap-256.v1",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::MeasuredNearestCap256V1 => 0x01,
        }
    }
}

/// Result of a workgroup-tree-width-policy lookup.
///
/// **Accepted public surface**, with the declaration pair and reader.
///
/// [`Self::Unknown`] is the fail-closed answer: a profile that declares nothing
/// does not offer the single-workgroup tree. It is not a preference, not a
/// clamp onto `256`, and not a substitution of the balanced partition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkgroupTreeWidthPolicyResolution {
    /// An exact declaration states this closed policy.
    Declared(WorkgroupTreeWidthPolicy),
    /// An exact declaration exists, but only from this later phase.
    Deferred {
        /// Earliest phase at which an exact declaration can resolve.
        available_at: AvailabilityPhase,
    },
    /// No declaration exists, so the tree is unavailable.
    Unknown,
}

/// One declared workgroup-tree-width policy and who vouches for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorkgroupTreeWidthPolicyFact {
    pub(super) policy: WorkgroupTreeWidthPolicy,
    pub(super) source: Arc<FactSourceProvenance>,
}

impl WorkgroupTreeWidthPolicyFact {
    pub(super) fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    pub(super) fn encode(&self, bytes: &mut Vec<u8>, source_index: usize) {
        bytes.push(self.policy.tag());
        encode_compact_index(bytes, source_index);
    }
}

/// Public scalar-declaration disposition.
///
/// Exact emulation is intentionally absent: only the compiler can verify that
/// emitted replacement operations prove an exact emulation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarSupport {
    /// The target directly honours the stated behaviour.
    Exact,
    /// The target explicitly cannot honour the stated behaviour.
    Unsupported,
}

impl ScalarSupport {
    pub(super) const fn means(self) -> HonouringMeans {
        match self {
            Self::Exact => HonouringMeans::SupportedExactly,
            Self::Unsupported => HonouringMeans::Unsupported,
        }
    }
}

/// Public synchronization-declaration disposition.
///
/// Two valued, and the negative is *statable*: a target that has been measured
/// not to provide a realization records that, and a target that was never asked
/// records nothing. Those are different states — a typed rejection and an
/// `Unknown` — and a vocabulary with only a positive spelling would collapse
/// them into one silence.
///
/// There is deliberately no "supported under a relaxation" spelling. A weaker
/// realization is a *different subject*, so a target that provides one declares
/// that subject; letting a caller's subject be satisfied by a neighbouring one
/// is exactly the composition the atomic fact exists to prevent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SynchronizationSupport {
    /// The target realizes exactly the declared subject.
    Realized,
    /// The target explicitly does not realize it.
    Unrealizable,
}

impl SynchronizationSupport {
    pub(super) const fn realization(self) -> SynchronizationRealization {
        match self {
            Self::Realized => SynchronizationRealization::Realized,
            Self::Unrealizable => SynchronizationRealization::Unrealizable,
        }
    }
}

/// Public subgroup-declaration disposition.
///
/// **Labelled draft** under ADR 0075. Tom accepted the two-valued *shape* on
/// 2026-08-11 — `Realized` and `Unrealizable` are explicit; silence is
/// `Unknown` — and has not accepted this crate's exact type spelling.
///
/// Two valued, and the negative is *statable*: a target that has been measured
/// not to provide a realization records that, and a target that was never asked
/// records nothing. Those are different states — a typed rejection and an
/// `Unknown` — and a vocabulary with only a positive spelling would collapse
/// them into one silence.
///
/// There is deliberately no "supported under a relaxation" spelling. A weaker
/// realization is a *different subject*, so a target that provides one declares
/// that subject; letting a caller's subject be satisfied by a neighbouring one
/// is exactly the composition the atomic fact exists to prevent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgroupSupport {
    /// The target realizes exactly the declared subject.
    Realized,
    /// The target explicitly does not realize it.
    Unrealizable,
}

impl SubgroupSupport {
    pub(super) const fn realization(self) -> SubgroupRealization {
        match self {
            Self::Realized => SubgroupRealization::Realized,
            Self::Unrealizable => SubgroupRealization::Unrealizable,
        }
    }
}

/// Result of a subgroup-realization lookup.
///
/// **Labelled draft** under ADR 0075, with [`SubgroupSupport`] and the
/// declaration pair.
///
/// [`Self::Unknown`] is the fail-closed answer and the overwhelmingly common
/// one: a profile that declares nothing about the subject answers it, and a
/// consumer may not read a neighbouring subject's row in its place. There is
/// deliberately no `Deferred` arm: no query vocabulary can ask a device whether
/// it realizes one complete subgroup subject, so a later-phase fact is
/// `Unknown` rather than a promise nothing can keep.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgroupRealizationResolution {
    /// An exact declaration states the target realizes this subject.
    Realized,
    /// An exact declaration states the target does not realize it.
    Unrealizable,
    /// No exact declaration exists for this subject, or none is available yet.
    Unknown,
}
