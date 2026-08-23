use super::super::{
    AvailabilityPhase, Axis, BroadcastAxisMapping, CompilationRequest, DeterministicBudgets,
    Extent, F32, InputKey, LogicalAccess, NormalizedOutput, OutputKey,
    PARAMETRIC_BROADCAST_ACCESS_TAG, RequestError, SemanticProgram, Shape, SourcedExtent,
    StrictF32NumericalContract, VerifiedTargetRequest, encode_access_relation,
    verify_planned_request, verify_request,
};
use super::support::{packaged_program, planning_capability_rule, recognize};
use std::sync::Arc;
use tiler_ir::semantic::{
    BroadcastAxisSource, F32Add, F32Broadcast, F32Multiply, SemanticProgramBuilder,
};
use tiler_ir::shape::{
    BindingSource, ExtentRelation, ExtentTerm, FactProvenance, GuardApplicability, RootBinding,
    SemanticInputConstraint, ShapeEnv, ShapeEnvBuilder, ShapeSymbol, SymbolScope, VariantGuard,
};

fn request_symbol(name: &str) -> ShapeSymbol {
    ShapeSymbol::new(SymbolScope::new("program/0").unwrap(), name).unwrap()
}

fn request_axis_binding(input: &str, axis: u32) -> RootBinding {
    RootBinding::new(
        BindingSource::InputDimension {
            input: InputKey::new(input).unwrap(),
            axis: Axis::new(axis),
        },
        AvailabilityPhase::LiveDevicePreflight,
        FactProvenance::RuntimeValidated,
    )
    .unwrap()
}

fn request_environment(bound_to: Option<u64>) -> Arc<ShapeEnv> {
    request_environment_rooted("a", bound_to)
}

/// The fixture environment with `n` rooted at `input[0]`.
///
/// The root input is a parameter because the live source projection must
/// follow the environment's exact root rather than the first declared
/// input, and only a fixture that can move the root can watch that.
fn request_environment_rooted(input: &str, bound_to: Option<u64>) -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let declared = request_symbol("n");
    draft.declare(declared.clone()).unwrap();
    draft
        .bind(&declared, request_axis_binding(input, 0))
        .unwrap();
    if let Some(value) = bound_to {
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::equal(ExtentTerm::Symbol(declared), ExtentTerm::Constant(value)),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
    }
    Arc::new(draft.build().unwrap())
}

/// An environment whose `n` is rooted at an interface parameter, not an
/// input dimension — a valid authored program outside the admitted live
/// population.
fn interface_parameter_environment() -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let declared = request_symbol("n");
    draft.declare(declared.clone()).unwrap();
    draft
        .bind(
            &declared,
            RootBinding::new(
                BindingSource::InterfaceParameter {
                    key: tiler_ir::shape::InterfaceParameterKey::new("len").unwrap(),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    Arc::new(draft.build().unwrap())
}

/// `(a * b) + c` over three rank-one `f32` inputs of one sourced extent.
fn three_input_elementwise_with(
    environment: Option<Arc<ShapeEnv>>,
    extents: &[SourcedExtent],
) -> SemanticProgram {
    let mut builder = match environment {
        Some(environment) => {
            SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap()
        }
        None => SemanticProgramBuilder::try_standard().unwrap(),
    };
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input_sourced::<F32>(InputKey::new(key).unwrap(), extents.to_vec())
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let root = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

fn symbolic_three_input_elementwise(bound_to: Option<u64>) -> SemanticProgram {
    three_input_elementwise_with(
        Some(request_environment(bound_to)),
        &[SourcedExtent::Symbol(request_symbol("n"))],
    )
}

fn literal_three_input_elementwise(extent: u64) -> SemanticProgram {
    three_input_elementwise_with(None, &[SourcedExtent::Static(Extent::new(extent))])
}

fn first_symbolic_extent(program: &SemanticProgram) -> SourcedExtent {
    program
        .inputs()
        .next()
        .and_then(|input| program.shape(input.value()).ok())
        .and_then(|shape| shape.extents().find(|extent| extent.as_static().is_none()))
        .expect("the symbolic fixture names at least one symbol")
}

fn scheduled_symbolic_extent(error: &crate::pipeline::CompileError) -> Option<&SourcedExtent> {
    match error {
        crate::pipeline::CompileError::UnsupportedCapability(
            RequestError::UnsupportedSymbolicExtent {
                phase: "schedule",
                rule: "symbolic-extent",
                extent,
            },
        ) => Some(extent),
        crate::pipeline::CompileError::Explained { source, .. } => {
            scheduled_symbolic_extent(source)
        }
        _ => None,
    }
}

/// Same-shape elementwise is admitted through strategy with extents left symbolic.
///
/// Watched failing under a deliberate perturbation: restoring
/// `static_shape` in `recognize_elementwise_output` makes this program
/// refuse as `UnsupportedSymbolicExtent { phase: "strategy" }` before a
/// `NormalizedProgram` exists.
#[test]
fn a_symbolic_elementwise_program_is_recognized_with_its_symbols() {
    let program = symbolic_three_input_elementwise(None);
    let request = CompilationRequest::governed(&program);
    assert!(
        std::ptr::eq(
            request
                .shape_environment
                .expect("a symbolic program carries its environment")
                .environment(),
            program
                .extent_sources()
                .expect("the constructed program owns its environment")
                .environment(),
        ),
        "the request must carry the program's own environment, not a second one",
    );
    let verified = verify_planned_request(request)
        .expect("same-shape symbolic elementwise must pass strategy selection");
    assert_eq!(
        verified.normalized.first_symbolic_extent(),
        Some(SourcedExtent::Symbol(request_symbol("n"))),
    );
    let pointwise = verified
        .normalized
        .outputs()
        .first()
        .and_then(NormalizedOutput::pointwise)
        .expect("the fixture is whole-program elementwise");
    assert_eq!(pointwise.shape.as_static(), None);
    assert_eq!(
        pointwise.shape.extents().collect::<Vec<_>>(),
        vec![SourcedExtent::Symbol(request_symbol("n"))],
    );
}

/// The verified target request of one symbolic fixture, and its program.
fn symbolic_target(bound_to: Option<u64>) -> (SemanticProgram, VerifiedTargetRequest) {
    let program = symbolic_three_input_elementwise(bound_to);
    let target = verify_planned_request(CompilationRequest::governed(&program))
        .expect("the admitted symbolic population verifies")
        .for_target(0)
        .expect("one governed target");
    (program, target)
}

/// The canonical live region and members of one symbolic target request.
fn live_region_of(
    target: &VerifiedTargetRequest,
) -> (
    crate::physical::ScheduledRegion,
    Vec<crate::region::SemanticStage>,
) {
    let [output] = target.normalized().outputs() else {
        panic!("the fixture declares one output");
    };
    crate::physical::pointwise_region(target, output, crate::physical::RegionWrite::ProgramOutput)
}

/// A still-unsupported symbolic population names the extent at schedule;
/// the literal neighbour compiles.
///
/// The admitted rank-one population no longer reaches this refusal — its
/// own test below proves the decline moved to program assembly — so the
/// subjects here are the populations the accepted surface leaves refused:
/// a mixed-rank domain, a symbol rooted at an interface parameter, and a
/// root input the region never reads densely. Each is perturbed
/// independently of the parametric-broadcast exception, so a missing
/// broadcast is provably not the only way a symbol reaches a plan.
#[test]
fn unsupported_symbolic_populations_keep_the_named_schedule_refusal() {
    // Rank two with one symbolic axis: same-shape, recognized, refused.
    let mixed_rank = three_input_elementwise_with(
        Some(request_environment(None)),
        &[
            SourcedExtent::Symbol(request_symbol("n")),
            SourcedExtent::Static(Extent::new(4)),
        ],
    );
    let extent = first_symbolic_extent(&mixed_rank);
    match crate::pipeline::compile(CompilationRequest::governed(&mixed_rank)) {
        Err(error) => assert_eq!(
            scheduled_symbolic_extent(&error),
            Some(&extent),
            "a higher-rank symbolic domain must keep the schedule refusal, got {error}"
        ),
        Ok(_) => panic!("a higher-rank symbolic domain must keep the schedule refusal"),
    }

    // A non-input root: the environment roots `n` at an interface
    // parameter, which has no accepted runtime input-axis realization.
    let parameter_rooted = three_input_elementwise_with(
        Some(interface_parameter_environment()),
        &[SourcedExtent::Symbol(request_symbol("n"))],
    );
    let extent = first_symbolic_extent(&parameter_rooted);
    match crate::pipeline::compile(CompilationRequest::governed(&parameter_rooted)) {
        Err(error) => assert_eq!(
            scheduled_symbolic_extent(&error),
            Some(&extent),
            "a non-input-rooted symbol must keep the schedule refusal, got {error}"
        ),
        Ok(_) => panic!("a non-input-rooted symbol must keep the schedule refusal"),
    }

    // A root input the region never reads: `b + c` declares `a` and roots
    // `n` there, but no dense read realizes `a[0]`, so there is no access
    // for the source marker to sit on.
    let unread_root = {
        let mut builder =
            SemanticProgramBuilder::try_standard_with_shape_environment(request_environment(None))
                .unwrap();
        let inputs: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|key| {
                builder
                    .input_sourced::<F32>(
                        InputKey::new(key).unwrap(),
                        vec![SourcedExtent::Symbol(request_symbol("n"))],
                    )
                    .unwrap()
            })
            .collect();
        let root = F32Add::apply(&mut builder, inputs[1], inputs[2]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };
    let extent = first_symbolic_extent(&unread_root);
    match crate::pipeline::compile(CompilationRequest::governed(&unread_root)) {
        Err(error) => assert_eq!(
            scheduled_symbolic_extent(&error),
            Some(&extent),
            "an unread root input must keep the schedule refusal, got {error}"
        ),
        Ok(_) => panic!("an unread root input must keep the schedule refusal"),
    }

    let literal = literal_three_input_elementwise(4);
    crate::pipeline::compile(CompilationRequest::governed(&literal))
        .expect("the literal neighbour of the symbolic elementwise program still compiles");
}

/// A distinct proved-equal symbol does not widen the exact-shape population.
///
/// The environment declares `m`, roots it at `b[0]`, and proves `n == m`;
/// the recognizer still compares exact `SourcedShape`, so the program is
/// refused at strategy rather than admitted through the live schedule on a
/// solver fact.
#[test]
fn a_proved_equal_symbol_does_not_widen_the_admitted_population() {
    let environment = {
        let mut draft = ShapeEnvBuilder::new();
        let n = request_symbol("n");
        let m = request_symbol("m");
        draft.declare(n.clone()).unwrap();
        draft.declare(m.clone()).unwrap();
        draft.bind(&n, request_axis_binding("a", 0)).unwrap();
        draft.bind(&m, request_axis_binding("b", 0)).unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::equal(ExtentTerm::Symbol(n), ExtentTerm::Symbol(m)),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        Arc::new(draft.build().unwrap())
    };
    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let a = builder
        .input_sourced::<F32>(
            InputKey::new("a").unwrap(),
            vec![SourcedExtent::Symbol(request_symbol("n"))],
        )
        .unwrap();
    let b = builder
        .input_sourced::<F32>(
            InputKey::new("b").unwrap(),
            vec![SourcedExtent::Symbol(request_symbol("m"))],
        )
        .unwrap();
    let root = F32Add::apply(&mut builder, a, b).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let program = builder.build().unwrap();
    let refusal = verify_planned_request(CompilationRequest::governed(&program))
        .expect_err("a differently spelled proved-equal symbol must refuse at recognition");
    assert_eq!(
        refusal.to_string(),
        "compile.unsupported.strategy.elementwise-shape: no installed capability can \
         compile this valid semantic program",
        "neither spelling equality nor proves_equal may widen the exact-shape population"
    );
}

/// The admitted population forms the verified source-bound live schedule.
///
/// The accepted fieldless spelling, end to end at the physical layer: a
/// rank-zero static outer domain of one work item, the exact root-realizing
/// read carrying `LiveRowMajorSource` at the decoded `a[0]` root, every
/// other read and the final write the fieldless consumer, one derived
/// input-extent operand, checked request binding, feasibility, and a
/// lowered kernel consuming exactly that operand. The retained request
/// still names the authored `n`, and a binding that proves `n == 4`
/// changes none of the schedule bytes while the literal `[4]` neighbour's
/// differ — the specialization boundary held from both sides.
#[test]
fn the_admitted_symbolic_population_forms_a_verified_source_bound_live_schedule() {
    use tiler_ir::schedule::LogicalAccess;

    let (_, target) = symbolic_target(None);
    assert_eq!(
        target.normalized().first_symbolic_extent(),
        Some(SourcedExtent::Symbol(request_symbol("n"))),
        "the retained compiler request still names the authored symbol",
    );
    let root = crate::physical::decode_live_extent_root(
        target.semantic_identity().shape_environment().as_bytes(),
        &request_symbol("n"),
        tiler_ir::schedule::RegionId::new(0),
    )
    .expect("the retained identity bytes decode to the root");
    assert_eq!(root.input, InputKey::new("a").unwrap());
    assert_eq!(root.axis, Axis::new(0));

    let (region, members) = live_region_of(&target);
    assert_eq!(region.index.iteration_shape.rank(), 0, "empty static outer");
    assert_eq!(region.schedule.work_items, 1, "one static outer invocation");
    let maps: Vec<LogicalAccess> = region
        .index
        .accesses
        .iter()
        .map(|access| access.map.clone())
        .collect();
    assert_eq!(
        maps,
        vec![
            LogicalAccess::LiveRowMajorSource {
                inner_axis: Axis::new(0)
            },
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajor,
        ],
        "one source marker on the root read, fieldless consumers elsewhere, \
         the final write included",
    );

    let verified = crate::physical::verify_schedule_with_feasibility(
        region.clone(),
        members.clone(),
        &target,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the source-bound live schedule verifies and binds");
    assert_eq!(
        tiler_ir::schedule::live_input_extents(verified.region()),
        vec![(tiler_ir::schedule::AccessOrdinal::new(0), Axis::new(0))],
        "the marker is the region's one runtime extent operand",
    );
    let kernel = crate::physical::lower_structured_kernel(&verified)
        .expect("the live schedule lowers to a verified kernel");
    let operands: Vec<_> = kernel.input_extents().collect();
    assert_eq!(operands.len(), 1);
    assert_eq!(
        operands[0].access,
        tiler_ir::schedule::AccessOrdinal::new(0)
    );
    assert_eq!(operands[0].axis, Axis::new(0));

    // The bound-symbol neighbour: `n == 4` proved, schedule bytes exact.
    let (_, bound_target) = symbolic_target(Some(4));
    let (bound_region, bound_members) = live_region_of(&bound_target);
    let bound = crate::physical::verify_schedule_with_feasibility(
        bound_region,
        bound_members,
        &bound_target,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("a bound symbol still verifies as the symbol");
    assert_eq!(
        verified.canonical_identity().as_bytes(),
        bound.canonical_identity().as_bytes(),
        "a binding that proves n == 4 must not move the schedule bytes",
    );

    // The literal `[4]` neighbour is a different schedule.
    let literal = literal_three_input_elementwise(4);
    let literal_target = verify_planned_request(CompilationRequest::governed(&literal))
        .unwrap()
        .for_target(0)
        .unwrap();
    let (literal_region, literal_members) = live_region_of(&literal_target);
    let literal_verified = crate::physical::verify_schedule_with_feasibility(
        literal_region,
        literal_members,
        &literal_target,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the literal neighbour verifies");
    assert_ne!(
        verified.canonical_identity().as_bytes(),
        literal_verified.canonical_identity().as_bytes(),
        "the live schedule and the baked [4] schedule are different subjects",
    );
}

/// The source marker projects to the environment's root, never the first
/// input.
///
/// Rebinding `n` to `c[0]` with the access order unchanged moves the
/// marker to read position 2; positions 0 and 1 become fieldless
/// consumers.
#[test]
fn the_source_marker_follows_the_environment_root_not_the_first_input() {
    use tiler_ir::schedule::LogicalAccess;

    let program = three_input_elementwise_with(
        Some(request_environment_rooted("c", None)),
        &[SourcedExtent::Symbol(request_symbol("n"))],
    );
    let target = verify_planned_request(CompilationRequest::governed(&program))
        .unwrap()
        .for_target(0)
        .unwrap();
    let (region, members) = live_region_of(&target);
    let maps: Vec<LogicalAccess> = region
        .index
        .accesses
        .iter()
        .map(|access| access.map.clone())
        .collect();
    assert_eq!(
        maps,
        vec![
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajorSource {
                inner_axis: Axis::new(0)
            },
            LogicalAccess::LiveRowMajor,
        ],
        "the marker moved to access 2 with the root, not stayed first",
    );
    crate::physical::verify_schedule_with_feasibility(
        region,
        members,
        &target,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the c-rooted live schedule verifies and binds");
}

/// A hand-built region whose marker sits on the wrong access is refused as
/// `request-binding`: intrinsic verification cannot see the semantic root,
/// and the compiler binding proves it independently.
#[test]
fn a_forged_source_marker_position_fails_request_binding() {
    use tiler_ir::schedule::LogicalAccess;

    let (_, target) = symbolic_target(None);
    let (mut region, members) = live_region_of(&target);
    // The fixture's root is `a[0]`, access 0. Nominate `b` instead.
    region.index.accesses[0].map = LogicalAccess::LiveRowMajor;
    region.index.accesses[1].map = LogicalAccess::LiveRowMajorSource {
        inner_axis: Axis::new(0),
    };
    let refusal = crate::physical::verify_schedule_with_feasibility(
        region,
        members,
        &target,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect_err("a marker off the decoded root must not bind");
    assert_eq!(
        refusal.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
        "equal runtime values cannot replace the exact a[0] authority",
    );
}

/// A launch minted over the determined representative extent cannot bind
/// the symbolic subject: plan specialization stays forbidden and its
/// refusal stays reachable.
#[test]
fn a_specialized_representative_launch_fails_request_binding() {
    let (_, target) = symbolic_target(Some(4));
    // The literal `[4]` region a folding formation step would mint.
    let literal = literal_three_input_elementwise(4);
    let literal_target = verify_planned_request(CompilationRequest::governed(&literal))
        .unwrap()
        .for_target(0)
        .unwrap();
    let (specialized, members) = live_region_of(&literal_target);
    assert_eq!(
        specialized.schedule.work_items, 4,
        "the specialized region launches over the bound value",
    );
    let refusal = crate::physical::verify_schedule_with_feasibility(
        specialized,
        members,
        &target,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect_err("a [4] launch must not bind the symbolic subject");
    assert_eq!(
        refusal.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
        "ExtentSources::determined never supplies schedule geometry",
    );
}

/// Truncated and bad-domain identity-subject bytes fail as the existing
/// compiler `request-binding` through the production decode mapping.
#[test]
fn corrupted_identity_subject_bytes_fail_as_request_binding() {
    let (_, target) = symbolic_target(None);
    let bytes = target
        .semantic_identity()
        .shape_environment()
        .as_bytes()
        .to_vec();
    let region = tiler_ir::schedule::RegionId::new(0);

    let truncated = crate::physical::decode_live_extent_root(
        &bytes[..bytes.len() - 1],
        &request_symbol("n"),
        region,
    )
    .expect_err("truncated identity bytes must not decode");
    assert_eq!(
        truncated.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
        "a truncated subject is the existing compiler request-binding refusal",
    );

    let mut bad_domain = bytes.clone();
    // The domain separator is length-framed at the front; flipping a byte
    // inside it is a subject from another domain.
    bad_domain[8] ^= 0xff;
    let bad = crate::physical::decode_live_extent_root(&bad_domain, &request_symbol("n"), region)
        .expect_err("bad-domain identity bytes must not decode");
    assert_eq!(
        bad.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
        "a bad-domain subject is the existing compiler request-binding refusal",
    );

    // An absent symbol on well-formed bytes is the same fail-closed rule:
    // no arm defaults an environment or selects another binding.
    let absent = crate::physical::decode_live_extent_root(
        &bytes,
        &ShapeSymbol::new(SymbolScope::new("program/0").unwrap(), "absent").unwrap(),
        region,
    )
    .expect_err("an undeclared symbol must not resolve a root");
    assert_eq!(
        absent.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
    );
}

/// A bound symbol is not folded into the compiled product.
///
/// The environment pins `n` to 4. The program still names the symbol, the
/// request still carries that environment, and compilation forms the live
/// schedule as the symbol — never a `[4]` plan.
///
/// **The value-never-enters-identity assertions below are unchanged; the wall
/// assertion beside them is what moved.** This used to close by requiring
/// `compile()` to decline at `program-assembly.named-output-symbolic`, which
/// stood only because packaging could not represent the shape-environment
/// subject. `tiler.kernel-program.v13` folds it, so the population packages and
/// the claim strengthens from "declines rather than compiling as 4" to the
/// directly testable "compiles, and what it compiles is not the `[4]` program".
/// The exact schedule-identity claim (bound and unbound schedule bytes equal,
/// literal `[4]` bytes different) remains
/// `the_admitted_symbolic_population_forms_a_verified_source_bound_live_schedule`'s.
#[test]
fn a_compiled_plan_does_not_fold_a_bound_extent_value() {
    let bound = symbolic_three_input_elementwise(Some(4));
    let extent = first_symbolic_extent(&bound);
    assert_eq!(
        extent,
        SourcedExtent::Symbol(request_symbol("n")),
        "a constraint that pins n to 4 must not rewrite the authored shape",
    );
    for value in bound.values() {
        assert_eq!(
            value.shape().as_static(),
            None,
            "no authored boundary may collapse to the bound value",
        );
    }
    let verified = verify_planned_request(CompilationRequest::governed(&bound))
        .expect("a bound symbol is still a recognized symbolic program");
    assert_eq!(
        verified.normalized.first_symbolic_extent(),
        Some(extent.clone()),
        "recognition must keep the authored symbol, not the bound value 4",
    );
    let compiled = crate::pipeline::compile(CompilationRequest::governed(&bound))
        .expect("a bound symbol packages as the symbol");
    let literal = literal_three_input_elementwise(4);
    let literal_compiled = crate::pipeline::compile(CompilationRequest::governed(&literal))
        .expect("the literal [4] neighbour still compiles");
    assert_ne!(
        packaged_program_identity(&compiled),
        packaged_program_identity(&literal_compiled),
        "a program that names n, even with n proved equal to 4, is not the [4] program",
    );

    // The packaged boundary keeps the symbol rather than the proved value: the
    // covered boundary is the zero-extent convention, and 4 appears nowhere in
    // it. A packaging step that folded the bound value would size this at 4.
    let packaged = packaged_program(&compiled);
    for value in packaged.core().values() {
        assert!(
            value
                .shape()
                .extents()
                .iter()
                .all(|extent| extent.get() != 4),
            "no packaged value may be sized by the bound extent value",
        );
    }

    // Where the live quantity *is* carried: as an `InputExtent` root over the
    // environment's own decoded root, resolved at live preflight from the
    // caller's buffer. This is the positive half of the assertion above — a
    // packaging step that folded the bound value would have no reason to
    // declare this root, and one that dropped the quantity entirely would leave
    // the accessible ranges sized by nothing.
    let rooted = packaged.core().abi_expressions().iter().any(|node| {
        matches!(
            node,
            tiler_ir::program::abi::ExprNode::Root(tiler_ir::program::abi::AbiRoot::InputExtent {
                key,
                axis,
            }) if key.as_str() == "a" && axis.get() == 0,
        )
    });
    assert!(
        rooted,
        "the live extent must be carried as a root over the environment's decoded root",
    );
}

/// The canonical kernel-program identity bytes one compiled target packaged.
fn packaged_program_identity(compiled: &crate::pipeline::CompilationProduct) -> Vec<u8> {
    packaged_program(compiled)
        .core()
        .canonical_identity()
        .as_bytes()
        .to_vec()
}

/// The admitted symbolic population passes schedule formation *and* packaging,
/// and its packaged boundary keeps the symbol.
///
/// **This replaces
/// `the_admitted_symbolic_population_declines_at_program_assembly_not_schedule`.**
/// Two walls fell in turn for this population and the retired names of both are
/// recorded here so a later reader can tell which one a regression restored:
/// the schedule-geometry refuse `UnsupportedSymbolicExtent { phase: "schedule",
/// rule: "symbolic-extent" }` went when the source-bound live schedule landed,
/// and the packaging refuse `program-assembly.named-output-symbolic` went at
/// `tiler.kernel-program.v13`, which folds the shape-environment subject so a
/// symbolic program's identity is complete rather than under-keyed.
///
/// Watched failing under two independent deliberate perturbations, each showing
/// a different assertion below is load-bearing. Restoring the unconditional
/// schedule gate makes this fail at the `scheduled_symbolic_extent` assertion
/// with `compile.schedule.symbolic-extent: program/0::n is a symbolic extent
/// this capability cannot plan over`. Restoring `named-output-symbolic` as an
/// unconditional refusal in `CoverAssembly::from_plan` makes it fail at the
/// `compile` call with `compile.unsupported.program-assembly.named-output-symbolic:
/// no installed capability can compile this valid semantic program`.
#[test]
fn the_admitted_symbolic_population_packages_a_verified_kernel_program() {
    let symbolic = symbolic_three_input_elementwise(None);
    crate::region::RegionGraph::from_program(&symbolic)
        .expect("region-graph construction must record a sourced boundary");
    crate::region::form_region_candidates(
        &symbolic,
        crate::request::DeterministicBudgets::governed(),
        crate::request::StrictF32NumericalContract::governed(),
    )
    .expect("region formation must accept the admitted symbolic population");

    let compiled = match crate::pipeline::compile(CompilationRequest::governed(&symbolic)) {
        Ok(compiled) => compiled,
        Err(error) => {
            assert_eq!(
                scheduled_symbolic_extent(&error),
                None,
                "the admitted population must pass the schedule gate, got {error}"
            );
            panic!("the admitted population must package, got {error}");
        }
    };

    // The packaged program's identity folds the program's own environment
    // subject beside its graph. Read rather than asserted by construction: a
    // fold that dropped the subject leaves this needle absent. The domain step
    // itself is pinned where the domain is declared — `tiler_ir`'s
    // `the_program_domain_separator_is_what_distinguishes_the_reinterpreting_steps`
    // and its `PINNED_IDENTITY_DOMAINS` row — rather than restated here, because
    // this crate's own pin census admits only the domains it declares.
    let identity = packaged_program_identity(&compiled);
    let subject = symbolic
        .semantic_identity()
        .shape_environment()
        .as_bytes()
        .to_vec();
    assert!(
        identity
            .windows(subject.len())
            .any(|window| window == subject.as_slice()),
        "the packaged identity must carry the program's own environment subject",
    );

    let literal = literal_three_input_elementwise(4);
    crate::pipeline::compile(CompilationRequest::governed(&literal))
        .expect("the literal neighbour still compiles");
}

/// The packaging population is exactly the admitted one, counted rather than
/// argued.
///
/// **The lift is condition-shaped, not population-shaped**, so the check that
/// matters is that making representation total did not widen *what compiles*.
/// The census walks every symbolic fixture this module can author and requires
/// each to compile exactly when `admits_source_bound_live_schedule` says the
/// request is admitted — including the parametric-broadcast carrier, which the
/// schedule gate lets past on its own separate arm and which must therefore
/// still decline at physical selection rather than falling into packaging.
///
/// The population is printed rather than trusted, and a floor is asserted, so a
/// fixture list that silently stopped covering its subject cannot look green.
#[test]
fn the_packaging_population_is_exactly_the_admitted_population() {
    let cases: Vec<(&str, SemanticProgram)> = vec![
        ("admitted-unbound", symbolic_three_input_elementwise(None)),
        (
            "admitted-bound-4",
            symbolic_three_input_elementwise(Some(4)),
        ),
        (
            "root-at-b",
            three_input_elementwise_with(
                Some(request_environment_rooted("b", None)),
                &[SourcedExtent::Symbol(request_symbol("n"))],
            ),
        ),
        (
            "interface-parameter-root",
            three_input_elementwise_with(
                Some(interface_parameter_environment()),
                &[SourcedExtent::Symbol(request_symbol("n"))],
            ),
        ),
        (
            "unread-root-input",
            three_input_elementwise_with(
                Some(request_environment_rooted("missing", None)),
                &[SourcedExtent::Symbol(request_symbol("n"))],
            ),
        ),
        (
            "parametric-broadcast",
            parametric_broadcast_only_program(
                parametric_broadcast_environment("n", (1, 32_768), None),
                "n",
            ),
        ),
    ];
    assert!(
        cases.len() >= 6,
        "the census must keep every symbolic arm the accepted surface names",
    );

    let mut admitted = 0_usize;
    let mut packaged = 0_usize;
    for (label, program) in &cases {
        let target = verify_planned_request(CompilationRequest::governed(program))
            .ok()
            .and_then(|verified| verified.for_target(0).ok());
        let admits = target
            .as_ref()
            .is_some_and(crate::physical::admits_source_bound_live_schedule);
        let compiles = crate::pipeline::compile(CompilationRequest::governed(program)).is_ok();
        println!("packaging census: {label}: admitted={admits} packaged={compiles}");
        admitted += usize::from(admits);
        packaged += usize::from(compiles);
        assert_eq!(
            admits, compiles,
            "{label}: the packaging population must equal the admitted population",
        );
    }
    // Three admitted: the unbound fixture, its constraint-bound neighbour, and
    // the one whose root moved to `b` — a root the region still reads densely.
    // Three refused: a root that is not an input dimension at all, a root input
    // the region never reads, and the parametric-broadcast carrier, which the
    // schedule gate lets past on its own arm and which must therefore decline at
    // physical selection rather than falling into packaging.
    assert_eq!(
        (admitted, packaged),
        (3, 3),
        "the census must exercise both answers, not one of them six times",
    );
}

/// Dropping the program's environment is a pairing refusal, not a schema one.
#[test]
fn dropping_the_program_environment_is_a_pairing_refusal() {
    let program = symbolic_three_input_elementwise(None);
    let mut request = CompilationRequest::governed(&program);
    request.shape_environment = None;
    match verify_request(request) {
        Err(RequestError::MismatchedShapeEnvironment) => {}
        Ok(_) => panic!("dropping the environment must refuse, got a verified request"),
        Err(error) => panic!("dropping the environment must be a pairing refusal, got {error}"),
    }
    assert_eq!(
        RequestError::MismatchedShapeEnvironment.to_string(),
        "compile.request.shape-environment: request must carry the program's own environment",
    );
}

fn parametric_broadcast_environment(
    symbol: &str,
    interval: (u64, u64),
    guard: Option<u64>,
) -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let declared = request_symbol(symbol);
    draft.declare(declared.clone()).unwrap();
    draft.bind(&declared, request_axis_binding("a", 0)).unwrap();
    draft
        .require(SemanticInputConstraint::new(
            ExtentRelation::interval(ExtentTerm::Symbol(declared.clone()), interval.0, interval.1)
                .unwrap(),
            FactProvenance::FrontendRequired,
        ))
        .unwrap();
    if let Some(value) = guard {
        draft
            .guard(VariantGuard::new(
                ExtentRelation::equal(ExtentTerm::Symbol(declared), ExtentTerm::Constant(value)),
                GuardApplicability::Schedule,
            ))
            .unwrap();
    }
    Arc::new(draft.build().unwrap())
}

/// `a * broadcast(w)` over `a: f32[n, 4]` and `w: f32[4]`.
fn parametric_broadcast_program(
    environment: Arc<ShapeEnv>,
    pad: &str,
) -> (SemanticProgram, BroadcastAxisMapping) {
    let pad_symbol = request_symbol(pad);
    let mapping = BroadcastAxisMapping::new(
        [
            SourcedExtent::Symbol(pad_symbol),
            SourcedExtent::Static(Extent::new(4)),
        ],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .expect("a symbolic rank-pad mapping is context-free");
    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let activation = builder
        .input_sourced::<F32>(
            InputKey::new("a").unwrap(),
            vec![
                SourcedExtent::Symbol(request_symbol(pad)),
                SourcedExtent::Static(Extent::new(4)),
            ],
        )
        .unwrap();
    let weight = builder
        .input_sourced::<F32>(
            InputKey::new("w").unwrap(),
            vec![SourcedExtent::Static(Extent::new(4))],
        )
        .unwrap();
    let widened = F32Broadcast::apply(&mut builder, &mapping, weight)
        .expect("the sourced mapping applies against the program's environment");
    let root = F32Multiply::apply(&mut builder, activation, widened).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    (builder.build().unwrap(), mapping)
}

/// A single sourced broadcast, so lowering has only the parametric
/// occurrence to refine. The fused `a * broadcast(w)` neighbour still
/// exercises recognition; its multiply keeps a static index law.
fn parametric_broadcast_only_program(environment: Arc<ShapeEnv>, pad: &str) -> SemanticProgram {
    let mapping = BroadcastAxisMapping::new(
        [
            SourcedExtent::Symbol(request_symbol(pad)),
            SourcedExtent::Static(Extent::new(4)),
        ],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .expect("a symbolic rank-pad mapping is context-free");
    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let weight = builder
        .input_sourced::<F32>(
            InputKey::new("w").unwrap(),
            vec![SourcedExtent::Static(Extent::new(4))],
        )
        .unwrap();
    let widened = F32Broadcast::apply(&mut builder, &mapping, weight)
        .expect("the sourced mapping applies against the program's environment");
    builder
        .output(OutputKey::new("result").unwrap(), widened)
        .unwrap();
    builder.build().unwrap()
}

fn recognized_parametric_read(program: &SemanticProgram) -> LogicalAccess {
    let verified = verify_planned_request(CompilationRequest::governed(program))
        .expect("a sourced broadcast must pass strategy selection");
    let pointwise = verified
        .normalized
        .outputs()
        .first()
        .and_then(NormalizedOutput::pointwise)
        .expect("the fixture is whole-program elementwise");
    pointwise
        .reads
        .iter()
        .map(|(_, map)| map.clone())
        .find(|map| matches!(map, LogicalAccess::ParametricBroadcast { .. }))
        .expect("recognition must retain the parametric carrier")
}

fn request_subject_bytes(program: &SemanticProgram) -> Vec<u8> {
    verify_planned_request(CompilationRequest::governed(program))
        .expect("the fixture admits a planned request")
        .for_target(0)
        .expect("the governed profile admits the fixture")
        .subject()
        .canonical_explain_subject_bytes()
}

/// One symbolic broadcast program reaches selection with its mapping and
/// environment unchanged.
///
/// Watched failing under a deliberate perturbation: restoring the static
/// domain gate in `plan_elementwise` refuses this program as
/// `UnsupportedSymbolicExtent { phase: "strategy" }` before a
/// `NormalizedProgram` exists.
#[test]
fn a_parametric_broadcast_program_is_recognized_with_its_carrier() {
    let environment = parametric_broadcast_environment("n", (1, 32_768), None);
    let identity = environment.identity().clone();
    let (program, mapping) = parametric_broadcast_program(environment, "n");
    let request = CompilationRequest::governed(&program);
    assert!(
        std::ptr::eq(
            request
                .shape_environment
                .expect("a symbolic program carries its environment")
                .environment(),
            program
                .extent_sources()
                .expect("the constructed program owns its environment")
                .environment(),
        ),
        "the request must carry the program's own environment, not a second one",
    );
    let verified =
        verify_planned_request(request).expect("a sourced broadcast must pass strategy selection");
    assert!(verified.normalized.carries_parametric_broadcast());
    let LogicalAccess::ParametricBroadcast {
        operand_shape,
        mapping: retained,
        environment: named,
    } = recognized_parametric_read(&program)
    else {
        panic!("recognition must retain ParametricBroadcast, not a concrete neighbour");
    };
    assert_eq!(
        operand_shape.extents().collect::<Vec<_>>(),
        vec![SourcedExtent::Static(Extent::new(4))],
    );
    assert_eq!(retained, mapping);
    assert_eq!(named, identity);
    assert_eq!(
        verified.normalized.first_symbolic_extent(),
        Some(SourcedExtent::Symbol(request_symbol("n"))),
    );
}

/// Perturbing a bound value does not change semantic, normalized-program, or
/// request identity.
///
/// The two programs share declarations, root bindings, and the positivity
/// interval. They differ only in a schedule variant guard pinning `n` to 4
/// or 10. Guards are outside `ShapeEnvIdentity`, so a compiler that folded
/// the pin into `BroadcastReplication` would be the thing that moved the
/// identities.
#[test]
fn a_bound_value_change_does_not_move_parametric_broadcast_identity() {
    let four = parametric_broadcast_program(
        parametric_broadcast_environment("n", (1, 32_768), Some(4)),
        "n",
    )
    .0;
    let ten = parametric_broadcast_program(
        parametric_broadcast_environment("n", (1, 32_768), Some(10)),
        "n",
    )
    .0;
    assert_eq!(four.semantic_identity(), ten.semantic_identity());
    assert_eq!(
        recognized_parametric_read(&four),
        recognized_parametric_read(&ten),
        "recognition must keep the same carrier; a fold to BroadcastReplication would move",
    );
    assert_eq!(
        request_subject_bytes(&four),
        request_subject_bytes(&ten),
        "request identity must not move with a bound value the environment does not author",
    );
    for program in [&four, &ten] {
        let LogicalAccess::ParametricBroadcast { operand_shape, .. } =
            recognized_parametric_read(program)
        else {
            panic!("a bound value must not fold the carrier into a concrete neighbour");
        };
        assert_eq!(operand_shape.as_static(), Some(&Shape::from_dims([4])));
        assert_eq!(
            program.extent_sources().and_then(
                |sources| sources.determined(&SourcedExtent::Symbol(request_symbol("n")))
            ),
            None,
            "a variant guard must not determine the authored symbol",
        );
    }
}

/// A provider lacking parametric support declines by the named capability
/// rule, not a static-signature or generic unsupported mask.
///
/// Watched failing under a deliberate perturbation: leaving the generic
/// symbolic-extent schedule refuse in front of physical selection reports
/// `phase: "schedule", rule: "symbolic-extent"` instead of the provider's
/// `parametric-broadcast` rule.
#[test]
fn a_provider_lacking_parametric_support_declines_by_named_rule() {
    let program = parametric_broadcast_only_program(
        parametric_broadcast_environment("n", (1, 32_768), None),
        "n",
    );
    crate::region::RegionGraph::from_program(&program)
        .expect("region-graph construction must record a sourced broadcast");
    crate::region::form_region_candidates(
        &program,
        DeterministicBudgets::governed(),
        StrictF32NumericalContract::governed(),
    )
    .expect("region formation must accept the parametric population");
    match crate::pipeline::compile(CompilationRequest::governed(&program)) {
        Err(error) => {
            assert_eq!(
                planning_capability_rule(&error),
                Some(("planning", "parametric-broadcast")),
                "a provider without parametric support must decline that named rule, got {error}"
            );
        }
        Ok(_) => panic!("a provider without parametric support must decline, got a product"),
    }

    let literal = literal_three_input_elementwise(4);
    crate::pipeline::compile(CompilationRequest::governed(&literal))
        .expect("the literal neighbour still compiles");
}

/// Two parametric mappings that differ in one pad symbol produce different
/// request-subject bytes. Concrete reindex and broadcast keep tags `0x01`
/// and `0x02`.
///
/// Watched failing under a deliberate perturbation: writing the parametric
/// carrier as `0x02` makes the two encodings share a tag with
/// `BroadcastReplication`.
#[test]
fn parametric_broadcast_request_subject_tag_is_injective() {
    let n_program = parametric_broadcast_program(
        parametric_broadcast_environment("n", (1, 32_768), None),
        "n",
    )
    .0;
    let t_env = {
        let mut draft = ShapeEnvBuilder::new();
        let declared = request_symbol("t");
        draft.declare(declared.clone()).unwrap();
        draft.bind(&declared, request_axis_binding("a", 0)).unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(declared), 1, 32_768).unwrap(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        Arc::new(draft.build().unwrap())
    };
    let t_program = parametric_broadcast_program(t_env, "t").0;
    assert_ne!(
        request_subject_bytes(&n_program),
        request_subject_bytes(&t_program),
        "two pad symbols must not share request-subject bytes",
    );

    let mut parametric_bytes = Vec::new();
    let parametric = recognized_parametric_read(&n_program);
    encode_access_relation(&mut parametric_bytes, &parametric);
    assert_eq!(
        parametric_bytes.first().copied(),
        Some(PARAMETRIC_BROADCAST_ACCESS_TAG),
        "the parametric carrier must take tag 0x05, not the refusal 0x00",
    );

    let concrete = BroadcastAxisMapping::new(
        [Extent::new(2), Extent::new(2)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .unwrap();
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), Shape::from_dims([2]))
        .unwrap();
    let widened = F32Broadcast::apply(&mut builder, &concrete, weight).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), widened)
        .unwrap();
    let concrete_program = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&concrete_program).expect("a literal broadcast is still BroadcastReplication")
    else {
        panic!("a literal broadcast is an elementwise region");
    };
    let (_, LogicalAccess::BroadcastReplication { .. }) = &recognized.reads[0] else {
        panic!("a wholly literal mapping must stay BroadcastReplication");
    };
    let mut concrete_bytes = Vec::new();
    encode_access_relation(&mut concrete_bytes, &recognized.reads[0].1);
    assert_eq!(concrete_bytes.first().copied(), Some(0x02));
    assert_ne!(
        parametric_bytes.first(),
        concrete_bytes.first(),
        "colliding the parametric tag with BroadcastReplication loses injectivity",
    );
}
