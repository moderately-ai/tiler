---
id: derive-the-metal-fenced-space-population
title: Derive the Metal fenced-space population
status: in-progress
priority: p2
dependencies: []
related: [derive-the-artifact-numerical-and-fenced-space-populations]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: terra-metal-fences
lease_expires_at: 1786403012
---
## Fact audit — split from the artifact population repair on 2026-08-08

**Fact.** `crates/tiler-metal/src/synchronization_requirement_tests.rs`, at `const FENCES: [FencedSpaces; 4]`, hand-enumerates the product of the two current boolean fields. `POPULATION` derives through `FENCES.len()`, so a third field can leave both the enumeration and the claimed 648-value population short.

**False source claim.** The adjacent comment, at `A third flag would leave this list at four entries`, says the population assertion would then fail. The assertion derives from the same short list and remains 648; it cannot detect the widened type.

**Boundary.** This is an independent Metal test population, not an artifact encoder or identity change. [`derive-the-artifact-numerical-and-fenced-space-populations`](derive-the-artifact-numerical-and-fenced-space-populations.md) owns its artifact copy and must not absorb this one.

## What closes this

- Derive the field count from one exhaustive `FencedSpaces` destructure and size the enumeration as the boolean product, following the private IR/artifact pattern without sharing test support across crates.
- Add a third boolean field as a temporary subject perturbation. Repair constructors and the field census while intentionally leaving the enumeration short; require the Metal population mechanism itself to report an eight-versus-four array size, quote the diagnostic, and restore.
- Correct the false comment and keep the 648 current count only as a consequence of the derived population.
- Change no production field, synchronization encoding, Metal behavior, public surface, identity, or artifact file.
