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
//! selection corresponds to, which numerical contract to compile under, and
//! where to look for the cache.
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
//! # The numerical contract was derived, and is now an open decision
//!
//! The region grammar has no numerical statement, so the expansion states one.
//! While a caller chose from four named presets there was nothing to choose: the
//! bound declaration's measured `f32` row flushes subnormals to zero, so the
//! strict and permit-reassociation contracts — both of which require preserved
//! input subnormals — were refused by the target's own numerical contract check
//! before any plan was assessed, and the relaxed one permits arithmetic
//! contraction, which `fusion_legality` declines for any region holding a
//! multiply adjacent to an add. One survived, and a test kept that a measured
//! fact rather than a preference.
//!
//! **That test has since fired, exactly as it was designed to.**
//! `NumericalContract` is composed from its dimensions now, so subnormal
//! flushing and ordered regrouping are resolved independently and this
//! declaration admits the combination of the two as well —
//! `the_bound_declaration_admits_the_two_flushing_contracts` pins the pair.
//! [`CONTRACT`] stays where it is because moving it would change what every
//! expanded program *means*, not how it is planned, and
//! `decide-the-inline-frontend-numerical-contract` is where that decision is
//! recorded for Tom.

use core::fmt;

use tiler_build::{
    BoundMetalCompileDeclaration, BoundMetalDeclarationError, MetalAssemblyError,
    MetalPlanBuildError, accept_or_publish_metal_plan,
};
use tiler_cache::expansion::{ExpansionCache, Resolution};
use tiler_compiler::session::{
    CompileFailure, CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure,
    compile,
};
use tiler_compiler::target::{TargetRequest, TargetRequestError};
use tiler_ir::semantic::SemanticProgram;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::family::{ArtifactFamilySelection, SelectedFamily};
use tiler_metal_aot::input::{MetalTarget, OptimizationLevel};

use crate::cache_root::{CacheRootDecision, RootEnvironment, RootRefusal, resolve};
use crate::delivery::{DeliveryPlan, FamilyDelivery, PlanRefusal};

/// The numerical contract every delivering expansion compiles under.
///
/// It was the sole survivor of an elimination and is now one of two admissible
/// contracts; see this module's documentation for what changed and where the
/// decision is recorded.
pub(crate) const CONTRACT: NumericalContract = NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32;

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
}

impl RouteFacts {
    /// Renders these facts as the Rust expression generated code names.
    ///
    /// The artifact and payload fields are supplied by name rather than by
    /// value, because [`DeliveryPlan::items_source`] already emitted the one
    /// byte-string literal and the `#[cfg]` selector that decides the position,
    /// and restating either here would embed the bytes twice.
    pub(crate) fn source(&self, artifact_binding: &str, payload_binding: &str) -> String {
        format!(
            "::tiler::__private::RouteFacts {{ artifact: {artifact_binding}, payload: \
             {payload_binding}, artifact_identity: {}, target_profile_key: {:?}, \
             target_profile_descriptor: {}, backend: {:?}, representation: {:?} }}",
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
}

/// Why an expansion could not deliver the artifact family it stated.
///
/// Typed and non-erasing (ADR 0074 convention 1): each authority's own refusal
/// is carried, because "could not compile" tells a consumer nothing about
/// whether to change the region, the `deliver` statement, the environment, or
/// the toolchain.
#[derive(Debug)]
pub(crate) enum AotRefusal {
    /// The region carries a symbolic extent, so it has no semantic program to
    /// optimize.
    SymbolicExtent,
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
    TargetCompile(Box<TargetCompileFailure>),
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
            Self::SymbolicExtent => formatter.write_str(
                "this region declares a symbolic extent, and a `deliver` statement selecting an \
                 artifact family compiles the region ahead of time — which needs every extent to \
                 be known at expansion time. Declare literal extents, or state `fallback-only`. \
                 `carry-symbolic-extents-into-the-semantic-program` is the work that removes this \
                 restriction",
            ),
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
            Self::TargetCompile(source) => write!(
                formatter,
                "{}",
                rendered_refusal(
                    source.class(),
                    &format!(
                        "for the declared Metal target profile under the `{CONTRACT:?}` numerical \
                         contract"
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
        CompileFailureClass::NoFeasiblePlan => format!(
            "the compiler recognizes this region and found no feasible plan {scope}. A declared \
             extent past the target's measured capacity is the usual cause, because feasibility is \
             a hard refusal with a reason rather than an expensive plan: try smaller extents, or \
             state `fallback-only` to expand with the semantic fallback on every target"
        ),
        CompileFailureClass::BudgetExhausted => format!(
            "the compiler stopped searching for a plan {scope} because a deterministic search \
             budget was exhausted; this is a fact about the region's size rather than about its \
             correctness, so a smaller region compiles and `fallback-only` expands without \
             compiling at all"
        ),
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
/// `environment` is the caller's snapshot of the two variables the cache-root
/// policy is defined over rather than something read here, for
/// [`crate::cache_root`]'s own reason: a decision that reaches for the process
/// environment cannot be exercised without one, and this crate forbids the
/// `unsafe` a test would need to mutate it.
/// `toolchain` is supplied rather than constructed for the reason
/// `Toolchain::with_launcher` exists: pointing it at a path that is not there
/// reaches the same `DriverError::ToolchainUnavailable` a host with no Apple
/// tools produces, which is how the retained-diagnostic path below is exercised
/// on a machine that does have them.
pub(crate) fn deliver(
    program: Option<&SemanticProgram>,
    selection: ArtifactFamilySelection,
    environment: &RootEnvironment,
    toolchain: &Toolchain,
) -> Result<Delivered, AotRefusal> {
    let program = program.ok_or(AotRefusal::SymbolicExtent)?;
    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().map_err(AotRefusal::Declaration)?;
    require_buildable(&selection, &declaration)?;

    let targets =
        TargetRequest::new([declaration.profile().clone()]).map_err(AotRefusal::TargetRequest)?;
    let batch =
        compile(CompileRequest::new(program, CONTRACT, targets)).map_err(AotRefusal::Compile)?;
    let compilation = batch
        .into_targets()
        .pop()
        .ok_or(AotRefusal::NoSelectedPlan)?
        .into_parts()
        .1
        .map_err(|failure| AotRefusal::TargetCompile(Box::new(failure)))?;
    let plan = compilation.selected().ok_or(AotRefusal::NoSelectedPlan)?;

    let cache = open_cache(environment)?;
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
fn open_cache(environment: &RootEnvironment) -> Result<ExpansionCache, AotRefusal> {
    match resolve(environment).map_err(AotRefusal::CacheRoot)? {
        CacheRootDecision::Directory { root, .. } => Ok(ExpansionCache::open(root)),
        CacheRootDecision::Disabled => Ok(ExpansionCache::disabled()),
    }
}

#[cfg(test)]
mod tests;
