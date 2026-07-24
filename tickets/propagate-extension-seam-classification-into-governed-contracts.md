---
id: propagate-extension-seam-classification-into-governed-contracts
title: Propagate the extension-seam classification into governed contracts
status: todo
priority: p2
dependencies: [draft-public-extension-seam-ownership-adr]
related: []
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, extensions, public-api]
---
Conditional on Tom accepting ADR 0078. That record classifies which surfaces Tiler intends as public extension seams, which are permanently internal, the maturity rung each has reached, and — most of its content — what a seam is *not*. Until it is propagated, the classification lives only in a decision record and no governed contract states it, which is the state `implementation_status: "partial"` reports.

On acceptance, represent the classification in the contracts that own the affected areas, without creating a second authority over what ADR 0078 already decides:

- `docs/operation-extensions.md` owns the public capability surface and the trust, identity, registration, and diagnostic obligations of a provider. It should gain the seam classification and the negative-space rules that constrain a provider surface — offering nothing is a legitimate local result, a resolved provider's claim is re-derived rather than inherited, an unenumerated capability fails closed as `Unknown`, an absent capability and a contended one are different findings, a reservation is not a capability, and a provider revision is provenance rather than a version negotiation.
- `docs/architecture.md` owns component ownership and the packaging profile. It should record which authorities are permanently internal, and the qualification ADR 0078 makes about explain (internal authority, public obligation) and feasibility (internal procedure, with the target-profile data left explicitly undecided).

Do not restate ADR 0078's reasoning or its open questions in either contract; cite the record. Do not propagate anything ADR 0078 leaves unassigned — the physical-implementation provider and the mature fusion numerical capability are recorded as open questions and must not acquire an intent by propagation.

Run `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` before completion.
