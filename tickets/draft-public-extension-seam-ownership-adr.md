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

## Outcome

[ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md) is drafted at `decision_status: "proposed"`, `implementation_status: "partial"`, against base `412ceae`. Tom accepts; nothing in it is operative until he does. It changes no visibility, no signature, and no behaviour.

**What moved since this ticket was filed.** The seams stopped being hypothetical. `pipeline::compile` now calls `lowering::resolve_lowering` unconditionally, `governed.rs` registers Tiler's four families through the same public `LoweringCapabilityRegistryBuilder` an external provider uses, `GovernedPhysicalProvider` is an ordinary implementor of `frontier::PhysicalImplementationProvider`, and the conformance block in `pipeline.rs` drives both an externally registered semantic operation set and an externally registered index-access lowering provider through `compile()`. The record therefore describes what exists rather than proposing a shape.

**What it settles.** The governing principle is that a seam is a propose-then-re-verify boundary: a provider proposes, and Tiler re-derives every fact the proposal would otherwise assert — output re-enters the ordinary checked path, provenance is stamped by the host rather than the provider, identity is versioned per ADR 0072, and every disposition is explainable. An inventory names each seam, the participation model it is intended to admit, and the highest of `AGENTS.md`'s four maturity claims it has actually reached. Most of the record is the negative space: offering nothing is a legitimate local result distinct from both a rejection and a compiler fault; a resolved provider's claim is re-derived rather than inherited; an unenumerated capability fails closed as `Unknown`; an absent capability and a contended one are different findings and neither is a preference; an exhausted proof budget is an `Unknown` gap that leaves the plan standing; a provider revision is provenance and not a version negotiation; a reservation is not a capability; and the `pub` keyword is neither necessary nor sufficient, because the tree contains all four combinations.

**The deferral is recorded as Tom decided it.** `frontier::PhysicalImplementationProvider` stays `pub(crate)` and unclassified, deferred for missing information rather than migration cost, with the prior data-versus-code question carried as an open question. One correction: the trigger's second clause — "when the optimizer conformance gate pushes a provider through the ordinary compile path" — is now ambiguous, because the gate pushed an external *lowering* provider while `enumerate_frontier` still runs with one in-crate physical provider. The record proposes the sharpened form (a physical implementation provider defined outside `tiler-compiler` reaching `enumerate_frontier` through `compile()`) and flags explicitly that this is a sharpening, not a reversal, and that if Tom reads the original clause as fired the deferral is ripe now.

**Two seams are deliberately left unassigned**, each with a recorded reason rather than a default: the physical-implementation provider, and the mature per-operation fusion numerical capability, which the operation-extension contract already reserves while `FusionNumericalCapabilities` is a compiler-owned table with no registration path and a `ProviderIdentity` used purely for attribution.

**Measurement boundary the record states rather than rounds up.** `tiler_compiler::capability` is `pub` while `CompilerCapabilitySnapshot`, `CompilationRequest`, its `capabilities` field, and `governed_index_access_capabilities` are all `pub(crate)` and the crate exports no entry point. An out-of-crate consumer can build a capability registry today and has no public way to install it, so the lowering seam is a tested guarantee for emit-and-refine and an architectural seam only for out-of-crate installation. That asymmetry is left as convention 7 working as designed, closing when `prototype-public-compiler-api` lands.

**Follow-ups filed.**

- [`correct-roadmap-capability-wiring-claims`](correct-roadmap-capability-wiring-claims.md) — `docs/roadmap.md` states in two places that the capability registry has no production caller and that no governed provider registers a capability. Both are false at `412ceae`. `contracts/navigation`, outside `propagate-lowering-capability-wiring-into-contracts`'s scopes.
- [`propagate-extension-seam-classification-into-governed-contracts`](propagate-extension-seam-classification-into-governed-contracts.md) — conditional on acceptance, so a proposed ADR never becomes operative by default.
- [`test-two-revisions-of-one-provider-as-a-capability-ambiguity`](test-two-revisions-of-one-provider-as-a-capability-ambiguity.md) — the two-revision behaviour is derived from `LoweringCapabilityKey`'s composition and `resolve`'s filter and is labelled Inference in the ADR because no test pins it.

**Gate.** `uv run --locked python scripts/docs.py render` passed (181 records, regenerating the ADR topic catalog and chronology from frontmatter). `uv run --locked python scripts/check_repository.py` passed. `git diff --check` clean. `ticketsplease lint` clean.
