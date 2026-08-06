---
id: re-price-the-envelope-band-consumers-against-the-re-derived-band
title: Re-price the envelope-band consumers against the re-derived band
status: in-progress
priority: p2
dependencies: []
related: [re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps, admit-an-age-bounded-automatic-eviction-into-the-expansion-cache, wire-the-env-configured-eviction-policy-through-the-deliver-path]
scopes: [implementation/cache, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [cache, artifacts, measurement, documentation]
claimed_from: todo
assignee: agent-reprice
lease_expires_at: 1786052636
---
`re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps` re-derived the envelope band `docs/research/embedding/self-contained-embedding.md` measured, by re-running `prototypes/serial-sum-compile` at `8bd720b8` and taking the minimum and maximum of its members' envelope lengths, the same derivation the original used.

**Measurement — the band moved and the compiled objects did not.** The six reduction-class members the original band was taken over now span **141,532 to 159,037 bytes** against 32,136 to 47,803; the producer's two newer contraction members are 89,250 and 90,737, so its whole population spans 89,250 to 159,037. Every one of the six `metallib` counts is **byte-identical** to the 2026-07-31 record, so none of the growth is backend output. On the hot-path spike's own fixture, whose zero-object envelope is pure fixed content, the overhead is 114,043 bytes at `8bd720b8` against 28,527 at `194744e6`, attributed as +65,363 bytes of canonical manifest, +20,153 bytes of `KernelProgramSubject` section, and a `BackendPayloadMetadata` section that did not move a byte.

That ticket updated the two records it holds scopes for — [the embedding note](../docs/research/embedding/self-contained-embedding.md) and [the hot-path note](../docs/research/cache/hot-path-efficiency.md), plus a dated correction in [the collection design](../docs/research/cache/bounded-collection.md) — and re-ran the sweep at the new endpoints. It could not reach the sites below, each of which quotes the superseded band or a figure derived from it as a **live** measurement.

## What is stale, and what it should read

**`crates/tiler-cache/src/expansion/collect.rs`** (`implementation/cache`). `MaxEntryAge::DEFAULT`'s documentation states its ground as "envelopes of 32,136–47,803 bytes, and the schedule decision records on…". The band is superseded; at 141,532–159,037 bytes the same entry count is roughly 0.9–1.6 GB rather than the 200–400 MB the ground projects, and the same 200–400 MB is roughly 1,300–2,800 entries. **The thirty-day window is a product choice under Tom's decision and this ticket does not re-decide it** — what it owes is a ground that states the current per-entry size and an explicit statement of whether the projection still supports thirty days. If it does not, that is a decision to surface rather than to make.

**`docs/open-questions.md`**, Q-ART-003 (`contracts/navigation`). It records "the largest real artifact is 47,803 bytes, 4.56% of the per-invocation ceiling, so nothing is near a gate", evaluated 2026-08-04 and unfired. At the re-derived band the largest is **159,037 bytes, 15.17%** of the 1,048,576-byte ceiling. The question's run-when condition is "proposing new delivery platforms or changing the current gates", and neither has happened, so this is a **re-evaluation of a recorded headroom rather than a fired trigger** — but a reader taking 4.56% forward would be reading a figure that is three and a third times out of date, and the order-of-magnitude headroom the note inferred from it is gone.

**Also check, and correct only if stale.** `docs/research/cache/bounded-collection.md`'s ground (2) already carries a dated correction and needs no second one. The ticket bodies that quote the band — `decide-the-expansion-cache-collection-schedule`, `admit-an-age-bounded-automatic-eviction-into-the-expansion-cache`, `prototype-macro-embedding-and-cargo-behavior`, `decide-whether-the-bundle-envelope-section-digest-is-redundant`, `measure-the-expansion-cache-hot-path-efficiency`, `re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal` — are **records of what was measured when they ran** and must not be rewritten; a terminal ticket's Outcome is evidence at its own commit.

## Closes when

1. `MaxEntryAge::DEFAULT`'s ground cites the re-derived band and states whether its steady-state projection still supports thirty days, with the superseded projection preserved rather than overwritten.
2. Q-ART-003 records the re-derived headroom beside the 2026-08-04 one, with the evaluation dated and the fired/not-fired verdict stated explicitly.
3. Anything the sweep above finds stale outside those two is either corrected or named here as deliberately left, with the reason.
