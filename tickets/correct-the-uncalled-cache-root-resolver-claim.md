---
id: correct-the-uncalled-cache-root-resolver-claim
title: Correct the uncalled cache-root resolver claim at all four sites
status: done
priority: p2
dependencies: []
related: [re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal]
scopes: [contracts/navigation, research/cache, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, cache, status-drift, graph-repair]
---
## User-visible outcome

Four documents stop asserting that nothing calls the cache-root resolver, so a reader is not told an accepted policy is unexercised when the expansion path exercises it on every invocation.

## Why this exists

**Fact — the expansion path calls it.** `crates/tiler-macros/src/aot.rs:386` calls `open_cache(environment)?` inside the expansion path, and `open_cache` (`:534`) calls `resolve(environment)` at `:535`, mapping its failure to `AotRefusal::CacheRoot` and opening an `ExpansionCache` on the resolved root or a disabled one.

**Fact — the same claim stands at four sites.**

| site | what it says |
| --- | --- |
| `docs/open-questions.md:198` (Q-ART-004) | "What is left is wiring rather than a question: nothing calls the resolver, because no expansion opens a cache." |
| `docs/status.md:37` | "What is deliberately *not* done is the wiring: nothing calls the resolver, because `tensor!` opens no cache and `tiler-macros` holds no edge to `tiler-cache`" |
| `docs/research/cache/root-policy.md:132` | "**Nothing calls the resolver, and nothing here can.** `tensor!` has no grammar and opens no cache" |
| [ADR 0089](../docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md)`:106` | "The resolver exists in `crates/tiler-macros/src/cache_root.rs` with unit tests over every case including every refusal, and nothing calls it: `tensor!` has no grammar and opens no cache, and `tiler-macros` holds no edge to `tiler-cache`" |

**Fact — two of the four carry a reproduction command that now refutes them.** Both `docs/status.md:37` and ADR 0089:106 offer "`grep -n 'tiler-cache' crates/tiler-macros/Cargo.toml` reports no match" as the check. Run it: `crates/tiler-macros/Cargo.toml:46` reads `tiler-cache.workspace = true`, with the rationale at `:33` and `:41`. The claim's own one-line test is what falsifies it — which is the property AGENTS.md asks a stated absence to have, working exactly as intended, one direction later.

**Inference — the live-owner sweep cannot reach this.** [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md) audits questions whose every referenced ticket is terminal. Q-ART-004 names a live owner — [`decide-the-expansion-cache-collection-schedule`](decide-the-expansion-cache-collection-schedule.md) for its open collection half — so it is correctly outside that sweep, and the stale sentence sits in the *closed* half of a question that is legitimately still open. Nothing else looks there.

## Work

Correct all four, in one change, to what is true: the resolver is called on the expansion path, and what remains unexercised is narrower. State the narrower residual exactly rather than deleting the sentence — `ExpansionCache::preflight` is still not called on a resolved root, which both `docs/research/cache/root-policy.md:131` and ADR 0089:107 record separately and which this ticket must not accidentally erase while fixing its neighbour. Where a site offers a reproduction command, replace it with one that passes.

## Boundaries

- The three scopes are the three files' homes plus the ADR's; all four sites move together, because a partial correction leaves the corpus disagreeing with itself, which is worse than agreeing wrongly.
- Do not raise ADR 0089's `implementation_status` as a side effect. Whether the wiring's existence supports a bump is a full read against the decision's clauses, and it belongs to [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md) if it holds anywhere.
- Q-ART-004 stays open on its collection half. Correcting a sentence in the closed half is not closing the question.

## Correction — 2026-08-02, this ticket's own line numbers had drifted

**Fact.** The body cites `crates/tiler-macros/src/aot.rs:386` for the `open_cache` call and `:534`/`:535` for the definition and its `resolve`. On the tree this landed against the call is at `:501` and the definition at `:649`, with `resolve` at `:650`. The claim itself held in every particular — the call chain, the `AotRefusal::CacheRoot` mapping, and the disabled-cache branch are all as described — only the coordinates moved. The corrected sites therefore cite `:501` and `:649`, and each pairs the citation with a `grep` that survives further drift rather than resting on a line number.

**Fact — the residual is unchanged and was re-verified rather than copied.** `grep -rln '\.preflight()' crates --include='*.rs'` reports only `crates/tiler-cache/src/expansion/tests.rs`, so `ExpansionCache::preflight` is still called by nothing outside its own tests and by no expansion. That is what each corrected site now names as the narrower remaining gap.

## Closes when

`grep -rn "nothing calls the resolver" docs/` returns no match, or returns only sentences that are true of the current tree; each corrected site names what is genuinely still unexercised; and every reproduction command a corrected site offers was run and observed to support the sentence it accompanies.
