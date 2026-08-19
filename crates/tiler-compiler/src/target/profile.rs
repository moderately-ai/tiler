//! The frozen profile and the resolution surface consumers read it through.
//!
//! A profile is immutable once [`super::builder::TargetProfileBuilder`] freezes
//! it, and every reader below answers for one exact subject at one availability
//! phase. Silence resolves `Unknown` rather than inheriting a neighbouring
//! subject's row, and what each family's `Unknown` licenses differs by family —
//! the resolution types in [`super::rows`] state which.

use std::sync::{Arc, OnceLock};

use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::SubgroupRealizationSubject;
use tiler_ir::semantic::ResolvedValueType;

use crate::target::ScalarArithmetic;
use crate::target::accuracy::ElementaryRealization;
use crate::target::builder::TargetProfileBuilder;
use crate::target::feasibility::CheckedTargetProfile;
use crate::target::key::TargetProfileKey;
use crate::target::rows::{
    BackendArithmeticLicence, CostRow, CostRowFact, DTypeDispatchability, DTypeDispatchabilityFact,
    DTypeDispatchabilityResolution, EvaluationOrderFact, EvaluationOrderPreservation,
    EvaluationOrderResolution, QuantitativeCapabilityDeclaration, ScalarHonourabilityDeclaration,
    SubgroupRealizationResolution, TargetCostRowResolution, WorkgroupTreeWidthPolicyFact,
    WorkgroupTreeWidthPolicyResolution,
};

// The test-only profiles below are built by mutating a governed draft, so they
// reach into the declaration vocabulary the resolution surface itself never
// names. Gated rather than imported unconditionally so a production build still
// reports an import this module has stopped needing.
#[cfg(test)]
use tiler_ir::schedule::{FlushedZeroSign, NumericalPermission, SubnormalMode};

#[cfg(test)]
use crate::target::feasibility::CapabilityAxis;
#[cfg(test)]
use crate::target::honourability::{
    DeclaredBehaviour, DimensionBehaviour, FactSourceProvenance, HonouringMeans,
    NumericalDimension, governed_profile_source,
};
#[cfg(test)]
use crate::target::rows::{SynchronizationSupport, WorkgroupTreeWidthPolicy};
#[cfg(test)]
use crate::target::source::TargetFactSource;

/// One immutable, intrinsically checked target declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetProfile {
    pub(super) data: Arc<TargetProfileData>,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct TargetProfileData {
    pub(super) key: TargetProfileKey,
    pub(super) checked: CheckedTargetProfile,
    pub(super) quantitative: Box<[QuantitativeCapabilityDeclaration]>,
    pub(super) scalar: Box<[ScalarHonourabilityDeclaration]>,
    pub(super) dispatchability: Box<[DTypeDispatchabilityFact]>,
    pub(super) evaluation_order: Box<[EvaluationOrderFact]>,
    pub(super) cost_rows: Box<[CostRowFact]>,
    pub(super) tree_width_policies: Box<[WorkgroupTreeWidthPolicyFact]>,
    pub(super) elementary: Box<[ElementaryRealization]>,
    pub(super) descriptor: Box<[u8]>,
}

impl TargetProfile {
    /// Returns the compiler-governed bounded prototype profile.
    ///
    /// # Panics
    ///
    /// Panics only if this compiler build's own governed declaration violates
    /// its construction invariants.
    pub fn governed() -> Self {
        static GOVERNED: OnceLock<TargetProfile> = OnceLock::new();
        GOVERNED
            .get_or_init(|| {
                TargetProfileBuilder::governed()
                    .build()
                    .expect("the governed target profile is intrinsically valid")
            })
            .clone()
    }

    /// Returns this profile's owned key.
    #[must_use]
    pub fn profile_key(&self) -> &TargetProfileKey {
        &self.data.key
    }

    pub(crate) fn checked(&self) -> &CheckedTargetProfile {
        &self.data.checked
    }

    /// Returns the complete canonical declaration bytes used for identity.
    #[must_use]
    pub fn canonical_descriptor(&self) -> &[u8] {
        &self.data.descriptor
    }

    /// Returns the elementary realizations this profile declared, in canonical
    /// row order.
    ///
    /// **Labelled draft** under ADR 0075. A borrowed view of the stored rows.
    /// The slice is empty when the profile declared none, including the
    /// governed profile until a later evidence ticket can discharge both
    /// halves of a Metal row. Assessment reads this view; it does not
    /// reconstruct governed rows from descriptor equality.
    #[must_use]
    pub fn declared_elementary_realizations(&self) -> &[ElementaryRealization] {
        &self.data.elementary
    }

    /// Resolves whether this target realizes one complete subgroup subject.
    ///
    /// **Labelled draft** under ADR 0075, with the declaration pair.
    ///
    /// The match is one equality over the whole subject. A neighbouring
    /// width, arithmetic type, or transfer is `Unknown`, not a partial match.
    /// Silence is `Unknown`. A later-phase fact is `Unknown` rather than
    /// deferred: there is no query contract that could obtain the answer
    /// before routing commits.
    #[must_use]
    pub fn subgroup_realization(
        &self,
        subject: SubgroupRealizationSubject,
        available_phase: AvailabilityPhase,
    ) -> SubgroupRealizationResolution {
        let mut resolved: Option<&crate::target::feasibility::SubgroupRealizationFact> = None;
        for fact in self.data.checked.subgroup() {
            if fact.subject() != subject || fact.phase() > available_phase {
                continue;
            }
            resolved = Some(match resolved {
                Some(current) if current.phase() >= fact.phase() => current,
                _ => fact,
            });
        }
        match resolved {
            None => SubgroupRealizationResolution::Unknown,
            Some(fact) => match fact.realization() {
                crate::target::feasibility::SubgroupRealization::Realized => {
                    SubgroupRealizationResolution::Realized
                }
                crate::target::feasibility::SubgroupRealization::Unrealizable => {
                    SubgroupRealizationResolution::Unrealizable
                }
            },
        }
    }

    pub(crate) fn request_subject_bytes(&self) -> &[u8] {
        &self.data.descriptor
    }

    /// Resolves dispatchability only for an exactly equal resolved type,
    /// preferring the latest declaration available through `available_phase`.
    #[must_use]
    pub fn dtype_dispatchability(
        &self,
        resolved_type: &ResolvedValueType,
        available_phase: AvailabilityPhase,
    ) -> DTypeDispatchabilityResolution {
        let mut now = None;
        let mut later = None;
        for fact in self
            .data
            .dispatchability
            .iter()
            .filter(|fact| &fact.resolved_type == resolved_type)
        {
            let phase = fact.source.phase();
            if phase <= available_phase {
                now = Some(fact.verdict);
            } else if later.is_none() {
                later = Some(phase);
            }
        }
        match (now, later) {
            (Some(DTypeDispatchability::Dispatchable), _) => {
                DTypeDispatchabilityResolution::Dispatchable
            }
            (Some(DTypeDispatchability::Unsupported), _) => {
                DTypeDispatchabilityResolution::Unsupported
            }
            (None, Some(available_at)) => DTypeDispatchabilityResolution::Deferred { available_at },
            (None, None) => DTypeDispatchabilityResolution::Unknown,
        }
    }

    /// Resolves whether a backend translation under `licence` preserves the
    /// emitted evaluation order for exactly `subject`, preferring the latest
    /// declaration available through `available_phase`.
    ///
    /// **Accepted public surface**, with the declaration constructors.
    ///
    /// Returns [`EvaluationOrderResolution::Unknown`] for a subject or licence
    /// this profile does not speak about, which is every subject and licence of
    /// a profile that declares none. Nothing is inherited: a neighbouring
    /// arithmetic type's row and the other licence's row are both silence here.
    #[must_use]
    pub fn evaluation_order_preservation(
        &self,
        subject: &ScalarArithmetic,
        licence: BackendArithmeticLicence,
        available_phase: AvailabilityPhase,
    ) -> EvaluationOrderResolution {
        let mut now = None;
        let mut later = None;
        for fact in self
            .data
            .evaluation_order
            .iter()
            .filter(|fact| &fact.subject == subject && fact.licence == licence)
        {
            let phase = fact.source.phase();
            if phase <= available_phase {
                now = Some(fact.preservation);
            } else if later.is_none() {
                later = Some(phase);
            }
        }
        match (now, later) {
            (Some(EvaluationOrderPreservation::Preserved), _) => {
                EvaluationOrderResolution::Preserved
            }
            (Some(EvaluationOrderPreservation::NotPreserved), _) => {
                EvaluationOrderResolution::NotPreserved
            }
            (None, Some(available_at)) => EvaluationOrderResolution::Deferred { available_at },
            (None, None) => EvaluationOrderResolution::Unknown,
        }
    }

    /// Resolves how many fold steps this target retires at once when saturated,
    /// preferring the latest declaration available through `available_phase`.
    ///
    /// **Accepted public surface**, accepted by Tom on 2026-08-07 under
    /// `accept-the-measured-cost-row-public-surface`, with the two declaration
    /// constructors.
    ///
    /// Returns [`TargetCostRowResolution::Unknown`] for a profile that declares
    /// nothing, which is every profile but the qualified Apple9 macOS one. That
    /// answer is an absence of preference and never a refusal: a consumer that
    /// treated it as a bound, a zero, or an infeasibility would invert the failure
    /// direction this row exists to avoid.
    #[must_use]
    pub fn saturated_parallel_fold_steps(
        &self,
        available_phase: AvailabilityPhase,
    ) -> TargetCostRowResolution {
        self.cost_row(CostRow::SaturatedParallelFoldSteps, available_phase)
    }

    fn cost_row(
        &self,
        row: CostRow,
        available_phase: AvailabilityPhase,
    ) -> TargetCostRowResolution {
        let mut now = None;
        let mut later = None;
        for fact in self.data.cost_rows.iter().filter(|fact| fact.row == row) {
            let phase = fact.source.phase();
            if phase <= available_phase {
                now = Some(fact.value);
            } else if later.is_none() {
                later = Some(phase);
            }
        }
        match (now, later) {
            (Some(value), _) => TargetCostRowResolution::Declared { value },
            (None, Some(available_at)) => TargetCostRowResolution::Deferred { available_at },
            (None, None) => TargetCostRowResolution::Unknown,
        }
    }

    /// Resolves the closed tree-width policy this target declared, preferring
    /// the latest declaration available through `available_phase`.
    ///
    /// **Accepted public surface**, accepted by Tom's 2026-08-11 delegation
    /// under `gate-the-workgroup-tree-on-an-explicit-qualified-width-policy`.
    ///
    /// Returns [`WorkgroupTreeWidthPolicyResolution::Unknown`] for a profile
    /// that declares nothing. That answer makes the single-workgroup tree
    /// unavailable. It is never a clamp onto `256`, never a substitution of
    /// the balanced partition, and never a preference.
    #[must_use]
    pub fn workgroup_tree_width_policy(
        &self,
        available_phase: AvailabilityPhase,
    ) -> WorkgroupTreeWidthPolicyResolution {
        let mut now = None;
        let mut later = None;
        for fact in &self.data.tree_width_policies {
            let phase = fact.source.phase();
            if phase <= available_phase {
                now = Some(fact.policy);
            } else if later.is_none() {
                later = Some(phase);
            }
        }
        match (now, later) {
            (Some(policy), _) => WorkgroupTreeWidthPolicyResolution::Declared(policy),
            (None, Some(available_at)) => {
                WorkgroupTreeWidthPolicyResolution::Deferred { available_at }
            }
            (None, None) => WorkgroupTreeWidthPolicyResolution::Unknown,
        }
    }

    #[cfg(test)]
    pub(crate) fn governed_without_numerical_declarations() -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.scalar.clear();
        builder
            .build()
            .expect("the sparse test profile is intrinsically valid")
    }

    /// Returns the governed profile plus the exact gather-index dispatch row.
    ///
    /// Test-only because this is a diagnostic-layering probe, not a production
    /// claim that the governed target can dispatch integer tensors. Building it
    /// through the ordinary profile builder retains every governed declaration
    /// and recomputes the complete canonical descriptor with the added row.
    #[cfg(test)]
    pub(crate) fn governed_with_gather_index_dispatch_for_test() -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_dtype_dispatchability(
                tiler_ir::semantic::gather_index_resolved_type(),
                DTypeDispatchability::Dispatchable,
                TargetFactSource(governed_profile_source()),
            )
            .expect("the exact gather-index test dispatch declaration is valid");
        builder
            .build()
            .expect("the widened test target profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn governed_with_grid_axis_limit(limit: u64) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::GridAxisThreads)
            .expect("the governed profile declares the grid-axis limit")
            .bound = limit;
        builder
            .build()
            .expect("the test target profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn governed_with_workgroup_limit_for_test(key: &str, limit: u32) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder
            .queries
            .retain(|query| query.axis != CapabilityAxis::WorkgroupThreads);
        builder
            .declare_max_threads_per_workgroup(limit, TargetFactSource(governed_profile_source()))
            .expect("the test workgroup limit replaces the governed query");
        builder
            .build()
            .expect("the bounded test target profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn governed_with_key_for_test(key: &str) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder
            .build()
            .expect("the keyed test target profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn without_numerical_declarations_for_test(key: &str) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder.scalar.clear();
        builder
            .build()
            .expect("the sparse keyed test profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn flush_only_for_test(key: &str) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder.scalar.retain(|declaration| {
            !matches!(
                declaration.dimension,
                NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals
            ) || matches!(
                declaration.behaviour,
                DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::PreservesSign,
                })
            )
        });
        builder
            .build()
            .expect("the flush-only test profile is intrinsically valid")
    }

    /// One exact subnormal/reassociation realization for request-population
    /// tests that must force distinct numerical-contract groups.
    #[cfg(test)]
    pub(crate) fn numerical_realization_for_test(
        key: &str,
        subnormals: SubnormalMode,
        reassociation: NumericalPermission,
    ) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder
            .scalar
            .retain(|declaration| match declaration.dimension {
                NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals => {
                    declaration.behaviour == DimensionBehaviour::Subnormals(subnormals)
                }
                NumericalDimension::Contraction => {
                    declaration.behaviour
                        == DimensionBehaviour::Transform(NumericalPermission::Forbidden)
                }
                NumericalDimension::Reassociation => {
                    declaration.behaviour == DimensionBehaviour::Transform(reassociation)
                }
                _ => true,
            });
        builder
            .build()
            .expect("the exact numerical test profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn with_grid_axis_limit_for_test(key: &str, limit: u64) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::GridAxisThreads)
            .expect("the governed profile declares the grid-axis limit")
            .bound = limit;
        builder
            .build()
            .expect("the bounded keyed test profile is intrinsically valid")
    }

    /// The governed profile with preserved input subnormals declared
    /// unsupported under `source`.
    ///
    /// A strict contract is then refused by a *named* authority rather than
    /// merely undeclared, which is the only shape that carries provenance: an
    /// undeclared dimension has no fact to cite. Varying `source` alone varies
    /// exactly the evidence and nothing about what was required.
    #[cfg(test)]
    pub(crate) fn refusing_preserved_subnormals_for_test(
        key: &str,
        source: Arc<FactSourceProvenance>,
    ) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
        let declaration = builder
            .scalar
            .iter_mut()
            .find(|declaration| {
                declaration.dimension == NumericalDimension::InputSubnormals
                    && declaration.behaviour == preserve
            })
            .expect("the governed profile declares preserved input subnormals");
        declaration.means = HonouringMeans::Unsupported;
        declaration.source = source;
        builder
            .build()
            .expect("the refusing keyed test profile is intrinsically valid")
    }

    /// The governed profile widened until a single-workgroup tree is assessable.
    ///
    /// **Deliberately test-only, and that is the finding rather than a
    /// convenience.** `TargetProfileBuilder::governed` declares
    /// `local-memory-bytes` as *zero* and declares nothing at all about
    /// synchronization, so the bounded prototype baseline rejects every
    /// cooperative region twice over — first on threadgroup memory it guarantees
    /// none of, then on a realization it has never been asked about. Both
    /// refusals are correct and both are exercised by their own tests; what they
    /// mean is that the baseline cannot be the profile a synchronized strategy is
    /// *admitted* against.
    ///
    /// Raising the baseline's own rows instead would be a capability claim: the
    /// prototype authority has no evidence for a threadgroup-memory budget or a
    /// barrier realization, and inventing one would promote support from a test's
    /// convenience — precisely what the atomic synchronization fact exists to
    /// prevent. `realize-parallel-reduction-strategies-on-metal` owns the real
    /// declaration, and the question of what the *prototype baseline* should
    /// guarantee is Tom's, not this ticket's.
    ///
    /// `synchronization` is `None` for a profile that has never been asked, which
    /// is what makes the undeclared rejection drivable separately from a declared
    /// refusal.
    #[cfg(test)]
    pub(crate) fn workgroup_tree_target_for_test(
        local_memory_bytes: u64,
        grid_axis_threads: u64,
        synchronization: Option<SynchronizationSupport>,
    ) -> Self {
        Self::workgroup_tree_target_with_cost_row_for_test(
            local_memory_bytes,
            grid_axis_threads,
            synchronization,
            None,
        )
    }

    /// The same widened test profile, optionally carrying the measured cost row.
    ///
    /// `None` and `Some` are the two halves of the silence rule the activating
    /// ticket's acceptance made testable: a profile declaring no row must select
    /// bit-identically to one built before the row existed, and this is the
    /// constructor that lets one compile drive both sides with nothing else
    /// varying.
    #[cfg(test)]
    pub(crate) fn workgroup_tree_target_with_cost_row_for_test(
        local_memory_bytes: u64,
        grid_axis_threads: u64,
        synchronization: Option<SynchronizationSupport>,
        saturated_parallel_fold_steps: Option<u64>,
    ) -> Self {
        Self::workgroup_tree_target_parts_for_test(
            local_memory_bytes,
            grid_axis_threads,
            synchronization,
            saturated_parallel_fold_steps,
            Some(WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1),
        )
    }

    /// The same widened test profile with no tree-width policy declared.
    ///
    /// The negative half of the policy gate: omission makes the tree unavailable
    /// and must not substitute `256` or `governed_partition`.
    #[cfg(test)]
    pub(crate) fn workgroup_tree_target_without_width_policy_for_test(
        local_memory_bytes: u64,
        grid_axis_threads: u64,
        synchronization: Option<SynchronizationSupport>,
    ) -> Self {
        Self::workgroup_tree_target_parts_for_test(
            local_memory_bytes,
            grid_axis_threads,
            synchronization,
            None,
            None,
        )
    }

    #[cfg(test)]
    fn workgroup_tree_target_parts_for_test(
        local_memory_bytes: u64,
        grid_axis_threads: u64,
        synchronization: Option<SynchronizationSupport>,
        saturated_parallel_fold_steps: Option<u64>,
        width_policy: Option<WorkgroupTreeWidthPolicy>,
    ) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        if let Some(policy) = width_policy {
            builder
                .declare_workgroup_tree_width_policy(
                    policy,
                    TargetFactSource(governed_profile_source()),
                )
                .expect("the test tree-width-policy declaration is valid");
        }
        if let Some(steps) = saturated_parallel_fold_steps {
            builder
                .declare_saturated_parallel_fold_steps(
                    steps,
                    TargetFactSource(governed_profile_source()),
                )
                .expect("the test cost-row declaration is valid");
        }
        for (axis, bound) in [
            (CapabilityAxis::LocalMemoryBytes, local_memory_bytes),
            (CapabilityAxis::GridAxisThreads, grid_axis_threads),
        ] {
            builder
                .quantitative
                .iter_mut()
                .find(|declaration| declaration.axis == axis)
                .expect("the governed profile declares this axis")
                .bound = bound;
        }
        if let Some(support) = synchronization {
            // Derived from the canonical tile's own edges rather than restated,
            // so a test profile cannot declare a realization the strategy does
            // not require and then "admit" it.
            let tile = tiler_ir::schedule::workgroup_tree_tile(2)
                .expect("two participants are within the enumeration bound");
            let subject = tiler_ir::schedule::required_subject(&tile.visibility_edges())
                .expect("the canonical tree tile carries one handoff");
            builder
                .declare_synchronization_realization(
                    subject,
                    support,
                    &TargetFactSource(governed_profile_source()),
                )
                .expect("the test synchronization declaration is valid");
        }
        builder
            .build()
            .expect("the widened test target profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn governed_declared_behaviours() -> Vec<DeclaredBehaviour> {
        TargetProfileBuilder::governed()
            .scalar
            .iter()
            .map(ScalarHonourabilityDeclaration::declared)
            .collect()
    }
}
