---
id: record-the-expansion-cache-root-policy-decision
title: Record the expansion cache root policy decision
status: done
priority: p2
dependencies: [choose-the-expansion-cache-root-policy]
related: [decide-the-expansion-cache-collection-schedule]
scopes: [contracts/decisions, contracts/integrations, contracts/navigation]
shared_scopes: [project/tickets]
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

**Correction — 2026-07-31, before the work started.** Tom accepted the complete boundary packet, recorded under "Accepted (2026-07-31)" in [`choose-the-expansion-cache-root-policy`](choose-the-expansion-cache-root-policy.md), so the ADR was written `accepted` rather than `proposed` — the shape ADR 0088 takes for an in-session ratification. The acceptance also released the `frontends.md` propagation this ticket gated on it, making both halves one change rather than a change and a remainder. The number, `applies_to`, and `evidence` are as stated.

**Correction — 2026-07-31.** The count above is wrong and is corrected here rather than propagated: `docs/research/cache/root-policy.md`'s "Alternatives eliminated" section carries **seven** eliminations, not six — the home-directory default with no override, environment-only, `CARGO_MANIFEST_DIR`, the target directory, a manifest key, macro syntax, and per-driver detection. The same undercount appears in that note's citing text and is left for whoever holds `research/cache`; `docs/open-questions.md` is corrected in this change.

**The contract propagation is the second half.** `docs/integration/frontends.md`'s Compiler cache section says "A default macOS user cache is used rather than consumer `OUT_DIR`. A documented override supports CI and sandboxed builds." That sentence stays true and becomes specific: the exact variable, the exact derived path, the disable value, and the refusal behaviour. Do not perform that edit before Tom accepts the spelling — a contract stating an unaccepted variable name is a contract that has to be corrected.

## Public boundary for Tom

The consumer-visible surface is Tom's under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) and is **unaccepted**: the variable spelling `TILER_EXPANSION_CACHE_DIR`, the disable value `off`, the derived path `$HOME/Library/Caches/ai.moderately.tiler/expansion`, and the refusal text a consumer reads. `choose-the-expansion-cache-root-policy` reported the exact packet. Drafting a *proposed* ADR does not need his acceptance; propagating into an accepted contract does.

## Outcome — 2026-07-31

**[ADR 0089](../docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) — "Resolve the expansion-cache root from an override or the user cache", `decision_status: accepted`.** It cites Tom's 2026-07-31 packet acceptance, `applies_to: ["tiler.contract.frontend-integration"]`, `evidence: ["tiler.research.cache.root-policy"]`, `depends_on: ["ADR-0050", "ADR-0082", "ADR-0088"]`, `implementation_status: partial`. Nine decision items: the two-input derivation and total override-first precedence; the `$HOME/Library/Caches/ai.moderately.tiler/expansion` default and why `Library/Caches` makes it private by construction; verbatim override with the exact `off` sentinel; the empty-override refusal; the five-tree non-private refusal with its `1777` measurement basis *and* the stated boundary of what a path alone can decide; the missing-`HOME` refusal with the preflight counter-argument preserved rather than dropped; the four typed refusal shapes, whose exact text is cited rather than copied a third time; the two-variable observation bound stated as the structural one-root-for-both-drivers guarantee; and the root's deliberate absence from cache identity. Alternatives-considered compresses all seven eliminations with each one's ground. The `frontends.md`-versus-Proposal reconciliation is cited as already resolved in the research note and is not reopened.

**Catalog and chronology.** `docs/decisions/README.md`: the theme entry sits in "Artifacts, build, and toolchains" (matching `catalog_group: artifacts-build-toolchains`, where 0082, 0083, 0085, and 0088 sit), alphabetically by title between 0079 and 0049; the chronology entry appends after 0088.

**Contract propagation.** `docs/integration/frontends.md`'s Compiler cache section keeps its two accepted shape sentences verbatim and continues into a new paragraph stating the exact variable, the verbatim rule, the `off` sentinel, the derived default, override-first total precedence, the empty-override refusal, the five refused trees with `$TMPDIR` explicitly surviving, the underivable-`HOME` refusal, and the root's absence from the key — citing ADR 0089. The status preamble gains one precise clause so the section is not read as delivered behaviour: the resolver exists and nothing calls it. Frontmatter `evidence` gains `tiler.research.cache.root-policy`.

**`docs/backends/metal.md` needs no edit, and the reasoning is the decision.** Its sentence — "The default lives in an OS-appropriate user cache with a CI/sandbox override, rather than consumer `OUT_DIR`" — is true clause by clause under the accepted policy, so nothing there went stale. Restating the derivation in a second accepted contract would create two authorities over one subject, which is the failure that document already avoids by deferring the filesystem contract to the supported-filesystems note rather than restating it; and the chooser is a *frontend* decision, which is why ADR 0089's `applies_to` names the frontend contract alone. `docs/backends/**` is `contracts/artifacts`, which this ticket does not hold, so an optional one-line cross-reference is reported to the coordinator rather than taken.

**The ADR 0088 stale sentence — corrected in place with a dated note, not left alone and not rewritten.** 0088's Consequences bullet still reads exactly as accepted; a `**Correction — 2026-07-31.**` paragraph is appended under it recording that `tiler-macros` now names a root (it still opens no cache), that ADR 0089 holds the policy, and that Q-ART-004 is narrower than the bullet leaves it. The judgement: 0088's own convention that "an admission record holds what was true when it was accepted" is an argument against *substituting* text, and this correction substitutes none. It is not an argument for leaving a present-tense claim — "today", "remains the open question of record" — standing after it became false, because the corpus gives a reader no way to date that clause, and the cost is a reader redoing a decision that has been made. The corpus's own style supports the appended form: ADR 0075 carries a dated "Resolved 2026-07-24" paragraph inside its own Open questions, and `docs/status.md` and the build-tool exercise note both carry dated corrections.

**Sweep.** `docs/status.md`'s cache-root bullet said the spelling was unaccepted and undecided; it now records the acceptance, cites ADR 0089 and the contract, and keeps the wiring gap as the live gap. `docs/open-questions.md` Q-ART-004's root half now reads closed with the acceptance and the ADR, and its "Close" line records the root half closing on 2026-07-31 while the question stays open on the collection half. The collection half's bullet is untouched.

**Coordinator-owed, outside this ticket's scopes.** `research/cache` — `docs/research/cache/root-policy.md`'s frontmatter still reads `disposition: "pending"` and carries no `adopted_by`, which ADR 0089 now makes wrong on both counts (with the matching line in `docs/research/README.md`, which is in this ticket's scope, left saying "pending" so the catalog keeps matching the frontmatter it is a view over); its Status line and closing "An unaccepted public boundary" bullet still say the spellings are unaccepted; and its "six eliminated alternatives" citations in `docs/research/cache/build-tool-exercise.md` undercount the seven that note actually carries. `implementation/frontend` — `crates/tiler-macros/src/cache_root.rs`'s "# Reviewed draft" module section says the names "must not be treated as an accepted surface until Tom accepts the exact spelling", which is now false; the `#![allow(dead_code, reason = …)]` text is still accurate.

## Closes when

ADR 0089 exists with `decision_status: proposed`, cites the research record as evidence, and is listed in the decisions catalog and chronology; and the frontend contract states the exact policy once Tom has accepted the spelling, or a bounded remainder ticket holds that half if he has not.

## Graph maintenance

- **One sentence in an accepted record went stale and is left for whoever holds `contracts/decisions` to judge.** [ADR 0088](../docs/decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md)'s Consequences say "`tiler-macros` neither opens a cache nor names a root today, and Q-ART-004 remains the open question of record." The first clause is now half wrong — it names a root and still opens no cache — and the second is narrower than it was. ADR 0088's own convention is that an admission record holds what was true when it was accepted, so this may be correctly left alone; deciding that is a `contracts/decisions` judgement and was not made from a branch that could not edit the file. **Judged 2026-07-31:** corrected in place with a dated appended note, original sentence untouched — the reasoning is in the Outcome above.
- Do not absorb Q-ART-004's accounting and GC half; [`decide-the-expansion-cache-collection-schedule`](decide-the-expansion-cache-collection-schedule.md) holds it.
- If Tom rejects a spelling, the correction lands in `tiler-macros` under `implementation/frontend`, which this ticket does not hold — file it rather than taking the scope.
