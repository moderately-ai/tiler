---
id: correct-the-four-thread-grid-rationales-the-measured-row-falsified
title: Correct the four-thread grid rationales the measured row falsified
status: done
priority: p2
dependencies: [establish-an-upper-bound-authority-for-the-metal-grid-axis-row]
related: [calibrate-and-activate-parallel-reduction-selection, raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells]
scopes: [implementation/frontend, implementation/metal-aot, implementation/runtime, research/program-planning, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, defect, target-profiles]
---
## User-visible outcome

No comment or document outside the crates that landed the measured grid-axis row still tells a reader the authoritative Metal profile admits four threads, so a fixture's small shape is not read as a capacity limit that no longer exists.

## Why this exists

**Fact.** `establish-an-upper-bound-authority-for-the-metal-grid-axis-row` moved `FIRST_MACOS_APPLE9`'s `grid_axis_threads` from `4` to a measured `268_435_456`. Nothing broke: the full workspace run was green at 2,470 tests, because every site below is prose rather than an assertion, and every fixture stays feasible under a wider bound. That is exactly why it needs its own ticket — a stale comment costs a reader rather than a gate, and each of these explains *why a shape is small* by citing a capacity that is now four million times larger.

The originating ticket held `implementation/build`, `implementation/compiler`, `research/target-profiles`, `contracts/optimizer`, and `contracts/navigation`, and corrected every site inside them. It did not absorb these silently and did not widen into five further scopes on a landing that also carried an identity-domain step. Two of the scopes below were held by a live ticket at that time.

## The exact sites

Enumerated so this ticket is executable without rediscovering them. Each is a doc comment or prose paragraph asserting a four-thread grid capacity or reasoning from one; none is an assertion.

**`implementation/frontend`**

- `crates/tiler-macros/src/aot/tests.rs:1111-1113` — `narrower_region`: "the bound declaration's measured grid-axis capacity is four threads, so an extent of eight has no feasible plan at all". An extent of eight is now feasible, so the stated reason for choosing a *narrower* region is gone; check whether the test still needs a second distinct program and say why.
- `crates/tiler-macros/src/aot/tests.rs:1340-1351` — `split_region`: "four is also the largest the bound declaration's measured grid-axis capacity admits", and records `[1, 8]` and `[2, 4]` as `NoFeasiblePlan`. Both now compile and retain all three strategies; only the `[1, 5]` `InvalidCompilerOutput` note survives, and that defect is itself closed.
- `crates/tiler-macros/src/region/tests.rs:522-524` and `:964-966` — "the bound declaration's measured grid-axis capacity is four threads" / "the most the bound declaration's measured grid-axis capacity admits".
- `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs:244-245` and `:275-278` — the same rationale in the consumer-facing facade test, including "wider or taller shapes are refused before any plan exists".

**`implementation/metal-aot`**

- `prototypes/serial-sum-compile/src/main.rs:136-137` — `ROWS`: "a deliberately conservative four-thread compile guarantee — the macOS 26.5 SDK contract proves that extent representable and states no maximum at all". **Still owed.**
- ~~`prototypes/serial-sum-compile/src/main.rs:199,210-211`~~ — `CONTRACTION_M`. **Corrected** by `publish-an-l3-contraction-cell-through-the-accepted-route`, which had to: that ticket publishes `w_decode_kv` as a second contraction member, so "the L3 profile's own cells are refused at this bound and are not published here" became false about this producer's own output rather than only about the row. The rewritten text says `2x2` is small because it is *discriminating* — a result with more than one row and more than one column separates the two operand access relations — and points at the new member for why it was not repointed.

**`implementation/runtime`**

- `prototypes/serial-sum-run/src/proof.rs:230-238` with `PARALLEL_ROWS = 1` — "The authoritative profile's `GridAxisThreads` row admits four threads… a second row makes it eight and the whole compilation fails `target.grid-axis`". **Still owed.**
- `prototypes/serial-sum-run/src/proof.rs`, `PUBLISHED_ROWS` — "stays inside the declared four-thread grid guarantee". **Still owed.**
- ~~`prototypes/serial-sum-run/src/proof.rs:4434`~~ — printed run text. **Corrected** by `publish-an-l3-contraction-cell-through-the-accepted-route`, for the same reason and with more force: the sentence claimed "the L3 profile's own cells are refused by this profile's four-thread grid-axis row and are not published here" in a run that had just dispatched one. The summary now names how many members were routed and how many had the SHA-256 of their executed bytes compared against a retained realization-probe measurement.

**Line numbers in the two `prototypes/` entries above are pre-2026-08-05 and have shifted**; that landing inserted several hundred lines into `proof.rs`. Locate the two remaining sites by the constants they document — `PARALLEL_ROWS` and the test module's `PUBLISHED_ROWS` — rather than by line. `rg -n 'four-thread|four threads' prototypes/` returned exactly three hits after it landed: those two, and `ROWS` in the producer.

**`research/program-planning`**

- `spikes/program-planning/reduction-crossover/README.md:40,48,62,64` — the retained sweep's README states "The profile's `GridAxisThreads` row is 4", quotes the superseded declaration comment verbatim, and says rerunning it is what reports the new domain. **The rerun has been done** and is recorded on the originating ticket: 24 of 36 shapes retain all three strategies, with zero grid-axis refusals, against 1 and 23 before. Retaining a new results directory beside the 2026-08-02 one belongs here or to `calibrate-and-activate-parallel-reduction-selection`; the 2026-08-02 record itself is a retained measurement and must not be rewritten.
- `spikes/program-planning/reduction-crossover/src/main.rs:57` — the contributor-ladder comment reasoning from the old edge.

**`contracts/numerics`**

- `docs/correctness-and-testing.md:210` — "Discriminating those two needs a contributor count at which their declared partitions differ, which this shape's grid-axis bound does not reach." It reaches now, which is the activation trigger for `separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`. Correct the sentence without disturbing the surrounding retained measurement, which stays true of the `1x4` case it describes.

## Non-goals

Moving the grid-axis row again, re-running the extent ladder, or retaining a second measurement. The row and its authority are settled by the originating ticket.

## Closes when

Every site above states what is true now or is deliberately removed, no document outside the originating ticket's scopes still asserts a four-thread grid capacity for the authoritative profile, and `rg -n 'four-thread|four threads' crates/ prototypes/ spikes/ docs/` returns nothing that refers to this row.

## Outcome — 2026-08-05

### The enumeration was reconciled, not trusted

`rg -n 'four-thread|four threads' prototypes/` at the base returned **exactly the three hits this ticket predicted** — `proof.rs:232` (`PARALLEL_ROWS`), `proof.rs:5495` (`PUBLISHED_ROWS`), `main.rs:141` (`ROWS`) — confirming the two sites the L3-publication landing corrected are gone. Line numbers had drifted as warned; every site was located by its constant.

Each corrected comment now gives the reason that survives the moved row rather than losing its purpose:

- `narrower_region` — **the second distinct program is still needed**, because `a_semantically_wrong_entry_is_a_typed_refusal_rather_than_a_silent_rebuild` publishes its envelope under the *approved* region's subject and asserts the two envelopes differ first. The extent stays at two for cost — each program is a real cold `xcrun metal` compilation — not for capacity.
- `split_region`, `splitting_serial_sum_region`, and the facade's `[rows: 1, cols: 4]` — four contributors is the **smallest** count `governed_partition` splits, so this is the smallest shape whose selected plan splits rather than the only one. The recorded `NoFeasiblePlan` / `InvalidCompilerOutput` refusals are named as falsified, and *which* wider shapes also select a split is left to `calibrate-and-activate-parallel-reduction-selection` rather than asserted unmeasured.
- `serial_sum_region` and the facade's `2x2` — the smallest shape at which the reduction is observable: more than one contributor so the fold has an order, more than one row so the dropped axis is distinguishable from a rank-zero collapse.
- `ROWS` (producer) and `PUBLISHED_ROWS` (runner) — the surviving reason is the pinned cross-crate pair: the runner asserts `PUBLISHED_ROWS != ROWS` so that a runner substituting its own row count for the artifact's declared one stays detectable. Raising either would erase the check.
- `PARALLEL_ROWS` — the grouping-sensitive case enumerates one row's orderings and a `const` assertion beside it stops the build if the value moves; both operand sets are one row of four values.

### `docs/correctness-and-testing.md` — the ticket's own premise was wrong, and was checked rather than relayed

This ticket said the moved row "reaches now, which is the activation trigger for `separate-the-tree-and-split-groupings-…`". **It is not.** That deferral's 2026-08-04 trigger check log records the trigger as *not fired*, and re-running its two commands confirms it: `governed_partition(contributors)` is read at `crates/tiler-compiler/src/physical.rs:1329` (`single_workgroup_tree_region`) and `:1513` (`split_reduction_regions`), and `workgroup_tree_tile` fixes `rounds: 1` at `crates/tiler-ir/src/schedule/cooperative.rs:887`. The tree and the split declare identical partitions at **every** contributor count, so widening the grid axis moves the reachable count and not the divergence. The corrected sentence says that instead of blaming the bound, and the deferral's own stale present-tense premise ("the row *is* `4`") was corrected in the same change with a dated log line.

### The reduction-crossover retention decision: not here

The 2026-08-05 rerun (24/36 shapes retaining all three, zero grid-axis refusals) is recorded in the README as an unretained count with its provenance, and **no second `results/` directory is added**, on four grounds:

1. `establish-an-upper-bound-authority-for-the-metal-grid-axis-row` already decided it — "recording a new result under `spikes/program-planning/` belongs to the calibration ticket" — and that is a merged outcome, not a suggestion.
2. The spike's own frontmatter names `calibrate-and-activate-parallel-reduction-selection` as its `ticket:`, and the 2026-08-02 directory is that ticket's retained evidence.
3. This ticket's Non-goals forbid retaining a second measurement.
4. A retained results directory is a positive claim carrying an execution environment and provenance; attaching one to a documentation-correction ticket that ran no harness would be a measurement with no measurer. The README states the absence and why, so it does not read as an oversight.

The 2026-08-02 record is untouched. What changed around it is tense and scope: the derivation section is marked as holding under the row as it stood, and the Boundary bullet predicting that shapes above four work items "will still be refused on the grid axis" is marked falsified.

### Straggler sweep — the enumeration was not exhaustive

`rg -n 'four.thread|four threads|Threads\(4\)|grid_axis_threads.*4\b' crates/ prototypes/ spikes/ docs/` returned **25 hits**, and a second sweep on the falsified *consequence* rather than the phrase (`retains all three|exactly one shape|only window|NoFeasiblePlan`) found more that the first missed. Reconciliation:

- **In scope, corrected** — the enumerated sites, plus `spikes/program-planning/reduction-crossover/src/main.rs`'s `CONTRIBUTORS` ladder rationale, whose "counts at and just above four locate the upper edge" was falsified along with the edge.
- **Correct as written, historical** — `crates/tiler-build/src/metal_plan.rs:1213,1219,1400,1465`, `docs/compiler/fusion-and-scheduling.md:379-383`, `docs/research/target-profiles/…-ledger.md:85,291`, `prototypes/serial-sum-compile/src/main.rs:239` (the `L3_CELL_M` doc, which says the cell "*was* unreachable while the row read four"). Each names the row as superseded.
- **Correct as written, different profile** — every "four" in `crates/tiler-compiler` outside the two sites below refers to `TargetProfileBuilder::governed`, the target-neutral prototype baseline, whose row was deliberately not moved. Also `crates/tiler-build/src/metal_declaration.rs:1564`, a deliberate perturbation.
- **Falsified but out of every scope this ticket holds** — `crates/tiler-compiler/src/physical.rs:1308-1313`, `crates/tiler-compiler/src/frontier.rs:3109-3112` and `:3175-3178` (all three point at `governed_partition` "for the derivation and the row that blocks it" while that function's own doc already records the row moving — a live self-contradiction), `docs/integration/frontends.md:17,497`, `docs/open-questions.md:161`, and `spikes/runtime/inline-dispatch/README.md:236,345` with `src/multi_entry.rs:132-138`. Filed as [`correct-the-four-thread-rationales-outside-the-corrected-scopes`](correct-the-four-thread-rationales-outside-the-corrected-scopes.md) rather than absorbed — the same choice the originating ticket made when it filed this one. Note `spikes/runtime/**` maps to `research/runtime`, not to the `implementation/runtime` this ticket holds.

### Scopes

No scope was added. The five declared scopes covered every site corrected here; the sites they did not cover became the filed ticket rather than a widening.
