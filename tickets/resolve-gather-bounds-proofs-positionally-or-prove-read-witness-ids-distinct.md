---
id: resolve-gather-bounds-proofs-positionally-or-prove-read-witness-ids-distinct
title: Resolve gather bounds proofs positionally or prove read witness ids distinct
status: todo
priority: p1
dependencies: []
related: [admit-the-selected-data-dependent-index-representation, carry-the-gather-relation-through-the-compiler-vertical]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, gather, identity, fail-closed]
---
## User-visible outcome

Every retained gather bounds proof in an admitted region is validated by exactly one rule, so a proof folded into canonical scheduled-region identity cannot be one nothing checked.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit. **Every element below was verified at source by the coordinator at `7d5fd8ad`**, and the four together are the finding — no single one is alarming.

**Fact — Rule 8 resolves a read's bounds proof by witness id.** `crates/tiler-ir/src/schedule/builder/elementwise.rs`, anchor `find(|record| record.id == read.bounds)`.

**Fact — every sibling resolves positionally.** `crates/tiler-ir/src/schedule/builder/proof.rs`'s `verify_proof_records` splits the last proof off as the write proof and zips the remainder against the reads — anchor `read_proofs.iter().zip(reads)` — enforcing a positional bijection.

**Fact — the only id-inequality rule is read-versus-write.** Same function, anchor `read_proofs.iter().any(|proof| proof.id == write_proof.id)`. **There is no read-versus-read distinctness rule anywhere.**

**Fact — the gather pair delegates all validation to Rule 8 and checks nothing itself.** `bounds_proof_refines_access` returns `true` unconditionally for `(BoundsProofKind::GatherSource { .. }, LogicalAccess::GatherSource { .. })` — anchor `LogicalAccess::GatherSource { .. }) => true`, on the same line as the `PartitionedCopySource` arm.

**Inference — two gather reads sharing one `BoundsWitnessId` leave a retained proof validated by nothing.** With equal relations, `.find` returns `proof[0]` twice; `proof[1]`'s retained static proof subject is then checked by no rule, while **it is still folded into canonical scheduled-region identity** (`crates/tiler-ir/src/schedule/model.rs`, anchor `push_len(&mut bytes, region.index.bounds_proofs.len());`). The state is constructible through the public `push_access` / `push_bounds_proof` with caller-chosen ids.

**Blast radius today is bounded, and that is why this is p1 rather than p0.** `crates/tiler-ir/src/kernel/lower.rs` refuses `LogicalAccess::GatherSource` outright at the `body-refinement` wall, so no wrong kernel result is reachable now. **But `kernel/verify.rs`'s `access_elements` already consumes `BoundsProofKind::GatherSource { source_shape }` to size a bound** — so this becomes live the moment the wall comes down. **Land this before any ticket that removes that wall.**

## Required work

- Re-audit all four Facts at your base with a per-Fact verdict; each is a one-line anchor and each was grepped against the file it names.
- Choose **by reading** between the two repairs, and say why: index positionally in Rule 8, matching every sibling; or add read-versus-read distinctness to `verify_proof_records`. They are not equivalent — the first makes the id irrelevant to resolution, the second keeps ids meaningful and makes collision an error. Say which invariant the vocabulary actually wants.
- Whichever you choose, **the unconditional `true` for the gather pair should stop being unconditional or should say in its own text exactly which rule discharges it**, so a later reader cannot mistake it for a check.

## Evidence

- **Construct the colliding state and show it refused**, quoting the failure text. Two gather reads, one `BoundsWitnessId`, equal relations, through the public builder. A repair asserted without that construction has not been demonstrated — the whole finding is that the state is constructible.
- Perturb each rule separately; a perturbation that reddens both cannot show which is load-bearing.
- Before trusting the new check, state what it would take for it to say *no*, and confirm that case is reachable. A sibling lane found its own gate made one rule unreachable by pigeonhole and caught it only by asking that.
- **State whether any identity value moves. Expected: none** — this is a refusal over already-encoded fields — but rederive rather than copy, and **stop and report** if one does.

## Non-goals

Removing the kernel `body-refinement` wall; the compiler vertical, which is its own ticket; and any change to what a gather proof contains.

## Closes when

No admitted region can retain a gather bounds proof that no rule validated, the colliding construction is watched being refused with its output quoted, each rule is perturbed separately, and no identity value has moved.
