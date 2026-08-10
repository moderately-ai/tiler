---
id: correct-the-two-participant-residue-s-smallest-count
title: Correct the two-participant residue's smallest count
status: in-progress
priority: p3
dependencies: []
related: [restate-the-tree-width-rule-outside-the-compiler-crate, bound-the-tree-cap-s-unmeasured-downward-direction, measure-the-tree-width-excursion-past-the-cap, complete-the-tree-cap-audit-sweep-inside-the-compiler-crate]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, documentation]
claimed_from: todo
assignee: terra-two-participant-residue
lease_expires_at: 1786411317
---
## Per-Fact audit — 2026-08-10, at base `7b2e245190f9b680c7825041252030ead602e71a`

| Ticket Fact | Verdict | Evidence |
| --- | --- | --- |
| The live `capped_tree_partition` paragraph says “The smallest is 1,042” and thereby conflates the two populations. | **verified** | `crates/tiler-compiler/src/physical.rs`, anchor `The rule does not chase that direction to the end`, has that antecedent. The independent enumeration below returns four as the smallest two-participant count and 1,042 as the first such count with a wider admissible choice. |
| The figures 1,133, 1,176, 1,042, 521, 265, and 254 are right. | **verified** | The reproducible enumeration recorded below returns 1,133 and 1,176; its first current two-participant counts are `4, 1042, 1046, 1082, 1094, 1114, 1126, 1138`, and its first current two-participant count with a wider divisor is 1,042. At 1,042 the admissible counts are 2 and 521, so the distances from 256 are 254 and 265. |
| Outside tickets, only the live compiler comment and the external contract contain this residue. | **false, repaired below** | The broad source search also finds retained research-audit reports under `docs/research/documentation/ticket-audit-2026-08-10/` and an unrelated spike TSV containing the numeral. The live consumer-facing source sites are still only `physical.rs` and `docs/compiler/fusion-and-scheduling.md`; the latter is already correct. |

Reproduction (an independent transcription of the current nearest-to-256 and superseded truncating rules):

```sh
awk 'function old(n, c, p, limit) { if (n < 4) return 0; c = (n / 2 < 256 ? int(n / 2) : 256); for (p = c; p >= 2; --p) if (n % p == 0) return p; limit = int(sqrt(n)); for (p = c + 1; p <= limit; ++p) if (n % p == 0) return p; return 0 } function current(n, c, p, below, limit) { if (n < 4) return 0; c = (n / 2 < 256 ? int(n / 2) : 256); below = 0; for (p = c; p >= 2; --p) if (n % p == 0) { below = p; break }; if (!below) { limit = int(sqrt(n)); for (p = c + 1; p <= limit; ++p) if (n % p == 0) return p; return 0 }; for (p = 257; p < 512 - below && p <= int(n / 2); ++p) if (n % p == 0) return p; return below } BEGIN { for (n = 0; n < 20000; ++n) { now = current(n); prior = old(n); if (now == 2) { ++now_two; if (now_two <= 8) now_first = now_first (now_first ? ", " : "") n; wider = 0; for (p = 3; p <= int(n / 2); ++p) if (n % p == 0) { wider = 1; break }; if (wider && !first_declining) first_declining = n }; if (prior == 2) { ++old_two; if (old_two <= 8) old_first = old_first (old_first ? ", " : "") n } } print "current_two_count=" now_two " first=[" now_first "]"; print "superseded_two_count=" old_two " first=[" old_first "]"; print "first_current_two_with_wider=" first_declining; print "n=4 current=" current(4) " old=" old(4); print "n=1042 current=" current(1042) " old=" old(1042) " divisors=[2, 521]" }'
```

It prints `current_two_count=1133 first=[4, 1042, 1046, 1082, 1094, 1114, 1126, 1138]`, `superseded_two_count=1176 first=[4, 514, 526, 538, 542, 554, 562, 566]`, and `first_current_two_with_wider=1042`.

## The claim, and why it is wrong

**Fact — read in full at `97282def`.** `crates/tiler-compiler/src/physical.rs`'s `capped_tree_partition` doc comment closes with the residue paragraph, anchor `` The rule does not chase that direction to the end ``:

> Below 20,000 contributors, 1,133 counts still take two participants, against 1,176 before. The smallest is 1,042 (`2 * 521`), where 521 is admissible, representable, and inside the qualified entry's workgroup width, and the rule still declines it because 521 is 265 above the cap while 2 is 254 below.

"The smallest" refers back to "1,133 counts", and the smallest of those is **four**, not 1,042. Four contributors admit exactly one participant count — two — so the tree takes it with nothing to decline. Enumerating `capped_tree_partition` over `0..20_000` gives the two-participant set as `{4, 1042, 1046, 1082, 1094, 1114, 1126, 1138, …}`: four, then a gap of more than a thousand.

**The counts themselves are right.** 1,133 and 1,176 both reproduce, and 1,042 is correct for the claim the sentence is *trying* to make — it is the smallest count at which the rule takes two participants **while declining an admissible wider one**. Below 1,042 every count taking two has no alternative to decline, and 1,042 is where the first one appears: `2 * 521` admits only `{2, 521}`, and 521 sits 265 above the cap where 2 sits 254 below.

**Inference — why the gap is that wide, which is what makes the correction worth stating rather than deleting.** A count takes two participants only when nothing in `3..=509` divides it. That forces `n = 2m` with `m` odd, `m` free of every prime factor at or below 509, and `m > 509`. The smallest such `m` is the prime 521, giving 1,042; the smallest *composite* one is `521^2`, giving `n = 542,882`. So between four and 1,042 the residue is genuinely empty, and a reader who wonders why it jumps is being told something real rather than reading a typo.

## What this owes

- The sentence corrected in `crates/tiler-compiler/src/physical.rs` so it names both populations without an implicit antecedent: four is the smallest count taking two participants; 1,042 is the first count where the rule takes two while declining an admissible wider count.
- No change to any figure, constant, or rule. 1,133, 1,176, 1,042, 521, 265, and 254 all verify by enumeration.
- **Corrected population audit — one live source site remains.** The live consumer-facing source sites are `physical.rs` and the already-correct `docs/compiler/fusion-and-scheduling.md` restatement. A broad repository search also finds retained research-audit reports and unrelated data containing these numerals, so it is not a source-site census. [`measure-the-tree-width-excursion-past-the-cap`](measure-the-tree-width-excursion-past-the-cap.md) already carries a 2026-08-09 correction naming four and 1,042 separately. This board audit added the same dated distinction to the historical Outcome of [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md). Do not edit either completed record again; only the live compiler comment remains false.

## Sequencing

[`complete-the-tree-cap-audit-sweep-inside-the-compiler-crate`](complete-the-tree-cap-audit-sweep-inside-the-compiler-crate.md) owns the prime fallback-loop subject in `pipeline/tests.rs` and the private target-domain naming in `target.rs`, and only verifies (does not re-edit) the upward/downward wording already on `capped_tree_partition`. Land these sequentially rather than dispatching them together as `implementation/compiler` scope-collision hygiene, not as a shared-file edit race on the residue paragraph. This ticket owns the residue paragraph; the completed-record corrections are already present.

## Closes when

The live source sentence names four as the smallest two-participant count and 1,042 as the first count that declines an admissible wider choice; the external compiler contract remains semantically unchanged because it is already correct; and the existing dated ticket-record corrections remain intact.

## How it was found

Filed 2026-08-08 by the worker on [`restate-the-tree-width-rule-outside-the-compiler-crate`](restate-the-tree-width-rule-outside-the-compiler-crate.md), which held `contracts/optimizer` and `contracts/numerics` but not `implementation/compiler`. That ticket restated the residue in `docs/compiler/fusion-and-scheduling.md` the correct way rather than copying this sentence, which is how the defect surfaced: the figures were regenerated from an independent enumeration instead of transcribed.

## Outcome — 2026-08-10

Corrected the one live compiler comment: four is the smallest count taking two participants, while 1,042 is the first where the rule takes two while declining an admissible wider count. The audit also narrowed the stale broad-search claim to the live compiler and contract sites after identifying retained audit reports and unrelated data as additional string matches. The independent enumeration recorded above reproduces 1,133, 1,176, 1,042, 521, 265, and 254. No rule, figure, external contract, or completed record changed.
