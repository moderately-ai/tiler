//! Host-applicability policy cases, every one of them device-free.
//!
//! The policy is separated from observation precisely so these run in the
//! ordinary gate: no `MTLDevice`, no Apple family query, and no host with the
//! measured OS build is needed to prove what the policy decides. This repository
//! runs no CI, so that portability is a property of the tests rather than a
//! guarantee something re-checks on every push.
//!
//! What the cases below establish, in the order the ticket requires it:
//!
//! 1. Each wrong predicate refuses under its own typed reason.
//! 2. Each unobserved predicate refuses as a missing observation, named.
//! 3. The exact measured row still refuses, naming the ADR 0086 predicate.
//! 4. No combination over a spanning input domain reaches a positive receipt —
//!    which the uninhabited `NativeTranslationAuthority` already makes
//!    structural, and which this enumerates anyway so a future change that
//!    inhabited the type without reopening ADR 0086 would fail here too.

use std::collections::BTreeSet;
use std::path::Path;

use crate::applicability::{
    MetalGpuFamily, MetalGpuFamilySupport, MetalHostApplicabilityPolicy,
    MetalHostApplicabilityRefusal, MetalHostObservation, MetalHostPredicate,
    evaluate_metal_host_applicability,
};

/// The exact row `FIRST_MACOS_APPLE9` was transcribed from.
fn measured_row() -> MetalHostObservation {
    MetalHostObservation::unobserved()
        .observing_os_family("macos")
        .observing_os_version("27.0")
        .observing_os_build("26A5388g")
        .observing_architecture("arm64")
        .observing_device_name("Apple M4 Max")
        .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9))
}

fn refuse(observation: &MetalHostObservation) -> MetalHostApplicabilityRefusal {
    evaluate_metal_host_applicability(
        MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9,
        observation,
    )
    .expect_err("no host earns a receipt while ADR 0086's authority is unknown")
}

/// The policy's required values are the ones the retained records carry.
///
/// Pinned here rather than trusted to the constant, because the constant is a
/// transcription: a silent edit to any of these six fields would widen or narrow
/// the measurement's validity scope with nothing else noticing.
#[test]
fn the_policy_states_the_exact_retained_row() {
    let policy = MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9;
    assert_eq!(policy.os_family(), "macos");
    assert_eq!(policy.os_version(), "27.0");
    assert_eq!(policy.os_build(), "26A5388g");
    assert_eq!(policy.architecture(), "arm64");
    assert_eq!(policy.device_name(), "Apple M4 Max");
    assert_eq!(policy.gpu_family(), MetalGpuFamily::Apple9);
    assert_eq!(
        policy.id(),
        "tiler.metal.host-applicability.macos-27.0-26A5388g-arm64-m4max-apple9.v1",
    );
}

/// The exact measured row refuses, and refuses for the ADR 0086 reason alone.
///
/// This is the case the decision is about: every public environment predicate
/// agrees with the retained observation, so anything short of naming the
/// translation authority would be the "public-row equivalence" alternative ADR
/// 0086 rejected.
#[test]
fn the_fully_matching_row_refuses_on_the_translation_authority() {
    let refusal = refuse(&measured_row());
    assert_eq!(
        refusal,
        MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority {
            policy: MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9.id(),
        },
    );
    assert_eq!(
        refusal.predicate(),
        MetalHostPredicate::NativeTranslationAuthority,
    );
    assert_eq!(
        refusal.rule(),
        "metal.host-applicability.unknown-translation-authority",
    );
    let rendered = refusal.to_string();
    assert!(
        rendered.contains("native-translation-authority"),
        "the refusal must name the unsatisfied predicate: {rendered}",
    );
    assert!(
        rendered.contains("ADR 0086"),
        "the refusal must cite the deciding record: {rendered}",
    );
}

/// Each wrong predicate refuses under its own variant, and not another's.
///
/// The population is named and counted: one case per environment predicate, and
/// the six predicates are exactly `MetalHostPredicate::ALL` minus the authority.
/// A uniform pass over six cases that all reported the same variant would be the
/// failure this asserts against, so each case pins the variant *and* the
/// predicate.
#[test]
fn each_wrong_predicate_has_its_own_typed_reason() {
    let row = measured_row();
    let cases: [(MetalHostApplicabilityRefusal, MetalHostObservation); 6] = [
        (
            MetalHostApplicabilityRefusal::OsFamilyMismatch {
                required: "macos",
                observed: "linux".to_owned(),
            },
            row.clone().observing_os_family("linux"),
        ),
        (
            MetalHostApplicabilityRefusal::OsVersionMismatch {
                required: "27.0",
                observed: "26.0".to_owned(),
            },
            row.clone().observing_os_version("26.0"),
        ),
        (
            MetalHostApplicabilityRefusal::OsBuildMismatch {
                required: "26A5388g",
                observed: "26A5388f".to_owned(),
            },
            row.clone().observing_os_build("26A5388f"),
        ),
        (
            MetalHostApplicabilityRefusal::ArchitectureMismatch {
                required: "arm64",
                observed: "x86_64".to_owned(),
            },
            row.clone().observing_architecture("x86_64"),
        ),
        (
            MetalHostApplicabilityRefusal::DeviceNameMismatch {
                required: "Apple M4 Max",
                observed: "Apple M4 Pro".to_owned(),
            },
            row.clone().observing_device_name("Apple M4 Pro"),
        ),
        (
            MetalHostApplicabilityRefusal::GpuFamilyMismatch {
                required: MetalGpuFamily::Apple9,
                observed: MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple8),
            },
            row.clone()
                .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple8)),
        ),
    ];

    let mut predicates = BTreeSet::new();
    for (expected, observation) in cases {
        let refusal = refuse(&observation);
        assert_eq!(refusal, expected);
        assert_eq!(
            refusal.rule(),
            "metal.host-applicability.outside-measured-row"
        );
        assert!(
            predicates.insert(refusal.predicate()),
            "two environment predicates reported the same reason: {refusal}",
        );
    }
    assert_eq!(
        predicates.len(),
        MetalHostPredicate::COUNT - 1,
        "every predicate except the translation authority must have a wrong-value case",
    );
    assert!(!predicates.contains(&MetalHostPredicate::NativeTranslationAuthority));
}

/// A device naming no Apple family is refused as a family mismatch, not as a
/// missing observation.
///
/// The two are different repairs: the first is hardware outside the measured
/// row, the second is an adapter that did not ask.
#[test]
fn a_device_naming_no_apple_family_is_a_family_mismatch() {
    let refusal = refuse(&measured_row().observing_gpu_family(MetalGpuFamilySupport::NoneNamed));
    assert_eq!(
        refusal,
        MetalHostApplicabilityRefusal::GpuFamilyMismatch {
            required: MetalGpuFamily::Apple9,
            observed: MetalGpuFamilySupport::NoneNamed,
        },
    );
    assert!(
        refusal.to_string().contains("no named Apple family"),
        "the refusal must say what the device reported: {refusal}",
    );
}

/// Every predicate that is simply not observed refuses as such, by name.
///
/// Built by *removing* one field from the full row rather than by adding one, so
/// each case differs from the accepted-shaped observation in exactly the field
/// it is about.
#[test]
fn each_unobserved_predicate_refuses_naming_itself() {
    let full = measured_row();
    let without: [(MetalHostPredicate, MetalHostObservation); 6] = [
        (
            MetalHostPredicate::OsFamily,
            MetalHostObservation::unobserved()
                .observing_os_version("27.0")
                .observing_os_build("26A5388g")
                .observing_architecture("arm64")
                .observing_device_name("Apple M4 Max")
                .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9)),
        ),
        (
            MetalHostPredicate::OsVersion,
            MetalHostObservation::unobserved()
                .observing_os_family("macos")
                .observing_os_build("26A5388g")
                .observing_architecture("arm64")
                .observing_device_name("Apple M4 Max")
                .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9)),
        ),
        (
            MetalHostPredicate::OsBuild,
            MetalHostObservation::unobserved()
                .observing_os_family("macos")
                .observing_os_version("27.0")
                .observing_architecture("arm64")
                .observing_device_name("Apple M4 Max")
                .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9)),
        ),
        (
            MetalHostPredicate::Architecture,
            MetalHostObservation::unobserved()
                .observing_os_family("macos")
                .observing_os_version("27.0")
                .observing_os_build("26A5388g")
                .observing_device_name("Apple M4 Max")
                .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9)),
        ),
        (
            MetalHostPredicate::DeviceName,
            MetalHostObservation::unobserved()
                .observing_os_family("macos")
                .observing_os_version("27.0")
                .observing_os_build("26A5388g")
                .observing_architecture("arm64")
                .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9)),
        ),
        (
            MetalHostPredicate::GpuFamily,
            MetalHostObservation::unobserved()
                .observing_os_family("macos")
                .observing_os_version("27.0")
                .observing_os_build("26A5388g")
                .observing_architecture("arm64")
                .observing_device_name("Apple M4 Max"),
        ),
    ];

    assert_eq!(without.len(), MetalHostPredicate::COUNT - 1);
    for (predicate, observation) in without {
        assert_ne!(
            observation, full,
            "each case must drop exactly one field of the measured row",
        );
        let refusal = refuse(&observation);
        assert_eq!(
            refusal,
            MetalHostApplicabilityRefusal::Unobserved { predicate },
        );
        assert_eq!(
            refusal.rule(),
            "metal.host-applicability.unobserved-predicate",
        );
    }

    // Nothing observed at all reports the first predicate rather than a summary.
    assert_eq!(
        refuse(&MetalHostObservation::unobserved()),
        MetalHostApplicabilityRefusal::Unobserved {
            predicate: MetalHostPredicate::OsFamily,
        },
    );
}

/// A missing observation is reported before a wrong value on a later predicate.
///
/// Order matters for explain output: an adapter that answered nothing about the
/// device should hear that, not that its OS build is fine.
#[test]
fn refusal_order_follows_the_declared_predicate_order() {
    let wrong_and_missing = MetalHostObservation::unobserved()
        .observing_os_family("macos")
        .observing_os_version("27.0")
        .observing_architecture("x86_64")
        .observing_device_name("Apple M4 Max")
        .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9));
    assert_eq!(
        refuse(&wrong_and_missing),
        MetalHostApplicabilityRefusal::Unobserved {
            predicate: MetalHostPredicate::OsBuild,
        },
        "the OS build precedes the architecture in MetalHostPredicate::ALL",
    );
}

/// No combination over a spanning input domain reaches a positive receipt.
///
/// The structural claim is the stronger one — `NativeTranslationAuthority` is
/// uninhabited, so an `Ok` arm has no value to carry — and this enumerates the
/// admissible inputs anyway. The domain spans every predicate in both
/// directions: matching and non-matching values, both family answers, and each
/// field's unobserved state.
///
/// The population is pinned to a **literal** 972, and each domain carries an
/// explicit array length so dropping a value is a compile error rather than a
/// quieter matrix. Deriving the expected count from `families.len() * …` was the
/// first version, and it could not fail: shrinking a domain shrank the
/// expectation with it, so a matrix cut from 972 cases to 648 still reported
/// full coverage. A count computed from the thing it is counting is not a count.
#[test]
fn no_admissible_observation_reaches_a_positive_receipt() {
    let families: [Option<&str>; 3] = [Some("macos"), Some("linux"), None];
    let versions: [Option<&str>; 3] = [Some("27.0"), Some("26.0"), None];
    let builds: [Option<&str>; 3] = [Some("26A5388g"), Some("26A5388f"), None];
    let architectures: [Option<&str>; 3] = [Some("arm64"), Some("x86_64"), None];
    let devices: [Option<&str>; 3] = [Some("Apple M4 Max"), Some("Apple iOS simulator GPU"), None];
    let supports: [Option<MetalGpuFamilySupport>; 4] = [
        Some(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9)),
        Some(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple5)),
        Some(MetalGpuFamilySupport::NoneNamed),
        None,
    ];

    let mut evaluated = 0_usize;
    let mut matched_row = 0_usize;
    for family in families {
        for version in versions {
            for build in builds {
                for architecture in architectures {
                    for device in devices {
                        for support in supports {
                            let mut observation = MetalHostObservation::unobserved();
                            if let Some(value) = family {
                                observation = observation.observing_os_family(value);
                            }
                            if let Some(value) = version {
                                observation = observation.observing_os_version(value);
                            }
                            if let Some(value) = build {
                                observation = observation.observing_os_build(value);
                            }
                            if let Some(value) = architecture {
                                observation = observation.observing_architecture(value);
                            }
                            if let Some(value) = device {
                                observation = observation.observing_device_name(value);
                            }
                            if let Some(value) = support {
                                observation = observation.observing_gpu_family(value);
                            }
                            let refusal = refuse(&observation);
                            evaluated += 1;
                            if refusal.predicate() == MetalHostPredicate::NativeTranslationAuthority
                            {
                                matched_row += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    assert_eq!(
        evaluated, 972,
        "the domain is 3 x 3 x 3 x 3 x 3 x 4 combinations; a smaller number means a domain \
         shrank and this stopped covering what it claims to",
    );
    assert_eq!(
        matched_row, 1,
        "exactly one combination is the measured row, and it too is refused",
    );
}

/// The policy is a pure function of its two inputs.
///
/// Re-evaluating the same observation returns the identical refusal, and no call
/// consulted anything that could move between them.
#[test]
fn evaluation_is_deterministic() {
    let row = measured_row();
    assert_eq!(refuse(&row), refuse(&row));
    let wrong = row.clone().observing_device_name("Apple M1");
    assert_eq!(refuse(&wrong), refuse(&wrong));
    assert_ne!(refuse(&row), refuse(&wrong));
}

/// Producer-owned types stay unnameable in this crate's public signatures.
///
/// This is the signature-level half of "no `Compilation`, no offline compiler
/// provenance, and no source-JIT compiler identity is an input": those types
/// live in `tiler-compiler` and `tiler-metal-aot`, and this crate cannot mention
/// either from its library target. `tiler-metal-aot` is a *development*
/// dependency, which reaches the golden-compilation tests and not the policy.
///
/// The other half is pinned as compile-fail doctests on
/// [`evaluate_metal_host_applicability`], which prove a `TargetProfileRef` and
/// artifact bytes are not admissible arguments — those two *are* nameable here,
/// because `tiler-artifact` is a dependency, so absence of a dependency edge
/// would not have caught them.
#[test]
fn the_dependency_set_keeps_producer_types_unnameable() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("this crate's own manifest is readable");
    let mut section = "";
    let mut normal = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = match line {
                "[dependencies]" => "normal",
                _ => "",
            };
            continue;
        }
        if section == "normal" && !line.is_empty() && !line.starts_with('#') {
            let name = line
                .split(['.', ' ', '='])
                .next()
                .expect("a dependency line names a package");
            normal.push(name.to_owned());
        }
    }
    assert_eq!(
        normal,
        ["tiler-artifact", "tiler-ir"],
        "adding a dependency here would make producer-owned types nameable in this policy's \
         signature; the host-applicability decision may not see one",
    );
}
