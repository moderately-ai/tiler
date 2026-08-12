---
id: admit-an-explicit-non-arithmetic-region-and-delivery-state
title: Admit an explicit non-arithmetic region and delivery state
status: todo
priority: p1
dependencies: [admit-the-concatenate-family-into-the-scheduled-region-vocabulary]
related: [admit-the-partitioned-copy-scheduled-region, derive-target-numerical-feasibility-from-reached-arithmetic-only]
scopes: [implementation/ir, implementation/artifact, implementation/build, implementation/compiler, contracts/foundation, contracts/numerics, contracts/artifacts, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, public-boundary, numerics, artifacts, identity, strict]
---
## Outcome

A scheduled region, KIR entry, and artifact can state either arithmetic with its complete numerical realization or a bit-preserving non-arithmetic computation for which arithmetic numerical requirements and delivery are explicitly not applicable. Invalid mixed states are unrepresentable; no optional field, default profile, silent absence, or inferred strict realization exists.

## Required boundary

Use exhaustive typed sums at every owning boundary, conceptually `RegionProgram::Arithmetic { scalar, numerical } | PartitionedCopy(...)`, structural requirements plus `NumericalRequirements::{NotApplicable, Arithmetic(...)}`, and an equally explicit artifact delivery form. Exact names follow the source audit, but arithmetic without numerics, copy with numerics, and an unclassified empty state must be impossible.

Preserve the caller's stated program contract as request meaning without asking a target to honour arithmetic a copy never executes. Mixed programs retain complete numerical delivery for arithmetic entries and explicitly classify copy entries as not applicable. Decode and construction reject unknown tags and inconsistent cross-entry claims.

## Identity and compatibility

Read every schedule, KIR, artifact, delivery, codec, cache, proof, and build consumer before choosing the encoding. Preserve legacy bytes only with an injectivity proof; otherwise step the owning domain or manifest schema deliberately and update ledgers and pins. Pre-alpha status is not permission to let an old reader reinterpret a new non-arithmetic record.

## Closes when

All construction and consumption sites are total over the new sum; missing, defaulted, and contradictory states fail closed; arithmetic records remain byte-for-byte or deliberately versioned; and independent subject perturbations prove every new discriminator and cross-check is load-bearing.
