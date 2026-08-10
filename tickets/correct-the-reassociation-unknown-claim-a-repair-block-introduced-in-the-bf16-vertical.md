---
id: correct-the-reassociation-unknown-claim-a-repair-block-introduced-in-the-bf16-vertical
title: Correct the reassociation-Unknown claim a repair block introduced in the BF16 vertical
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## A correction introduced a new false claim, which is the pattern this repository keeps hitting

`crates/tiler-conformance/src/bf16_vertical.rs`'s module header, in the bullet **"Reassociation is withheld rather than proved"**, states that `BF16_FACT_REASSOCIATION_PERMITTED` is `false` and "the question stays open at the operation vocabulary, so a contract that *permits* regrouping leaves the obligation `Unknown`".

**False for the region it is written about.** Coordinator-verified: `push_reduction_obligations` in `crates/tiler-compiler/src/fusion_legality.rs` discharges `ReductionReassociation` as **`SoundProof`** when `!has_reduction || reassociation == Forbidden`. The BF16 vertical is `(x * 1.5) + 0.0` — **pointwise, no reduction** — so `!has_reduction` short-circuits to `SoundProof` *regardless of what the contract permits*. The `Unknown { "unproven-reassociation" }` branch requires a reduction **and** a permitting contract~~, which is the surviving contraction wall and a different region~~ **(struck 2026-08-10 — that clause is itself false; see Worker finding / Outcome. Unknown reassociation is not the BF16 contraction wall: that wall is `unrealized-contraction` / `ArithmeticContraction` under Forbidden reassociation; the explicit `unproven-reassociation` assertion site is `a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction` over an f32 serial-sum region)**.

**How it got here, and why that matters more than the sentence.** This text landed on 2026-08-07 inside a *repair block* correcting the coordinator's own earlier over-general claim that reassociation is "withheld as `Unknown`". The repair fixed the framing and then restated a narrower version of the same error. That is the third time this session a correction has introduced a fresh false claim, and it is exactly the failure `AGENTS.md` now warns about: **never restate a false Fact in new words.**

Found by the worker on `correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents`, which held `contracts/*` and could not reach `crates/`.

## What is true, stated so the repair does not overshoot a third time

For this vertical's pointwise region the obligation is discharged **`SoundProof`, vacuously** — nothing in the region raises a reduction order to preserve. Say *that*, and keep the honest residue separate: a vacuous discharge is not evidence the reductions are correct, only that none were present. `BF16_FACT_REASSOCIATION_PERMITTED` being `false` is a true and separate fact about the operation vocabulary; it is not what decides this obligation.

**Do not write "records `Unknown`"** in any form for a reduction-free region.

## Closes when

The bullet states the vacuous `SoundProof` discharge with its correct ground; no reduction-free region is described as leaving reassociation `Unknown`; ~~the surviving contraction wall is named as the place the `Unknown` branch is actually reached~~ **(struck — see the worker finding below; this clause is itself false)** the place the `Unknown` branch is actually reached is named correctly; and the change is verified by reading `push_reduction_obligations` rather than by citing this ticket.

## Worker finding: one clause of this ticket was itself wrong

**Fact, verified by reading `crates/tiler-compiler/src/fusion_legality.rs` in full.** The ticket body's central claim is correct — `push_reduction_obligations` discharges `ReductionReassociation` as `SoundProof` under `!has_reduction || reassociation == Forbidden`, so a reduction-free region short-circuits before the contract's reassociation resolution is read. That is what the repair states.

**But the "Closes when" clause naming the contraction wall as the place the `Unknown` branch is reached is false, and following it would have made this the fifth false repair.** The surviving BF16 wall, `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall` (`crates/tiler-compiler/tests/bf16_numerical_contract.rs`), stops on a **different obligation**: it asserts the trace contains `unrealized-contraction`, which is `FusionObligation::ArithmeticContraction`, not `ReductionReassociation`. Its contract is `NumericalContractBuilder::strict_bf16()` with only the subnormal rows and `contraction` overridden, so its **reassociation resolution is `Forbidden`** — meaning that region takes the `SoundProof` arm on *both* disjuncts. The `Unknown { "unproven-reassociation" }` branch is not reached there at all.

**Where it is actually reached.** It needs a reduction member *and* a permitting contract, which is an `f32` region today: `tiler-compiler fusion_legality::tests::a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction` puts `serial_sum_program` (which carries `StrictSerialF32Sum`) to `StrictF32NumericalContract::governed_reassociating()` and asserts `unknown.obligation() == FusionObligation::ReductionReassociation` with `unknown.reason() == "unproven-reassociation"`. Both cited names resolve against `cargo nextest list -p tiler-compiler`.

**Why no BF16 region can reach it.** `tiler-ir` registers exactly three BF16 op keys — `constant_bf16_op` (ValueSource), `multiply_bf16_op` and `add_bf16_op` (ElementwiseArithmetic) — and no fold, so `is_reduction` is false for every member a BF16 region can hold. The vacuous discharge is therefore a property of the BF16 vocabulary, not only of this vertical's pointwise shape.

## Outcome — done, 2026-08-07

Landed at merge `cd489e6a` (worker commit `0e116005`). `make full` exit 0, 1,091 release tests.

### The worker refused the coordinator's instruction, and was right

My brief told it to "name the surviving contraction wall as the place the `Unknown` branch is actually reached." **That would have been the fifth false claim in this chain.** It refused, verified, and reported — coordinator-confirmed on both grounds:

1. `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall` asserts `unrealized-contraction`, which is `FusionObligation::ArithmeticContraction` — **a different obligation entirely**.
2. That test's contract is `NumericalContractBuilder::strict_bf16()`, which routes through `strict()` and sets `reassociation: NumericalPermission::Forbidden`. So the region satisfies **both** disjuncts and takes the `SoundProof` arm. It is not an `Unknown` site at all.

The `Unknown` branch is explicitly asserted at `fusion_legality::tests::a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction`, over `serial_sum_program` under `governed_reassociating()` — an **f32 reduction** region, asserting `unproven-reassociation` explicitly. Both names verified against `cargo nextest list`. Production can also reach the branch for any reduction-bearing region under a permitting contract; that test is the named checkable site, not a claim of unique runtime reachability.

That is five coordinator-authored errors in this one chain, each caught by a worker reading the source before writing. The reading obligation in `AGENTS.md` is doing exactly the work it was added for.

### The correction, and why it overshoots in neither direction

`push_reduction_obligations` discharges `SoundProof` when `!has_reduction || reassociation == Forbidden`, and **`!has_reduction` short-circuits before the contract is read** — so no reduction-free region can record `Unknown` under any contract.

The replacement says neither `Unknown` nor *proved*: a `SoundProof` recorded over no contributors "is evidence that none were present, not evidence that any are right". `BF16_FACT_REASSOCIATION_PERMITTED` is kept as its own bullet — a vocabulary-level open question, explicitly **not** the thing deciding the obligation.

**The vacuity ground was strengthened beyond the brief.** Rather than "no fold registered", it is now the checkable population: three BF16 keys — `constant_bf16_op` as a value source, `multiply_bf16_op` and `add_bf16_op` elementwise — so `is_reduction` is false for **every member a BF16 region can hold**. That makes it a property of the vocabulary rather than of this one pointwise vertical.

The worker also confirmed `StrictF32NumericalContract` is the *general* contract type despite its historical name, so the discharge argument genuinely applies to BF16 rather than being an f32-only path.

70 tests passed, 1 skipped (the deliberate `#[ignore]`). One LEAK verdict on an unrelated test — the known macOS pipe-inheritance race AGENTS.md describes, not a new unreaped child.
