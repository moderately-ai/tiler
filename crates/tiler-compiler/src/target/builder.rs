//! The producer-side construction surface that mints one immutable profile.
//!
//! Every `declare_*` pair validates and refuses atomically before insertion, so
//! a failed declaration leaves no repairable draft; [`TargetProfileBuilder::build`]
//! then revalidates the whole draft, canonicalizes each family into its stored
//! order, and freezes the checked profile and its complete descriptor together.
//! The canonical sort orders here are identity-bearing: they are the order
//! [`super::descriptor::complete_descriptor`] writes each family in.

use std::sync::Arc;

use tiler_ir::program::abi::{
    AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
};
use tiler_ir::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
    SubgroupRealizationSubject, SubnormalMode, SynchronizationSubject,
};
use tiler_ir::semantic::{F32, ResolvedValueType};

use crate::target::accuracy::ElementaryRealization;
use crate::target::descriptor::complete_descriptor;
use crate::target::error::TargetProfileBuildError;
use crate::target::feasibility::{
    CapabilityAxis, CapabilityFact, CapabilityQuery, CheckedTargetProfile,
    DeclaredSubgroupRealization, DeclaredSynchronizationRealization, FactProvenance,
    MAX_TARGET_PROFILE_DESCRIPTOR_BYTES, SubgroupRealization,
};
use crate::target::honourability::{
    DimensionBehaviour, FactSourceProvenance, NumericalDimension, governed_profile_source,
};
use crate::target::key::{TargetProfileIdentity, TargetProfileKey};
use crate::target::profile::{TargetProfile, TargetProfileData};
use crate::target::rows::{
    BackendArithmeticLicence, CostRow, CostRowFact, DTypeDispatchability, DTypeDispatchabilityFact,
    DeviceAddressWidth, EvaluationOrderFact, EvaluationOrderPreservation, IndexArithmeticSupport,
    QuantitativeCapabilityDeclaration, QuantitativeCapabilityQueryDeclaration,
    ScalarHonourabilityDeclaration, ScalarSupport, SubgroupSupport, SynchronizationSupport,
    WorkgroupTreeWidthPolicy, WorkgroupTreeWidthPolicyFact,
};
use crate::target::source::{TargetCompileProfileMeasurementSource, TargetFactSource};
use crate::target::{GOVERNED_TARGET_PROFILE_KEY, ScalarArithmetic};

/// Consuming producer-side builder for one immutable target profile.
#[derive(Clone, Debug)]
pub struct TargetProfileBuilder {
    pub(super) key: TargetProfileKey,
    pub(super) quantitative: Vec<QuantitativeCapabilityDeclaration>,
    pub(super) queries: Vec<QuantitativeCapabilityQueryDeclaration>,
    pub(super) scalar: Vec<ScalarHonourabilityDeclaration>,
    pub(super) dispatchability: Vec<DTypeDispatchabilityFact>,
    pub(super) synchronization: Vec<DeclaredSynchronizationRealization>,
    pub(super) evaluation_order: Vec<EvaluationOrderFact>,
    pub(super) cost_rows: Vec<CostRowFact>,
    pub(super) tree_width_policies: Vec<WorkgroupTreeWidthPolicyFact>,
    pub(super) elementary: Vec<ElementaryRealization>,
    pub(super) subgroup: Vec<DeclaredSubgroupRealization>,
    pub(super) subgroup_query: Option<TargetPropertyQuery>,
}

impl TargetProfileBuilder {
    /// Starts one externally declared sparse profile from a validated key.
    ///
    /// Every omitted quantitative axis remains unknown.
    #[must_use]
    pub fn new(key: TargetProfileKey) -> Self {
        Self {
            key,
            quantitative: Vec::new(),
            queries: Vec::new(),
            scalar: Vec::new(),
            dispatchability: Vec::new(),
            synchronization: Vec::new(),
            evaluation_order: Vec::new(),
            cost_rows: Vec::new(),
            tree_width_policies: Vec::new(),
            elementary: Vec::new(),
            subgroup: Vec::new(),
            subgroup_query: None,
        }
    }

    /// Declares one whole elementary-realization subject.
    ///
    /// **Labelled draft** under ADR 0075. The subject is already validated:
    /// its operation comes from a verified contract, both evidence records are
    /// complete, and its source is compile-profile-phase. This method stores
    /// the canonical row and refuses only an exact duplicate. Distinct
    /// contracts for one operation remain separate candidates. No row is
    /// replaced, merged, or preferred, and a half that cannot discharge is
    /// still stored so assessment can refuse it as `undischarged-evidence`.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError::DuplicateElementaryRealization`] when
    /// the same complete row is already declared.
    pub fn declare_elementary_realization(
        &mut self,
        realization: ElementaryRealization,
    ) -> Result<(), TargetProfileBuildError> {
        if self
            .elementary
            .iter()
            .any(|existing| existing == &realization)
        {
            return Err(TargetProfileBuildError::DuplicateElementaryRealization);
        }
        self.elementary.push(realization);
        Ok(())
    }

    /// Declares whether this target realizes one *complete* subgroup subject.
    ///
    /// **Labelled draft** under ADR 0075. The whole subject is one argument on
    /// purpose, and there is deliberately no per-dimension spelling — no
    /// `declare_subgroup_width`, no `declare_subgroup_arithmetic`. Each
    /// dimension is separately true of some realization on some machine, so a
    /// profile able to state them independently would let a caller's conjunction
    /// be satisfied by facts none of which is about it. A target realizing two
    /// subjects declares two facts.
    ///
    /// The verdict is stated rather than implied by presence, so a measured
    /// negative is recordable: an absent declaration is `Unknown` and rejects
    /// before executable-frontier admission, while
    /// [`SubgroupSupport::Unrealizable`] is a typed refusal a caller can act on.
    ///
    /// Generic construction validates provenance and structure. Backend-family
    /// correspondence stays in the backend-owned binding layer; there is no
    /// default row, inherited target-family row, or generic wrong-backend guess.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError::DuplicateSubgroupRealization`] when a
    /// fact for the same subject and phase is already declared.
    pub fn declare_subgroup_realization(
        &mut self,
        subject: SubgroupRealizationSubject,
        support: SubgroupSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_subgroup_with_source(subject, support, source.0)
    }

    /// Declares a measured subgroup realization.
    ///
    /// **Labelled draft** under ADR 0075, with the constructor above.
    ///
    /// Taking [`TargetCompileProfileMeasurementSource`] rather than the general
    /// [`TargetFactSource`] is what fixes its validity at
    /// [`TargetFactValidityScope::MeasuredEnvironment`](crate::target::TargetFactValidityScope::MeasuredEnvironment)
    /// and stops it widening into a portable claim.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError::DuplicateSubgroupRealization`] when a
    /// fact for the same subject and phase is already declared.
    pub fn declare_measured_subgroup_realization(
        &mut self,
        subject: SubgroupRealizationSubject,
        support: SubgroupSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_subgroup_with_source(subject, support, source.0)
    }

    fn declare_subgroup_with_source(
        &mut self,
        subject: SubgroupRealizationSubject,
        support: SubgroupSupport,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        if !source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        let realization = support.realization();
        if let Some(existing) = self
            .subgroup
            .iter()
            .find(|declared| declared.subject() == subject && declared.phase() == source.phase())
        {
            let exact_duplicate = existing.realization() == realization;
            let contradiction = existing.realization() != realization;
            if exact_duplicate || contradiction {
                return Err(TargetProfileBuildError::DuplicateSubgroupRealization);
            }
        }
        self.subgroup.push(DeclaredSubgroupRealization::new(
            subject,
            realization,
            source,
        ));
        Ok(())
    }

    /// Declares the one prepared-entry query that supplies the exact prepared
    /// pipeline's subgroup execution width (ADR 0094 decision 7).
    ///
    /// **Labelled draft** under ADR 0075, in the subgroup-realization family.
    /// The accepted 2026-08-11 prepared-width gate requires exactly one
    /// profile-level `PreparedKernelPreflight` subgroup-width query on any
    /// profile that declares a subgroup subject `Realized`: the compile-profile
    /// fact licenses the schedule, and the exact prepared pipeline must still
    /// report the literal width the compiler verified before routing commits.
    /// No query is inferred from a backend family, and no compile-profile row
    /// alone can discharge the gate.
    ///
    /// This is deliberately separate from
    /// [`Self::declare_subgroup_realization`]: that method records what the
    /// target realizes, while this one records how an exact prepared entry
    /// will produce the confirming width later. It is profile-level rather
    /// than per-subject because one prepared pipeline has one execution width;
    /// each requiring entry's required value is derived from its own verified
    /// atomic subject.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError::InvalidSubgroupQueryPhase`] for a
    /// query at any phase but `PreparedKernelPreflight`, and
    /// [`TargetProfileBuildError::DuplicateSubgroupWidthQuery`] for a second
    /// declaration. A missing or orphan query refuses at [`Self::build`],
    /// where the final realization set is known.
    pub fn declare_subgroup_width_query(
        &mut self,
        query: TargetPropertyQuery,
    ) -> Result<(), TargetProfileBuildError> {
        if query.available_at() != AvailabilityPhase::PreparedKernelPreflight {
            return Err(TargetProfileBuildError::InvalidSubgroupQueryPhase {
                required: AvailabilityPhase::PreparedKernelPreflight,
                actual: query.available_at(),
            });
        }
        if self.subgroup_query.is_some() {
            return Err(TargetProfileBuildError::DuplicateSubgroupWidthQuery);
        }
        self.subgroup_query = Some(query);
        Ok(())
    }

    /// Declares whether this target realizes one *complete* synchronization
    /// subject.
    ///
    /// The whole subject is one argument on purpose, and there is deliberately
    /// no per-dimension spelling — no `declare_barrier_execution_scope`, no
    /// `declare_fenced_spaces`. Each dimension is separately true of some
    /// realization on some machine, so a profile able to state them
    /// independently would let a caller's conjunction be satisfied by facts none
    /// of which is about it. A target realizing two subjects declares two facts.
    ///
    /// The verdict is stated rather than implied by presence, so a measured
    /// negative is recordable: an absent declaration is `Unknown` and rejects
    /// before executable-frontier admission, while
    /// [`SynchronizationSupport::Unrealizable`] is a typed refusal a caller can
    /// act on.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError`] when a fact for the same subject and
    /// phase is already declared, or when the subject fences no memory domain —
    /// a fence over nothing publishes nothing, so no handoff could consume it.
    pub fn declare_synchronization_realization(
        &mut self,
        subject: SynchronizationSubject,
        support: SynchronizationSupport,
        source: &TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        let realization = support.realization();
        if subject.fenced_spaces.is_empty() {
            return Err(TargetProfileBuildError::VacuousSynchronizationSubject);
        }
        let source = source.provenance();
        if let Some(existing) = self
            .synchronization
            .iter()
            .find(|declared| declared.subject() == subject && declared.phase() == source.phase())
        {
            // Exact restatement and same-key contradiction refuse independently
            // of each other and of sort order. The public error is one variant
            // because the uniqueness key already excludes the verdict; a second
            // public type would be a new boundary Tom has not accepted.
            let exact_duplicate = existing.realization() == realization;
            let contradiction = existing.realization() != realization;
            if exact_duplicate || contradiction {
                return Err(TargetProfileBuildError::DuplicateSynchronizationRealization);
            }
        }
        self.synchronization
            .push(DeclaredSynchronizationRealization::new(
                subject,
                realization,
                source,
            ));
        Ok(())
    }

    pub(super) fn governed() -> Self {
        // The grid row is the deliberately conservative four-thread guarantee
        // of this bounded macOS Metal profile, not a hardware maximum. In the
        // macOS 26.5 SDK, `MTLComputeCommandEncoder.h` documents
        // `dispatchThreads:threadsPerThreadgroup:` as accepting an
        // arbitrarily-sized grid whose dimensions need not be threadgroup
        // multiples, and `MTLTypes.h` defines each `MTLSize` dimension as
        // `NSUInteger`; the API is available from macOS 10.13. Those primary
        // declarations prove that extent four is representable on the governed
        // profile. They do not prove 65,535, an Apple-family maximum, or any
        // prepared pipeline's workgroup capacity. The shared source below
        // identifies the compiler-governed prototype guarantee; the production
        // Metal profile ticket replaces it with its full per-row authority
        // ledger rather than mislabelling this as a device measurement.
        let source = TargetFactSource(governed_profile_source());
        let mut builder = Self::new(TargetProfileKey::governed(GOVERNED_TARGET_PROFILE_KEY));
        builder
            .declare_max_threads_per_grid_axis(4, source.clone())
            .expect("the governed grid-axis declaration is valid");
        builder
            .declare_max_threads_per_workgroup_query(
                TargetPropertyQuery::new(
                    TargetPropertyKey::new(
                        "tiler.target.prepared-entry.max-threads-per-workgroup.v1",
                    )
                    .expect("the governed workgroup property key is valid"),
                    AvailabilityPhase::PreparedKernelPreflight,
                    TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1)
                        .expect("the governed target-query provider identity is valid"),
                )
                .expect("the governed workgroup query is deferred"),
            )
            .expect("the governed workgroup query declaration is valid");
        // Four is the widest signature the bounded profile can now assemble: a
        // recognized pointwise body has three leaves, so at most three input
        // tensors plus one output, which is also exactly what the strict-affine
        // dequantize region binds. It is stated as the governed budget's own
        // `buffers` bound rather than derived per strategy, so a plan the
        // request already admitted cannot then be refused for a binding count
        // the same build considered legal.
        //
        // It remains a compiler-governed prototype guarantee, **not** a device
        // measurement: `declare_measured_max_buffer_bindings_per_entry` is the
        // separate constructor a measured profile uses. Metal's own
        // documented per-stage buffer argument table bounds this far above —
        // the production profile ticket declares that figure with its per-row
        // authority ledger — so a conservative four claims nothing this
        // prototype authority cannot support.
        builder
            .declare_max_buffer_bindings_per_entry(4, source.clone())
            .expect("the governed binding declaration is valid");
        builder
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .expect("the governed index-arithmetic declaration is valid");
        builder
            .declare_device_memory(true, source.clone())
            .expect("the governed device-memory declaration is valid");
        builder
            .declare_local_memory_bytes(0, source)
            .expect("the governed local-memory declaration is valid");
        builder.scalar = governed_target_honourability();
        // This is a compiler-governed prototype, target-neutral dispatch fact.
        // It does not claim Metal support or any device-family measurement.
        builder
            .declare_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Dispatchable,
                TargetFactSource(governed_profile_source()),
            )
            .expect("the governed F32 dispatch declaration is valid");
        builder
    }

    fn declare_quantitative(
        &mut self,
        axis: CapabilityAxis,
        bound: u64,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let declaration = QuantitativeCapabilityDeclaration {
            axis,
            bound,
            source,
        };
        declaration.validate()?;
        if self.queries.iter().any(|query| query.axis == axis) {
            return Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery { axis: axis.key() },
            );
        }
        if self.quantitative.iter().any(|existing| {
            existing.axis == declaration.axis
                && existing.source.phase() == declaration.source.phase()
        }) {
            return Err(TargetProfileBuildError::DuplicateQuantitativeCapability {
                axis: declaration.axis.key(),
                phase: declaration.source.phase(),
            });
        }
        self.quantitative.push(declaration);
        Ok(())
    }

    /// Declares the maximum launch-grid extent along one axis.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_max_threads_per_grid_axis(
        &mut self,
        bound: u64,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::GridAxisThreads, bound, source.0)
    }

    /// Declares a measured maximum launch-grid extent along one axis.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_max_threads_per_grid_axis(
        &mut self,
        bound: u64,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::GridAxisThreads, bound, source.0)
    }

    /// Declares the maximum number of threads in one workgroup.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_max_threads_per_workgroup(
        &mut self,
        bound: u32,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::WorkgroupThreads, u64::from(bound), source.0)
    }

    /// Declares a measured maximum number of threads in one workgroup.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_max_threads_per_workgroup(
        &mut self,
        bound: u32,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::WorkgroupThreads, u64::from(bound), source.0)
    }

    /// Declares the prepared-entry query that supplies the exact maximum
    /// threads-per-workgroup value for a future compiled kernel.
    ///
    /// This is deliberately separate from
    /// [`Self::declare_max_threads_per_workgroup`]: that method records an
    /// available value, while this one records how an exact prepared entry will
    /// produce a value later.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting a duplicate or wrong-phase
    /// query. A live-device maximum cannot substitute for the prepared
    /// pipeline's function-specific maximum.
    pub fn declare_max_threads_per_workgroup_query(
        &mut self,
        query: TargetPropertyQuery,
    ) -> Result<(), TargetProfileBuildError> {
        if query.available_at() != AvailabilityPhase::PreparedKernelPreflight {
            return Err(TargetProfileBuildError::InvalidQuantitativeQueryPhase {
                axis: CapabilityAxis::WorkgroupThreads.key(),
                required: AvailabilityPhase::PreparedKernelPreflight,
                actual: query.available_at(),
            });
        }
        if self
            .quantitative
            .iter()
            .any(|existing| existing.axis == CapabilityAxis::WorkgroupThreads)
        {
            return Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                    axis: CapabilityAxis::WorkgroupThreads.key(),
                },
            );
        }
        if self
            .queries
            .iter()
            .any(|existing| existing.axis == CapabilityAxis::WorkgroupThreads)
        {
            return Err(TargetProfileBuildError::DuplicateQuantitativeQuery {
                axis: CapabilityAxis::WorkgroupThreads.key(),
            });
        }
        self.queries.push(QuantitativeCapabilityQueryDeclaration {
            axis: CapabilityAxis::WorkgroupThreads,
            query,
        });
        Ok(())
    }

    /// Declares the maximum distinct buffer bindings per kernel entry.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_max_buffer_bindings_per_entry(
        &mut self,
        bound: u32,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::BufferBindings, u64::from(bound), source.0)
    }

    /// Declares a measured maximum number of buffer bindings per kernel entry.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_max_buffer_bindings_per_entry(
        &mut self,
        bound: u32,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::BufferBindings, u64::from(bound), source.0)
    }

    /// Declares support for the governed KIR index-arithmetic family.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_index_arithmetic(
        &mut self,
        support: IndexArithmeticSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::IndexArithmeticU64,
            support.bound(),
            source.0,
        )
    }

    /// Declares measured support for the governed KIR index-arithmetic family.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_index_arithmetic(
        &mut self,
        support: IndexArithmeticSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::IndexArithmeticU64,
            support.bound(),
            source.0,
        )
    }

    /// Declares the exact device address-model width.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_device_address_width(
        &mut self,
        width: DeviceAddressWidth,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::DeviceAddressWidthBits,
            u64::from(width.bits()),
            source.0,
        )
    }

    /// Declares a measured exact device address-model width.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_device_address_width(
        &mut self,
        width: DeviceAddressWidth,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::DeviceAddressWidthBits,
            u64::from(width.bits()),
            source.0,
        )
    }

    /// Declares whether an explicitly addressable device memory space exists.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_device_memory(
        &mut self,
        supported: bool,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::DeviceAddressSpace,
            u64::from(supported),
            source.0,
        )
    }

    /// Declares measured support for an explicitly addressable device memory space.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_device_memory(
        &mut self,
        supported: bool,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::DeviceAddressSpace,
            u64::from(supported),
            source.0,
        )
    }

    /// Declares the maximum explicitly staged local memory in bytes.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_local_memory_bytes(
        &mut self,
        bound: u64,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::LocalMemoryBytes, bound, source.0)
    }

    /// Declares a measured maximum explicitly staged local memory size.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_local_memory_bytes(
        &mut self,
        bound: u64,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::LocalMemoryBytes, bound, source.0)
    }

    fn declare_scalar(
        &mut self,
        subject: ScalarArithmetic,
        dimension: NumericalDimension,
        behaviour: DimensionBehaviour,
        support: ScalarSupport,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let declaration = ScalarHonourabilityDeclaration {
            subject,
            dimension,
            behaviour,
            means: support.means(),
            source,
        };
        declaration.validate()?;
        if self.scalar.iter().any(|existing| {
            existing.dimension == declaration.dimension
                && existing.subject == declaration.subject
                && existing.behaviour == declaration.behaviour
                && existing.source.phase() == declaration.source.phase()
        }) {
            return Err(TargetProfileBuildError::DuplicateScalarDeclaration);
        }
        self.scalar.push(declaration);
        Ok(())
    }

    /// Declares support for one exact input-subnormal behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_input_subnormals(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: SubnormalMode,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::InputSubnormals,
            DimensionBehaviour::Subnormals(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one exact result-subnormal behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_result_subnormals(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: SubnormalMode,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ResultSubnormals,
            DimensionBehaviour::Subnormals(behaviour),
            support,
            source.0,
        )
    }

    /// Declares the one measured scalar input-subnormal realization delivered
    /// by a compiler profile, and explicitly refuses the other two realizations.
    ///
    /// The input-subnormal dimension receives a complete, exclusive three-row
    /// table. If that dimension already contains any row for the exact subject
    /// at any phase or behaviour, this operation refuses before inserting
    /// anything.
    ///
    /// # Errors
    ///
    /// Returns a typed conflict naming the exact subject, dimension, and phase
    /// of the first pre-existing row, without mutating the builder.
    pub fn declare_measured_input_subnormal_behaviour(
        &mut self,
        subject: ScalarArithmetic,
        delivered: SubnormalMode,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_measured_subnormal_dimension(
            subject,
            NumericalDimension::InputSubnormals,
            delivered,
            source,
        )
    }

    /// Declares the one measured scalar result-subnormal realization delivered
    /// by a compiler profile, and explicitly refuses the other two realizations.
    ///
    /// The result-subnormal dimension receives a complete, exclusive three-row
    /// table. If that dimension already contains any row for the exact subject
    /// at any phase or behaviour, this operation refuses before inserting
    /// anything.
    ///
    /// # Errors
    ///
    /// Returns a typed conflict naming the exact subject, dimension, and phase
    /// of the first pre-existing row, without mutating the builder.
    pub fn declare_measured_result_subnormal_behaviour(
        &mut self,
        subject: ScalarArithmetic,
        delivered: SubnormalMode,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_measured_subnormal_dimension(
            subject,
            NumericalDimension::ResultSubnormals,
            delivered,
            source,
        )
    }

    fn declare_measured_subnormal_dimension(
        &mut self,
        subject: ScalarArithmetic,
        dimension: NumericalDimension,
        delivered: SubnormalMode,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        debug_assert!(matches!(
            dimension,
            NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals
        ));
        if let Some(existing) = self
            .scalar
            .iter()
            .find(|existing| existing.subject == subject && existing.dimension == dimension)
        {
            return Err(TargetProfileBuildError::ConflictingSubnormalDeclaration {
                subject: Box::new(subject),
                dimension: existing.dimension.key(),
                phase: existing.source.phase(),
            });
        }

        let (preserve, signed, positive) = match delivered {
            SubnormalMode::Preserve => (
                ScalarSupport::Exact,
                ScalarSupport::Unsupported,
                ScalarSupport::Unsupported,
            ),
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            } => (
                ScalarSupport::Unsupported,
                ScalarSupport::Exact,
                ScalarSupport::Unsupported,
            ),
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            } => (
                ScalarSupport::Unsupported,
                ScalarSupport::Unsupported,
                ScalarSupport::Exact,
            ),
        };
        let rows = [
            (SubnormalMode::Preserve, preserve),
            (
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::PreservesSign,
                },
                signed,
            ),
            (
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::AlwaysPositive,
                },
                positive,
            ),
        ];
        let source = source.0;
        let declarations = rows.map(|(behaviour, support)| ScalarHonourabilityDeclaration {
            subject: subject.clone(),
            dimension,
            behaviour: DimensionBehaviour::Subnormals(behaviour),
            means: support.means(),
            source: Arc::clone(&source),
        });
        for declaration in &declarations {
            declaration.validate()?;
        }
        self.scalar.extend(declarations);
        Ok(())
    }

    /// Declares support for one contraction permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_contraction(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Contraction,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one contraction permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_contraction(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Contraction,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one reassociation permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_reassociation(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Reassociation,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one reassociation permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_reassociation(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Reassociation,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one operand-permutation permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_permutation(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Permutation,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one operand-permutation permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_permutation(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Permutation,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one signed-zero permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_signed_zero(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::SignedZero,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one signed-zero permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_signed_zero(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::SignedZero,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one reciprocal-transform permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_reciprocal_transform(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ReciprocalTransform,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one reciprocal-transform permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_reciprocal_transform(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ReciprocalTransform,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one approximation envelope.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_approximate_intrinsics(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: tiler_ir::schedule::ApproximationEnvelope,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ApproximateIntrinsics,
            DimensionBehaviour::Approximation(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one approximation envelope.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_approximate_intrinsics(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: tiler_ir::schedule::ApproximationEnvelope,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ApproximateIntrinsics,
            DimensionBehaviour::Approximation(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one NaN-assumption behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_nan_assumptions(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: ExceptionalValueAssumption,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::NanAssumptions,
            DimensionBehaviour::ExceptionalValue(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one NaN-assumption behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_nan_assumptions(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: ExceptionalValueAssumption,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::NanAssumptions,
            DimensionBehaviour::ExceptionalValue(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one infinity-assumption behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_infinity_assumptions(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: ExceptionalValueAssumption,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::InfinityAssumptions,
            DimensionBehaviour::ExceptionalValue(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one infinity-assumption behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_infinity_assumptions(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: ExceptionalValueAssumption,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::InfinityAssumptions,
            DimensionBehaviour::ExceptionalValue(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one observable materialization-rounding behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_materialization_rounding(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: tiler_ir::schedule::MaterializationRounding,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::MaterializationRounding,
            DimensionBehaviour::Rounding(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one observable materialization-rounding behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_materialization_rounding(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: tiler_ir::schedule::MaterializationRounding,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::MaterializationRounding,
            DimensionBehaviour::Rounding(behaviour),
            support,
            source.0,
        )
    }

    /// Declares dispatchability for one exact full resolved value type.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    ///
    /// No neighbouring nominal type, parameterized type, or encoded value
    /// inherits this declaration.
    pub fn declare_dtype_dispatchability(
        &mut self,
        resolved_type: ResolvedValueType,
        verdict: DTypeDispatchability,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_dtype_dispatchability_with_source(resolved_type, verdict, source.0)
    }

    /// Declares measured dispatchability for one exact full resolved value type.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    ///
    /// No neighbouring nominal type, parameterized type, or encoded value
    /// inherits this declaration.
    pub fn declare_measured_dtype_dispatchability(
        &mut self,
        resolved_type: ResolvedValueType,
        verdict: DTypeDispatchability,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_dtype_dispatchability_with_source(resolved_type, verdict, source.0)
    }

    /// Declares whether a backend translation under one arithmetic-rewriting
    /// licence preserves the evaluation order the emitted program pins.
    ///
    /// **Accepted public surface**, accepted by Tom on 2026-08-06 under
    /// `accept-the-evaluation-order-preservation-target-fact`.
    ///
    /// The fact is keyed by the exact scalar subject as well as the licence,
    /// because nothing establishes that one width's answer is another's — the
    /// measurement behind the vocabulary is `f32` only, and the two subnormal
    /// dimensions are already measured *disagreeing* across widths on one Apple
    /// row. A subject or licence this profile does not speak about resolves
    /// [`EvaluationOrderResolution::Unknown`](crate::target::EvaluationOrderResolution::Unknown) rather than inheriting a
    /// neighbour's row.
    ///
    /// A profile that declares nothing here answers `Unknown` for every subject
    /// and licence, which is the fail-closed default: the oracle's refusal class
    /// 3 refuses a plan whose pinned order the backend may change rather than
    /// qualifying it.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_evaluation_order_preservation(
        &mut self,
        subject: ScalarArithmetic,
        licence: BackendArithmeticLicence,
        preservation: EvaluationOrderPreservation,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_evaluation_order_with_source(subject, licence, preservation, source.0)
    }

    /// Declares a measured evaluation-order-preservation row.
    ///
    /// **Accepted public surface**, with the constructor above.
    ///
    /// The measured spelling is the one a target row is expected to use: the
    /// property is a fact about an exact backend compiler build, which no
    /// normative document this repository holds states — the vendored MSL 4.0
    /// and 4.1 specifications contain no occurrence of `evaluation order` at
    /// all, and the sentence that comes closest is already refuted as a
    /// universal claim by the same profile's subnormal rows.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_measured_evaluation_order_preservation(
        &mut self,
        subject: ScalarArithmetic,
        licence: BackendArithmeticLicence,
        preservation: EvaluationOrderPreservation,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_evaluation_order_with_source(subject, licence, preservation, source.0)
    }

    /// Declares how many fold steps this target retires at once when its launch
    /// saturates the device.
    ///
    /// **Accepted public surface**, accepted by Tom on 2026-08-07 under
    /// `accept-the-measured-cost-row-public-surface`, with
    /// [`TargetCostRowResolution`](crate::target::TargetCostRowResolution) and the measured
    /// constructor below.
    ///
    /// This is a **cost row, not a capability axis**, and the difference is
    /// load-bearing rather than presentational. A capability axis is a hard bound
    /// a feasibility predicate reads, and silence about one is an `Unknown` that
    /// never reaches an executable frontier. Nothing reads this row for
    /// feasibility, so declaring it that way would make silence render a profile
    /// unexecutable for a quantity no predicate consults. Silence here means *no
    /// preference*: a profile declaring nothing selects exactly as it did before
    /// this row existed, byte for byte, and its canonical descriptor does not move.
    ///
    /// A value of zero is admitted and is a statement rather than an absence — it
    /// says the target retires no fold step in parallel — but no consumer in this
    /// build acts on it, because a selector dividing by it would have nothing to
    /// compare.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_saturated_parallel_fold_steps(
        &mut self,
        steps: u64,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_cost_row(CostRow::SaturatedParallelFoldSteps, steps, source.0)
    }

    /// Declares a measured saturated-parallel-fold-step count.
    ///
    /// **Accepted public surface**, accepted by Tom on 2026-08-07 under
    /// `accept-the-measured-cost-row-public-surface`, with the constructor above.
    ///
    /// The measured spelling is the one a target row is expected to use, and it is
    /// the *only* one any profile in this repository uses. The quantity is a
    /// property of one device under one toolchain, fitted from a dispatch sweep;
    /// no normative document states it, and none could. Taking
    /// [`TargetCompileProfileMeasurementSource`] rather than the general
    /// [`TargetFactSource`] is what fixes its validity at
    /// [`TargetFactValidityScope::MeasuredEnvironment`](crate::target::TargetFactValidityScope::MeasuredEnvironment)
    /// and stops it widening into a portable claim.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_measured_saturated_parallel_fold_steps(
        &mut self,
        steps: u64,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_cost_row(CostRow::SaturatedParallelFoldSteps, steps, source.0)
    }

    fn declare_cost_row(
        &mut self,
        row: CostRow,
        value: u64,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let fact = CostRowFact { row, value, source };
        fact.validate()?;
        if self
            .cost_rows
            .iter()
            .any(|existing| existing.row == row && existing.source.phase() == fact.source.phase())
        {
            return Err(TargetProfileBuildError::DuplicateCostRow {
                row: row.key(),
                phase: fact.source.phase(),
            });
        }
        self.cost_rows.push(fact);
        Ok(())
    }

    /// Declares the closed tree-width policy this target uses when it offers
    /// the single-workgroup tree.
    ///
    /// **Accepted public surface**, accepted by Tom's 2026-08-11 delegation
    /// under `gate-the-workgroup-tree-on-an-explicit-qualified-width-policy`,
    /// with [`WorkgroupTreeWidthPolicyResolution`](crate::target::WorkgroupTreeWidthPolicyResolution)
    /// and the measured constructor below.
    ///
    /// This is **not a cost row and not a capability axis**. A cost row's
    /// silence means no preference; this family's silence makes the tree
    /// unavailable. A capability axis would make silence render a profile
    /// unexecutable for a quantity no feasibility predicate reads. The policy
    /// is a qualification on offering one strategy, decided before a region
    /// exists. There is no public numeric cap, no default, and no clamp.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_workgroup_tree_width_policy(
        &mut self,
        policy: WorkgroupTreeWidthPolicy,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_tree_width_policy(policy, source.0)
    }

    /// Declares a measured workgroup-tree-width policy.
    ///
    /// **Accepted public surface**, with the constructor above.
    ///
    /// The measured spelling is the one a target row is expected to use, and it
    /// is the *only* one any production profile in this repository uses. Taking
    /// [`TargetCompileProfileMeasurementSource`] rather than the general
    /// [`TargetFactSource`] is what fixes its validity at
    /// [`TargetFactValidityScope::MeasuredEnvironment`](crate::target::TargetFactValidityScope::MeasuredEnvironment)
    /// and stops it widening into a portable claim.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_measured_workgroup_tree_width_policy(
        &mut self,
        policy: WorkgroupTreeWidthPolicy,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_tree_width_policy(policy, source.0)
    }

    fn declare_tree_width_policy(
        &mut self,
        policy: WorkgroupTreeWidthPolicy,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let fact = WorkgroupTreeWidthPolicyFact { policy, source };
        fact.validate()?;
        if self
            .tree_width_policies
            .iter()
            .any(|existing| existing.source.phase() == fact.source.phase())
        {
            return Err(TargetProfileBuildError::DuplicateWorkgroupTreeWidthPolicy {
                phase: fact.source.phase(),
            });
        }
        self.tree_width_policies.push(fact);
        Ok(())
    }

    fn declare_evaluation_order_with_source(
        &mut self,
        subject: ScalarArithmetic,
        licence: BackendArithmeticLicence,
        preservation: EvaluationOrderPreservation,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let fact = EvaluationOrderFact {
            subject,
            licence,
            preservation,
            source,
        };
        fact.validate()?;
        let key = fact.subject_key();
        if self
            .evaluation_order
            .iter()
            .any(|existing| existing.subject_key() == key)
        {
            return Err(
                TargetProfileBuildError::DuplicateEvaluationOrderPreservation {
                    licence: licence.key(),
                    phase: fact.source.phase(),
                },
            );
        }
        self.evaluation_order.push(fact);
        Ok(())
    }

    fn declare_dtype_dispatchability_with_source(
        &mut self,
        resolved_type: ResolvedValueType,
        verdict: DTypeDispatchability,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let fact = DTypeDispatchabilityFact {
            resolved_type,
            verdict,
            source,
        };
        fact.validate()?;
        if self.dispatchability.iter().any(|existing| {
            existing.resolved_type == fact.resolved_type
                && existing.source.phase() == fact.source.phase()
        }) {
            return Err(TargetProfileBuildError::DuplicateDispatchability);
        }
        self.dispatchability.push(fact);
        Ok(())
    }

    /// Verifies and freezes this profile.
    ///
    /// # Errors
    ///
    /// Returns the first intrinsic checking or bounded-descriptor diagnostic.
    /// Public declaration methods reject invalid or duplicate rows atomically,
    /// before insertion, so a failed build has no repairable draft to return.
    pub fn build(self) -> Result<TargetProfile, TargetProfileBuildError> {
        self.freeze()
    }

    pub(super) fn validate_declarations(&self) -> Result<(), TargetProfileBuildError> {
        for declaration in &self.quantitative {
            declaration.validate()?;
            if self
                .queries
                .iter()
                .any(|query| query.axis == declaration.axis)
            {
                return Err(
                    TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                        axis: declaration.axis.key(),
                    },
                );
            }
            if self
                .quantitative
                .iter()
                .filter(|candidate| {
                    candidate.axis == declaration.axis
                        && candidate.source.phase() == declaration.source.phase()
                })
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateQuantitativeCapability {
                    axis: declaration.axis.key(),
                    phase: declaration.source.phase(),
                });
            }
        }
        for declaration in &self.queries {
            if declaration.axis == CapabilityAxis::WorkgroupThreads
                && declaration.query.available_at() != AvailabilityPhase::PreparedKernelPreflight
            {
                return Err(TargetProfileBuildError::InvalidQuantitativeQueryPhase {
                    axis: declaration.axis.key(),
                    required: AvailabilityPhase::PreparedKernelPreflight,
                    actual: declaration.query.available_at(),
                });
            }
            if self
                .queries
                .iter()
                .filter(|candidate| candidate.axis == declaration.axis)
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateQuantitativeQuery {
                    axis: declaration.axis.key(),
                });
            }
        }
        for declaration in &self.scalar {
            declaration.validate()?;
            if self
                .scalar
                .iter()
                .filter(|candidate| {
                    candidate.dimension == declaration.dimension
                        && candidate.subject == declaration.subject
                        && candidate.behaviour == declaration.behaviour
                        && candidate.source.phase() == declaration.source.phase()
                })
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateScalarDeclaration);
            }
        }
        for fact in &self.dispatchability {
            fact.validate()?;
            if self
                .dispatchability
                .iter()
                .filter(|candidate| {
                    candidate.resolved_type == fact.resolved_type
                        && candidate.source.phase() == fact.source.phase()
                })
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateDispatchability);
            }
        }
        for fact in &self.synchronization {
            let key = fact.sort_key();
            let same_key: Vec<_> = self
                .synchronization
                .iter()
                .filter(|candidate| candidate.sort_key() == key)
                .collect();
            if same_key.len() <= 1 {
                continue;
            }
            let exact_duplicate = same_key
                .windows(2)
                .any(|pair| pair[0].realization() == pair[1].realization());
            let contradiction = same_key
                .windows(2)
                .any(|pair| pair[0].realization() != pair[1].realization());
            if exact_duplicate || contradiction {
                return Err(TargetProfileBuildError::DuplicateSynchronizationRealization);
            }
        }
        for fact in &self.evaluation_order {
            fact.validate()?;
            let key = fact.subject_key();
            if self
                .evaluation_order
                .iter()
                .filter(|candidate| candidate.subject_key() == key)
                .count()
                > 1
            {
                return Err(
                    TargetProfileBuildError::DuplicateEvaluationOrderPreservation {
                        licence: fact.licence.key(),
                        phase: fact.source.phase(),
                    },
                );
            }
        }
        for fact in &self.cost_rows {
            fact.validate()?;
            if self
                .cost_rows
                .iter()
                .filter(|candidate| {
                    candidate.row == fact.row && candidate.source.phase() == fact.source.phase()
                })
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateCostRow {
                    row: fact.row.key(),
                    phase: fact.source.phase(),
                });
            }
        }
        for fact in &self.tree_width_policies {
            fact.validate()?;
            if self
                .tree_width_policies
                .iter()
                .filter(|candidate| candidate.source.phase() == fact.source.phase())
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateWorkgroupTreeWidthPolicy {
                    phase: fact.source.phase(),
                });
            }
        }
        for fact in &self.elementary {
            if self
                .elementary
                .iter()
                .filter(|candidate| *candidate == fact)
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateElementaryRealization);
            }
        }
        for fact in &self.subgroup {
            let key = fact.sort_key();
            let same_key: Vec<_> = self
                .subgroup
                .iter()
                .filter(|candidate| candidate.sort_key() == key)
                .collect();
            if same_key.len() <= 1 {
                continue;
            }
            let exact_duplicate = same_key
                .windows(2)
                .any(|pair| pair[0].realization() == pair[1].realization());
            let contradiction = same_key
                .windows(2)
                .any(|pair| pair[0].realization() != pair[1].realization());
            if exact_duplicate || contradiction {
                return Err(TargetProfileBuildError::DuplicateSubgroupRealization);
            }
        }
        // The accepted prepared-width gate's profile contract, decided here
        // where the final realization set is known: a `Realized` subject with
        // no executable width path leaves ADR 0094 decision 7's confirmation
        // undischargeable, and a query with no `Realized` subject claims a
        // realization this profile never made. `Unrealizable` requires no
        // positive query.
        let declares_realized = self
            .subgroup
            .iter()
            .any(|declared| declared.realization() == SubgroupRealization::Realized);
        match &self.subgroup_query {
            Some(_) if !declares_realized => {
                return Err(TargetProfileBuildError::OrphanSubgroupWidthQuery);
            }
            None if declares_realized => {
                return Err(TargetProfileBuildError::MissingSubgroupWidthQuery);
            }
            _ => {}
        }
        Ok(())
    }

    fn canonicalize(&mut self) {
        self.quantitative
            .sort_by_key(|declaration| (declaration.axis, declaration.source.phase()));
        self.queries.sort_by_key(|declaration| declaration.axis);
        self.scalar.sort_by_cached_key(|declaration| {
            let mut bytes = Vec::new();
            declaration.encode(&mut bytes);
            bytes
        });
        self.dispatchability.sort_by(|left, right| {
            left.resolved_type
                .cmp(&right.resolved_type)
                .then(left.source.phase().cmp(&right.source.phase()))
        });
        // Subject, then phase — the complete uniqueness key, excluding the
        // verdict so a contradiction cannot survive as two adjacent rows
        // whose sort order would pick a winner. The complete descriptor
        // encodes this family in the order this sort produces.
        self.synchronization
            .sort_by_key(DeclaredSynchronizationRealization::sort_key);
        // Subject, then licence, then phase — so the rows of one (subject,
        // licence) group are contiguous and phase-ascending, which is what makes
        // `evaluation_order_preservation`'s "latest available phase wins" scan
        // deterministic rather than dependent on declaration order.
        self.evaluation_order
            .sort_by_cached_key(EvaluationOrderFact::subject_key);
        // Row, then phase — so one row's declarations are contiguous and
        // phase-ascending, which is what makes the reader's "latest available
        // phase wins" scan deterministic rather than declaration-order dependent.
        self.cost_rows
            .sort_by_key(|fact| (fact.row, fact.source.phase()));
        self.tree_width_policies
            .sort_by_key(|fact| fact.source.phase());
        // Whole-row canonical encoding, so two profiles that declare the same
        // rows in different insertion orders share one identity, and distinct
        // contracts for one operation stay distinct candidates.
        self.elementary
            .sort_by_cached_key(ElementaryRealization::sort_key);
        // Subject, then phase — the complete uniqueness key, excluding the
        // verdict so a contradiction cannot survive as two adjacent rows
        // whose sort order would pick a winner. The complete descriptor
        // encodes this family in the order this sort produces.
        self.subgroup
            .sort_by_key(DeclaredSubgroupRealization::sort_key);
    }

    fn freeze(mut self) -> Result<TargetProfile, TargetProfileBuildError> {
        self.validate_declarations()?;
        self.canonicalize();
        let identity = TargetProfileIdentity::from_key(self.key.clone());
        let numerical: Vec<_> = self
            .scalar
            .iter()
            .map(ScalarHonourabilityDeclaration::declared)
            .collect();
        let fact = |declaration: &QuantitativeCapabilityDeclaration| {
            CapabilityFact::new(
                declaration.axis,
                declaration.bound,
                declaration.source.phase(),
                declaration.source.authority(),
                declaration.source.validity(),
                FactProvenance::declared_by(identity.clone()),
            )
        };
        let honourability = numerical
            .iter()
            .map(|declared| declared.attributed_to(identity.clone()))
            .collect();
        let checked = CheckedTargetProfile::new_complete(
            identity.clone(),
            self.quantitative.iter().map(fact).collect(),
            self.queries
                .iter()
                .map(|declaration| {
                    CapabilityQuery::new(declaration.axis, declaration.query.clone())
                })
                .collect(),
            honourability,
            self.synchronization
                .iter()
                .map(|declared| declared.clone().attributed_to(identity.clone()))
                .collect(),
            self.subgroup
                .iter()
                .map(|declared| declared.clone().attributed_to(identity.clone()))
                .collect(),
            self.subgroup_query.clone(),
        )
        .map_err(TargetProfileBuildError::from)?;

        let descriptor = complete_descriptor(
            &self.key,
            &self.quantitative,
            &self.queries,
            &self.scalar,
            &self.dispatchability,
            &self.synchronization,
            &self.evaluation_order,
            &self.cost_rows,
            &self.tree_width_policies,
            &self.elementary,
            &self.subgroup,
            self.subgroup_query.as_ref(),
        );
        if descriptor.len() > MAX_TARGET_PROFILE_DESCRIPTOR_BYTES {
            return Err(TargetProfileBuildError::DescriptorTooLong {
                actual: descriptor.len(),
                max: MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
            });
        }
        let Self {
            key,
            quantitative,
            queries: _,
            scalar,
            dispatchability,
            synchronization: _,
            evaluation_order,
            cost_rows,
            tree_width_policies,
            elementary,
            subgroup: _,
            subgroup_query: _,
        } = self;
        Ok(TargetProfile {
            data: Arc::new(TargetProfileData {
                key,
                checked,
                quantitative: quantitative.into_boxed_slice(),
                scalar: scalar.into_boxed_slice(),
                dispatchability: dispatchability.into_boxed_slice(),
                evaluation_order: evaluation_order.into_boxed_slice(),
                cost_rows: cost_rows.into_boxed_slice(),
                tree_width_policies: tree_width_policies.into_boxed_slice(),
                elementary: elementary.into_boxed_slice(),
                descriptor: descriptor.into_boxed_slice(),
            }),
        })
    }

    #[cfg(test)]
    pub(super) fn try_build(self) -> Result<TargetProfile, TargetProfileBuildError> {
        self.build()
    }
}

fn governed_target_honourability() -> Vec<ScalarHonourabilityDeclaration> {
    let exact =
        |dimension, behaviour| ScalarHonourabilityDeclaration::governed_exact(dimension, behaviour);
    vec![
        exact(
            NumericalDimension::InputSubnormals,
            DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        ),
        exact(
            NumericalDimension::InputSubnormals,
            DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            }),
        ),
        exact(
            NumericalDimension::ResultSubnormals,
            DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        ),
        exact(
            NumericalDimension::ResultSubnormals,
            DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            }),
        ),
        exact(
            NumericalDimension::Contraction,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        exact(
            NumericalDimension::Contraction,
            DimensionBehaviour::Transform(NumericalPermission::Permitted),
        ),
        exact(
            NumericalDimension::Reassociation,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        exact(
            NumericalDimension::Reassociation,
            DimensionBehaviour::Transform(NumericalPermission::Permitted),
        ),
        exact(
            NumericalDimension::Permutation,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        exact(
            NumericalDimension::SignedZero,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        // The two elementary dimensions follow the contraction/reassociation
        // idiom above: both resolutions of each are declared, and each is
        // honoured exactly rather than approximately. `Forbidden` is delivered
        // by the arithmetic actually performed — nothing in the governed
        // prototype path substitutes a reciprocal for a division or selects an
        // approximate intrinsic. The widened resolutions are honoured for the
        // reassociation row's reason: a permission names a set of legal
        // results, the delivered realization is one of them, and this target
        // runs the one Tiler selected rather than substituting another. The
        // `BackendElementary` envelope in particular is a *maximum* accuracy
        // the caller authorizes consuming, and an exactly rounded elementary
        // result lies within every such envelope, so delivering the precise
        // contract honours the authorization without asserting any
        // approximation authority this profile does not have.
        exact(
            NumericalDimension::ReciprocalTransform,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        exact(
            NumericalDimension::ReciprocalTransform,
            DimensionBehaviour::Transform(NumericalPermission::Permitted),
        ),
        exact(
            NumericalDimension::ApproximateIntrinsics,
            DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden),
        ),
        exact(
            NumericalDimension::ApproximateIntrinsics,
            DimensionBehaviour::Approximation(ApproximationEnvelope::BackendElementary),
        ),
        exact(
            NumericalDimension::NanAssumptions,
            DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        ),
        exact(
            NumericalDimension::InfinityAssumptions,
            DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        ),
    ]
}
