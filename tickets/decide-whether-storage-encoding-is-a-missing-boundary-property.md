---
id: decide-whether-storage-encoding-is-a-missing-boundary-property
title: Decide whether storage encoding is a missing boundary property
status: done
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

## Outcome

**Decided: storage encoding is a distinct boundary property, not subsumed by storage layout class.** `docs/compiler/optimizer.md` now names it in the boundary-property list and names encoding repacking beside contiguous materialization and layout conversion in the enforcer list.

**Why not subsumption.** Layout class answers which logical coordinate maps to which position; encoding answers how one element is represented at that position. They vary independently, and the counterexample is concrete: a blocked layout of bit-packed `u4` and a blocked layout of unpacked `u4` share a layout class and differ in encoding. No layout class can express that difference, so a contract that relied on subsumption would have no way to state the requirement or to check that a producer met it.

**It passes the admission test that excludes dtype.** `reconcile-dtype-cast-enforcer-with-boundary-properties` established the test: a property qualifies when a producer can realize the same semantic value several ways and the choice is unobservable in the value. Packed and unpacked `u4` represent the same values; a narrowing dtype change does not. The contract's own reason for excluding dtype — that satisfaction is subsumption and the dtype analogue is the erased narrowing ADRs 0009 and 0010 forbid — does not apply, because neither encoding erases anything.

**The decisive evidence was that the enforcer already exists.** ADR 0047 names "materialization/repacking" as an enforcer family, and the transfer taxonomy separates `MaterializeLayout` from `RepackEncoding` and keeps both distinct from `ConvertDtype`. Stronger still, `TransferStage` carries an explicit `semantics: PreserveStorageEncoding` field — a transfer would have no reason to *declare* that it preserves encoding unless encoding were a dimension it could otherwise change. So an accepted enforcer was supplying a property the optimizer contract did not name, which is the gap this closes rather than a new mechanism it introduces.

**Subsumption is stated per encoding family, not assumed.** Alignment's ordering ("16-byte satisfies 4-byte") does not transfer: an unpacked producer does not satisfy a packed requirement by being cheaper to read, and a packed one does not satisfy an unpacked requirement by being denser. Recording encoding as an ordered property by analogy with alignment would have been the same class of error as admitting dtype.

**The companion-parameter question, answered.** The ticket asks whether a quantized value's scale and zero-point storage is part of this property or a separate one. Neither: it is not a boundary property at all. `docs/ir.md` makes a quantized tensor "one first-class semantic tensor value even when its runtime representation has several components", with the versioned scheme, component roles, and coordinate maps in its *static type contract*, and with component tensors entering as ordered operands to an explicit assembly or conversion operation. A schedule that added, dropped, or re-roled a component would change which values the boundary carries, which the enforcer rule forbids outright. What stays physical is that "physical packing and addressing remain storage decisions" and that artifact lowering "may expand one logical quantized argument or result into several verified physical bindings" — encoding and layout applied per component. So the obligation this creates is that the properties are stated per component of a multi-component value, not that a further property names the companions.

**Evidence.** `uv run --locked python scripts/docs.py render` passes at 181 records; full repository gate green. This is a contract decision with no implementation: `implement-boundary-property-model` owns realizing the property, and the per-component obligation above is a constraint on it rather than a claim that it holds today.
