---
id: prove-the-governed-tag-tables-injective
title: Prove the governed tag tables injective
status: todo
priority: p2
dependencies: [derive-the-artifact-numerical-and-fenced-space-populations]
related: [prove-the-exhaustible-encoder-injectivity-claims-natively]
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [verification, identity, injectivity, evidence-upgrade]
---
## User-visible outcome

Every governed `tag()` table reached only by an *inexhaustible* identity encoder is backed by an exhaustive pairwise-distinctness test over its whole variant set, so a duplicated tag literal fails the build's gate instead of silently folding two operations, address spaces, or authorities onto one identity.

## Why this exists (found while proving the exhaustible encoders, 2026-08-07)

**Fact.** `prove-the-exhaustible-encoder-injectivity-claims-natively` classified every canonical-identity *encoder* in `tiler-ir` and `tiler-artifact` and landed exhaustive injectivity tests for the 19 whose whole input domain is enumerable. Tag tables reached by one of those encoders are covered by it. The artifact round-trip authority is the source anchor `fn every_governed_tag_table_round_trips`; its payload-carrying numerical populations are complete only after [`derive-the-artifact-numerical-and-fenced-space-populations`](derive-the-artifact-numerical-and-fenced-space-populations.md), so this ticket must read that landed result rather than repeating the old blanket seven-table claim.

**Fact — current syntactic census, not yet the owed-set classification.** At `94ee4730`, `rg -n 'fn tag' crates/tiler-ir/src crates/tiler-artifact/src --glob '*.rs'` finds 65 functions: 54 in `tiler-ir` and 11 in `tiler-artifact`. One artifact hit is the generic codec-decoder helper `fn tag<T>(...)`, leaving **64 enum/tag methods**. Some are already covered by the 19 whole-encoder tests or by a complete `from_tag` inverse; the remainder are the tables this ticket owes. The old “about 50” estimate and the directory counts below predate later vocabularies and are not a complete current manifest.

**Historical starting population — must be reclassified, not copied into tests.** The filing inventory named the kernel, program/ABI, numerics, schedule, shape, semantic, index, and ten artifact tables, including `FactAuthority` and `FactValidityScope`. A current directory census is 12 kernel, 10 program/ABI, 7 numerics, 11 schedule, 5 shape, 7 semantic, and 2 index methods, plus the same ten artifact methods: 64 total. The historical `semantic/ 8` and `index/ 3` counts are false at this base. Before editing, produce an exact manifest that marks each of the 64 as (a) already covered by an exhaustive encoder, (b) covered by a complete left inverse, or (c) owed here, and read every table in category (c) in full. This classification—not the syntactic count—is the closing population.

**Inference.** `FactAuthority` and `FactValidityScope` deserve first attention: both assign tags deliberately *out of declaration order*. The source anchors `The tags are deliberately not in declaration order` and `` `MeasuredEnvironment` carries `0x05` `` state why, and this is exactly the shape where a hand-checked literal table is easiest to get wrong and hardest to spot in review.

## The work

1. Re-derive the exact 64-method manifest above and record why every method is already covered or owed. For each owed table, enumerate its variants with an array sized by `core::mem::variant_count` so a widened vocabulary is a build error at the list. `#![cfg_attr(test, feature(variant_count))]` is already declared in both crates.
2. Assert the tags are pairwise distinct and count the population walked, so a shrunk enumeration fails rather than passing vacuously.
3. Where a `from_tag` inverse exists, also assert the round trip and that every unclaimed byte refuses — several already do this and need only the population guard.
4. Watch each new check fail on a planted duplicate literal before trusting it.
5. Do not weaken the existing round-trip tests; these sit beside them.

## Closes when

Every table in the enumeration above has a passing exhaustive distinctness test with a `variant_count`-guarded population, each watched failing on a planted duplicate tag, and any table deliberately left out is named with its reason.
