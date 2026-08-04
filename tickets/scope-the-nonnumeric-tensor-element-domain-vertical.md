---
id: scope-the-nonnumeric-tensor-element-domain-vertical
title: Scope the nonnumeric tensor element domain vertical
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, nonnumeric]
---
## User-visible outcome

String and bytes, object and variant, temporal, structured and record, and categorical or dictionary domains have one owner, so a future frontend that needs one finds a mapped question rather than an unconsidered gap.

## Why this exists

**Fact.** [The mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md)'s `## Nonnumeric tensor element domains` classifies each: string tensors are "a genuine tensor element domain in systems such as ONNX, but requires offsets/buffers and a separate operation family"; object and variant are runtime-managed or opaque; temporal is a parameterized semantic domain over integer storage with a separate operation family; structured and record is a compound schema rather than one scalar arithmetic dtype; and categorical or dictionary is an encoded relational domain rather than primitive integer semantics. Recognizing them "does not require admitting them to the initial tensor-kernel optimizer".

**Fact.** [The dtype support ledger](../docs/dtype-support.md) records them as type-system reservations at recognized identity and physical carrier, and `absent/unsupported` everywhere else.

**Inference — they are one track because their shared obligation precedes all their differences.** Each needs offsets, variable-length buffers, lifetimes, and its own operation family before any of them is a tensor element at all. That obligation is not the numeric tensor ABI, and answering it once is what the track buys.

## Activation trigger

A named frontend or product workload requires the exact domain and can define its operation and lifetime contracts. **Numeric dtype breadth does not fire it**, which the ledger's trigger states directly.

## Closes when

The trigger has fired and the selected domain's operation, lifetime, storage, and ABI contracts are stated — or the domains are explicitly excluded from the intended product surface by a recorded decision.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-14 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- Non-tensor graph values are **not** this ticket's. Tokens, resources, handles, PRNG keys, shapes, indices, tuples, futures, and control values are owned on other axes; that record's `## Families routed off the dtype axis` names each owner.
