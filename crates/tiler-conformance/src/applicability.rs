//! Whether this host may **offer** the profile it is about to route under, and
//! the observation that question is asked from.
//!
//! # Two different authority questions, and only one of them is ever claimed
//!
//! **"Is this host eligible to offer the declared profile?"** is asked by
//! [`refuse_to_offer_the_declared_profile`], from a host observation and nothing
//! else, and the answer is always no:
//! [ADR 0086](../../../docs/decisions/0086-require-attributable-or-attested-native-translation.md)
//! decides that native device translation of a metallib during pipeline creation
//! is a typed capability fact whose authority is `Unknown` on every macOS row
//! currently observable, and ADR 0043's disposal of `Unknown` keeps an unknown
//! candidate out of an executable frontier. The refusal is the deliverable, and
//! it is stated **before** any routing commit, because a refusal after a commit
//! would be a fallback ADR 0051 does not permit.
//!
//! **"Does this artifact name the profile the producer declared?"** is what
//! `crate::envelope` answers. It is **producer-declared equality, NOT
//! host-earned eligibility**, and every run that uses it says so in those words.
//!
//! Keeping the two apart is the whole reason this module exists separately from
//! the route: nothing about an artifact, a compilation, or a compiler identity
//! can reach the observation below, so a green route can never be read as an
//! eligibility claim.
//!
//! # Everything here is device-free
//!
//! The policy evaluation, the four ambient predicates, the architecture
//! normalization, and the probed-family vocabulary all compose on any host. The
//! *two* fields a device contributes — the name it reports for itself and the
//! Apple family it claims — arrive as arguments, which is what lets every
//! refusal below be exercised in the gate rather than only on hardware. In
//! particular the registry ID never reaches the policy: ADR 0086 excludes it by
//! name, because the retained records report two different values for one named
//! Apple M4 Max.

use std::process::Command;

use tiler_metal::applicability::{
    AppleGpuFamilyConstant, MetalGpuFamilySupport, MetalHostApplicabilityPolicy,
    MetalHostApplicabilityRefusal, MetalHostObservation, evaluate_metal_host_applicability,
};

/// What a Metal binding could learn about the Apple families a device supports.
///
/// Two outcomes rather than a bare [`MetalGpuFamilySupport`], because "the
/// device named no family this vocabulary knows" and "this binding could not
/// ask" are different facts with different repairs — the first is a host to
/// change and the second is a Metal binding to upgrade — and a measurement
/// boundary that collapsed them would report an unasked question as an answer.
///
/// Declared here rather than beside the probe that fills it, because it names
/// only governed `tiler-metal` types: a host with no Metal binding at all can
/// still state, compare, and refuse one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProbedGpuFamily {
    /// The governed vocabulary's own answer, from a walk a binding completed.
    Answered(MetalGpuFamilySupport),
    /// The vocabulary named an enumerator the binding cannot, so the device was
    /// never asked and there is no answer to report.
    Unnameable(AppleGpuFamilyConstant),
}

impl std::fmt::Display for ProbedGpuFamily {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Answered(MetalGpuFamilySupport::Highest(family)) => {
                formatter.write_str(family.as_str())
            }
            Self::Answered(MetalGpuFamilySupport::NoneNamed) => {
                formatter.write_str("no named Apple family")
            }
            Self::Unnameable(constant) => write!(
                formatter,
                "unobserved: the governed vocabulary names MTLGPUFamily {constant}, which this \
                 binding cannot name, so this device was never asked",
            ),
        }
    }
}

/// Reads one `sw_vers` field, or nothing when the tool does not answer.
///
/// A tool that is missing, fails, or prints nothing leaves the predicate
/// *unobserved* rather than supplying a placeholder. The policy has a typed
/// refusal for an unanswered predicate, and inventing a value here would spend
/// that distinction to make an adapter bug look like a host fact.
pub(crate) fn sw_vers(field: &str) -> Option<String> {
    let output = Command::new("/usr/bin/sw_vers").arg(field).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Normalizes a Rust architecture name into the spelling the records use.
///
/// `std::env::consts::ARCH` reports `aarch64` for the machine every retained
/// record spells `arm64`. Exactly that one spelling is mapped; everything else
/// passes through unchanged, so an architecture nobody measured is refused by
/// its own name rather than renamed into the one the policy wants.
pub(crate) fn normalized_architecture(arch: &str) -> &str {
    if arch == "aarch64" { "arm64" } else { arch }
}

/// Observes the four host predicates that need no device.
///
/// Split from the device half so the composition is exercised without Metal,
/// and so the policy's own cases never need either. Nothing here reads an
/// artifact, a compilation, or a compiler identity: the whole point of the
/// separate applicability check is that it cannot be satisfied by the producer's
/// declaration.
pub(crate) fn observe_host_environment() -> MetalHostObservation {
    let mut observation = MetalHostObservation::unobserved()
        .observing_os_family(std::env::consts::OS)
        .observing_architecture(normalized_architecture(std::env::consts::ARCH));
    if let Some(version) = sw_vers("-productVersion") {
        observation = observation.observing_os_version(version);
    }
    if let Some(build) = sw_vers("-buildVersion") {
        observation = observation.observing_os_build(build);
    }
    observation
}

/// States the probed family on an observation, or deliberately states nothing.
///
/// Leaving the predicate unset is the adapter saying it did not ask, and
/// `MetalHostApplicabilityRefusal::Unobserved { predicate: GpuFamily }` is the
/// typed outcome that already exists for exactly that. Calling
/// `observing_gpu_family` with anything at all would be the adapter claiming it
/// asked.
pub(crate) fn stating_probed_family(
    observation: MetalHostObservation,
    probed: ProbedGpuFamily,
) -> MetalHostObservation {
    match probed {
        ProbedGpuFamily::Answered(support) => observation.observing_gpu_family(support),
        ProbedGpuFamily::Unnameable(_) => observation,
    }
}

/// The production offer path: earn the right to offer the declared profile, or
/// refuse.
///
/// This is the only route in this crate that *claims authority*, and it returns
/// a refusal rather than an environment because on every host observable today
/// the answer is
/// [`MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority`]. The
/// returned value is used for exactly one thing: reporting what was refused and
/// why.
///
/// It does not gate the routes that follow it, and that separation is recorded
/// rather than convenient: the runtime machinery is worth exercising on
/// hardware, and the honest way to keep exercising it is to state that the route
/// runs on producer-declared equality and makes no applicability claim. Gating
/// on this refusal would stop the value proof from running while proving nothing
/// new, because the refusal is structural —
/// [`tiler_metal::applicability::MetalHostEligibility`] holds an uninhabited
/// authority, so no host can produce a receipt.
///
/// # Panics
///
/// Panics if a host ever earns a receipt. Unreachable at the type level inside
/// `tiler-metal`, where the receipt is visibly uninhabited; from here it is an
/// opaque struct, so the arm is required, and writing it costs nothing because
/// reaching it needs a superseding decision under ADR 0086 rather than a code
/// change.
pub(crate) fn refuse_to_offer_the_declared_profile(
    observation: &MetalHostObservation,
) -> MetalHostApplicabilityRefusal {
    let policy = MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9;
    match evaluate_metal_host_applicability(policy, observation) {
        Ok(receipt) => panic!(
            "a host earned an eligibility receipt under {}, which is impossible without a \
             superseding ADR 0086 decision",
            receipt.policy().id(),
        ),
        Err(refusal) => refusal,
    }
}

/// Renders one observation for a run's own output.
pub(crate) fn describe(observation: &MetalHostObservation, probed: ProbedGpuFamily) -> String {
    format!(
        "os {}/{}/{}, arch {}, device {}, family {probed}",
        observation.os_family().unwrap_or("unobserved"),
        observation.os_version().unwrap_or("unobserved"),
        observation.os_build().unwrap_or("unobserved"),
        observation.architecture().unwrap_or("unobserved"),
        observation.device_name().unwrap_or("unobserved"),
    )
}

#[cfg(test)]
mod tests {
    use tiler_metal::applicability::{
        MetalGpuFamily, MetalGpuFamilySupport, MetalHostApplicabilityPolicy,
        MetalHostApplicabilityRefusal, MetalHostObservation, MetalHostPredicate,
        evaluate_metal_host_applicability,
    };

    use super::{
        ProbedGpuFamily, describe, normalized_architecture, observe_host_environment,
        refuse_to_offer_the_declared_profile, stating_probed_family,
    };

    /// Exactly one architecture spelling is rewritten, and nothing else is.
    ///
    /// The mapping exists because `std::env::consts::ARCH` and every retained
    /// record disagree on one name. A map that rewrote anything else would turn
    /// an unmeasured architecture into the measured one and hide the refusal the
    /// policy exists to produce.
    #[test]
    fn the_architecture_normalization_rewrites_one_spelling() {
        assert_eq!(normalized_architecture("aarch64"), "arm64");
        assert_eq!(normalized_architecture("arm64"), "arm64");
        for untouched in ["x86_64", "aarch64_be", "arm64e", "riscv64", ""] {
            assert_eq!(
                normalized_architecture(untouched),
                untouched,
                "only the `aarch64` spelling may be rewritten",
            );
        }
    }

    /// The device-free half answers the two ambient predicates it can and
    /// invents no device ones.
    ///
    /// Asserts which predicates were *answered*, not what they say, because what
    /// they say is the very thing the policy is allowed to disagree with. The
    /// `sw_vers` pair is deliberately not asserted present: the tool does not
    /// exist off macOS, and a case that required it would be a silent
    /// host-dependence in a module whose whole claim is that it has none.
    #[test]
    fn the_device_free_observation_answers_only_the_device_free_predicates() {
        let observation = observe_host_environment();
        assert_eq!(observation.os_family(), Some(std::env::consts::OS));
        assert_eq!(
            observation.architecture(),
            Some(normalized_architecture(std::env::consts::ARCH)),
        );
        assert_eq!(observation.device_name(), None);
        assert_eq!(observation.gpu_family(), None);
        if cfg!(target_os = "macos") {
            assert!(
                observation
                    .os_version()
                    .is_some_and(|value| !value.is_empty()),
                "sw_vers -productVersion answered nothing on a macOS host",
            );
            assert!(
                observation
                    .os_build()
                    .is_some_and(|value| !value.is_empty()),
                "sw_vers -buildVersion answered nothing on a macOS host",
            );
        }
    }

    /// A device-free observation can never reach the translation-authority
    /// predicate, because predicates before it are unanswered.
    #[test]
    fn a_device_free_observation_refuses_before_the_authority() {
        let refusal = refuse_to_offer_the_declared_profile(&observe_host_environment());
        assert_ne!(
            refusal.predicate(),
            MetalHostPredicate::NativeTranslationAuthority,
            "an observation missing the device predicates must refuse on one of them",
        );
    }

    /// Composing both halves leaves no predicate unanswered.
    ///
    /// The device values are stated here rather than read from a device, so this
    /// runs without Metal. What it proves is about the *adapter*: the
    /// device-free half plus the two device fields covers every predicate the
    /// policy evaluates, so whatever refusal a real host gets is about the host
    /// and not about a field nobody filled in.
    #[test]
    fn the_composed_observation_answers_every_predicate() {
        let policy = MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9;
        let complete = MetalHostObservation::unobserved()
            .observing_os_family(policy.os_family())
            .observing_os_version(policy.os_version())
            .observing_os_build(policy.os_build())
            .observing_architecture(policy.architecture())
            .observing_device_name(policy.device_name())
            .observing_gpu_family(MetalGpuFamilySupport::Highest(policy.gpu_family()));
        let refusal = refuse_to_offer_the_declared_profile(&complete);
        assert!(
            !matches!(refusal, MetalHostApplicabilityRefusal::Unobserved { .. }),
            "the adapter left a predicate unanswered: {refusal}",
        );
        assert_eq!(
            refusal.predicate(),
            MetalHostPredicate::NativeTranslationAuthority,
            "a fully observed row must reach ADR 0086's authority refusal and stop there",
        );
    }

    /// An enumerator a binding cannot name leaves the predicate unobserved.
    ///
    /// The pair is the point. The same observation, differing only in what the
    /// probe could learn, reaches ADR 0086's authority refusal when the device
    /// answered and stops at `Unobserved { predicate: GpuFamily }` when it was
    /// never asked — which is the policy's own word for an adapter that did not
    /// ask, and not the `GpuFamilyMismatch` a `false` answer would have
    /// produced.
    #[test]
    fn an_unnameable_enumerator_leaves_the_family_predicate_unobserved() {
        let policy = MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9;
        let measured = MetalHostObservation::unobserved()
            .observing_os_family(policy.os_family())
            .observing_os_version(policy.os_version())
            .observing_os_build(policy.os_build())
            .observing_architecture(policy.architecture())
            .observing_device_name(policy.device_name());

        let answered = stating_probed_family(
            measured.clone(),
            ProbedGpuFamily::Answered(MetalGpuFamilySupport::Highest(policy.gpu_family())),
        );
        assert_eq!(
            evaluate_metal_host_applicability(policy, &answered)
                .expect_err("no host earns a receipt")
                .predicate(),
            MetalHostPredicate::NativeTranslationAuthority,
            "an answered probe must carry the row past the GPU-family predicate",
        );

        let unnameable = stating_probed_family(
            measured,
            ProbedGpuFamily::Unnameable(MetalGpuFamily::Apple9.apple_constant()),
        );
        assert_eq!(
            evaluate_metal_host_applicability(policy, &unnameable)
                .expect_err("no host earns a receipt"),
            MetalHostApplicabilityRefusal::Unobserved {
                predicate: MetalHostPredicate::GpuFamily,
            },
            "a probe that could not ask must refuse as unobserved, not as a mismatch",
        );
    }

    /// Each probed outcome renders as itself, so a report cannot read an unasked
    /// question as an answer.
    #[test]
    fn a_probed_family_renders_the_question_it_could_not_ask() {
        let answered =
            ProbedGpuFamily::Answered(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9));
        assert_eq!(answered.to_string(), MetalGpuFamily::Apple9.as_str());
        assert_eq!(
            ProbedGpuFamily::Answered(MetalGpuFamilySupport::NoneNamed).to_string(),
            "no named Apple family",
        );
        let unnameable =
            ProbedGpuFamily::Unnameable(MetalGpuFamily::Apple9.apple_constant()).to_string();
        assert!(
            unnameable.contains("never asked"),
            "an unasked question must not render as a device that answered: {unnameable}",
        );

        // And the composed report carries whichever of the three it was handed.
        let rendered = describe(&observe_host_environment(), answered);
        assert!(rendered.contains("device unobserved"), "{rendered}");
        assert!(
            rendered.ends_with(MetalGpuFamily::Apple9.as_str()),
            "{rendered}",
        );
    }
}
