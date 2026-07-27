//! Offline-compiler validation of the checked-in golden MSL fixtures.
//!
//! The golden tests in [`crate::tests`] pin emitted bytes and structure. They
//! cannot detect a change that keeps a fixture byte-identical to what emission
//! produces while making it uncompilable, because none of them runs a Metal
//! compiler. These tests close that gap by compiling every golden through
//! `tiler-metal-aot`, the fail-closed offline driver that owns `metal` and
//! `metallib` invocation and provenance capture. Shelling out to `xcrun` from
//! here instead would duplicate that driver and skip its typed diagnostics.
//!
//! The driver is a **development** dependency. `tiler-metal` emits source and
//! must not acquire Apple tool discovery in a consumer's build graph, and
//! keeping the edge out of the normal graph leaves the eventual
//! `tiler-metal-aot` → `tiler-metal` direction available.
//!
//! # Self-skip contract
//!
//! Every compiling test resolves the toolchain first through
//! [`resolved_toolchain`] and returns early when none is present, so the
//! repository gate stays green on a host or CI runner without Xcode. The skip
//! is narrow on purpose: only `ToolchainUnavailable` and `SdkUnavailable`
//! count as an absent toolchain. Every other `DriverError` means the driver
//! reached the tools and something else went wrong, which is a defect these
//! tests report. The classifying match is exhaustive, so a new variant must be
//! classified deliberately rather than defaulting to a skip.
//!
//! Two mechanisms keep a skip from being mistaken for a pass:
//!
//! - Each branch announces itself on standard error. `cargo test -p
//!   tiler-metal --lib -- --nocapture golden_compilation` prints either the
//!   resolved `metal`, `metallib`, and SDK identities or the exact reason the
//!   toolchain did not resolve. The plain gate captures that output, so the
//!   test names carry the `when_a_toolchain_resolves` suffix as the standing
//!   reminder that they are conditional.
//! - Setting `TILER_REQUIRE_METAL_TOOLCHAIN` turns a skip into a failure. This
//!   is the one supported ambient input here, and it can only make the tests
//!   stricter; nothing in this module lets an environment variable weaken a
//!   check.
//!
//! # Which contract governs the goldens
//!
//! Two different records are called a "contract" here and they are decided
//! separately. The **declared realization** is the program's, baked into the
//! emitted bytes; the **compiler realization** is the driver's flag row,
//! selected by [`golden_request`]. The goldens are governed under the strict
//! declared realization — `tiler.test.strict-f32`, preserving subnormals on
//! both dimensions — and under the strict driver baseline.
//!
//! The declared half was reconsidered against the flush-accepting alternative
//! and deliberately left strict. Three reasons, and the third is the decisive
//! one:
//!
//! - Nothing about the compiler evidence would change. Neither subnormal mode
//!   names a compiler selection, so the flag row is identical either way, and
//!   the emitted *bodies* are identical too — no operation is emitted to
//!   realize a flush, because this backend expresses no emulation. Rebaselining
//!   would change every entry-point symbol, since the canonical kernel identity
//!   encodes the profile key and both subnormal dimensions, and buy no coverage.
//! - Under the strict realization these are the only checked-in artifacts that
//!   pin the non-empty unrealizable-obligation provenance block, which is what a
//!   caller keeping only the emitted text reads.
//! - **There is no flush-accepting contract this crate can name.** The governed
//!   one is registered in `tiler-compiler`, which `tiler-metal` does not and
//!   must not depend on, so a "flush golden" would carry a crate-local key that
//!   merely resembles it. Writing the registered key as a string literal here
//!   would duplicate a versioned identity across a boundary with no compile-time
//!   link, and a rename on the owning side would leave a golden silently
//!   claiming the wrong contract. Recording the governed flush contract's bytes
//!   belongs to a component that can name it.
//!
//! One consequence is worth stating rather than leaving to be discovered: the
//! units compiled below are ones
//! [`require_declared_realization`](crate::record::MetalTranslationUnit::require_declared_realization)
//! refuses. That is intentional and is itself evidence — it shows the refusal is
//! a Tiler conformance decision about a contract the target cannot honour, not a
//! compiler rejection of the source. The honoured-flush case is covered by
//! `crate::tests` over the same fixture kernel rather than by a fixture file.
//!
//! # Measurement
//!
//! On an Apple M4 Max under macOS 27.0 (build 26A5388g) with Metal 32023.883
//! and macOS SDK 26.5 (build 25F70), every fixture in `goldens/` compiles and
//! links through this driver under `-target air64-apple-macos13.0
//! -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise
//! -ffp-contract=off`, and the four-entry-point portfolio unit links into one
//! library carrying all four `tiler_kernel_*` symbols.
//!
//! Library size is deliberately **not** asserted. A 14,620-byte link of the
//! four goldens was recorded at commit `59060b5`; after `e24f4c5` changed the
//! emitted source the same command yields 14,716 bytes. A byte count is a
//! property of the exact source and toolchain, so this module asserts
//! structure, provenance, and reproducibility instead. That divergence is also
//! the reason this module exists: a hand-run measurement stopped being true
//! within the hour and nothing noticed.

use std::ffi::OsStr;
use std::path::Path;

use tiler_metal_aot::diagnostic::{CompileStage, DriverError};
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{
    AppleSdk, CompileRequest, DeploymentMinimum, FpContract, MathMode, MetalTarget, MslVersion,
    NumericalRealization, OptimizationLevel,
};

use crate::emit::emit_translation_unit;
use crate::record::MetalNumericalRequirement;
use crate::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalFloatArithmeticType, MetalFlushedZeroSign,
    MetalPlatform, MetalSubnormalArithmetic, MetalSubnormalArithmeticFacts, MetalTargetFacts,
    MslLanguageVersion,
};

/// The ambient input that turns an absent toolchain into a failure.
const REQUIRE_TOOLCHAIN: &str = "TILER_REQUIRE_METAL_TOOLCHAIN";

/// Every checked-in golden fixture, by file name and embedded content.
///
/// `every_checked_in_golden_is_compiled_by_this_module` proves this list covers
/// the whole `goldens/` directory, so a new fixture cannot be added without
/// being compiled.
const GOLDENS: [(&str, &str); 4] = [
    (
        "pointwise_scale_bias.metal",
        include_str!("../goldens/pointwise_scale_bias.metal"),
    ),
    (
        "reduction_single_axis.metal",
        include_str!("../goldens/reduction_single_axis.metal"),
    ),
    (
        "reduction_multi_axis.metal",
        include_str!("../goldens/reduction_multi_axis.metal"),
    ),
    (
        "reduction_fused_multiply_add.metal",
        include_str!("../goldens/reduction_fused_multiply_add.metal"),
    ),
];

/// The driver-side statement of the target every golden was emitted for.
///
/// `tiler-metal` states a target as [`MetalTargetFacts`] and the driver states
/// it as [`MetalTarget`]. These are two vocabularies for one target, so
/// `every_golden_declares_the_target_the_driver_compiles_it_for` checks that
/// they agree on *this* target rather than assuming it. That check is pointwise
/// and is about the fixtures; the two vocabularies are checked to name the same
/// sets in `crate::target_correspondence`.
fn driver_target() -> MetalTarget {
    MetalTarget::new(
        AppleSdk::MacOs,
        DeploymentMinimum::new(13, 0),
        MslVersion::Metal3_1,
    )
}

/// The emitter-side statement of that same target.
///
/// This duplicates [`crate::tests::target`] deliberately: the fixture header
/// assertions compare the goldens against *this* record, so a divergence
/// between the two shows up as a failing assertion instead of being inherited.
fn emitter_facts() -> MetalTargetFacts {
    MetalTargetFacts::new(
        MslLanguageVersion::Metal3_1,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(13, 0),
        LaunchIndexRealization::ThreadPositionInGridUInt,
        MetalSubnormalArithmeticFacts::unmeasured()
            .stating(
                MetalFloatArithmeticType::F32,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            )
            .stating(
                MetalFloatArithmeticType::F16,
                MetalSubnormalArithmetic::PreservesSubnormals,
            )
            .stating(
                MetalFloatArithmeticType::Bf16,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            ),
        31,
    )
}

/// Builds the compilation request the goldens are governed to compile under.
fn golden_request(source: &str) -> CompileRequest {
    CompileRequest::new(
        source,
        driver_target(),
        OptimizationLevel::Default,
        NumericalRealization::strict_baseline(),
    )
}

/// Returns whether `realization` delivers what `requirement` demands.
///
/// The match is exhaustive within the defining crate, so a new requirement
/// variant stops this module compiling until someone names the driver selection
/// that satisfies it. A requirement no selection satisfies must not reach a
/// compilation silently.
fn realization_honours(
    requirement: MetalNumericalRequirement,
    realization: NumericalRealization,
) -> bool {
    match requirement {
        MetalNumericalRequirement::SafeMathMode => realization.math_mode == MathMode::Safe,
        MetalNumericalRequirement::NoFloatingPointContraction => {
            realization.fp_contract == FpContract::Off
        }
    }
}

/// Resolves the offline toolchain, or returns `None` on a host without one.
///
/// See the module documentation for the skip contract this implements.
fn resolved_toolchain() -> Option<Toolchain> {
    let toolchain = Toolchain::system();
    match toolchain.resolve(AppleSdk::MacOs) {
        Ok(resolved) => {
            eprintln!(
                "golden_compilation: compiling with metal {:?} / metallib {:?} (SDK {} build {})",
                resolved.metal.version.lines().next().unwrap_or_default(),
                resolved.metallib.version.lines().next().unwrap_or_default(),
                resolved.sdk.version,
                resolved.sdk.build,
            );
            Some(toolchain)
        }
        Err(
            error @ (DriverError::ToolchainUnavailable { .. } | DriverError::SdkUnavailable { .. }),
        ) => {
            assert!(
                std::env::var_os(REQUIRE_TOOLCHAIN).is_none(),
                "{REQUIRE_TOOLCHAIN} is set, but no qualified Apple Metal toolchain \
                 resolved: {error}"
            );
            eprintln!(
                "golden_compilation: skipped, no qualified Apple Metal toolchain resolved: {error}"
            );
            None
        }
        Err(
            error @ (DriverError::ToolFailure { .. }
            | DriverError::Host { .. }
            | DriverError::EmptyArtifact { .. }),
        ) => panic!(
            "toolchain resolution failed for a reason that is not an absent toolchain: {error}"
        ),
    }
}

/// Returns the entry-point symbols declared by one emitted translation unit.
fn entry_symbols(source: &str) -> Vec<&str> {
    const MARKER: &str = "kernel void ";
    source
        .match_indices(MARKER)
        .map(|(start, _)| {
            let rest = &source[start + MARKER.len()..];
            let end = rest
                .find('(')
                .expect("an emitted entry point declares a parameter list");
            &rest[..end]
        })
        .collect()
}

/// Returns the canonicalization helper's declarator text and its symbol.
fn helper_declaration(source: &str) -> (&str, &str) {
    const PREFIX: &str = "static inline float ";
    let start = source
        .find(PREFIX)
        .expect("every arithmetic golden emits the canonicalization helper");
    let rest = &source[start + PREFIX.len()..];
    let end = rest
        .find('(')
        .expect("the helper declares a parameter list");
    (&source[start..start + PREFIX.len() + end], &rest[..end])
}

/// Returns whether the compiled library carries `symbol` verbatim.
fn library_names(metallib: &[u8], symbol: &str) -> bool {
    metallib
        .windows(symbol.len())
        .any(|window| window == symbol.as_bytes())
}

/// The compiled fixtures must be the complete `goldens/` directory.
///
/// Without this, adding a fifth fixture would silently stay outside compiler
/// validation while still passing a byte-stability test.
#[test]
fn every_checked_in_golden_is_compiled_by_this_module() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("goldens");
    let mut found: Vec<String> = std::fs::read_dir(&directory)
        .expect("the golden fixture directory is checked in")
        .map(|entry| entry.expect("a readable golden directory entry").path())
        .filter(|path| path.extension() == Some(OsStr::new("metal")))
        .map(|path| {
            path.file_name()
                .expect("a golden fixture has a file name")
                .to_str()
                .expect("golden fixture names are UTF-8")
                .to_owned()
        })
        .collect();
    found.sort();
    let mut compiled: Vec<String> = GOLDENS.iter().map(|(name, _)| (*name).to_owned()).collect();
    compiled.sort();
    assert_eq!(
        found,
        compiled,
        "every fixture in {} must be compiled through the offline driver",
        directory.display()
    );
}

/// The fixtures, the emitter's target facts, and the driver's target must agree.
///
/// A golden compiled for a target it was not emitted for would be a green test
/// over the wrong evidence, and the two crates spell the same target in
/// different vocabularies, so the agreement is asserted rather than assumed.
///
/// This is deliberately a *pointwise* check on one governed target. It would
/// stay green if either crate gained a language standard or an artifact family
/// the other lacked, because it never looks at a variant these fixtures do not
/// use. `crate::target_correspondence` owns that totality obligation; do not
/// read this test as covering it.
#[test]
fn every_golden_declares_the_target_the_driver_compiles_it_for() {
    let facts = emitter_facts();
    let target = driver_target();
    assert_eq!(facts.language.std_token(), target.msl_version.std_token());
    assert_eq!(facts.platform.as_str(), target.platform().as_str());
    assert_eq!(
        facts.deployment_minimum.major(),
        target.deployment_minimum.major()
    );
    assert_eq!(
        facts.deployment_minimum.minor(),
        target.deployment_minimum.minor()
    );
    assert_eq!(target.triple(), "air64-apple-macos13.0");

    let language = format!(
        "// Metal Shading Language: {}",
        target.msl_version.std_token()
    );
    let family = format!(
        "// Artifact family: {} (deployment minimum {})",
        target.platform().as_str(),
        target.deployment_minimum,
    );
    for (name, source) in GOLDENS {
        assert!(source.contains(&language), "{name} must declare {language}");
        assert!(source.contains(&family), "{name} must declare {family}");
    }
}

/// The realization the goldens compile under must satisfy what emission demands.
///
/// The requirement set comes from a live emission rather than a hand-written
/// list, so a newly recorded requirement is checked automatically. Both halves
/// matter: the typed selection must deliver the obligation, and the two crates
/// must spell the flag identically, since a silent divergence in either would
/// compile the source under numerics it does not tolerate.
#[test]
fn the_strict_realization_honours_every_requirement_emission_records() {
    let unit = emit_translation_unit(&[&crate::tests::pointwise_kernel()], &emitter_facts())
        .expect("the bounded pointwise fixture emits");
    let realization = NumericalRealization::strict_baseline();
    let flags = golden_request(unit.source()).compile_flags();
    assert!(
        !unit.numerical_requirements().is_empty(),
        "the fixture must record at least one requirement for this check to mean anything"
    );
    for requirement in unit.numerical_requirements() {
        assert!(
            realization_honours(*requirement, realization),
            "the strict baseline does not deliver {requirement}"
        );
        assert!(
            flags.iter().any(|flag| flag == requirement.flag()),
            "the driver does not select {}, which emission requires; flags were {flags:?}",
            requirement.flag()
        );
    }
}

/// Every golden fixture compiles to AIR and links into a Metal library.
///
/// This is the check the byte-stability goldens cannot make. It asserts real
/// output: the linked library begins with the `MTLB` magic, names the fixture's
/// entry-point symbol, and carries provenance showing the exact flags reached
/// the compiler.
#[test]
fn every_golden_compiles_and_links_when_a_toolchain_resolves() {
    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    for (name, source) in GOLDENS {
        let artifact = toolchain
            .compile(&golden_request(source))
            .unwrap_or_else(|error| panic!("golden {name} must compile: {error}"));
        assert_eq!(&artifact.metallib[..4], b"MTLB", "{name}");
        let symbols = entry_symbols(source);
        assert!(!symbols.is_empty(), "{name} must declare an entry point");
        for symbol in symbols {
            assert!(
                library_names(&artifact.metallib, symbol),
                "{name}: the linked library does not name {symbol}"
            );
        }
        assert_eq!(artifact.provenance.target_triple, "air64-apple-macos13.0");
        assert_eq!(
            artifact.provenance.numerical,
            NumericalRealization::strict_baseline(),
            "{name}"
        );
        assert!(!artifact.provenance.metal.version.is_empty(), "{name}");
        assert!(!artifact.provenance.sdk.build.is_empty(), "{name}");
        eprintln!(
            "golden_compilation: {name} linked {} bytes",
            artifact.metallib.len()
        );
    }
}

/// The multi-kernel translation unit links every entry point into one library.
///
/// No golden pins this form, because a portfolio shares one prologue and one
/// helper across entry points and is therefore not any single fixture's bytes.
/// It is the shape a real multi-kernel artifact uses, so it is compiled from a
/// live emission.
#[test]
fn the_portfolio_unit_links_every_entry_point_when_a_toolchain_resolves() {
    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let pointwise = crate::tests::pointwise_kernel();
    let single = crate::tests::single_axis_reduction_kernel();
    let multi = crate::tests::multi_axis_reduction_kernel();
    let fused = crate::tests::fused_reduction_kernel();
    let unit = emit_translation_unit(&[&pointwise, &single, &multi, &fused], &emitter_facts())
        .expect("the bounded portfolio emits");
    assert_eq!(unit.entry_points().len(), 4);

    let artifact = toolchain
        .compile(&golden_request(unit.source()))
        .unwrap_or_else(|error| panic!("the portfolio unit must compile: {error}"));
    assert_eq!(&artifact.metallib[..4], b"MTLB");
    for entry in unit.entry_points() {
        assert!(
            library_names(&artifact.metallib, entry.symbol()),
            "the linked library does not name {}",
            entry.symbol()
        );
    }
    eprintln!(
        "golden_compilation: portfolio linked {} bytes with {} entry points",
        artifact.metallib.len(),
        unit.entry_points().len()
    );
}

/// A golden whose helper definition is renamed must be rejected.
///
/// This proves the compiling tests have teeth. Renaming only the definition
/// leaves every call site referring to an undeclared function, which is exactly
/// the failure mode the byte-stability goldens cannot see: source that still
/// looks well formed and no longer compiles. The rejection must also be typed,
/// not a panic or a partial artifact.
#[test]
fn a_golden_whose_helper_is_renamed_is_rejected_when_a_toolchain_resolves() {
    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let (name, source) = GOLDENS[0];
    let (declarator, symbol) = helper_declaration(source);
    let uses = source.matches(symbol).count();
    assert!(uses > 1, "{name}: the helper must be defined and called");
    let broken = source.replacen(declarator, "static inline float tiler_absent_helper", 1);
    assert_eq!(
        broken.matches(symbol).count(),
        uses - 1,
        "only the definition may be renamed, so the calls stay undeclared"
    );

    let error = toolchain
        .compile(&golden_request(&broken))
        .expect_err("an undeclared function call must not compile");
    match error {
        DriverError::ToolFailure { stage, stderr, .. } => {
            assert_eq!(stage, CompileStage::Metal);
            assert!(
                !stderr.is_empty(),
                "the compiler must explain the rejection"
            );
        }
        other => panic!("expected a metal-stage ToolFailure, got {other:?}"),
    }
}

/// One golden compiles to identical library bytes twice.
///
/// The driver compiles through a scratch directory whose name differs on every
/// call, so this shows that per-run host state does not leak into the artifact.
/// That is a property the source goldens cannot express, and the eventual
/// artifact cache depends on it.
///
/// **Measurement.** Verified on Metal 32023.883 under macOS 27.0 (build
/// 26A5388g). It is a host-qualified observation, not a portable guarantee
/// about every toolchain build.
#[test]
fn one_golden_compiles_to_identical_bytes_twice_when_a_toolchain_resolves() {
    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let (name, source) = GOLDENS[0];
    let first = toolchain
        .compile(&golden_request(source))
        .unwrap_or_else(|error| panic!("golden {name} must compile: {error}"));
    let second = toolchain
        .compile(&golden_request(source))
        .unwrap_or_else(|error| panic!("golden {name} must compile: {error}"));
    assert_eq!(
        first.metallib, second.metallib,
        "{name}: the compiled library must not depend on the scratch path or the clock"
    );
    assert_eq!(first.provenance.fingerprint, second.provenance.fingerprint);
}
