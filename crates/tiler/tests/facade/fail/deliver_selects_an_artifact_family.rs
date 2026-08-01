//! A region selecting an artifact family is refused, and the refusal says what
//! is not wired.
//!
//! Both accepted productions parse — a syntax error here would be a *different*
//! diagnostic, which is what makes this file evidence that the family list is
//! admitted and not merely tolerated — and both then reach the same fail-closed
//! refusal: no expansion runs the offline Metal driver yet, so there is no
//! compiled payload for a selected family to deliver.
//!
//! Emitting the semantic fallback instead is the one thing ADR 0053 forbids
//! outright. A selected family is *required* when the consumer target matches
//! it, so a quiet fallback would give exactly the target that asked for an
//! artifact the thing it asked not to have, with no diagnostic anywhere.
//!
//! The golden beside this file records that each refusal lands on its own
//! `deliver` keyword rather than on the invocation, so a consumer with several
//! regions is told which one it was.

fn main() {
    // The ergonomic production.
    let _profile = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        deliver macos;
        out a * b
    };

    // The escape hatch, on the governed floors.
    let _list = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        deliver macos 14.0, ios 17.0;
        out a * b
    };

    // A floor above the governed one is equally well-formed, and equally
    // undeliverable: the refusal is about compilation, never about the version.
    let _raised = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        deliver ios 18.2;
        out a * b
    };
}
