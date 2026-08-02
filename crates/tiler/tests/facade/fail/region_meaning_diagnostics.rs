//! Every refusal about what a region *means* lands on the token that caused it.
//!
//! The companion to `region_syntax_diagnostics.rs`: these regions parse. What
//! they fail is a rule about names, shapes, or the bounded profile, and each is
//! reported at the declaration or the operator responsible rather than at the
//! invocation.

fn main() {
    // An axis naming a symbol no `sym` statement declares, refused at the axis.
    let _undeclared_symbol = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[k];
        contract flush_subnormals_to_zero_f32;
        out a * b
    };

    // A declared symbol nothing sources, refused at its declaration.
    let _unsourced_symbol = tiler::tensor! {
        sym n, m;
        in a: f32[n];
        contract flush_subnormals_to_zero_f32;
        out a
    };

    // Two operands under one interface key, refused at the second.
    let _duplicate_operand = tiler::tensor! {
        sym n;
        in a: f32[n], a: f32[n];
        contract flush_subnormals_to_zero_f32;
        out a
    };

    // One symbol declared twice, refused at the second declaration.
    let _duplicate_symbol = tiler::tensor! {
        sym n;
        sym n;
        in a: f32[n];
        contract flush_subnormals_to_zero_f32;
        out a
    };

    // A body reference to a name no `in` statement declares, refused at the
    // reference.
    let _unknown_operand = tiler::tensor! {
        sym n;
        in a: f32[n];
        contract flush_subnormals_to_zero_f32;
        out a * b
    };

    // Shapes that are neither equal nor scalar, refused at the operator.
    let _incompatible_shapes = tiler::tensor! {
        in a: f32[4], b: f32[5];
        contract flush_subnormals_to_zero_f32;
        out a * b
    };

    // Two different symbols are not one shape: nothing at expansion time proves
    // `n` and `m` take one value.
    let _distinct_symbols = tiler::tensor! {
        sym n, m;
        in a: f32[n], b: f32[m];
        contract flush_subnormals_to_zero_f32;
        out a * b
    };

    // `RegionBindError::NoOperands` has no case here on purpose: every body atom
    // is an operand reference, so a region with no `in` statement is refused as
    // an unknown operand first and that refusal is unreachable through the
    // grammar. It remains `binding`'s authority for any other caller and is
    // exercised by `a_region_without_operands_is_refused` in its own tests.
}
