---
id: draft-public-extension-seam-ownership-adr
title: Draft a proposed ADR naming the intended public extension seams
status: in-progress
priority: p2
dependencies: []
related: [draft-public-api-conventions-adr, draft-public-boundary-approval-policy-adr, prototype-physical-implementation-frontier, prototype-operation-capability-registry]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, extensions, public-api]
claimed_from: todo
assignee: agent-draft-public-extension-seam-ownership-adr
lease_expires_at: 1784929885
---
Record, as a **proposed** ADR, which surfaces are *intended* to become public
extension seams at maturity and which are permanently internal. Today that
intent is implicit and inconsistent, so promotion is decided case-by-case at the
moment of promotion — the most expensive time to decide it.

## The live inconsistency (evidence)

Two structurally similar registries/authorities already differ with no recorded
reason: `tiler_compiler::capability` is `pub mod` (a public lowering-capability
registration surface), while `feasibility`, `fusion_legality`, `frontier`,
`cover`, and `selection` are private draft modules. The concrete open question
is `frontier`'s `PhysicalImplementationProvider` trait: it is the seam a physical
implementation provider implements, which is exactly the shape a third party
would plug into, yet it is `pub(crate)` today. Either answer is defensible; what
is not defensible is leaving it unstated until someone needs it.

## What to record (proposal)

For each candidate seam, state the intent and the consequence:

- **Intended public extension seam** — third parties may implement it, so it
  carries versioned identity, validation, feasibility, explainability, and a
  compatibility commitment (`AGENTS.md`: "extensible" must not mean unknown
  behaviour is optimizable). Candidates: operation/semantic registration,
  lowering capability, reference evaluation, and — open — physical
  implementation providers.
- **Permanently internal** — an authority the compiler owns, free to change
  shape without a compatibility story. Candidates: cover enumeration, plan
  selection, feasibility assessment, fusion legality, explain.

Say for each whether the mature form is expected to admit *third-party* providers
or only built-in ones registered through the same path, since that distinction —
not the `pub` keyword — is what creates the durable obligation.

## Decided 2026-07-24: the physical-provider trait is deliberately deferred, with a trigger

Tom was asked directly about `frontier`'s `PhysicalImplementationProvider` and,
after examining the reasoning, chose to defer it rather than classify it now.
Record it that way — as an explicit deferral with a trigger, not as an unexamined
gap:

- **Visibility stays `pub(crate)`.** Nothing forces a change, and — important
  context that invalidated an earlier argument — Tiler has no external consumers
  and is nowhere near an alpha, so there is **no compatibility cost in either
  direction**. Do not justify this deferral by reversibility or migration cost;
  that asymmetry does not exist yet. It is deferred because we lack the
  information, not because reversing would be expensive.
- **The design stays cross-crate-ready**, which costs nothing because it already
  is: the trait takes a versioned `ProviderIdentity`, hands the provider a
  read-only `ImplementationContext`, and has the host re-verify every proposal
  rather than trusting it.
- **The question that actually decides it** is prior and architectural: does
  target-specific scheduling knowledge live as typed **target-profile data**
  consumed by in-compiler providers, or as **code in backend crates**? Note this
  is not "third party versus us" — Rust visibility is per-crate, so even a
  first-party `tiler-metal` implementing providers would force `pub`. Current
  evidence leans toward data (target profiles are already typed data feeding
  feasibility and the frontier, and `tiler-metal` is scoped as pure
  structured-KIR→MSL lowering, downstream of scheduling), but that is an
  inference from an architecture no backend has ever exercised.
- **Trigger for reconsideration:** the first time a backend genuinely needs to
  contribute a physical implementation, or when the optimizer conformance gate
  pushes a provider through the ordinary compile path — whichever comes first.
  The Metal vertical answers this empirically, so guessing now buys nothing.

## Deliverable and boundaries

Create the ADR at the next free number with `decision_status: "proposed"`, its
`ticket` field pointing at this ticket, and each undecided seam listed as an
explicit open question rather than assigned by default. Do **not** mark it
accepted and do **not** change any visibility here: this ticket records intent
only. Actually promoting or demoting a surface is separate implementation work
that must satisfy the conventions and approval-policy ADRs, and may clarify or
amend the open questions this ADR leaves explicit.

Run `uv run --locked python scripts/docs.py render` and the full documentation
gate before completion.
