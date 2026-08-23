use super::super::{
    BTreeSet, BudgetRefusal, BudgetResource, DeterministicBudgets, F32, InputKey, OutputKey,
    RequestError, SemanticProgram, Shape, check_program_budgets, verify_program,
};
use super::support::laws_of;
use tiler_ir::semantic::{F32Add, SemanticProgramBuilder};

/// Builds a program declaring exactly `inputs` inputs and `outputs` ordered
/// named outputs over `operations` occurrences, so a budget's `reported` value
/// can be placed on either side of its bound.
///
/// Every occurrence is one `f32` add producing one value, so
/// `value_count() == inputs + operations`. That is the same identity the
/// decoder layer has — no occurrence in it produces more than one value —
/// and it is the identity `semantic_values` is sized against. The chain
/// consumes every declared input before it starts re-reading the last, so no
/// declared input is left unreached.
///
/// The outputs are the chain's last `outputs` accumulator values, so the
/// output arity moves without moving any of the other three counts: that
/// independence is what lets a probe exceed exactly one of the five bounds.
fn budget_probe(inputs: usize, operations: usize, outputs: usize) -> SemanticProgram {
    assert!(inputs >= 2, "the chain's first add needs two operands");
    assert!(
        operations >= inputs - 1,
        "fewer adds than inputs would leave a declared input unreached",
    );
    assert!(
        (1..=operations).contains(&outputs),
        "each declared output publishes one of the chain's own results",
    );
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let declared: Vec<_> = (0..inputs)
        .map(|index| {
            builder
                .input::<F32>(
                    InputKey::new(format!("input{index}")).unwrap(),
                    Shape::from_dims([2, 3]),
                )
                .unwrap()
        })
        .collect();
    let mut accumulator = declared[0];
    let mut results = Vec::with_capacity(operations);
    for step in 0..operations {
        let operand = declared[(step + 1).min(inputs - 1)];
        accumulator = F32Add::apply(&mut builder, accumulator, operand).unwrap();
        results.push(accumulator);
    }
    for (ordinal, result) in results[operations - outputs..].iter().enumerate() {
        builder
            .output(OutputKey::new(format!("result{ordinal}")).unwrap(), *result)
            .unwrap();
    }
    let program = builder.build().unwrap();
    assert_eq!(program.input_count(), inputs);
    assert_eq!(program.operation_count(), operations);
    assert_eq!(program.output_count(), outputs);
    assert_eq!(program.value_count(), inputs + operations);
    program
}

/// Each widened budget refuses the program one step past it, and the
/// decoder layer's own measured counts are admitted.
///
/// The five program-scoped bounds are sized to that layer, so the admitted
/// neighbours are its two measured rows exactly — eighteen declared inputs
/// and three ordered named outputs over sixty-two occurrences and eighty
/// values at the decode row, and over fifty-eight and seventy-six at the
/// prefill row — and the decode row sits *on* all five bounds rather than
/// under them.
///
/// Refusals are observed through [`verify_program`], which is the entry the
/// budgets guard; admission is observed at [`check_program_budgets`],
/// because clearing the budget gate is the whole of what a budget can
/// promise. `verify_program` still refuses the layer's *shape* at the
/// recognizer under a rule this widening deliberately does not touch, so an
/// admitted probe here is evidence about size and about nothing else.
/// Every budget resource carries its own stable key.
///
/// A duplicate would make two budgets indistinguishable everywhere the key
/// is what travels — the rule key of a request refusal, the resource key of
/// an explain record, the reason code of a failure detail — so a caller told
/// which budget refused would be told the wrong one, silently.
///
/// The population is sized by `variant_count` rather than written out, so a
/// budget added to the vocabulary and not to `ALL` fails the build here
/// rather than shrinking the set this test checks while it still reports no
/// duplicate. The census is printed for the same reason: "nothing ran" must
/// not be able to look green.
#[test]
fn every_budget_resource_key_is_distinct() {
    let keys: BTreeSet<&'static str> = BudgetResource::ALL
        .iter()
        .map(|resource| resource.key())
        .collect();
    assert_eq!(
        keys.len(),
        BudgetResource::ALL.len(),
        "two budget resources share a stable key: {keys:?}",
    );
    assert_eq!(
        BudgetResource::ALL.len(),
        15,
        "the vocabulary changed size; every dependent claim about it needs re-reading",
    );
}

/// The three internal stop vocabularies map onto the shared one injectively.
///
/// Each `resource()` is exhaustive, so `rustc` already proves it total. What
/// it cannot prove is that two internal budgets do not land on one public
/// row, which would report a region stop as a cover stop or the reverse.
///
/// [`crate::cover::CoverBudgetResource::Refusals`] is deliberately absent
/// from the image: it refuses no compilation, and its `None` is what keeps
/// that exclusion typed rather than an inequality at the consuming site.
#[test]
fn the_stop_vocabularies_map_onto_distinct_shared_resources() {
    let region = [
        crate::region::RegionBudgetResource::Members,
        crate::region::RegionBudgetResource::BoundaryOutputs,
        crate::region::RegionBudgetResource::LiveValues,
        crate::region::RegionBudgetResource::CandidatesPerSeed,
        crate::region::RegionBudgetResource::Expansions,
    ];
    let mut image: Vec<BudgetResource> = region.iter().map(|stop| stop.resource()).collect();
    image.extend(
        [
            crate::cover::CoverBudgetResource::Covers,
            crate::cover::CoverBudgetResource::Expansions,
            crate::cover::CoverBudgetResource::Refusals,
        ]
        .iter()
        .filter_map(|stop| stop.truncating_resource()),
    );
    image.push(crate::selection::PlanBudgetResource::Combinations.resource());

    let distinct: BTreeSet<BudgetResource> = image.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        image.len(),
        "two stops share one row: {image:?}"
    );
    assert_eq!(
        image.len(),
        8,
        "five region stops, two cover stops, one plan stop"
    );
    assert!(
        crate::cover::CoverBudgetResource::Refusals
            .truncating_resource()
            .is_none(),
        "the explanation budget refuses no compilation and holds no row",
    );

    // Every one of the eight is a search or shape stop reached after a
    // target is consulted. The five program-scoped and two report-only
    // explain rows are exactly the ones no stop vocabulary maps onto.
    for resource in BudgetResource::ALL {
        let outside_stop_vocabularies = matches!(
            resource,
            BudgetResource::SemanticValues
                | BudgetResource::SemanticOperations
                | BudgetResource::Regions
                | BudgetResource::HostExpressionNodes
                | BudgetResource::Buffers
                | BudgetResource::ExplainDetailRecords
                | BudgetResource::ExplainDetailCanonicalBytes
        );
        assert_eq!(
            outside_stop_vocabularies,
            !distinct.contains(&resource),
            "{resource:?} is claimed by both a stop vocabulary and another refusal authority",
        );
    }
}

/// Every resource reports exactly one of the four provenances, and the
/// population is sized from the type.
///
/// Categories are defined by how the compared number was produced, not by
/// whether it can be described abstractly as a bound. An exact completed
/// count is mathematically both an upper and a lower bound; a conservative
/// envelope computed before selection is not a reachable plan's demand; a
/// search stop is a floor on unexplored work, not the budget success needs;
/// a construction stop is an exact attempted prefix, not the complete
/// trace's demand.
///
/// The match is wildcard-free over [`BudgetResource::ALL`], which is itself
/// sized by `variant_count`, so a sixteenth resource is a build error here
/// rather than a census that still reports four classes over a smaller set.
#[test]
fn every_budget_resource_reports_exactly_one_provenance() {
    let mut exact = 0usize;
    let mut envelope = 0usize;
    let mut search = 0usize;
    let mut construction = 0usize;
    for resource in BudgetResource::ALL {
        let expected = match resource {
            BudgetResource::SemanticValues
            | BudgetResource::SemanticOperations
            | BudgetResource::RegionMembers
            | BudgetResource::RegionBoundaryOutputs
            | BudgetResource::RegionLiveValues => BudgetRefusal::ExactDemand,
            BudgetResource::Regions
            | BudgetResource::HostExpressionNodes
            | BudgetResource::Buffers => BudgetRefusal::PlanningUpperBound,
            BudgetResource::RegionCandidatesPerSeed
            | BudgetResource::RegionExpansions
            | BudgetResource::RegionCovers
            | BudgetResource::RegionCoverExpansions
            | BudgetResource::PhysicalPlanCombinations => BudgetRefusal::SearchLowerBound,
            BudgetResource::ExplainDetailRecords | BudgetResource::ExplainDetailCanonicalBytes => {
                BudgetRefusal::ConstructionLowerBound
            }
        };
        assert_eq!(
            resource.refusal(),
            expected,
            "{resource:?} reports the wrong provenance",
        );
        match expected {
            BudgetRefusal::ExactDemand => exact += 1,
            BudgetRefusal::PlanningUpperBound => envelope += 1,
            BudgetRefusal::SearchLowerBound => search += 1,
            BudgetRefusal::ConstructionLowerBound => construction += 1,
        }
    }
    assert_eq!(
        (
            exact,
            envelope,
            search,
            construction,
            exact + envelope + search + construction,
        ),
        (5, 3, 5, 2, BudgetResource::ALL.len()),
        "provenance census changed; re-read every dependent claim. exact={exact} envelope={envelope} search={search} construction={construction} total={}",
        BudgetResource::ALL.len(),
    );
    eprintln!(
        "budget-resource provenance census: exact={exact} envelope={envelope} search={search} construction={construction} total={}",
        BudgetResource::ALL.len(),
    );
}

#[test]
fn each_widened_budget_refuses_the_program_one_step_past_it() {
    let governed = DeterministicBudgets::governed();

    for (inputs, operations) in [(18, 62), (18, 58)] {
        assert_eq!(
            check_program_budgets(&budget_probe(inputs, operations, 3), governed),
            Ok(()),
            "the decoder layer's measured row {inputs}/{operations} is admitted",
        );
    }

    // Exceeding `semantic_values` alone is not expressible: the bound is
    // exactly the eighteen inputs plus the sixty-two occurrences, so one
    // more value is one more input or one more occurrence. Which resource is
    // reported is therefore the check order's guarantee rather than an
    // accident, and it is the first one.
    assert_eq!(
        verify_program(
            &budget_probe(19, 62, 3),
            governed,
            &laws_of(&budget_probe(19, 62, 3))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::SemanticValues,
            limit: 80,
            reported: 81,
        }),
    );

    assert_eq!(
        verify_program(
            &budget_probe(17, 63, 3),
            governed,
            &laws_of(&budget_probe(17, 63, 3))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::SemanticOperations,
            limit: 62,
            reported: 63,
        }),
    );

    // One further declared output is four further dispatches, and it is the
    // *only* one of these five probes that moves along the output axis. It
    // exceeds all three derived bounds at once — sixteen dispatches,
    // fifty-five expression nodes, and thirty-four buffers — and `regions`
    // is the one that reports, which is the check order's guarantee again.
    assert_eq!(
        verify_program(
            &budget_probe(18, 62, 4),
            governed,
            &laws_of(&budget_probe(18, 62, 4))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::Regions,
            limit: 12,
            reported: 16,
        }),
    );

    assert_eq!(
        verify_program(
            &budget_probe(19, 18, 3),
            governed,
            &laws_of(&budget_probe(19, 18, 3))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::HostExpressionNodes,
            limit: 51,
            reported: 53,
        }),
    );

    // `buffers` is reached only once the bound that shadows it moves, and
    // the shadowing is a property of the two bounds rather than of this
    // test: both are derived from the declared input count and both are
    // tight at eighteen, so a nineteen-input program exceeds them together
    // and the earlier check reports. The perturbation widens
    // `host_expression_nodes` to exactly what nineteen inputs and three
    // outputs need and leaves `buffers` at its governed value, so what is
    // observed refusing is the governed bound.
    let unshadowed = DeterministicBudgets {
        host_expression_nodes: 53,
        ..governed
    };
    assert_eq!(
        verify_program(
            &budget_probe(19, 18, 3),
            unshadowed,
            &laws_of(&budget_probe(19, 18, 3))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::Buffers,
            limit: 30,
            reported: 31,
        }),
    );
}
