---
id: correct-adr-0089-and-root-policy-preflight-uncalled-claims
title: Correct ADR 0089 and root-policy claims that the cache-root preflight is uncalled
status: in-progress
priority: p3
dependencies: []
related: [call-the-expansion-cache-preflight-on-the-resolved-root, correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called]
scopes: [contracts/decisions, research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, frontend, diagnostics, documentation]
claimed_from: todo
assignee: terra-cache-preflight-prose
lease_expires_at: 1786408090
---
## User-visible outcome

ADR 0089 and the root-policy research note stop live-asserting that `ExpansionCache::preflight` is never called on a resolved root, so decision and research prose agree with the landed frontend probe and with the navigation/integrations corrections already done.

## Why this exists

**Fact — the probe is called.** [`call-the-expansion-cache-preflight-on-the-resolved-root`](call-the-expansion-cache-preflight-on-the-resolved-root.md) is `done`: `crates/tiler-macros/src/preflight.rs` probes the resolved root once per process from `aot::open_cache` on the `Directory` arm, and an unsuitable verdict is one attributable stderr line without failing the build. Reproduce: `grep -rn report_unsuitable_root crates/tiler-macros/src/`.

**Fact — two documents still live-assert the opposite.** [ADR 0089](../docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) Consequences still contains `ExpansionCache::preflight` is still not called on a resolved root. `docs/research/cache/root-policy.md` Unsupported cases still contains `**`ExpansionCache::preflight` is not called.**` Neither has a Corrected successor. Reproduce: `grep -n 'preflight' docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md docs/research/cache/root-policy.md`.

**Fact — the three-document sibling deliberately left these sites.** [`correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called`](correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called.md) delivered status, open-questions, and frontends under `contracts/navigation` and `contracts/integrations`. Its Non-goals and Fact audit name ADR 0089 and root-policy as residual debt outside that ticket's scopes and Required delivery. This remainder owns those two sites only.

**Why not reopen the implementer.** The implementation ticket's scope is `implementation/frontend`; it must not absorb decision or research prose. Path-only root policy (what ADR 0089 decides) remains true; only the uncalled-probe Consequence is false.

## Required delivery

- Correct the ADR 0089 Consequences bullet that asserts preflight is still not called: dated correction or rewrite stating the probe is called from `tiler-macros` `open_cache` / `preflight` once per process, that an unsuitable answer is diagnostic-only (never a refusal), and that this ADR still only decides path-only root policy from override or user cache — preflight remains the I/O capability report that composes with that decision rather than replacing it.
- Correct the `docs/research/cache/root-policy.md` Unsupported cases bullet the same way (or rephrase so the unsupported list describes the pure chooser evidence boundary without asserting the expansion path never probes).
- Prefer the corpus's **Corrected \<date\>** idiom so historical wording stays readable; do not silently delete without a successor sentence that a later census can trust.
- Any reproduction command a corrected site offers must pass against the tree after the edit.

## Non-goals

- Changing the diagnostic, gate amortization, or call site. Those are implemented and tested on the related implementer.
- Re-opening whether the probe should refuse a build. It must not.
- Re-correcting status, open-questions, or frontends (already done on the related docs ticket).
- Bumping ADR 0089 `implementation_status` as a side effect of the prose fix; that is a full re-audit against the decision's clauses if needed elsewhere.

## Closes when

Neither ADR 0089 nor `docs/research/cache/root-policy.md` live-asserts that `ExpansionCache::preflight` is uncalled on a resolved root without an immediate dated successor that states the landed probe; each site's stated reproduction (if any) supports the corrected claim.

## Provenance

Filed 2026-08-10 from Phase B ticket audit of [`call-the-expansion-cache-preflight-on-the-resolved-root`](call-the-expansion-cache-preflight-on-the-resolved-root.md) and the residual named by [`correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called`](correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called.md). Population leftover relative to the three-document docs close, not a reopen of the frontend implementation.
