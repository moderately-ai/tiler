//! Where an opaque call runs, and which storage it may address.
//!
//! A slice of `implement-opaque-physical-call-providers`.
//!
//! # Why this reuses the boundary vocabulary
//!
//! Placement and memory domains already have a governed vocabulary in
//! [`crate::boundary`] — [`ExecutionAffinity`] and [`MemoryDomainClass`] — built
//! for the same ADR 0047 contract this ticket is subject to. Declaring a second
//! set here would make two authorities over one concept, and the failure mode is
//! the one `AGENTS.md` names directly: two types with the same shape are not the
//! same concept, and a reader matching one against the other draws a confident
//! wrong conclusion. So this module adds no domain vocabulary; it adds only the
//! *declaration* an opaque call makes in that vocabulary.
//!
//! # The rule
//!
//! An opaque call must state where it runs and what it may address, and an
//! undeclared placement is **refused**, not defaulted. Defaulting to the
//! bounded profile's device affinity would be convenient and would be the same
//! error as a permissive effect default: the compiler cannot see the call's
//! body, so a placement it did not state is a placement nobody knows.

use crate::boundary::{AdmittedMemoryDomains, ExecutionAffinity, MemoryDomainClass};
use core::fmt;

/// Why a placement declaration was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "declaration outcome for the opaque-call seam this vocabulary is being built for"
)]
///
/// There is deliberately no "admits no domain" variant. `AdmittedMemoryDomains`
/// already refuses an empty set at construction, so a check for one here could
/// never fire — and an error variant that cannot be reached reads as a check
/// while being none, which is worse than its absence.
pub(crate) enum PlacementError {
    /// The call declares a domain the compiler does not allocate in.
    ///
    /// Carries the offending class so the rejection names it rather than only
    /// reporting that something was wrong.
    UnsupportedDomain(MemoryDomainClass),
}

impl fmt::Display for PlacementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDomain(class) => write!(
                formatter,
                "placement.unsupported-domain: {class} is not allocated by this profile"
            ),
        }
    }
}

/// Where an opaque call runs and what storage it may address.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the placement declaration; lands ahead of the provider seam that carries it"
)]
pub(crate) struct CallPlacement {
    affinity: ExecutionAffinity,
    domains: AdmittedMemoryDomains,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft accessors whose consumer is the not-yet-written opaque-call seam"
)]
impl CallPlacement {
    /// Declares a placement, refusing one this profile cannot satisfy.
    ///
    /// `supported` is the set of classes the compiler actually allocates in —
    /// passed rather than read from a constant, so a widened profile does not
    /// require editing this module and a test can drive the rejection path
    /// without a profile that permits it.
    pub(crate) fn declare(
        affinity: ExecutionAffinity,
        domains: AdmittedMemoryDomains,
        supported: &[MemoryDomainClass],
    ) -> Result<Self, PlacementError> {
        if let Some(unsupported) = domains
            .classes()
            .iter()
            .find(|class| !supported.contains(class))
        {
            return Err(PlacementError::UnsupportedDomain(*unsupported));
        }
        Ok(Self { affinity, domains })
    }

    /// The affinity the call runs on.
    pub(crate) const fn affinity(&self) -> ExecutionAffinity {
        self.affinity
    }

    /// The memory-domain classes the call may address.
    pub(crate) const fn domains(&self) -> &AdmittedMemoryDomains {
        &self.domains
    }

    /// Whether a value in `class`, produced on `affinity`, is reachable by this
    /// call without a transfer.
    ///
    /// Both conditions, deliberately. A value in an admitted domain but on
    /// another affinity still needs a transfer, and a value on the right
    /// affinity in an unadmitted domain still needs one. Answering with either
    /// alone would report a call as able to read storage it cannot address.
    ///
    /// Only the domain half is currently exercised: the bounded profile has one
    /// symbolic affinity (`ExecutionAffinity::PRIMARY`), so no test can supply a
    /// second one to fail the affinity half against. The conjunction is written
    /// for the profile that has two, and until then it is unverified in that
    /// direction.
    pub(crate) fn reaches(&self, affinity: ExecutionAffinity, class: MemoryDomainClass) -> bool {
        self.affinity == affinity && self.domains.classes().contains(&class)
    }
}

impl fmt::Display for CallPlacement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.affinity, self.domains)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domains(classes: impl IntoIterator<Item = MemoryDomainClass>) -> AdmittedMemoryDomains {
        AdmittedMemoryDomains::new(classes).expect("a non-empty admitted set")
    }

    /// A supported declaration is admitted; an unsupported class is named.
    ///
    /// Driven against the accepting case too, so a `declare` that refused
    /// everything would fail here rather than pass.
    #[test]
    fn an_unsupported_domain_is_refused_by_name() {
        let affinity = ExecutionAffinity::PRIMARY;
        let supported = [MemoryDomainClass::Device];

        assert!(
            CallPlacement::declare(affinity, domains([MemoryDomainClass::Device]), &supported)
                .is_ok(),
            "a device-only placement was refused"
        );
        assert_eq!(
            CallPlacement::declare(
                affinity,
                domains([MemoryDomainClass::Device, MemoryDomainClass::HostVisible]),
                &supported
            ),
            Err(PlacementError::UnsupportedDomain(
                MemoryDomainClass::HostVisible
            )),
            "an unsupported class was admitted, or was refused without being named"
        );
    }

    /// An unadmitted domain is not reachable even on the right affinity.
    ///
    /// This covers only half the conjunction. The affinity half cannot be tested
    /// on a profile with one symbolic affinity — there is no second value to
    /// pass — so a `reaches` that ignored its affinity argument entirely would
    /// still pass here. Stated rather than left for a reader to assume from a
    /// green test.
    #[test]
    fn reaching_a_value_needs_both_the_affinity_and_the_domain() {
        let affinity = ExecutionAffinity::PRIMARY;
        let placement = CallPlacement::declare(
            affinity,
            domains([MemoryDomainClass::Device]),
            &[MemoryDomainClass::Device],
        )
        .expect("supported");

        assert!(placement.reaches(affinity, MemoryDomainClass::Device));
        assert!(
            !placement.reaches(affinity, MemoryDomainClass::Shared),
            "an unadmitted domain was reported reachable on the right affinity"
        );
    }
}
