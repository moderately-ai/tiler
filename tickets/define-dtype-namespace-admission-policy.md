---
id: define-dtype-namespace-admission-policy
title: Define dtype namespace and admission policy
status: review
priority: p0
dependencies: []
related: [enumerate-the-mature-tensor-dtype-taxonomy, numerical-policy-contract]
scopes: [research/numerics, contracts/core]
shared_scopes: []
paths: []
tags: [tiler-research, numerics, governance]
claimed_from: todo
assignee: codex
lease_expires_at: 1784506819
---
Choose canonical namespace authority for standards-backed built-in dtypes, then finalize admission gates and the initial built-in/external/extension classification from docs/research/numerics/dtype-identity-admission-policy.md. Preserve one canonical key and keep recognition separate from execution support.

## Outcome

ADR 0034 selects Tiler-governed keys with mandatory normative references for
formats admitted into the built-in vocabulary. Published external identities
remain external; descriptors are immutable; incompatible meaning changes
require new key versions; exact equivalence is explicit and conformance-tested.
The remaining catalog classification can proceed under these rules.
