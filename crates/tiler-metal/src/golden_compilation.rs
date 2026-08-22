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
//! links through this driver under `-target air64-apple-macos14.0
//! -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise
//! -ffp-contract=off`, and the four-entry-point portfolio unit links into one
//! library carrying all four `tiler_kernel_*` symbols.
//!
//! **The BF16 fixture is the newest, and it moved a different boundary.**
//! `pointwise_scale_bias_bf16.metal` is the first golden emitted at a width
//! other than `f32`, so this run is the measurement that the Apple offline
//! toolchain accepts the `bfloat` spelling, the `ushort` constant carrier, and
//! the `bfloat` NaN-canonicalization helper this backend now emits.
//!
//! **Measurement — recorded on a later toolchain row than the paragraph above,
//! and deliberately not merged into it.** On an Apple M4 Max under macOS 27.0
//! (build 26A5388g) with Xcode 27.0 (build 27A5228h), Metal 32023.921
//! (`metalfe-32023.921`, AIR-LLD 32023.921), and macOS SDK 27.0 (build
//! 26A5388f), all seven fixtures compile and link under the same flag row;
//! `pointwise_scale_bias_bf16.metal` links 3,635 bytes and the linked library
//! names `tiler_kernel_7c905e3938dc8d91`. Stripping the `ushort` narrowing from
//! its constant is rejected at the `metal` stage with `as_type cast from
//! 'unsigned int' to 'bfloat' is not allowed`, which is what makes the carrier
//! a measured requirement rather than a stylistic choice.
//!
//! **What this compile evidence is not.** It says the source translates and
//! links for `air64-apple-macos14.0`. It says nothing about whether a device
//! *runs* it, which for `bfloat` is a per-family fact the retained Apple record
//! owns and the two families disagree on: finding 26 records the iOS Simulator
//! compiling and linking every `bfloat` module and then failing pipeline
//! creation. It also says nothing about the BF16 numerical row, which was
//! measured under `-std=metal4.0` for `air64-apple-macos26.0` and is carried by
//! the target profile, not by this compilation.
//!
//! **The cooperative fixture moved the boundary before it.**
//! `cooperative_workgroup_reduction.metal` is the first golden whose entry point
//! declares threadgroup storage, reads `[[thread_index_in_threadgroup]]`, stages
//! values, and carries a `threadgroup_barrier` — so this run is the measurement
//! that the Apple toolchain accepts the staged, fenced body this backend now
//! emits, rather than only the flat per-invocation bodies it emitted before.
//! Recorded on the same host and toolchain as the row above.
//!
//! **Measurement — the two structural fixtures, on the toolchain row above.**
//! On an Apple M4 Max under macOS 27.0 (build 26A5388g) with Xcode 27.0 (build
//! 27A5228h), Metal 32023.921 (`metalfe-32023.921`, AIR-LLD 32023.921), and
//! macOS SDK 27.0 (build 26A5388f), all nine fixtures compile and link under
//! `-target air64-apple-macos14.0 -std=metal3.1 -O2 -fmetal-math-mode=safe
//! -fmetal-math-fp32-functions=precise -ffp-contract=off`.
//! `structural_mirrored_reindex.metal` links 3,555 bytes and names
//! `tiler_kernel_c757efe9a62e2e41`; `structural_widening_broadcast.metal` links
//! 3,555 bytes and names `tiler_kernel_27edf18e107d5dde`. This is the run that
//! turns "the emitter writes an unsigned mirrored difference" into "the Apple
//! offline toolchain accepts the `uint64_t` divide/wrap/subtract/scale/add chain
//! a reindex mirror emits" — no earlier fixture emits a subtraction in the index
//! role at all.
//!
//! **What that measurement is not, stated because this construct invites the
//! confusion.** It says the mirrored offset chain translates and links. It says
//! nothing about the *values* the chain produces on a device, and in particular
//! nothing about the wrap: `c - (extent - 1)` compiles and links just as
//! cleanly, and computes an index near `2^64`. The defence against that form is
//! the structured kernel verifier's body-refinement check, not this compilation
//! — `crate::tests::the_verifier_refuses_a_reordered_mirror_before_emission_sees_it`
//! is where it is pinned.
//!
//! **Measurement — the elementary fixture, and it is the first golden whose
//! acceptance depends on a name.** On an Apple M4 Max under macOS 27.0 (build
//! 26A5388g) with Xcode 27.0 (build 27A5228h), Metal 32023.921
//! (`metalfe-32023.921`, AIR-LLD 32023.921), and macOS SDK 27.0 (build
//! 26A5388f), all ten fixtures compile and link under the flag row above.
//! `elementary_silu_activation.metal` links 3,779 bytes and the linked library
//! names `tiler_kernel_b1e08c4feb69be47`. Until it was checked in, every claim
//! this crate made about `precise::exp` and the division operator was a string
//! match over emitted text against a compiler that had never seen either.
//! Replacing its call with `precise::exp()` is rejected at the `metal` stage
//! with `no matching function for call to 'exp'`, the diagnostic naming the
//! `metal_math` overloads it resolved against, which is what makes the
//! acceptance a binding to a declared function rather than a parse.
//!
//! **Measurement — which exponential the linked library actually references,
//! on the same row.** The AIR intrinsic is named in the artifact, so the
//! question "did the emitted spelling get the contracted function?" can be
//! asked of the library rather than of the source. Across the two spellings and
//! two flag rows:
//!
//! | | `-fmetal-math-fp32-functions=precise` | `=fast`, `-fmetal-math-mode=fast` |
//! |---|---|---|
//! | `precise::exp(v7)` | `air.exp.f32`, 3,779 bytes | `air.exp.f32`, 3,955 bytes |
//! | `exp(v7)` | `air.exp.f32`, 3,779 bytes | `air.fast_exp.f32`, 3,971 bytes |
//!
//! Both governed-row libraries are byte-identical, so no compilation under the
//! flags Tiler selects can distinguish the two spellings — that is the bound on
//! every other test here. The fast column is where they diverge, and it is the
//! measurement that the emitted namespace, not the flag, is what holds the
//! precise selection when the flag is absent.
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
    ApplePlatform, AppleSdk, CompileRequest, DeploymentMinimum, Fp32Functions, FpContract,
    MathMode, MetalTarget, MslVersion, NumericalRealization, OptimizationLevel,
};

use crate::emit::emit_translation_unit;
use crate::record::MetalNumericalRequirement;
use crate::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalEmissionRealization,
    MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
    MetalSubnormalArithmeticFacts, MetalTargetFacts, MslLanguageVersion,
};

/// The ambient input that turns an absent toolchain into a failure.
const REQUIRE_TOOLCHAIN: &str = "TILER_REQUIRE_METAL_TOOLCHAIN";

/// Every checked-in golden fixture, by file name and embedded content.
///
/// `every_checked_in_golden_is_compiled_by_this_module` proves this list covers
/// the whole `goldens/` directory, so a new fixture cannot be added without
/// being compiled.
const GOLDENS: [(&str, &str); 13] = [
    (
        "pointwise_scale_bias.metal",
        include_str!("../goldens/pointwise_scale_bias.metal"),
    ),
    // The one fixture emitted at a width other than `f32`. Compiling it is what
    // turns "the emitter writes `bfloat`" into "the Metal compiler accepts the
    // `bfloat` spelling, the `ushort` constant carrier, and the `bfloat` NaN
    // helper this backend emits", none of which any `f32` golden can say — and
    // the carrier is the half most likely to be wrong in a way that still looks
    // right, since `as_type` requires equal sizes and the `f32` spelling applied
    // at this width does not compile at all.
    (
        "pointwise_scale_bias_bf16.metal",
        include_str!("../goldens/pointwise_scale_bias_bf16.metal"),
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
    (
        "contraction_strict_tensor.metal",
        include_str!("../goldens/contraction_strict_tensor.metal"),
    ),
    // The one fixture that composes threadgroup storage, a barrier, a round
    // loop, and a guarded load in one entry point. Compiling it is what turns
    // "the emitter writes a conditional operand read" into "the Metal compiler
    // accepts two threadgroup allocations, a barrier inside a loop body, and a
    // subscript reached only through a conditional operator" — and the last is
    // the half most likely to be wrong in a way that still looks right, because
    // a spelling that evaluated both arms would compile just as cleanly and read
    // past the end of the operand on exactly this fixture's partial block.
    // The two rank-four fixtures, and the pair is deliberate. Every other golden
    // here contracts a rank-two output; these are the first whose iteration
    // space carries four axes, so compiling them is what turns "the emitter
    // writes a four-axis address chain" into "the Metal compiler accepts the
    // nested division-and-remainder chain this backend emits to recover four
    // coordinates from one linear invocation index".
    //
    // They also differ from each other in exactly one way — whether operand 1
    // reads its contracted axis last or in the middle — which is the sharpest
    // identity hazard in the pinned workload and the one a positional rather
    // than role-keyed lowering would collapse.
    (
        "contraction_attention_score.metal",
        include_str!("../goldens/contraction_attention_score.metal"),
    ),
    (
        "contraction_attention_value.metal",
        include_str!("../goldens/contraction_attention_value.metal"),
    ),
    (
        "contraction_tiled_cooperative.metal",
        include_str!("../goldens/contraction_tiled_cooperative.metal"),
    ),
    // The one fixture whose entry point declares threadgroup storage, reads a
    // local invocation coordinate, and carries a barrier. Compiling it is what
    // turns "the emitter produces this text" into "the Metal compiler accepts a
    // cooperative kernel", which no other golden can say.
    (
        "cooperative_workgroup_reduction.metal",
        include_str!("../goldens/cooperative_workgroup_reduction.metal"),
    ),
    // The two structural fixtures, and the pair is deliberate. The mirror is the
    // only one whose body emits `BinaryOp::IndexSubtract`, so compiling it is
    // what turns "the emitter writes an unsigned difference" into "the Metal
    // compiler accepts the mirrored offset chain this backend now emits". The
    // broadcast is the only fixture whose *read* and *write* buffers declare
    // different element counts on a one-read entry point, which is the
    // signature shape a widening read needs.
    //
    // They are also the first goldens carrying no arithmetic at all — a
    // structural family computes nothing — so they are the only checked-in
    // artifacts whose provenance block states an *empty* unrealizable-obligation
    // list beside a non-empty one everywhere else. That makes the "none" wording
    // reachable evidence rather than an unexercised branch of `assemble`.
    (
        "structural_mirrored_reindex.metal",
        include_str!("../goldens/structural_mirrored_reindex.metal"),
    ),
    (
        "structural_widening_broadcast.metal",
        include_str!("../goldens/structural_widening_broadcast.metal"),
    ),
    // The one fixture whose body calls an elementary function and divides.
    // Every other golden's arithmetic is `*`, `+`, and comparison — operators
    // the compiler cannot fail to know — so this is the first checked-in
    // artifact whose acceptance depends on a *name* resolving: `precise::exp`
    // is a call into a nested standard-library namespace, declared by a header
    // this backend never reads, and a namespace-qualified call is the class of
    // spelling that satisfies a string assertion and still fails to translate.
    // It is also the only fixture emitting `/` between two `float`s, which is
    // the spelling MSL Table 8.1 states an accuracy for and therefore the one
    // the activation contract is derived against.
    (
        "elementary_silu_activation.metal",
        include_str!("../goldens/elementary_silu_activation.metal"),
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
        ApplePlatform::MacOs,
        DeploymentMinimum::new(14, 0),
        MslVersion::Metal3_1,
    )
    .expect("MSL 3.1 is admitted from macOS 14")
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
        MetalDeploymentMinimum::new(14, 0),
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

const fn emission_realization() -> MetalEmissionRealization {
    MetalEmissionRealization::new(LaunchIndexRealization::ThreadPositionInGridUInt)
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
        MetalNumericalRequirement::PreciseFp32Functions => {
            realization.fp32_functions == Fp32Functions::Precise
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
    assert_eq!(facts.language.revision(), target.msl_version().revision());
    assert_eq!(facts.platform.as_str(), target.platform().as_str());
    assert_eq!(
        facts.deployment_minimum.major(),
        target.deployment_minimum().major()
    );
    assert_eq!(
        facts.deployment_minimum.minor(),
        target.deployment_minimum().minor()
    );
    assert_eq!(target.triple(), "air64-apple-macos14.0");

    let language = format!(
        "// Metal Shading Language: {}",
        target.msl_version().semantic_name()
    );
    let family = format!(
        "// Artifact family: {} (deployment minimum {})",
        target.platform().as_str(),
        target.deployment_minimum(),
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
    let unit = emit_translation_unit(
        &[&crate::tests::pointwise_kernel()],
        &emitter_facts(),
        emission_realization(),
    )
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
        assert_eq!(artifact.provenance.target_triple, "air64-apple-macos14.0");
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
    let unit = emit_translation_unit(
        &[&pointwise, &single, &multi, &fused],
        &emitter_facts(),
        emission_realization(),
    )
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

/// The realization the BF16 unit compiles under satisfies what it demands.
///
/// The sibling of `the_strict_realization_honours_every_requirement_emission_records`
/// at the other width, and it is not redundant with it. Finding 28 measures one
/// per-dtype difference in the strictest cell — under `safe` with
/// `-ffp-contract=fast`, `f16` fuses and `bf16` does not — so a contraction
/// conclusion drawn at one width is not evidence at another, and the flag the
/// driver actually selects for a BF16 compilation is asserted here rather than
/// inherited from the `f32` fixture's row.
///
/// **This checks the selection, not the fusion.** That `-ffp-contract=off`
/// suppresses a BF16 fusion is the retained probe's measurement; what this adds
/// is that the flag emission requires is the flag this compilation passes.
#[test]
fn the_strict_realization_honours_what_the_bf16_unit_records() {
    let unit = emit_translation_unit(
        &[&crate::tests::bf16_pointwise_kernel()],
        &emitter_facts(),
        emission_realization(),
    )
    .expect("the bounded bf16 pointwise fixture emits");
    let realization = NumericalRealization::strict_baseline();
    let flags = golden_request(unit.source()).compile_flags();
    assert!(
        unit.numerical_requirements()
            .contains(&MetalNumericalRequirement::NoFloatingPointContraction),
        "a bf16 unit forbidding contraction must record the defence at its own width"
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

/// The BF16 golden's `ushort` constant carrier is rejected when removed.
///
/// This is what makes the BF16 compile evidence non-vacuous, and it is the one
/// perturbation the `f32` goldens cannot express. `as_type` requires its source
/// and result to have the same size, and an unsuffixed MSL integer literal is
/// `uint` — so the `f32` spelling applied at `bfloat`'s width is a compile
/// error rather than a stylistic difference. Deleting the narrowing leaves
/// source that still reads as a bit-pattern reinterpretation and does not
/// compile, which is exactly the failure a byte-stability golden cannot see.
///
/// It also bounds the claim in the other direction: the fixture compiles
/// *because* the carrier is there, not because the Metal compiler is lenient
/// about `bfloat`.
#[test]
fn the_bf16_golden_without_its_ushort_carrier_is_rejected_when_a_toolchain_resolves() {
    const CARRIER: &str = "as_type<bfloat>(ushort(0x4000u))";
    const STRIPPED: &str = "as_type<bfloat>(0x4000u)";

    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let name = "pointwise_scale_bias_bf16.metal";
    let source = GOLDENS
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .map(|(_, source)| *source)
        .expect("the bf16 fixture is compiled by this module");
    assert_eq!(
        source.matches(CARRIER).count(),
        1,
        "{name}: the fixture must carry exactly one such constant"
    );
    let broken = source.replacen(CARRIER, STRIPPED, 1);
    assert!(broken.contains(STRIPPED), "the perturbation must apply");

    let error = toolchain
        .compile(&golden_request(&broken))
        .expect_err("a bfloat reinterpretation of a uint literal must not compile");
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

/// A cooperative golden stripped of its workgroup storage must be rejected.
///
/// The sibling of `a_golden_whose_helper_is_renamed_is_rejected_when_a_toolchain_resolves`,
/// and it exists for the same reason at the one construct that test cannot
/// reach. The cooperative fixture is the only one whose entry point declares
/// threadgroup storage, and its declaration is emitted by a different code path
/// from every other line in the file — so "the goldens compile" would stay green
/// if that path emitted nothing at all and the fixture were rebaselined to
/// match. Deleting the declaration leaves every staged access referring to an
/// undeclared identifier, which is exactly the failure a byte-stability golden
/// cannot see.
///
/// This is what makes the cooperative fixture's compile evidence non-vacuous:
/// it shows the Metal compiler is actually reading the staged body, not merely
/// accepting a file that happens to parse.
#[test]
fn a_cooperative_golden_without_its_staging_is_rejected_when_a_toolchain_resolves() {
    const DECLARATION: &str = "    threadgroup float tg0[3];\n";

    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let name = "cooperative_workgroup_reduction.metal";
    let source = GOLDENS
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .map(|(_, source)| *source)
        .expect("the cooperative fixture is compiled by this module");
    assert!(
        source.contains(DECLARATION),
        "{name}: the fixture must declare the tile's workgroup storage"
    );
    let uses = source.matches("tg0").count();
    assert!(
        uses > 1,
        "{name}: the staging must be declared and then accessed"
    );
    let broken = source.replacen(DECLARATION, "", 1);
    assert_eq!(
        broken.matches("tg0").count(),
        uses - 1,
        "only the declaration may be removed, so the staged accesses stay undeclared"
    );

    let error = toolchain
        .compile(&golden_request(&broken))
        .expect_err("a staged access to an undeclared allocation must not compile");
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

/// The mirrored golden stripped of its wrap must be rejected.
///
/// The sibling of the two perturbations above at the construct they cannot
/// reach, and it is deliberately aimed at the *bound* rather than at the
/// subtraction. `emit::binary_realization` records that the mirror's
/// non-negativity rests on `c < extent`, and that in this fixture the bound is
/// the emitted `%` two statements above the difference. Deleting that statement
/// leaves the difference referring to an undeclared identifier, so the compiler
/// reports it — which shows the bound-establishing statement is load-bearing
/// emitted text that the toolchain actually reads, not a comment about one.
///
/// **What it deliberately does not claim.** The exchanged form
/// `c - (extent - 1)` compiles perfectly well and computes a wrapped index; no
/// compile-stage test can catch it, which is why that perturbation lives in
/// `crate::tests` against the structured kernel verifier instead of here.
#[test]
fn the_mirrored_golden_without_its_wrap_is_rejected_when_a_toolchain_resolves() {
    const WRAP: &str = "        uint64_t v8 = v0 % v7;\n";

    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let name = "structural_mirrored_reindex.metal";
    let source = GOLDENS
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .map(|(_, source)| *source)
        .expect("the mirrored fixture is compiled by this module");
    assert!(
        source.contains(WRAP),
        "{name}: the fixture must emit the decode's own wrap"
    );
    let uses = source.matches("v8").count();
    assert!(
        uses > 1,
        "{name}: the wrapped coordinate must be defined and then mirrored"
    );
    let broken = source.replacen(WRAP, "", 1);
    assert_eq!(
        broken.matches("v8").count(),
        uses - 1,
        "only the wrap may be removed, so the mirrored difference stays undefined"
    );

    let error = toolchain
        .compile(&golden_request(&broken))
        .expect_err("a difference over an undefined coordinate must not compile");
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

/// The elementary golden's call stripped of its operand must be rejected.
///
/// The sibling of the three perturbations above at the construct they cannot
/// reach, and it is aimed at the *call* rather than at a statement that feeds
/// it. Deleting the operand leaves `precise::exp()`, which is rejected only if
/// the compiler resolved the qualified name against the overload set the Metal
/// standard library declares — the observed diagnostic names those candidates
/// from `metal_math` — so it shows the toolchain is binding this call to a real
/// declaration rather than parsing a plausible-looking file. That binding is
/// the entire new fact this fixture carries: no other golden's arithmetic
/// depends on a name resolving at all.
///
/// **What it deliberately does not claim.** It says nothing about *which* of
/// the two exponentials the emitter selected. Stripping the `precise::`
/// qualification instead of the operand leaves `exp(v7)`, which compiles — and
/// under this request's flag row links to a byte-identical library — so no
/// rejection test built on [`golden_request`] can reach the namespace at all.
/// That question needs a second flag row before it becomes observable, and it
/// is [`the_precise_namespace_survives_a_fast_row_when_a_toolchain_resolves`]
/// that asks it.
#[test]
fn the_elementary_golden_without_its_operand_is_rejected_when_a_toolchain_resolves() {
    const CALL: &str = "precise::exp(v7)";
    const STRIPPED: &str = "precise::exp()";

    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let name = "elementary_silu_activation.metal";
    let source = GOLDENS
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .map(|(_, source)| *source)
        .expect("the elementary fixture is compiled by this module");
    assert_eq!(
        source.matches(CALL).count(),
        1,
        "{name}: the fixture must carry exactly one elementary call"
    );
    let broken = source.replacen(CALL, STRIPPED, 1);
    assert!(broken.contains(STRIPPED), "the perturbation must apply");

    let error = toolchain
        .compile(&golden_request(&broken))
        .expect_err("a nullary call to a unary elementary function must not compile");
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

/// The emitted namespace selects the precise intrinsic without the flag's help.
///
/// The emitter writes `precise::exp` rather than `exp` and states why: the
/// unqualified spelling selects `air.fast_exp.f32` under the compiler's own
/// default, which is fast math, and the fast family's accuracy is MSL Table 8.2's
/// input-dependent bound rather than the constant the registered activation
/// contract is derived from. Substituting it is what ADR 0076 forbids. Until
/// this test, that reasoning was carried by an emission probe and a string
/// assertion over emitted text; here the *linked library* is asked which
/// intrinsic it references, which is the only artifact that actually answers.
///
/// It is a two-by-two because either half alone proves the wrong thing:
///
/// - Under the governed row the two spellings link to `air.exp.f32` alike — and
///   on the recorded row to byte-identical libraries — so a compilation under
///   the flags Tiler selects *cannot* distinguish them. That is the honest bound
///   on every other test in this module, and asserting it keeps the next reader
///   from citing the golden's compilation as evidence for the namespace.
/// - Under a fast row the spellings diverge, and only then. That divergence is
///   what makes the namespace a second line of defence rather than decoration:
///   an omitted or mis-selected flag still gets the contracted intrinsic.
///
/// **The fast row is a perturbation, not a supported configuration.** Nothing in
/// Tiler compiles under it; it is constructed here to observe what the flag is
/// defending against. And the assertion is deliberately about the *current*
/// toolchain: were a future `metal` to stop selecting the fast intrinsic for the
/// unqualified spelling, this test would go red, and the right response is to
/// re-derive the emitter's rationale rather than to relax the assertion, because
/// the premise it rests on would have changed.
#[test]
fn the_precise_namespace_survives_a_fast_row_when_a_toolchain_resolves() {
    /// The precise binary32 exponential AIR intrinsic, referenced by name in the
    /// linked library. Not a substring of the fast one, so the two discriminate.
    const PRECISE: &str = "air.exp.f32";
    /// The fast binary32 exponential, the substitution being defended against.
    const FAST: &str = "air.fast_exp.f32";

    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let name = "elementary_silu_activation.metal";
    let qualified = GOLDENS
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .map(|(_, source)| *source)
        .expect("the elementary fixture is compiled by this module");
    let unqualified = qualified.replacen("precise::exp(", "exp(", 1);
    assert!(
        !unqualified.contains("precise::"),
        "{name}: the perturbation must remove the only qualification"
    );
    // The compiler's own function-selection default, which is what an omitted
    // flag would deliver. Contraction stays `off`: it is a separate permission,
    // and holding it fixed keeps this about the exponential.
    let fast_row = NumericalRealization::new(MathMode::Fast, Fp32Functions::Fast, FpContract::Off);
    let compile = |source: &str, realization| {
        toolchain
            .compile(&CompileRequest::new(
                source,
                driver_target(),
                OptimizationLevel::Default,
                realization,
            ))
            .expect("both spellings translate under both rows")
    };

    let governed = NumericalRealization::strict_baseline();
    for (label, source) in [("qualified", qualified), ("unqualified", &unqualified)] {
        let artifact = compile(source, governed);
        assert!(
            library_names(&artifact.metallib, PRECISE),
            "{label}: the governed row must select {PRECISE}"
        );
        assert!(
            !library_names(&artifact.metallib, FAST),
            "{label}: the governed row must not reach {FAST}"
        );
    }
    assert_eq!(
        compile(qualified, governed).metallib,
        compile(&unqualified, governed).metallib,
        "the governed row cannot distinguish the two spellings at all, which is \
         precisely why the namespace needs the evidence below"
    );

    let qualified_fast = compile(qualified, fast_row);
    assert!(
        library_names(&qualified_fast.metallib, PRECISE),
        "the emitted namespace must hold the precise selection without the flag"
    );
    assert!(!library_names(&qualified_fast.metallib, FAST));
    let unqualified_fast = compile(&unqualified, fast_row);
    assert!(
        library_names(&unqualified_fast.metallib, FAST),
        "without the namespace the fast row substitutes {FAST}; if it no longer \
         does, the emitter's stated reason for writing the namespace has changed"
    );
    assert!(!library_names(&unqualified_fast.metallib, PRECISE));
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

/// One live-extent translation unit compiles to one library; baking N is another subject.
#[test]
fn one_live_extent_library_and_pipeline_subject_at_two_n_when_a_toolchain_resolves() {
    let Some(toolchain) = resolved_toolchain() else {
        return;
    };
    let live = crate::tests::live_row_major_kernel();
    let unit = emit_translation_unit(&[&live], &emitter_facts(), emission_realization())
        .expect("the live-extent kernel emits");
    let live_request = golden_request(unit.source());
    let prepared = toolchain
        .prepare(&live_request)
        .unwrap_or_else(|error| panic!("the live-extent unit must prepare: {error}"));
    let live_request_again = golden_request(unit.source());
    let again = toolchain
        .prepare(&live_request_again)
        .unwrap_or_else(|error| panic!("the live-extent unit must prepare twice: {error}"));
    let live_library = prepared.identity().as_bytes().to_vec();
    let again_library = again.identity().as_bytes().to_vec();
    assert_eq!(
        live_library, again_library,
        "library identity is the compilation subject; N is not in it",
    );
    let symbol = unit.entry_points()[0].symbol();
    let pipeline = [live_library.as_slice(), symbol.as_bytes()].concat();
    let pipeline_again = [again_library.as_slice(), symbol.as_bytes()].concat();
    assert_eq!(
        pipeline, pipeline_again,
        "pipeline identity is library identity plus the entry symbol",
    );

    let compiled = prepared
        .compile()
        .unwrap_or_else(|error| panic!("the live-extent unit must compile: {error}"));
    assert_eq!(&compiled.metallib[..4], b"MTLB");
    assert!(
        library_names(&compiled.metallib, symbol),
        "the linked library must name {symbol}"
    );
    assert!(
        !unit.source().contains("14ul") && !unit.source().contains("15ul"),
        "the compiled source must not bake either N: {}",
        unit.source()
    );

    let baked_14 = crate::tests::baked_dense_kernel(14);
    let baked_15 = crate::tests::baked_dense_kernel(15);
    let baked_14_unit =
        emit_translation_unit(&[&baked_14], &emitter_facts(), emission_realization())
            .expect("baked N = 14 emits");
    let baked_15_unit =
        emit_translation_unit(&[&baked_15], &emitter_facts(), emission_realization())
            .expect("baked N = 15 emits");
    let baked_14_request = golden_request(baked_14_unit.source());
    let baked_14_id = toolchain
        .prepare(&baked_14_request)
        .expect("baked N = 14 prepares")
        .identity()
        .as_bytes()
        .to_vec();
    let baked_15_request = golden_request(baked_15_unit.source());
    let baked_15_id = toolchain
        .prepare(&baked_15_request)
        .expect("baked N = 15 prepares")
        .identity()
        .as_bytes()
        .to_vec();
    assert_ne!(
        live_library.as_slice(),
        baked_14_id.as_slice(),
        "baking N = 14 must change library identity",
    );
    assert_ne!(
        baked_14_id, baked_15_id,
        "baking neighbouring extents must change library identity",
    );
    eprintln!(
        "golden_compilation: live-extent library linked {} bytes as {symbol}",
        compiled.metallib.len()
    );
}
