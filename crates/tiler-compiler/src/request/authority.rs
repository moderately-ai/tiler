//! What a caller submits, and the installed authority it is bound to.
//!
//! The request schema version, the frozen lowering-capability and scalar
//! authorities every resolved provider is driven through, the exact capability
//! one recognized occurrence resolved to, and the borrowed request itself. It
//! holds no recognized shape and no verified outcome: everything here is stated
//! by the caller or installed before admission runs.

use super::*;

pub(super) const REQUEST_SCHEMA_VERSION: u32 = 2;

/// Returns whether the request carries the program's own environment.
///
/// Compared by the inner `ShapeEnv` pointer, not by a second constructed
/// wrapper: two `ExtentSources` over one `Arc<ShapeEnv>` are one environment,
/// and two independently built environments are two even when their identities
/// happen to encode alike. That is the ambiguity
/// [`tiler_ir::index::IndexRegionBuilder::new_with_shape_environment`] exists
/// to prevent.
pub(super) fn carries_program_environment(
    carried: Option<&ExtentSources>,
    program: &SemanticProgram,
) -> bool {
    match (carried, program.extent_sources()) {
        (None, None) => true,
        (Some(carried), Some(owned)) => std::ptr::eq(carried.environment(), owned.environment()),
        _ => false,
    }
}

/// The exact lowering capability whose provider realized one occurrence.
///
/// Both halves are retained because ADR 0072 keeps them separate: the
/// [`ProviderIdentity`] revision is the admitting provider's own
/// output-affecting revision, and the [`LoweringCapabilityRevision`] covers the
/// exact lowering that provider registered for this family and signature. One
/// provider may own several capabilities at independent revisions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct LoweringProviderIdentity {
    provider: ProviderIdentity,
    subject: LoweringCapabilitySubject,
    capability_revision: LoweringCapabilityRevision,
}

impl LoweringProviderIdentity {
    /// Binds one resolved capability's provider, structured subject, and revision.
    pub(crate) const fn new(
        provider: ProviderIdentity,
        subject: LoweringCapabilitySubject,
        capability_revision: LoweringCapabilityRevision,
    ) -> Self {
        Self {
            provider,
            subject,
            capability_revision,
        }
    }

    /// Returns the exact lowering capability that lowered the occurrence.
    pub(crate) const fn subject(&self) -> &LoweringCapabilitySubject {
        &self.subject
    }

    /// Returns the admitting provider identity.
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the resolved capability's output-affecting revision.
    pub(crate) const fn capability_revision(&self) -> LoweringCapabilityRevision {
        self.capability_revision
    }
}

/// The installed lowering authority one compilation request is bound to.
///
/// The snapshot carries the frozen lowering-capability registry the compile path
/// resolves every recognized occurrence through, together with the exact frozen
/// scalar authority that registry was registered against. Neither is a
/// compile-time constant: an out-of-crate provider registered into the registry
/// drives compilation the same way the governed profile does.
#[derive(Clone, Debug)]
pub(crate) struct CompilerCapabilitySnapshot {
    pub(super) schema_version: u32,
    pub(super) lowering: FrozenLoweringCapabilityRegistry,
    pub(super) scalars: FrozenScalarRegistry,
}

impl CompilerCapabilitySnapshot {
    /// Binds one installed lowering registry and the scalar authority it was
    /// registered against.
    pub(crate) fn new(
        lowering: FrozenLoweringCapabilityRegistry,
        scalars: FrozenScalarRegistry,
    ) -> Self {
        Self {
            schema_version: REQUEST_SCHEMA_VERSION,
            lowering,
            scalars,
        }
    }

    /// Returns the lowering capabilities the bounded profile ships with.
    ///
    /// The snapshot is assembled once and shared. Assembly is deterministic and
    /// depends on nothing outside this crate and `tiler-ir`.
    ///
    /// # Panics
    ///
    /// Panics when Tiler's own governed profile violates the public capability
    /// contract, which is a defect in this crate rather than a caller error.
    pub(crate) fn governed() -> Self {
        static GOVERNED: OnceLock<CompilerCapabilitySnapshot> = OnceLock::new();
        GOVERNED
            .get_or_init(|| {
                let scalars =
                    governed_scalars().expect("the governed scalar authority is well formed");
                let lowering = governed_lowering_capabilities(&scalars)
                    .expect("the governed lowering capabilities are well formed");
                Self::new(lowering, scalars)
            })
            .clone()
    }

    /// Returns the installed lowering-capability registry.
    pub(crate) const fn lowering(&self) -> &FrozenLoweringCapabilityRegistry {
        &self.lowering
    }

    /// Returns the scalar authority every resolved provider emits against.
    pub(crate) const fn scalars(&self) -> &FrozenScalarRegistry {
        &self.scalars
    }

    /// Returns the registry's canonical provenance.
    pub(crate) fn registry_identity(&self) -> &CanonicalLoweringRegistryIdentity {
        self.lowering.canonical_identity()
    }

    /// Returns a snapshot whose registry admits no lowering capability at all.
    ///
    /// It is the smallest installed authority that still pairs correctly with
    /// the governed scalar profile, so a fixture can distinguish "the registry
    /// resolved nothing" from "the request was malformed".
    #[cfg(test)]
    pub(crate) fn without_capabilities() -> Self {
        let scalars = governed_scalars().expect("the governed scalar authority is well formed");
        let lowering = crate::capability::LoweringCapabilityRegistryBuilder::new(
            scalars.semantic_authority().clone(),
            scalars.clone(),
        )
        .expect("the governed scalar registry retains its exact semantic authority")
        .freeze();
        Self::new(lowering, scalars)
    }
}

/// Two snapshots are equal exactly when their declared authority is.
///
/// The canonical registry identity binds every registered capability's family,
/// operation, signature, provider, capability revision, and reached authority,
/// together with the composed semantic and scalar snapshots. Provider
/// implementations are deliberately outside it: a provider whose emitted
/// lowering changes must raise its capability revision, which is inside it.
impl PartialEq for CompilerCapabilitySnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.registry_identity() == other.registry_identity()
            && self.scalars.snapshot_identity() == other.scalars.snapshot_identity()
    }
}

impl Eq for CompilerCapabilitySnapshot {}

#[derive(Clone, Debug)]
pub(crate) struct CompilationRequest<'a> {
    pub(crate) program: &'a SemanticProgram,
    /// The program's own environment, never a second caller-supplied one.
    ///
    /// `None` when the program has only literal extents. Two environments over
    /// one program is the ambiguity
    /// [`tiler_ir::index::IndexRegionBuilder::new_with_shape_environment`]
    /// exists to prevent; [`verify_request`] refuses a request that does not
    /// carry this exact environment.
    pub(crate) shape_environment: Option<&'a ExtentSources>,
    /// The caller's ordered numerical-contract preference. Required, with no
    /// `Default` and no ambient fallback (ADR 0076 item 2).
    pub(crate) numerical_contracts: NumericalContractPreference,
    pub(crate) budgets: DeterministicBudgets,
    pub(crate) target_profiles: Vec<TargetProfile>,
    pub(crate) capabilities: CompilerCapabilitySnapshot,
}

impl CompilationRequest<'_> {
    /// Builds the fixed governed compilation-request fixture.
    ///
    /// Conformance and unit tests use this exact combination of the program's
    /// own shape environment, strict-`f32` numerical contract, deterministic
    /// budgets, target profile, and installed lowering capabilities. Production
    /// resolves the public caller's stated preference through
    /// [`Self::governed_preferring`].
    #[allow(
        dead_code,
        reason = "crate-internal fixed governed fixture used by conformance and unit tests; the public CompileRequest path resolves caller preferences through governed_preferring until a production caller needs this exact fixture"
    )]
    pub(crate) fn governed(program: &SemanticProgram) -> CompilationRequest<'_> {
        Self::governed_under(program, StrictF32NumericalContract::governed())
    }

    /// Builds the governed request under one caller-stated numerical contract.
    ///
    /// The contract is a parameter with no default. On the measured Apple row
    /// the strictest reading is unhonourable, so a strict default would make
    /// every Apple compilation fail with a rejection the caller never asked for
    /// and leave the knob reachable only by reading that rejection.
    pub(crate) fn governed_under(
        program: &SemanticProgram,
        numerical_contract: StrictF32NumericalContract,
    ) -> CompilationRequest<'_> {
        Self::governed_preferring(
            program,
            NumericalContractPreference::exactly(numerical_contract),
        )
    }

    /// Builds the governed request under a caller-stated ordered preference.
    ///
    /// The list is resolved by the caller's stated order against each target's
    /// declared honourability; the first honourable entry wins. No authority
    /// below this boundary may reorder it, and none may rank the entries by cost.
    pub(crate) fn governed_preferring(
        program: &SemanticProgram,
        numerical_contracts: NumericalContractPreference,
    ) -> CompilationRequest<'_> {
        CompilationRequest {
            program,
            shape_environment: program.extent_sources(),
            numerical_contracts,
            budgets: DeterministicBudgets::governed(),
            target_profiles: vec![TargetProfile::governed()],
            capabilities: CompilerCapabilitySnapshot::governed(),
        }
    }
}
