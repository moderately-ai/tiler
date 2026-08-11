---
id: frame-provider-identities-before-using-them-as-explain-keys
title: Frame provider identities before using them as explain keys
status: todo
priority: p1
dependencies: []
related: [reconcile-the-operation-identity-and-governed-key-grammars, replace-flat-selected-lowering-capability-keys-with-structured-subjects, emit-typed-opaque-call-frontier-rejection-records]
scopes: [implementation/ir, implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [identity, explain, diagnostics, correctness, public-boundary]
---
## User-visible outcome

Explain evidence names every registered provider injectively and accepts the full provider-identity vocabulary rather than collapsing dotted component boundaries or refusing a legal provider only because its rendered key exceeds an unrelated display bound.

## Why this slice exists

**Fact at `b03d1e7699d4f7cfbfb6ee7a903e2d2fbe16af18`.** `ProviderIdentity` validates and canonically encodes namespace and name as distinct length-framed components. `ProviderRef::registered` in `crates/tiler-compiler/src/explain.rs` instead joins them with `format!("{}.{}", namespace, name)` and validates the result against the explain key's 255-byte ceiling.

**Inference.** Legal providers `("a.b", "c")` and `("a", "b.c")` therefore become one explain key. Two maximum legal components can also be refused after registration. Explain text is not artifact identity, but retained evidence must still name the authority it claims without collision or late refusal.

## Required delivery

- Perform the repository-required source-first per-Fact audit at the implementation base; the Fact above is stale until then.
- Replace the delimiter-composed provider key with a structured or opaque received-identity carrier whose canonical bytes preserve the namespace/name boundary and revision.
- Keep human-readable rendering separate from equality and canonical explain evidence.
- Reconcile explain encoding/version/pins and every provider-ref consumer without narrowing `ProviderIdentity` or introducing a default/fallback provider.
- Perturb the dotted boundary pair, maximum component sizes, and revision independently with assertions unchanged, and quote each failure.

## Non-goals

Changing artifact provider provenance, provider installation/selection policy, provider precedence, or the selected lowering capability subject owned by the related implementation ticket.

## Closes when

Every legal registered provider can be represented in explain evidence, distinct structured identities stay distinct, all encoded evidence and pins reconcile, and independent review confirms presentation text is not authority.
