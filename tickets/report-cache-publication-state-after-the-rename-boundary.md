---
id: report-cache-publication-state-after-the-rename-boundary
title: Report the true cache publication state after atomic rename
status: done
priority: p1
dependencies: []
related: [implement-the-expansion-cache-protocol, accept-the-tiler-cache-public-boundary, inject-deterministic-expansion-cache-io-failures]
scopes: [implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, correctness, diagnostics]
---
A caller must be able to distinguish “no entry was published” from “a valid
entry was published but a later durability or cleanup operation failed.”

## Fact

The cache's atomic rename is the publication point. Once it succeeds, another
process may observe the valid immutable entry. A later parent-directory sync or
lock-release failure cannot undo that fact. Current paths can nevertheless
return `Uncached` with a `PublicationRefusal`, which describes the entry as
unpublished.

## Outcome

Model publication, durability, and cleanup as separate facts. Pre-rename
failures report that no content entry was published. Post-rename failures
report a published valid entry together with weakened durability or cleanup
status. No outcome or explanation may claim that a successful rename did not
occur.

## Closes when

The outcome vocabulary matches the filesystem state at every failure boundary,
callers cannot mistake a published entry for a rebuildable miss, and positive
and fault-injected tests cover both sides of the publication point.

## Outcome

Done. Publication, durability, and cleanup are three facts, and the split is structural rather than documented.

**The defect, at two sites.** `fs::rename` in `ExpansionCache::publish` is the publication point.

- `crates/tiler-cache/src/expansion/store.rs`, the `Fsync` entry-directory sync. It returned `Err(PublicationRefusal::Unavailable)`, and `resolve` maps every `Err` from `publish` to `ProtocolOutcome::Uncached`. So a published, valid, readable entry was reported to the caller as not stored. The comment above that `return` already argued the opposite — "the entry is already published and valid … reported rather than rolled back" — so the reasoning was right and the return shape contradicted it.
- The lock release after a successful publish. It set `publication_refusal` and then returned `published: true`, so one report claimed both that the entry was published and that its publication was refused.

**The fix.** `publish` now returns `Result<Published, PublicationRefusal>`. `Published` is only constructible after a successful rename and carries the quarantine outcome plus an optional durability shortfall, so "published, but" is representable and `Err` means what it says: no content entry exists. `PublicationRefusal` is now strictly pre-publication, which makes its existing doc promise true rather than nearly true. `CacheReport` gains `durability_shortfall` and `cleanup_shortfall`, both `Option<CacheUnavailable>`, both settable only after the rename.

**Evidence, and it fails for the right reason.** Four tests. Two reproduce the original defects and were confirmed to catch them by reverting each fix in isolation: restoring the post-rename `Err` fails with "a post-rename failure must not be reported as uncached", and restoring the lock-release reporting fails with "the lock release failed and is reported". A third asserts an ordinary `Fsync` publication reports no shortfall at all, so the first two cannot pass against a version that sets both fields unconditionally. The fourth states the property the others are instances of: no `Uncached` outcome ever leaves a content entry behind, asserted against a real pre-rename refusal (an oversize bundle, refused before any temporary exists).

Each post-rename test also asserts the filesystem agrees with the report — the entry is at its content path and a second reader observes it.

**A seam had to be added, and it is narrower than the existing one.** `fault::reach` implements a kill point with `process::abort` and so cannot make a call *return an error*, which is exactly the state at issue: a rename that succeeded followed by a step that failed. `fault::Injection` is a `cfg(test)` thread-local with a drop guard, covering the two post-rename sites. Thread-local rather than the environment variable `reach` uses, because that one has to cross a process boundary for the re-exec harness and these faults are observed in-process by the test that armed them; a thread-local keeps one test's arming off every other test's writer.

**Left to the child ticket, deliberately.** `inject-deterministic-expansion-cache-io-failures` depends on this one and owns inducing *every* cache I/O boundary, including its stated decision about whether a read-only directory or a full filesystem image is preferable to a seam for the boundaries this ticket did not need. Two post-rename injection points are what the publication-boundary property required; they are a starting point for that ticket to extend or replace, not an answer to its question.

Gate: `make full` green (969 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck).
