---
id: share-the-staged-ownership-test-between-the-recognizer-and-the-physical-speller
title: Share the staged ownership test between the recognizer and the physical speller
status: review
priority: p3
dependencies: []
related: [admit-a-scheduled-region-for-a-staged-elementary-family]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner, maintenance]
claimed_from: todo
assignee: agent-ownership-dedup
lease_expires_at: 1786070081
---
## User-visible outcome

The predicate "this member set is a region of this staged occurrence" has one authority, so a widening of the staged shape (a three-stage law, or atoms spanning members) changes one site instead of silently diverging two.

## The duplication, found by the 2026-08-06 session audit

**Fact.** `spell_output`'s `Staged` arm in `crates/tiler-compiler/src/physical.rs` (near :873 at the audit's base) re-implements, verbatim, the private `NormalizedOutput::owns_region_members` `Staged` arm in `crates/tiler-compiler/src/request.rs` (near :1820): `!members.is_empty() && members.iter().all(|atom| atom.member() == normalized.member)`. The comment above the physical copy reads "the ownership test is the recognized partition's own", which reads as delegation but is a copy — `owns_region_members` is a private `fn` unreachable from `physical.rs`, and nothing asserts the two agree. This is the second-account-of-one-fact drift AGENTS.md warns about, at exactly the predicate the next staged widening must change in both places.

## The work

Either make `owns_region_members` `pub(crate)` and have `spell_output` call it (the comment then becomes true as written), or add a test asserting the two predicates agree over the staged fixture's member sets including the refusing cases (empty set, straddling atoms). Prefer the first — one authority beats an agreement test — unless reading exposes a reason the physical layer must not depend on the recognizer's method. Verify the two sites' line positions on your base; the audit's are facts about e4ccc6d9.

## Closes when

One site owns the predicate (or an agreement test pins the pair with a watched-failing perturbation), and the physical arm's comment states what is actually true.

## Outcome 2026-08-06 — the preferred fix applied: one authority, and the delegation is measured

`8a093fcb9be0a5f5af375a86882792e281c71e26`, on `452bc91c` (`crates/tiler-compiler/src/physical.rs`, `crates/tiler-compiler/src/request.rs`; +24 −9).

### The duplication, re-derived on this base

**Fact.** The audit's finding holds unchanged at `452bc91c`, at moved lines. `NormalizedOutput::owns_region_members`'s `Staged` arm (`request.rs:1820`) and `spell_output`'s `Staged` arm (`physical.rs:873`) both read `!members.is_empty() && members.iter().all(|atom| atom.member() == normalized.member)`. `spell_staged`'s evolution since the audit did not absorb the copy: it decides fold/pass/wall *after* the ownership question, on the `[fold]` / `[pass]` / neither split, and it is called only once ownership has already been answered — so the two arms remained verbatim twins with nothing asserting their agreement.

### The fix

The ticket's preferred form, and reading exposed no reason to prefer the fallback. `owns_region_members` is `pub(crate)`, and the physical arm is now

```rust
NormalizedOutput::Staged(normalized) => output
    .owns_region_members(members)
    .then(|| spell_staged(normalized, position, members, write)),
```

so the comment that read as delegation is delegation.

**What `pub(crate)` exposes, read arm by arm before widening it.** All five: the reduction's three parts, the two single-part shapes, and the epilogue's own part or any of its producer's. Nothing there is a fact the crate should not see — `region.rs:157` already documents this method by name as one of the three attribution comparisons the planner makes, and `physical::spell_region` already runs its own scan over the recognized outputs rather than going through `NormalizedProgram::output_for_region`. The one hazard worth naming is that a future crate-local caller might reach for this predicate where it wants `output_for_region`, which additionally decides the declaration-order tie-break between two claimants of an admitted overlap; the doc comment now separates the two questions explicitly.

**Only the staged arm can delegate wholesale, and the arm says why.** The pointwise, contraction, serial-sum and epilogue arms compare member lists themselves because each must know *which* part matched to name a spelling kind — prologue versus reduction versus fused, epilogue versus producer. A staged occurrence's partition has one part and *which stage* is `spell_staged`'s question, so ownership is the whole of that arm's question and the boolean is all it needs.

### Perturbation: the delegation is load-bearing, with a control

**Perturbed the shared predicate only** — `request.rs`'s `Staged` arm reduced to `!members.is_empty()`, dropping the member-equality conjunct — and ran `pipeline::tests::a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit`.

*On the fixed tree the physical wall vocabulary moved*, at `pipeline/tests.rs:2634`:

```text
left:  {"region-staged-family-unspellable": 3}
right: {"region-partial-coverage": 2, "region-staged-family-unspellable": 1}
```

The two regions grouping a stage of the normalization with the consuming multiply stopped falling through the scan as partial coverage and were claimed by the staged arm, where `spell_staged` declines them — exactly the reassignment the delegation predicts.

*The control isolates the delegation as the cause.* Restoring the pre-fix verbatim copy in `physical.rs` while keeping the same perturbation leaves the wall map at `:2634` **untouched** — the physical arm ignores the recognizer entirely, as it did before this ticket — and the test instead fails one assertion later, at `:2645`, on the role attribution count (3 → 5), which reads `output_for_region` rather than `spell_region`. So the wall move is attributable to the delegation and to nothing else: a change confined to `request.rs` reaches the physical speller's decision only because the copy is gone.

Both perturbations reverted; the tree was diffed against the pre-perturbation patch and is byte-identical to it.

### Checks

`cargo fmt --check`; `cargo check -p tiler-compiler --all-targets`; `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `cargo nextest run -p tiler-compiler` **718 passed, 1 skipped**; `cargo nextest run --workspace`; `cargo test --workspace --doc`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler`; `tkt lint`; `git diff --check`; `tkt guard` against `452bc91c`; `make full` on the branch. Results in the worker report.
