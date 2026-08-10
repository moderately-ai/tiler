---
id: call-the-expansion-cache-preflight-on-the-resolved-root
title: Call the expansion cache preflight on the resolved root
status: done
priority: p3
dependencies: []
related: [prototype-expansion-content-cache, decide-the-expansion-cache-collection-schedule, configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction, correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called, correct-adr-0089-and-root-policy-preflight-uncalled-claims]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [cache, frontend, diagnostics]
---
## User-visible outcome

A consumer whose cache root sits on a filesystem the expansion cache's protocol cannot rely on learns that from a diagnostic naming the root and the missing capability, instead of from entries that quietly never hit.

## Why this exists

**Fact — the report is implemented, published, and never taken.** `crates/tiler-cache/src/expansion.rs:115` declares `mod preflight;` and `:131` re-exports `PreflightReport` and `PreflightVerdict`. `crates/tiler-macros/src/cache_root.rs:374` cites it in a doc comment — "`ExpansionCache::preflight` reports the filesystem …" — and that doc comment is the *only* occurrence of the word in the whole crate. Reproduce with `grep -rn preflight crates/tiler-macros/src/`, which returns exactly that one line and no call site.

**Correction — 2026-08-05 (delivered; ticket body recorded 2026-08-10).** The three sentences above are filing motivation, not a live claim. The expansion path now probes: `crates/tiler-macros/src/preflight.rs` defines `report_unsuitable_root` / `reported_to` and a process `PreflightGate`; `aot::open_cache` calls it on the `Directory` arm after opening the cache; `lib.rs` threads `PreflightGate::process()` into `aot::deliver`. Reproduce with `grep -rn report_unsuitable_root crates/tiler-macros/src/`, which returns the definition and the call site. `cache_root::checked`'s doc now points at [`crate::preflight`] against the produced root (once per process, stderr, never a refusal) rather than naming an uncalled API. Line anchors in the retired Fact (`expansion.rs:115` / `:131`, `cache_root.rs:374`) are historical and must not be reasserted as current coordinates.

**Inference — a doc comment that names a check nobody runs reads as a delivered capability.** The next worker takes the old `cache_root` sentence as a statement that the resolved root is preflighted, because that is what the sentence said. The root resolver itself is complete and accepted under [ADR 0089](../docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md), and `open_cache` does open a cache on the decided root; what was absent at filing was the filesystem-capability question between those two steps. That gap is closed in code (see Outcome); residual false "not called" prose remains only in ADR 0089 Consequences and `docs/research/cache/root-policy.md` Unsupported cases, owned by [`correct-adr-0089-and-root-policy-preflight-uncalled-claims`](correct-adr-0089-and-root-policy-preflight-uncalled-claims.md).

**Why this is a diagnostic gap and not a correctness one, stated so the priority is legible.** The cache protocol already fails closed per operation — a publication that cannot take its lock, rename atomically, or validate is not published, and a hit that does not validate is not served. So an unsuitable filesystem costs recompilation and attribution, never a wrong artifact. That is what makes this p3 rather than p1, and it is also why it must not be absorbed silently: the cost lands on a consumer as "the cache does nothing and I cannot tell why". The framing still holds after delivery: the probe reports and never maps into `AotRefusal`.

## Required delivery

- Call `ExpansionCache::preflight` on the resolved root, once per process at most, and decide from the returned `PreflightVerdict` what the expansion does — which must not be failing the build, on the same reasoning that makes an unreadable eviction variable a typed refusal *of the eviction* rather than of the build.
- Surface an unsuitable verdict the way the eviction refusal is surfaced: one attributable line on the expanding process's standard error naming the root, the capability that is missing, and the fact that expansion continues. Read `docs/integration/frontends.md`'s "Compiler cache" section first and match it rather than inventing a second diagnostic shape.
- Correct `crates/tiler-macros/src/cache_root.rs` doc on `checked` in the same change if the delivered behaviour differs from what it claims. A doc comment and the source disagreeing is the defect this ticket exists because of.
- Watch the refusal fire: point a resolved root at a filesystem or a permission state whose verdict is not the healthy one, and observe the line. A diagnostic whose failure path is never exercised reports success for a population it never examined.

## Non-goals

- The eviction schedule, its bound, or its spelling. Those are decided; see [`configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction`](configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction.md) for the one part still deferred.
- Any new public item on `tiler-cache`. `preflight` is already public; this ticket calls it.
- Correcting ADR 0089 or `docs/research/cache/root-policy.md`. Those are outside `implementation/frontend`; see the related remainder.

## Closes when

The expansion path calls `ExpansionCache::preflight` on the resolved root, an unsuitable verdict produces an attributable diagnostic without failing the build, that diagnostic is watched firing, and `cache_root.rs`'s doc comment describes what the code does.

## Outcome — delivered 2026-08-05 (ticket body recorded 2026-08-10)

Implementation under `implementation/frontend` in `tiler-macros`:

- `crates/tiler-macros/src/preflight.rs`: process-static `PreflightGate` (one `AtomicBool` claim), `report_unsuitable_root` / `reported_to`, `UnsuitableRoot` Display naming root, each missing property with `(refuted)` / `(not probed)`, that expansion continues, and `TILER_EXPANSION_CACHE_DIR` / `off`. Disabled cache (`off`) does not claim the gate.
- `aot::open_cache` on the `Directory` arm calls `report_unsuitable_root(preflight, &cache)` after open; returns `Ok(cache)` either way (never a build refusal over the probe).
- `lib.rs` expand threads `preflight::PreflightGate::process()` into `aot::deliver` when the selection invokes the backend compiler.
- `cache_root::checked` doc: filesystem properties are taken by [`crate::preflight`] against the produced root, once per process, reported on stderr, never a refusal.
- Watched firing and amortization: `crates/tiler-macros/src/preflight/tests.rs` — `an_unwritable_root_reports_one_attributable_line` (module docs: **This is the watched firing**), `a_gate_admits_one_probe_per_process`, `a_root_that_answers_for_everything_reports_nothing`, `a_disabled_cache_probes_nothing_and_spends_no_gate`, `every_probed_property_is_rendered_and_a_refutation_reads_as_one`.
- Consumer contract alignment for the probe lives on [`correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called`](correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called.md) (status, open-questions, frontends). No new public `tiler-cache` surface.

Reproduce: `grep -rn report_unsuitable_root crates/tiler-macros/src/`.

## Provenance

Found by [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md) while closing Q-ART-004. That question's root half carried this residue in its own body; closing the question would have dropped it, so it is filed rather than absorbed, and the closure record in `docs/open-questions.md` names this ticket as its owner.

## Fact audit — 2026-08-10

**Verified delivered.** Close conditions in Closes when hold in the tree (Outcome). Status `done` retained.

**Obsolete as live claim.** Why-this-exists "never taken / only doc comment" Fact and its line anchors — superseded by the Correction and Outcome above.

**Residual outside this ticket.** ADR 0089 Consequences still live-asserts `ExpansionCache::preflight` is still not called on a resolved root; `docs/research/cache/root-policy.md` Unsupported cases still live-asserts `ExpansionCache::preflight` is not called. Owned by [`correct-adr-0089-and-root-policy-preflight-uncalled-claims`](correct-adr-0089-and-root-policy-preflight-uncalled-claims.md), not by reopening this implementer.
