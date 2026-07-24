---
id: propagate-accepted-api-conventions-into-governed-contracts
title: Propagate accepted ADR 0074 conventions into the contracts it governs
status: in-progress
priority: p2
dependencies: []
related: [draft-public-api-conventions-adr, harden-public-enums-non-exhaustive, extend-canonical-identity-encodings-for-reserved-variants, disambiguate-presentation-label-from-semantic-key-accessors]
scopes: [contracts/foundation]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, public-api, contracts]
claimed_from: todo
assignee: agent-propagate-accepted-api-conventions-into-governed-contracts
lease_expires_at: 1784912789
---
ADR 0074 is now **accepted**, and declares
`applies_to: ["tiler.contract.ir", "tiler.contract.architecture"]`. The typed
governance edge exists and the documentation gate validates it structurally, but
a passing gate does not check that the destination contracts' **prose** actually
states the rules the ADR now governs. An accepted decision whose destination
contract is silent is exactly the "conflicting terminology / stale status" drift
the documentation contract warns about.

Check each destination and propagate what is genuinely normative *there*:

- `docs/ir.md` — its "Shared IR construction lifecycle" section is where the
  construction, identity, encoding, and error conventions become normative for
  the shared IR.
- `docs/architecture.md` — owns component boundaries, which is where the rule
  about when a module may be `pub` versus a crate-private draft authority
  belongs.

Do **not** duplicate the ADR into the contracts. The ADR is the decision and its
reasoning; a contract states the resulting rule and cites the ADR. Restate only
the conventions that are actually normative for that document — if only some of
the seven apply to a given contract, say only those, and record why the others do
not belong there rather than copying all seven for symmetry.

Leave ADR 0074's open questions alone: the descriptor-accessor style is owned by
`unify-schedule-index-region-with-verified-index-region`, the `key()` naming
hazard by `disambiguate-presentation-label-from-semantic-key-accessors`, and
whether ADR 0074 should also name `tiler.contract.optimizer` is deliberately
deferred until a reviewed compiler facade exists. If propagation reveals that a
convention is wrong or unstatable in its destination, amend the ADR explicitly
rather than quietly weakening the contract text.

Run `uv run --locked python scripts/docs.py render` and the full documentation
gate before completion.
