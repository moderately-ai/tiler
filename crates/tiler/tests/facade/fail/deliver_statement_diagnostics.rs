//! Every `deliver` refusal lands on the token that caused it.
//!
//! The same property the region grammar's own diagnostics fixture records, for
//! the statement Tom accepted on 2026-07-31. A delivery policy decides which
//! targets a consumer's build compiles artifacts for, so a refusal that named
//! the invocation would leave the consumer to find which of profile name,
//! family name, or version was wrong.
//!
//! The two vocabularies are split deliberately across the cases below: an
//! unknown *profile* is refused against the four profile names, and an unknown
//! *family* against the two family names, because a list and a profile do not
//! accept the same words. `ios-device` appears as a family case for that reason
//! — it is the driver's own identifier, which the consumer surface does not
//! publish.
//!
//! Each region differs from an accepted one in exactly one token, and each is
//! written on its own lines so the caret column is legible in the golden.

fn main() {
    // A profile name this frontend does not publish, refused at the name.
    let _unknown_profile = tiler::tensor! {
        in a: f32[4];
        deliver macos-and-tvos;
        out a
    };

    // The underscored spelling of an accepted profile is still not one.
    let _underscored_profile = tiler::tensor! {
        in a: f32[4];
        deliver fallback_only;
        out a
    };

    // A driver family identifier is not a family a list may name.
    let _unknown_family = tiler::tensor! {
        in a: f32[4];
        deliver ios-device 17.0;
        out a
    };

    // A deployment minimum below the governed floor, refused at the version and
    // carrying the driver's own reason.
    let _below_floor = tiler::tensor! {
        in a: f32[4];
        deliver macos 13.0;
        out a
    };

    // A version that is not `<major>.<minor>`, refused at the literal.
    let _malformed_version = tiler::tensor! {
        in a: f32[4];
        deliver macos 14;
        out a
    };

    // A list whose second family states no minimum.
    let _malformed_list = tiler::tensor! {
        in a: f32[4];
        deliver macos 14.0, ios;
        out a
    };

    // One family stated twice, refused at the repetition.
    let _repeated_family = tiler::tensor! {
        in a: f32[4];
        deliver macos 14.0, macos 15.0;
        out a
    };

    // Two `deliver` statements, refused at the second keyword.
    let _repeated_statement = tiler::tensor! {
        in a: f32[4];
        deliver fallback-only;
        deliver fallback-only;
        out a
    };
}
