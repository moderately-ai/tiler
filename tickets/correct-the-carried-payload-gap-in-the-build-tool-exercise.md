---
id: correct-the-carried-payload-gap-in-the-build-tool-exercise
title: Correct the carried-payload gap in the build-tool exercise record
status: done
priority: p3
dependencies: []
related: []
scopes: [research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, correction]
---
## User-visible outcome

The build-tool exercise record no longer lists a gap that evidence has since closed, so a reader planning work against it is not told to reach something already reached.

## Why

`docs/research/cache/build-tool-exercise.md` section 6 lists "a carried compiled payload" among the cases it did not reach, with the reason "the envelope declares its payload by descriptor rather than carrying object bytes" and the requirement "a backend compilation in the spike, which needs the Metal toolchain". That is no longer true of the corpus. `docs/research/embedding/self-contained-embedding.md` records envelopes produced by `prototypes/serial-sum-compile` carrying compiled `metallib` objects of 3,491-7,158 bytes, resolved through the public `get_or_publish` and validated on every hit by the real `decode_artifact`.

The correction was not taken on `prototype-macro-embedding-and-cargo-behavior` because that record belongs to `exercise-the-expansion-cache-under-cargo-and-rust-analyzer` and editing it needs the `research/cache` scope, which that ticket did not hold. The claim is superseded by evidence either way; what is deferred is only where the record says so.

## Closes when

1. Section 6's "a carried compiled payload" row is corrected rather than deleted, naming the evidence that closed it and preserving what the original gap protected.
2. Section 8's traceability, and any other sentence in that note whose truth depended on the payload being declared rather than carried, is swept and corrected in the same change.
3. Any surviving narrower gap is stated as such. The two stand-in subject facets are a different question and belong to `bind-the-cache-subject-to-the-carried-payload-provenance`; do not absorb them.

## Outcome (2026-07-31)

**Fact.** Section 6's row is corrected rather than deleted: it keeps the original gap and reason, and its "what it would need" cell now records the 2026-07-31 closure by the self-contained embedding note, with the evidence named (metallib-carrying envelopes through the public `get_or_publish`, every hit validated by the real `decode_artifact`) and the original protection stated. Section 6's narrowing paragraph gained a dated correction marking its final sentence no longer true of the corpus while retaining the paragraph as this note's own measurement boundary — its rows never carried object bytes and still do not. The same stale sentence in `spikes/cache/README.md` was corrected in the same change.

**Fact — the sweep.** `grep -n "declared\|carried\|descriptor" docs/research/cache/build-tool-exercise.md` over the remaining matches: line 57 (file descriptors), 114 (declared populations), 153 (declared populations) are unrelated senses; line 137's stand-in subject facets are the different question `bind-the-cache-subject-to-the-carried-payload-provenance` owns and are preserved untouched, per this ticket's third closing condition. Section 8's outcome 1 describes the state at that note's own run date and its ADR 0050 deferral is unchanged — the ADR correction stays with `correct-adr-0050-end-to-end-hit-status`.
