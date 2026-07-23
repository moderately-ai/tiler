---
id: define-implementation-status-for-superseded-decisions
title: Define implementation status for superseded decisions
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, governance, metadata]
claimed_from: todo
assignee: agent-define-implementation-status-for-superseded-decisions
lease_expires_at: 1784834740
---
The ADR status audit found a corpus inconsistency it correctly declined to fix, because the metadata contract does not define the case. ADR 0057 carries `decision_status: "superseded"` together with `implementation_status: "implemented"`, but the state it claims to have implemented — a workspace `rust-version = "1.89"` floor — is no longer true: ADR 0067 replaced it with the pinned nightly, and no `rust-version` is declared in the workspace or any member manifest. Verified 2026-07-23.

The underlying question is a governance one and belongs to `docs/document-metadata.md`: what does `implementation_status` mean once `decision_status` is `superseded`? Three readings are defensible and the contract currently permits all of them — that the field records what was historically built (so "implemented" stays and is read historically), that it records the present state of the codebase (so a superseded decision whose implementation was replaced should not read "implemented"), or that the field is meaningless on a superseded record and should be constrained or dropped.

Decide one reading, state it normatively in the metadata contract, enforce it in `scripts/docs.py` where it is mechanically checkable, then apply it to ADR 0057 and audit the other superseded decision (ADR 0056) for the same class of drift. Do not simply flip 0057's value without settling the rule, or the next audit will reopen it. Run the full documentation gate before completion.
