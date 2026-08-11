---
id: admit-an-invocation-scoped-gather-index-validation-receipt
title: Admit an invocation-scoped gather-index validation receipt
status: todo
priority: p1
dependencies: [admit-the-selected-data-dependent-index-representation, admit-a-storage-carrier-for-integer-program-inputs]
related: [accept-the-invocation-scoped-gather-validation-public-surface, validate-device-resident-gather-indices-before-dispatch, admit-a-zero-copy-exclusive-lease-for-validated-gather-indices, generalize-invocation-bound-index-validation-beyond-gather]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/runtime, implementation/build, implementation/frontend, implementation/conformance, contracts/artifacts, contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, gather, validation, fail-closed, public-boundary, identity]
---
## User-visible outcome

An explicitly selected gather route over a host-visible U32 index input runs only after preflight validates the exact values and seals an immutable invocation snapshot; invalid, missing, stale, or mismatched evidence refuses before routing commit.

## Strict first-pass contract

- Static proof remains the first lane and requires no receipt.
- The only dynamic input is host-visible U32 storage. Preflight uses the governed `decide_gather_index` rule over every element and the exact gathered extent, then copies the validated values into immutable receipt-owned storage.
- The sealed receipt binds the exact gather occurrence, logical index type, extent, program binding, snapshot content, and invocation attempt. It is neither artifact identity nor timeless program proof, cannot be forged through public fields, and cannot be reused after any subject changes.
- The artifact and runtime carry a mandatory named validation obligation under the exact domain/version decision ADR 0108 accepts. A plan carrying it has no dispatch authority until the matching receipt is consumed.
- Every refusal is typed and explainable. Out-of-range is a semantic input error naming position, value, and extent—not a plan miss—and never causes clamp, wrap, reference execution, variant substitution, or backend fallback.
- Mutable zero-copy storage, device-resident or device-produced indices, validation callbacks, caller assertions, and inline kernel checks refuse as unsupported.

## Required evidence

Pin the complete obligation/receipt populations from their types. Perturb occurrence, extent, type, binding, snapshot bytes, invocation generation, missing receipt, duplicate consumption, post-validation mutation attempt, and every forbidden fallback independently with unchanged assertions. Prove the validated bytes are exactly the bytes dispatched.

## Closes when

The narrow host-visible path reaches preflight and one-way commit with no check/use gap; all excluded inputs fail closed; artifact, cache, explain, and public-boundary consequences are coherent; the exact surface is handed to its acceptance ticket; and targeted plus full gates pass.
