//! Every syntax refusal lands on the token that caused it.
//!
//! This file is the operand-level span evidence Tom's syntax decision turned
//! on: candidate C was eliminated because "a typed error can name the region but
//! not the operand", so a grammar that reported everything at the invocation
//! would have thrown away the reason it was chosen. The golden beside this file
//! records the exact caret position of each diagnostic, so a refusal that
//! silently moved to the invocation would fail rather than merely read worse.
//!
//! Each region is written on its own lines so the caret column is legible, and
//! each differs from the approved region in exactly one token.

fn main() {
    // An element type this profile does not register, refused at the *element
    // type* and not at the operand.
    let _wrong_dtype = tiler::tensor! {
        sym n;
        in a: f64[n], b: f32[n];
        out a * b
    };

    // An operator with no registered operation, refused at the operator.
    let _wrong_operator = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        out a - b
    };

    // A multi-character operator is refused whole rather than read as its first
    // character.
    let _compound_operator = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        out a += b
    };

    // A named operation call, refused at the name.
    let _named_call = tiler::tensor! {
        sym n;
        in a: f32[n];
        out relu(a)
    };

    // A literal extent that is not a plain integer, refused at the literal.
    let _suffixed_extent = tiler::tensor! {
        in a: f32[4usize];
        out a
    };

    // A region with no result.
    let _no_body = tiler::tensor! {
        sym n;
        in a: f32[n];
    };

    // Tokens after the result expression.
    let _trailing = tiler::tensor! {
        sym n;
        in a: f32[n];
        out a;
    };

    // An empty invocation.
    let _empty = tiler::tensor!();

    // A statement that opens with something other than a region keyword.
    let _not_a_statement = tiler::tensor! {
        let a = 1;
        out a
    };
}
