//! Out-of-crate proof that a region *search* budget costs alternatives only.
//!
//! `DeterministicBudgets::governed` states that every `region_*` bound "bounds a
//! *search*, and exhausting one costs an alternative while the verified input
//! and complete coverage survive". The two bounds that really are search bounds
//! — `region_candidates_per_seed` and `region_expansions` — now honour that for
//! both extremes of the partition lattice rather than for the unfused one
//! alone, which is what
//! `region-expansion-exhaustion-loses-the-only-feasible-plan` reported: at
//! twelve semantic operations a shared-constant multiply chain exhausted
//! `region_expansions` before growth reached the whole-program candidate, every
//! surviving cover named an unimplemented region, and the compilation refused
//! `NoFeasiblePlan` — the only implementable plan lost to a bound documented to
//! cost an alternative.
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

/// Past the largest region this profile admits, the refusal names that bound.
///
/// `region_members` is 32, and it bounds one region's admissible *shape* rather
/// than a search: the whole-program region of a thirty-three-operation chain is
/// refused by a declared property of the profile, and with no smaller
/// implementable cover the program has no plan. That is an exhausted
/// deterministic budget, so it carries `BudgetExhausted` — the class whose
/// action is to widen a bound — and never `NoFeasiblePlan`, which the public
/// surface documents as "a hard target rejection, never an exhausted analysis
/// budget".
#[test]
fn a_chain_wider_than_the_largest_admitted_region_refuses_as_an_exhausted_budget() {
    let program = chain_program(33);
    let failure = compile_governed(&program, CONTRACT)
        .expect_err("no region this profile admits covers a thirty-three-operation chain");
    assert_eq!(failure.class(), CompileFailureClass::BudgetExhausted);
}
