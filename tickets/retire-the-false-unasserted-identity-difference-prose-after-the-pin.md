---
id: retire-the-false-unasserted-identity-difference-prose-after-the-pin
title: Retire the false unasserted identity-difference prose after the pin
status: done
priority: p2
dependencies: []
related: [pin-the-differing-identity-positions-beside-the-carrier-positions-constant, replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin, recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix]
scopes: [contracts/artifacts, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, docs]
---

The implementation pin for the BF16-versus-F32 carrier-only identity difference has already landed. This ticket rewrites the live contract and ledger sentences that still claim nothing asserts it. **No crate change. No new pin. No reopening absolute lengths or offsets.**

Parent implementation: [`pin-the-differing-identity-positions-beside-the-carrier-positions-constant`](pin-the-differing-identity-positions-beside-the-carrier-positions-constant.md) at `b03f2b81`. That ticket's scopes were `implementation/artifact` only; the contracts prose that still denies the pin is this remainder.

## Source audit before edits — 2026-08-10

1. **Verified.** In `docs/artifact-abi.md`, the source anchor `What is left unpinned, stated rather than left for a reader to assume.` still says `no test asserts either` and `Until that lands`. In `crates/tiler-artifact/src/program/codec/tests.rs`, `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` now asserts equal identity lengths before asserting the positional-difference population against `DIFFERING_IDENTITY_POSITIONS`.
2. **Imprecise.** In `docs/artifact-abi.md`, the neighboring anchor `Measurement, of the carrier-only forged pair.` restates that the identities differ at exactly four positions without naming the landed pin, but that sentence does not itself call the property unasserted. The false authority classification comes from the later `What is left unpinned` paragraph. The correction must date and retire that classification while naming the assertion and constant.
3. **Verified.** In `docs/dtype-support.md`, the anchor `the four-byte identity difference still held when it was checked` still continues `but nothing asserts it either`, despite the two assertions above.
4. **Verified boundary.** `const DIFFERING_CARRIER_POSITIONS` and `const DIFFERING_IDENTITY_POSITIONS` are separate subjects used by separate assertions in the codec test. The parent ticket and commit `b03f2b811f901e373f84c0f9b9e159261981caa9` record independent perturbations for the identity length and identity-position population. No test pins an absolute identity length or any byte offset.

The imprecision does not change the purpose: this remains a docs-only repair of the two live false absence claims and the neighboring stale authority description.

## What is false today

1. `docs/artifact-abi.md` paragraph beginning `What is left unpinned, stated rather than left for a reader to assume.` still says the forged pair's equal identity length and four differing positions have `and no test asserts either.` and `Until that lands, this paragraph's four is a measurement and not a guarantee.` Both properties are asserted in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` via length equality then `DIFFERING_IDENTITY_POSITIONS = 4`.

2. The neighbouring Measurement clause in the same document still restates that the two canonical identities `differ at exactly four byte positions` without naming the landed test and constant; the later paragraph is what falsely classifies that property as unasserted.

3. `docs/dtype-support.md` still says the four-byte identity difference `nothing asserts it either` (anchor that phrase beside the retired absolute-length retirement prose).

## What closes this

Rewrite all three sites so they name the landed pin rather than claiming absence:

- equal identity length precondition and `DIFFERING_IDENTITY_POSITIONS` in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` (`crates/tiler-artifact/src/program/codec/tests.rs`);
- keep the two constants separate subjects and do **not** pin byte offsets (already the test rule);
- do not reintroduce absolute identity lengths or absolute offsets as present-tense unasserted digits.

House style: prefer a pin citation or a dated correction that the unpinned claim was true only until the pin ticket landed — not a silent digit swap that re-owns the count as contract prose.

## Explicitly not in scope

- Reopening absolute envelope lengths, absolute identity lengths, or absolute offsets.
- Editing `crates/**` or re-opening the implementation pin ticket.
- Collapsing `DIFFERING_IDENTITY_POSITIONS` with `DIFFERING_CARRIER_POSITIONS`.

## Outcome

The two contracts now classify their old unasserted wording as history of the gap closed by the parent pin and name the live test authority. The artifact ABI correction states that the forged-pair Measurement is now a tested property: equal canonical-identity lengths are asserted before the positional population is asserted against `DIFFERING_IDENTITY_POSITIONS`. The dtype ledger strikes its false `nothing asserts it either` clause and records the same derivation.

Both corrections keep `DIFFERING_IDENTITY_POSITIONS` distinct from the digest-sensitive `DIFFERING_CARRIER_POSITIONS`, and neither introduces an absolute identity length, absolute envelope length, or byte offset. The residual source hits are intentional history: `docs/artifact-abi.md` retains the old `What is left unpinned` paragraph immediately before its dated correction, `docs/dtype-support.md` retains `nothing asserts it either` only inside the struck clause immediately followed by its dated correction, and this ticket quotes both as the audited trigger.

Checks on the completed docs-and-ticket delta: `make citations`; `tkt lint --format json`; `git diff --check`; and, after commit, `tkt guard tkt/retire-the-false-unasserted-identity-difference-prose-after-the-pin --base 1a716aeb1b523c520a985796b380d18343c1e0cc --format json`. No path in the delta invalidates a published full gate under the repository carry rule, so no crate/package/full gate is rerun.
