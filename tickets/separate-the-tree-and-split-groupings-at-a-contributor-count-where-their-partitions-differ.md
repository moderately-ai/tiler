---
id: separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ
title: Separate the tree and split groupings at a contributor count where their partitions differ
status: done
priority: p3
dependencies: []
related: [drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies, raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells, calibrate-and-activate-parallel-reduction-selection, establish-an-upper-bound-authority-for-the-metal-grid-axis-row, bound-the-tree-cap-s-unmeasured-downward-direction, complete-the-tree-cap-audit-sweep-inside-the-compiler-crate]
scopes: [implementation/runtime, implementation/conformance, contracts/numerics]
shared_scopes: [project/tickets]
tags: [numerics, reductions, evidence-gap]
paths: []
---
## The activation trigger fired on 2026-08-07; this is now open work

**Filed `deferred` because the case could not be constructed at any shape the profile admitted. That ceased to be true when the tree took its measured participant cap** — see the dated entry at the end of this ticket for the verification. Discriminating contributor counts are now reachable, so the ticket is `todo`.

**What it still owes is narrower than what it was filed for.** The compiler now *produces* the divergence; what remains missing is the **case**, not the count. No grouping-sensitive operand set has been driven through both strategies at a discriminating shape on hardware, so the four-contributor run remains the corpus's only device observation of a different-but-permitted answer, and it still does not separate the tree from the split. Everything under "What this ticket would then owe" stands unchanged. "The gap, stated precisely" and "Trigger for reconsideration" below are retained as the dated record of what deferred this and how it was eliminated; read them as history, not as current truth.

## The gap, stated precisely

**Measurement, 2026-08-02, Apple M4 Max / macOS 27.0 `26A5388g` / `Apple metal version 32023.883` / `nightly-2026-07-19`.** [`drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies`](drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies.md) drove operands `0x3f400000, 0x3e800000, 0x33400000, 0x33000000` through all three alternatives on the qualified host:

| Alternative | Declared partition | Answer |
| --- | --- | --- |
| serial fold | 4 of 1 | `3f800000` |
| single-workgroup tree | 2 of 2 | `3f800001` |
| multi-pass split | 2 of 2 | `3f800001` |

Each matched **its own** declared grouping bit for bit, which is what made it the first observation in the corpus of a reassociation-permitting program producing a different-but-permitted answer.

**Fact — at four contributors the tree and the split declare the *same* partition, and still do.** As of 2026-08-02 both took it from `governed_partition`; since the tree's cap landed they reach it through different functions that agree at this count, and the cooperative tile's `rounds: 1` makes the tree's grouping identical to the split's either way. So that case separates the two parallel strategies from the serial fold, and **does not separate them from each other**. Nothing about that is a defect in the case; it is a property of the shape.

**Fact, as of 2026-08-02 — four contributors was the ceiling this profile admitted for the shape.** The declared Metal profile's `grid_axis_threads` row was `4`, and the split's pointwise stage launches one invocation per element, so a wider row failed `target.grid-axis` before any plan composed. That bound was taken to be what made a discriminating contributor count unreachable rather than merely unchosen. **It was not the cause.** The row is now a measured `268_435_456`, wider counts compose, and no discriminating count appeared — the 2026-08-04 entry in the trigger check log below records what actually blocks it.

## Trigger for reconsideration

**A contributor count at which `governed_partition` yields a different partition for the tree than for the split becomes reachable.** Two routes were named when this was filed. The first — [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md), widening the grid-axis row that then capped the shape at four — has landed and is **eliminated**: it moves the reachable count without moving the divergence. The second named route was a tile whose `rounds` exceeds one, making the tree's grouping differ from the split's at a count the profile already admits. The instruction that caught the first route's insufficiency — check which arrived before assuming it was the grid-axis row — is why this line is an elimination rather than a hope.

**Neither named route is what fired it.** A third arrived that this ticket did not anticipate: the two strategies stopped reading the same function. The trigger's wording above — phrased as `governed_partition` yielding different partitions for the tree and the split — presumes one function serves both, which is exactly the premise that has now gone. Read the trigger by its intent, which the body states throughout: *a reachable contributor count at which the two declared partitions differ*. `rounds` is still fixed at one and the multi-round route remains unexplored; it is simply no longer the only way here.

## What this ticket would then owe

- A contributor count where the two declared partitions genuinely differ, **read from each plan's own published launch geometry** rather than assumed — the existing case established that discipline and it is not to be relaxed.
- Operands grouping-sensitive at *that* count, stated as exact `f32` bit patterns.
- Each strategy's answer matched against **its own** declared grouping via `tiler_reference::strict_partitioned_sum`, not against a tolerance and not against a shared oracle. The existing case's elimination — why a derived bound and why permitted-set membership were both discarded — holds here and must not be re-litigated.
- The discriminating refusal watched firing: one strategy held to the *other's* partition must be refused, and the refused value must itself be legal under the contract. That is the wrong-but-in-range refusal, and it is the whole point of the exercise.

## Explicit non-goals

No cost model and no selection change — [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md) owns those. No new strategy. This ticket adds discriminating *evidence*; it does not widen the grid-axis row itself.

## Graph maintenance

- Filed 2026-08-02 at integration of the producing ticket, which recorded the limit rather than hunting around it and asked whether it should become a ticket. It should: a bounded capability with a named activation trigger belongs on the board rather than in a closed ticket's prose.
- Do **not** convert this to `todo` because the grid-axis ticket is merely claimed. The trigger is a *reachable* discriminating count, which means that work landed and the count verified reachable — check it, do not infer it.

## Trigger check log

- 2026-08-04 — **not fired, and the named concrete route landed and turns out to be insufficient.** The grid-axis row this ticket blamed *did* move: [`establish-an-upper-bound-authority-for-the-metal-grid-axis-row`](establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md) is `done` and the authoritative declaration now carries a measured `grid_axis_threads: 268_435_456` (`crates/tiler-build/src/metal_declaration.rs:225`, was `4`), so wider shapes are reachable and more than one shape now retains all three strategies. **That does not produce a discriminating count.** The ticket's own trigger is a count at which the *tree's* partition differs from the *split's*, and both still read the identical value from one function — `single_workgroup_tree_region` calls `governed_partition(contributors)` (`crates/tiler-compiler/src/physical.rs:1159`) exactly as `partial_reduction_region` does, and `workgroup_tree_tile` fixes `rounds: 1` (`crates/tiler-ir/src/schedule/cooperative.rs:887`), which is precisely the condition the ticket recorded as making the two groupings identical. Widening the row moves the count but not the divergence, at **every** count. The surviving route is therefore the ticket's second one alone: a cooperative tile whose `rounds` exceeds one. The ticket's own instruction — "check which arrived before assuming it was the grid-axis row" — is what caught this. Recheck: `grep -n 'governed_partition(contributors)' crates/tiler-compiler/src/physical.rs` and `grep -n 'rounds: 1' crates/tiler-ir/src/schedule/cooperative.rs`.
- 2026-08-05 — **not fired; the 2026-08-04 verdict re-verified by re-running its own commands.** `correct-the-four-thread-grid-rationales-the-measured-row-falsified` had to decide what `docs/correctness-and-testing.md` should say in place of "which this shape's grid-axis bound does not reach", so it checked rather than relayed. `governed_partition(contributors)` is read at `crates/tiler-compiler/src/physical.rs:1329` (inside `single_workgroup_tree_region`) and `:1513` (inside `split_reduction_regions`) — the previous entry's `:1159` is line drift, not a moved call — and `workgroup_tree_tile` still fixes `rounds: 1` at `crates/tiler-ir/src/schedule/cooperative.rs:887`. The two groupings remain identical at every count, and the body's stale grid-axis premise above was corrected in the same change. Recheck: `grep -n 'governed_partition(contributors)' crates/tiler-compiler/src/physical.rs` and `grep -n 'rounds: 1' crates/tiler-ir/src/schedule/cooperative.rs`.
- 2026-08-07 — **FIRED, by a route this ticket did not name.** Verified by the coordinator on the merged tree at `9415b450`, not relayed from the worker's report. [`cap-the-tree-reduction-participants-at-the-measured-256`](cap-the-tree-reduction-participants-at-the-measured-256.md) landed the tree's own participant rule, so the two strategies no longer read one function: `single_workgroup_tree_region` calls `capped_tree_partition(contributors)` (`crates/tiler-compiler/src/physical.rs:2547`) and `split_reduction_regions` calls `governed_partition(contributors)` (`:2733`). The previous two entries' recheck command, `grep -n 'governed_partition(contributors)' crates/tiler-compiler/src/physical.rs`, now returns **one** hit where it returned two, and that hit is the split. `rounds: 1` is unchanged at `crates/tiler-ir/src/schedule/cooperative.rs:891`, so the second named route is untouched and simply no longer required. **Discriminating counts, read off the two rules:** the smallest is **12** (tree 6 partitions of 2, split 4 of 3); at **8,192** the tree takes 256 of 32 against the split's 128 of 64; `tiler_compiler::pipeline::tests::the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs` pins 2,561 such counts below 4,096 and reads the tree's from the region's own declared cooperative topology. Moved `deferred` → `todo` by the coordinator at integration. **What fired is the count, not the case** — no hardware run has separated the two groupings, which is the whole of what this ticket still owes. Recheck: `grep -n 'capped_tree_partition(contributors)\|governed_partition(contributors)' crates/tiler-compiler/src/physical.rs` — two hits at two different functions is the fired state.

## Repaired before dispatch, 2026-08-07 — the ticket is satisfiable, but its scope was not

Verified by the coordinator against `crates/tiler-compiler/src/physical.rs`, the conformance crate, and `docs/correctness-and-testing.md`.

### The separating count is real, and so is an operand set for it

**Twelve contributors.** `capped_tree_partition` sets `ceiling = min(256, contributors / 2) = 6` and walks *down* for the largest divisor at or below it, giving `{partitions: 6, per: 2}`. `governed_partition` walks down from `isqrt(12) = 3` for the largest divisor at or below *that*, giving `{partitions: 4, per: 3}`. The two rules read **opposite ends of the divisor lattice**, which is why they diverge at all. Counts 4 through 11 agree (4, 6, 8, 9, 10 identical; 5, 7, 11 prime, both `None`), so 12 is genuinely the minimum. The cap changes the *choice*, never the domain — over 0..200,000 the two rules' decline sets are identical.

A grouping-sensitive operand set at 12 exists: the current four-contributor operands `0x3f400000, 0x3e800000, 0x33400000, 0x33000000` padded with eight `+0.0`, which in binary32 gives tree (6×2) `0x3f800001` against split (4×3) `0x3f800000` — the serial fold also `0x3f800000`. Both are order-preserving blocked regroupings, so each is legal under `FLUSH_AND_REASSOCIATE_F32`, which makes the fourth owed bullet — the wrong-but-in-range refusal — **non-vacuous**: holding the tree to the split's 4×3 refuses a value the contract permits.

**Caveat the worker must not skip:** zero-padding weakens the dropped-contributor detection `PARALLEL_OPERANDS` carries. The operand *pair* discipline needs a real 12-wide second set, not this one alone.

### Scopes were under-declared — this was the blocking defect

`implementation/runtime` covers only `crates/tiler-runtime/**` and `prototypes/serial-sum-run/**`. Two required files fell outside it, and `tkt guard` would have exited 6:

- **`crates/tiler-conformance/src/serial_sum.rs`** (`implementation/conformance`, now added) is the **authoritative device home**. `carry-the-device-executed-value-proof-into-the-conformance-crate` landed the grouping-sensitive case there under ADR 0106, and the file's module header states that "the corpus's only device observation of a different-but-permitted reassociated answer lives here." It is gated by `make full`; the prototype is `cargo run` only, so the prototype copy alone is not the deliverable. It fixes `pub(crate) const PARALLEL_COLUMNS: u64 = 4`. **Both line citations in this bullet were stale and are replaced by anchors, 2026-08-07 by the worker** — the header sentence was at `:61`–`:63` and not `:51`, and `PARALLEL_COLUMNS` at `:138` and not `:127`, read at this branch's own base `aebd16c0`. Neither drift changed what the bullet asserts; the anchors are what survive the next edit.
- **`docs/correctness-and-testing.md`** (`contracts/numerics`, now added) carries the standing claim this ticket's landing falsifies, in the paragraph beginning "**Measurement, and the boundary it does not exceed.**": "no grouping-sensitive operand set has been driven through both strategies at such a shape on hardware, so the four-contributor run above is still the corpus's only device observation". **The line citation was stale and is replaced by an anchor, 2026-08-07 by the worker** — the paragraph is line 221 and not 219 at base `aebd16c0`.

`docs/roadmap.md:435` ("at a `1x4` shape") is `contracts/navigation` and is **deliberately left out** — file it as a ledger update rather than widening this branch further.

### Two stale line citations in the 2026-08-07 trigger entry

`capped_tree_partition(contributors)` is called at **`physical.rs:2612`**, not `:2547`; `governed_partition(contributors)` at **`:2798`**, not `:2733`. Both drifted ~65 lines; the calls exist in the named functions. The definitions are at `:2219` and `:2128`. Everything else in that entry verifies, including the 2,561 divergent counts below 4,096 and the 8,192 case (tree 256×32 against split 128×64).

### The trigger log is spent

This ticket is `todo`. Its `## Trigger check log` and the instruction "do **not** convert this to `todo`" are **spent** and must be read as history — the recheck now permanently returns the fired state, which is a check that can never say no. Kept for provenance, binding on nobody.

## Closes when

The ticket had no closing gate; its de facto conditions sat under "What this ticket would then owe", written in the subjunctive while it was `deferred`. It closes when all four of those are discharged **in `crates/tiler-conformance`** (the gated home, not only the prototype), `docs/correctness-and-testing.md:219`'s claim is corrected to name what was driven and on what host, and the report states the contributor count, both declared partitions, both returned values, and the host and toolchain row the run is bounded to.

## Worker record, 2026-08-07 — all four owed bullets discharged in `crates/tiler-conformance`

Branch `tkt/separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`, base `aebd16c0`, work commit **`9c46b5aeed784234e684fd870770b6f8a50fa8ef`**. Closure is the coordinator's; this section records what was verified and measured.

### Per-Fact audit at this base, before any edit

Read in full: `AGENTS.md`, this ticket, `crates/tiler-compiler/src/physical.rs` (4,690 lines), `crates/tiler-conformance/src/{lib,serial_sum,measurement,portability}.rs` and `serial_sum/tests.rs`, `docs/correctness-and-testing.md`.

- **The two rules are read by two different functions.** *Verified.* `single_workgroup_tree_region` calls `capped_tree_partition(contributors)` and `split_reduction_regions` calls `governed_partition(contributors)`.
- **The repair block's replacement line citations, `physical.rs:2612` and `:2798`, definitions at `:2219` and `:2128`.** *Verified, all four exact at this base.* The repair block corrected the trigger entry's `:2547`/`:2733`, and its own numbers had not drifted further.
- **Twelve is the smallest separating count; tree `{6, 2}` against split `{4, 3}`.** *Verified, re-derived rather than adopted.* Counts 4, 6, 8, 9, 10 give both rules the same partition; 5, 7, 11 are prime and both decline; 12 is the first disagreement.
- **The cap moves the choice and never the domain.** *Verified* over `0..200_000`: the two rules' `None` sets are identical.
- **2,561 divergent counts below 4,096, and 8,192 giving tree 256×32 against split 128×64.** *Verified.*
- **`bound-the-tree-cap-s-unmeasured-downward-direction` may move the count set.** *Verified it has not:* that ticket is still `todo`, so `capped_tree_partition`'s rule at this base is the one the count was derived from.
- **The four-contributor operands padded with eight `+0.0` give tree `0x3f800001` against split `0x3f800000`.** *Verified in binary32, and then observed on hardware.*
- **`crates/tiler-conformance/src/serial_sum.rs` states the only-observation claim "at `:51`" and fixes `PARALLEL_COLUMNS = 4` "at `:127`".** *Both false as line citations* — `:61`–`:63` and `:138` respectively; repaired above to anchors. The claim itself was **imprecise about its home**: the sentence sits in the module header *and* on `tests::every_retained_alternative_rounds_the_way_its_declared_grouping_rounds`, and only the second is the case that carries it.
- **`docs/correctness-and-testing.md`'s standing claim is at line 219.** *False as a line citation* — the paragraph is 221; 219 is the declared-grouping-oracle paragraph. Repaired above to an anchor.
- **`docs/roadmap.md:435` is `contracts/navigation`.** *Verified*, and deliberately untouched; filed as [`correct-the-roadmap-s-1x4-reduction-execution-row-for-the-twelve-contributor-run`](correct-the-roadmap-s-1x4-reduction-execution-row-for-the-twelve-contributor-run.md).

### The measurement

**Measurement, 2026-08-07.** Host row: Apple M4 Max, GPU family Apple9, macOS 27.0 build `26A5388g`, `arm64`; offline compiler `Apple metal version 32023.921 (metalfe-32023.921)`, linker `AIR-LLD 32023.921`, SDK `macosx 27.0` build `26A5388f`; profile `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`, AOT target `air64-apple-macos26.0` under `metal4.0`, 25,493 `metallib` bytes; Rust toolchain `nightly-2026-07-19`. **This is a different offline-compiler row from the ticket's 2026-08-02 `32023.883` measurement.**

A `1x12` reduction under `FLUSH_AND_REASSOCIATE_F32`, all three retained alternatives emitted, linked, dispatched, and read back:

| Alternative | Declared partition, read from its own published geometry | Answer |
| --- | --- | --- |
| serial fold | 12 of 1 | `3f800000` |
| single-workgroup tree | **6 of 2**, widest workgroup 6, 32 threadgroup bytes | **`3f800001`** |
| multi-pass split | **4 of 3**, 3 encoders | **`3f800000`** |

Each matched its own declared grouping bit for bit, and each parallel alternative's bits were **refused** by the other's declared grouping — a value this contract permits, refused by the same function that admitted the device's own bits. The split's answer coincides with the serial fold's here, so this shape separates the tree from the split and the four-contributor shape remains the one where both parallel strategies diverge from the fold. Neither subsumes the other.

### The operand-pair caveat, discharged rather than skipped

The padded set was measured to be exactly as weak as the repair block warned: of the 144 single-contributor corruptions it leaves **81** undetected under the tree's grouping and **98** under the split's. A genuine twelve-wide second set was therefore added — `SEPARATING_EXACT_OPERANDS`, twelve distinct powers of two — on which every one of the 58,786 order-preserving groupings produces `0x457ff000`, every subset sum is distinct, and **0 of 144** corruptions escape under either grouping. Both sets are dispatched at `1x12`; all four counts are pinned device-free.

### Files changed and checks

`crates/tiler-conformance/src/serial_sum.rs`, `crates/tiler-conformance/src/serial_sum/tests.rs`, `crates/tiler-conformance/src/portability.rs`, `docs/correctness-and-testing.md`.

`cargo fmt --check`; `cargo nextest run -p tiler-conformance` — **75 run, 75 passed, 1 skipped**, the skip being the pre-existing `#[ignore]`d whole-profile envelope run at `crates/tiler-conformance/src/envelope/tests.rs`; `cargo clippy -p tiler-conformance --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-conformance`; `cargo test --doc` (workspace); `tkt lint`; `./check-citations.sh`; `git diff --check`; `tkt guard`. **Caveat on the rustdoc gate:** every module in this crate is `#[cfg(test)]`, so `cargo doc` documents `lib.rs`'s header alone and cannot fail for any doc comment this change adds.

The portability floor moved **67 → 72** with the population, by the same five, and was watched failing at 74.

### Each new test watched failing on its own subject

- separating count set to 8, where the two rules agree → the tree publishes `{4, 2}` where `{6, 2}` was required;
- the exact twelve-wide set replaced by the padded one → the corruption census reports `(144, 81)` where `(144, 0)` was required;
- the split held to the tree's own partition → the two declared groupings collapse to one value;
- **each strategy held to its own partition instead of the other's** → "the single-workgroup-tree returned `[3f800001]`, which the other parallel strategy's declared `{6, 2}` also produces; the two groupings did not separate on this device". This is the refusal itself failing, on the bits the device returned;
- the padded set dispatched against the exact set's oracle → the serial fold returns `[3f800000]` where `[457ff000]` was required.

### One graph edge that was missing

`bound-the-tree-cap-s-unmeasured-downward-direction` (`todo`, `implementation/compiler`) proposes changing `capped_tree_partition`'s rule. If it lands first the discriminating counts move — 12 probably survives, since the cap picks the widest there, which is the direction that ticket argues for, but **the count set as a whole is not stable**. Now `related`, along with `complete-the-tree-cap-audit-sweep-inside-the-compiler-crate`, which already cross-references this ticket. Neither is a hard dependency; re-derive the count on the branch's own base rather than trusting this section's 12 if either has landed.

## Current-rule correction — 2026-08-09

[`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md)
has now landed. As the worker record anticipated, twelve remains the smallest
separating count and the delivered device measurement remains valid: the tree
takes 6 partitions of 2 while the split takes 4 of 3. The wider census did
move. Under the current nearest-to-256 rule, 3,530 counts below 4,096 admit a
width and **2,350**, not the historical 2,561, choose different tree and split
partitions. `the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs`
pins the current population. This correction changes neither the retained
measurement nor its host/toolchain boundary.
