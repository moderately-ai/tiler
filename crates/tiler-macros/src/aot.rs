//! The expansion-time AOT flow: optimize, emit, identify, look up, compile,
//! publish, read back.
//!
//! `docs/integration/frontends.md` specifies this as eight synchronous steps a
//! proc macro performs, and this module is the seventh through the second of
//! them — everything after [`crate::region`] has a verified public logical
//! program and before [`crate::delivery`] turns a plan into tokens. Nothing here
//! is a new authority: `tiler_compiler::session` optimizes, `tiler_metal`
//! emits, `tiler_metal_aot` compiles, `tiler_artifact` identifies, and
//! `tiler_cache` stores, and `tiler_build::accept_or_publish_metal_plan`
//! sequences the five. What this module owns is the three decisions a *frontend*
//! must make and no one below it can: which target declaration a stated family
//! selection corresponds to, where to look for the cache, and — since Tom made
//! the eviction automatic on 2026-08-04 — at which point in the flow that cache
//! is trimmed. The numerical contract used to be one of them and is now the
//! region's own statement; see below.
//!
//! # Expansion runs inside `rustc`, so what runs when matters
//!
//! Everything below happens in the compiler's process while the consumer's
//! crate is being compiled. Two consequences are load-bearing:
//!
//! - **`FallbackOnly` reaches none of it.** [`deliver`] is called only for a
//!   selection that `invokes_backend_compiler`, so a region stating nothing, or
//!   stating `fallback-only`, opens no cache, resolves no root, and spawns no
//!   process. Two tests hold it: `crate::delivery`'s
//!   `fallback_only_states_a_selection_that_invokes_no_backend_compiler` pins
//!   the flag [`crate::expand`] branches on, and
//!   `a_fallback_only_selection_is_refused_before_any_backend_work` below pins
//!   what happens if the branch is ever wrong — a refusal, not a compilation.
//! - **`TILER_EXPANSION_CACHE_DIR=off` reaches all of it.** ADR 0089 spells the
//!   value "expand, compile, embed, and cache nothing", so a delivering region
//!   under it runs the same eight steps and resolves to
//!   `Resolution::Uncached` — the artifact is built, validated, and embedded,
//!   and no consumer of the same subject is spared the work.
//!   [`open_cache`] states that by handing back a cache with no root at all
//!   rather than by declining, and
//!   `a_disabled_cache_delivers_the_region_and_publishes_no_file` below watches
//!   one directory both ways to hold it.
//! - **A cache hit compiles nothing, and resolves the toolchain anyway.** The
//!   compiler fingerprint is an input to the compilation identity, so it must be
//!   read *before* the identity that decides hit or miss exists:
//!   `Toolchain::prepare` runs four `xcrun` queries — two `--find` and two
//!   `--show-sdk-*` — and then executes the two located binaries to read their
//!   reported versions, on every expansion; only the `metal` and `metallib`
//!   compilation runs are inside the miss closure. `docs/integration/frontends.md`
//!   states this as the contract and carries the measurement: a warm `cargo check`
//!   expansion costs one resolution, 44–97 ms on the measured host, and a live
//!   rust-analyzer session pays it once per settled edit inside a region.
//!
//!   The `--version` executions are the load-bearing half and are deliberately
//!   *not* `xcrun` invocations: they run the binaries `prepare` already located,
//!   so the folded version describes the compiler that will produce the bytes.
//!   The four `xcrun` answers are themselves served from Apple's own
//!   `$TMPDIR/xcrun_db` cache, which is why re-running them buys less than it
//!   appears to — and why the invariant is that identity folds a fingerprint read
//!   by executing the binaries this same prepared token will execute, rather than
//!   that a resolution happens on some schedule.
//! - **A publication may trim the cache; a hit never does.** Tom's 2026-08-04
//!   decision makes the eviction automatic, and [`deliver`] is where it fires:
//!   after `accept_or_publish_metal_plan` resolves to `Resolution::Published`,
//!   under the bound [`crate::eviction`] read from the environment, at most once
//!   per process. Every other route runs nothing — a hit, an `off` cache, a
//!   `fallback-only` region, and a publication in a process that already swept.
//!   That module owns the variable, the default, the opt-out, the amortization
//!   rule, and what becomes of the report; what this module owns is the single
//!   `Resolution::Published` test that keeps a scan off the hit path.
//! - **Every resolution reads its retention back, and a quiet one says
//!   nothing.** The entry this expansion resolved to carries the debug text the
//!   Metal producer retained, and [`crate::retention`] turns it into a note on
//!   standard error when a retained run has bytes. It runs on a hit as well as
//!   on a publication, because the retention is stored precisely so a hit can
//!   serve it; it is never fatal, because a retention exists only where the
//!   compilation succeeded. That module owns the predicate, the message, and
//!   why the note is neither a `compile_error!` nor a spanned diagnostic. Tom
//!   accepted that caller-visible note on 2026-08-11 as ungated, nonfatal, and
//!   byte-faithful; it names only the completed AOT/cache phase.
//! - **A rooted cache is probed once per process, and the probe refuses
//!   nothing.** [`open_cache`] hands the root it just opened to
//!   [`crate::preflight`], which reports an unsuitable filesystem on standard
//!   error and lets the expansion continue. It runs *before* the cache is used
//!   rather than after a publication, because the symptom it describes is a root
//!   that never publishes; a disabled cache has no root and is skipped without
//!   spending the process's one probe.
//!
//! # The one family this frontend can build, and why the check is equality
//!
//! `tiler_build::BoundMetalCompileDeclaration` publishes exactly one
//! constructor, `first_macos_apple9`, and its own documentation says why:
//! "Widening this to another Apple family, OS row, or dtype is a new
//! measurement rather than a new argument." Its total projection onto the
//! driver is `air64-apple-macos26.0` at `-std=metal4.0`.
//!
//! A stated selection therefore has to *equal* that target, family by family,
//! and [`buildable_target`] is where that is checked. Equality rather than
//! compatibility, because both halves of the triple are consumer-visible
//! promises: a selection naming macOS 14.0 and an artifact compiled for macOS
//! 26.0 would deliver, under a `#[cfg(target_os = "macos")]` gate that cannot
//! see a deployment minimum, a `metallib` that a macOS 15 consumer cannot load —
//! and `docs/research/apple-targets/numerical-behaviour.md` records that a
//! wrong-family `metallib` loads and dispatches without error, so nothing
//! downstream would catch it. [`crate::delivery`]'s `PROFILE_MSL_VERSION` is
//! what makes the accepted `deliver macos;` spelling reach this target at all.
//!
//! A selection naming several families is refused for the *same* reason, once
//! per family it names that no declaration measures — `deliver macos-and-ios;`
//! is refused because iOS has no declaration, not because a second payload
//! would be one too many. Stating it the other way round would be the more
//! flattering error and the wrong one: the missing measurement is the binding
//! constraint, and it is upstream of every machinery question.
//!
//! One thing is nonetheless needed before `deliver macos-and-ios;` can succeed,
//! and it is a *measurement* rather than machinery.
//! `first-authoritative-ios-metal-compile-declaration` owns the second measured
//! declaration and is blocked on a physical iOS device. The envelope half is
//! done: `carry-one-payload-per-artifact-family-in-one-envelope` landed one
//! payload per delivery position, and `tiler_build`'s
//! `one_envelope_carries_one_payload_per_artifact_family` drives the production
//! seam over two families end to end, through a `#[cfg(test)]` fixture that may
//! not escape `cfg(test)` because its measured rows were taken on a macOS host.
//! [`deliver`] therefore hands `accept_or_publish_metal_plan` a declaration run
//! that today has exactly one entry, and gains a second when the measurement
//! does.
//!
//! # The numerical contract was derived, then decided, and is now the region's
//!
//! This module used to state a contract, because the region grammar had no way
//! to. While a caller chose from four named presets there was nothing to choose:
//! the bound declaration's measured `f32` row flushes subnormals to zero, so the
//! strict and permit-reassociation contracts — both of which require preserved
//! input subnormals — were refused by the target's own numerical contract check
//! before any plan was assessed, and the relaxed one permits arithmetic
//! contraction, which `fusion_legality` declines for any region holding a
//! multiply adjacent to an add. One survived, and a test kept that a measured
//! fact rather than a preference.
//!
//! That test fired, exactly as it was designed to: `NumericalContract` is
//! composed from its dimensions now, so subnormal flushing and ordered
//! regrouping resolve independently and this declaration admits the combination
//! of the two as well — `the_bound_declaration_admits_the_two_flushing_contracts`
//! pins the pair. Two admissible contracts is a real choice, and it is not a
//! frontend's to make silently.
//!
//! **Tom decided it on 2026-08-01, at the live session** (relayed and executed by
//! `state-the-numerical-contract-in-the-region-grammar`, whose provenance is
//! `decide-the-inline-frontend-numerical-contract`): there is no default, the
//! region states its numerical contract in its own text, and a region that
//! states none is refused at expansion with a diagnostic naming what to write.
//! So there is no contract constant here any more. [`deliver`] compiles under
//! the contract its caller resolved from the region's own `contract` statement,
//! and [`crate::numerics`] owns that vocabulary and both of its refusals.
//!
//! What stays here is the downstream half, and it is deliberately separate: a
//! stated contract this declaration cannot honour is refused by the compiler's
//! own target feasibility check, and reaches a consumer as
//! [`AotRefusal::TargetCompile`] naming the contract that was stated.
//!
//! # Reaching the `metal` stage's own refusal
//!
//! [`retained`] carries whatever the offline driver refused with, and for a long
//! time the only refusal anything could produce was `ToolchainUnavailable` — a
//! host with no Apple tools. A `CompileStage::Metal` *nonzero exit* is a
//! different event, and `retain-and-attribute-a-real-msl-failure-through-an-expansion`
//! asked which invocations can reach one. There are exactly two routes, and only
//! one of them is a consumer's to hit.
//!
//! **No region text can make `metal` reject the emitted source.** Everything a
//! consumer writes is a shape, an operation, a contract name, or a family name;
//! none of it reaches the MSL as a token. `tiler_metal`'s emitter names each
//! entry point `tiler_kernel_<digest of the kernel's canonical identity>`, each
//! NaN-canonicalization helper from the canonical bit pattern it enforces, each
//! staging allocation from its scheduled `StagingId`, and each buffer parameter
//! `b<argument-table ordinal>`; scalar constants are emitted as hexadecimal bit
//! patterns through `as_type<float>`. An `InputKey` or `OutputKey` a region
//! declares never appears in the translation unit at all — not even in the
//! signature comments, which name a `TensorRole` and an ordinal. A region that
//! reaches the driver has also passed the compiler's request recognizer, the
//! program verifier, and the structured-kernel verifier, and emission itself
//! refuses an unrealizable address space, buffer access, or binding count as
//! `MetalEmitError` before any process is spawned. So a `metal` rejection *of
//! the source* means Tiler emitted MSL its own backend believed was legal, which
//! is a defect in `tiler-metal` or in this frontend — never something an
//! invocation can be written to cause, and never something a consumer can fix by
//! changing the region.
//!
//! **A build host can reach one without any Tiler defect.** Nothing between here
//! and `Toolchain::run_stage` compares the language standard the bound
//! declaration requests against the `metal` that was resolved:
//! `Toolchain::resolve` locates the tool and reads its version banner, and the
//! banner is folded into identity rather than checked for a capability. An Apple
//! toolchain predating the declaration's measured MSL 4.0 therefore accepts the
//! resolution, runs, and refuses `-std=metal4.0` with its own diagnostic —
//! reaching a consumer as this family's retained `compile_error!`, which is the
//! correct outcome: the remedy is the host's toolchain, and the `#[cfg]` gate
//! keeps an unrelated fallback-only target building.
//!
//! Both routes are exercised against the host's real `metal` binary by
//! `a_real_metal_front_end_rejection_is_retained_under_its_family` below, which
//! names which one needed injection.
//!
//! **What the retained text does not carry is a region span, deliberately.** A
//! real MSL diagnostic names a line and column in the *emitted* translation unit,
//! and no correspondence from that position back to an `out` sub-expression
//! exists to attach: `tiler_ir`'s semantic program holds no frontend spans and
//! must not, since the compiler core stays independent of any frontend's tokens,
//! and the emitter derives every name from an identity digest rather than from
//! anything a region wrote. Building one would be a new public correspondence
//! carried across three crates, and it would point at a construct that is not at
//! fault in either route above — the source-rejection route is a Tiler defect
//! whose reader is a Tiler developer, and the host route is about the machine
//! rather than the program. It is filed as
//! `carry-a-source-correspondence-from-region-text-to-emitted-msl` and deferred
//! with that trigger: the first invocation-controlled text that reaches the
//! emitted MSL reopens it.

use core::fmt;

use tiler_build::{
    BoundMetalCompileDeclaration, BoundMetalDeclarationError, DTypeDispatchability,
    MetalAssemblyError, MetalPlanBuildError, accept_or_publish_metal_plan,
};
use tiler_cache::expansion::{ExpansionCache, Resolution};
use tiler_compiler::session::{
    BudgetRefusal, CompileFailure, CompileFailureClass, CompileRequest, TargetCompileFailure,
    compile,
};
use tiler_compiler::target::{TargetRequest, TargetRequestError};
use tiler_ir::schedule::ArithmeticType;
use tiler_ir::semantic::SemanticProgram;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::family::{ArtifactFamilySelection, SelectedFamily};
use tiler_metal_aot::input::{MetalTarget, OptimizationLevel};

use crate::cache_root::{CacheRootDecision, RootEnvironment, RootRefusal, resolve};
use crate::delivery::{DeliveryPlan, FamilyDelivery, PlanRefusal};
use crate::eviction::EvictionSchedule;
use crate::numerics::StatedContract;
use crate::preflight::{PreflightGate, report_unsuitable_root};
use crate::retention::report_retained_output;

/// The optimization level every delivering expansion compiles at.
///
/// `-O2`'s equivalent, the driver's own default. It is stated here rather than
/// left implicit because it is an input to the compilation identity, so a change
/// is a cache-key change and should read as one.
const OPTIMIZATION: OptimizationLevel = OptimizationLevel::Default;

/// What one delivering expansion produced, ready to become tokens.
///
/// The plan and the route facts are returned together because they describe one
/// artifact from two sides — what the consumer's `#[cfg]` selects, and what the
/// consumer's loader is told about the bytes — and separating them would let an
/// expansion emit one without the other.
#[derive(Debug)]
pub(crate) struct Delivered {
    /// The `#[cfg]`-gated items this selection contributes.
    pub(crate) plan: DeliveryPlan,
    /// The producer-declared facts about the embedded artifact.
    ///
    /// `None` when every selected family retained a diagnostic instead of
    /// building: there are no bytes for a route to name, and the consumer target
    /// that matched the family fails to compile rather than routing anything.
    pub(crate) route_facts: Option<RouteFacts>,
}

/// Everything an expansion tells the consumer's loader about the bytes it
/// embedded.
///
/// Every field is read off the artifact the driver just produced or off the
/// declaration it was produced under, never restated from a constant here: a
/// frontend-local copy of a backend key or a profile descriptor is a second
/// authority that can disagree with the bytes shipped beside it.
#[derive(Debug)]
pub(crate) struct RouteFacts {
    artifact_identity: Vec<u8>,
    target_profile_key: String,
    target_profile_descriptor: Vec<u8>,
    backend: String,
    representation: String,
    dtype_dispatch: Vec<(ArithmeticType, DTypeDispatchability)>,
}

impl RouteFacts {
    /// Renders these facts as the Rust expression generated code names.
    ///
    /// The artifact and payload fields are supplied by name rather than by
    /// value, because [`DeliveryPlan::items_source`] already emitted the one
    /// byte-string literal and the `#[cfg]` selector that decides the position,
    /// and restating either here would embed the bytes twice.
    pub(crate) fn source(&self, artifact_binding: &str, payload_binding: &str) -> String {
        let dtype_dispatch = self
            .dtype_dispatch
            .iter()
            .map(|(arithmetic, verdict)| {
                format!(
                    "({}, {})",
                    arithmetic_type_path(*arithmetic),
                    dtype_dispatch_path(*verdict),
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "::tiler::__private::RouteFacts {{ artifact: {artifact_binding}, payload: \
             {payload_binding}, artifact_identity: {}, target_profile_key: {:?}, \
             target_profile_descriptor: {}, backend: {:?}, representation: {:?}, dtype_dispatch: \
             &[{dtype_dispatch}] }}",
            crate::delivery::byte_string_literal(&self.artifact_identity),
            self.target_profile_key,
            crate::delivery::byte_string_literal(&self.target_profile_descriptor),
            self.backend,
            self.representation,
        )
    }

    /// Returns the canonical artifact identity these facts name.
    #[cfg(test)]
    pub(crate) fn artifact_identity(&self) -> &[u8] {
        &self.artifact_identity
    }

    /// Returns the dtype-dispatchability rows these facts carry.
    #[cfg(test)]
    pub(crate) fn dtype_dispatch(&self) -> &[(ArithmeticType, DTypeDispatchability)] {
        &self.dtype_dispatch
    }
}

/// Renders one arithmetic type as the path generated code names it.
///
/// Exhaustive, for `storage_scalar_path`'s reason applied to a second
/// vocabulary: widening the arithmetic set must be a build error here rather
/// than a verdict this frontend silently cannot spell. The path is the facade's
/// re-export of the artifact vocabulary, because that is the one an expansion's
/// consumer already depends on.
const fn arithmetic_type_path(arithmetic: ArithmeticType) -> &'static str {
    match arithmetic {
        ArithmeticType::F16 => "::tiler::artifact::program::ArithmeticType::F16",
        ArithmeticType::Bf16 => "::tiler::artifact::program::ArithmeticType::Bf16",
        ArithmeticType::F32 => "::tiler::artifact::program::ArithmeticType::F32",
        ArithmeticType::F64 => "::tiler::artifact::program::ArithmeticType::F64",
    }
}

/// Renders one dispatchability verdict as the path generated code names it.
///
/// The *runtime's* two-valued vocabulary rather than the compiler's, because the
/// value being emitted is a field of a host's `ExecutionEnvironment` and that is
/// the vocabulary a host states. The compiler's third and fourth resolutions
/// never arrive here: `BoundMetalCompileDeclaration::dtype_dispatchability_rows`
/// omits an `Unknown` or `Deferred` dtype rather than reporting one, so a row
/// this frontend emits is always an exact declaration.
const fn dtype_dispatch_path(verdict: DTypeDispatchability) -> &'static str {
    match verdict {
        DTypeDispatchability::Dispatchable => "::tiler::runtime::load::DTypeDispatch::Dispatchable",
        DTypeDispatchability::Unsupported => "::tiler::runtime::load::DTypeDispatch::Unsupported",
    }
}

/// Why an expansion could not deliver the artifact family it stated.
///
/// Typed and non-erasing (ADR 0074 convention 1): each authority's own refusal
/// is carried, because "could not compile" tells a consumer nothing about
/// whether to change the region, the `deliver` statement, the environment, or
/// the toolchain.
#[derive(Debug)]
pub(crate) enum AotRefusal {
    /// The one authoritative Metal compile declaration would not assemble.
    Declaration(BoundMetalDeclarationError),
    /// The stated selection names families no bound declaration measures.
    UnbuildableFamilies {
        /// The stated families that have no measured declaration, rendered.
        ///
        /// A subset of the selection rather than all of it: a statement mixing
        /// a measured family with an unmeasured one names only the second here,
        /// because that is the part a consumer changes. Empty means the
        /// selection named no family at all.
        stated: Vec<String>,
        /// The one target the bound declaration compiles for, rendered.
        buildable: String,
    },
    /// The target request naming the bound declaration's profile was refused.
    TargetRequest(TargetRequestError),
    /// The compiler refused the region before any target was qualified.
    Compile(CompileFailure),
    /// The compiler refused the region against the declared target profile.
    ///
    /// The stated contract's name is carried beside the failure because this is
    /// where a contract the target cannot honour arrives: a region stating
    /// `strict_f32` is refused here rather than in the grammar, and a diagnostic
    /// that did not name the contract would leave a consumer with no reason to
    /// look at the statement they wrote.
    TargetCompile {
        /// The name the region stated its contract by.
        contract: &'static str,
        /// The compiler's own target-scoped refusal.
        source: Box<TargetCompileFailure>,
    },
    /// The compiler produced no selected plan for the declared target.
    NoSelectedPlan,
    /// No expansion cache root could be resolved.
    CacheRoot(RootRefusal),
    /// Emission, AOT compilation, artifact assembly, or cache resolution failed.
    Build(Box<MetalPlanBuildError>),
    /// The produced artifact does not carry the one payload a plan needs.
    MalformedArtifact {
        /// What was wrong with it.
        detail: &'static str,
    },
    /// The selection and the outcome claimed for it do not form a plan.
    MalformedPlan(PlanRefusal),
}

impl fmt::Display for AotRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declaration(source) => write!(
                formatter,
                "the authoritative Metal compile-time declaration did not assemble, so nothing \
                 could be compiled for the stated artifact families: {source}"
            ),
            Self::UnbuildableFamilies { stated, buildable } => write!(
                formatter,
                "this `deliver` statement names {}, and no measured Metal compile-time \
                 declaration exists for it. One does exist, for {buildable}, and it is the only \
                 one: a declaration is assembled from measured rows, and widening it to another \
                 Apple family is a new measurement rather than a new argument — the retained MSL \
                 4.0 measurement covers macOS alone. A selected family must not silently become \
                 fallback on a matching target, so a family with no declaration is a refusal \
                 rather than a quiet downgrade. State `deliver macos;`, or state `fallback-only` \
                 to expand with the semantic fallback on every target. \
                 `first-authoritative-ios-metal-compile-declaration` is the work that measures a \
                 second one",
                rendered_list(stated),
            ),
            Self::TargetRequest(source) => write!(
                formatter,
                "the declared Metal target profile is not a compilable target request: {source}"
            ),
            Self::Compile(source) => {
                write!(formatter, "{}", rendered_refusal(source.class(), "at all"))
            }
            Self::TargetCompile { contract, source } => write!(
                formatter,
                "{}",
                rendered_refusal(
                    source.class(),
                    &format!(
                        "for the declared Metal target profile under the `{contract}` numerical \
                         contract this region states"
                    ),
                ),
            ),
            Self::NoSelectedPlan => formatter.write_str(
                "the compiler admitted this region for the declared Metal target profile but \
                 selected no plan, so there is nothing to emit",
            ),
            Self::CacheRoot(source) => write!(formatter, "{source}"),
            Self::Build(source) => write!(
                formatter,
                "the offline Metal compilation for the stated artifact families failed: {source}"
            ),
            Self::MalformedArtifact { detail } => write!(
                formatter,
                "`tiler::tensor!` compiled an artifact it cannot describe ({detail}); this is a \
                 defect in `tiler-macros`, not in the invocation"
            ),
            Self::MalformedPlan(source) => write!(
                formatter,
                "`tiler::tensor!` cannot deliver what it compiled; this is a defect in \
                 `tiler-macros`, not in the invocation: {source}"
            ),
        }
    }
}

/// Renders a compiler refusal as something a consumer can act on.
///
/// Derived from [`CompileFailureClass`] rather than from the refusal's `Debug`
/// rendering, and the difference is the whole point: `UnsupportedCapability {
/// phase: "strategy", rule: "input-arity" }` is a true sentence about the
/// compiler and tells a consumer who wrote a region nothing about what to
/// change. The class is the compiler's own typed statement of *which boundary*
/// refused, each boundary has a different consumer action, and the stable rule
/// key is carried through so the exact check is still nameable.
///
/// What this deliberately does not do is restate which whole programs the
/// compiler recognizes. That set belongs to
/// `crates/tiler-compiler/src/request.rs` and is being widened by
/// `admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary`,
/// so a copy of it here would be a second authority that goes stale in silence.
/// The two shapes named below are a weaker and durable claim — the ones
/// `crate::region`'s own tests compile from a region today — and they stay true
/// under any widening, because widening what is recognized cannot unrecognize
/// them.
///
/// `scope` completes "no plan …": what a pre-target refusal and a target-scoped
/// one mean is not the same thing, and a consumer reading the first should not
/// think a different target would help.
fn rendered_refusal(class: CompileFailureClass, scope: &str) -> String {
    match class {
        // Same-shape symbolic elementwise is recognized and formed; `compile()`
        // then declines at schedule because `IndexRegion` still requires a
        // fixed launch geometry. That is not an unrecognized program shape, and
        // naming the general recognizer here would send a consumer to rewrite a
        // region the compiler already accepted.
        CompileFailureClass::UnsupportedCapability {
            rule: "symbolic-extent",
        } => format!(
            "the compiler recognizes this region and cannot schedule a launch over a symbolic \
             extent, so it has no plan {scope} (the check that refused is `symbolic-extent`). A \
             `deliver` statement compiles the region during this expansion, and the compiler can \
             launch only when every iteration extent is a literal. Declare literal extents, or \
             state `fallback-only` to expand with the semantic fallback on every target"
        ),
        CompileFailureClass::UnsupportedCapability { rule } => format!(
            "this region denotes a whole program the compiler does not recognize, so it has no \
             plan {scope} (the check that refused is `{rule}`). A `deliver` statement compiles the \
             region during this expansion, and the compiler plans a region only when the whole \
             program its `out` expression denotes is one it recognizes. Two are known to compile \
             from a region today: a pointwise chain over the declared operands, as in `out (a * b) \
             + c`, and a strict serial sum over one scaled and shifted operand, as in `out \
             strict_serial_sum(x * 2.0 + 1.0, [cols])`. Write the region as one of those, or state \
             `fallback-only` to expand with the semantic fallback on every target. \
             `admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary` is the \
             work that widens what is recognized"
        ),
        // Two usual causes rather than one, and the first is new: since the
        // region states its own numerical contract, a contract the target's
        // measured behaviour cannot honour arrives here as a hard refusal.
        // Naming only the extent would send a consumer who wrote `contract
        // strict_f32;` to shrink a shape that was never the problem.
        CompileFailureClass::NoFeasiblePlan => format!(
            "the compiler recognizes this region and found no feasible plan {scope}. Feasibility \
             is a hard refusal with a reason rather than an expensive plan, and two causes are \
             usual: a numerical contract the target profile's measured behaviour cannot honour, \
             and a declared extent past its measured capacity. State a contract the target can \
             honour, try smaller extents, or state `fallback-only` to expand with the semantic \
             fallback on every target"
        ),
        // The resource, its bound, and the demand, because a consumer told only
        // that "a budget" was exhausted has to read compiler source to learn
        // which one — the reading these fields exist to remove.
        //
        // The two halves are split on `refusal()` rather than merged, and the
        // text this replaced was the merged form: it said the compiler "stopped
        // searching", which is true of a truncating stop and false of every
        // budget a macro expansion can actually reach. A bounding refusal is a
        // fact about the region's declared size that no further search escapes,
        // so "write a smaller region" is the right action; a truncating one
        // stopped a search a wider bound might finish, where that same advice
        // sends a consumer to change a region that was not the problem.
        CompileFailureClass::BudgetExhausted {
            resource,
            limit,
            actual,
        } => {
            let refused = format!(
                "the deterministic budget `{}` refused this region, so it has no plan {scope}: \
                 the limit is {limit} and the demand was {actual}",
                resource.key()
            );
            match resource.refusal() {
                BudgetRefusal::Bounding => format!(
                    "{refused}. That is a fact about the region's size rather than about its \
                     correctness, and no amount of further search reaches a plan under this \
                     bound, so a smaller region compiles and `fallback-only` expands without \
                     compiling at all"
                ),
                BudgetRefusal::Truncated => format!(
                    "{refused}, which is a lower bound on the space the search did not reach \
                     rather than a size this region requires. The search stopped before it \
                     finished, so this is a fact about how far the compiler looked rather than \
                     about the region; `fallback-only` expands without compiling at all"
                ),
            }
        }
        CompileFailureClass::InvalidRequest { rule } => format!(
            "`tiler::tensor!` built a compile request the compiler refused as malformed \
             (`{rule}`), so this region has no plan {scope}; this is a defect in `tiler-macros`, \
             not in the invocation"
        ),
        CompileFailureClass::InvalidCompilerOutput => format!(
            "the compiler produced output its own verifier refused, so this region has no plan \
             {scope}; this is a defect in Tiler, not in the invocation"
        ),
        // `CompileFailureClass` is `#[non_exhaustive]`, so a class added after
        // this build still reaches a consumer as a refusal rather than as a
        // pattern-match failure. It renders the class because that is the only
        // thing this frontend knows about a boundary it has never seen.
        other => format!(
            "the compiler refused this region, so it has no plan {scope}: {other:?}. State \
             `fallback-only` to expand with the semantic fallback on every target"
        ),
    }
}

/// Renders a stated family list the way a diagnostic names it.
fn rendered_list(stated: &[String]) -> String {
    match stated {
        [] => "no artifact family".to_owned(),
        [one] => one.clone(),
        many => many.join(", "),
    }
}

/// Renders one selected family the way a diagnostic names it.
fn rendered_family(selected: SelectedFamily) -> String {
    format!(
        "{} {} at MSL {}",
        selected.family.as_str(),
        selected.deployment_minimum,
        selected.msl_version.revision(),
    )
}

/// Renders one governed compile target the way a diagnostic names it.
fn rendered_target(target: MetalTarget) -> String {
    format!(
        "{} {} at MSL {}",
        target.platform().as_str(),
        target.deployment_minimum(),
        target.msl_version().revision(),
    )
}

/// Runs the offline Metal driver for one stated selection and returns what the
/// expansion must emit.
///
/// # Errors
///
/// Returns the exact refusing authority. Every one of them is a refusal rather
/// than a downgrade to the semantic fallback, because ADR 0053 makes a selected
/// family *required* on a matching consumer target.
/// `contract` is the region's own stated numerical contract, resolved by
/// [`crate::numerics`] and passed through unchanged: nothing here may narrow,
/// widen, or substitute it to make the declared target feasible, so a contract
/// this declaration cannot honour becomes [`AotRefusal::TargetCompile`] rather
/// than a compilation under a different meaning.
/// `environment` is the caller's snapshot of the two variables the cache-root
/// policy is defined over rather than something read here, for
/// [`crate::cache_root`]'s own reason: a decision that reaches for the process
/// environment cannot be exercised without one, and this crate forbids the
/// `unsafe` a test would need to mutate it.
/// `preflight` is the process's one root probe, supplied rather than reached for
/// so that a test states its own: the rule is per process, and a `static` gate
/// alone would let whichever test ran first decide whether any of the others
/// probed at all.
/// `eviction` is the automatic cache eviction's policy and its process's
/// amortization, supplied for the same reason and read at the same point: the
/// bound is resolved beside the root, so an unusable statement is reported on
/// any delivering expansion rather than only on one that happens to publish.
/// `toolchain` is supplied rather than constructed for the reason
/// `Toolchain::with_launcher` exists: an explicit launcher is how a host that
/// *has* Apple tools exercises the refusals of one that does not. Pointing it at
/// a path that is not there reaches the same `DriverError::ToolchainUnavailable`
/// a non-macOS host produces, and pointing it at a launcher whose `--find metal`
/// answers with a wrapper around the real compiler reaches
/// `DriverError::ToolFailure` at `CompileStage::Metal` — the two refusals
/// [`retained`] carries, both exercised below.
pub(crate) fn deliver(
    program: &SemanticProgram,
    contract: StatedContract,
    selection: ArtifactFamilySelection,
    environment: &RootEnvironment,
    preflight: &PreflightGate,
    eviction: &EvictionSchedule<'_>,
    toolchain: &Toolchain,
) -> Result<Delivered, AotRefusal> {
    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().map_err(AotRefusal::Declaration)?;
    require_buildable(&selection, &declaration)?;

    let targets =
        TargetRequest::new([declaration.profile().clone()]).map_err(AotRefusal::TargetRequest)?;
    let batch = compile(CompileRequest::new(program, contract.contract(), targets))
        .map_err(AotRefusal::Compile)?;
    let compilation = batch
        .into_targets()
        .pop()
        .ok_or(AotRefusal::NoSelectedPlan)?
        .into_parts()
        .1
        .map_err(|failure| AotRefusal::TargetCompile {
            contract: contract.name(),
            source: Box::new(failure),
        })?;
    let plan = compilation.selected().ok_or(AotRefusal::NoSelectedPlan)?;

    let cache = open_cache(environment, preflight)?;
    // Resolved here rather than after the publication below, so a consumer whose
    // statement cannot be read hears about it on any delivering expansion rather
    // than only on one that misses. It decides nothing about this artifact.
    let eviction_bound = eviction.bound();
    let accepted = match accept_or_publish_metal_plan(
        &cache,
        toolchain,
        program,
        plan,
        std::slice::from_ref(&declaration),
        OPTIMIZATION,
    ) {
        Ok(accepted) => accepted,
        // A toolchain or external-compiler failure is *family-scoped*, and the
        // contract says so: it "is retained as a family-scoped diagnostic and
        // emitted under that family's governed consumer `#[cfg]`; it is fatal
        // when the consumer target matches that requested family but does not
        // break an unrelated fallback-only target". A macro host with no Apple
        // tools must still be able to build a Linux consumer of a region that
        // stated `deliver macos;`, and an unconditional `compile_error!` here
        // would break exactly that.
        Err(failure) => return retained(selection, failure),
    };

    // The one place an automatic eviction runs. `Published` is the whole of
    // "off the hit path": a `Hit` read an entry and pays nothing, and an
    // `Uncached` resolution has no cache to trim. A publication has just spawned
    // `metal` and `metallib`, so the scan rides on work far larger than itself,
    // and [`EvictionSchedule::sweep`] admits at most one pass per process on top
    // of that. The report is deliberately dropped — `crate::eviction` states why
    // and what stands in for it.
    if let Some(bound) = eviction_bound
        && matches!(accepted.resolution(), Resolution::Published { .. })
    {
        eviction.sweep(&cache, bound);
    }

    // Read back on every resolution rather than only on the one that compiled.
    // A hit serves the retention the publishing build stored — `tiler_build`'s
    // `a_succeeding_stages_output_returns_from_a_validated_cache_hit` pins that
    // it does so without re-entering the compiler — so a developer whose cache
    // is warm still sees what the tools said about the artifact this expansion
    // resolved, rather than a diagnostic that existed once on whichever machine
    // published first.
    report_retained_output(match accepted.resolution() {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => {
            entry.retained_debug()
        }
        Resolution::Uncached { retained, .. } => retained,
    });

    let artifact = accepted.artifact();
    let [payload] = artifact.payloads() else {
        return Err(AotRefusal::MalformedArtifact {
            detail: "the produced artifact does not carry exactly one payload",
        });
    };
    let route_facts = RouteFacts {
        artifact_identity: artifact.canonical_identity().as_bytes().to_vec(),
        target_profile_key: payload.compatibility.key.as_str().to_owned(),
        target_profile_descriptor: payload.compatibility.descriptor.as_bytes().to_vec(),
        backend: payload.backend.as_str().to_owned(),
        representation: payload.representation.as_str().to_owned(),
        // Read off the declaration this artifact was compiled under, which is
        // the only other field here whose source is the declaration rather than
        // the produced envelope — and for the same reason the envelope-read
        // fields have theirs: the artifact carries no dtype-dispatchability row
        // of its own, so a consumer that needs one either receives the profile's
        // or receives nothing and refuses. `require_buildable` already proved
        // this declaration is the one every stated family compiles against.
        dtype_dispatch: declaration.dtype_dispatchability_rows(),
    };

    // The envelope is read from the resolution rather than re-encoded from the
    // verified artifact, so the bytes a consumer embeds are the exact bytes the
    // cache validated — on a hit those are the stored ones, and re-encoding
    // would embed a second encoding nothing checked against them.
    let envelope = match accepted.resolution() {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => {
            entry.envelope_bytes().to_vec()
        }
        Resolution::Uncached { envelope, .. } => envelope.clone(),
    };

    // One family, one payload: `require_buildable` already refused anything
    // else, so the positional outcome list has exactly one entry.
    let plan = DeliveryPlan::new(selection, envelope, vec![FamilyDelivery::Payload])
        .map_err(AotRefusal::MalformedPlan)?;
    Ok(Delivered {
        plan,
        route_facts: Some(route_facts),
    })
}

/// Retains a family-scoped build failure, or refuses a target-neutral one.
///
/// The split is the contract's: a *toolchain* failure is a fact about the macro
/// host and belongs under the family's consumer `#[cfg]`, while emission,
/// artifact assembly, cache-protocol, and correspondence failures are facts
/// about the program or about this frontend and must be unconditional — a
/// consumer whose target matched no family would otherwise build successfully
/// against a defect nothing reported.
///
/// The retained plan carries no artifact, which is what `DeliveryPlan::new`
/// requires of a plan where nothing built, and no route facts, because there is
/// no artifact for a route to name. The consumer target that matched gets the
/// diagnostic; every other target gets the fallback.
fn retained(
    selection: ArtifactFamilySelection,
    failure: MetalPlanBuildError,
) -> Result<Delivered, AotRefusal> {
    let family_scoped = matches!(
        failure,
        MetalPlanBuildError::Preparation(MetalAssemblyError::Driver(_))
            | MetalPlanBuildError::CacheCompilation(MetalAssemblyError::Driver(_))
    );
    if !family_scoped {
        return Err(AotRefusal::Build(Box::new(failure)));
    }
    let diagnostic = format!(
        "`tiler::tensor!` could not compile this region's artifact on this build host: {failure}",
    );
    let deliveries = selection
        .families()
        .iter()
        .map(|_| FamilyDelivery::Retained(diagnostic.clone()))
        .collect();
    let plan =
        DeliveryPlan::new(selection, Vec::new(), deliveries).map_err(AotRefusal::MalformedPlan)?;
    Ok(Delivered {
        plan,
        route_facts: None,
    })
}

/// Refuses a stated selection naming a family no bound declaration measures.
///
/// The refusal names the families that are *missing a declaration*, not the
/// whole stated selection. `deliver macos-and-ios;` states three families and
/// two of them are the problem, so listing all three beside the one buildable
/// target read as a contradiction — macOS appeared both as something the
/// statement "selects" and as the target the frontend builds. What a consumer
/// has to change is the unmeasured entries, so those are what the diagnostic
/// names.
///
/// A selection naming *no* family is refused here too, with an empty list. It
/// cannot arrive from [`crate::expand`], which calls [`deliver`] only for a
/// selection that `invokes_backend_compiler` — and that is exactly why the
/// refusal is kept: it is what a `FallbackOnly` selection meets if that branch
/// is ever wrong, so the wrong branch produces a diagnostic rather than a
/// compilation. `a_fallback_only_selection_is_refused_before_any_backend_work`
/// holds it, and an "every stated family is measured" check alone would pass
/// the empty case vacuously.
fn require_buildable(
    selection: &ArtifactFamilySelection,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<(), AotRefusal> {
    let buildable = declaration.aot_target();
    let stated = selection.families();
    let unmeasured: Vec<String> = stated
        .iter()
        .copied()
        .filter(|selected| {
            selected.family != buildable.platform()
                || selected.deployment_minimum != buildable.deployment_minimum()
                || selected.msl_version != buildable.msl_version()
        })
        .map(rendered_family)
        .collect();
    if !stated.is_empty() && unmeasured.is_empty() {
        return Ok(());
    }
    Err(AotRefusal::UnbuildableFamilies {
        stated: unmeasured,
        buildable: rendered_target(buildable),
    })
}

/// Opens the expansion cache this host's policy decides on.
///
/// `off` is a decision and not a refusal, so it opens a cache that stores
/// nothing rather than declining to expand: ADR 0089 spells the value "expand,
/// compile, embed, and cache nothing", which means a delivering region still
/// compiles and still embeds its artifact — it simply shares the compiler work
/// with nobody. `ExpansionCache::disabled` takes no root, which is what keeps
/// that promise structural rather than remembered.
///
/// A decided root is probed here, once per process, and the probe decides
/// nothing about this expansion: [`crate::preflight`] reports a filesystem the
/// publication protocol cannot rely on and returns. The call sits on the
/// `Directory` arm rather than after the `match`, because `off` has no root to
/// probe and must not spend the process's one probe on having none.
fn open_cache(
    environment: &RootEnvironment,
    preflight: &PreflightGate,
) -> Result<ExpansionCache, AotRefusal> {
    match resolve(environment).map_err(AotRefusal::CacheRoot)? {
        CacheRootDecision::Directory { root, .. } => {
            let cache = ExpansionCache::open(root);
            report_unsuitable_root(preflight, &cache);
            Ok(cache)
        }
        CacheRootDecision::Disabled => Ok(ExpansionCache::disabled()),
    }
}

#[cfg(test)]
mod tests;
