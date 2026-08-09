//! A consumer target matching a selected family that failed to build sees that
//! family's retained diagnostic, and the build fails.
//!
//! The plan this stands in for selects macOS alone, and the offline toolchain was
//! unavailable. ADR 0053 makes this fatal rather than a quiet fallback: a
//! selected family is *required* when the consumer target matches it, so
//! `docs/integration/frontends.md` requires the retained toolchain diagnostic to
//! be emitted "as a `#[cfg]`-gated `compile_error!` item" — otherwise, as
//! `docs/correctness-and-testing.md` puts it, broken generated code could ship
//! unnoticed.
//!
//! Nothing built, so there is no envelope and no selector: the gated
//! `compile_error!` is the whole delivery. Its message is the driver's own text,
//! carried rather than replaced.
//!
//! The item is at column zero because generated tokens carry no indentation, and
//! it is **byte-identical** to what
//! `tiler_macros::delivery::DeliveryPlan::items_source` emits for that plan;
//! `the_retained_diagnostic_fixture_compiles_what_this_emitter_produces` in the
//! macro crate reads this file and asserts it.

fn main() {
    let _ = {
#[cfg(all(target_os = "macos", target_abi = ""))]
::tiler::__private::__tiler_compile_error!("xcrun: error: unable to find utility \"metal\"");
        "the semantic fallback this consumer must not silently receive"
    };
}
