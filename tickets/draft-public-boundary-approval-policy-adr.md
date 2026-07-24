---
id: draft-public-boundary-approval-policy-adr
title: Draft a proposed ADR for the public-boundary approval policy
status: in-progress
priority: p1
dependencies: [draft-public-api-conventions-adr]
related: [draft-public-extension-seam-ownership-adr]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, governance, process]
claimed_from: todo
assignee: agent-draft-public-boundary-approval-policy-adr
lease_expires_at: 1784908598
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

## Policy to record — **Tom decided this boundary on 2026-07-24**

The exact split below is no longer a proposal to survey: Tom was asked the
boundary directly and chose it, over a tighter variant (also bring him every new
public *type*) and a looser one (let the coordinator promote `pub(crate)` to
`pub` unaided). Record it as his decision with that context, and do not re-open
the alternatives as though undecided — note only that the looser variant was
declined because promotion is the moment a surface becomes externally
load-bearing, and may be revisited once the optimizer conformance gate has
actually exercised a seam end to end.

This ticket's dependency is satisfied: ADR 0074 (the conventions) was accepted on
2026-07-24, so "conforms to the conventions" now names something checkable, which
is what makes the no-approval half of this policy safe rather than open-ended.

- **No approval required:** a new compiler-internal authority introduced as a
  `pub(crate)` draft; additive `#[non_exhaustive]` growth; a new public error,
  provenance, or identity *record* that conforms to the conventions ADR; tests;
  documentation.
- **Always requires Tom:** a new **publicly reachable namespace** — a new crate,
  or a new `pub mod` in a crate root or in an already-public module; a new public
  **trait** (an extension seam something else implements); any breaking change to
  an existing public signature; promoting a module or type from `pub(crate)` to
  `pub`.

### Amendment 2026-07-24: the namespace reformulation

The first item originally read only "a new crate", which was **narrower than the
practice it was calibrated against**: `tiler_ir::schedule` and `tiler_ir::kernel`
were both new public *modules* with large surfaces (the latter ~4,600 lines) and
both went to Tom under the prior judgement-based rule, yet the literal wording
would not have required it. `AGENTS.md` already lists "module" alongside crate,
trait, and type, so the omission was a drafting artifact rather than a considered
narrowing.

Tom was asked directly and accepted the reformulation above over the alternative
"a new public module with a *substantial* surface" — declined because
"substantial" reintroduces exactly the judgement term that makes `AGENTS.md`'s
existing "key public … boundaries" ambiguous, which is the ambiguity this policy
exists to remove. Record the accepted cost explicitly: a trivial two-item
`pub mod` now also requires review, judged acceptable because such modules are
expected to be rare.

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
