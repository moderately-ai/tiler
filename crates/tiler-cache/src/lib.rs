//! The cross-process expansion cache protocol ADR 0050 accepts.
//!
//! Expanding an inline Tiler region runs an external compiler, which is the
//! expensive step and the one two concurrent Cargo and rust-analyzer processes
//! are most likely to duplicate. This crate is the storage protocol that lets
//! them share the result: one immutable, self-validating bundle per complete
//! compilation key, published by one atomic rename, validated in full on every
//! hit.
//!
//! # It is an accelerator, and that is a correctness statement
//!
//! Nothing here may turn a cache problem into a compilation problem. A missing
//! root, an unwritable directory, a corrupt entry, a lock that cannot be taken —
//! each of those degrades to compiling without publishing, never to a failed
//! expansion, because ADR 0050's alternatives section rejects the opposite:
//! treating cache failure as compilation failure "would make an optional
//! accelerator a correctness dependency".
//!
//! The converse is equally load-bearing and is where the mistake would be
//! costly. Falling open is *not* silence. Every rejection this crate makes is a
//! typed reason the caller receives inside a [`expansion::CacheReport`], so a
//! cache that is permanently refusing every entry is observable rather than
//! merely slow. There is no path through this crate that produces a miss
//! without producing the reason for it.
//!
//! # Why this crate exists rather than a module in another one
//!
//! Tom decided this on 2026-07-25
//! (`decide-the-expansion-cache-owner-and-digest-authority`). The accepted
//! ownership table had assigned the cross-process cache to `tiler-metal-aot`
//! while the same row decided that crate's dependency closure is empty, and
//! ADR 0050 requires every hit to be validated against the governed digest
//! `tiler.digest.sha-256.v1`, which lives in `tiler-artifact`. That row was not
//! merely awkward, it was unsatisfiable. `tiler-metal-aot` keeps the
//! dependency-free property ADR 0077 admitted it for, and the cache reaches the
//! governed digest through [`tiler_artifact`] rather than owning a second one.
//! [ADR 0082](https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md)
//! records the admission.
//!
//! # Public boundary status
//!
//! [`expansion`] is a **reviewed draft boundary** (ADR 0074 §7, ADR 0075), on
//! the same footing as [`tiler_artifact::program`] and `tiler_runtime::load`. It
//! is `pub` so its shape can be reviewed as a whole; it is not an accepted
//! public facade until Tom accepts the exact interface.

pub mod expansion;
