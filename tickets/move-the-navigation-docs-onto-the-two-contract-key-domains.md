---
id: move-the-navigation-docs-onto-the-two-contract-key-domains
title: Move the navigation docs onto the two numerical-contract key domains
status: todo
priority: p2
dependencies: []
related: [state-and-check-a-bf16-numerical-contract]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, bf16]
---
## User-visible outcome

Every navigation document that names the numerical-contract key domain names
both of them, and no reader is left with the pre-BF16 statement.

## Why this is a separate ticket

**Fact.** `state-and-check-a-bf16-numerical-contract` added the sibling domain
`tiler.contract.bf16.v1` beside `tiler.contract.f32.v2`, updated the two
contracts it holds (`docs/numerical-semantics.md`,
`docs/correctness-and-testing.md`) and the identity ledger
(`docs/artifact-abi.md`, `contracts/foundation`), and could not reach
`contracts/navigation`: that scope was held exclusively by the live
`derive-the-operation-family-and-signature-delivery-graph`. Editing a scope
another live ticket holds is admissible only against a verified file-level
disjointness check, and that sibling's branch had no commits to check against,
so the edit was split out rather than taken on an empty verification.

## Scope keys

- `docs/status.md` states "The numerical-contract key domain is
  `tiler.contract.f32.v2`". It is now one of two, and the `bf16` domain was
  added rather than stepped, so no pin moved — say both.
- `docs/open-questions.md` Q-SEM-001's close note describes the key as the
  encoding "under `tiler.contract.f32.v2`". Same correction; the close itself
  still holds.
- `docs/dtype-support.md` should record that a BF16 *numerical contract* is now
  statable and checked, while BF16 *execution* remains unsupported — the
  distinction the compiler test asserts rather than assumes.
- Nothing here changes an identity or a decision; it is a stale-assertion sweep.

## Required evidence

- Every occurrence found by reading, not only by grepping the domain string:
  the exact check is `rg -n 'contract\.f32\.v2|numerical-contract key domain' docs/`,
  and each hit is read in place before editing.
- `tkt lint` green.

## Closes when

No navigation document asserts a single numerical-contract key domain, and the
BF16 statable/unsupported distinction is stated where dtype support is
catalogued.
