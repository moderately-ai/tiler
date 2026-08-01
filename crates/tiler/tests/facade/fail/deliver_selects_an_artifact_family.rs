//! A region selecting an artifact family this frontend cannot build is refused,
//! and the refusal names the one it can.
//!
//! Both accepted productions parse — a syntax error here would be a *different*
//! diagnostic, which is what makes this file evidence that the family list is
//! admitted and not merely tolerated. What each region below then meets is a
//! different gate, and keeping them in one file is what makes the gates legible
//! as a set:
//!
//! - a symbolic extent has no semantic program to compile ahead of time;
//! - a family the one bound Metal compile-time declaration does not measure has
//!   no target to compile for;
//! - a deployment minimum below the governed floor for the standard Tiler
//!   compiles with is the driver's own refusal, at the version that stated it.
//!
//! Emitting the semantic fallback instead is the one thing ADR 0053 forbids
//! outright. A selected family is *required* when the consumer target matches
//! it, so a quiet fallback would give exactly the target that asked for an
//! artifact the thing it asked not to have, with no diagnostic anywhere.
//!
//! The golden beside this file records that each refusal lands on its own token
//! rather than on the invocation, so a consumer with several regions is told
//! which one it was.

fn main() {
    // A symbolic extent: the region is well formed and its policy is
    // buildable, and there is still nothing to compile, because the shape is
    // not known until the values arrive.
    let _symbolic = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        deliver macos;
        out a * b
    };

    // A family with no measured compile-time declaration.
    let _ios = tiler::tensor! {
        in a: f32[4], b: f32[4];
        deliver ios;
        out a * b
    };

    // Several families, one of which is measured: the refusal names only the
    // unmeasured ones, so the golden beside this file is also the evidence that
    // `macos` is absent from a list of what could not be built.
    let _both = tiler::tensor! {
        in a: f32[4], b: f32[4];
        deliver macos-and-ios;
        out a * b
    };

    // A floor below the governed one for the standard the profiles select.
    let _low = tiler::tensor! {
        in a: f32[4], b: f32[4];
        deliver macos 14.0;
        out a * b
    };
}
