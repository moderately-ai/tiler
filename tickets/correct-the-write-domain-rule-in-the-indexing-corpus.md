---
id: correct-the-write-domain-rule-in-the-indexing-corpus
title: Correct the write-domain rule in the indexing corpus
status: todo
priority: p2
dependencies: []
related: [admit-sub-range-write-domains-for-unequal-partitions, state-the-oracle-boundary-for-sub-domain-write-roots, scope-the-concatenate-fusion-role-and-lowering]
scopes: [research/indexing, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, indexing, corpus]
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
