---
id: retire-the-false-unasserted-identity-difference-prose-after-the-pin
title: Retire the false unasserted identity-difference prose after the pin
status: todo
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

## What is false today

1. `docs/artifact-abi.md` paragraph beginning `What is left unpinned, stated rather than left for a reader to assume.` still says the forged pair's equal identity length and four differing positions have `and no test asserts either.` and `Until that lands, this paragraph's four is a measurement and not a guarantee.` Both properties are asserted in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` via length equality then `DIFFERING_IDENTITY_POSITIONS = 4`.

2. The neighbouring Measurement clause in the same document still restates that the two canonical identities `differ at exactly four byte positions` as unasserted measurement prose rather than naming the test and constant.

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
