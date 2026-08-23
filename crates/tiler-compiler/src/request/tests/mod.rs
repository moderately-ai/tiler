//! The request boundary's unit tests and control populations.
//!
//! # Mapping rule
//!
//! Split from one 7,171-line file
//! (`split-the-compiler-request-test-monolith-into-focused-modules`) mirroring
//! the production seam this directory's parent already has —
//! [`authority`](super::authority), [`budget`](super::budget),
//! [`contract`](super::contract), [`elementwise`](super::elementwise),
//! [`folded`](super::folded), [`graph`](super::graph),
//! [`normal_form`](super::normal_form), [`recognize`](super::recognize),
//! [`refusal`](super::refusal), [`structural`](super::structural),
//! [`subject`](super::subject), [`verified`](super::verified), and
//! [`verify`](super::verify) — plus [`gather`], a cross-cutting child for the
//! gather family, whose tests exercise [`recognize`](super::recognize)'s
//! `recognize_gather`, [`normal_form`](super::normal_form)'s
//! `NormalizedGather`, and [`subject`](super::subject)'s gather encoding
//! together rather than splitting one family's tests across three files that
//! each hold a fragment no reader could use alone. Each other child module
//! holds the tests for its one production module's behaviour, plus any
//! fixture used only there. A fixture used by tests in more than one child
//! lives in [`support`] instead, so it is defined exactly once.
//!
//! `authority`, `refusal`, and `structural` have no test child: every test
//! that exercises one of their items does so through the family gate that
//! calls it, so their behaviour is covered by that gate's own child module
//! (mirroring the schedule-builder split's `diagnostics.rs`, which has no
//! test child for the same reason).
//!
//! `graph` also carries the parametric-broadcast tests (a second symbolic
//! subject read from [`graph`](super::graph)'s `sourced_shape`/`static_shape`
//! helpers and exercised end to end through the physical layer): the combined
//! file comes in well under the 1,500-line bound the split enforces, so no
//! separate overflow sibling was needed the way schedule-builder's
//! `reduction.rs` needed `extrema_reduction.rs`/`squared_reduction.rs`.

mod budget;
mod contract;
mod elementwise;
mod folded;
mod gather;
mod graph;
mod normal_form;
mod recognize;
mod subject;
pub(super) mod support;
mod verified;
mod verify;
