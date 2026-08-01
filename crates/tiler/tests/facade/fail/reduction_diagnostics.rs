//! Every refusal about a reduction or a scalar constant lands on its own token.
//!
//! A separate file from `region_syntax_diagnostics.rs` and
//! `region_meaning_diagnostics.rs` rather than more cases inside them, because
//! the golden beside each of those records caret positions by line: appending to
//! one would rewrite every later entry's line number and make an unrelated diff
//! look like a diagnostic change.
//!
//! Both halves are here together on purpose. A reduction's *syntax* refusals and
//! its *meaning* refusals are one subject for a consumer — "this reduction is
//! wrong, here is where" — and splitting them by which module noticed would
//! organize the evidence around the compiler rather than around the reader.
//!
//! Each region differs from a working one in exactly one token, and the working
//! one is exercised by `../pass/inline_region_executes.rs`.

fn main() {
    // A whole number written without its point, refused at the literal: `x * 2`
    // is not what the surrounding Rust would have accepted either.
    let _integer_constant = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * 2 + 1.0, [cols])
    };

    // A suffixed constant, refused at the literal.
    let _suffixed_constant = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * 2.0f32 + 1.0, [cols])
    };

    // A constant no `f32` can hold, refused at the literal rather than rounded
    // to an infinity.
    let _unrepresentable_constant = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * 1e40 + 1.0, [cols])
    };

    // A reduction with no axis list, refused after its operand.
    let _no_axes = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * 2.0 + 1.0)
    };

    // An empty axis list, refused at the list.
    let _empty_axes = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * 2.0 + 1.0, [])
    };

    // An axis argument that is not a list, refused at what was found.
    let _unbracketed_axis = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * 2.0 + 1.0, cols)
    };

    // A named call this profile does not register, refused at the name.
    let _unregistered_call = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out reduce_sum(x * 2.0 + 1.0, [cols])
    };

    // An axis name no axis of the reduced expression answers to, refused at the
    // name, and the refusal offers the names that exist.
    let _unknown_axis = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * 2.0 + 1.0, [depth])
    };

    // An axis with a literal extent and no name cannot be reduced: a reduction
    // names axes rather than counting positions.
    let _unnamed_axis = tiler::tensor! {
        in x: f32[2, 2];
        out strict_serial_sum(x * 2.0 + 1.0, [cols])
    };

    // A name two axes answer to. `f32[n, n]` stays a legal square shape; what is
    // refused is asking it which axis `n` means.
    let _ambiguous_axis = tiler::tensor! {
        sym n;
        in x: f32[n, n];
        out strict_serial_sum(x * 2.0 + 1.0, [n])
    };

    // One axis named twice by one reduction.
    let _repeated_axis = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * 2.0 + 1.0, [cols, cols])
    };

    // Two operands naming one axis position differently, refused at the operator
    // that combined them rather than resolved in favour of the left.
    let _conflicting_names = tiler::tensor! {
        in x: f32[rows: 2, cols: 2], y: f32[rows: 2, depth: 2];
        out strict_serial_sum(x * y + 1.0, [cols])
    };

    // Negation is not an operation this profile registers: a `-` signs a literal
    // and nothing else.
    let _negated_operand = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        out strict_serial_sum(x * -x + 1.0, [cols])
    };
}
