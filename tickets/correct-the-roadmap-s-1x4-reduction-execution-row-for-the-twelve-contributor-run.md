---
id: correct-the-roadmap-s-1x4-reduction-execution-row-for-the-twelve-contributor-run
title: Correct the roadmap's 1x4 reduction execution row for the twelve-contributor run
status: done
priority: p3
dependencies: []
related: [separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reductions, documentation]
---
## Fact audit at `c0b2f06b`, before any edit

Every claim below was re-read in full at this base rather than taken from this ticket's own text or from the dispatch brief.

| Fact | Verdict | Evidence |
| --- | --- | --- |
| The paragraph lives in a `docs/roadmap.md` section called "Operation and dtype support" | **false** | `grep -c "Operation and dtype support" docs/roadmap.md` returns **0**. `grep -n "^## " docs/roadmap.md` lists fourteen sections and the paragraph sits under `## Operation-family support matrix`. Dtype support is a *different* document, `docs/dtype-support.md`, which this very paragraph links to and calls "the separate dtype support maturity ledger" precisely so the two slices are not read as one. Repaired above. |
| The paragraph states the `1x4` clause | **verified** | Anchor `all three retained reduction strategies were emitted, linked, dispatched, and bit-compared at a `1x4` shape`, `grep -cF` returns 1. |
| It "attributes every dispatch in the section" to `32023.883` | **imprecise** | The sentence reads `Every one of those dispatches is the same host row` — *those*, meaning the contraction toy, the `w_decode_kv` cell, and the `1x4` reduction it has just enumerated. The same paragraph then names a **different** toolchain row for the SiLU clause (`the Xcode 27.0 / Metal 32023.921 row the structural pair already sat on`), so it never claimed one row for the whole section. What was actually wrong is narrower and is what the correction fixes: the enumeration itself had not gained the `1x12` run, so a reader counting executed reduction shapes off this paragraph got one. |
| The related ticket landed a second executed shape at `1x12`, all three alternatives, on the `32023.921` row | **verified** | `docs/correctness-and-testing.md`, anchor `the separating case has now been driven on hardware`; and the producing ticket's own measurement table under `### The measurement`, which states the host row and calls it "a different offline-compiler row from the ticket's 2026-08-02 `32023.883` measurement". |
| Tree `6` of `2` returning `0x3f800001`, split `4` of `3` returning `0x3f800000` | **verified, and re-verified against the 2026-08-08 rule change** | Both figures are pinned device-free in `crates/tiler-conformance/src/serial_sum/tests.rs` at `separating_tree_partition` and `separating_split_partition`, and the returned bits at `assert_eq!(tree, vec![0x3f80_0001]);` / `assert_eq!(split, vec![0x3f80_0000]);`. Neither moved: see the correction below. |
| The four-contributor sentence's "both parallel strategies `0x3f800001`" is a property of the narrower shape | **verified** | At `1x12` the split returns `0x3f800000`, the fold's value; `docs/correctness-and-testing.md`'s anchor `the split's answer there coincides with the serial fold's` states the same asymmetry from the other side. |
| `docs/correctness-and-testing.md` already carries the measurement and its boundary in full | **verified** | Two paragraphs plus the operand-set discipline paragraph, all read. |
| The producing ticket declared `implementation/runtime`, `implementation/conformance`, and `contracts/numerics` | **verified** | Its frontmatter reads exactly `scopes: [implementation/runtime, implementation/conformance, contracts/numerics]`. Its repair block also filed this ticket by name and cited `docs/roadmap.md:435`, which is stale as a line citation — the paragraph is at 451 at this base, which is why the correction above cites anchors. |

## Correction to the dispatch brief — twelve is not where the two *tree* rules diverge

**The brief's premise is false, and it matters because it was the stated reason to expect this row to have moved.** It asserted that "twelve is specifically where the old and new rules first diverge", meaning the superseded "largest admissible count not exceeding the cap" and the landed "admissible count nearest the cap, ties to the narrower". Those two first diverge at **514** (`2 * 257`), not at twelve, and the compiler's own documentation says so: `capped_tree_partition`'s doc comment in `crates/tiler-compiler/src/physical.rs` derives the case under the anchor `` at 514 contributors (`2 * 257`) the only ``. A count above the cap must leave two contributors per partition, so the upward search is unreachable below 514 and empty at every count this ticket's shape ranges over — `crates/tiler-conformance/src/serial_sum.rs`'s `SEPARATING_COLUMNS` states exactly that, under the anchor `it is unreachable below 514`. **Both anchors are quoted at the width they actually grep**: each doc comment wraps, so the longer sentence a reader would naturally cite returns 0 hits and reads as absence.

What twelve *is* the smallest of is the divergence between the **tree's** rule and the **split's** `governed_partition`, which is a different pair. The already-landed [`restate-the-tree-width-rule-outside-the-compiler-crate`](restate-the-tree-width-rule-outside-the-compiler-crate.md) audited this at `97282def` and recorded it: "the tree takes 6 partitions of 2 at twelve contributors under both the old rule and the new one, and the split takes 4 of 3". So the 2026-08-08 landing touches nothing in this row, and the figures above are unmoved rather than luckily unchanged.

## The stale sentence, and what falsifies it

**Repaired 2026-08-08 — this named the section "Operation and dtype support", which `docs/roadmap.md` has never had.** The wording is substituted rather than dated beside, because `git log -S 'Operation and dtype support' -- docs/roadmap.md` is empty: the string was never in the file, so there is no point-in-time reading it records. The correct section is `## Operation-family support matrix`, and the retired phrase conflated it with the separate `docs/dtype-support.md` ledger that the paragraph itself exists to keep distinct.

`docs/roadmap.md`'s operation-family support matrix opens with a wide-support paragraph — the one beginning "Wide operation support is a durable project goal" — which states that under `NumericalContract::FLUSH_AND_REASSOCIATE_F32` "all three retained reduction strategies were emitted, linked, dispatched, and bit-compared **at a `1x4` shape**", and closes its enumeration with "Every one of those dispatches is the same host row — … offline Metal compiler `32023.883`". **Repaired 2026-08-08 — that clause read "attributes every dispatch in the section", which over-states it.** The sentence is scoped by *those* to the three dispatches it has just listed, and the same paragraph names a different toolchain row (`the Xcode 27.0 / Metal 32023.921 row the structural pair already sat on`) a few clauses later, so the paragraph never made a section-wide row claim. The defect is that its enumeration had not gained the `1x12` run, which is narrower and is what the correction fixes.

**Fact, 2026-08-07.** [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md) landed a second executed shape. All three retained alternatives were emitted, linked, dispatched, and bit-compared at **`1x12`** as well, from `crates/tiler-conformance`'s `serial_sum` vertical, on **Apple M4 Max / Apple9 / macOS 27.0 build `26A5388g` / offline compiler `Apple metal version 32023.921` / SDK `macosx 27.0` build `26A5388f`** — a different offline-compiler row from the `32023.883` the paragraph names. At that shape the single-workgroup tree published 6 partitions of 2 and returned `0x3f800001` while the multi-pass split published 4 of 3 and returned `0x3f800000`, each matched against its own declared grouping; the four-contributor sentence's "both parallel strategies `0x3f800001`" is a property of the narrower shape and not of the strategies.

`docs/correctness-and-testing.md` already carries the new measurement and its boundary in full. This ticket is the navigation ledger catching up, not a second authority for the numbers.

## Why it was not folded into the landing branch

`docs/roadmap.md` is `contracts/navigation`; the producing ticket declared `implementation/runtime`, `implementation/conformance`, and `contracts/numerics`. Widening that branch to take a fourth scope for one paragraph would have put a navigation-catalog edit inside an evidence branch, so the producing ticket's own "Scopes were under-declared" repair block deliberately left this out and asked for it to be filed.

## Closes when

The wide-support paragraph names both executed reduction shapes, `1x4` and `1x12`, states the offline-compiler row each is bounded to rather than one row for the whole section, and no longer implies that all three strategies agree at every executed shape. Read the paragraph in full before editing: it carries several other corrected-in-tense claims whose shape is load-bearing.

## What landed

One paragraph in `docs/roadmap.md`, one file, no other path touched. The two clauses named above are **dated beside rather than substituted**, because `git log -S` puts both in `c4e1ed63` on 2026-08-06 and the `1x12` run is `9c46b5ae` on 2026-08-07: each was true when it was written, so the correction is an addition in tense and the retired readings stay in place. The added clause, anchored at `Corrected 2026-08-08 — a second reduction shape has executed`, states the second executed shape and its producing ticket, bounds the `32023.883` sentence to the dispatches it enumerates, gives the `1x12` run its own compiler row (`Apple metal version 32023.921`, SDK `macosx 27.0` build `26A5388f`, same Apple M4 Max / Apple9 / `26A5388g` host), records that the tree returned `0x3f800001` where the split and the fold both returned `0x3f800000`, and says that neither run is evidence about which strategy is faster.

**How the figures were obtained.** None was derived arithmetically. The bit patterns and both declared partitions are read from the device-free pins in `crates/tiler-conformance/src/serial_sum/tests.rs` (`separating_tree_partition`, `separating_split_partition`, `assert_eq!(tree, vec![0x3f80_0001]);`, `assert_eq!(split, vec![0x3f80_0000]);`) and cross-checked against `docs/correctness-and-testing.md`, which this ticket names as the authority. The toolchain row is quoted from that document and from the producing ticket's own measurement table.

**What was deliberately not restated.** The tree's participant-selection rule, its `s <= 509` arithmetic bound, its exhaustive `0..4_096` population, and the `Unknown` cost direction all stay where they are owned — `crates/tiler-compiler/src/physical.rs`, `docs/compiler/fusion-and-scheduling.md`, and `docs/correctness-and-testing.md`. A roadmap paragraph that carried them would put a measured claim and an unmeasured one in one sentence.

## Sibling census — five neighbours read, three verified unchanged, two filed

`grep -on "prototype execution row[s]*\|execution row[s]*\|same host row\|backend-executed\|executed on a backend" docs/roadmap.md` returns **eleven** hits across **five** lines. Each was read.

- The wide-support paragraph itself — corrected above.
- The rung-ladder correction paragraph (anchor `every rung of this ladder now carries a record and no capability`) and the tensor-contraction family row, both of which say `two prototype execution rows` about the **contraction** — *verified unchanged*, since nothing about a reduction touches them.
- The reductions-beyond-strict-sum row — *verified unchanged*. Its own rule, anchored at `are physical and never semantic under ADR 0012`, is why a second executed shape moves no rung in this table, and it is the rule the correction cites.
- The strict serial sum row and the pointwise `f32` row, which share the cell text `R7 bounded to checked target-neutral layers and one prototype execution row` — **both now undercount**, since `crates/tiler-conformance/src/serial_sum.rs` was added on 2026-08-07 and dispatches both families on the `32023.921` row. Out of this ticket's deliverable and filed as [`refresh-the-navigation-execution-row-census-for-the-conformance-crate-device-runs`](refresh-the-navigation-execution-row-census-for-the-conformance-crate-device-runs.md).

**One neighbour outside `docs/roadmap.md`, found because the phrase `1x4` is not confined to it.** `grep -rn "1x4\|1x12" docs/*.md docs/compiler/*.md` returns `docs/correctness-and-testing.md` (the authority, current) and `docs/status.md`, which is the same `contracts/navigation` scope. `docs/status.md`'s device-execution census — anchor `device execution, and it is four runs rather than one` — enumerates four runs and omits both 2026-08-07 device executions, one of which that same document records as a Measurement in its own authoritative-profile bullet. Filed in the same ticket.
