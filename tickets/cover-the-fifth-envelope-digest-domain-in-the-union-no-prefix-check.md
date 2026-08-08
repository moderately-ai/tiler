---
id: cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check
title: Cover the fifth envelope digest domain in the union no-prefix check
status: in-progress
priority: p1
dependencies: []
related: [date-the-artifact-abis-metal-golden-enumeration-to-its-step]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786170921
---
## A hashed domain that a normative obligation requires be checked, and is not

`docs/artifact-abi.md` is normative and explicit: *"**A new governed domain in either container must be added to the union check**, and adding one to the envelope-local check alone does not discharge this obligation."*

**`PAYLOAD_IDENTITY_DOMAIN` discharges neither half.** Coordinator-verified — `grep -rn PAYLOAD_IDENTITY_DOMAIN crates/tiler-artifact/src` returns exactly two lines: its declaration in `program/codec/payload.rs` (`b"tiler.artifact-envelope.payload-identity.v1\0"`) and its single hash site in the same file. It appears in **neither** the union check nor the envelope-local one, and `proof/tests.rs`'s union test hard-codes `let domains: [&[u8]; 8]`.

So the envelope hashes under **five** digest domains rather than four, the crate under **nine** rather than eight, and **8 of 9 are checked**.

## The property holds today; the check does not establish it

The auditor compared the strings by hand: `…payload-identity…` prefixes nothing in the set and nothing prefixes it. **So this is not a live collision** — it is an unchecked one, which is exactly what the normative sentence exists to prevent. A future domain could collide with `PAYLOAD_IDENTITY_DOMAIN` and no test would notice.

**Verify that by hand yourself before relying on it**, and then make the check say it rather than a reader having to.

## Requirements

- **Add it to the union check** — the authority the document names — and to the envelope-local check, since the document treats those as two halves rather than alternatives.
- **The hard-coded `8` is the defect's own cause.** A count literal beside a list is what let a ninth domain be added without anything failing. Derive the population, or floor and assert it, so the next domain is a build error or a red test rather than a silent omission. This repository has hit that shape repeatedly.
- **Correct the counts in the prose too.** `docs/artifact-abi.md` currently contradicts itself: one passage says "these four and the envelope's three are the **seven** the union no-prefix obligation covers", another in the same document says "the crate's **eight** governed domains", and a third says "four domain separators". Reconcile all of them against the true population.

  **The count sites are five, not three — verified 2026-08-07 at base `7c371155` by `verify-and-file-the-remaining-maturity-audit-leads`.** Beside the three named above, the "four domain separators" claim is made twice more, in passages about the *public surface* rather than about the digest: `docs/artifact-abi.md "including the four digest domain separators"` in the codec-promotion Fact, and `docs/artifact-abi.md "The framing magic, the four domain separators, the schema versions"` in the wire-form Fact. Both are the same falsified count reached in a different context, and the `Closes when` phrase "three inconsistent counts" would let them survive a sweep that reads it literally. Reconcile all five.
- **Watch it fail**: add a probe domain that prefixes an existing one and confirm the union check reddens by name. Perturb the subject, not the assertion.

## Closes when

Every domain the crate hashes appears in the union check; the population is derived or floored rather than a literal; all five of `docs/artifact-abi.md`'s inconsistent counts agree with the code; and the check has been watched failing on a planted prefix collision.

## Scheduling note — one file, two live claims

`date-the-artifact-abis-metal-golden-enumeration-to-its-step` also holds `contracts/artifacts` and also edits `docs/artifact-abi.md`, for an unrelated subject (the Metal golden corpus named at the `tiler.schedule.v5` step). The subjects do not overlap but the scope and the file do, so the two must be sequenced rather than run concurrently.
