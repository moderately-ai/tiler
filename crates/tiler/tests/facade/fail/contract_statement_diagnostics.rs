//! Every refusal about a region's numerical contract lands where it belongs.
//!
//! Tom decided on 2026-08-01, at the live session, that there is no default
//! numerical contract: a region states one in its own text, and a region that
//! states none is refused with a diagnostic naming what to write. This file is
//! the consumer-visible half of that decision — the goldens beside it are what a
//! consumer actually reads.
//!
//! The two refusals are deliberately different in *where* they land. An unknown
//! name is a token the consumer wrote, so it is reported at the name; an unstated
//! contract is an absence, so no token is responsible and it is reported at the
//! invocation. A refusal that reported both at the invocation would leave a
//! consumer who misspelled one name to find which of several statements was
//! wrong.
//!
//! Each region differs from an accepted one in exactly one way, and each is
//! written on its own lines so the caret column is legible in the golden.

fn main() {
    // No contract at all: refused at the invocation, because the mistake is that
    // nothing was written. The diagnostic is the deliverable here — it names the
    // statement to add and every name it may take.
    let _unstated = tiler::tensor! {
        in a: f32[4], b: f32[4];
        out a * b
    };

    // A name this frontend does not publish, refused at the name.
    let _unknown = tiler::tensor! {
        in a: f32[4], b: f32[4];
        contract fast_math;
        out a * b
    };

    // The constant's own Rust casing is not the region's spelling. Refused
    // rather than case-folded, so each contract has one spelling and not two.
    let _shouted = tiler::tensor! {
        in a: f32[4], b: f32[4];
        contract FLUSH_SUBNORMALS_TO_ZERO_F32;
        out a * b
    };

    // The `deliver` vocabulary's hyphenated style is not this statement's: the
    // `-` sits where the terminator was required, so the refusal names that
    // token rather than reporting an unpublished contract called `flush`.
    let _hyphenated = tiler::tensor! {
        in a: f32[4], b: f32[4];
        contract flush-subnormals-to-zero-f32;
        out a * b
    };

    // Two statements, refused at the second keyword: two contracts would be two
    // answers to what one region's arithmetic means.
    let _repeated = tiler::tensor! {
        in a: f32[4], b: f32[4];
        contract strict_f32;
        contract relaxed_f32;
        out a * b
    };

    // A statement naming nothing.
    let _nameless = tiler::tensor! {
        in a: f32[4], b: f32[4];
        contract;
        out a * b
    };

    // A contract this frontend publishes and the one bound compile declaration
    // cannot honour: the grammar admits it, and the compiler's own target
    // feasibility check is what refuses it. That split is why the region above
    // is refused at the token and this one is not — whether a target honours a
    // stated contract is not a question the grammar pre-answers.
    let _unhonourable = tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        deliver macos;
        contract strict_f32;
        out (a * b) + c
    };
}
