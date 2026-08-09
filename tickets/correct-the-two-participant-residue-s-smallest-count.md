---
id: correct-the-two-participant-residue-s-smallest-count
title: Correct the two-participant residue's smallest count
status: todo
priority: p3
dependencies: []
related: [restate-the-tree-width-rule-outside-the-compiler-crate, bound-the-tree-cap-s-unmeasured-downward-direction, measure-the-tree-width-excursion-past-the-cap]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, documentation]
---
## The claim, and why it is wrong

**Fact — read in full at `97282def`.** `crates/tiler-compiler/src/physical.rs`'s `capped_tree_partition` doc comment closes with the residue paragraph, anchor `` The rule does not chase that direction to the end ``:

> Below 20,000 contributors, 1,133 counts still take two participants, against 1,176 before. The smallest is 1,042 (`2 * 521`), where 521 is admissible, representable, and inside the qualified entry's workgroup width, and the rule still declines it because 521 is 265 above the cap while 2 is 254 below.

"The smallest" refers back to "1,133 counts", and the smallest of those is **four**, not 1,042. Four contributors admit exactly one participant count — two — so the tree takes it with nothing to decline. Enumerating `capped_tree_partition` over `0..20_000` gives the two-participant set as `{4, 1042, 1046, 1082, 1094, 1114, 1126, 1138, …}`: four, then a gap of more than a thousand.

**The counts themselves are right.** 1,133 and 1,176 both reproduce, and 1,042 is correct for the claim the sentence is *trying* to make — it is the smallest count at which the rule takes two participants **while declining an admissible wider one**. Below 1,042 every count taking two has no alternative to decline, and 1,042 is where the first one appears: `2 * 521` admits only `{2, 521}`, and 521 sits 265 above the cap where 2 sits 254 below.

**Inference — why the gap is that wide, which is what makes the correction worth stating rather than deleting.** A count takes two participants only when nothing in `3..=509` divides it. That forces `n = 2m` with `m` odd, `m` free of every prime factor at or below 509, and `m > 509`. The smallest such `m` is the prime 521, giving 1,042; the smallest *composite* one is `521^2`, giving `n = 542,882`. So between four and 1,042 the residue is genuinely empty, and a reader who wonders why it jumps is being told something real rather than reading a typo.

## What this owes

- The sentence corrected in `crates/tiler-compiler/src/physical.rs` so it names both populations without an implicit antecedent: four is the smallest count taking two participants; 1,042 is the first count where the rule takes two while declining an admissible wider count.
- No change to any figure, constant, or rule. 1,133, 1,176, 1,042, 521, 265, and 254 all verify by enumeration.
- **False original population claim — the live source/docs surface is narrow, but two completed records also repeat the ambiguity.** The source search outside tickets returns only `physical.rs` and the already-correct `docs/compiler/fusion-and-scheduling.md` restatement. The complete records [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md) and [`measure-the-tree-width-excursion-past-the-cap`](measure-the-tree-width-excursion-past-the-cap.md) both say “The smallest is 1,042” after naming the 1,133-count population. Preserve their historical outcomes and add dated corrections that distinguish the smallest member, four, from the first member declining a wider admissible count, 1,042.

## Sequencing

[`complete-the-tree-cap-audit-sweep-inside-the-compiler-crate`](complete-the-tree-cap-audit-sweep-inside-the-compiler-crate.md) also edits `physical.rs`. Land these sequentially rather than dispatching them together. This ticket owns the residue paragraph and the two completed-record corrections; the sweep owns the prime fallback-loop subject and the private target-domain naming.

## Closes when

The live source sentence names four as the smallest two-participant count and 1,042 as the first count that declines an admissible wider choice; the external compiler contract remains semantically unchanged because it is already correct; and both completed ticket records carry dated corrections without rewriting their historical outcomes.

## How it was found

Filed 2026-08-08 by the worker on [`restate-the-tree-width-rule-outside-the-compiler-crate`](restate-the-tree-width-rule-outside-the-compiler-crate.md), which held `contracts/optimizer` and `contracts/numerics` but not `implementation/compiler`. That ticket restated the residue in `docs/compiler/fusion-and-scheduling.md` the correct way rather than copying this sentence, which is how the defect surfaced: the figures were regenerated from an independent enumeration instead of transcribed.
