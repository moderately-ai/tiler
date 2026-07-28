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

use crate::call_abi::{CallAbi, ParameterRole};
use crate::call_placement::CallPlacement;
use crate::effects::{Aliasing, CallEffects, Elimination};
use core::fmt;

/// A way two of a call's declarations contradict each other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the coherence vocabulary; lands with the check that produces it, ahead of the registration seam"
)]
pub(crate) enum IncoherentDeclaration {
    /// The ABI declares an in-place parameter while the effects claim results
    /// are distinct from inputs.
    ///
    /// An `InOut` parameter *is* a result occupying an input's storage, so
    /// `Aliasing::Distinct` beside one is not a stricter promise — it is a false
    /// one, and a caller trusting it would reuse storage the call overwrote.
    InPlaceParameterDeclaredDistinct,
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
    reason = "see the enum's own allow: the stable code lands with the vocabulary, ahead of the explain records that will report it"
)]
impl IncoherentDeclaration {
    /// The stable code naming this contradiction.
    pub(crate) const fn code(self) -> &'static str {
        match self {
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
#[allow(
    dead_code,
    reason = "the checked declaration; lands ahead of the registration seam that will carry it"
)]
pub(crate) struct OpaqueCallDeclaration {
    abi: CallAbi,
    effects: CallEffects,
    placement: CallPlacement,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft accessors whose consumer is the not-yet-written registration seam"
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
    ) -> Result<Self, Vec<IncoherentDeclaration>> {
        let mut faults = Vec::new();

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::{AdmittedMemoryDomains, ExecutionAffinity, MemoryDomainClass};
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
        CallAbi::declare(parameters).expect("a well-formed abi")
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
        )
        .expect_err("an in-place parameter with distinct results is incoherent");
        assert!(faults.contains(&IncoherentDeclaration::InPlaceParameterDeclaredDistinct));
    }

    /// A call writing through a parameter cannot be removable.
    #[test]
    fn a_call_that_writes_a_parameter_cannot_be_removable() {
        let faults = OpaqueCallDeclaration::check(
            abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]),
            CallEffects::declared(Elimination::Removable, Motion::Ordered, Aliasing::Distinct),
            placement(),
        )
        .expect_err("a call writing a parameter cannot be removable");
        assert!(
            faults.contains(&IncoherentDeclaration::WritesThroughParameterButDeclaredRemovable)
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
        )
        .expect_err("two contradictions");
        assert_eq!(
            faults.len(),
            2,
            "only {} of two contradictions was reported: {faults:?}",
            faults.len()
        );
    }
}
