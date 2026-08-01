//! The versioned map from an Apple artifact family to the consumer `#[cfg]`
//! predicate that selects it.
//!
//! ADR 0053 gates a delivered payload or a retained diagnostic by "the family's
//! versioned consumer-target `#[cfg]` predicate", and
//! `docs/integration/frontends.md` states what that obliges: "The mapping from
//! family to consumer `cfg` predicate is versioned Tiler data and covered by
//! generated-code tests." This module is that data.
//!
//! # Why the frontend owns it and the driver cannot
//!
//! A family's predicate is a fact about a *Rust* target, and
//! [`tiler_metal_aot`] knows only about `xcrun`. ADR 0077 item 1 states that the
//! driver "does not implement the proc-macro layer", and `docs/architecture.md`
//! assigns "emit artifact plus runtime/fallback tokens" to the frontend
//! proc-macro crate. The family vocabulary is still the driver's — this module
//! matches [`ApplePlatform`] exhaustively rather than restating it — so widening
//! that vocabulary is a compile error here rather than a family with no
//! predicate.
//!
//! # Why every predicate names both keys
//!
//! **Measurement — `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, 2026-07-31,
//! `rustc --print cfg --target <triple>`.** `target_abi` is emitted for every
//! target, including as the empty string:
//!
//! | target | `target_os` | `target_abi` |
//! | --- | --- | --- |
//! | `aarch64-apple-darwin` | `macos` | `` |
//! | `aarch64-apple-ios` | `ios` | `` |
//! | `aarch64-apple-ios-sim` | `ios` | `sim` |
//! | `x86_64-apple-ios` | `ios` | `sim` |
//! | `aarch64-apple-ios-macabi` | `ios` | `macabi` |
//! | `aarch64-apple-tvos` | `tvos` | `` |
//! | `aarch64-apple-tvos-sim` | `tvos` | `sim` |
//! | `aarch64-apple-visionos` | `visionos` | `` |
//! | `aarch64-apple-visionos-sim` | `visionos` | `sim` |
//! | `aarch64-apple-watchos` | `watchos` | `` |
//! | `aarch64-apple-watchos-sim` | `watchos` | `sim` |
//! | `x86_64-unknown-linux-gnu` | `linux` | `` |
//!
//! It confirms the distinctions
//! `docs/research/macro-environment/proc-macro-build-environment.md` records,
//! and it is why no predicate is written on `target_os` alone. Three governed
//! families — iOS device, the iOS simulator, and Mac Catalyst — all report
//! `target_os = "ios"`, so `target_os = "ios"` as an iOS-device predicate would
//! deliver a device payload to a simulator and to a Catalyst consumer.
//! `docs/research/apple-targets/numerical-behaviour.md` records that a
//! wrong-family `metallib` loads and dispatches without error, so nothing
//! downstream would catch it.
//!
//! Pairing both keys on *every* family, including macOS where `target_os` alone
//! is currently sufficient, is the same argument applied forward: `macabi` is
//! what a second ABI on an existing `target_os` looks like, and a family whose
//! predicate ignores `target_abi` silently acquires the next one.
//!
//! `--print cfg` needs no standard library for the target it is asked about,
//! which is why the table above reaches eleven targets this host cannot compile
//! for. For the five `docs/correctness-and-testing.md` names normatively, the
//! derivation no longer stops at the `cfg` set:
//! `crate::delivery::tests::every_emitted_shape_compiles_as_the_five_target_matrix_says`
//! compiles the delivery emitter's gated output *for* each of them and records
//! the resulting matrix, so "a nonmatching target compiles the semantic
//! fallback" rests on a build that ran.
//!
//! # What "versioned" obliges
//!
//! [`MAP_VERSION`] names this exact table. Widening the map — a new family, a
//! changed predicate, a third governed key — without bumping it is a defect,
//! because generated code embedded in an already-built consumer was gated by the
//! old table. `the_versioned_map_is_pinned_row_by_row` in this module's tests
//! pins every row so the change cannot pass unnoticed, and
//! `each_family_predicate_matches_exactly_its_own_rust_target` checks the table
//! against `rustc`'s own answer rather than against this module's reading of it.
//!
//! The version does not yet reach an identity subject, because the frontend
//! computes no artifact identity. A region can state a selected family since
//! Tom accepted the `deliver` statement, but no expansion compiles one — the
//! statement is refused before emission — so every expansion delivers
//! `FallbackOnly`, which ADR 0053 defines as invoking no backend compiler. The
//! slice that first compiles a selected family is what folds it in.

use tiler_metal_aot::input::ApplePlatform;

/// The version of this exact family-to-predicate table.
///
/// Domain-qualified rather than a bare integer so it cannot be mistaken for
/// another versioned subject's counter (ADR 0074 convention 3).
#[allow(
    dead_code,
    reason = "the version names the table for the tests that pin it and for the compilation \
              identity that will fold it in. Nothing reads it during an expansion yet because \
              every expansion delivers `FallbackOnly` — a stated selected family is refused \
              before emission, since nothing compiles one — so no expansion embeds a predicate \
              at all; the slice that first compiles a selected family is what makes it an \
              identity input."
)]
pub(crate) const MAP_VERSION: &str = "tiler.frontend.family-consumer-cfg.v1";

/// The `cfg` key naming the consumer's operating system.
const TARGET_OS_KEY: &str = "target_os";

/// The `cfg` key naming the consumer's ABI within that operating system.
const TARGET_ABI_KEY: &str = "target_abi";

/// The consumer-target `cfg` values that identify one artifact family.
///
/// A leaf value record with private fields: it is produced only by
/// [`consumer_cfg`], so there is no way to construct a pair the map does not
/// state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ConsumerCfg {
    target_os: &'static str,
    target_abi: &'static str,
}

impl ConsumerCfg {
    /// Renders the `#[cfg]` predicate that selects this family, without the
    /// surrounding attribute.
    ///
    /// Both keys are always present and always in this order, so two families'
    /// predicates are comparable as text and a reader can see at a glance that
    /// no family is gated on `target_os` alone.
    pub(crate) fn predicate(self) -> String {
        format!(
            "all({TARGET_OS_KEY} = \"{}\", {TARGET_ABI_KEY} = \"{}\")",
            self.target_os, self.target_abi,
        )
    }
}

/// Returns the consumer-target `cfg` predicate values selecting `family`.
///
/// Matched exhaustively and without a wildcard arm: a family added to the
/// driver's vocabulary must be given a predicate here or the frontend does not
/// build. A wildcard would instead give the new family whatever the fallback arm
/// said, which is how a family comes to share another's predicate.
pub(crate) const fn consumer_cfg(family: ApplePlatform) -> ConsumerCfg {
    match family {
        ApplePlatform::MacOs => ConsumerCfg {
            target_os: "macos",
            target_abi: "",
        },
        ApplePlatform::IOsDevice => ConsumerCfg {
            target_os: "ios",
            target_abi: "",
        },
        ApplePlatform::IOsSimulator => ConsumerCfg {
            target_os: "ios",
            target_abi: "sim",
        },
        ApplePlatform::MacCatalyst => ConsumerCfg {
            target_os: "ios",
            target_abi: "macabi",
        },
        ApplePlatform::TvOsDevice => ConsumerCfg {
            target_os: "tvos",
            target_abi: "",
        },
        ApplePlatform::TvOsSimulator => ConsumerCfg {
            target_os: "tvos",
            target_abi: "sim",
        },
        ApplePlatform::VisionOsDevice => ConsumerCfg {
            target_os: "visionos",
            target_abi: "",
        },
        ApplePlatform::VisionOsSimulator => ConsumerCfg {
            target_os: "visionos",
            target_abi: "sim",
        },
        ApplePlatform::WatchOsDevice => ConsumerCfg {
            target_os: "watchos",
            target_abi: "",
        },
        ApplePlatform::WatchOsSimulator => ConsumerCfg {
            target_os: "watchos",
            target_abi: "sim",
        },
    }
}

// `pub(crate)` rather than private because `crate::delivery`'s tests evaluate the
// predicates *this* module renders against `rustc`'s own answer for a real
// target, and a second evaluator written beside them would be a second model of
// one grammar — the duplication that lets a widened predicate pass under a model
// that no longer describes it.
#[cfg(test)]
pub(crate) mod tests;
