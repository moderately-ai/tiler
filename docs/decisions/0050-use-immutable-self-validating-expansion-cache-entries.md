---
schema: "tiler-doc/v1"
id: "ADR-0050"
kind: "decision"
title: "Use immutable self-validating expansion-cache entries"
topics: ["cache", "artifacts", "concurrency", "durability"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.artifact-abi", "tiler.contract.frontend-integration", "tiler.contract.metal-backend"]
evidence: ["tiler.research.cache.crash-race-protocol"]
ticket: "cache-crash-race-harness"
---

# 0050: Use immutable self-validating expansion-cache entries

**Status:** accepted

## Context

Cargo and rust-analyzer may run equivalent proc-macro expansions concurrently.
The external Metal compiler is expensive, writers may die at any publication
phase, cache entries may be corrupt or deleted, and the cache is not a runtime
dependency. A lock alone cannot make partial or misplaced bytes correct.

## Decision

The expansion cache stores one immutable self-validating bundle per complete
compilation key. Readers validate bounded framing, embedded key, schemas,
manifest, section lengths/digests, and required meanings on every hit.

On a miss, a writer opens a stable per-key lock file, takes an OS advisory lock,
rechecks, compiles into process-owned state, writes a create-new unique
same-filesystem temporary file, reopens and validates it completely, and
publishes with one atomic rename. The lock suppresses duplicate work;
validation, immutability, complete identity, and atomic publication provide
correctness.

Internal GC retains lock files and takes the key lock before eviction. Cache
I/O failures fall open to validated uncached compilation. Compiler, target, and
artifact failures remain hard expansion errors. The default durability promise
is process-crash safety, not power-loss persistence.

## Consequences

- A killed writer cannot expose a partial final entry.
- Corrupt, truncated, misplaced, or schema-invalid entries are misses.
- Arbitrary external recursive deletion may cause duplicate work but cannot
  authorize unvalidated bytes.
- Generated Rust and binaries remain valid after whole-cache deletion.
- Standard-library locking implies MSRV 1.89 or a separately audited adapter.
- Stronger `fsync`/full-flush policies remain explicit measured options.

## Alternatives considered

PID lock files require unsafe stale-owner recovery. Multi-file entry
directories expose partial publication. Locking readers adds contention without
removing the need for validation. Treating cache failure as compilation failure
would make an optional accelerator a correctness dependency.

## Traceability

Applies to artifact publication and inline expansion. The crash/race report and process harness exercise the accepted protocol. [ADR 0083](0083-keep-process-crash-as-the-default-cache-durability.md) closes the durability-default gate: `ProcessCrash` remains the default on measurements from two supported APFS hosts, while power-loss survival remains explicitly outside the implemented `Fsync` policy's claim.

**Correction, 2026-08-06 — the governed digest this record requires a hit to be validated against no longer lives in `tiler-artifact`.** [ADR 0104](0104-fold-the-per-record-graph-identity-as-a-digest.md) moved `DigestAlgorithm`, `Digest`, `DIGEST_BYTES`, and the wire tag table to a new bottom crate, `tiler-digest`, because the shared IR needed the same one governed algorithm and sits below every other member. This record's decision is unchanged and so is the path `tiler-cache` reaches: `tiler-artifact` re-exports the three names from `tiler_artifact::program`, and ADR 0082 item 2's single decided edge is untouched. The algorithm, the wire tag `0x01`, the key `tiler.digest.sha-256.v1`, and every governed digest byte are identical. What moved is the crate that owns the mapping from tag to implementation, and it moved to make that one-authority property structural rather than a consequence of which crate needed it first.

[ADR 0082](0082-admit-tiler-cache-as-the-expansion-cache-owner.md) names the component that implements this record — `tiler-cache` — after the previously assigned owner proved unable to reach the governed digest this decision requires it to validate against. `implementation_status` remains `partial` rather than `complete`: the namespace, locking, bundle, validation, publication, replacement, reporting, composition of the complete key, cross-process crash and race behaviour, and measured durability default are implemented. Two connected end-to-end boundaries remain. `tiler-metal-aot` still keeps `CompilationIdentity::as_bytes` crate-private, so a production caller cannot supply the backend-compilation facet (`promote-the-metal-aot-compilation-identity`); and the complete inline orchestrator has not yet carried a compiled backend object through publication and a validated hit (`prototype-inline-aot-integration-proof`). These are reachability and integration gaps in the accepted protocol, so changing the status to `implemented` would overstate what a consumer can execute today.

The crash and race behaviour is measured against the bundle `tiler-cache` publishes, not only against the spike's miniature frame. `expansion::harness` re-executes the crate's own test binary so a child is a real process, and a `cfg(test)` seam makes it abort inside the real publication path at each of nine named phases. That in-crate harness is a bounded measurement on one host and substitutes a stand-in payload validator for `decode_artifact` because a real artifact envelope needs `tiler-ir`, which [ADR 0082](0082-admit-tiler-cache-as-the-expansion-cache-owner.md) item 2 decides this crate does not depend on.

The separate [Cargo and rust-analyzer exercise](../research/cache/build-tool-exercise.md) lifts that validator limitation at the integration boundary. Its orchestrator holds the required crates, produces an envelope through a genuine `tiler-compiler` session, encodes it with `tiler-artifact`, resolves it through the public `ExpansionCache::get_or_publish`, and validates every hit with the real `decode_artifact`. Its [recorded macOS result](../../spikes/cache/results/build-tool-exercise-macos-27.0-2026-07-25.tsv) measures concurrent multi-process behaviour under both build tools named in this ADR's context: three overlapping Cargo builds produced four publications and eight validated hits for four keys, and Cargo and `rust-analyzer-proc-macro-srv` concurrently produced four publications and four validated hits. The remaining artifact gap is narrower: that envelope declares its payload by descriptor rather than carrying object bytes, so no compiled backend object has yet travelled through a cache entry.

The complete key is now composed as one canonical byte run:
`tiler_cache::expansion::ComposedSubject` frames the backend compilations and the
artifact program wrapped around them as separately tagged, counted,
length-prefixed facets, and `lookup` and `get_or_publish` accept nothing else, so
under-keying is unrepresentable rather than merely documented. One facet is still
unreachable, and it is the backend-compilation one: `tiler-metal-aot` keeps
`CompilationIdentity::as_bytes` `pub(crate)`, so no other crate can obtain those
bytes, which `promote-the-metal-aot-compilation-identity` covers. The composer
refuses an empty facet, so the cache is composable and not yet usable.

An earlier revision of this paragraph named the *artifact-program* facet as the
one without a producer, on the ground that `CanonicalArtifactProgramIdentity`
needs the payload digest and so cannot exist before compilation. That was a
description of the code and it was wrong: the payload digest is derived from the
payload *metadata* — source, target, flags, and toolchain provenance, and no
object byte — so every fact the identity folds is a compilation input.
`check_payload_identity` re-proves this on every decode, and
`payload_identity_follows_the_compilation_subject_and_not_the_object` asserts
that relinking one source leaves artifact identity equal. `PayloadMetadata::identity`
and `ArtifactProgramBuilder::push_pending_payload` now expose that derivation
without an object. The correction is to what this section *described*, not to
what this ADR decided; the decision, its rationale, and its status are unchanged.
