---
schema: "tiler-doc/v1"
id: "ADR-0050"
kind: "decision"
title: "Use immutable self-validating expansion-cache entries"
topics: ["cache", "artifacts", "concurrency", "durability"]
decision_status: "accepted"
implementation_status: "not-started"
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

Applies to artifact publication and inline expansion. The crash/race report and
process harness exercise the accepted protocol; production durability remains bounded.
