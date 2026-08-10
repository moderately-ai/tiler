---
id: reproduce-the-identical-output-chain-stage-key-collision
title: Reproduce the identical-output-chain stage-key collision at the current compiler boundary
status: in-progress
priority: p2
dependencies: []
related: [refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, multi-output, identity, research]
claimed_from: todo
assignee: sol-stage-key-collision
lease_expires_at: 1786402510
---
## Why this evidence is separate

**Historical Measurement — 2026-08-06.** At base `afdac9c9`, two independent same-shaped epilogue chains over different declared inputs reached `InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))`; changing one chain's extent avoided the collision. The current compiler test module preserves that finding only as prose under the anchor `two chains of identical shape assemble two stages carrying one canonical key`. It deliberately uses different extents in its executable fixture, so it does not prove the old failure still occurs.

**Fact — current identity seam.** `stage_key` derives a stage key from the bound kernel identity and proof-bound coverage. `verify_unambiguous` rejects two equal stage keys. Program identity then orders stages and names value definitions through those keys. Changing the key's subject or merging stage instances is therefore an identity question; reproducing the failure is not.

## Work

1. Read the complete multi-output fixture, program assembly, stage-key derivation, unambiguous-key verification, and compiler failure mapping.
2. Add a current compiler regression using two independently declared inputs and outputs whose producer chains have the same shape. Keep the different-extent neighbor and a one-chain control.
3. Record the exact current public failure. If the pair now compiles, record the program/stage population and why the historical collision disappeared.
4. If it still collides, inspect the two assembled stages and state exactly which fields agree and which distinct bindings, values, outputs, launches, or coverage facts the current stage key omits.
5. Perturb the subject—shape, binding, or occurrence ownership rather than the expected error—and quote the changed result.

Do not change stage keys, program identity, IR verification, assembly, or the public failure vocabulary here.

## Closes when

The historical collision is either reproduced at the current public compile boundary with a source-backed stage comparison and controls, or proved obsolete with the landing that removed it. The dependent decision ticket carries the resulting evidence rather than the 2026-08-06 measurement alone.
