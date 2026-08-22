---
id: resolve-gather-bounds-proofs-positionally-or-prove-read-witness-ids-distinct
title: Resolve gather bounds proofs positionally or prove read witness ids distinct
status: in-progress
priority: p1
dependencies: []
related: [admit-the-selected-data-dependent-index-representation, carry-the-gather-relation-through-the-compiler-vertical]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, gather, identity, fail-closed]
claimed_from: todo
assignee: worker-boundsproof
lease_expires_at: 1787441390
---
## User-visible outcome

Every retained gather bounds proof in an admitted region is validated by exactly one rule, so a proof folded into canonical scheduled-region identity cannot be one nothing checked.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit. **Every element below was verified at source by the coordinator at `7d5fd8ad`**, and the four together are the finding — no single one is alarming.

**Fact — Rule 8 resolves a read's bounds proof by witness id.** `crates/tiler-ir/src/schedule/builder/elementwise.rs`, anchor `find(|record| record.id == read.bounds)`.

**Fact — every sibling resolves positionally.** `crates/tiler-ir/src/schedule/builder/proof.rs`'s `verify_proof_records` splits the last proof off as the write proof and zips the remainder against the reads — anchor `read_proofs.iter().zip(reads)` — enforcing a positional bijection.

> **Imprecise — narrowed 2026-08-22 at `6f3c2594`.** True of the schedule builder's own gates, and the copy gate is a second instance (`crates/tiler-ir/src/schedule/builder/copy.rs`, anchor `region.index.bounds_proofs[position]`). It is **not** true of the tree: `crates/tiler-ir/src/kernel/verify.rs`'s `access_elements` resolves **by id** (anchor `find(|proof| proof.id == access.bounds)`). That second id-keyed resolver is what decides the repair below, because indexing rule 8 positionally would leave it resolving an ambiguous key.

**Fact — the only id-inequality rule is read-versus-write.** Same function, anchor `read_proofs.iter().any(|proof| proof.id == write_proof.id)`. **There is no read-versus-read distinctness rule anywhere.**

> **Verified, then retired by this ticket's own repair — 2026-08-22.** Both halves were true at `6f3c2594`. The quoted anchor now greps **zero** in the file it names, because the repair replaced that narrower clause with whole-list distinctness; the live anchor is `witness_ids.windows(2).any(|pair| pair[0] == pair[1])` in the same function. Recorded because a reader who greps the retired line and finds nothing would otherwise conclude the Fact was never true, which is the false-absence direction — the read-versus-write property is not gone, it is subsumed, and `a_read_proof_and_the_write_proof_may_not_claim_one_bounds_witness` pins that it survived.

**Fact — the gather pair delegates all validation to Rule 8 and checks nothing itself.** `bounds_proof_refines_access` returns `true` unconditionally for `(BoundsProofKind::GatherSource { .. }, LogicalAccess::GatherSource { .. })` — anchor `LogicalAccess::GatherSource { .. }) => true`, on the same line as the `PartitionedCopySource` arm.

> **Imprecise — narrowed 2026-08-22 at `6f3c2594`.** The unconditional `true` is verified. The premise attached to it in `Required work` — that a reader cannot tell the arm is delegating — was already false: the arm's comment named its discharging rule at this base (anchor `rule in the pointwise gate`). What the comment did *not* say is the part that mattered, that the delegation is total only while witness ids are distinct, since the rule it delegates to resolves by id while this arm is reached positionally. That precondition is what the repair added.

**Inference — WITHDRAWN 2026-08-22 at base `6f3c2594` (worker-boundsproof). Two *gather* reads sharing one `BoundsWitnessId` are refused today, so the retained-proof hole this Inference described does not exist in that shape.** The Inference claimed the state was constructible through the public builder. It is not, and the reason is a rule the audit did not weigh: rule 5 (`gather-address-read-shared`) forbids two gathers naming one address read, so their `index_access` ordinals are always distinct; rule 8 then resolves the second gather's proof by id onto the *first* gather's record and compares `index_access`, which cannot match. Both refusal paths were constructed and watched:

- distinct address reads, one witness id → `GatherAddressRead { source_access: Some(AccessOrdinal(1)), index_access: AccessOrdinal(3), rule: ProofMismatch }`;
- one address read as well → `GatherAddressRead { source_access: Some(AccessOrdinal(0)), index_access: AccessOrdinal(2), rule: IndexShared }`.

This is exactly the pigeonhole hazard the brief warned about: the gate's own rules made the named case unreachable. Both are now pinned as regressions in `crates/tiler-ir/src/schedule/builder/gather_tests.rs`.

**Fact — the real defect is wider than the Inference and is live today, with no gather involved.** The witness collision *is* constructible and *was* admitted; it just does not need two gathers. Any two reads whose proofs are individually well formed against their own positional access may share one id, because the positional zip proves each record where it sits and no rule compared the ids to each other. The reachable and consequential shape is a gather beside its own address read: the gather's `GatherSource` proof and the address read's `LinearRange { element_count: 2 }` each refine their own access, so nothing objects — and then `access_elements` resolves the address read by id onto the *gather's* record and sizes its buffer parameter from `element_count(source_shape)`, `12884901888` elements for a read of two addresses.

**Blast radius — CORRECTED 2026-08-22. The original paragraph read that the `body-refinement` wall made this unreachable today; that is false, and understated the severity.** The wall bounds only the two-gather shape the Inference described. `crates/tiler-ir/src/kernel/verify.rs`'s `access_elements` resolves **by id** (anchor `find(|proof| proof.id == access.bounds)`), and it is called from `crates/tiler-ir/src/kernel/lower.rs` (anchor `read_elements: reads`) to fill `CanonicalPlan::read_elements` — so a wrong count reaches the *emitted kernel*, not only the verifier. Measured at base `6f3c2594` before the repair: a `BroadcastReplication` read over a two-element operand, whose own proof records `element_count: 2`, answered `access_elements = Ok(6)` after colliding with a `LinearIdentity` read's witness. The severity ordering stands (p1, land before the wall comes down) because no admitted *program* path constructs a colliding region today — but the refusal is not gated behind the wall.

## Required work

- Re-audit all four Facts at your base with a per-Fact verdict; each is a one-line anchor and each was grepped against the file it names.
- Choose **by reading** between the two repairs, and say why: index positionally in Rule 8, matching every sibling; or add read-versus-read distinctness to `verify_proof_records`. They are not equivalent — the first makes the id irrelevant to resolution, the second keeps ids meaningful and makes collision an error. Say which invariant the vocabulary actually wants.
- Whichever you choose, **the unconditional `true` for the gather pair should stop being unconditional or should say in its own text exactly which rule discharges it**, so a later reader cannot mistake it for a check.

## Evidence

- **Construct the colliding state and show it refused**, quoting the failure text. Two gather reads, one `BoundsWitnessId`, equal relations, through the public builder. A repair asserted without that construction has not been demonstrated — the whole finding is that the state is constructible.
- Perturb each rule separately; a perturbation that reddens both cannot show which is load-bearing.
- Before trusting the new check, state what it would take for it to say *no*, and confirm that case is reachable. A sibling lane found its own gate made one rule unreachable by pigeonhole and caught it only by asking that.
- **State whether any identity value moves. Expected: none** — this is a refusal over already-encoded fields — but rederive rather than copy, and **stop and report** if one does.

## Repair taken — 2026-08-22 at base `6f3c2594` (worker-boundsproof)

**Read-versus-read distinctness, generalized to whole-list distinctness in `verify_proof_records`.** The alternative — indexing rule 8 positionally — was eliminated, not merely ranked lower, and the evidence that eliminates it is the corrected Fact above:

- It repairs nothing that is broken. The two-gather collision it targets is already refused by rules 5 and 8 together, demonstrated above.
- It leaves the live defect intact. `access_elements` resolves by id and would keep doing so, so the address-read collision would still size a buffer from the wrong proof. A repair that makes one of two id-keyed resolvers stop using ids does not make the key unambiguous for the other.
- It contradicts the vocabulary. `BoundsWitnessId` is documented as a region-local *reference* to a witness and `BoundsProof::id` as the *identity referenced by the proving access*. Positional resolution would make the id decorative while both resolvers still keyed on it.

Distinctness spans the whole proof list rather than read-versus-read alone, subsuming the narrower read-versus-write clause: the write proof is resolved by id too, and one invariant should have one authority. The gather arm of `bounds_proof_refines_access` stays delegating — a positional field comparison there would be unreachable, since `verify_gather_address_reads` runs first and rule 8 already compares those fields — so what it gained is a statement of the precondition that makes the delegation total.

**Identity: nothing moved, rederived rather than copied.** `encode_identity` (`crates/tiler-ir/src/schedule/model.rs`) is untouched, as are `push_bounds_proof` and the `tiler.schedule.v7` domain separator; the diff adds no encoded field and reorders none. The byte-exact pin `STRICT_F32_REGION_IDENTITY_HEX` is unchanged in the diff and its three assertion sites pass. The admitted set *narrows* — regions with duplicate witness ids no longer verify — but no surviving region's bytes change, which is the distinction between a refusal and an identity step.

**Evidence.** The colliding construction and both perturbations are pinned as tests: `a_gather_and_an_address_read_may_not_share_one_bounds_witness`, `two_gathers_sharing_one_bounds_witness_are_refused_by_the_association_gate`, and `distinct_witnesses_across_two_gathers_and_their_address_reads_still_verify` in `crates/tiler-ir/src/schedule/builder/gather_tests.rs`; `two_read_proofs_may_not_claim_one_bounds_witness` and `a_read_proof_and_the_write_proof_may_not_claim_one_bounds_witness` in `crates/tiler-ir/src/schedule/builder/tests.rs`.

**What it takes for the new rule to say *no*, and that the case is reachable.** Disabling only the distinctness clause admits all three collision subjects — read-versus-read, read-versus-write, and gather-versus-address-read all go green-to-red — while `two_gathers_sharing_one_bounds_witness_are_refused_by_the_association_gate` stays green, since rule 8 catches that one first. Disabling only rule 8's `index_access` comparison reddens the two gather rule tests and leaves the three distinctness tests green, and turns the two-gather refusal from `ProofMismatch` into `ProofReference` — showing the distinctness clause is a real backstop for the gather path rather than dead weight. Each perturbation reddens a distinct set, so neither is carrying the other.

## Non-goals

Removing the kernel `body-refinement` wall; the compiler vertical, which is its own ticket; and any change to what a gather proof contains.

## Closes when

No admitted region can retain a gather bounds proof that no rule validated, the colliding construction is watched being refused with its output quoted, each rule is perturbed separately, and no identity value has moved.

> **Widened 2026-08-22 to match the corrected finding.** The gather-only framing is narrower than what the repair had to close and narrower than what it did close. `verify_proof_records` is the one gate every family reaches — pointwise, the strict-affine `u4` dequantize, the copy family, reduction, and all three contractions — so the delivered condition is: **no admitted region, of any family, can carry two bounds proofs claiming one witness identity**, which makes both id-keyed resolvers total. The purpose is unchanged and the repair is one of the two the ticket named; only the affected population is larger than the gather relation.
