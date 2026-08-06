---
id: re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps
title: Re-derive the measured envelope band the cache hot-path sweeps
status: in-progress
priority: p2
dependencies: []
related: [wire-the-delivered-realization-record-into-the-artifact]
scopes: [research/cache, research/embedding]
shared_scopes: [project/tickets]
paths: []
tags: [research, cache, measurement]
claimed_from: todo
assignee: agent-envelope-band
lease_expires_at: 1786051056
---
`spikes/cache/hot-path-efficiency` sweeps two envelope lengths, `SIZES = [32_136, 47_803]`, and its own comment says why those and not round numbers: they are the exact endpoints of the envelope band `docs/research/embedding/self-contained-embedding.md` measured, and measuring at a measured band's endpoints is what lets a reader put a cost against sizes the corpus already claims are realistic.

**Fact — the band no longer contains one envelope.** Re-running the harness on 2026-08-05 (`wire-the-delivered-realization-record-into-the-artifact`, based at `55d1d09f`) aborts on the harness's own precondition before any row is produced:

```
# envelope fixed overhead=114025 bytes; compiled in 11.777584ms
an envelope cannot be smaller than its 114025 byte fixed overhead; asked for 32136
```

`EnvelopeFactory::exactly` panics deliberately here rather than rounding, on the reasoning its own documentation states: a spike that silently reported a 33 KB envelope as a 32 KB one would put the wrong number beside every per-byte cost it measured.

**Fact — this predates the ticket that found it, and the arithmetic is on the spike's README.** The retained 2026-08-04 results record a fixed overhead of 28,527 bytes. The delivered-realization record contributes 2,453 canonical bytes to this envelope — measured by printing `canonical_bytes().len()` from the repaired harness — carried twice, once inside the folded artifact identity the manifest also carries and once as its own framed run, so roughly 4.9 KB of the current 114,025. The remaining ≈109 KB accumulated between 2026-08-04 and `55d1d09f`, an interval over which `spikes/target-profiles/scalar-cpu-vertical` independently records its own envelope growing from 21,296 to 82,918 bytes with no delivered-realization record involved.

## What this is not

**Not "raise `SIZES` to something above the overhead".** The endpoints are a *measured* claim borrowed from another document, and replacing them with round numbers above the current overhead would substitute convenience for evidence and leave the research note quoting per-byte costs against a band nothing measures any more.

## Closes when

1. The envelope band `docs/research/embedding/self-contained-embedding.md` states is re-derived against the current artifact encoding, or that document records why the earlier band still stands and what the hot-path sweep should measure instead.
2. `spikes/cache/hot-path-efficiency`'s `SIZES` names endpoints derived from that answer, with the derivation stated where the constant is.
3. The spike runs end to end again and records a result; `docs/research/cache/hot-path-efficiency.md` states which of its numbers the re-run reproduced and which moved, keeping the superseded values rather than overwriting them.
4. The two retained 2026-08-04 results are left in place: they are evidence taken at their own commit, and a re-run under a new label is what replaces them.

## Trigger check log

- 2026-08-05: not a deferral — this is dispatchable work with no gate on it. Filed at `todo`. The condition that produced it reproduces with `cd spikes/cache/hot-path-efficiency && cargo build --release && ./target/release/cache-hot-path-efficiency --quick`.
