//! The one ambient input this crate reads, and nothing else in this file.
//!
//! Isolated in a module of its own so the gate's own source can be held to
//! reading none. [`super::tests`] censuses both files: a policy read that
//! drifted back into the reporting path turns that census red instead of
//! passing unnoticed, which is the whole reason the read is here rather than
//! three lines away inside [`super::require_or_report`].
//!
//! # Why an ambient input at all, when the fixture needed none
//!
//! Because this gate's caller is a Rust test function. `make full` invokes these
//! runs through `cargo nextest run --workspace`, and a test function takes
//! no arguments, so the only policy such a call site can state as a literal is
//! one fixed at compile time. [`HostPolicy::Require`] everywhere reddens the
//! workspace gate on every host without an Apple toolchain and a Metal device,
//! which is precisely the host the crate header promises still runs the
//! deterministic half; [`HostPolicy::Report`] everywhere discards the ability to
//! make an unmeasurable host a hard failure, a capability
//! `conform-the-bf16-vertical-end-to-end` and
//! `produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate`
//! each watched working in both directions with quoted output. A human
//! hardening one run by hand has no other channel into a test function's
//! policy, so the input is retained.
//!
//! The independent backend fixture under `tests/independent_backend/` needs no
//! such input because its unavailable host is *manufactured* — an unsatisfiable
//! stack request — so both of its policies are reachable from a literal on any
//! host. Nothing manufactures an absent Apple toolchain here: the runs above
//! report whichever half the machine actually offers.
//!
//! # It can only make a run stricter
//!
//! An unset variable resolves to [`HostPolicy::Report`], which is what every
//! ordinary run already does, and presence is the whole signal, so there is no
//! value a caller can spell that weakens a check.

use super::HostPolicy;

/// The ambient input that turns an unavailable measured half into a failure.
pub(crate) const REQUIRE_MEASUREMENT: &str = "TILER_REQUIRE_METAL_CONFORMANCE";

/// Resolves a caller's host policy from the one ambient input this crate reads.
///
/// Set to anything at all, the empty string included: only presence is read.
pub(crate) fn require_measurement_policy() -> HostPolicy {
    if std::env::var_os(REQUIRE_MEASUREMENT).is_some() {
        HostPolicy::Require {
            named: REQUIRE_MEASUREMENT,
        }
    } else {
        HostPolicy::Report
    }
}
