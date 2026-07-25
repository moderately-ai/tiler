//! The production translation from emitted target facts to a compile target.
//!
//! `tiler-metal` names the target an emission *declares it was written for*.
//! `tiler-metal-aot` names the target a compilation is *invoked for*. They are
//! separate vocabularies on purpose, and `tiler_metal::target_correspondence`
//! is the test that keeps the two in step — but a test cannot convert. Neither
//! crate may depend on the other, so no conversion can exist inside either, and
//! this producer is the first component that sees both at once. That is the
//! obligation the correspondence module says its orchestrator inherits.
//!
//! # Why every map here is total
//!
//! A wildcard arm in any of these matches could only invent the counterpart it
//! failed to recognize — an `AppleSdk` for an unknown family, a `-std` token for
//! an unknown standard. The bundle would then carry a provenance header
//! describing one compilation and bytes produced by another, and nothing
//! downstream could detect the disagreement, because both halves would be
//! internally well formed. So the matches are exhaustive over enums both crates
//! deliberately keep exhaustive (ADR 0074 convention 5b), and a family or a
//! standard added to either vocabulary is a build failure here.
//!
//! # What this deliberately does not do
//!
//! It does not choose an SDK for a family that has more than one, because no
//! governed family does. It does not widen a deployment minimum to satisfy a
//! standard, and it does not silently substitute a nearby target: the request
//! is what the emission declared, or it is a rejection.

#![allow(
    dead_code,
    reason = "the target translation is landed ahead of its production caller (ADR 0074 convention 7). It reserves the emitter-to-driver correspondence that `tiler_metal::target_correspondence` says its orchestrator inherits, and its first non-test caller is the assembly step that drives an emission into a compilation. That step cannot exist yet: `tiler_compiler::pipeline::compile` and `CompilationRequest` are `pub(crate)` behind a private module, so no out-of-crate caller can obtain a verified kernel to emit. `prototype-public-compiler-api` owns that boundary."
)]

use tiler_metal::target::{
    MetalDeploymentMinimum, MetalPlatform, MetalTargetFacts, MslLanguageVersion,
};
use tiler_metal_aot::input::{AppleSdk, DeploymentMinimum, MetalTarget, MslVersion};

/// Derives the compile target a translation unit's declared facts require.
///
/// Total by construction: every field of [`MetalTargetFacts`] that the driver
/// also names is mapped by an exhaustive match, and the fields the driver does
/// not name — the launch-index realization, the subnormal fact, and the buffer
/// binding limit — are emission concerns that do not reach a compile
/// invocation.
#[must_use]
pub fn compile_target(facts: MetalTargetFacts) -> MetalTarget {
    MetalTarget::new(
        sdk_for(facts.platform),
        deployment_minimum(facts.deployment_minimum),
        msl_version(facts.language),
    )
}

/// Selects the SDK that produces one declared artifact family.
///
/// The driver's `AppleSdk` is a tool-discovery vocabulary with no counterpart in
/// the emitter, so this is the one place the two are related. It is a function
/// rather than a lookup because the relation is one SDK per governed family; a
/// family with two SDKs would need a stated selection rule instead of this map.
const fn sdk_for(family: MetalPlatform) -> AppleSdk {
    match family {
        MetalPlatform::MacOs => AppleSdk::MacOs,
        MetalPlatform::IOsDevice => AppleSdk::IPhoneOs,
        MetalPlatform::IOsSimulator => AppleSdk::IPhoneSimulator,
    }
}

/// Restates one declared MSL standard as the standard to compile with.
const fn msl_version(language: MslLanguageVersion) -> MslVersion {
    match language {
        MslLanguageVersion::Metal3_0 => MslVersion::Metal3_0,
        MslLanguageVersion::Metal3_1 => MslVersion::Metal3_1,
    }
}

/// Restates one declared deployment minimum as the driver's own record.
///
/// Both are `{major, minor}` value records, so this carries the components
/// across rather than reinterpreting them; the two types stay distinct because
/// one is an emission declaration and the other a compile input.
const fn deployment_minimum(minimum: MetalDeploymentMinimum) -> DeploymentMinimum {
    DeploymentMinimum::new(minimum.major(), minimum.minor())
}

#[cfg(test)]
mod tests {
    use super::{compile_target, msl_version, sdk_for};
    use tiler_metal::target::{
        LaunchIndexRealization, MetalDeploymentMinimum, MetalFlushedZeroSign, MetalPlatform,
        MetalSubnormalArithmetic, MetalTargetFacts, MslLanguageVersion,
    };
    use tiler_metal_aot::input::{ApplePlatform, AppleSdk, MslVersion};

    fn facts(platform: MetalPlatform, language: MslLanguageVersion) -> MetalTargetFacts {
        MetalTargetFacts::new(
            language,
            platform,
            MetalDeploymentMinimum::new(13, 0),
            LaunchIndexRealization::ThreadPositionInGridUInt,
            MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            },
            31,
        )
    }

    /// The family a compilation produces is the family the emission declared.
    ///
    /// This is the property the translation exists for, and it is asserted
    /// through `AppleSdk::platform()` rather than against a second hand-written
    /// table, so the test cannot agree with a wrong map by repeating it.
    #[test]
    fn every_declared_family_compiles_for_that_same_family() {
        for (declared, expected) in [
            (MetalPlatform::MacOs, ApplePlatform::MacOs),
            (MetalPlatform::IOsDevice, ApplePlatform::IOsDevice),
            (MetalPlatform::IOsSimulator, ApplePlatform::IOsSimulator),
        ] {
            let target = compile_target(facts(declared, MslLanguageVersion::Metal3_1));
            assert_eq!(target.platform(), expected);
            assert_eq!(sdk_for(declared).platform(), expected);
        }
    }

    /// The compiled standard is the standard the source declares it was written
    /// against, compared by `-std` token rather than by variant name.
    #[test]
    fn every_declared_standard_compiles_under_its_own_std_token() {
        for declared in [MslLanguageVersion::Metal3_0, MslLanguageVersion::Metal3_1] {
            let target = compile_target(facts(MetalPlatform::MacOs, declared));
            assert_eq!(
                target.msl_version.std_token(),
                declared.std_token(),
                "the compiled standard must be the declared one",
            );
        }
    }

    /// The deployment minimum reaches the triple the driver will compile with.
    #[test]
    fn the_declared_deployment_minimum_reaches_the_target_triple() {
        let mut declared = facts(MetalPlatform::MacOs, MslLanguageVersion::Metal3_1);
        declared.deployment_minimum = MetalDeploymentMinimum::new(14, 2);
        assert_eq!(compile_target(declared).triple(), "air64-apple-macos14.2");

        let mut simulator = facts(MetalPlatform::IOsSimulator, MslLanguageVersion::Metal3_1);
        simulator.deployment_minimum = MetalDeploymentMinimum::new(16, 0);
        assert_eq!(
            compile_target(simulator).triple(),
            "air64-apple-ios16.0-simulator",
        );
    }

    /// Every governed SDK is reachable from some declared family.
    ///
    /// Without this, a family could be mapped onto the wrong SDK and every
    /// pointwise case above would still pass as long as it was mapped
    /// consistently. Requiring the map to be onto the full SDK set is what
    /// catches a collapsed arm.
    #[test]
    fn the_family_map_reaches_every_governed_sdk() {
        let reached: Vec<AppleSdk> = [
            MetalPlatform::MacOs,
            MetalPlatform::IOsDevice,
            MetalPlatform::IOsSimulator,
        ]
        .into_iter()
        .map(sdk_for)
        .collect();
        for sdk in [
            AppleSdk::MacOs,
            AppleSdk::IPhoneOs,
            AppleSdk::IPhoneSimulator,
        ] {
            assert!(reached.contains(&sdk), "{} is unreachable", sdk.selector());
        }
    }

    /// The standard map is injective, so two standards cannot collapse to one.
    #[test]
    fn the_standard_map_is_injective() {
        assert_ne!(
            msl_version(MslLanguageVersion::Metal3_0),
            msl_version(MslLanguageVersion::Metal3_1),
        );
        assert_eq!(
            msl_version(MslLanguageVersion::Metal3_0),
            MslVersion::Metal3_0
        );
    }
}
