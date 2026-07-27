//! The expansion cache: its key, its namespace, its bundle, and its protocol.
//!
//! # The five properties this module is specified by
//!
//! `AGENTS.md` names them together, and each has an exact mechanism here rather
//! than an intention:
//!
//! **Complete cache identity.** A key that omits an input does not make a cache
//! slower; it makes a validated hit return an artifact built from different
//! inputs. [`CacheKey::derive`] digests a [`ComposedSubject`] under the governed
//! algorithm and this crate's own domain, and a composed subject is
//! constructable only by naming every facet of the envelope a bundle carries —
//! the backend compilations *and* the artifact program wrapped around them. What
//! this crate can prove about that subject is stated exactly under [`CacheKey`]
//! and [`SubjectFacets`], and what it cannot prove is stated there too rather
//! than assumed.
//!
//! **Validation on every hit.** [`ExpansionCache::lookup`] has no fast path.
//! Every read decodes the whole bundle frame — magic, schema, algorithm,
//! reserved bits, exact total length, embedded key, every section's bounds and
//! digest — re-derives the key from the carried subject, and then hands the
//! carried envelope to [`tiler_artifact::program::decode_artifact`], which
//! re-proves the manifest digest, every artifact section digest, and the
//! artifact's canonical identity. Nothing is returned before all of it passes.
//!
//! **Immutable entries.** A final entry is never opened for writing, appended
//! to, or edited in place. It is created by one `rename` and afterwards only
//! read, replaced by another whole `rename`, or unlinked.
//!
//! **Atomic publication.** One `rename` from a `create_new` temporary file on
//! the same filesystem is the only operation that makes an entry visible. The
//! temporary is validated through a *separate descriptor* first, so the bytes
//! that are checked are the bytes on disk rather than the bytes in a buffer
//! this process still owns.
//!
//! **Defined crash and race behaviour.** The per-key lock is an OS advisory
//! lock on a stable lock file, released by the kernel when the last descriptor
//! closes, so a killed writer needs no recovery algorithm — there is no PID
//! file, no timestamp lease, and no stale-lock deletion rule. The lock
//! suppresses duplicate compiler work and is deliberately *not* the correctness
//! boundary: correctness comes from the four properties above, each of which
//! holds against a writer that dies at any point.
//!
//! # What is exercised here, and what is not
//!
//! The threaded properties — mutual exclusion on one key, the post-lock recheck
//! that a waiter performs, publication, replacement, immutability, and every
//! framing rejection — are tested in this crate's `expansion::tests`.
//!
//! **The cross-process crash and race properties are measured in
//! `expansion::harness`, against this bundle.** A thread that returns is not a
//! process that was
//! killed, so those properties need real processes stopped at each publication
//! phase; the harness re-executes this crate's own test binary and the armed
//! child aborts inside [`ExpansionCache`]'s publication path, at each of nine
//! named phases. It is a *bounded measurement on one host* — see
//! [`spikes/cache/README.md`](https://github.com/moderately-ai/tiler/blob/main/spikes/cache/README.md)
//! for the exact command and the recorded result — and not a portable
//! guarantee. What the harness substitutes, and why that substitution does not
//! reach these properties, is stated in its own module documentation.
//!
//! # Collection is present, and not yet reachable
//!
//! The private `collect` module implements whole-cache accounting, a bounded
//! collection, and an out-of-service purge. Every one of its types is
//! `pub(crate)` under ADR 0074 convention 7 and none is re-exported here, so
//! **no consumer can collect anything today**. The review that would promote it
//! is `accept-the-tiler-cache-public-boundary`.
//!
//! Its bound defaults to removing nothing, it never blocks on a key lock, and it
//! names every entry it removes. What it preserves of the five properties above,
//! and by which mechanism, is stated in its own module documentation and in
//! [`docs/research/cache/bounded-collection.md`](https://github.com/moderately-ai/tiler/blob/main/docs/research/cache/bounded-collection.md).
//!
//! Likewise, no in-crate test here builds a *real* artifact envelope. Doing so
//! needs a `SemanticProgram`, which needs a frozen registry holding live
//! inferencer implementations, and this crate deliberately does not depend on
//! `tiler-ir`. The public path's delegation to the artifact decoder is proven
//! negatively — bytes that are not an artifact produce a typed artifact
//! rejection through the public API — and the protocol itself is exercised
//! through a crate-private seam that accepts any payload validator. A positive
//! end-to-end hit carrying a real compiled artifact belongs to the orchestrator
//! that holds both crates.

mod bundle;
mod collect;
#[cfg(test)]
mod fault;
#[cfg(test)]
mod harness;
#[cfg(test)]
mod hot_path;
mod key;
mod layout;
mod limits;
mod lock;
mod preflight;
mod report;
mod store;
mod subject;

#[cfg(test)]
mod tests;

pub use bundle::{BundleRejection, BundleSection};
pub use key::{CacheKey, KEY_LABEL_BYTES, KeyTextRejection};
pub use layout::PathRejection;
pub use limits::Limits;
pub use report::{
    CacheOperation, CacheReport, CacheUnavailable, EntryRejection, MissReason, PublicationRefusal,
    QuarantineOutcome,
};
pub use store::{
    CachedEntry, Durability, Eviction, ExpansionCache, Lookup, PublishFailure, Resolution,
    SweepReport,
};
pub use subject::{ComposedSubject, SubjectFacet, SubjectFacets, SubjectRefusal};
