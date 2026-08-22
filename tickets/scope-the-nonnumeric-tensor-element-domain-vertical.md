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

**Fact.** [The dtype support ledger](../docs/dtype-support.md) records them as type-system reservation at recognized identity, physical carrier, and ABI/materialization; `absent/unsupported` on every other maturity column.

**Inference — they are one track because their shared obligation precedes all their differences.** Each needs offsets, variable-length buffers, lifetimes, and its own operation family before any of them is a tensor element at all. That obligation is not the numeric tensor ABI, and answering it once is what the track buys.

## Activation trigger

A named frontend or product workload requires the exact domain and can define its operation and lifetime contracts. **Numeric dtype breadth does not fire it**, which the ledger's trigger states directly.

## Closes when

The trigger has fired and the selected domain's operation, lifetime, storage, and ABI contracts are stated — or the domains are explicitly excluded from the intended product surface by a recorded decision.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-14 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- Non-tensor graph values are **not** this ticket's. Tokens, resources, handles, PRNG keys, shapes, indices, tuples, futures, and control values are owned on other axes; that record's `## Families routed off the dtype axis` names each owner.

## Trigger check log

- 2026-08-04 — **not fired.** Track D-14's trigger is checked under `#### D-14 — Nonnumeric tensor element domains` in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md) (trigger paragraph): no named frontend or product workload requires string, object/variant, temporal, structured, or categorical domains, and numeric dtype breadth is the stated anti-trigger.
- 2026-08-09 — **not fired.** The active frontend and conformance work still binds numeric and predicate-shaped tensor programs only; no named consumer requires string/bytes, object/variant, temporal, record, or categorical tensor elements with operation and lifetime contracts.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **Checkable half.** `rg -n 'pub fn \w+_op\(\) -> OpKey' crates/tiler-ir/src/semantic --glob '*.rs'` reports the **19** registered operation-key constructors, and `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` reports **50 unique governed keys** — unique keys through `sort -u`, not lines of output, and no key in either census names a string, bytes, object, variant, temporal, record, or categorical element domain — every governed key is a numeric or predicate identity or an operation over one. A key naming one of those domains is the changed answer, and the check is one-directional: it can say *not fired* and cannot say *fired*. **This condition is not mechanically checkable, and saying so is the repair.** The trigger proper is a named frontend or product workload that requires the exact domain **and can define its operation and lifetime contracts**, which is a design commitment rather than a repository state. A human must read `docs/research/numerics/mature-dtype-taxonomy.md`'s `## Nonnumeric tensor element domains` and `docs/dtype-support.md`'s trigger for a consumer that has supplied those contracts. Numeric dtype breadth explicitly does not fire it, so a widened key census is not evidence here. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
