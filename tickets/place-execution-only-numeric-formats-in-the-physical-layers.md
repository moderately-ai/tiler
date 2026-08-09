---
id: place-execution-only-numeric-formats-in-the-physical-layers
title: Place execution-only numeric formats in the physical layers
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, scope-the-block-scaled-compound-value-vertical, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, target-profiles]
---
## User-visible outcome

TF32, the PTX `.ue4m3` and `.ue8m0` scale encodings, `x86_fp80`, and `ppc_fp128` are statable as physical facts where a backend needs them, and remain unstatable as logical element identities.

## Why this exists

**Fact.** [The mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md)'s `### Target ABI and execution-only floating formats` classifies TF32 as a compute or operand precision contract, `.ue4m3` as a target scale-data format distinct from the signed `f8E4M3*` logical elements, `x86_fp80` as a target ABI extended format that is not binary128, and `ppc_fp128` as a double-double that is not binary128 either. Its conclusion 7 keeps them in operation and physical compute contracts "unless an explicit tensor interchange use case proves otherwise".

**Fact.** [The dtype support ledger](../docs/dtype-support.md)'s `### Execution-only formats` records an architectural seam at the numerical contract and type-system reservations at the physical carrier and kernel-vocabulary layers, with recognized identity deliberately `absent/unsupported`.

**Inference.** This track's whole content is *where* such a fact lives, not what the element semantics are, because these formats have no element semantics to give. That is why it is one track over four unrelated formats: they share exactly one obligation, and it is a placement obligation.

## Activation trigger

A selected backend operation needs one of them **and** can state its conversion boundaries, its delivered numerical behaviour, its target detection, its artifact identity, and its refusal. Promoting any of them into logical identity requires a separate semantic decision that this ticket does not pre-authorize.

## Closes when

The trigger has fired and the physical fact is stated at the layer that owns it with its conversion boundary and refusal, and no logical identity was minted.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-11 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- A target-profile or Metal change this eventually needs is that owner's, not this ticket's; declare the scope when the trigger fires rather than reserving it now.

## Trigger check log

- 2026-08-04 — **not fired.** Track D-11's trigger is checked in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md):217: no selected backend operation needs TF32, `.ue4m3`, `.ue8m0`, `x86_fp80`, or `ppc_fp128`, and none could yet state its conversion boundaries, delivered behaviour, target detection, artifact identity, and refusal.
- 2026-08-09 — **not fired.** A fresh source and ledger read still finds these names only in the research taxonomy, the support ledger, and the semantic-catalog negative census; no selected backend operation, target-profile fact, kernel type, or physical carrier names one. The trigger therefore remains a real selected physical consumer, not the existence of the taxonomy row.
