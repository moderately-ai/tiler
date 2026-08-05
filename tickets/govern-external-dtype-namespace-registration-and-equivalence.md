---
id: govern-external-dtype-namespace-registration-and-equivalence
title: Govern external dtype namespace registration and equivalence
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, define-dtype-namespace-admission-policy, register-the-accepted-built-in-dtype-catalog, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, governance, extensions]
---
## User-visible outcome

An external or vendor-owned dtype identity can be registered, collided with, and proven equivalent to a built-in through versioned conformance — or refused by name — instead of being governed by a policy nothing exercises.

## What is settled, and what is not

**Fact — the ownership direction is settled.** [ADR 0034](../docs/decisions/0034-tiler-governed-built-in-dtype-keys.md) fixes that built-ins use Tiler-governed keys carrying mandatory normative references, that an already-published external canonical identity is supported in place and never rekeyed, and that exact external equivalence is explicit, versioned, and conformance-tested.

**Fact — registration is not.** [The admission policy](../docs/research/numerics/dtype-identity-admission-policy.md) closes with: "Namespace registration and collision governance for external providers remain an API-design task, but the ownership direction is fixed." ADR 0034's own realization section records the same split from the other side — no external identity, alias table, or equivalence evidence exists to exercise the policy, and no same-format owner check runs before minting a key, so the correctly-external OCP spellings are preserved by a test asserting non-registration rather than by an admission check.

**Inference — the ledger's dry run compresses this and a reader can be misled.** [The dtype support ledger](../docs/dtype-support.md)'s vendor column reads "**fails: no vendor namespace policy exists**" at rung 1. That is right about registration and wrong if read as covering ownership, which ADR 0034 decided.

## Members this track owns

`f8E3M4` and `f8E4M3` in the MLIR/StableHLO IEEE convention; the FNUZ variants; IBM HFP8 `f8E4M3B11FNUZ`; NVFP4 and other vendor block recipes for their *identity* half; GGML-family and other project codecs; and learned or codebook quantizers with no admitted canonical descriptor. Similar spelling, equal width, or a matching descriptor shape is never equivalence.

## Why it is deferred rather than open

Two independent reasons, either sufficient. The registration and collision-governance design is an API-design task and therefore a public boundary reserved to Tom under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md). And the same-format owner check ADR 0034 requires is **vacuous** while no external identity exists to collide with, so building it now would produce a check that cannot fail — the shape this repository's own process rule warns about.

## Activation trigger

A real consumer publishes a stable owner-namespaced identity with an immutable descriptor, a normative reference, encode and decode vectors, an operation set, storage and ABI, runtime refusal rules, target evidence, and versioned conformance — **or** a proposal arrives to mint a built-in key for a format an external owner already publishes, which is the case the missing owner check exists to refuse.

## Closes when

The trigger has fired, the registration and collision boundary is accepted by Tom, the same-format owner check runs before minting and is watched refusing a real collision, and equivalence is established by versioned conformance rather than by descriptor resemblance.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-13 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- [`route-the-reserved-numeric-families-through-the-extension-boundary`](route-the-reserved-numeric-families-through-the-extension-boundary.md) depends on this, because the extension boundary is the only route those families have.
