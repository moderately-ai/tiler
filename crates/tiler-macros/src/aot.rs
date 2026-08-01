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
//! - **A cache hit compiles nothing, and resolves the toolchain anyway.** The
//!   compiler fingerprint is an input to the compilation identity, so it must be
//!   read *before* the identity that decides hit or miss exists:
//!   `Toolchain::prepare` runs five `xcrun` queries — two `--find` and three
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
//!   The five `xcrun` answers are themselves served from Apple's own
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
//! A selection naming several families is refused for a different reason and
//! says so: one envelope carries one payload per built family, and the
//! single-payload cache orchestration below builds one. Widening that is
//! `deliver-several-artifact-families-from-one-expansion`.
//!
//! # The numerical contract is derived, not chosen
//!
//! The region grammar has no numerical statement, so the expansion states one.
//! There is nothing to choose: the bound declaration's measured `f32` row
//! flushes subnormals to zero, so `StrictF32` and `ReassociateF32` — both of
//! which require preserved input subnormals — are refused by the target's own
//! numerical contract check before any plan is assessed, and `RelaxedF32`
//! permits arithmetic contraction, which `fusion_legality` declines for any
//! region holding a multiply adjacent to an add. One contract survives, and
//! `only_one_numerical_contract_is_admissible_for_the_bound_declaration` is what
//! keeps that a measured fact rather than a preference: if the declaration ever
//! admits a second, that test fails and the choice becomes a real one to put to
//! Tom.

use core::fmt;

use tiler_build::{
    BoundMetalCompileDeclaration, BoundMetalDeclarationError, MetalAssemblyError,
    MetalPlanBuildError, accept_or_publish_metal_plan,
};
use tiler_cache::expansion::{ExpansionCache, Resolution};
use tiler_compiler::session::{
    CompileFailure, CompileRequest, NumericalContract, TargetCompileFailure, compile,
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
/// Derived rather than selected; see this module's documentation for the
/// elimination and for the test that keeps it one.
const CONTRACT: NumericalContract = NumericalContract::FlushSubnormalsToZeroF32;

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
    /// The stated selection names families this frontend cannot build.
    UnbuildableFamilies {
        /// What the stated selection asks for, rendered.
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
    /// The consumer disabled the expansion cache, which a delivering expansion
    /// cannot yet honour.
    CacheDisabled,
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
                "this `deliver` statement selects {}, and this frontend compiles exactly one \
                 target today: {buildable}. A selected family must not silently become fallback \
                 on a matching target, so a selection it cannot build is a refusal rather than a \
                 quiet downgrade. State `deliver macos;`, or state `fallback-only` to expand with \
                 the semantic fallback on every target",
                rendered_list(stated),
            ),
            Self::TargetRequest(source) => write!(
                formatter,
                "the declared Metal target profile is not a compilable target request: {source}"
            ),
            Self::Compile(source) => write!(
                formatter,
                "this region has no plan the compiler admits: {source:?}"
            ),
            Self::TargetCompile(source) => write!(
                formatter,
                "this region has no plan for the declared Metal target profile under the \
                 `{CONTRACT:?}` numerical contract: {source:?}"
            ),
            Self::NoSelectedPlan => formatter.write_str(
                "the compiler admitted this region for the declared Metal target profile but \
                 selected no plan, so there is nothing to emit",
            ),
            Self::CacheRoot(source) => write!(formatter, "{source}"),
            Self::CacheDisabled => formatter.write_str(
                "`TILER_EXPANSION_CACHE_DIR` is set to `off`, and a `deliver` statement selecting \
                 an artifact family compiles through the expansion cache, which has no \
                 store-nothing mode yet. Set it to an absolute directory path only you can write, \
                 or state `fallback-only`. `expand-a-delivering-region-with-the-cache-disabled` \
                 is the work that removes this restriction",
            ),
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
        &declaration,
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

/// Refuses a stated selection the bound declaration cannot compile.
fn require_buildable(
    selection: &ArtifactFamilySelection,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<(), AotRefusal> {
    let buildable = declaration.aot_target();
    let stated = selection.families();
    let matches = matches!(
        stated,
        [only]
            if only.family == buildable.platform()
                && only.deployment_minimum == buildable.deployment_minimum()
                && only.msl_version == buildable.msl_version()
    );
    if matches {
        return Ok(());
    }
    Err(AotRefusal::UnbuildableFamilies {
        stated: stated.iter().copied().map(rendered_family).collect(),
        buildable: rendered_target(buildable),
    })
}

/// Opens the expansion cache at the root this host's policy resolves.
fn open_cache(environment: &RootEnvironment) -> Result<ExpansionCache, AotRefusal> {
    match resolve(environment).map_err(AotRefusal::CacheRoot)? {
        CacheRootDecision::Directory { root, .. } => Ok(ExpansionCache::open(root)),
        CacheRootDecision::Disabled => Err(AotRefusal::CacheDisabled),
    }
}

#[cfg(test)]
mod tests;
