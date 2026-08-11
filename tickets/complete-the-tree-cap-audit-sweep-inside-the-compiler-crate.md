---
id: complete-the-tree-cap-audit-sweep-inside-the-compiler-crate
title: Complete the tree cap audit sweep inside the compiler crate
status: in-progress
priority: p3
dependencies: []
related: [cap-the-tree-reduction-participants-at-the-measured-256, correct-the-two-participant-residue-s-smallest-count]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [doc-drift, test-coverage]
claimed_from: todo
assignee: terra-tree-cap-audit
lease_expires_at: 1786410519
---
## Why this exists

The post-landing audit of [`cap-the-tree-reduction-participants-at-the-measured-256`](cap-the-tree-reduction-participants-at-the-measured-256.md) produced findings in two places. Everything outside `crates/tiler-compiler/` was fixed inline by the coordinator on 2026-08-07. The three items below are **inside** that crate, which a live claim held at the time, so they were filed rather than raced. Each was independently verified, not relayed.

## Per-Fact audit — 2026-08-10 at `fbf7f32ea8093e01a53c226f3c27cb9664f91813`

| Ticket Fact | Verdict | Evidence |
| --- | --- | --- |
| `65_537` reaches the fallback with no loop iteration, while prime `66_067` reaches its above-cap search for exactly one iteration. | **verified** | In `crates/tiler-compiler/src/physical.rs`, anchor `while candidate <= limit`, the search starts at `ceiling + 1` and is guarded by the integer square root. `factor 65537 66067` reports both prime; `256^2 = 65,536 < 65,537`, so its floor root is 256 and candidate 257 fails the guard. `257^2 = 66,049 < 66,067 < 66,564 = 258^2`, so its floor root is 257 and candidate 257 is checked once. An independent enumeration finds `66,049 = 257^2` as the first *composite* with any fallback iteration; `66,067` remains the first prime, as the Fact says. |
| `three_strategy_domain` overclaims that it reads every feasibility condition. | **verified** | `crates/tiler-compiler/src/target.rs`, anchors `Every condition is read` and `fn three_strategy_domain`, show only both partition predicates and the grid-axis inequality. The same file's `workgroup_tree_target_for_test` doc, anchor `local-memory-bytes\` as *zero*`, says the governed baseline declares no local memory and no synchronization realization, both of which the helper does not consult. |
| The upward/downward evidence separation already landed and needs no duplicate edit. | **verified** | `crates/tiler-compiler/src/physical.rs`, anchors `Upward: empirical evidence, one host` and `Downward: two claims, deliberately not sharing a sentence`, both remain. |

The audit found no false Fact that changes this ticket's purpose, authority, public boundary, or identity surface.

### Review correction — 2026-08-10 at `699ad6c4`

Independent review found the first repair's watched failure incomplete: its
arithmetic preconditions prove that `66_067` *should* enter the loop, but an
early production `return None` immediately before `let limit =
contributors.isqrt()` leaves every assertion green. Item 1 therefore needs a
test-only, thread-local count recorded inside the actual above-cap loop; the
test must clear it, observe zero for `65_537`, and observe exactly one candidate
check for `66_067`. This changes neither the production rule nor any identity or
public surface.

## The three items

**1. A test assertion that cannot reach the branch its comment names.** `crates/tiler-compiler/src/pipeline/tests.rs`, in `the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs`:

```rust
// A prime count admits nothing, in this branch as in the other.
assert_eq!(crate::physical::capped_tree_partition(65_537), None);
```

For 65,537 the ceiling is `min(256, 32_768) = 256` and `isqrt(65_537) = 256`, so the above-cap search guard `candidate <= limit` is false immediately and **the loop body executes zero times**. The function does enter the fallback branch; the stale claim was that its search ran. The smallest prime whose fallback loop actually iterates is **66,067** (`isqrt = 257`, one iteration) — verified by computing both quantities. Replace the constant and call the subject the above-cap *search*. The composite case at `257 * 257` already executes that loop and is fine.

*This assertion was added by the coordinator at review, so the ticket that landed the cap is not at fault for it.*

**2. A claim in `target.rs` that the landing made false.** `three_strategy_domain`'s doc was strengthened to "**Every condition** is read from the code that decides it rather than restated: `governed_partition` is what withholds the split, `capped_tree_partition` is what withholds the tree … and the grid-axis bound …". On the profile this function is used for, `TargetProfile::governed()` declares `local_memory_bytes(0)` and no synchronization realization — which is exactly why `tree_target()` has to override both. So the tree is withheld there at **every** shape by the local-memory row, which `three_strategy_domain` does not read. The same landing states this in the other file: `physical.rs` says "the prototype baseline declares zero and refuses every tree at every width". Two files in one landing say opposite things about one profile. Narrow the claim to the conditions it actually reads, or read the others.

**3. Completed before this sweep — verify, do not duplicate.** The old `capped_tree_partition` wording that called the whole rule calibrated is gone. Current source separates `Upward: empirical evidence, one host` from `Downward: two claims, deliberately not sharing a sentence`, and the downward ticket is done. Confirm those anchors remain and make no second correction here.

## Also noted, deliberately not required here

The earlier audit also over-counted the dead assertions and carried a stale census. The current loop checks the meaningful widened window `partitions <= 2 * MEASURED_TREE_PARTICIPANT_CAP - 2`; the separation population is **2,350**, not 2,561. The `assert_ne!` implied by the two exact widths and `257 > 256` remain redundant but harmless. Tighten them only if touching those lines anyway.

## A distinct gap, recorded so it is not lost

**The global execution gap is closed; a portable interpreter neighbour is optional coverage, not this ticket's blocker.** `crates/tiler-conformance` now drives the separating count `SEPARATING_COLUMNS = 12` through all three retained alternatives on the qualified device. `the_tree_and_the_split_round_differently_at_the_separating_count` observes the tree's 6×2 grouping and the split's 4×3 grouping producing different permitted bits, while `every_retained_alternative_computes_the_declared_contributor_set_at_the_separating_count` checks coverage. A `KirMachine` test at 12 could add host-independent evidence, but the ticket's statement that no changed width is executed anywhere is false and does not remain in the closing condition.

## Closes when

Items 1 and 2 are fixed, item 3's already-landed wording is confirmed unchanged, and the package's checks pass. No new interpreter or conformance work is required here.

## Graph maintenance

Filed 2026-08-07 by the coordinator, holding the compiler-crate half of an audit sweep whose other half landed the same day. The split is a scope-collision artefact, not a judgement that these matter less.
