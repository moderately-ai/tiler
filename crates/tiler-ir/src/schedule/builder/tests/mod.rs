//! Bounded tests for the scheduled-region transactional builder and its
//! intrinsic verifier.
//!
//! # Mapping rule
//!
//! Split from one 7,177-line file (`split-the-schedule-builder-test-monolith-into-focused-modules`)
//! mirroring the production seam this directory's parent already has —
//! `contraction.rs`, `copy.rs`, `coverage.rs`, `elementwise.rs`, `family.rs`,
//! `intrinsic.rs`, `proof.rs`, `reduction.rs`, `tile.rs` — plus the whole-region
//! generic construction rules (`ComponentAlreadySet`, `IncompleteRegion`) that
//! belong to none of those families, in [`builder_rules`]. Each child module
//! holds the tests for one production module's behaviour, plus any fixture
//! used only by that module. A fixture used by tests in more than one child
//! lives in [`support`] (or, purely for the size bound below, its overflow
//! sibling [`support_contraction`]) instead, so it is defined exactly once.
//!
//! `reduction.rs` mirrors the production `reduction.rs` alone, but at
//! 1,931 lines that single file would have exceeded this split's own size
//! bound, so its extrema/maximum-fold subject and its squaring-prologue
//! subject were further split into [`extrema_reduction`] and
//! [`squared_reduction`] — both still map to `schedule::builder::reduction`,
//! not to a distinct production module. `support.rs` was split the same way,
//! for the same reason, into [`support`] and [`support_contraction`].
//!
//! `diagnostics.rs` has no test child: every test that exercises one of its
//! refusal constructors does so through the family gate that calls it, so its
//! behaviour is covered by that family's own child module.
//!
//! `support` is `pub(super)` rather than private: `strict_numerical` is
//! reached from `super::gather_tests`, a `builder`-level sibling of this
//! `tests` module rather than a descendant of it, exactly as it was when
//! this directory was the single flat `tests.rs` file.

mod builder_rules;
mod contraction;
mod copy;
mod coverage;
mod elementwise;
mod extrema_reduction;
mod family;
mod intrinsic;
mod proof;
mod reduction;
mod squared_reduction;
pub(super) mod support;
mod support_contraction;
mod tile;
