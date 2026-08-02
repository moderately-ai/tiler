//! The identifiers an expansion emits carry the spans the region wrote them at.
//!
//! This is the emission half of the span contract, and nothing else checks it:
//! the refusals in the sibling fixtures are produced *by* the macro, whereas
//! these are produced by rustc against the tokens the macro emitted. If the
//! expansion attributed its operand references to the call site instead, both
//! carets below would move to the whole invocation and this golden would fail.
//!
//! The `in` list is the right place for them: it is the only part of a region
//! that says where its values come from.

fn main() {
    let _missing = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        contract flush_subnormals_to_zero_f32;
        out a * b
    };
}
