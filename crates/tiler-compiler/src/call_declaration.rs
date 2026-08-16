//! An opaque call's four declarations, checked against each other.
//!
//! A slice of `implement-opaque-physical-call-providers`. The ABI
//! ([`crate::call_abi`]), the effects ([`crate::effects`]), the placement
//! ([`crate::call_placement`]), and the pressure estimates
//! ([`crate::estimate`]) each validate on their own. This module checks the
//! thing none of them can see: whether they **agree**.
//!
//! # Why a separate check rather than richer constructors
//!
//! Each declaration is built by the provider independently, and each is
//! individually well formed in ways that still contradict its siblings. An ABI
//! does not know what effects were declared; an effect declaration does not know
//! the parameter list. Pushing the cross-check into either constructor would
//! mean one of them taking the other as an argument, which fixes a construction
//! order that providers have no reason to share.
//!
//! # Why a contradiction is a defect and not a rejection
//!
//! A provider whose declarations disagree has not described a call that this
//! compiler cannot run — it has described no call at all, since there is no
//! single behaviour consistent with what it said. That is the same distinction
//! [`crate::rewrite::ProviderDefect`] draws: a rejection is an ordinary
//! outcome, and a contract violation is not. Reporting one as the other would
//! let a caller counting infeasible candidates count broken providers among
//! them.
//!
//! # Applicability is deliberately absent
//!
//! `crate::frontier::TargetApplicability` already resolves which providers
//! apply to a target profile, over governed `TargetProfileKey`s, with canonical
//! deduplicated ordering. An opaque-call provider uses that rather than a
//! second predicate over the same question.

use crate::boundary::{
    AlignmentGuarantee, AlignmentRequirement, AvailabilityGuarantee, AvailabilityRequirement,
    GuaranteedProperties, GuaranteedProperty, MaterializationForm, RequiredProperties,
    RequiredProperty, VisibilityGuarantee, VisibilityRequirement,
};
use crate::call_abi::{CallAbi, CallParameter, ParameterLayout, ParameterRole};
use crate::call_placement::CallPlacement;
use crate::effects::{Aliasing, CallEffects, Elimination};
use core::fmt;
use tiler_ir::schedule::ResourceRequirements;

/// How many work items a dispatch of the call performs.
///
/// `physical::assess_region` proves resources against a target profile and needs
/// this count. A scheduled region reads it from its schedule; an opaque call has
/// none, so it declares how its work scales.
///
/// # Why not a plain number
///
/// A fixed count is honest for a call that does the same work whatever it is
/// given, and wrong for most real ones — a call over a tensor usually does work
/// proportional to that tensor. Declaring a bare number would force
/// shape-dependent calls to either lie or refuse to be declared, and a lie here
/// is a feasibility verdict that is confidently incorrect: too small admits a
/// call the target cannot run, too large rejects one it can.
///
/// # Why per-parameter rather than per-call
///
/// The count follows a *particular* tensor. A call reducing a large input to a
/// small output does work proportional to the input, not the output, and only
/// the call knows which. Naming the parameter says so; naming nothing would
/// leave the frontier to guess.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "no production opaque-call provider constructs work scaling; frontier admission consumes it from test providers until caller-supplied physical providers reach the compile path"
)]
pub(crate) enum WorkScaling {
    /// One work item per element of the tensor bound to this parameter.
    PerElementOf(&'static str),
    /// A fixed count, whatever the call is given.
    Fixed(u64),
}

/// A way two of a call's declarations contradict each other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "provider-side declaration checking produces this vocabulary, and production installs no opaque-call provider until caller-supplied physical providers reach the compile path"
)]
pub(crate) enum IncoherentDeclaration {
    /// The ABI declares an in-place parameter while the effects claim results
    /// are distinct from inputs.
    ///
    /// An `InOut` parameter *is* a result occupying an input's storage, so
    /// `Aliasing::Distinct` beside one is not a stricter promise — it is a false
    /// one, and a caller trusting it would reuse storage the call overwrote.
    InPlaceParameterDeclaredDistinct,
    /// The work scaling names a parameter the ABI does not declare.
    ///
    /// A scaling that follows a parameter nobody declared cannot be evaluated,
    /// and the resulting work count would have to be invented — which is a
    /// feasibility verdict nothing supports.
    WorkScalingNamesUnknownParameter(&'static str),
    /// The call declares fewer buffer bindings than it has parameters.
    ///
    /// Every parameter must be bound, so a binding count below the parameter
    /// count describes a call that cannot be invoked. Caught here rather than at
    /// dispatch because the two numbers come from different declarations and
    /// neither can see the other.
    FewerBindingsThanParameters {
        /// Buffer bindings the resources declare.
        bindings: u32,
        /// Parameters the ABI declares.
        parameters: usize,
    },
    /// The effects claim the call is removable while the ABI declares a
    /// parameter it writes that is not among its results.
    ///
    /// A call that writes storage a caller handed it is observable through that
    /// storage, whether or not anything reads a returned value. Declaring it
    /// removable would let dead-result elimination discard a write the caller
    /// is relying on.
    WritesThroughParameterButDeclaredRemovable,
}

#[allow(
    dead_code,
    reason = "test diagnostics read the stable contradiction code; production has no provider-side declaration check to report until caller-supplied physical providers reach the compile path"
)]
impl IncoherentDeclaration {
    /// The stable code naming this contradiction.
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::FewerBindingsThanParameters { .. } => {
                "declaration.fewer-bindings-than-parameters"
            }
            Self::WorkScalingNamesUnknownParameter(_) => {
                "declaration.work-scaling-unknown-parameter"
            }
            Self::InPlaceParameterDeclaredDistinct => "declaration.inplace-declared-distinct",
            Self::WritesThroughParameterButDeclaredRemovable => "declaration.writes-but-removable",
        }
    }
}

impl fmt::Display for IncoherentDeclaration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

/// One opaque call's complete declaration, with its parts checked against each
/// other.
///
/// Holding one is evidence that the four declarations are mutually consistent —
/// not that the call is feasible, which is a separate question asked against a
/// target profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OpaqueCallDeclaration {
    abi: CallAbi,
    effects: CallEffects,
    placement: CallPlacement,
    /// How many work items a dispatch performs; see [`WorkScaling`].
    work: WorkScaling,
    /// Exact or proven-upper-bound requirements, used for hard feasibility.
    ///
    /// The first of the ticket's three evidence classes, and the only one
    /// feasibility may consult. A provider that wants its call admitted must
    /// state requirements it can **prove** — an uncertain
    /// [`crate::estimate::ResourceEstimate`] deliberately has no conversion into
    /// this, and that absence is what stops an unproven number deciding whether
    /// a plan is legal.
    resources: ResourceRequirements,
}

#[allow(
    dead_code,
    reason = "declaration checking is provider-side and test-exercised; production frontier admission consumes already-checked declarations until caller-supplied physical providers populate the registry"
)]
impl OpaqueCallDeclaration {
    /// Checks the declarations against each other and bundles them.
    ///
    /// Returns **every** contradiction found rather than the first, in a stable
    /// order, so a provider author fixing one does not have to resubmit to
    /// discover the next. This mirrors `boundary::unsatisfied_properties`, which
    /// collects for the same reason.
    pub(crate) fn check(
        abi: CallAbi,
        effects: CallEffects,
        placement: CallPlacement,
        resources: ResourceRequirements,
        work: WorkScaling,
    ) -> Result<Self, Vec<IncoherentDeclaration>> {
        let mut faults = Vec::new();

        if let WorkScaling::PerElementOf(name) = work
            && abi.parameter(name).is_none()
        {
            faults.push(IncoherentDeclaration::WorkScalingNamesUnknownParameter(
                name,
            ));
        }

        if (resources.buffer_bindings as usize) < abi.parameters().len() {
            faults.push(IncoherentDeclaration::FewerBindingsThanParameters {
                bindings: resources.buffer_bindings,
                parameters: abi.parameters().len(),
            });
        }

        let has_in_place = abi
            .parameters()
            .iter()
            .any(|parameter| parameter.role() == ParameterRole::InOut);
        if has_in_place && effects.aliasing() == Aliasing::Distinct {
            faults.push(IncoherentDeclaration::InPlaceParameterDeclaredDistinct);
        }

        let writes_through_parameter = abi
            .parameters()
            .iter()
            .any(|parameter| parameter.role().writes());
        if writes_through_parameter && effects.elimination() == Elimination::Removable {
            faults.push(IncoherentDeclaration::WritesThroughParameterButDeclaredRemovable);
        }

        if faults.is_empty() {
            Ok(Self {
                abi,
                effects,
                placement,
                work,
                resources,
            })
        } else {
            Err(faults)
        }
    }

    /// The checked ABI.
    pub(crate) const fn abi(&self) -> &CallAbi {
        &self.abi
    }

    /// The checked effect declaration.
    pub(crate) const fn effects(&self) -> CallEffects {
        self.effects
    }

    /// The checked placement.
    pub(crate) const fn placement(&self) -> &CallPlacement {
        &self.placement
    }

    /// How many work items a dispatch performs.
    pub(crate) const fn work(&self) -> WorkScaling {
        self.work
    }

    /// The proven resource requirements hard feasibility consults.
    pub(crate) const fn resources(&self) -> &ResourceRequirements {
        &self.resources
    }
}

/// The typed properties a call requires of the tensor bound to one parameter.
///
/// The first half of the boundary derivation. Each property has exactly one
/// authority in the declaration, which is why the declaration has the parts it
/// does:
///
/// - layout, encoding, alignment — the parameter's own spec, which states them
///   in the direction its role has;
/// - execution affinity and admitted memory domains — the placement;
/// - availability and visibility — fixed for a *read*: the call needs the value
///   after the producing dispatch and readable without a further coherence
///   action, which is what reading it at all means.
///
/// Materialization is `MaterializedBuffer` and does not come from the effects.
/// The effects' `Aliasing` says whether a *result* may share storage with an
/// input; it says nothing about the form an input arrives in, and reading it as
/// though it did would let a call that returns views also declare it accepts
/// them.
///
/// # Errors
///
/// Returns `None` when the parameter's layout does not state a requirement,
/// which `CallAbi::declare` already refuses for a read role — so this is
/// unreachable through a checked ABI and is `Option` rather than a panic
/// because that reachability is an invariant of another type, not of this one.
pub(crate) fn required_properties_for(
    parameter: &CallParameter,
    placement: &CallPlacement,
) -> Option<RequiredProperties> {
    let layout = match parameter.spec().layout {
        ParameterLayout::Required(layout)
        | ParameterLayout::Both {
            requires: layout, ..
        } => layout,
        ParameterLayout::Guaranteed(_) => return None,
    };
    RequiredProperties::new([
        RequiredProperty::StorageLayout(layout),
        RequiredProperty::StorageEncoding(parameter.spec().encoding),
        RequiredProperty::Alignment(AlignmentRequirement::from_alignment(
            parameter.spec().alignment,
        )),
        RequiredProperty::Materialization(MaterializationForm::MaterializedBuffer),
        RequiredProperty::ExecutionAffinity(placement.affinity()),
        RequiredProperty::MemoryDomain(placement.domains().clone()),
        RequiredProperty::Availability(AvailabilityRequirement::AfterProducingDispatch),
        RequiredProperty::Visibility(VisibilityRequirement::ReadableOnRequiringAffinity),
    ])
    .ok()
}

/// Why a guarantee could not be derived for a parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GuaranteeError {
    /// The parameter is read-only, so there is nothing it guarantees.
    NotAWrite,
    /// The placement admits more than one memory domain, so the call has not
    /// said which one it writes into.
    ///
    /// A requirement names an admitted *set* and a guarantee names the one
    /// domain an allocation is in — `crate::boundary` documents that asymmetry
    /// deliberately. A call admitting two and guaranteeing neither has not
    /// described where its output lives, and picking one would be inventing the
    /// answer.
    AmbiguousWriteDomain,
}

/// The typed properties a call guarantees of the tensor bound to one write
/// parameter.
///
/// The mirror of [`required_properties_for`], and the differences are the ones
/// the boundary vocabulary makes deliberately:
///
/// - the layout is the spec's *guaranteed* side;
/// - availability is `AfterOwnDispatch` and visibility `CoherentOnProducingAffinity`,
///   which is what producing a value means;
/// - the memory domain is one class, not a set — see [`GuaranteeError::AmbiguousWriteDomain`];
/// - **materialization comes from the effects here**, and only here. `Aliasing`
///   is a statement about *results*, so `MayAliasInputs` makes this an
///   `AliasView` and `Distinct` makes it a `MaterializedBuffer`. The requirement
///   side does not consult it, because it says nothing about incoming values.
pub(crate) fn guaranteed_properties_for(
    parameter: &CallParameter,
    effects: CallEffects,
    placement: &CallPlacement,
) -> Result<GuaranteedProperties, GuaranteeError> {
    let layout = match parameter.spec().layout {
        ParameterLayout::Guaranteed(layout)
        | ParameterLayout::Both {
            guarantees: layout, ..
        } => layout,
        ParameterLayout::Required(_) => return Err(GuaranteeError::NotAWrite),
    };
    let [domain] = placement.domains().classes() else {
        return Err(GuaranteeError::AmbiguousWriteDomain);
    };
    let materialization = match effects.aliasing() {
        Aliasing::Distinct => MaterializationForm::MaterializedBuffer,
        Aliasing::MayAliasInputs => MaterializationForm::AliasView,
    };
    GuaranteedProperties::new([
        GuaranteedProperty::StorageLayout(layout),
        GuaranteedProperty::StorageEncoding(parameter.spec().encoding),
        GuaranteedProperty::Alignment(AlignmentGuarantee::from_alignment(
            parameter.spec().alignment,
        )),
        GuaranteedProperty::Materialization(materialization),
        GuaranteedProperty::ExecutionAffinity(placement.affinity()),
        GuaranteedProperty::MemoryDomain(*domain),
        GuaranteedProperty::Availability(AvailabilityGuarantee::AfterOwnDispatch),
        GuaranteedProperty::Visibility(VisibilityGuarantee::CoherentOnProducingAffinity),
    ])
    .map_err(|_| GuaranteeError::NotAWrite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiler_ir::schedule::{ExceptionalValueAssumption, NumericalPermission, SubnormalMode};

    /// Resources ample enough that only the fault under test can fire.
    fn resources(bindings: u32) -> ResourceRequirements {
        ResourceRequirements {
            buffer_bindings: bindings,
            threads_per_workgroup: 1,
            local_memory_bytes: 0,
            requires_device_memory: true,
            index_arithmetic: tiler_ir::schedule::IndexArithmetic::CompleteU64,
            synchronization: None,
            subgroup: None,
            input_subnormals: SubnormalMode::Preserve,
            result_subnormals: SubnormalMode::Preserve,
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
            permutation: NumericalPermission::Forbidden,
            signed_zero: NumericalPermission::Forbidden,
            nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
        }
    }

    use crate::boundary::{
        AdmittedMemoryDomains, ByteAlignment, ExecutionAffinity, LayoutGuarantee,
        LayoutRequirement, MemoryDomainClass, StorageEncoding, StorageScalar,
    };
    use crate::call_abi::{ParameterLayout, ParameterSpec};
    /// A spec carrying the bounded profile's storage answers.
    fn spec(name: &'static str, role: ParameterRole) -> ParameterSpec {
        ParameterSpec {
            name,
            role,
            layout: match role {
                ParameterRole::In => ParameterLayout::Required(LayoutRequirement::DenseRowMajor),
                ParameterRole::Out => ParameterLayout::Guaranteed(LayoutGuarantee::DenseRowMajor),
                ParameterRole::InOut => ParameterLayout::Both {
                    requires: LayoutRequirement::DenseRowMajor,
                    guarantees: LayoutGuarantee::DenseRowMajor,
                },
            },
            encoding: StorageEncoding::Unpacked,
            alignment: ByteAlignment::natural_for(StorageScalar::F32),
        }
    }

    use crate::effects::Motion;

    fn placement() -> CallPlacement {
        CallPlacement::declare(
            ExecutionAffinity::PRIMARY,
            AdmittedMemoryDomains::new([MemoryDomainClass::Device]).expect("non-empty"),
            &[MemoryDomainClass::Device],
        )
        .expect("supported")
    }

    fn abi(parameters: impl IntoIterator<Item = (&'static str, ParameterRole)>) -> CallAbi {
        CallAbi::declare(parameters.into_iter().map(|(name, role)| spec(name, role)))
            .expect("a well-formed abi")
    }

    /// Consistent declarations are admitted.
    ///
    /// Without this the two rejection tests below would pass against a `check`
    /// that refused everything.
    #[test]
    fn consistent_declarations_are_admitted() {
        let declaration = OpaqueCallDeclaration::check(
            abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]),
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct),
            placement(),
            resources(8),
            WorkScaling::Fixed(1),
        );
        assert!(
            declaration.is_ok(),
            "consistent declarations were rejected: {declaration:?}"
        );
    }

    /// An in-place parameter contradicts a distinct-results claim.
    #[test]
    fn an_in_place_parameter_cannot_claim_distinct_results() {
        let faults = OpaqueCallDeclaration::check(
            abi([("buffer", ParameterRole::InOut)]),
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct),
            placement(),
            resources(8),
            WorkScaling::Fixed(1),
        )
        .expect_err("an in-place parameter with distinct results is incoherent");
        assert!(faults.contains(&IncoherentDeclaration::InPlaceParameterDeclaredDistinct));
    }

    /// The generic call ABI remains capable of stating a coherent in-place
    /// parameter even though regional opaque-call binding currently refuses it.
    #[test]
    fn an_in_place_parameter_with_may_alias_inputs_is_coherent() {
        let declaration = OpaqueCallDeclaration::check(
            abi([("buffer", ParameterRole::InOut)]),
            CallEffects::declared(
                Elimination::Required,
                Motion::Ordered,
                Aliasing::MayAliasInputs,
            ),
            placement(),
            resources(1),
            WorkScaling::Fixed(1),
        );
        assert_eq!(
            declaration
                .as_ref()
                .map(|declaration| declaration.abi().parameters().len()),
            Ok(1),
            "the lower-level ABI must not inherit the regional refusal: {declaration:?}",
        );
    }

    /// A call writing through a parameter cannot be removable.
    #[test]
    fn a_call_that_writes_a_parameter_cannot_be_removable() {
        let faults = OpaqueCallDeclaration::check(
            abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]),
            CallEffects::declared(Elimination::Removable, Motion::Ordered, Aliasing::Distinct),
            placement(),
            resources(8),
            WorkScaling::Fixed(1),
        )
        .expect_err("a call writing a parameter cannot be removable");
        assert!(
            faults.contains(&IncoherentDeclaration::WritesThroughParameterButDeclaredRemovable)
        );
    }

    /// A call binding fewer buffers than it has parameters cannot be invoked.
    ///
    /// Driven against a sufficient count too, so a check comparing the wrong way
    /// round — or refusing everything — fails here rather than passing.
    #[test]
    fn fewer_bindings_than_parameters_is_incoherent() {
        let two_parameters = || abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let effects =
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct);

        assert!(
            OpaqueCallDeclaration::check(
                two_parameters(),
                effects,
                placement(),
                resources(2),
                WorkScaling::Fixed(1),
            )
            .is_ok(),
            "exactly enough bindings was refused"
        );

        let faults = OpaqueCallDeclaration::check(
            two_parameters(),
            effects,
            placement(),
            resources(1),
            WorkScaling::Fixed(1),
        )
        .expect_err("one binding cannot serve two parameters");
        assert!(
            faults.contains(&IncoherentDeclaration::FewerBindingsThanParameters {
                bindings: 1,
                parameters: 2,
            })
        );
    }

    /// Every contradiction is reported, not only the first.
    ///
    /// A provider author fixing one should not have to resubmit to find the
    /// next. A `check` returning early would pass both tests above and fail
    /// this one.
    #[test]
    fn every_contradiction_is_reported() {
        let faults = OpaqueCallDeclaration::check(
            abi([("buffer", ParameterRole::InOut)]),
            CallEffects::declared(Elimination::Removable, Motion::Ordered, Aliasing::Distinct),
            placement(),
            resources(8),
            WorkScaling::Fixed(1),
        )
        .expect_err("two contradictions");
        assert_eq!(
            faults.len(),
            2,
            "only {} of two contradictions was reported: {faults:?}",
            faults.len()
        );
    }

    /// Every governed property is stated, and each from its own authority.
    ///
    /// The count assertion is what catches a derivation that silently omits a
    /// dimension: a requirement no guarantee speaks to fails closed, so an
    /// omitted property would make the boundary compose only by accident.
    #[test]
    fn a_parameter_requirement_states_every_governed_property() {
        let abi = abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let parameter = abi.parameter("input").expect("declared");
        let properties = required_properties_for(parameter, &placement())
            .expect("a read parameter states a requirement");

        assert_eq!(
            properties.properties().len(),
            crate::boundary::CANONICAL_PROPERTIES.len(),
            "the derivation omitted a governed property"
        );
        for property in crate::boundary::CANONICAL_PROPERTIES {
            assert!(
                properties.get(property).is_some(),
                "{property} was not derived"
            );
        }
    }

    /// The affinity and domains come from the placement, not from a default.
    #[test]
    fn placement_supplies_the_affinity_and_domains() {
        let abi = abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let parameter = abi.parameter("input").expect("declared");
        let placement = placement();
        let properties = required_properties_for(parameter, &placement).expect("a read parameter");

        assert_eq!(
            properties.get(crate::boundary::BoundaryProperty::ExecutionAffinity),
            Some(&RequiredProperty::ExecutionAffinity(placement.affinity()))
        );
        assert_eq!(
            properties.get(crate::boundary::BoundaryProperty::MemoryDomain),
            Some(&RequiredProperty::MemoryDomain(placement.domains().clone()))
        );
    }

    /// A write-only parameter states no requirement.
    ///
    /// Its layout is a guarantee, so there is nothing to require — and returning
    /// a requirement anyway would put a made-up layout into a contract.
    #[test]
    fn a_write_only_parameter_states_no_requirement() {
        let abi = abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let parameter = abi.parameter("output").expect("declared");
        assert!(
            required_properties_for(parameter, &placement()).is_none(),
            "an output parameter produced a requirement"
        );
    }

    /// A write parameter guarantees every governed property.
    #[test]
    fn a_parameter_guarantee_states_every_governed_property() {
        let abi = abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let parameter = abi.parameter("output").expect("declared");
        let effects =
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct);
        let properties = guaranteed_properties_for(parameter, effects, &placement())
            .expect("a write parameter guarantees");

        assert_eq!(
            properties.properties().len(),
            crate::boundary::CANONICAL_PROPERTIES.len(),
            "the derivation omitted a governed property"
        );
        assert_eq!(
            properties.get(crate::boundary::BoundaryProperty::Availability),
            Some(&GuaranteedProperty::Availability(
                AvailabilityGuarantee::AfterOwnDispatch
            ))
        );
    }

    /// Aliasing decides materialization on the guarantee side, and only there.
    ///
    /// `MayAliasInputs` means a result occupies an input's storage, which is an
    /// alias view rather than a buffer. The requirement side must not move with
    /// it — a call that returns views does not thereby accept them.
    #[test]
    fn aliasing_decides_the_guaranteed_materialization_only() {
        let abi = abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let output = abi.parameter("output").expect("declared");
        let input = abi.parameter("input").expect("declared");

        for (aliasing, expected) in [
            (Aliasing::Distinct, MaterializationForm::MaterializedBuffer),
            (Aliasing::MayAliasInputs, MaterializationForm::AliasView),
        ] {
            let effects = CallEffects::declared(Elimination::Required, Motion::Ordered, aliasing);
            let guaranteed = guaranteed_properties_for(output, effects, &placement())
                .expect("a write parameter");
            assert_eq!(
                guaranteed.get(crate::boundary::BoundaryProperty::Materialization),
                Some(&GuaranteedProperty::Materialization(expected)),
                "aliasing did not decide the guaranteed materialization"
            );

            let required = required_properties_for(input, &placement()).expect("a read parameter");
            assert_eq!(
                required.get(crate::boundary::BoundaryProperty::Materialization),
                Some(&RequiredProperty::Materialization(
                    MaterializationForm::MaterializedBuffer
                )),
                "aliasing moved the required materialization, which it must not"
            );
        }
    }

    /// A read-only parameter guarantees nothing, and an ambiguous write domain
    /// is refused rather than resolved.
    #[test]
    fn a_read_parameter_and_an_ambiguous_domain_are_refused() {
        let abi = abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let effects =
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct);

        assert_eq!(
            guaranteed_properties_for(
                abi.parameter("input").expect("declared"),
                effects,
                &placement()
            ),
            Err(GuaranteeError::NotAWrite)
        );

        let two_domains = CallPlacement::declare(
            ExecutionAffinity::PRIMARY,
            AdmittedMemoryDomains::new([MemoryDomainClass::Device, MemoryDomainClass::Shared])
                .expect("non-empty"),
            &[MemoryDomainClass::Device, MemoryDomainClass::Shared],
        )
        .expect("supported");
        assert_eq!(
            guaranteed_properties_for(
                abi.parameter("output").expect("declared"),
                effects,
                &two_domains
            ),
            Err(GuaranteeError::AmbiguousWriteDomain),
            "a call admitting two domains guaranteed one anyway"
        );
    }

    /// Work scaling must name a parameter the ABI declares.
    ///
    /// A scaling following a parameter nobody declared cannot be evaluated, and
    /// the work count would have to be invented — which is a feasibility verdict
    /// nothing supports. Both accepting forms are driven, so a check refusing
    /// everything fails here.
    #[test]
    fn work_scaling_must_name_a_declared_parameter() {
        let two = || abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let effects =
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct);

        for work in [WorkScaling::Fixed(64), WorkScaling::PerElementOf("input")] {
            assert!(
                OpaqueCallDeclaration::check(two(), effects, placement(), resources(8), work,)
                    .is_ok(),
                "a well-formed scaling was refused: {work:?}"
            );
        }

        let faults = OpaqueCallDeclaration::check(
            two(),
            effects,
            placement(),
            resources(8),
            WorkScaling::PerElementOf("absent"),
        )
        .expect_err("a scaling naming an undeclared parameter is incoherent");
        assert!(
            faults.contains(&IncoherentDeclaration::WorkScalingNamesUnknownParameter(
                "absent"
            ))
        );
    }
}
