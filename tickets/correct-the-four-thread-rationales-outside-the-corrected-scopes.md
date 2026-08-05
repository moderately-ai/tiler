---
id: correct-the-four-thread-rationales-outside-the-corrected-scopes
title: Correct the four-thread rationales outside the corrected scopes
status: todo
priority: p2
dependencies: []
related: [correct-the-four-thread-grid-rationales-the-measured-row-falsified, establish-an-upper-bound-authority-for-the-metal-grid-axis-row, calibrate-and-activate-parallel-reduction-selection]
scopes: [implementation/compiler, contracts/integrations, contracts/navigation, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, defect, target-profiles]
---
## User-visible outcome

No comment or document anywhere still tells a reader that the *authoritative* macOS Metal profile admits four grid-axis threads, or reasons from a consequence that bound produced — so the last three documents asserting "exactly one shape retains all three reduction strategies" stop contradicting the function whose doc comment already records that the row moved.

## Why this exists

**Fact.** `establish-an-upper-bound-authority-for-the-metal-grid-axis-row` moved `FIRST_MACOS_APPLE9`'s `grid_axis_threads` from `4` to a measured `268_435_456`, and `correct-the-four-thread-grid-rationales-the-measured-row-falsified` then corrected every falsified rationale inside `implementation/frontend`, `implementation/metal-aot`, `implementation/runtime`, `research/program-planning`, and `contracts/numerics`. Its straggler sweep found more sites than its own enumeration held, in four scopes it did not carry. It filed these rather than widening — the same choice the originating ticket made when it filed that ticket.

**These are not merely stale, they are self-contradicting.** `crates/tiler-compiler/src/physical.rs`'s `governed_partition` doc is already fully corrected: it carries a **Measurement, 2026-08-04** paragraph recording that the row moved and that the same inequality now admits a wide domain. Three doc comments in the same crate point *at that function* "for the derivation and the row that blocks it" while asserting the superseded conclusion. A reader following the link is told two different things.

## The exact sites

Enumerated so this ticket is executable without rediscovering them. None is an assertion; every one is prose.

**`implementation/compiler`**

- `crates/tiler-compiler/src/physical.rs:1308-1313` — `single_workgroup_tree_region`: "That measured evidence is not currently obtainable … one shape on the authoritative profile retains all three strategies, so no crossover exists to calibrate against. The participant count is doubly fixed as a result — the one measurable shape has four contributors, which this function splits into two partitions of two, so the workgroup width does not vary either." The participant count is no longer doubly fixed. What survives is that the *tree* and the *split* both read one `governed_partition`, so the width is fixed by that function rather than by a shape — which is the same fact `separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ` records as its surviving blocker.
- `crates/tiler-compiler/src/frontier.rs:3109-3112` (`propose_split`) and `:3175-3178` (`propose_workgroup_tree`) — identical two sentences in both: "That preference is still unassigned, and not for want of trying: on the authoritative profile exactly one shape retains all three strategies, so there is no crossover to measure." The preference *is* still unassigned, and that half is true; the reason given is not. Correct both together — they are the same claim in two places and fixing one would leave the other.

**Do not "correct" the governed-baseline mentions in this crate.** `crates/tiler-compiler/src/target.rs:1725`, `pipeline/conformance.rs:1039`, `pipeline/tests.rs:1647,3693`, `pipeline/trace.rs:1817`, `tests/composed_family_recognition.rs:85`, and `tests/contraction_direct_path.rs:81` all say "four" about `TargetProfileBuilder::governed`, the *target-neutral prototype baseline* keyed `tiler.prototype-target-neutral-baseline.v1`. That row was **deliberately not moved** — a macOS Apple9 device measurement is evidence about one target and cannot widen a baseline standing in for every target — and the originating ticket's outcome states so explicitly. They are correct as written.

**`contracts/integrations`**

- `docs/integration/frontends.md:497` — "**Boundary — one shape.** `[rows: 1, cols: 4]` is the only window that selects a split on the bound macOS declaration today; `[rows: 1, cols: 8]` and `[rows: 2, cols: 4]` are `NoFeasiblePlan` and `[rows: 1, cols: 5]` is `InvalidCompilerOutput`." All three refusals are falsified: the first two hit the superseded grid-axis row, and the third hit `correct-the-declined-strategy-record-for-an-unsplittable-reduction`, which is `done`. The surviving statement is that `[1, 4]` is the *smallest* shape whose selected plan splits — `governed_partition` needs two partitions of at least two — and that the measurement retained here was taken at that one shape.
- `docs/integration/frontends.md:17` — "the one shape it is measured at", in the same sense. It is still one shape *measured*; check whether the sentence reads as a capacity claim in context before editing it, and leave it if it does not.

**`contracts/navigation`**

- `docs/open-questions.md:161` — "the authoritative macOS profile's grid-axis bound is today a conservative representability *floor*, and a retained sweep measures that it collapses the parallel-reduction comparable domain to exactly one shape". Both clauses are false: the bound is a retained measurement at 268,435,456, and the owner ticket named in the same sentence is `done`. The question's live owners need re-deriving from the board rather than editing in place.

**`research/runtime`**

- `spikes/runtime/inline-dispatch/README.md:236` — "**Measurement — `[rows: 1, cols: 4]` is the window, not a taste.** Measured on `BoundMetalCompileDeclaration::first_macos_apple9` under this contract, `[rows: 1, cols: 8]` and `[rows: 2, cols: 4]` are refused as `NoFeasiblePlan` … and `[rows: 1, cols: 5]` is refused as `InvalidCompilerOutput`." Same three falsified refusals. **The retained measurement itself must not be rewritten** — the dispatch it records happened at `[1, 4]` on a named host and stays true; what needs correcting is the present-tense claim about why no other shape was available.
- `spikes/runtime/inline-dispatch/README.md:345` — "`[rows: 1, cols: 4]` under `flush_and_reassociate_f32` is the only window that selects a split on the bound declaration today".
- `spikes/runtime/inline-dispatch/src/multi_entry.rs:132-138` — `dispatch_region`'s doc, the same three refusals in source.

## Implementation keys

Preserve what each comment is *for*. Every one of these explains why a shape or a boundary is small; a shape chosen because it was the largest feasible is now merely the smallest useful, and the replacement must give the surviving reason rather than delete the sentence. The pattern to copy is `crates/tiler-build/src/metal_plan.rs`'s `reassociating_program`, which states the old reason as history and the surviving one as the reason.

Do **not** re-measure to fill a gap. Where the honest answer is that a wider shape's behaviour is now unmeasured, say so and name `calibrate-and-activate-parallel-reduction-selection` as its owner; a rerun of the reduction-crossover sweep and its retained result belong to that ticket.

## Closes when

Every site above states what is true now or is deliberately removed, no document asserts a four-thread grid capacity or a one-shape three-strategy domain for the authoritative profile, and `rg -n 'four-thread|four threads|exactly one shape|only window' crates/ prototypes/ spikes/ docs/` returns nothing that refers to the authoritative row rather than to the prototype baseline or to a dated retained record.
