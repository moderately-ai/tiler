//! This member's uninherited lint table, held against the workspace's.
//!
//! This crate dropped `[lints] workspace = true` at `43f685f` and restated the
//! workspace table with the unsafe-code lint at `deny`, so the one function
//! that must call an Objective-C buffer API can carry a narrow, reasoned
//! `#[allow]`. That divergence had no check of any kind:
//! [ADR 0079](../../../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md)
//! records that `scripts/check_workspace.py` pinned it until `e197176` deleted
//! the Python gate, and that since then the `deny` could be widened to `allow`,
//! or a lint added or dropped from either side, with nothing failing.
//!
//! # Why this file is one `#[path]` line
//!
//! `crates/tiler-conformance` carries the same divergence in the same shape —
//! the workspace table restated with the same one entry at the same level — and
//! `crates/tiler-conformance/src/lints.rs` is a working, fail-closed reader for
//! exactly it. Everything that module asserts is stated relative to *its own*
//! `CARGO_MANIFEST_DIR`: it walks up two directories to the workspace root,
//! checks that root declares a workspace and lists the member it was compiled
//! into, reads both lint tables out of the two files, floors the population so
//! a scan that stopped recognising entries cannot report no drift, and requires
//! the difference to be exactly `rust.unsafe_code` at `forbid` against `deny`.
//! Compiled here, every one of those statements is about this member, and all
//! of them hold.
//!
//! So this check is that module, run a second time from a second root, rather
//! than a second parser written to the same shape. A copy would drift against
//! the original exactly as the original exists to stop the manifests drifting,
//! which is the same failure one layer up.
//!
//! # What does not transfer, stated rather than left to be discovered
//!
//! The included module's **prose** is written from the conformance crate's
//! point of view and is not rewritten here, because editing it would fork it.
//! Read as documentation of this member, three things in it are wrong: the two
//! functions needing the relaxed level live in that crate's `device_buffer`
//! module and this member's live in `src/buffer.rs`; the decision recorded for
//! that crate is Tom's of 2026-08-07 while this member's divergence predates it
//! and is the precedent it was matched against; and its statement that
//! `prototypes/serial-sum-run` "has no check of any kind" is what this file
//! closes. Every **assertion** in it is member-relative and none of that
//! applies to them.
//!
//! Sharing by path rather than by dependency is deliberate and is the only
//! route available: `crates/tiler/tests/dependency_direction.rs` forbids a
//! workspace package from depending on `tiler-conformance`, whose
//! reverse-dependent set is empty by design, and the module is private to it in
//! any case. `crates/tiler/tests/labelled_diagnostic.rs` reads this member's
//! source across the same frontier for the same reason.
//!
//! The member set this file belongs to — which members may diverge at all, and
//! whether each one that does has a check like this — is
//! `crates/tiler/tests/workspace_lint_inheritance.rs`, which names this file as
//! this member's.

#[path = "../../../crates/tiler-conformance/src/lints.rs"]
mod lints;
