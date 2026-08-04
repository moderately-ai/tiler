---
id: call-the-expansion-cache-preflight-on-the-resolved-root
title: Call the expansion cache preflight on the resolved root
status: todo
priority: p3
dependencies: []
related: [prototype-expansion-content-cache, decide-the-expansion-cache-collection-schedule, configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [cache, frontend, diagnostics]
---
## User-visible outcome

A consumer whose cache root sits on a filesystem the expansion cache's protocol cannot rely on learns that from a diagnostic naming the root and the missing capability, instead of from entries that quietly never hit.

## Why this exists

**Fact — the report is implemented, published, and never taken.** `crates/tiler-cache/src/expansion.rs:115` declares `mod preflight;` and `:131` re-exports `PreflightReport` and `PreflightVerdict`. `crates/tiler-macros/src/cache_root.rs:374` cites it in a doc comment — "`ExpansionCache::preflight` reports the filesystem …" — and that doc comment is the *only* occurrence of the word in the whole crate. Reproduce with `grep -rn preflight crates/tiler-macros/src/`, which returns exactly that one line and no call site.

**Inference — a doc comment that names a check nobody runs reads as a delivered capability.** The next worker takes `cache_root.rs:374` as a statement that the resolved root is preflighted, because that is what the sentence says. The root resolver itself is complete and accepted under [ADR 0089](../docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md), and `open_cache` (`crates/tiler-macros/src/aot.rs:649`) does open a cache on the decided root; what is absent is the filesystem-capability question between those two steps.

**Why this is a diagnostic gap and not a correctness one, stated so the priority is legible.** The cache protocol already fails closed per operation — a publication that cannot take its lock, rename atomically, or validate is not published, and a hit that does not validate is not served. So an unsuitable filesystem costs recompilation and attribution, never a wrong artifact. That is what makes this p3 rather than p1, and it is also why it must not be absorbed silently: the cost lands on a consumer as "the cache does nothing and I cannot tell why".

## Required delivery

- Call `ExpansionCache::preflight` on the resolved root, once per process at most, and decide from the returned `PreflightVerdict` what the expansion does — which must not be failing the build, on the same reasoning that makes an unreadable eviction variable a typed refusal *of the eviction* rather than of the build.
- Surface an unsuitable verdict the way the eviction refusal is surfaced: one attributable line on the expanding process's standard error naming the root, the capability that is missing, and the fact that expansion continues. Read `docs/integration/frontends.md`'s "Compiler cache" section first and match it rather than inventing a second diagnostic shape.
- Correct `crates/tiler-macros/src/cache_root.rs:374` in the same change if the delivered behaviour differs from what it claims. A doc comment and the source disagreeing is the defect this ticket exists because of.
- Watch the refusal fire: point a resolved root at a filesystem or a permission state whose verdict is not the healthy one, and observe the line. A diagnostic whose failure path is never exercised reports success for a population it never examined.

## Non-goals

- The eviction schedule, its bound, or its spelling. Those are decided; see [`configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction`](configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction.md) for the one part still deferred.
- Any new public item on `tiler-cache`. `preflight` is already public; this ticket calls it.

## Closes when

The expansion path calls `ExpansionCache::preflight` on the resolved root, an unsuitable verdict produces an attributable diagnostic without failing the build, that diagnostic is watched firing, and `cache_root.rs`'s doc comment describes what the code does.

## Provenance

Found by [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md) while closing Q-ART-004. That question's root half carried this residue in its own body; closing the question would have dropped it, so it is filed rather than absorbed, and the closure record in `docs/open-questions.md` names this ticket as its owner.
