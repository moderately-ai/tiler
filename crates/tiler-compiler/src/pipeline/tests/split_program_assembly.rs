use super::support::{semantic_case_with_axis, split_request};
use super::*;

/// The split's three-stage assembly: the same two-region cover, with its
/// reduction realized by a partial pass and a combining pass.
///
/// The combining pass claims the reduction occurrence's *second* stage — the
/// partial pass claims its first, and the two realize one occurrence between
/// them. Only first stages project onto kernel-program coverage, so the combine
/// is still an uncovering stage at program scope, which whole-program
/// verification admits only because the split contract below is declared.
fn split_assembly(
    request: &crate::request::VerifiedTargetRequest,
    scheduled: &[crate::physical::VerifiedScheduledRegion],
) -> crate::program::CoverAssembly {
    let subject = request.serial_sum();
    let partial = scheduled[1].region().index.iteration_shape.clone();
    let partition = crate::physical::declared_partial_partition(scheduled[1].region())
        .expect("the partial pass declares its split");
    crate::program::CoverAssembly::stated(
        scheduled.to_vec(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (partial, ValueRole::Temporary),
            (subject.output_shape.clone(), ValueRole::Output),
        ],
        vec![
            crate::program::AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![
                    crate::program::AssemblyBinding::Input(0),
                    crate::program::AssemblyBinding::Internal(0),
                ],
            },
            crate::program::AssemblyStage {
                coverage: subject.members.reduction().to_vec(),
                bindings: vec![
                    crate::program::AssemblyBinding::Internal(0),
                    crate::program::AssemblyBinding::Internal(1),
                ],
            },
            crate::program::AssemblyStage {
                coverage: subject
                    .members
                    .reduction()
                    .iter()
                    .map(|atom| atom.next_stage())
                    .collect(),
                bindings: vec![
                    crate::program::AssemblyBinding::Internal(1),
                    crate::program::AssemblyBinding::Internal(2),
                ],
            },
        ],
        vec![crate::program::AssemblySplit {
            producer: 1,
            combiner: 2,
            partial: 1,
            result: 2,
            occurrence: subject.members.reduction()[0].member(),
            partition,
        }],
        Vec::new(),
        Vec::new(),
        vec![(subject.output_key.clone(), 2)],
    )
    .expect("the split assembly is well formed")
}

fn split_regions(
    request: &crate::request::VerifiedTargetRequest,
) -> Vec<crate::physical::VerifiedScheduledRegion> {
    let (raw, members) = crate::physical::pointwise_region(
        request,
        request.sole_output(),
        crate::physical::RegionWrite::Materialized,
    );
    let mut regions = vec![
        crate::physical::verify_schedule(
            raw,
            members,
            request,
            &crate::lowering::ResolvedLowering::unresolved_for_test(),
        )
        .expect("the prologue verifies"),
    ];
    let split = crate::physical::split_reduction_regions(
        request,
        request.sole_output(),
        crate::physical::RegionWrite::ProgramOutput,
    )
    .expect("a four-contributor relaxed request admits the split");
    assert_eq!(split.partition.partitions, 2);
    assert_eq!(split.partition.contributors_per_partition, 2);
    for (raw, members) in split.stages {
        regions.push(
            crate::physical::verify_schedule(
                raw,
                members,
                request,
                &crate::lowering::ResolvedLowering::unresolved_for_test(),
            )
            .expect("each pass verifies"),
        );
    }
    regions
}

/// The assembled split program reproduces the oracle for its own chosen order.
///
/// **The comparison is `strict_partitioned_sum` and never the serial fold.** A
/// split computes a *different* value from the serial reduction — that is what
/// reassociation means — so comparing against the serial answer could only ever
/// pass under a tolerance, and a tolerance is exactly the check that cannot fail
/// for the reason it exists. The oracle for the order the split actually
/// performs is the partitioned sum, and the comparison is bit for bit.
///
/// **The oracle's input is re-derived here and never read back from the run.**
/// It used to be the program's own executed prologue output, justified on the
/// grounds that the split leaves the prologue alone and that re-implementing
/// `scale * x + bias` in the test would assert the test's arithmetic. That is
/// the shared-implementation failure `docs/correctness-and-testing.md` names: a
/// prologue wrong in the implementation is wrong identically in the oracle, and
/// every comparison below still agrees. `split_request` builds the recognized
/// prologue from `2.0` and `1.0`, so the test computes `x * 2 + 1` itself and
/// requires the executed prologue to reproduce it before any fold is compared.
///
/// That direct prologue comparison is load-bearing rather than a restatement of
/// the folds below it. This fixture's magnitudes absorb the bias entirely, so a
/// prologue that regressed to `x * 2 + 2` reaches neither the partials nor the
/// total, and the cancellation maps every scale to `+0.0` at the total. The
/// control below pins both of those blind spots.
#[test]
fn the_assembled_split_program_matches_the_partitioned_sum_oracle() {
    let shape = Shape::from_dims([1, 4]);
    // The prologue maps these to `2e20, 3, -2e20, 3`, whose serial fold
    // `((2e20 + 3) - 2e20) + 3` is `3` while the split's
    // `(2e20 + 3) + (-2e20 + 3)` is `0`. A fixture without that cancellation
    // would let an implementation that never split pass every assertion below.
    let values: Vec<f32> = vec![1.0e20_f32, 1.0, -1.0e20, 1.0];
    let (semantic, request) = split_request(shape.clone());
    let scheduled = split_regions(&request);
    let program = crate::program::build_kernel_program(
        &semantic,
        &request,
        &split_assembly(&request, &scheduled),
    )
    .expect("the split program verifies");
    assert_eq!(program.stage_count(), 3);

    let kernels: Vec<_> = scheduled
        .iter()
        .map(|region| crate::physical::lower_structured_kernel(region).expect("each pass lowers"))
        .collect();
    // The prologue the recognized program applies before the fold, applied here
    // so the oracle's contributors are derived independently of this run.
    let scaled: Vec<f32> = values
        .iter()
        .map(|value| value * 2.0_f32 + 1.0_f32)
        .collect();
    let pointwise = interpret_fused(&kernels[0], &values);
    assert_eq!(
        bits_of(&pointwise),
        bits_of(&scaled),
        "the prologue pass does not compute the recognized `x * 2 + 1`"
    );
    let partials = interpret_fused(&kernels[1], &pointwise);
    let actual = interpret_fused(&kernels[2], &partials);

    let scaled_tensor = f32_tensor(shape, &scaled);
    let axes = [Axis::new(1)];
    let expected_partials =
        tiler_reference::strict_partial_sums(&scaled_tensor, &axes, 2, 2).unwrap();
    let expected = tiler_reference::strict_partitioned_sum(&scaled_tensor, &axes, 2, 2).unwrap();
    assert_eq!(
        bits_of(&partials),
        tensor_bits(&expected_partials),
        "the partial pass staged values the oracle's partial fold does not produce"
    );
    assert_eq!(
        bits_of(&actual),
        tensor_bits(&expected),
        "the assembled split program does not compute its own declared order"
    );

    // The serial fold of the same prologue output disagrees, which is what makes
    // the exact comparison above discriminating.
    let serial_regions = crate::physical::build_scheduled_regions(
        &request,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .unwrap();
    let serial = interpret_fused(
        &crate::physical::lower_structured_kernel(&serial_regions[1]).unwrap(),
        &pointwise,
    );
    assert_ne!(
        bits_of(&serial),
        bits_of(&actual),
        "the fixture no longer distinguishes the two orders, so this test would \
         pass for an implementation that never split"
    );
}

/// The negative control for the oracle's provenance above.
///
/// A regressed prologue stands in for the program's own: `x * 3 + 1` where the
/// recognized program means `x * 2 + 1`. Five properties, each a separate
/// claim, and each watched failing on its own.
///
/// The executed prologue is not the regression, so a program that regressed to
/// it fails here as well as next door. Fed the regression, an oracle derived
/// from that same output — the shape this test carried until the repair —
/// reproduces it exactly at *both* comparisons, so the defective provenance has
/// no way to say *no*; that refusal is what deriving the contributors
/// independently supplies. The independently derived partial fold does refuse
/// it. The independently derived total does not, because this fixture cancels
/// and both scales sum to `+0.0`: the total alone is blind to the prologue,
/// which is why the partial comparison is not redundant and why the prologue is
/// compared directly rather than only through its folds.
#[test]
fn a_regressed_prologue_is_refused_only_where_the_oracle_is_derived_independently() {
    let shape = Shape::from_dims([1, 4]);
    let values: Vec<f32> = vec![1.0e20_f32, 1.0, -1.0e20, 1.0];
    let axes = [Axis::new(1)];
    let (_, request) = split_request(shape.clone());
    let kernels: Vec<_> = split_regions(&request)
        .iter()
        .map(|region| crate::physical::lower_structured_kernel(region).expect("each pass lowers"))
        .collect();

    let regressed: Vec<f32> = values
        .iter()
        .map(|value| value * 3.0_f32 + 1.0_f32)
        .collect();
    assert_ne!(
        bits_of(&interpret_fused(&kernels[0], &values)),
        bits_of(&regressed),
        "the executed prologue is the regression this control stands against"
    );

    let regressed_partials = interpret_fused(&kernels[1], &regressed);
    let regressed_total = interpret_fused(&kernels[2], &regressed_partials);
    let regressed_tensor = f32_tensor(shape.clone(), &regressed);
    assert_eq!(
        bits_of(&regressed_partials),
        tensor_bits(&tiler_reference::strict_partial_sums(&regressed_tensor, &axes, 2, 2).unwrap()),
        "an oracle derived from the executed prologue refuses the partial fold of \
         a regressed prologue, so the repaired provenance is not what supplies \
         that refusal"
    );
    assert_eq!(
        bits_of(&regressed_total),
        tensor_bits(
            &tiler_reference::strict_partitioned_sum(&regressed_tensor, &axes, 2, 2).unwrap()
        ),
        "an oracle derived from the executed prologue refuses the total of a \
         regressed prologue, so the repaired provenance is not what supplies \
         that refusal"
    );

    let scaled: Vec<f32> = values
        .iter()
        .map(|value| value * 2.0_f32 + 1.0_f32)
        .collect();
    let scaled_tensor = f32_tensor(shape, &scaled);
    assert_ne!(
        bits_of(&regressed_partials),
        tensor_bits(&tiler_reference::strict_partial_sums(&scaled_tensor, &axes, 2, 2).unwrap()),
        "the independently derived partial fold cannot tell the two prologues \
         apart, so nothing here refuses a regressed prologue"
    );
    assert_eq!(
        bits_of(&regressed_total),
        tensor_bits(&tiler_reference::strict_partitioned_sum(&scaled_tensor, &axes, 2, 2).unwrap()),
        "the fixture no longer cancels at the total, so the direct prologue \
         comparison is no longer the only guard the bias regression has"
    );
}

/// The split's three-stage program declares the contract its two passes share.
#[test]
fn the_split_program_declares_its_partial_reduction_and_dispatch_order() {
    let (semantic, request) = split_request(Shape::from_dims([1, 4]));
    let scheduled = split_regions(&request);
    let program = crate::program::build_kernel_program(
        &semantic,
        &request,
        &split_assembly(&request, &scheduled),
    )
    .unwrap();
    let core = program.core();
    assert_eq!(core.stages().len(), 3);
    assert_eq!(core.values().len(), 4);

    let declared: Vec<_> = core.partial_reductions().collect();
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0].partitions(), 2);
    assert_eq!(declared[0].contributors_per_partition(), 2);
    assert_eq!(declared[0].total_contributors(), Some(4));
    assert_eq!(
        declared[0].producer().kernel(),
        core.stages().nth(1).unwrap().kernel()
    );
    assert_eq!(
        declared[0].combiner().kernel(),
        core.stages().nth(2).unwrap().kernel()
    );
    assert_eq!(declared[0].partial().role(), ValueRole::Temporary);
    assert_eq!(declared[0].result().role(), ValueRole::Output);
    assert_eq!(declared[0].partial().shape(), &Shape::from_dims([1, 2]));

    // The final pass covers no occurrence: the partial pass already claims the
    // reduction the two of them realize. The whole-program verifier admits that
    // only because the split above is declared — without it the stage is one
    // that computes nothing, and `UncoveringStage` rejects it.
    let coverage: Vec<usize> = core.stages().map(|stage| stage.coverage().len()).collect();
    assert_eq!(coverage, vec![4, 1, 0]);

    // Two ordering edges, both justified by data flow rather than declared. The
    // second is the visibility transition a split relies on instead of a
    // barrier: the pass boundary *is* the dispatch boundary.
    assert_eq!(core.dependencies().len(), 2);
    let ordered: Vec<ValueRole> = core
        .dependencies()
        .map(|edge| match edge.reason() {
            DependencyReasonView::Data(value) => value.role(),
            DependencyReasonView::StorageHandoff(_) => panic!("expected a data edge"),
        })
        .collect();
    assert_eq!(ordered, vec![ValueRole::Temporary, ValueRole::Temporary]);
}

/// The widened budgets admit the split program and still refuse a wider one.
///
/// The widening is an upper bound, not a licence: a request whose stated budget
/// is narrower than the shape this profile may assemble is refused at the
/// request boundary, and a program exceeding the budget its request states is
/// refused at assembly. Both directions are driven, because a widening that
/// only ever admitted would be indistinguishable from removing the check.
#[test]
fn the_widened_budgets_admit_the_split_program_and_still_refuse_a_narrower_request() {
    let semantic = semantic_case_with_axis(
        Shape::from_dims([1, 4]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    // The pre-widening values. The request boundary refuses them by name,
    // rather than admitting a request whose split it would later fail to build.
    for (resource, narrow) in [
        (BudgetResource::Regions, 2_u32),
        (BudgetResource::Buffers, 3),
    ] {
        let mut request = CompilationRequest::governed_under(
            &semantic,
            StrictF32NumericalContract::governed_relaxed(),
        );
        match resource {
            BudgetResource::Regions => request.budgets.regions = narrow,
            _ => request.budgets.buffers = narrow,
        }
        assert!(
            matches!(
                verify_planned_request(request),
                Err(crate::request::RequestError::BudgetExceeded { resource: named, .. })
                    if named == resource
            ),
            "a budget too narrow for the split program was admitted: {resource:?}"
        );
    }

    // And the program-side check still bites: a request stating exactly the
    // widened buffer budget admits the split, one stating less does not reach
    // it at all, so the value that separates them is the one that moved.
    //
    // The budget is `30` rather than the `4` this test first pinned, then the
    // `6`, then the `21`, because it is sized to the largest program shape the
    // profile may be asked to admit and that is the eighteen-input, three-output
    // decoder layer under a per-output derivation. The *requirement* this
    // one-input, one-output program places on it is five — the declared input,
    // the prologue's temporary, the split's staged partial tensor, the fold's
    // staged result an epilogue would read, and the output — and
    // `verify_program` derives that from the declared arities, which is what the
    // `buffers: 3` refusal above drives. That is the point of the pair: the
    // bound moved and the derived demand did not, so a widening that had removed
    // the check would fail the loop above.
    let (semantic, request) = split_request(Shape::from_dims([1, 4]));
    let scheduled = split_regions(&request);
    assert!(
        crate::program::build_kernel_program(
            &semantic,
            &request,
            &split_assembly(&request, &scheduled)
        )
        .is_ok()
    );
    assert_eq!(request.budgets().buffers, 30);
    // `regions` moved from `3` to `4` with the epilogue admission and from a
    // literal `4` to a derived four *per declared output* with multi-output
    // admission, so the governed value is the decoder layer's three outputs
    // times that chain. This split program's own demand is still four by the
    // derivation and three by the plan it actually assembles, which is why
    // nothing above this line moved.
    assert_eq!(request.budgets().regions, 12);
}

/// **The closing evidence of
/// `admit-a-reassociating-contract-without-contraction`.** The recognized
/// serial-sum program compiles under a reassociation-permitting contract,
/// `compile` retains the three-stage split beside the two-stage serial one, and
/// the selected plan is still the serial one.
///
/// The last clause is the one that matters for the ticket boundary: the split is
/// *enumerated and retained*, and the serial plan is selected because the
/// structural cost model prices two dispatches and a staged partial tensor above
/// one dispatch and no temporary. Nothing here calibrates anything, and nothing
/// here declares a cost row — the governed profile this compiles against carries
/// none, so `activate-measured-reduction-selection-from-a-target-cost-row`'s
/// silence rule is exactly why this assertion is unchanged by that landing.
///
/// The strict compilation at the end is the perturbation: the same program under
/// a contract that forbids reassociation retains no three-stage alternative at
/// all, so the assertion above cannot pass for a build that never split.
#[test]
fn the_reassociating_contract_reaches_the_split_through_compile() {
    /// The stage counts of one compilation's retained alternatives, ascending.
    fn retained_stage_counts(product: &CompilationProduct) -> Vec<usize> {
        let mut counts: Vec<usize> = product.targets[0]
            .portfolio
            .alternatives
            .iter()
            .map(|alternative| alternative.program.stage_count())
            .collect();
        counts.sort_unstable();
        counts
    }

    // Four contributors: the extent `governed_partition` splits two-by-two and
    // the largest the governed target's declared grid-axis guarantee admits.
    let semantic = semantic_case_with_axis(
        Shape::from_dims([1, 4]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let product = compile(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_reassociating(),
    ))
    .expect("the reassociating contract compiles the recognized serial sum");
    let target = &product.targets[0];

    // Two stages is the materialized prologue-then-reduce plan; three is the
    // same cover with its reduction realized as a partial and a final pass. The
    // whole-program fused plan is absent, and on a *different* obligation: it
    // contains the reduction, whose permitted reassociation `derive_fusion_legality`
    // does not prove — see
    // `fusion_legality::tests::a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction`.
    assert_eq!(retained_stage_counts(&product), vec![2, 3]);

    let selected = target
        .portfolio
        .alternatives
        .iter()
        .find(|alternative| {
            alternative.stable_id == target.portfolio.selection.selected_alternative_id
        })
        .expect("the selected alternative is one of the retained ones");
    assert_eq!(
        selected.program.stage_count(),
        2,
        "the split was selected; preference belongs to calibration, not to this ticket"
    );

    // Perturbation: forbidding reassociation withholds the split entirely, so
    // the three-stage retention above is a property of the contract rather than
    // of the program.
    let strict = compile(CompilationRequest::governed(&semantic))
        .expect("the strict contract compiles the same program");
    assert_eq!(retained_stage_counts(&strict), vec![1, 2]);
}

/// A staged realization's explain record names both regions and the handed value.
///
/// **Stated against the fact derivation rather than a compilation, and the
/// boundary is deliberate.** No standard operation carries the staged
/// realization law today — the normalization that will needs a governed
/// reciprocal square root that does not exist — and the recognizer admits only
/// the pointwise, serial-sum, and contraction families, so no program a caller
/// can state reaches `record_refinement` with a chain. Testing the derivation is
/// therefore what is reachable; the one-line call site in `record_refinement` is
/// not covered by this and is stated as such rather than implied.
#[test]
fn a_staged_realization_names_its_regions_and_its_handed_value() {
    use tiler_ir::index::{
        DomainRole, IndexRegionBuilder, ScalarAttributes, StagedInputSource, TensorRole as IrRole,
        VerifiedIndexRegion, VerifiedIndexRegionSequence, multiply_f32_scalar_op,
    };
    use tiler_ir::shape::Extent;

    let scalars = crate::governed::governed_scalars().unwrap();
    // Emits `out[i] = mul(in[0][i], in[last][i])` over `[extent]`, so a
    // one-input region squares and a two-input region multiplies its operands.
    let product = |inputs: &[u64], extent: u64| -> VerifiedIndexRegion {
        let mut builder = IndexRegionBuilder::new(scalars.clone()).unwrap();
        let point = builder
            .dimension(DomainRole::Parallel, Extent::new(extent))
            .unwrap();
        let coordinate = builder.dimension_expr(point).unwrap();
        let tensors: Vec<_> = inputs
            .iter()
            .map(|input| {
                builder
                    .tensor(
                        IrRole::Input,
                        F32::resolved_type(),
                        Shape::from_dims([*input]),
                    )
                    .unwrap()
            })
            .collect();
        let left = builder.read(tensors[0], &[point], &[coordinate]).unwrap();
        let right = builder
            .read(*tensors.last().unwrap(), &[point], &[coordinate])
            .unwrap();
        let value = builder
            .apply(
                multiply_f32_scalar_op(),
                ScalarAttributes::empty(),
                &[left, right],
            )
            .unwrap()
            .get(0)
            .unwrap();
        let output = builder
            .tensor(
                IrRole::Output,
                F32::resolved_type(),
                Shape::from_dims([extent]),
            )
            .unwrap();
        let write = builder.write(output, &[point], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
        builder.build().unwrap()
    };

    let fold = product(&[4], 4);
    let pass = product(&[4, 4], 4);
    let chained = VerifiedIndexRegionSequence::try_new(
        vec![fold.clone(), pass.clone()],
        vec![
            vec![StagedInputSource::Occurrence(0)],
            vec![
                StagedInputSource::Occurrence(1),
                StagedInputSource::Intermediate(0),
            ],
        ],
    )
    .unwrap();

    let base = || {
        PredicateAssessment::proven(
            "kernel.index-region-refines-occurrence",
            EvidenceBasis::ExhaustiveFinite,
        )
        .unwrap()
    };
    let staged = super::trace::with_realization_facts(base(), &chained).unwrap();
    let facts: BTreeMap<String, crate::explain::FactValue> = staged
        .facts()
        .iter()
        .map(|fact| (fact.key().as_str().to_owned(), fact.value().clone()))
        .collect();

    assert_eq!(
        facts.get("realization-stages"),
        Some(&crate::explain::FactValue::Count(2))
    );
    // Both regions are named, and they are named *distinctly*: a derivation that
    // reported one stage twice would satisfy a count assertion alone.
    let named = |ordinal: usize| match facts.get(&format!("realization-stage-{ordinal}-region")) {
        Some(crate::explain::FactValue::Identity(key)) => key.clone(),
        other => panic!("stage {ordinal} is not named: {other:?}"),
    };
    assert_ne!(named(0), named(1));
    assert_eq!(
        facts.get("realization-intermediate-0-producer"),
        Some(&crate::explain::FactValue::Count(0))
    );
    assert_eq!(
        facts.get("realization-intermediate-0-consumer"),
        Some(&crate::explain::FactValue::Count(1))
    );
    assert_eq!(
        facts.get("realization-intermediate-0-elements"),
        Some(&crate::explain::FactValue::Count(4))
    );

    // The perturbation that keeps every existing record's rendering unmoved: a
    // one-stage realization adds nothing at all, so no governed compilation's
    // explain output changes because this derivation exists.
    let single =
        super::trace::with_realization_facts(base(), &VerifiedIndexRegionSequence::single(pass))
            .unwrap();
    assert!(single.facts().is_empty());
}

// ---------------------------------------------------------------------------
// Measured reduction selection from a target cost row
// ---------------------------------------------------------------------------
//
// `activate-measured-reduction-selection-from-a-target-cost-row`. The evidence
// below, in the order the ticket asks for it: the design premise (the parallel
// plans are structurally *dominated*, so a term confined to the non-dominated
// view could decide nothing), the unchanged golden a silent profile owes, the
// mutation proof on the declared term, the shapes checked against the retained
// TSV rather than re-argued, and the explain row naming the term and both sides
// of the `max`.
//
// **The profile is a widened test profile rather than the authoritative Apple9
// declaration**, for the reason `workgroup_tree_target_for_test` gives at
// length: this crate cannot see `tiler-build`'s bound declaration, and raising
// the prototype baseline's own rows would be a capability claim. The value used
// below is the one that declaration carries, so the two agree on the number
// without this crate depending on that one.
