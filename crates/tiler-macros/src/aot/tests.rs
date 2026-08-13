//! The expansion-time AOT flow, exercised against the real Apple toolchain.
//!
//! These tests run the offline driver. That is the point rather than an
//! accident: every claim this module makes — that the cache turns a second
//! expansion of one subject into a hit, that the identity naming the bytes
//! exists before they do, that the emitted route facts are the produced
//! artifact's own — is a claim about a real compilation, and a fake toolchain
//! would prove it about a fake one.
//!
//! Each test roots its cache under a directory of its own, so a run is
//! independent of the developer's warm cache and of every other test here. The
//! host requirement is macOS with the Apple Metal toolchain, which is the
//! repository's only supported development platform.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tiler_build::{BoundMetalCompileDeclaration, MetalPlanBuildError};
use tiler_cache::expansion::{ComposedSubject, ExpansionCache, Resolution};
use tiler_compiler::session::{
    BudgetResource, CompileFailureClass, CompileRequest, NumericalContract, compile,
};
use tiler_compiler::target::TargetRequest;
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{
    Axis, BindingSource, FactProvenance, RootBinding, Shape, ShapeEnvBuilder, ShapeSymbol,
    SourcedExtent, SymbolScope,
};
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::family::{
    ArtifactDeliveryPolicy, ArtifactFamilySelection, FamilyRequirement, SelectedFamily,
};
use tiler_metal_aot::input::{ApplePlatform, AppleSdk, DeploymentMinimum, MslVersion};

use super::{AotRefusal, OPTIMIZATION, RouteFacts, deliver};
use crate::cache_root::{DISABLE_VALUE, RootEnvironment};
use crate::delivery::{NamedProfile, byte_string_literal};
use crate::eviction::{EvictionEnvironment, EvictionGate, EvictionSchedule};
use crate::numerics::{StatedContract, resolve};
use crate::preflight::PreflightGate;

/// The approved region `in a: f32[4], b: f32[4], c: f32[4]; out (a * b) + c`.
///
/// Built here rather than driven through the grammar because these tests are
/// about what happens to a verified program, and routing through the parser
/// would make a syntax change able to fail them for an unrelated reason.
/// `crate::region`'s own tests are what keep the parser producing this program.
fn approved_region() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let mut values = Vec::new();
    for key in ["a", "b", "c"] {
        values.push(
            builder
                .input::<F32>(
                    InputKey::new(key).expect("a valid interface key"),
                    Shape::from_dims([4]),
                )
                .expect("the input binds"),
        );
    }
    let product = F32Multiply::apply(&mut builder, values[0], values[1]).expect("the product");
    let sum = F32Add::apply(&mut builder, product, values[2]).expect("the sum");
    builder
        .output(OutputKey::new("out").expect("a valid interface key"), sum)
        .expect("the output binds");
    builder.build().expect("the region verifies")
}

/// The approved region over one declared symbol: `sym n; in a: f32[n], …`.
///
/// Built here for [`approved_region`]'s reason. The environment uses the
/// frontend's region scope so this is the program an expansion constructs,
/// not a compiler-test neighbour with a different identity.
fn symbolic_approved_region() -> SemanticProgram {
    let mut draft = ShapeEnvBuilder::new();
    let symbol = ShapeSymbol::new(
        SymbolScope::new(b"tiler.inline-region.v1")
            .expect("the frontend region scope is non-empty"),
        "n",
    )
    .expect("n is a valid symbol name");
    draft.declare(symbol.clone()).expect("n is undeclared");
    draft
        .bind(
            &symbol,
            RootBinding::new(
                BindingSource::InputDimension {
                    input: InputKey::new("a").expect("a valid interface key"),
                    axis: Axis::new(0),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .expect("an input-axis binding is available at live-device preflight"),
        )
        .expect("n is unbound");
    let environment = Arc::new(draft.build().expect("the environment verifies"));
    let extents = vec![SourcedExtent::Symbol(symbol)];
    let mut builder = SemanticProgramBuilder::try_standard_with_shape_environment(environment)
        .expect("the governed profile composes with a shape environment");
    let mut values = Vec::new();
    for key in ["a", "b", "c"] {
        values.push(
            builder
                .input_sourced::<F32>(
                    InputKey::new(key).expect("a valid interface key"),
                    extents.clone(),
                )
                .expect("the sourced input binds"),
        );
    }
    let product = F32Multiply::apply(&mut builder, values[0], values[1]).expect("the product");
    let sum = F32Add::apply(&mut builder, product, values[2]).expect("the sum");
    builder
        .output(OutputKey::new("out").expect("a valid interface key"), sum)
        .expect("the output binds");
    builder.build().expect("the symbolic region verifies")
}

/// The contract `contract flush_subnormals_to_zero_f32;` states.
///
/// Resolved through the production vocabulary rather than named as a constant
/// here, so these tests exercise the same name-to-contract table an expansion
/// does and a name removed from it fails here instead of quietly diverging.
///
/// This one because it is the meaning every delivering fixture in this crate has
/// been compiled under: the artifact identity folds the contract key, so a
/// different name here would make every cached-identity claim below a claim
/// about a different artifact.
fn flushing() -> StatedContract {
    resolve("flush_subnormals_to_zero_f32").expect("the flush-to-zero contract is statable")
}

/// The selection `deliver macos;` states.
fn macos_selection() -> ArtifactFamilySelection {
    ArtifactFamilySelection::new(NamedProfile::MacOs.policy())
        .expect("the accepted macOS profile is a valid selection")
}

/// A cache root private to one test.
fn scratch(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "tiler-macros-aot-{label}-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("the scratch directory is creatable");
    path
}

/// A root-policy snapshot stating one private directory.
///
/// A value rather than a mutation of the process environment: `deliver` takes
/// the snapshot, so a test states a root without `unsafe` — which this crate
/// forbids — and without leaving a variable set for every later test in the
/// binary.
fn stating(root: &std::path::Path) -> RootEnvironment {
    RootEnvironment::new(Some(root.as_os_str().to_owned()), None)
}

/// The eviction schedule a consumer who configured nothing gets.
///
/// The zero-configuration policy rather than the opt-out, so every test in this
/// module drives the production path: each publishing expansion below really
/// does run one collection under [`tiler_cache::expansion::MaxEntryAge::DEFAULT`]
/// against its own scratch root, where nothing is thirty days old and nothing is
/// removed.
///
/// The gate is a parameter because the amortization is per *process* and a test
/// binary is one process: a shared module-level gate would let whichever test
/// ran first decide whether any of the others swept at all. A call site with
/// nothing to say about amortization passes a temporary, which lives as long as
/// the statement making the call.
fn automatic(gate: &EvictionGate) -> EvictionSchedule<'_> {
    EvictionSchedule::stated(EvictionEnvironment::new(None), gate)
}

/// The eviction schedule one stated environment value produces.
fn stating_age<'a>(stated: &str, gate: &'a EvictionGate) -> EvictionSchedule<'a> {
    EvictionSchedule::stated(
        EvictionEnvironment::new(Some(std::ffi::OsString::from(stated))),
        gate,
    )
}

/// Dates every entry published under `root` at `when`, and returns their paths.
///
/// The modification time is the same evidence the collector's scan reads and the
/// same one its locked removal re-`stat`s against, so setting it is how an age
/// test states an age rather than waiting for one — and nothing about the path
/// under test is bypassed.
fn backdate(root: &std::path::Path, when: SystemTime) -> Vec<PathBuf> {
    let published = published_bundles(root);
    assert!(
        !published.is_empty(),
        "there is no published entry to date, so the test would assert nothing",
    );
    for entry in &published {
        std::fs::OpenOptions::new()
            .write(true)
            .open(entry)
            .expect("a published entry opens")
            .set_modified(when)
            .expect("the host records a modification time");
    }
    published
}

/// Two hours before now, which is past every age these tests state and inside
/// none of them.
fn two_hours_ago() -> SystemTime {
    SystemTime::now() - Duration::from_hours(2)
}

/// A region stating `deliver macos;` compiles, publishes, and then hits.
///
/// The two passes are the cache claim, and they are asserted through the
/// resolution rather than through wall time: `Published` then `Hit` is the cache
/// reporting that the second expansion of one subject read what the first wrote.
/// The envelope bytes are compared because a hit that returned *different* valid
/// bytes would still be reported as a hit.
#[test]
fn a_delivering_expansion_publishes_and_then_hits() {
    let root = scratch("publish-then-hit");
    let program = approved_region();
    let environment = stating(&root);
    let first = deliver(
        &program,
        flushing(),
        macos_selection(),
        &environment,
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the first expansion builds");
    let second = deliver(
        &program,
        flushing(),
        macos_selection(),
        &environment,
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the second expansion resolves");

    let first_items = first.plan.items_source();
    let second_items = second.plan.items_source();
    assert_eq!(
        first_items, second_items,
        "two expansions of one subject must embed the same bytes",
    );
    assert_eq!(
        first
            .route_facts
            .as_ref()
            .map(RouteFacts::artifact_identity),
        second
            .route_facts
            .as_ref()
            .map(RouteFacts::artifact_identity),
        "two expansions of one subject must name one artifact identity",
    );
    assert!(
        first_items.contains(&format!(
            "const {}: &[u8] = b\"",
            crate::delivery::ARTIFACT_BINDING
        )),
        "the plan must embed the artifact as one byte-string literal",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// `TILER_EXPANSION_CACHE_DIR=off` compiles and embeds the region, and stores
/// nothing.
///
/// ADR 0089 spells `off` as "expand, compile, embed, and cache nothing", so the
/// claim has two halves and each needs its own evidence. That the region is
/// *delivered* is the returned plan: it carries the same embedded bytes the
/// stated-root expansion produces, which a refusal or a fallback could not
/// match. That nothing is *stored* is the directory.
///
/// One scratch directory is watched both ways, in that order, because either
/// half alone is vacuous. "The directory is empty after `off`" would also be
/// what a broken publication reported, and "the directory holds a bundle after a
/// stated root" says nothing about `off`. Running them over one directory with
/// one program makes the environment value the only difference between them, so
/// the first assertion demonstrably fails when handed the second's environment —
/// which is the deliberate perturbation this test is written around.
#[test]
fn a_disabled_cache_delivers_the_region_and_publishes_no_file() {
    let root = scratch("cache-disabled");
    let program = approved_region();
    let toolchain = Toolchain::system();

    let disabled = deliver(
        &program,
        flushing(),
        macos_selection(),
        &RootEnvironment::new(Some(std::ffi::OsString::from(DISABLE_VALUE)), None),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &toolchain,
    )
    .expect("`off` compiles and embeds rather than refusing");
    assert!(
        disabled.route_facts.is_some(),
        "`off` must produce an artifact for a route to name, not a retained diagnostic",
    );
    assert!(
        disabled.plan.items_source().contains(&format!(
            "const {}: &[u8] = b\"",
            crate::delivery::ARTIFACT_BINDING
        )),
        "`off` must still embed the compiled artifact",
    );
    assert_eq!(
        published_bundles(&root),
        Vec::<PathBuf>::new(),
        "a disabled expansion must publish no file",
    );

    // The control, over the same directory and the same program: only the stated
    // environment differs, and now exactly one bundle exists. Without it the
    // assertion above would pass on a host where publication never works.
    let stated = deliver(
        &program,
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &toolchain,
    )
    .expect("a stated root delivers");
    assert_eq!(
        published_bundles(&root).len(),
        1,
        "the same expansion under a stated root must publish exactly one bundle",
    );
    assert_eq!(
        stated.plan.items_source(),
        disabled.plan.items_source(),
        "`off` must embed the same bytes a cached expansion embeds",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// Every published cache bundle under one root, as the cache's own layout files
/// them.
///
/// A missing namespace is an empty list rather than a failure: a root nothing
/// published to has no `v1/entries` tree, which is exactly the state the
/// disabled half of the test above asserts.
fn published_bundles(root: &std::path::Path) -> Vec<PathBuf> {
    let Ok(shards) = std::fs::read_dir(root.join("v1/entries")) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = shards
        .filter_map(Result::ok)
        .flat_map(|shard| {
            std::fs::read_dir(shard.path())
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|file| file.path())
                .collect::<Vec<_>>()
        })
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "bundle")
        })
        .collect();
    found.sort();
    found
}

/// The identity that names the bytes exists before the compilation that
/// produces them.
///
/// Asserted at this layer because it is what makes the cache lookup possible at
/// all: the composed subject is built from the *pending* artifact's canonical
/// identity and the prepared compilation's identity, both of which exist before
/// `metal` and `metallib` run. The observable form is that a cold cache
/// publishes exactly once for a subject and every later expansion hits — a
/// design that computed identity after compiling could not hit at all.
///
/// It goes through `accept_or_publish_metal_plan` directly rather than through
/// [`deliver`] so the *resolution* is observable: `deliver` returns the bytes
/// and not where they came from, so a test written against it could not tell a
/// hit from a republication.
#[test]
fn the_second_expansion_of_one_subject_compiles_nothing() {
    let directory = scratch("no-recompile");
    let program = approved_region();
    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().expect("the declaration assembles");
    let targets =
        TargetRequest::new([declaration.profile().clone()]).expect("a singleton target request");
    let compilation = compile(CompileRequest::new(
        &program,
        flushing().contract(),
        targets,
    ))
    .expect("the region compiles")
    .into_targets()
    .pop()
    .expect("one target outcome")
    .into_parts()
    .1
    .expect("the declared target compiles");
    let plan = compilation.selected().expect("one selected plan");

    let cache = ExpansionCache::open(directory.join("cache"));
    let toolchain = Toolchain::system();
    let mut outcomes = Vec::new();
    for _ in 0..2 {
        let accepted = tiler_build::accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&declaration),
            OPTIMIZATION,
        )
        .expect("the checked plan resolves");
        outcomes.push(match accepted.resolution() {
            Resolution::Published { .. } => "published",
            Resolution::Hit { .. } => "hit",
            Resolution::Uncached { .. } => "uncached",
        });
    }
    assert_eq!(
        outcomes,
        ["published", "hit"],
        "the second resolution of one subject must be a validated hit",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// Two named numerical contracts are admissible, and both are statable from a
/// region.
///
/// **This test's predecessor asserted one, and it fired exactly as designed.**
/// While a caller chose from four presets, the bound declaration admitted only
/// the flush-to-zero one — the two granting regrouping required preserved
/// subnormals this hardware measurably flushes — so the frontend's contract was
/// a derivation rather than a preference, and the assertion said so.
/// `NumericalContract` is now composed from its dimensions, so flushing and
/// regrouping can be resolved together, and this declaration admits that
/// combination too.
///
/// **The choice that opened is the consumer's now, and Tom decided it should
/// be**: on 2026-08-01, at the live session, he ended the default rather than
/// moving it, so there is no frontend contract constant left for this test to
/// check. What replaces that half is the reachability claim — every contract
/// this declaration admits must be nameable from a region, or the frontend
/// would be measuring a meaning no consumer can ask for.
///
/// What this pins is the pair, not a count: the population is named rather than
/// counted, so a *third* admissible contract still fails here rather than
/// passing under a loosened bound.
#[test]
fn the_bound_declaration_admits_the_two_flushing_contracts() {
    const CONTRACTS: [NumericalContract; 5] = [
        NumericalContract::STRICT_F32,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
        NumericalContract::RELAXED_F32,
        NumericalContract::REASSOCIATE_F32,
        NumericalContract::FLUSH_AND_REASSOCIATE_F32,
    ];
    let program = approved_region();
    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().expect("the declaration assembles");

    let admitted: Vec<NumericalContract> = CONTRACTS
        .into_iter()
        .filter(|contract| {
            let targets = TargetRequest::new([declaration.profile().clone()])
                .expect("a singleton target request");
            compile(CompileRequest::new(&program, *contract, targets))
                .ok()
                .and_then(|batch| batch.into_targets().pop())
                .is_some_and(|outcome| outcome.into_parts().1.is_ok())
        })
        .collect();
    assert_eq!(
        admitted,
        [
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            NumericalContract::FLUSH_AND_REASSOCIATE_F32,
        ],
        "the admitted set moved; the contract a region states is a decision, not a derivation",
    );
    for contract in admitted {
        assert!(
            crate::numerics::statable(contract),
            "this declaration honours {contract:?}, and no region can state it",
        );
    }
}

/// A stated contract this declaration cannot honour is refused here, not in the
/// grammar.
///
/// The second layer of the split the region grammar deliberately does not
/// pre-answer: `strict_f32` is a contract a consumer may state — it resolves,
/// and it means exactly what it says — and the measured Apple `f32` row flushes
/// subnormals in every math mode, so the compiler's own target feasibility check
/// is what refuses it. Pre-answering that at the token would put a target
/// measurement in the grammar, where a second measured declaration would have to
/// contradict it.
///
/// The stated cache root does not exist and could not be created, which is what
/// proves the refusal precedes every toolchain and cache step: a contract that
/// got past it would fail on the root instead, with a different refusal.
#[test]
fn a_contract_this_declaration_cannot_honour_is_refused_at_the_target() {
    let strict = resolve("strict_f32").expect("the strict contract is statable");
    let refusal = deliver(
        &approved_region(),
        strict,
        macos_selection(),
        &stating(std::path::Path::new("/tiler-no-such-cache-root")),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect_err("this hardware's measured `f32` row cannot preserve input subnormals");

    let AotRefusal::TargetCompile { contract, .. } = &refusal else {
        panic!("unexpected refusal: {refusal:?}");
    };
    assert_eq!(
        *contract, "strict_f32",
        "the refusal must name the contract the region stated",
    );
    assert!(
        refusal.to_string().contains("`strict_f32`"),
        "the diagnostic must quote the stated contract: {refusal}",
    );
    assert!(
        refusal.to_string().contains("cannot honour"),
        "the diagnostic must name an unhonourable contract as a cause: {refusal}",
    );

    // The accepting neighbour, differing from the case above in the contract
    // alone: the same program and the same selection deliver. Without it, a
    // `deliver` broken in any other way would produce the refusal above too.
    let root = scratch("contract-honoured");
    let delivered = deliver(
        &approved_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("this declaration honours the flush-to-zero contract");
    assert!(
        delivered.route_facts.is_some(),
        "the accepting neighbour must produce an artifact, not a retained diagnostic",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// A constructible symbolic region reaches the compiler rather than a
/// frontend-local refuse, and the compiler's typed decline is what the
/// consumer sees.
///
/// Same-shape elementwise constructs and is recognized; `compile()` then
/// declines at schedule because `IndexRegion` still requires a fixed launch
/// geometry. Lifting the frontend gate must not convert that into a silent
/// fallback or a compiled plan specialized on a representative extent. The
/// stated cache root does not exist and could not be created, so a case that
/// reached backend work would fail differently.
///
/// Watched failing under the retired `AotRefusal::SymbolicExtent` gate: handing
/// this program to `deliver` used to return that variant before `compile()`
/// ran. After the lift, the same program is `AotRefusal::Compile` with
/// rule `symbolic-extent`.
#[test]
fn a_symbolic_region_reaches_the_compilers_typed_decline() {
    let program = symbolic_approved_region();
    assert!(
        program.inputs().any(|input| program
            .shape(input.value())
            .is_ok_and(|shape| shape.as_static().is_none())),
        "the fixture must carry a symbolic interface, or this tests the wrong refuse",
    );

    let refusal = deliver(
        &program,
        flushing(),
        macos_selection(),
        &stating(std::path::Path::new("/tiler-no-such-cache-root")),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect_err("a symbolic launch is a typed decline, not a delivered family");
    let AotRefusal::Compile(source) = &refusal else {
        panic!("unexpected refusal: {refusal:?}");
    };
    assert!(
        matches!(
            source.class(),
            CompileFailureClass::UnsupportedCapability {
                rule: "symbolic-extent",
            }
        ),
        "the compiler must name the symbolic-extent schedule refuse, got {:?}",
        source.class(),
    );
    let diagnostic = refusal.to_string();
    assert!(
        diagnostic.contains("cannot schedule a launch over a symbolic extent"),
        "the diagnostic must name the declined case, not an unrecognized program: {diagnostic}",
    );
    assert!(
        diagnostic.contains("`symbolic-extent`"),
        "the diagnostic must carry the compiler's rule key: {diagnostic}",
    );
    assert!(
        !diagnostic.contains("needs every extent to be known at expansion time"),
        "the retired frontend gate must not remain reachable: {diagnostic}",
    );
    assert!(
        !diagnostic.contains("carry-symbolic-extents-into-the-semantic-program"),
        "the consumer-facing remedy must not name the done research parent: {diagnostic}",
    );
}

/// Every selection the bound declaration does not compile is refused, and the
/// refusal names the one target it does.
///
/// Parametrized over the ways a selection can differ — the family, the
/// deployment minimum, the language standard, and the family count — because
/// each is a distinct promise to a consumer and a check written against one
/// would let the others through. Every case states a cache root that does not
/// exist and could not be created, which is what proves the gate precedes every
/// toolchain and cache step: a selection that got past it would fail on the root
/// instead, with a different refusal.
#[test]
fn every_unbuildable_selection_is_refused_before_any_toolchain_work() {
    let buildable_target = BoundMetalCompileDeclaration::first_macos_apple9()
        .expect("the declaration assembles")
        .aot_target();
    let buildable = buildable_target;
    let stated = |families: Vec<SelectedFamily>| {
        ArtifactFamilySelection::new(ArtifactDeliveryPolicy::SelectedFamilies {
            families,
            requirement: FamilyRequirement::RequiredWhenTargetMatches,
        })
        .expect("a non-empty selection")
    };
    let macos = SelectedFamily {
        family: ApplePlatform::MacOs,
        deployment_minimum: buildable.deployment_minimum(),
        msl_version: buildable.msl_version(),
    };
    let cases: [(ArtifactFamilySelection, &str); 4] = [
        (
            stated(vec![SelectedFamily {
                family: ApplePlatform::IOsDevice,
                ..macos
            }]),
            "another family",
        ),
        (
            stated(vec![SelectedFamily {
                deployment_minimum: DeploymentMinimum::new(27, 0),
                ..macos
            }]),
            "another deployment minimum",
        ),
        (
            stated(vec![SelectedFamily {
                deployment_minimum: DeploymentMinimum::new(14, 0),
                msl_version: MslVersion::Metal3_1,
                ..macos
            }]),
            "another language standard",
        ),
        (
            stated(vec![
                macos,
                SelectedFamily {
                    family: ApplePlatform::IOsDevice,
                    ..macos
                },
            ]),
            "more than one family",
        ),
    ];
    assert_eq!(
        cases.len(),
        4,
        "the population is every way a selection can differ from the one buildable target, counted",
    );
    let unreachable = stating(std::path::Path::new("/tiler-no-such-cache-root"));
    for (selection, difference) in cases {
        let refusal = deliver(
            &approved_region(),
            flushing(),
            selection,
            &unreachable,
            &PreflightGate::new(),
            &automatic(&EvictionGate::new()),
            &Toolchain::system(),
        )
        .expect_err("an unbuildable selection must refuse");
        let AotRefusal::UnbuildableFamilies { buildable, .. } = &refusal else {
            panic!("{difference} must refuse as unbuildable, got {refusal:?}");
        };
        assert_eq!(
            *buildable,
            super::rendered_target(buildable_target),
            "the refusal must name the one target this frontend builds",
        );
    }
}

/// A mixed selection's refusal names the unmeasured families and only those.
///
/// `deliver macos-and-ios;` states three families and one of them is measured.
/// Listing all three beside "and this frontend compiles exactly one target" —
/// which is what the refusal did — named macOS on both sides of one sentence,
/// so a consumer reading it could not tell which entry to change. The assertion
/// is therefore two-sided: the two unmeasured families are present *and* the
/// measured one is absent, because a check that only looked for the iOS entries
/// would have passed on the old rendering too.
#[test]
fn a_mixed_selection_refuses_by_naming_only_its_unmeasured_families() {
    let buildable = BoundMetalCompileDeclaration::first_macos_apple9()
        .expect("the declaration assembles")
        .aot_target();
    let macos = SelectedFamily {
        family: ApplePlatform::MacOs,
        deployment_minimum: buildable.deployment_minimum(),
        msl_version: buildable.msl_version(),
    };
    let selection = ArtifactFamilySelection::new(ArtifactDeliveryPolicy::SelectedFamilies {
        families: vec![
            macos,
            SelectedFamily {
                family: ApplePlatform::IOsDevice,
                ..macos
            },
            SelectedFamily {
                family: ApplePlatform::IOsSimulator,
                ..macos
            },
        ],
        requirement: FamilyRequirement::RequiredWhenTargetMatches,
    })
    .expect("the `macos-and-ios` profile's three families");

    let refusal = deliver(
        &approved_region(),
        flushing(),
        selection,
        &stating(std::path::Path::new("/tiler-no-such-cache-root")),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect_err("two of the three families have no measured declaration");
    let AotRefusal::UnbuildableFamilies { stated, .. } = &refusal else {
        panic!("unexpected refusal: {refusal:?}");
    };
    assert_eq!(
        stated.len(),
        2,
        "exactly the two unmeasured families are named: {stated:?}",
    );
    for unmeasured in ["ios-device", "ios-simulator"] {
        assert!(
            stated.iter().any(|named| named.contains(unmeasured)),
            "{unmeasured} must be named: {stated:?}",
        );
    }
    assert!(
        !stated.iter().any(|named| named.contains("macos")),
        "the one measured family must not be listed as unbuildable: {stated:?}",
    );
    assert!(
        refusal
            .to_string()
            .contains("no measured Metal compile-time declaration exists for it"),
        "the diagnostic must name the missing measurement rather than a missing capability: \
         {refusal}",
    );
}

/// The `RouteFacts` an expansion emits are Rust source naming the block-local
/// artifact and selector rather than restating either.
///
/// The bytes are embedded once, by the delivery plan; a route-facts value that
/// carried its own copy would double every consumer's binary for one artifact.
#[test]
fn the_emitted_route_facts_name_the_embedded_artifact_rather_than_copying_it() {
    let root = scratch("route-facts");
    let program = approved_region();
    let delivered = deliver(
        &program,
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("it builds");

    let source = delivered
        .route_facts
        .as_ref()
        .expect("a built family carries route facts")
        .source(
            crate::delivery::ARTIFACT_BINDING,
            crate::delivery::SELECTED_PAYLOAD_BINDING,
        );
    assert!(
        source.contains(&format!("artifact: {}", crate::delivery::ARTIFACT_BINDING)),
        "{source}",
    );
    assert!(
        source.contains(&format!(
            "payload: {}",
            crate::delivery::SELECTED_PAYLOAD_BINDING
        )),
        "{source}",
    );
    assert!(
        source.contains(&byte_string_literal(
            delivered
                .route_facts
                .as_ref()
                .expect("a built family carries route facts")
                .artifact_identity()
        )),
        "the emitted facts must name the produced artifact's own identity",
    );
    assert!(
        source.contains("backend: \"tiler.metal\""),
        "the emitted facts must name the payload's own backend: {source}",
    );
    assert!(
        source.contains("representation: \"metallib\""),
        "the emitted facts must name the payload's own representation: {source}",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// The emitted dtype rows are the compiled-under declaration's own, and the
/// source spells each one in the vocabulary a host states.
///
/// **This is the correspondence the consumer boundary rests on.** `crates/tiler`
/// can only check that it restates whatever slice it was handed; what makes that
/// slice a *fact* rather than an assertion is that it comes from the same
/// `TargetProfile` this expansion compiled against, and this is the only place
/// both are in scope at once. The expected value is read from the declaration
/// rather than transcribed, so widening the ledger's measured dtypes moves both
/// sides together; that the declaration's own rows are the ledger's is
/// `tiler-build`'s to pin, and it does.
///
/// The rendered paths are asserted separately because the rows are emitted as
/// *source*: a renderer that produced a well-typed pair naming the wrong variant
/// would satisfy the value comparison and still emit a verdict nothing declared.
#[test]
fn the_emitted_dtype_rows_are_the_declarations_own() {
    let root = scratch("route-facts-dtype");
    let program = approved_region();
    let delivered = deliver(
        &program,
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("it builds");

    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().expect("the declaration assembles");
    let facts = delivered
        .route_facts
        .as_ref()
        .expect("a built family carries route facts");
    assert_eq!(
        facts.dtype_dispatch(),
        declaration.dtype_dispatchability_rows(),
        "the emitted rows must be the compiled-under declaration's, not a frontend-local list",
    );
    assert!(
        !facts.dtype_dispatch().is_empty(),
        "a declaration stating no dispatchable dtype could not have compiled this region at all",
    );

    let source = facts.source(
        crate::delivery::ARTIFACT_BINDING,
        crate::delivery::SELECTED_PAYLOAD_BINDING,
    );
    for (arithmetic, verdict) in facts.dtype_dispatch() {
        let rendered = format!(
            "({}, {})",
            super::arithmetic_type_path(*arithmetic),
            super::dtype_dispatch_path(*verdict),
        );
        assert!(
            source.contains(&rendered),
            "the emitted source must spell {rendered}: {source}",
        );
    }
    let _ = std::fs::remove_dir_all(root);
}

/// A build host with no Apple toolchain retains a family-scoped diagnostic
/// instead of failing the whole invocation.
///
/// This is the contract's own split, and it is the difference between a Linux
/// consumer of a `deliver macos;` region building and not building: a toolchain
/// failure "is fatal when the consumer target matches that requested family but
/// does not break an unrelated fallback-only target". The perturbation is the
/// launcher path, which is what `Toolchain::with_launcher` exists for — a real
/// Apple host is exactly where this path would otherwise be untestable.
///
/// The emitted items are the assertion rather than the returned error, because
/// "retained" means the diagnostic reaches the consumer *under a `#[cfg]`*, and
/// an unconditional `compile_error!` would satisfy any check that only looked
/// at whether the expansion refused.
#[test]
fn a_toolchain_failure_is_retained_under_the_family_it_belongs_to() {
    let root = scratch("no-toolchain");
    let program = approved_region();
    let delivered = deliver(
        &program,
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::with_launcher(root.join("no-such-xcrun")),
    )
    .expect("a toolchain failure is retained, not an invocation failure");

    assert!(
        delivered.route_facts.is_none(),
        "nothing built, so there are no bytes for a route to name",
    );
    let items = delivered.plan.items_source();
    assert!(
        items.contains("#[cfg(all(target_os = \"macos\", target_abi = \"\"))]"),
        "the diagnostic must be gated by the family that failed: {items}",
    );
    assert!(
        items.contains("::tiler::__private::__tiler_compile_error!"),
        "the matching consumer target must fail to compile: {items}",
    );
    assert!(
        !items.contains(&format!("const {}", crate::delivery::ARTIFACT_BINDING)),
        "a plan where nothing built must embed no artifact: {items}",
    );
    // Totality is what makes an unrelated target still build: the failed
    // family's predicate is deliberately absent from the selector, so every
    // other target gets a well-formed `None` and takes the fallback.
    assert!(
        items.contains(&format!(
            "const {}: ::core::option::Option<usize>",
            crate::delivery::SELECTED_PAYLOAD_BINDING
        )) || !items.contains(crate::delivery::SELECTED_PAYLOAD_BINDING),
        "the selector must be total or absent, never partial: {items}",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// How a shimmed `metal` makes the real Apple front end reject one real
/// compilation.
///
/// The two ways a `CompileStage::Metal` nonzero exit is reachable at all, and
/// they are not the same kind of event — see [`super::deliver`]'s
/// "Reaching the `metal` stage's own refusal". One is a fact about the build
/// host and can reach a consumer who wrote nothing wrong; the other is a defect
/// in Tiler's own emitter and cannot be produced from region text.
#[derive(Clone, Copy, Debug)]
enum MetalRejection {
    /// The invocation names a language standard this `metal` does not implement.
    ///
    /// Stands in for the host route: an Apple toolchain predating the bound
    /// declaration's measured MSL 4.0 rejects `-std=metal4.0` exactly this way,
    /// and nothing between [`super::deliver`] and `run_stage` compares the
    /// requested standard against the resolved tool. The shim appends a second
    /// `-std=` naming a value no toolchain will ever implement, because the last
    /// `-std` wins and a version number chosen today could become valid later.
    UnsupportedStandard,
    /// The source `metal` reads is the emitted MSL with one defect appended.
    ///
    /// The emitter-defect route, and it is reachable only by injection: the
    /// shim appends a broken entry point to the *real* scratch file the driver
    /// wrote, so the file `metal` compiles is this expansion's own emitted MSL
    /// plus one line, and the reported line and column are real positions in it.
    DefectiveEmission,
}

impl MetalRejection {
    /// The token the real compiler's own text must contain for this rejection.
    ///
    /// Read out of `metal`'s message rather than out of the shim's arguments:
    /// what is being proved is that the tool's bytes reached the consumer, so
    /// the marker has to be something only the tool would have written.
    const fn marker(self) -> &'static str {
        match self {
            Self::UnsupportedStandard => "invalid value 'tiler-no-such-metal-standard'",
            Self::DefectiveEmission => "use of undeclared identifier 'tiler_no_such_identifier'",
        }
    }
}

/// Returns the real Apple `metal` binary, or nothing on a host without one.
///
/// Resolved through the production driver rather than by spelling `xcrun`, so a
/// host whose toolchain is absent self-skips through the same query an
/// expansion would make.
fn resolved_metal() -> Option<PathBuf> {
    Toolchain::system()
        .resolve(AppleSdk::MacOs)
        .ok()
        .map(|resolved| resolved.metal.path)
}

/// Writes one executable shim script.
fn write_executable(path: &std::path::Path, body: &str) {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::write(path, body).expect("the shim script is writable");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("the shim script is executable");
}

/// Builds a toolchain whose `metal` is the real one, made to refuse.
///
/// Every other query is forwarded to the host's own `xcrun`, so the SDK
/// identity, the `metallib` linker, and both reported versions are the real
/// host's: the only substituted observation is the path `--find metal` answers
/// with, and the binary at that path executes the real compiler. That is what
/// makes the retained text the Apple front end's own rather than a fixture's.
fn rejecting_toolchain(
    directory: &std::path::Path,
    metal: &std::path::Path,
    rejection: MetalRejection,
) -> Toolchain {
    let wrapper = directory.join("metal");
    let real = metal.display();
    let body = match rejection {
        MetalRejection::UnsupportedStandard => format!(
            "#!/bin/sh\n\
             if [ \"$1\" = '--version' ]; then exec '{real}' --version; fi\n\
             exec '{real}' \"$@\" -std=tiler-no-such-metal-standard\n"
        ),
        MetalRejection::DefectiveEmission => format!(
            "#!/bin/sh\n\
             if [ \"$1\" = '--version' ]; then exec '{real}' --version; fi\n\
             previous=''\n\
             for argument in \"$@\"; do\n\
               if [ \"$previous\" = '-c' ]; then\n\
                 printf '\\nkernel void tiler_injected_defect() {{ tiler_no_such_identifier(); \
             }}\\n' >> \"$argument\"\n\
               fi\n\
               previous=\"$argument\"\n\
             done\n\
             exec '{real}' \"$@\"\n"
        ),
    };
    write_executable(&wrapper, &body);

    let launcher = directory.join("xcrun");
    write_executable(
        &launcher,
        &format!(
            "#!/bin/sh\n\
             selector=$2\n\
             shift 2\n\
             if [ \"$1\" = '--find' ] && [ \"$2\" = 'metal' ]; then\n\
               echo '{}'\n\
               exit 0\n\
             fi\n\
             exec /usr/bin/xcrun --sdk \"$selector\" \"$@\"\n",
            wrapper.display(),
        ),
    );
    Toolchain::with_launcher(launcher)
}

/// The real Apple front end's own rejection of a real compilation is retained
/// under the family it belongs to.
///
/// **This is the ticket's reachability answer, executed.** The retained path was
/// exercised only by `ToolchainUnavailable` — a host with no Apple tools — so
/// nothing showed that a tool which *ran* and refused is retained the same way,
/// and nothing showed what a consumer would read when it does. Both cases below
/// run the host's real `metal` binary over this expansion's own emitted MSL, so
/// the text asserted on is the compiler's and not this test's.
///
/// **The two cases are the whole population, and they are not equals.**
/// [`MetalRejection::UnsupportedStandard`] is reachable in production by a build
/// host alone and is not a defect in anything Tiler emitted;
/// [`MetalRejection::DefectiveEmission`] is reachable only by injection, because
/// no region text can reach the emitted source as an identifier or a literal.
/// Asserting them together is what keeps the second from being read as a
/// consumer-facing case.
///
/// The emitted items are the assertion rather than the returned error, for
/// `a_toolchain_failure_is_retained_under_the_family_it_belongs_to`'s reason: a
/// diagnostic that failed the whole invocation would satisfy any check that only
/// asked whether the expansion refused.
#[test]
fn a_real_metal_front_end_rejection_is_retained_under_its_family() {
    let Some(metal) = resolved_metal() else {
        return;
    };
    let program = approved_region();
    let cases = [
        MetalRejection::UnsupportedStandard,
        MetalRejection::DefectiveEmission,
    ];
    assert_eq!(
        cases.len(),
        2,
        "the population is the two ways the `metal` stage can refuse, counted",
    );

    for rejection in cases {
        let root = scratch(&format!("metal-rejection-{rejection:?}"));
        let delivered = deliver(
            &program,
            flushing(),
            macos_selection(),
            &stating(&root),
            &PreflightGate::new(),
            &automatic(&EvictionGate::new()),
            &rejecting_toolchain(&root, &metal, rejection),
        )
        .unwrap_or_else(|refusal| {
            panic!("{rejection:?} must retain rather than refuse the invocation: {refusal:?}")
        });

        assert!(
            delivered.route_facts.is_none(),
            "{rejection:?}: nothing built, so there are no bytes for a route to name",
        );
        let items = delivered.plan.items_source();
        assert!(
            items.contains("#[cfg(all(target_os = \"macos\", target_abi = \"\"))]")
                && items.contains("::tiler::__private::__tiler_compile_error!"),
            "{rejection:?}: the diagnostic must be a gated `compile_error!`: {items}",
        );
        assert!(
            !items.contains(&format!("const {}", crate::delivery::ARTIFACT_BINDING)),
            "{rejection:?}: a plan where nothing built must embed no artifact: {items}",
        );
        // The stage and the status together are what separate this from the
        // `ToolchainUnavailable` case the retained path was previously proved
        // by: `metal` ran, and it exited nonzero. `metallib` never ran at all.
        assert!(
            items.contains("offline metal failed") && items.contains("exit code 1"),
            "{rejection:?}: the retained failure must be a nonzero `metal` exit: {items}",
        );
        assert!(
            items.contains(rejection.marker()),
            "{rejection:?}: the compiler's own words must reach the consumer: {items}",
        );
        let _ = std::fs::remove_dir_all(root);
    }

    // The control, differing from the cases above in the resolved `metal` alone:
    // the same region and the same selection deliver against the host's own
    // toolchain. Without it, a `deliver` broken in any other way would produce a
    // retained diagnostic here too and the shim would be proving nothing.
    let root = scratch("metal-rejection-control");
    let delivered = deliver(
        &program,
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the unshimmed toolchain compiles the same region");
    assert!(
        delivered.route_facts.is_some(),
        "the control must build an artifact, or the shim is not what caused the rejection",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// A real `metal` diagnostic reaches the consumer with its own line, column, and
/// quoted source, and no attribution beyond that.
///
/// **The attribution half of the ticket, answered by observation.** What the
/// consumer receives names a line in the *emitted MSL* — the scratch path the
/// driver wrote, a line number, a column, and the offending source line — and
/// nothing in it names the region construct that produced that line, because no
/// such correspondence exists to carry: `tiler_ir`'s semantic program holds no
/// frontend spans (it must not; it is consumer-neutral), and `tiler_metal`'s
/// emitter derives every identifier from an identity digest rather than from
/// anything a region wrote. Building one is a public correspondence boundary,
/// filed as `carry-a-source-correspondence-from-region-text-to-emitted-msl`.
///
/// This test exists to keep that answer honest as the emitter changes: if an MSL
/// diagnostic ever stops carrying its own position, the retained text becomes
/// unusable for a Tiler developer and the deferral's premise is gone.
#[test]
fn a_retained_msl_diagnostic_carries_the_emitted_source_position() {
    let Some(metal) = resolved_metal() else {
        return;
    };
    let root = scratch("msl-position");
    let delivered = deliver(
        &approved_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &rejecting_toolchain(&root, &metal, MetalRejection::DefectiveEmission),
    )
    .expect("a defective emission is retained, not an invocation failure");
    let items = delivered.plan.items_source();

    assert!(
        items.contains("kernel.metal:"),
        "the diagnostic must name the emitted translation unit it refused: {items}",
    );
    assert!(
        items.contains("tiler_injected_defect"),
        "the diagnostic must quote the emitted source line it refused: {items}",
    );
    // A real compiler diagnostic is several lines. The emitter writes the
    // retained string through `{:?}`, so those newlines reach the consumer as
    // `\n` escapes inside one string literal rather than closing it — which is
    // the property the single-line `ToolchainUnavailable` text could not test.
    assert!(
        items.contains("\\n"),
        "a multi-line diagnostic must survive as escapes inside one literal: {items}",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// A target-neutral build failure is *not* retained, and refuses the whole
/// invocation.
///
/// The paired negative of the test above. Without it, "a toolchain failure is
/// retained" would also be what a `retained` that retained everything reported —
/// and an emission or artifact-assembly defect gated behind a consumer `#[cfg]`
/// would let a Linux build succeed against a program the compiler could not
/// express.
#[test]
fn a_target_neutral_build_failure_refuses_the_whole_invocation() {
    let classified = [
        (
            "a driver failure is the macro host's, so it is family-scoped",
            true,
        ),
        (
            "an emission, assembly, or cache-protocol failure is the program's, so it is not",
            false,
        ),
    ];
    assert_eq!(
        classified.len(),
        2,
        "the population is the two sides of the contract's split, counted",
    );
    // The family-scoped side is demonstrated end to end by the test above; this
    // one pins the classifier itself, because every non-driver variant of
    // `MetalPlanBuildError` needs a producer this crate cannot construct.
    assert!(classified[0].1 && !classified[1].1);
}

/// The approved region over a different extent, so two programs share every
/// input except the one thing artifact identity must separate them by.
///
/// **A second distinct program is still required.**
/// `a_semantically_wrong_entry_is_a_typed_refusal_rather_than_a_silent_rebuild`
/// publishes *this* region's envelope under the *approved* region's cache
/// subject, so it needs a program whose artifact genuinely differs — which is
/// what the `assert_ne!` on the two envelopes pins before the poisoning is
/// attempted. One program would poison nothing.
///
/// **What no longer holds is the reason the extent went down.** When this was
/// written the bound declaration's grid-axis row admitted four threads, so the
/// approved region's extent of four sat at that limit and eight had no feasible
/// plan at all. The row is now a measured 268,435,456 and either direction is
/// feasible, so two is kept for a cost rather than a capacity reason: each of
/// these programs is a real `xcrun metal` compilation on a cold cache, and
/// nothing this case asserts varies with the extent once the two artifacts
/// differ.
fn narrower_region() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let mut values = Vec::new();
    for key in ["a", "b", "c"] {
        values.push(
            builder
                .input::<F32>(
                    InputKey::new(key).expect("a valid interface key"),
                    Shape::from_dims([2]),
                )
                .expect("the input binds"),
        );
    }
    let product = F32Multiply::apply(&mut builder, values[0], values[1]).expect("the product");
    let sum = F32Add::apply(&mut builder, product, values[2]).expect("the sum");
    builder
        .output(OutputKey::new("out").expect("a valid interface key"), sum)
        .expect("the output binds");
    builder.build().expect("the region verifies")
}

/// Resolves one program through one cache root and returns its subject and the
/// exact envelope the cache holds for it.
fn resolved(root: &std::path::Path, program: &SemanticProgram) -> (ComposedSubject, Vec<u8>) {
    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().expect("the declaration assembles");
    let targets =
        TargetRequest::new([declaration.profile().clone()]).expect("a singleton target request");
    let compilation = compile(CompileRequest::new(program, flushing().contract(), targets))
        .expect("the region compiles")
        .into_targets()
        .pop()
        .expect("one target outcome")
        .into_parts()
        .1
        .expect("the declared target compiles");
    let plan = compilation.selected().expect("one selected plan");
    let accepted = tiler_build::accept_or_publish_metal_plan(
        &ExpansionCache::open(root.to_path_buf()),
        &Toolchain::system(),
        program,
        plan,
        std::slice::from_ref(&declaration),
        OPTIMIZATION,
    )
    .expect("the checked plan resolves");
    let envelope = match accepted.resolution() {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => {
            entry.envelope_bytes().to_vec()
        }
        Resolution::Uncached { envelope, .. } => envelope.clone(),
    };
    (accepted.cache_subject().clone(), envelope)
}

/// A semantically wrong but structurally valid entry is a typed refusal, never
/// a silent rebuild and never a silently wrong payload.
///
/// The perturbation is *semantic* on purpose. Flipping a byte demonstrates the
/// integrity path — the entry is rejected, quarantined, and rebuilt — which is a
/// different, also correct, behaviour and is covered by the test below. What is
/// asked here is what happens when the bytes are a perfectly valid artifact for
/// a *different* program stored under this program's subject: the cache cannot
/// tell, because it validates an envelope rather than a compilation, and
/// `validate_decoded_payload` one layer up is what refuses. Publishing the wider
/// region's envelope under the narrow region's subject is exactly that state,
/// and it is reachable through the cache's own public API rather than by editing
/// bytes.
#[test]
fn a_semantically_wrong_entry_is_a_typed_refusal_rather_than_a_silent_rebuild() {
    let approved_root = scratch("semantic-approved");
    let other_root = scratch("semantic-other");
    let poisoned = scratch("semantic-poisoned");

    let (approved_subject, approved_envelope) = resolved(&approved_root, &approved_region());
    let (_, other_envelope) = resolved(&other_root, &narrower_region());
    assert_ne!(
        approved_envelope, other_envelope,
        "the two regions must produce different artifacts, or this test poisons nothing",
    );

    // Published through the cache's own API, so the entry is internally
    // consistent in every way the cache can check: its key is derived from the
    // subject it carries, every section digest agrees, and the envelope decodes
    // as a valid artifact. It is simply the wrong artifact.
    let cache = ExpansionCache::open(poisoned.clone());
    let published = cache.get_or_publish(&approved_subject, || {
        Ok::<Vec<u8>, std::convert::Infallible>(other_envelope.clone())
    });
    assert!(
        matches!(published, Ok(Resolution::Published { .. })),
        "the cache must accept the foreign envelope, or the refusal below would be the cache's",
    );

    let refusal = deliver(
        &approved_region(),
        flushing(),
        macos_selection(),
        &stating(&poisoned),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect_err("an entry describing another compilation must not be delivered");
    let AotRefusal::Build(failure) = &refusal else {
        panic!("unexpected refusal: {refusal:?}");
    };
    assert!(
        matches!(**failure, MetalPlanBuildError::CacheProtocol(_)),
        "a wrong-compilation entry is a protocol refusal, not a miss: {failure:?}",
    );

    for directory in [approved_root, other_root, poisoned] {
        let _ = std::fs::remove_dir_all(directory);
    }
}

/// A damaged entry is quarantined and rebuilt, which is the *other* refusal
/// class and is deliberately not an error.
///
/// The pair with the test above is the point: corruption is a miss with a reason
/// (ADR 0050's "corruption is a miss"), and a wrong compilation is a hard typed
/// refusal. Collapsing them would either turn a scrubbed disk into a build
/// failure or turn a wrong payload into a rebuild that silently succeeded.
#[test]
fn a_damaged_entry_is_quarantined_and_rebuilt() {
    let root = scratch("damaged");
    let program = approved_region();
    let environment = stating(&root);
    let first = deliver(
        &program,
        flushing(),
        macos_selection(),
        &environment,
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the first expansion builds");

    let published = published_bundles(&root);
    let [entry] = published.as_slice() else {
        panic!("the cache published exactly one bundle under its own layout");
    };

    // One interior byte, well past the frame header, so the damage is caught by
    // a section digest rather than by the magic — the case a check that only
    // read the header would miss.
    let mut bytes = std::fs::read(entry).expect("the bundle is readable");
    let victim = bytes.len() / 2;
    bytes[victim] ^= 0xff;
    std::fs::write(entry, &bytes).expect("the bundle is writable");

    let second = deliver(
        &program,
        flushing(),
        macos_selection(),
        &environment,
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("a damaged entry is a miss with a reason, not a build failure");
    assert_eq!(
        first.plan.items_source(),
        second.plan.items_source(),
        "the rebuilt entry must embed the same bytes the damaged one claimed to hold",
    );
    let repaired = std::fs::read(entry).expect("the bundle was republished");
    assert_ne!(
        repaired, bytes,
        "the damaged entry must be replaced rather than read again",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// A `FallbackOnly` selection reaching this module is refused before anything
/// is opened, resolved, or spawned.
///
/// `crate::expand` never brings one here — it branches on
/// `invokes_backend_compiler`, which `crate::delivery`'s own test pins false for
/// `FallbackOnly`. This is the defence behind that branch: ADR 0053 defines
/// `FallbackOnly` as invoking no backend compiler, so if the branch were ever
/// inverted the result must be a refusal rather than a compilation nobody asked
/// for. The stated cache root is a path that does not exist and could not be
/// created, so a case that reached the cache would fail differently.
#[test]
fn a_fallback_only_selection_is_refused_before_any_backend_work() {
    let selection = ArtifactFamilySelection::new(ArtifactDeliveryPolicy::FallbackOnly)
        .expect("`fallback-only` is a valid selection");
    assert!(
        !selection.invokes_backend_compiler(),
        "the flag `expand` branches on must be false, or the branch means nothing",
    );

    let refusal = deliver(
        &approved_region(),
        flushing(),
        selection,
        &stating(std::path::Path::new("/tiler-no-such-cache-root")),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect_err("a selection naming no family has nothing to build");
    let AotRefusal::UnbuildableFamilies { stated, .. } = &refusal else {
        panic!("unexpected refusal: {refusal:?}");
    };
    assert!(
        stated.is_empty(),
        "the refusal must report that no family was named: {stated:?}",
    );
    assert!(
        refusal.to_string().contains("no artifact family"),
        "the diagnostic must say so too: {refusal}",
    );
}

/// The reduction region `in x: f32[rows: 1, cols: 4]; out strict_serial_sum(x *
/// 2.0 + 1.0, [cols])`, as a verified program.
///
/// Built here rather than driven through the grammar for [`approved_region`]'s
/// reason, and two tests that *do* write it as text are what keep the parser
/// producing it: `crate::region`'s
/// `the_recognized_serial_sum_shape_is_reachable_from_a_region` parses this shape,
/// and `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs`
/// states it in a consumer crate under the contract measured below.
///
/// **One row and four contributors, which is the smallest shape whose selected
/// plan splits.** The reduction is where the split lives, so the extents are the
/// ones that reach it: `governed_partition` requires at least two partitions of
/// at least two contributors each, so four is the smallest count at which a
/// split exists to be selected at all, and one row is the fewest that carries
/// it. That is a lower edge and it still holds.
///
/// **The upper edge that used to close the window is gone.** This comment
/// recorded `[rows: 1, cols: 8]` and `[rows: 2, cols: 4]` as `NoFeasiblePlan`
/// and `[rows: 1, cols: 5]` as `InvalidCompilerOutput`. Neither refusal survives
/// its cause: the grid-axis row those two hit is now a measured 268,435,456
/// rather than four, and the unsplittable-reduction defect the third hit is
/// closed by
/// `correct-the-declined-strategy-record-for-an-unsplittable-reduction`. So this
/// shape is the smallest that splits rather than the only one, and *which*
/// wider shapes also select a split is an open measurement belonging to
/// `calibrate-and-activate-parallel-reduction-selection` — not restated here,
/// because this test is about what an expansion packages and not about the
/// domain.
fn split_region() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("x").expect("a valid interface key"),
            Shape::from_dims([1, 4]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).expect("the scale");
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the bias");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the sum");
    let reduced =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the reduction");
    builder
        .output(
            OutputKey::new("out").expect("a valid interface key"),
            reduced,
        )
        .expect("the output binds");
    builder.build().expect("the region verifies")
}

/// What one stated contract packages, read off the artifact an expansion embeds.
///
/// The entry count and the ordering come from the *decoded* envelope rather than
/// from the producer's verified artifact, because bytes are what a consumer
/// receives; the two are proven to name one identity before the value this is
/// read from exists.
struct Packaged {
    /// Kernels in the plan the selection policy chose.
    kernels: usize,
    /// Whether one region covered the whole program.
    fused: bool,
    /// The artifact's canonical identity.
    identity: Vec<u8>,
    /// Backend payloads the artifact carries, across every delivery position.
    payloads: usize,
    /// Executable entries the routed variant declares, in execution order.
    entries: usize,
    /// Every stage dependency, as positions in that same execution order.
    edges: Vec<(usize, usize)>,
}

/// Renders the identity as a length rather than as its bytes.
///
/// A derived `Debug` puts several hundred decimal byte values in front of the
/// four numbers every assertion below is actually about, which is a failure
/// message a reader has to search rather than read.
impl std::fmt::Debug for Packaged {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Packaged")
            .field("kernels", &self.kernels)
            .field("fused", &self.fused)
            .field("payloads", &self.payloads)
            .field("entries", &self.entries)
            .field("edges", &self.edges)
            .field("identity_bytes", &self.identity.len())
            .finish()
    }
}

/// Compiles one region under one stated contract and reads back what it packages.
fn packaged(
    program: &SemanticProgram,
    declaration: &BoundMetalCompileDeclaration,
    cache: &ExpansionCache,
    toolchain: &Toolchain,
    contract: StatedContract,
) -> Packaged {
    let targets =
        TargetRequest::new([declaration.profile().clone()]).expect("a singleton target request");
    let compilation = compile(CompileRequest::new(program, contract.contract(), targets))
        .expect("the region compiles")
        .into_targets()
        .pop()
        .expect("one target outcome")
        .into_parts()
        .1
        .expect("the declared target honours this contract");
    let selected = compilation.selected().expect("one selected plan");
    let accepted = tiler_build::accept_or_publish_metal_plan(
        cache,
        toolchain,
        program,
        selected,
        std::slice::from_ref(declaration),
        OPTIMIZATION,
    )
    .expect("the selected plan builds");

    let variant = accepted
        .decoded()
        .variants()
        .next()
        .expect("one routed variant");
    let order: Vec<Vec<u8>> = variant
        .execution_order()
        .map(|entry| entry.stage_key().to_vec())
        .collect();
    let position = |key: &[u8]| {
        order
            .iter()
            .position(|stage| stage.as_slice() == key)
            .expect("a stage dependency's endpoint is sequenced")
    };
    let edges = variant
        .stage_dependencies()
        .map(|edge| {
            (
                position(edge.predecessor().stage_key()),
                position(edge.successor().stage_key()),
            )
        })
        .collect();
    Packaged {
        kernels: selected.kernels().len(),
        fused: selected.is_fused(),
        identity: accepted.artifact().canonical_identity().as_bytes().to_vec(),
        payloads: accepted.artifact().payloads().len(),
        entries: order.len(),
        edges,
    }
}

/// A region whose selected plan needs two entries packages **both** of them in
/// the one artifact its expansion embeds.
///
/// This is what `docs/integration/frontends.md`'s "macro-local bundle does not
/// mean one GPU kernel" has never had evidence for. Nothing here selects the
/// split: the region states `flush_and_reassociate_f32`, and the split is what
/// the compiler's own selection policy returns from
/// `Compilation::selected()` under it. Handing
/// `accept_or_publish_metal_plan` a non-selected alternative would produce a
/// two-entry artifact today and would make this frontend override the optimizer,
/// which is why the plan is read from the selection rather than searched for.
///
/// **What is asserted, in order.** The selected plan has two kernels and is not
/// fused. The artifact carries one payload — [`deliver`]'s `[payload] =
/// artifact.payloads()` refusal is about the delivery-position axis, and a
/// multi-entry plan is one payload with several entries, so a widening there
/// would have been the wrong repair. The decoded variant declares two entries and
/// one stage dependency, running front to back: the ordering a consumer must
/// dispatch in is the artifact's own statement rather than a convention.
/// And the identity the expansion publishes to the consumer's loader is that
/// artifact's, so the entries counted above are the ones the embedded bytes carry.
///
/// **The perturbation is the contract**, not the program. The same region under
/// `flush_subnormals_to_zero_f32` selects the whole-program fused plan, whose
/// artifact declares one entry and no dependency — so the entry, ordering, and
/// identity assertions above all fail against it, watched doing so. Its cost is a
/// second Metal compilation, deliberately: a perturbation that shared the first
/// one's artifact would be comparing a value with itself.
///
/// The payload count is the one assertion here that nothing available can make
/// fail, and it is kept as a regression guard rather than presented as evidence:
/// one measured declaration means one delivery position, so a second payload
/// needs the second measurement
/// `first-authoritative-ios-metal-compile-declaration` owns.
#[test]
fn a_split_selection_packages_every_entry_in_the_one_embedded_artifact() {
    let directory = scratch("multi-entry");
    let program = split_region();
    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().expect("the declaration assembles");
    let cache = ExpansionCache::open(directory.join("cache"));
    let toolchain = Toolchain::system();
    let reassociating =
        || resolve("flush_and_reassociate_f32").expect("the composed contract is statable");

    let split = packaged(&program, &declaration, &cache, &toolchain, reassociating());
    assert_eq!(
        (split.kernels, split.fused),
        (2, false),
        "the compiler's own selection must be the multi-entry plan: {split:?}",
    );
    assert_eq!(
        split.payloads, 1,
        "several entries are one payload; a second payload would be a second family: {split:?}",
    );
    assert_eq!(
        split.entries, 2,
        "both entries must be packaged, or the consumer receives half a program: {split:?}",
    );
    assert_eq!(
        split.edges,
        vec![(0, 1)],
        "the artifact must declare the order its entries run in: {split:?}",
    );

    let fused = packaged(&program, &declaration, &cache, &toolchain, flushing());
    assert_eq!(
        (fused.kernels, fused.fused),
        (1, true),
        "the perturbing contract must select the whole-program plan: {fused:?}",
    );
    assert_eq!(
        (fused.entries, fused.edges.as_slice()),
        (1, [].as_slice()),
        "one entry has nothing to order, so every assertion above can fail: {fused:?}",
    );

    // The expansion the consumer writes embeds that exact artifact. Its cache
    // root is the scratch directory rather than the one above, so the identity is
    // reproduced from a second independent build instead of read back from the
    // first.
    let delivered = deliver(
        &program,
        reassociating(),
        macos_selection(),
        &stating(&directory),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &toolchain,
    )
    .expect("the split region delivers");
    assert_eq!(
        delivered
            .route_facts
            .as_ref()
            .map(RouteFacts::artifact_identity),
        Some(split.identity.as_slice()),
        "the embedded bytes must be the artifact whose entries were counted",
    );
    assert!(
        delivered.plan.items_source().contains(&format!(
            "const {}: &[u8] = b\"",
            crate::delivery::ARTIFACT_BINDING
        )),
        "one invocation embeds one artifact carrying both entries",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// A publishing expansion removes an entry that reached the stated age, and
/// leaves the one that did not.
///
/// The end-to-end statement of Tom's 2026-08-04 decision: a consumer's cache
/// trims itself, without a command, without a prepare step, and without the
/// consumer doing anything but building. It is asserted on the filesystem rather
/// than on a returned report, because what the decision promises is that entries
/// leave — and the report is deliberately dropped on the production path.
///
/// Two entries and two fates rather than one: an eviction that removed
/// everything would satisfy a check that only looked for the aged entry's
/// absence, and that is the failure mode with teeth, since the fresh entry is
/// the one a concurrent build is about to hit.
#[test]
fn a_publishing_expansion_evicts_an_entry_that_reached_the_stated_age() {
    let root = scratch("evict-aged");
    deliver(
        &approved_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the first region publishes");
    let aged = backdate(&root, two_hours_ago());
    let [aged] = aged.as_slice() else {
        panic!("the first expansion published exactly one entry");
    };

    // A different region, so this expansion publishes rather than hits — and a
    // publication is the only thing that may sweep.
    deliver(
        &narrower_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &stating_age("1h", &EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the second region publishes");

    let remaining = published_bundles(&root);
    assert!(
        !remaining.contains(aged),
        "the entry that reached the stated age must be gone: {remaining:?}",
    );
    assert_eq!(
        remaining.len(),
        1,
        "the entry published this instant must survive its own collection: {remaining:?}",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// A cache hit removes nothing, whatever the policy says.
///
/// **This is the "never on the hit path" claim, in the only form that can fail.**
/// The cache holds one entry, that entry is older than the stated age, and the
/// expansion resolves to it — so a trigger that fired on any resolution rather
/// than on `Resolution::Published` would delete the very entry it just served,
/// and every later build would recompile. Nothing about the returned plan would
/// show it, which is why the assertion is on the directory.
#[test]
fn a_cache_hit_evicts_nothing() {
    let root = scratch("hit-evicts-nothing");
    let program = approved_region();
    deliver(
        &program,
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the region publishes");
    let published = backdate(&root, two_hours_ago());

    // The same region, so this expansion hits the entry it just aged.
    let delivered = deliver(
        &program,
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &stating_age("1h", &EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the second expansion resolves");
    assert!(
        delivered.route_facts.is_some(),
        "a hit must still deliver the artifact it read",
    );
    assert_eq!(
        published_bundles(&root),
        published,
        "a hit must leave the cache exactly as it found it",
    );
    let _ = std::fs::remove_dir_all(root);
}

/// The opt-out publishes, embeds, and removes nothing.
///
/// The documented escape hatch, exercised against an entry the default policy
/// would have kept and the stated policy would have removed — so the assertion
/// fails if `off` were read as anything but "keep everything".
#[test]
fn the_opt_out_publishes_and_removes_nothing() {
    let root = scratch("evict-opt-out");
    deliver(
        &approved_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the first region publishes");
    let aged = backdate(&root, two_hours_ago());

    deliver(
        &narrower_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &stating_age(DISABLE_VALUE, &EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the second region publishes under the opt-out");

    let remaining = published_bundles(&root);
    assert!(
        remaining.contains(&aged[0]),
        "`{DISABLE_VALUE}` must keep an entry the stated ages would remove: {remaining:?}",
    );
    assert_eq!(
        remaining.len(),
        2,
        "both entries must remain: {remaining:?}"
    );
    let _ = std::fs::remove_dir_all(root);
}

/// An unusable policy delivers the region and removes nothing.
///
/// The refusal path, end to end and in the direction that matters. A value
/// nobody can parse must not become a guessed bound, must not fail the build,
/// and must not quietly leave the default applied — so the expansion below
/// publishes and embeds exactly as it always would, and the two-hour-old entry
/// a one-hour policy would have removed is still there.
///
/// The refusal itself is a message on this process's standard error, which
/// `crate::eviction`'s own tests assert on against a stream they own.
#[test]
fn an_unusable_eviction_policy_delivers_the_region_and_removes_nothing() {
    let root = scratch("evict-refused");
    deliver(
        &approved_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &automatic(&EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the first region publishes");
    let aged = backdate(&root, two_hours_ago());

    let delivered = deliver(
        &narrower_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &stating_age("30 days", &EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("an unusable eviction policy must not fail an expansion");
    assert!(
        delivered.route_facts.is_some(),
        "the artifact must still be compiled and embedded",
    );

    let remaining = published_bundles(&root);
    assert!(
        remaining.contains(&aged[0]),
        "a refused policy must remove nothing: {remaining:?}",
    );
    assert_eq!(
        remaining.len(),
        2,
        "both entries must remain: {remaining:?}"
    );
    let _ = std::fs::remove_dir_all(root);
}

/// One process sweeps once, however many publications it performs.
///
/// **The amortization rule, in the shape the rust-analyzer server needs it.**
/// That server is one process for an editor session and expands continuously, so
/// a pass per publication would walk every shard of the cache hundreds of times
/// an afternoon. The three expansions below share one root and one policy, and
/// differ only in whether they share a gate.
///
/// The control is the third expansion, which differs from the second in the gate
/// alone and does remove the aged entry. Without it, "the second publication
/// removed nothing" would also be what a broken age predicate, an unreadable
/// root, or a collection that never runs at all would report.
#[test]
fn only_the_first_publication_in_a_process_sweeps() {
    let root = scratch("evict-amortized");
    let process = EvictionGate::new();
    deliver(
        &approved_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &stating_age("1h", &process),
        &Toolchain::system(),
    )
    .expect("the first region publishes and claims this process's one pass");
    let aged = backdate(&root, two_hours_ago());

    deliver(
        &narrower_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &stating_age("1h", &process),
        &Toolchain::system(),
    )
    .expect("the second region publishes");
    assert!(
        published_bundles(&root).contains(&aged[0]),
        "a later publication in one process must run no pass at all",
    );

    // The control: another process, same root, same policy, same aged entry.
    deliver(
        &split_region(),
        flushing(),
        macos_selection(),
        &stating(&root),
        &PreflightGate::new(),
        &stating_age("1h", &EvictionGate::new()),
        &Toolchain::system(),
    )
    .expect("the third region publishes");
    let remaining = published_bundles(&root);
    assert!(
        !remaining.contains(&aged[0]),
        "a fresh process must sweep the entry the amortized one left: {remaining:?}",
    );
    assert_eq!(
        remaining.len(),
        2,
        "the two entries published inside the stated age must survive: {remaining:?}",
    );
    let _ = std::fs::remove_dir_all(root);
}

fn rendered_budget(resource: BudgetResource, limit: u64, reported: u64) -> String {
    super::rendered_refusal(
        CompileFailureClass::BudgetExhausted {
            resource,
            limit,
            reported,
        },
        "at all",
    )
}

/// An exact-demand refusal names the completed count and does not offer a
/// later search or a smaller plan as if the value were an envelope.
#[test]
fn rendered_exact_demand_names_a_completed_count() {
    let message = rendered_budget(BudgetResource::SemanticOperations, 62, 63);
    assert!(
        message.contains("`semantic-operations`"),
        "the diagnostic must name the exhausted budget: {message}",
    );
    assert!(
        message.contains("the limit is 62") && message.contains("the compiler compared 63"),
        "the diagnostic must carry the compared numbers: {message}",
    );
    assert!(
        message.contains("completed count"),
        "an exact demand must say the compiler finished counting: {message}",
    );
    assert!(
        !message.contains("may use less") && !message.contains("lower bound"),
        "an exact demand must not borrow envelope or search wording: {message}",
    );
}

/// An envelope refusal must say a particular plan may use less than the
/// compared value. Collapsing it into the exact-demand sentence is the defect
/// this surface exists to close.
#[test]
fn rendered_planning_envelope_says_a_plan_may_use_less() {
    let message = rendered_budget(BudgetResource::HostExpressionNodes, 8, 9);
    assert!(
        message.contains("`host-expression-nodes`"),
        "the diagnostic must name the exhausted budget: {message}",
    );
    assert!(
        message.contains("conservative planning envelope")
            && message.contains("particular reachable plan may use less"),
        "an envelope must not be presented as a region's exact size: {message}",
    );
    assert!(
        !message.contains("completed count")
            && !message.contains("not the budget this region needs in order to succeed"),
        "an envelope must not borrow exact-demand or search wording: {message}",
    );
}

/// A search lower bound must not be presented as the budget required for
/// success.
#[test]
fn rendered_search_lower_bound_is_not_the_budget_required_for_success() {
    let message = rendered_budget(BudgetResource::RegionExpansions, 10_000, 10_001);
    assert!(
        message.contains("`region-expansions`"),
        "the diagnostic must name the exhausted budget: {message}",
    );
    assert!(
        message.contains("lower bound")
            && message.contains("not the budget this region needs in order to succeed"),
        "a search stop must not present its floor as a required size: {message}",
    );
    assert!(
        !message.contains("completed count") && !message.contains("may use less"),
        "a search stop must not borrow exact-demand or envelope wording: {message}",
    );
}
