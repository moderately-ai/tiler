---
id: record-the-expansion-cache-root-policy-decision
title: Record the expansion cache root policy decision
status: todo
priority: p2
dependencies: [choose-the-expansion-cache-root-policy]
related: [decide-the-expansion-cache-collection-schedule]
scopes: [contracts/decisions, contracts/integrations, contracts/navigation]
shared_scopes: []
paths: []
tags: [cache, frontend, adr]
---
## User-visible outcome

The expansion cache's root policy is a numbered decision a reader finds in the ADR index, and the frontend contract states the exact derivation rather than only its shape.

## Why this exists

[`choose-the-expansion-cache-root-policy`](choose-the-expansion-cache-root-policy.md) made the choice, implemented it as a crate-private draft in `tiler-macros`, and recorded the elimination in [`docs/research/cache/root-policy.md`](../docs/research/cache/root-policy.md). It could not write the ADR: `docs/decisions/[0-9]*.md` is the `contracts/decisions` scope and that ticket holds `implementation/frontend`, `research/cache`, and `contracts/navigation` only. Writing one anyway would have been a scope escape, so the boundary is recorded here rather than crossed there.

## Implementation keys

**Fact — the decision and its rejected alternatives are already written.** `docs/research/cache/root-policy.md` carries the derivation, the precedence, the disable value, the world-writable-tree refusal with its 2026-07-31 mode measurements, six eliminated alternatives with the ground each fails on, the measurement boundary, and five unsupported cases. The ADR restates the decision and cites that note as `evidence`; it does not re-derive it.

**Fact — one apparent conflict is resolved in that note and the ADR must not reopen it.** `docs/research/cache/build-tool-exercise.md` Section 3 proposes that the root "must be made explicitly rather than defaulted into a home directory", while accepted `docs/integration/frontends.md` and `docs/backends/metal.md` both state that the default *is* an OS user cache with a CI/sandbox override. The accepted contracts win under `AGENTS.md`'s authority order; what the proposal protects — no unstated, undocumented, unrefusable root — is preserved by stating the derivation exactly.

**The ADR is `proposed`, not accepted.** Its number is 0089 unless a higher one has landed by then; `applies_to` is `tiler.contract.frontend-integration`; `evidence` names `tiler.research.cache.root-policy`. Add it to `docs/decisions/README.md`'s catalog and chronology in the same change, per the conventions there.

**The contract propagation is the second half.** `docs/integration/frontends.md`'s Compiler cache section says "A default macOS user cache is used rather than consumer `OUT_DIR`. A documented override supports CI and sandboxed builds." That sentence stays true and becomes specific: the exact variable, the exact derived path, the disable value, and the refusal behaviour. Do not perform that edit before Tom accepts the spelling — a contract stating an unaccepted variable name is a contract that has to be corrected.

## Public boundary for Tom

The consumer-visible surface is Tom's under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) and is **unaccepted**: the variable spelling `TILER_EXPANSION_CACHE_DIR`, the disable value `off`, the derived path `$HOME/Library/Caches/ai.moderately.tiler/expansion`, and the refusal text a consumer reads. `choose-the-expansion-cache-root-policy` reported the exact packet. Drafting a *proposed* ADR does not need his acceptance; propagating into an accepted contract does.

## Closes when

ADR 0089 exists with `decision_status: proposed`, cites the research record as evidence, and is listed in the decisions catalog and chronology; and the frontend contract states the exact policy once Tom has accepted the spelling, or a bounded remainder ticket holds that half if he has not.

## Graph maintenance

- **One sentence in an accepted record went stale and is left for whoever holds `contracts/decisions` to judge.** [ADR 0088](../docs/decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md)'s Consequences say "`tiler-macros` neither opens a cache nor names a root today, and Q-ART-004 remains the open question of record." The first clause is now half wrong — it names a root and still opens no cache — and the second is narrower than it was. ADR 0088's own convention is that an admission record holds what was true when it was accepted, so this may be correctly left alone; deciding that is a `contracts/decisions` judgement and was not made from a branch that could not edit the file.
- Do not absorb Q-ART-004's accounting and GC half; [`decide-the-expansion-cache-collection-schedule`](decide-the-expansion-cache-collection-schedule.md) holds it.
- If Tom rejects a spelling, the correction lands in `tiler-macros` under `implementation/frontend`, which this ticket does not hold — file it rather than taking the scope.
