---
id: complete-the-tree-cap-audit-sweep-inside-the-compiler-crate
title: Complete the tree cap audit sweep inside the compiler crate
status: todo
priority: p3
dependencies: []
related: [cap-the-tree-reduction-participants-at-the-measured-256]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [doc-drift, test-coverage]
---
## Why this exists

The post-landing audit of [`cap-the-tree-reduction-participants-at-the-measured-256`](cap-the-tree-reduction-participants-at-the-measured-256.md) produced findings in two places. Everything outside `crates/tiler-compiler/` was fixed inline by the coordinator on 2026-08-07. The three items below are **inside** that crate, which a live claim held at the time, so they were filed rather than raced. Each was independently verified, not relayed.

## The three items

**1. A test assertion that cannot reach the branch its comment names.** `crates/tiler-compiler/src/pipeline/tests.rs`, in `the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs`:

```rust
// A prime count admits nothing, in this branch as in the other.
assert_eq!(crate::physical::capped_tree_partition(65_537), None);
```

For 65,537 the ceiling is `min(256, 32_768) = 256` and `isqrt(65_537) = 256`, so the above-cap loop's guard `candidate <= limit` is false immediately and **the loop body executes zero times**. The `None` comes from falling out of both loops, not from the fallback branch. The smallest prime whose fallback loop actually iterates is **66,067** (`isqrt = 257`, one iteration) — verified by computing both quantities. Replace the constant and keep the comment. The composite case at `257 * 257` does exercise the branch and is fine.

*This assertion was added by the coordinator at review, so the ticket that landed the cap is not at fault for it.*

**2. A claim in `target.rs` that the landing made false.** `three_strategy_domain`'s doc was strengthened to "**Every condition** is read from the code that decides it rather than restated: `governed_partition` is what withholds the split, `capped_tree_partition` is what withholds the tree … and the grid-axis bound …". On the profile this function is used for, `TargetProfile::governed()` declares `local_memory_bytes(0)` and no synchronization realization — which is exactly why `tree_target()` has to override both. So the tree is withheld there at **every** shape by the local-memory row, which `three_strategy_domain` does not read. The same landing states this in the other file: `physical.rs` says "the prototype baseline declares zero and refuses every tree at every width". Two files in one landing say opposite things about one profile. Narrow the claim to the conditions it actually reads, or read the others.

**3. An overclaim to re-scope, owned elsewhere but stated here for the sweep.** `capped_tree_partition`'s "the rule is the calibrated one" covers a domain the calibration never measured. The substance belongs to [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md); do **not** duplicate the fix here. If that ticket lands first, check only that no wording in this crate still overstates.

## Also noted, deliberately not required here

Three assertions in the same test cannot fail — the in-loop `partitions <= MEASURED_TREE_PARTICIPANT_CAP` over `0..4_096` (the above-cap branch needs 66,049), an `assert_ne!` implied by two equalities above it, and `257 > 256`. None is harmful and the test's two counted populations (3,530 and 2,561) do carry its weight. Tighten them only if touching those lines anyway.

## A distinct gap, recorded so it is not lost

**No tree is executed at any width the cap changed.** The only test that runs a tree through the interpreter covers extents 8 and 6 — both counts where the two rules agree, both with two contributors per participant. The new test calls `verify_schedule` but never lowers or interprets. So there is no executed evidence for a tree at a changed width, including the higher fan-in shapes the cap now produces (8,192 → 32 per participant). Extent 12 through the existing `KirMachine` harness would close it **without hardware**, and it is distinct from both the filed local-memory band and the hardware gap in [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md). Whoever takes this ticket should either close it or split it out rather than leave it in prose.

## Closes when

Items 1 and 2 are fixed, item 3 is confirmed consistent with whatever the owning ticket decided, the device-free execution gap is closed or split into its own ticket, and the package's checks pass.

## Graph maintenance

Filed 2026-08-07 by the coordinator, holding the compiler-crate half of an audit sweep whose other half landed the same day. The split is a scope-collision artefact, not a judgement that these matter less.
