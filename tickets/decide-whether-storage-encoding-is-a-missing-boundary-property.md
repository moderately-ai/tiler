---
id: decide-whether-storage-encoding-is-a-missing-boundary-property
title: Decide whether storage encoding is a missing boundary property
status: todo
priority: p2
dependencies: []
related: [reconcile-dtype-cast-enforcer-with-boundary-properties, implement-boundary-property-model]
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, optimizer, physical-planning]
---
Surfaced while resolving `reconcile-dtype-cast-enforcer-with-boundary-properties`. That ticket established the admission test for the boundary-property list in `docs/compiler/optimizer.md`: a property qualifies when a producer can realize the same semantic value several ways and the choice is unobservable in the value. Resolved value dtype fails that test and is now named as a semantic trait rather than a property. Storage encoding appears to pass it and is absent from the list anyway.

The list currently holds storage layout class and contiguous axes, alignment and vectorizable width, materialized buffer / alias-view / opaque runtime value, and device and address space. "Storage layout class" plausibly covers row-major, strided, and blocked addressing. It is not obvious that it covers encoding: a sub-byte integer under ADR 0028 may be bit-packed or unpacked, and a quantized value under ADRs 0029 and 0030 carries companion scale and zero-point storage. Both choices preserve represented values, both are producer-side schedule decisions, and both would need an enforcer to reconcile a mismatch.

`docs/research/transfers/transfer-synchronization-and-resource-lifetime.md` already models this: its taxonomy separates `MaterializeLayout` ("same logical value and dtype; addressing/layout may change") from `RepackEncoding` ("explicitly changes storage encoding"), and ADR 0047 names "materialization/repacking" as one enforcer family, so repacking is already an accepted enforcer whose corresponding property the optimizer contract does not name.

Decide one of: storage layout class already subsumes encoding and the contract should say so; or encoding is a distinct boundary property owing the same satisfaction, subsumption, child-requirement-derivation, and dominance treatment alignment has, with repacking as its enforcer. If the second, check whether a quantized value's companion parameters are part of the same property or a separate one, since a scale tensor is a distinct value rather than an encoding of the quantized one.
