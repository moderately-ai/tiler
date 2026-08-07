//! Out-of-crate proof of what each class of region budget costs a caller.
//!
//! `DeterministicBudgets::governed` once stated that every `region_*` bound
//! "bounds a *search*, and exhausting one costs an alternative while the
//! verified input and complete coverage survive", and that was half right. The
//! two bounds that really are search bounds — `region_candidates_per_seed` and
//! `region_expansions` — now honour it for
//! both extremes of the partition lattice rather than for the unfused one
//! alone, which is what
//! `region-expansion-exhaustion-loses-the-only-feasible-plan` reported: at
//! twelve semantic operations a shared-constant multiply chain exhausted
//! `region_expansions` before growth reached the whole-program candidate, every
//! surviving cover named an unimplemented region, and the compilation refused
//! `NoFeasiblePlan` — the only implementable plan lost to a bound documented to
//! cost an alternative.
//!
//! The other three — `region_members`, `region_boundary_outputs`, and
//! `region_live_values` — bound one region's admissible *shape* and can refuse
//! a program outright, which is why they are now derivations over the
//! declaration rather than constants. The last two tests here are that
//! population: the sizes the superseded `region_members` constant refused, and
//! the first size past the program-scoped bound that still refuses.
//!
//! The chain family is the one the measurement was taken on
//! (`spikes/program-planning/identity-growth`): one `tiler::constant-f32@1` and
//! `operations - 1` `tiler::multiply-f32@1` over a rank-1 extent-4 `f32`
//! tensor. Its constant feeds every multiply, so the connected sets grow
//! exponentially in the operation count and the expansion bound binds early;
//! that is the property this file depends on and the reason a *chain* of adds
//! would not reproduce it.

use tiler_compiler::session::{CompileFailureClass, NumericalContract, compile_governed};
use tiler_ir::semantic::{
    F32, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;

/// The extent every program here is built over, held fixed and small.
const EXTENT: u64 = 4;

/// The contract the measured sweep stated, and the one this family compiles at.
const CONTRACT: NumericalContract = NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32;

/// Builds one input, one shared constant, and `operations - 1` multiplies.
fn chain_program(operations: usize) -> SemanticProgram {
    assert!(
        operations >= 2,
        "the chain needs a multiply to make its constant output-reachable"
    );
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([EXTENT]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let mut current = input;
    for _ in 1..operations {
        current = F32Multiply::apply(&mut builder, current, scale).expect("the product applies");
    }
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            current,
        )
        .expect("the output binds");
    let program = builder.build().expect("the program verifies");
    assert_eq!(program.operation_count(), operations);
    program
}

/// The twelve-operation program the defect was measured at compiles.
///
/// Eleven compiled and twelve did not, and nothing about the twelfth operation
/// is a program property: the graph is one operation wider, every budget on the
/// program's *size* still admits it, and the recognizer names the same
/// pointwise family. What refused it was where enumeration stopped.
#[test]
fn the_twelve_operation_chain_that_exhausts_the_expansion_budget_compiles() {
    let program = chain_program(12);
    let compilation = compile_governed(&program, CONTRACT)
        .expect("an exhausted expansion budget must not cost the only feasible plan");
    let selected = compilation
        .selected()
        .expect("the portfolio names a selected alternative");
    let covered: usize = selected
        .abi()
        .kernel_program()
        .stages()
        .map(|stage| stage.coverage().len())
        .sum();
    assert_eq!(
        covered, 12,
        "the selected plan covers every semantic operation"
    );
}

/// The neighbouring points either side of the old wall compile too.
///
/// Eleven is the point that already compiled and must keep its plan; thirteen
/// is past the wall and inside the same regime. Both are asserted so a fix that
/// moved the wall by one instead of removing it fails here.
#[test]
fn the_chain_compiles_on_both_sides_of_the_removed_wall() {
    for operations in [11, 13] {
        let program = chain_program(operations);
        compile_governed(&program, CONTRACT)
            .unwrap_or_else(|failure| panic!("{operations} operations compile: {failure:?}"));
    }
}

/// The population a bare `region_members` constant refused now compiles.
///
/// **This file used to assert the opposite at its first point, and the change
/// is admission rather than a wider search.** `region_members` was the constant
/// `32` while every bound on the program's own *size* admitted sixty-two
/// occurrences, so 33..=62 refused `BudgetExhausted` on a compiler-internal
/// ceiling: the recognized partition of this family is its whole program, no
/// smaller region is implementable, and the whole-program region was the one
/// candidate the constant refused.
/// `derive-the-region-shape-budgets-from-the-declaration` made the bound
/// `semantic_operations`, on the ground that a region's members are a subset of
/// the program's own occurrences, and the whole range plans.
///
/// The assertions are what separate admission from a search that merely ran
/// longer. A search bound costing alternatives leaves *some* plan behind — that
/// is what the tests above prove of `region_expansions`. Here the plan is a
/// single dispatch whose coverage is every one of the program's occurrences,
/// which is precisely the whole-program region the shape bound refused: a wider
/// search could not have produced it, because the candidate it is built from
/// was rejected at formation.
#[test]
fn the_population_the_member_bound_refused_compiles_as_one_whole_program_region() {
    for operations in 33..=62 {
        let program = chain_program(operations);
        let compilation = compile_governed(&program, CONTRACT).unwrap_or_else(|failure| {
            panic!("{operations} operations plan under the derived member bound: {failure:?}")
        });
        let selected = compilation
            .selected()
            .expect("the portfolio names a selected alternative");
        let stages: Vec<usize> = selected
            .abi()
            .kernel_program()
            .stages()
            .map(|stage| stage.coverage().len())
            .collect();
        assert_eq!(
            stages,
            vec![operations],
            "{operations} operations must plan as one region covering the whole program",
        );
    }
}

/// Past the program-size bound, the refusal is still an exhausted budget.
///
/// **The wall moved from a region bound to a program bound, and it is still a
/// wall.** Sixty-three occurrences exceed `semantic_operations`, which
/// `check_program_budgets` refuses before any target is consulted, so the
/// refusal carries `BudgetExhausted` — the class whose action is to widen a
/// bound — and never `NoFeasiblePlan`, which the public surface documents as "a
/// hard target rejection, never an exhausted analysis budget".
///
/// Which resource was exhausted is deliberately not asserted here: the public
/// failure carries the class alone, and
/// `carry-the-exhausted-resource-through-the-budget-refusal` owns widening it.
#[test]
fn a_chain_past_the_program_size_bound_refuses_as_an_exhausted_budget() {
    let program = chain_program(63);
    let failure = compile_governed(&program, CONTRACT)
        .expect_err("sixty-three occurrences exceed the governed operation budget");
    assert_eq!(failure.class(), CompileFailureClass::BudgetExhausted);
}
