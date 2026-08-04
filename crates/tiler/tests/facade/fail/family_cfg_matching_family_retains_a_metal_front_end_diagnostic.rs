//! A consumer whose target matches a family the Apple front end refused sees
//! `metal`'s own diagnostic, in full, and the build fails.
//!
//! The sibling `family_cfg_matching_family_retains_its_diagnostic.rs` stands in
//! for a host with no Apple tools at all, whose retained text is one line. This
//! one stands in for a `metal` that ran and refused, and the difference is what
//! it exists to pin: a real compiler diagnostic is *several* lines with a caret
//! rule and quoted source, and `tiler_macros::delivery::DeliveryPlan::items_source`
//! writes the retained string through `{:?}`, so those newlines must reach the
//! consumer as `\n` escapes inside one literal rather than closing it. Nothing
//! proved that before, because no retained text had ever contained a newline.
//!
//! **Measurement — the retained text below is a verbatim capture, not a
//! composition.** It was produced on 2026-08-04 by
//! `tiler_macros::aot::tests::a_retained_msl_diagnostic_carries_the_emitted_source_position`
//! on macOS 27.0 / Apple M4 Max under Metal Toolchain 27A5228f, running the
//! host's real `metal` binary over that expansion's own emitted MSL. Three parts
//! of it are that run's and cannot be reproduced byte for byte on another: the
//! scratch translation unit's path, the `[…]` executable path — which is the
//! shim that made the real compiler refuse, because
//! `MetalRejection::DefectiveEmission` substitutes the path `--find metal`
//! answers with — and the line and column, which are positions in the MSL that
//! expansion emitted. What is *not* this run's is the shape: the framing, the
//! stage, the exit status, and the compiler's own bytes are what any `metal`
//! refusal retains.
//!
//! **The failure this stands in for is a defect in Tiler's emitter, not
//! something a consumer wrote.** No region text can reach the emitted MSL as an
//! identifier or a literal — entry points, helpers, and staging are named from
//! identity digests, buffers are `b0`, `b1`, …, and constants are emitted as hex
//! bit patterns — so a `metal` refusal of the emitted *source* is unreachable
//! from any invocation and was reached here by injection. The one route a
//! consumer can hit without a Tiler defect is a build host whose `metal`
//! predates the language standard the bound declaration measures; that is the
//! same retained shape with the compiler's flag rejection in place of this
//! source rejection.
//!
//! Nothing built, so there is no envelope and no selector: the gated
//! `compile_error!` is the whole delivery. The item is at column zero because
//! generated tokens carry no indentation, and it is **byte-identical** to what
//! `tiler_macros::delivery::DeliveryPlan::items_source` emits for that plan;
//! `the_metal_front_end_fixture_compiles_what_this_emitter_produces` in the
//! macro crate reads this file and asserts it.

fn main() {
    let _ = {
#[cfg(all(target_os = "macos", target_abi = ""))]
const _: () = { ::core::compile_error!("`tiler::tensor!` could not compile this region's artifact on this build host: Metal AOT driver failed: offline metal failed [/var/folders/7k/00gbj8p92d938w7bqf3k78040000gn/T/tiler-macros-aot-msl-position-7338-ThreadId(2)/metal] (exit code 1): /var/folders/7k/00gbj8p92d938w7bqf3k78040000gn/T/tiler-metal-aot-7338-0-1785871605869051000/kernel.metal:63:39: error: use of undeclared identifier 'tiler_no_such_identifier'\nkernel void tiler_injected_defect() { tiler_no_such_identifier(); }\n                                      ^\n1 error generated."); };
        "the semantic fallback this consumer must not silently receive"
    };
}
