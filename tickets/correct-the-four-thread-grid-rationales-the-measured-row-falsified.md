---
id: correct-the-four-thread-grid-rationales-the-measured-row-falsified
title: Correct the four-thread grid rationales the measured row falsified
status: todo
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

- `prototypes/serial-sum-compile/src/main.rs:136-137` — `ROWS`: "a deliberately conservative four-thread compile guarantee — the macOS 26.5 SDK contract proves that extent representable and states no maximum at all".
- `prototypes/serial-sum-compile/src/main.rs:199,210-211` — `CONTRACTION_M`: "`M * N <= 4` is the entire shape budget", and "the L3 profile's own cells are refused at this bound", quoting `required: Threads(1024), available: Threads(4)`. **Those cells are now reachable**, which is the whole point of `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`; coordinate with it rather than editing past it.

**`implementation/runtime`**

- `prototypes/serial-sum-run/src/proof.rs:230-238` with `PARALLEL_ROWS = 1` at `:5059` — "The authoritative profile's `GridAxisThreads` row admits four threads… a second row makes it eight and the whole compilation fails `target.grid-axis`".
- `prototypes/serial-sum-run/src/proof.rs:5052-5056` — "stays inside the declared four-thread grid guarantee".
- `prototypes/serial-sum-run/src/proof.rs:4434` — printed run text naming "this profile's four-thread grid-axis row". This one reaches a reader as program output rather than as a comment, so it is the highest-severity item here.

**`research/program-planning`**

- `spikes/program-planning/reduction-crossover/README.md:40,48,62,64` — the retained sweep's README states "The profile's `GridAxisThreads` row is 4", quotes the superseded declaration comment verbatim, and says rerunning it is what reports the new domain. **The rerun has been done** and is recorded on the originating ticket: 24 of 36 shapes retain all three strategies, with zero grid-axis refusals, against 1 and 23 before. Retaining a new results directory beside the 2026-08-02 one belongs here or to `calibrate-and-activate-parallel-reduction-selection`; the 2026-08-02 record itself is a retained measurement and must not be rewritten.
- `spikes/program-planning/reduction-crossover/src/main.rs:57` — the contributor-ladder comment reasoning from the old edge.

**`contracts/numerics`**

- `docs/correctness-and-testing.md:210` — "Discriminating those two needs a contributor count at which their declared partitions differ, which this shape's grid-axis bound does not reach." It reaches now, which is the activation trigger for `separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`. Correct the sentence without disturbing the surrounding retained measurement, which stays true of the `1x4` case it describes.

## Non-goals

Moving the grid-axis row again, re-running the extent ladder, or retaining a second measurement. The row and its authority are settled by the originating ticket.

## Closes when

Every site above states what is true now or is deliberately removed, no document outside the originating ticket's scopes still asserts a four-thread grid capacity for the authoritative profile, and `rg -n 'four-thread|four threads' crates/ prototypes/ spikes/ docs/` returns nothing that refers to this row.
