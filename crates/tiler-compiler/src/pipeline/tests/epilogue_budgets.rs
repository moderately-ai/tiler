use super::support::retained_stage_counts;
use super::*;

/// `sum(x * x, axis 1) * k` published as one ordered named output.
///
/// The widest producer chain the recognizer can spell for one output: a prologue
/// staging `x * x`, a split fold's partial and final passes, and an elementwise
/// epilogue reading the fold's staged result. Used below both alone and beside a
/// second copy, which is what makes a two-output program out of two
/// independently widest ones.
fn epilogue_chain_output(
    builder: &mut SemanticProgramBuilder,
    key: &str,
    output: &str,
    scale_bits: u32,
    columns: u64,
) {
    let x = builder
        .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([1, columns]))
        .unwrap();
    let squared = F32Multiply::apply(builder, x, x).unwrap();
    let reduced = StrictSerialF32Sum::apply(builder, squared, [Axis::new(1)]).unwrap();
    let scale = F32Constant::apply(builder, scale_bits).unwrap();
    let scaled = F32Multiply::apply(builder, reduced, scale).unwrap();
    builder
        .output(OutputKey::new(output).unwrap(), scaled)
        .unwrap();
}

/// One epilogue chain, published as the program's only output.
fn one_chain_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    epilogue_chain_output(&mut builder, "x", "scaled", 2.0_f32.to_bits(), 4);
    builder.build().unwrap()
}

/// Two independent epilogue chains, published as two ordered named outputs.
///
/// The two fold different declared inputs at different extents, and the extents
/// are what keep them distinct rather than a stylistic choice: two chains of
/// identical shape assemble two stages carrying one canonical key, which the
/// shared program layer refuses under `AmbiguousCanonicalKey` — a caller's valid
/// program reported as invalid compiler output. That is a separate defect, filed
/// as
/// `refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output`;
/// this fixture stays clear of it so it measures the budget and nothing else.
///
/// The second chain folds two columns rather than four, so its fold has no split
/// to offer and its chain is three dispatches against the first's four. Seven is
/// therefore this program's widest assembled plan against a derived bound of
/// eight, because the derivation is an upper bound over every plan the request
/// could reach rather than each plan's exact count.
fn two_chain_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    epilogue_chain_output(&mut builder, "x", "scaled", 2.0_f32.to_bits(), 4);
    epilogue_chain_output(&mut builder, "y", "halved", 3.0_f32.to_bits(), 2);
    builder.build().unwrap()
}

/// The widest chain one declared output reaches is four dispatches, and the
/// `regions` budget derives its actual as exactly that number per output.
///
/// **This is the measurement the `regions` budget's derivation rests on**, and
/// it moved with the epilogue admission. Before it, the widest assembled chain
/// was the split reduction's three stages — prologue, partial, final — and
/// `check_program_budgets` spelled that three as a literal. A fold that stages
/// its result can now feed an epilogue, so the same split plus its consumer is
/// four.
///
/// **The literal is gone and the four survives inside a derivation**, because a
/// plan covers every declared output: the constant was a bound on the whole plan
/// only while recognition could name one output, and since multi-output
/// admission it bounds one output's chain. So the budget is a bound on the
/// caller's declaration after all, which
/// `the_two_chain_program_is_refused_by_regions_until_the_budget_admits_both`
/// drives from the other side.
///
/// The reassociating contract is what admits the split, so the request states
/// it explicitly. Under a contract forbidding reassociation the same program
/// retains only the three-stage chain, which is the neighbour below.
#[test]
fn the_widest_assembled_plan_is_the_split_reduction_with_its_epilogue() {
    let semantic = one_chain_program();

    assert_eq!(
        retained_stage_counts(
            &semantic,
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ),
        vec![3, 4],
        "a reassociating contract retains the unsplit chain and the split one",
    );
    // The neighbour, so the four above is attributable to the split rather than
    // to the epilogue alone: forbidding reassociation withdraws the split and
    // exactly the four-stage alternative disappears.
    assert_eq!(
        retained_stage_counts(
            &semantic,
            crate::request::StrictF32NumericalContract::governed(),
        ),
        vec![3],
    );
    assert_eq!(
        crate::request::DeterministicBudgets::governed().regions,
        12,
        "the governed budget is four dispatches for each of the layer's three outputs",
    );

    // The refusing direction, and the verdict this one output had before
    // `regions` became a derivation: `4 × 1` is exactly the literal it replaced,
    // so a bound of three refuses this program now as it did then. Driving both
    // directions is what separates a derivation from a removed check.
    let mut narrow = CompilationRequest::governed_preferring(
        &semantic,
        crate::request::NumericalContractPreference::ordered(vec![
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ])
        .unwrap(),
    );
    narrow.budgets.regions = 3;
    assert_eq!(
        crate::request::verify_request(narrow).err(),
        Some(crate::request::RequestError::BudgetExceeded {
            resource: BudgetResource::Regions,
            limit: 3,
            reported: 4,
        }),
    );
}

/// The widest assembled plan binds four buffers for each declared output, and
/// the `buffers` budget derives its actual as exactly that.
///
/// **This is the measurement the per-output four rests on, and it corrects an
/// under-report that predates multi-output.** The derivation enumerated the
/// prologue's materialized temporary, a split's staged partial tensor, and the
/// output — three — and stopped there, missing the fold's staged result an
/// elementwise epilogue reads across. One declared input under the widest chain
/// assembles five values against a derived four, which is a boundary admitting a
/// request assembly then refuses: exactly the failure the derivation exists to
/// prevent.
///
/// Both programs are asserted, because one output cannot separate "four per
/// output" from "four plus a program-scoped constant". The two-output program's
/// widest plan binds nine values over two declared inputs, which only the
/// per-output reading predicts.
///
/// **Each measurement is tied to the derivation rather than left beside it**, by
/// stating a budget one below the measured count and requiring the *boundary* to
/// report. A derivation that under-reports admits that request and lets assembly
/// refuse it, so the assertion fails the moment the per-output term is wrong —
/// which a pair of bare `assert_eq!`s on the measured counts would not do.
#[test]
fn the_widest_assembled_plan_binds_four_buffers_per_declared_output() {
    let reassociating = || {
        crate::request::NumericalContractPreference::ordered(vec![
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ])
        .unwrap()
    };
    let widest_value_count = |semantic: &SemanticProgram| {
        let product = compile(CompilationRequest::governed_preferring(
            semantic,
            reassociating(),
        ))
        .expect("the chain compiles");
        product.targets[0]
            .portfolio
            .alternatives
            .iter()
            .map(|alternative| alternative.program.core().values().len())
            .max()
            .expect("at least one alternative is retained")
    };
    // A budget one below the widest plan's own value count must be refused by
    // the *request boundary*, which is only true when the derived actual covers
    // that plan.
    let refuses_one_below = |semantic: &SemanticProgram, widest: usize| {
        let mut narrow = CompilationRequest::governed_preferring(semantic, reassociating());
        let limit = u32::try_from(widest).unwrap() - 1;
        narrow.budgets.buffers = limit;
        assert_eq!(
            crate::request::verify_request(narrow).err(),
            Some(crate::request::RequestError::BudgetExceeded {
                resource: BudgetResource::Buffers,
                limit: u64::from(limit),
                reported: u64::try_from(semantic.input_count() + 4 * semantic.output_count())
                    .unwrap(),
            }),
            "the boundary must refuse a budget the widest plan exceeds",
        );
    };

    let one = one_chain_program();
    assert_eq!(one.input_count(), 1);
    assert_eq!(one.output_count(), 1);
    let widest = widest_value_count(&one);
    assert_eq!(widest, 5, "one input plus four per output");
    refuses_one_below(&one, widest);

    let two = two_chain_program();
    assert_eq!(two.input_count(), 2);
    assert_eq!(two.output_count(), 2);
    let widest = widest_value_count(&two);
    assert_eq!(
        widest, 9,
        "two inputs, four buffers for the split chain and three for the unsplit one",
    );
    refuses_one_below(&two, widest);

    // The derivation is the bound those two sit under, and the governed value is
    // that derivation over the eighteen-input, three-output decoder layer.
    assert_eq!(crate::request::DeterministicBudgets::governed().buffers, 30);
}

/// A two-chain program is refused by name under the pre-widening `regions`
/// budget and compiles once the governed value admits both chains.
///
/// **This is the failure the derivation exists to prevent, driven from both
/// sides.** Under the literal four the boundary admitted this program — a region
/// count was not a function of the declaration — and assembly then had to refuse
/// a seven-dispatch plan it had promised to build. The refusal is now at
/// `verify_request`, by name, before any plan is chosen.
///
/// Four is deliberately the *pre-widening* value rather than an arbitrary narrow
/// one: it is exactly the bound that was correct for one output and silently
/// wrong for two, so the assertion is about the change and not about budgets in
/// general.
#[test]
fn the_two_chain_program_is_refused_by_regions_until_the_budget_admits_both() {
    let semantic = two_chain_program();
    assert_eq!(semantic.output_count(), 2);

    let mut narrow = CompilationRequest::governed_preferring(
        &semantic,
        crate::request::NumericalContractPreference::ordered(vec![
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ])
        .unwrap(),
    );
    narrow.budgets.regions = 4;
    assert_eq!(
        crate::request::verify_request(narrow).err(),
        Some(crate::request::RequestError::BudgetExceeded {
            resource: BudgetResource::Regions,
            limit: 4,
            reported: 8,
        }),
        "the pre-widening bound must refuse two widest chains by name",
    );

    // And the governed value admits it, so the refusal above is attributable to
    // the bound rather than to anything else about the program.
    assert_eq!(
        retained_stage_counts(
            &semantic,
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ),
        vec![6, 7],
    );
}

/// A two-output program exceeding a stated `buffers` budget is refused at
/// `verify_request`, and never reaches `verify_host_contract`.
///
/// **Which stage refuses is the whole assertion.** Both stages check the same
/// resource, and only one of them can report a caller's request as a caller's
/// request: `verify_request` returns [`RequestError::BudgetExceeded`] naming the
/// resource, its limit, and the reported value, while `verify_host_contract` returns
/// `ProgramError::Storage { rule: "buffer-budget" }`, which reaches the caller
/// as `InvalidCompilerOutput` — the compiler accusing itself of a defect the
/// caller caused. An actual derived over one output alone under-reports a
/// two-output program and moves the refusal to the second stage.
///
/// The stated budget is one below this program's derived actual, so the boundary
/// refuses; the governed budget admits it, which is what makes the refusal
/// attributable.
#[test]
fn a_two_output_program_over_its_buffer_budget_is_refused_at_the_request_boundary() {
    let semantic = two_chain_program();
    // Two declared inputs and four per declared output.
    let derived = semantic.input_count() + 4 * semantic.output_count();
    assert_eq!(derived, 10);

    let mut narrow = CompilationRequest::governed_preferring(
        &semantic,
        crate::request::NumericalContractPreference::ordered(vec![
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ])
        .unwrap(),
    );
    narrow.budgets.buffers = u32::try_from(derived).unwrap() - 1;

    let failure = compile(narrow).expect_err("the stated buffer budget refuses this program");
    let source = match &failure {
        CompileError::Explained { source, .. } => source.as_ref(),
        other => other,
    };
    assert_eq!(
        source,
        &CompileError::BudgetExhausted(crate::request::RequestError::BudgetExceeded {
            resource: BudgetResource::Buffers,
            limit: 9,
            reported: 10,
        }),
        "the refusal must be the request boundary's, not assembly's",
    );
    // Stated as its own claim rather than left implicit in the equality above:
    // reaching `verify_host_contract` would have produced this instead.
    assert!(
        !matches!(source, CompileError::InvalidCompilerOutput(_)),
        "a caller's budget was reported as a compiler-output defect",
    );

    // The same program under the governed budget compiles, so what the stated
    // budget refused is the budget and not the program.
    assert_eq!(
        retained_stage_counts(
            &semantic,
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ),
        vec![6, 7],
    );
}

/// A one-input, one-output host-expression envelope is not any reachable plan's
/// exact demand.
///
/// `check_program_budgets` derives the compared value as two nodes per input,
/// four per output, and three program-scoped nodes: one input and one output
/// therefore reach nine. That is an upper bound over every plan the request
/// could reach. The widest one-input chain this profile retains declares seven,
/// which is the source's own reason that the envelope cannot also be each
/// plan's exact count.
#[test]
fn a_one_input_host_expression_envelope_exceeds_a_reachable_plan() {
    let semantic = one_chain_program();
    assert_eq!(semantic.input_count(), 1);
    assert_eq!(semantic.output_count(), 1);
    let envelope = semantic
        .input_count()
        .saturating_mul(2)
        .saturating_add(semantic.output_count().saturating_mul(4))
        .saturating_add(3);
    assert_eq!(envelope, 9, "one input and one output reach nine");

    let product = compile(CompilationRequest::governed_preferring(
        &semantic,
        crate::request::NumericalContractPreference::ordered(vec![
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ])
        .unwrap(),
    ))
    .expect("the one-input chain compiles");
    let widest = product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .map(|alternative| alternative.program.core().abi_expressions().len())
        .max()
        .expect("at least one alternative is retained");
    assert_eq!(
        widest, 7,
        "the widest one-input chain declares seven host-expression nodes",
    );
    assert_ne!(
        widest, envelope,
        "an upper bound over every reachable plan cannot also be each plan's exact count",
    );
    assert_eq!(
        crate::request::BudgetResource::HostExpressionNodes.refusal(),
        crate::request::BudgetRefusal::PlanningUpperBound,
    );

    let mut narrow = CompilationRequest::governed_preferring(
        &semantic,
        crate::request::NumericalContractPreference::ordered(vec![
            crate::request::StrictF32NumericalContract::governed_flush_and_reassociate(),
        ])
        .unwrap(),
    );
    narrow.budgets.host_expression_nodes = 8;
    assert_eq!(
        crate::request::verify_request(narrow).err(),
        Some(crate::request::RequestError::BudgetExceeded {
            resource: crate::request::BudgetResource::HostExpressionNodes,
            limit: 8,
            reported: 9,
        }),
        "the request gate compares the envelope of nine, not the reachable plan's seven",
    );
}
