//! The expansion cache: its key, its namespace, its bundle, and its protocol.
//!
//! # The five properties this module is specified by
//!
//! `AGENTS.md` names them together, and each has an exact mechanism here rather
//! than an intention:
//!
//! **Complete cache identity.** A key that omits an input does not make a cache
//! slower; it makes a validated hit return an artifact built from different
//! inputs. [`CacheKey::derive`] digests a canonical *subject* the producer
//! supplies, under the governed algorithm and this crate's own domain. What this
//! crate can prove about that subject is stated exactly under [`CacheKey`], and
//! what it cannot prove is stated there too rather than assumed.
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
//! framing rejection — are tested in this crate.
//!
//! **The cross-process crash and race properties are not tested here, and this
//! crate claims nothing about them.** They need real processes killed at each
//! publication phase, which
//! [`spikes/cache/cache_harness.rs`](https://github.com/moderately-ai/tiler/blob/main/spikes/cache/cache_harness.rs)
//! does — for the spike's own miniature frame, on one measured host, not for
//! the bundle this module publishes. A threaded test is not evidence for a
//! process-crash property and is not offered as one; `port-the-cache-harness-to-the-production-bundle`
//! owns closing that gap.
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
mod key;
mod layout;
mod limits;
mod lock;
mod report;
mod store;

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
