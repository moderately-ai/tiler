---
id: scope-the-complex-arithmetic-vertical
title: Scope the complex arithmetic vertical
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, enumerate-the-mature-tensor-operation-and-signature-taxonomy, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, complex]
---
## User-visible outcome

`tiler::complex@1<ComponentTypeKey>` has an owner for the step past identity, and a reader can tell that an ordering comparison over it is intentionally invalid rather than merely unsupported.

## Why this exists

**Fact.** [ADR 0037](../docs/decisions/0037-parameterize-complex-dtype-identity.md) recognizes the parameterized family and keeps planar versus interleaved storage physical. [The dtype support ledger](../docs/dtype-support.md) records the constructor registered with its ordered real-then-imaginary component contract over exactly f16, f32, and f64, every other component including `complex<bf16>` and nested complex refused by typed reason, and no operation admitting it.

**Fact — the family's first requiring consumer is on the operation axis.** [The mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-40 spectral transforms is "the first family that *requires* the complex identity to be more than recognized", because a real transform's result is complex. Its `RQ-OP-12` for F-42 dense linear-algebra decompositions also carries complex operands and concludes that the family is consumer-owned by derivation.

**Fact — one obligation is a refusal, not a feature.** An ordering comparison over a type with no defined total order is on that record's **intentionally invalid** list, and the corpus's `ULP_FORMAT_RULES` already refuse an accuracy contract over an unordered type. This track must not be read as owing either.

**Inference.** Complex is the second compound value in the catalog after the quantized families: one logical value whose ordered components may occupy one buffer or two. That is the same structural break the ledger's dry run records for MX at rung 9, arrived at from a different direction, so the storage decision is not a detail that can wait for lowering.

## Activation trigger

A named operation and component type, plus the branch-cut, exceptional-value, accuracy, storage, ABI, target, and conformance choices [the ledger's trigger](../docs/dtype-support.md) already enumerates.

## Closes when

The trigger has fired and the vertical is stated with its component oracle, its planar-or-interleaved storage decision, and the exact operations it admits — or complex is explicitly excluded from the intended product surface by a recorded decision.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-6 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).

## Trigger check log

- 2026-08-04 — **not fired.** Track D-6's trigger is checked under `#### D-6 — Complex` in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md): no named operation and component type has arrived, and F-40 spectral transforms — the first family that would require it — is unadmitted.
- 2026-08-09 — **not fired.** The governed `tiler::complex@1` constructor still admits the same f16/f32/f64 component identities, but no operation accepts a complex value and no named workload supplies the branch-cut, storage, ABI, target, and conformance choices the trigger requires. Catalog recognition is not operation admission.
