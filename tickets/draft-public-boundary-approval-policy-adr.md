---
id: draft-public-boundary-approval-policy-adr
title: Draft a proposed ADR for the public-boundary approval policy
status: todo
priority: p1
dependencies: [draft-public-api-conventions-adr]
related: [draft-public-extension-seam-ownership-adr]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, governance, process]
---
Record, as a **proposed** ADR, which changes require Tom's explicit review before
merge and which a coordinator may merge autonomously. `AGENTS.md` today says Tom
must review "key public crate, module, trait, type, and call-site boundaries"
without defining the boundary of *key*, so every ambiguous case costs a
round-trip and the ambiguity is resolved by judgement rather than by rule.

Depends on `draft-public-api-conventions-adr` because the autonomous half of the
policy is meaningless until "conforms to the conventions" names something
checkable.

## Why now (evidence)

Across the authorities landed so far, three changes went to Tom (a new crate, a
new public module, one public method plus its error type) and all three were
approved essentially as designed; four others never needed him because they were
`pub(crate)` private drafts. The single substantive review catch was a missing
`#[non_exhaustive]` — a convention gap. So per-case approval had a low catch rate
relative to its latency, and its one catch is better served by a written
convention. That argues for a policy that spends Tom's attention on genuine
compatibility commitments instead of on conformance.

## Policy to record (proposal — the exact boundary is the decision)

- **No approval required:** a new compiler-internal authority introduced as a
  `pub(crate)` draft; additive `#[non_exhaustive]` growth; a new public error,
  provenance, or identity *record* that conforms to the conventions ADR; tests;
  documentation.
- **Always requires Tom:** a new crate; a new public **trait** (an extension seam
  third parties implement, so it is a durable compatibility commitment); any
  breaking change to an existing public signature; promoting a module or type
  from `pub(crate)` to `pub`.

Record explicitly that a coordinator's terminal-merge authority is conditional on
the objective gates that already exist — a green `scripts/check_repository.py`, a
`ticketsplease guard` with no scope escape, scope conformance, and a full review
of the actual diff rather than an agent's summary — and that any of those failing
returns the change to Tom regardless of category.

## Deliverable and boundaries

Create the ADR at the next free number with `decision_status: "proposed"`, its
`ticket` field pointing at this ticket, the evidence above in Context, and every
unsettled boundary listed as an explicit open question. Do **not** mark it
accepted, and do **not** edit `AGENTS.md` here: if the policy is accepted,
propagating it into the working contract is a follow-up so that a *proposed* ADR
never silently becomes the operative rule.

Run `uv run --locked python scripts/docs.py render` and the full documentation
gate before completion.
