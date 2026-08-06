---
id: re-price-the-envelope-band-consumers-against-the-re-derived-band
title: Re-price the envelope-band consumers against the re-derived band
status: review
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

## Outcome — 2026-08-06, based at `f4821f2b`

**Both sites are re-priced, the sweep found no third live citer, and no decision ripened — the thirty-day projection survives the new band, with a named trigger that would end it.**

### 1. `MaxEntryAge::DEFAULT`'s ground

The four-bullet ground is untouched, and two dated paragraphs follow it, in the convention [the collection design](../docs/research/cache/bounded-collection.md)'s ground (2) used — the superseded arithmetic is preserved as what the choice was argued from, and the correction states both figures. Added verbatim:

> **Corrected 2026-08-06 — the band the second ground cites has moved, and the window itself is not re-decided here.** Re-running that note's own producer over the same members against the current artifact encoding gives **141,532–159,037 bytes** per envelope. Every carried `metallib` is byte-identical to the 2026-07-31 record, so the growth is entirely artifact encoding rather than backend output, compiler flags, or a Metal toolchain difference. The second ground therefore projects roughly **0.9–1.6 GB** where it says 200–400 MB, and the 200–400 MB it names is now roughly **1,300–2,800 entries**. Its 2026-07-31 figures are left above rather than overwritten, because they are what this choice was argued from; `docs/research/cache/hot-path-efficiency.md`'s Section 9 carries the re-derivation and the attribution, and the collection design carries the matching correction to the same ground.
>
> **The projection still supports thirty days, at roughly a quarter of the margin it had.** Three of the four grounds do not depend on per-entry size at all — the eviction asymmetry, the re-keying that drives growth, and the cold first build a shorter window buys — and the second one's comparison survives in kind rather than by a hair: 0.9–1.6 GB is still well under the Cargo output a single gate of this workspace produces, which `AGENTS.md` puts at 7–15 GB. What is gone is the order of magnitude between them. **One further growth the size of the one just measured would put the steady state at 4–7 GB**, and the second ground would then state the opposite of what it is cited for; that is this window's reconsideration trigger from the disk side, and firing it is a product decision rather than one this crate makes.

**The thirty-day verdict, and why no decision is escalated.** The ground's second clause was cited for one thing — that the steady state is "small beside the build caches already on the same machine" — and at 0.9–1.6 GB that clause is still true, against the 7–15 GB `AGENTS.md` measures one gate's Cargo output at. Grounds (1), (3), and (4) are arguments about eviction asymmetry, toolchain re-keying, and a cold first build, and none of them reads a per-entry size, so the band cannot move them. The window is therefore left exactly where Tom put it and **nothing is escalated**; what the correction adds is the sensitivity, because the margin fell 4.4× and a repeat of the growth just measured would put the projection at 4–7 GB and invert the clause. Naming that as the disk-side reconsideration trigger is the honest form of "not re-decided here": a reader now has the number at which the argument fails rather than an assurance.

### 2. Q-ART-003

Inserted after the 2026-08-04 sentence, which is retained:

> **Re-evaluated 2026-08-06 and still not fired, with the headroom three and a third times smaller than that:** the largest real artifact is **159,037 bytes, 15.17%** of the 1,048,576-byte per-invocation ceiling, and the member [the embedding note](research/embedding/self-contained-embedding.md#5-the-gates-as-numbers) embedded is 146,324 bytes, 13.96%. This is a re-evaluation of a recorded number and **not** a trigger firing, and the distinction is the run-when condition above read literally: no delivery platform has been proposed and neither gate has changed, so nothing this question watches has happened. What moved is the artifact encoding — the band was re-derived at `8bd720b8` by re-running the producer that set it, with every carried `metallib` byte-identical to the 2026-07-31 record, so all of the growth is what the canonical manifest describes rather than backend output ([the hot-path note's Section 9](research/cache/hot-path-efficiency.md#9-the-re-run-at-the-re-derived-band-2026-08-06) attributes it). The 4.56% above is retained as the 2026-07-31 measurement it was. A reader carrying it forward would be reading a figure three and a third times out of date and, more consequentially, would be inferring an order of magnitude of headroom that no longer exists: roughly two thirds of one more threefold growth exhausts the per-invocation gate. Whether the encoding owes a budget is [`attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget`](../tickets/attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget.md)'s, not this question's.

**Verdict, stated rather than implied: not fired.** The run-when condition is "proposing new delivery platforms or changing the current 1 MiB per invocation and 32-invocation/3.2 MiB package gates", and the sweep confirms neither happened — the gates are quoted unchanged from [the cost note](../docs/research/embedding/embedded-artifact-costs.md) and no platform proposal exists. Both anchors were checked against the target headings.

### 3. The sweep

`grep -rnI` over the tree, excluding `target/` and `.git/`, for `32,136`, `47,803`, `4.56`, `141,532`, `159,037`, `15.17`, and separately for the derived `200–400`, `36,838`, `3.51%`, and `ten to twenty megabytes`. Every hit is accounted for:

- **Corrected, in scope:** `crates/tiler-cache/src/expansion/collect.rs:192` and `docs/open-questions.md:213` — the two sites above. Nothing else under `crates/` quotes the band or its derived figures; `crates/tiler-macros/src/eviction.rs`, the only other consumer of the constant, cites `MaxEntryAge::DEFAULT` by name and states no ground of its own, and [the frontend contract](../docs/integration/frontends.md)'s "Compiler cache" section likewise defers to "whose ground is stated on the constant itself". Both are correct *because* of this change rather than needing one.
- **Already carrying a dated correction, and out of scope:** [the collection design](../docs/research/cache/bounded-collection.md) ground (2) (`research/cache`), [the embedding note](../docs/research/embedding/self-contained-embedding.md) §5 and its unsupported-payload row (`research/embedding`), [the hot-path note](../docs/research/cache/hot-path-efficiency.md) §9 and §1's banner, and `spikes/cache/hot-path-efficiency/README.md` and `harness/src/main.rs`'s `SIZES` (`research/cache`). Left untouched.
- **Named and deliberately left — a live figure inside retained evidence.** [The hot-path note](../docs/research/cache/hot-path-efficiency.md) line 143, in Section 5, reads "the 30-day window's own ground projects 200–400 MB, which at these envelope sizes is roughly 5,000–10,000 entries". It is stale *as arithmetic* and correct *as evidence*: Section 1's banner declares Sections 2 to 8 retained unedited as measurements of a smaller envelope, and Section 9.6 re-prices the scan-to-hit ratio the sentence supports (460–590× → 178–213×) and says in terms that a re-reader must not carry the old figure forward. Editing it would break the retention rule the note is built on, and the scope is `research/cache`, which this ticket does not hold.
- **Ticket bodies, not rewritten, as the body above requires:** `decide-the-expansion-cache-collection-schedule`, `admit-an-age-bounded-automatic-eviction-into-the-expansion-cache`, `prototype-macro-embedding-and-cargo-behavior`, `re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`, `decide-whether-the-bundle-envelope-section-digest-is-redundant`, `measure-the-expansion-cache-hot-path-efficiency`, `wire-the-delivered-realization-record-into-the-artifact`, `re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps`, and `attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget`. Each is evidence at its own commit.
- **Unrelated hits:** `4.56`/`15.17` substrings in `docs/research/numerics/`, `spikes/numerics/`, and `spikes/program-planning/` result data, and `36,838` in `spikes/embedding/self_contained.py`, which is that spike's own payload length rather than a band endpoint.

### 4. Checks

`cargo fmt --check`; `cargo check -p tiler-cache`; `cargo clippy -p tiler-cache --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-cache` — all clean. `cargo nextest run -p tiler-cache`: **125 passed, 1 skipped**, unchanged, as a doc-comment-only delta must be. `tkt lint`, `git diff --check`, and `tkt guard` clean.

**The `crates/` delta is doc-comment only** — 27 added `///` lines inside `MaxEntryAge::DEFAULT`'s documentation, no executable line, no signature, no test, and no `Cargo.toml`. It is a `crates/` path, so the reuse rule in `AGENTS.md` does not admit carrying a previous gate on paths alone; what it means for sizing the integration gate is that no behaviour can have moved.
