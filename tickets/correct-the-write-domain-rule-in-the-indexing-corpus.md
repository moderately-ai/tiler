---
id: correct-the-write-domain-rule-in-the-indexing-corpus
title: Correct the write-domain rule in the indexing corpus
status: in-progress
priority: p2
dependencies: []
related: [admit-sub-range-write-domains-for-unequal-partitions, state-the-oracle-boundary-for-sub-domain-write-roots, scope-the-concatenate-fusion-role-and-lowering]
scopes: [research/indexing, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, indexing, corpus]
claimed_from: todo
assignee: agent-write-domain
lease_expires_at: 1786042934
---
## User-visible outcome

Two corpus documents stop asserting a refusal the code no longer performs, so a reader planning around the concatenate lowering is not told the write-domain rule still forecloses it.

## Why this exists

**Fact — two documents state the equality rule as a current fact, and both are load-bearing where they say it.**

- `docs/research/indexing/concatenate-fusion-role-and-lowering.md:97` — "`crates/tiler-ir/src/index/builder.rs:1308-1310` — `prepare_access` returns `IndexBuildError::InvalidWriteDomain` unless a write's domain set equals the region's complete parallel dimension set. A write cannot iterate a sub-range, and two writes in one region share one iteration domain, so an operand-sized write and a result-sized write cannot coexist there." This is item 2 of the record's refusal list, which is the evidence its fork elimination rests on.
- `docs/open-questions.md:276` — the Q-SHAPE-006 bullet lists `index/builder.rs:1308` `InvalidWriteDomain` as one of four sites at which the partitioned write is refused.

**Fact — [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) changed both the rule and the line numbers.** A write's domain may now be any subset of the region's parallel dimensions; `InvalidWriteDomain` survives meaning only "this domain names a non-parallel dimension". Two writes in one region no longer share one iteration domain, which is the exact clause the research record uses to conclude the operand-sized and result-sized writes cannot coexist.

**Inference — the conclusions those documents draw are unaffected; only the premises are stale.** The fork elimination selected the partitioned write and this relaxation is what makes it expressible, so the corrections are to the refusal list rather than to the verdict. Q-SHAPE-006's restated trigger — the first family whose *read* map is genuinely case-split over one tensor — does not move either.

## What the work is

Correct both passages to the current rule, keeping the historical claim legible as history rather than deleting it: each was true at the commit it cites, and a reader tracing the elimination needs to see what was refused when the decision was made. Re-read the cited sites before rewriting, because the line numbers moved with the change.

Check the same documents for the neighbouring claim that the partition's members must own equal shares, which was the corollary of the equality rule and is what the relaxation removed.

## Explicit non-goals

- The tickets that state the old rule as a dated fact about a past commit. Those are history and read correctly as such.
- `crates/tiler-reference/src/oracle.rs`'s two doc blocks, which are prose stating the same premise but are code-adjacent and belong to [`state-the-oracle-boundary-for-sub-domain-write-roots`](state-the-oracle-boundary-for-sub-domain-write-roots.md), which must change the behaviour in the same commit as the doc.

## Closes when

Both passages state the current rule with correct line references, and no document in `docs/` asserts that a write must iterate the complete parallel dimension set.

## Graph maintenance

- `research/indexing` for the research record and `contracts/navigation` for `docs/open-questions.md`, read from `ticketsplease.toml`'s scope map.
- Filed by [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md), whose scopes are `implementation/ir` and `project/tickets` and which could reach neither document.

## Outcome — 2026-08-06, executed by the coordinator (session subagent limit reached)

**Both passages state the current rule, and the drift was wider than this ticket recorded — two of the four refusal sites moved, not one.** Re-reading the cited sites at base `59e93632` before rewriting, as instructed, found that beside the `InvalidWriteDomain` equality rule this ticket names, the construction-time `DuplicateOutputTensor` refusal is gone entirely: `grep -rn DuplicateOutputTensor crates/` returns nothing, `output()`'s doc (`crates/tiler-ir/src/index/builder.rs:1909-1927`) states that several roots may name one output tensor, and the partition obligations are discharged at verification under `OutputPartitionUncovered`, `OutputPartitionRangesOverlap`, and `OutputPartitionDoubleWritten`, decided by `decide_partition_by_interval` (`builder/proof.rs:1122`, dispatched at `:329`). The exhaustive ownership walk now governs only a sole-owner root (`owns_alone`, `proof.rs:271`); `MultipleWriters` (`program/verify.rs:203`) is unchanged in meaning.

**The research record** keeps its four-item list as the evidence the elimination ran against — each item was true at the record's base — with the heading and intro moved to past tense and one dated correction paragraph after the list stating each site's current state with current line references. Three neighbouring repairs the re-read forced: the "sentence being corrected" paragraph's "currently reads" claim (stale since the transfer executed) moved to past tense; the transfer-note sentence now scopes its byte-identity claim to the transfer event, since this ticket's correction of the landed bullet makes the blockquote a transfer record rather than the bullet's current text; and reproducible check 3, whose positive control ("the first returns both names") had gone false, now checks the current state and uses the empty `DuplicateOutputTensor` grep as an observed-discharge control.

**The navigation bullet** (`docs/open-questions.md`, Q-SHAPE-006) is rewritten in place to current truth: the write-ownership contract the surviving alternative "owes" has landed, the two construction-time refusals are discharged, and the sole-owner and program-layer rules are stated as what remains. The restated trigger is untouched, as the ticket's inference predicted.

**The equal-shares corollary population is zero.** `grep -rn 'equal shares|equally sized|own equal|one iteration domain' docs/` finds no surviving assertion outside the corrected line; the only statements of the corollary are in the relaxation ticket's own Outcome, which is dated history and this ticket's stated non-goal. `crates/tiler-reference/src/oracle.rs`'s two doc blocks were left per the non-goal — they belong to `state-the-oracle-boundary-for-sub-domain-write-roots` (done; its landing carried them).

**Checks.** `tkt lint` ok; `git diff --check` clean; `git diff --name-only` = the two docs plus this ticket; `tkt guard --base 59e93632` run from the branch.
