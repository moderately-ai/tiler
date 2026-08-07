---
id: correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence
title: Correct the scope-set claim in ADR 0106's missing-component evidence
status: in-progress
priority: p3
dependencies: []
related: [survey-what-belongs-in-the-conformance-crate, record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, conformance, correction]
claimed_from: todo
assignee: agent-adr-fix
lease_expires_at: 1786128633
---
## User-visible outcome

[ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md)'s Context states its evidence for the missing-component claim correctly, so a reader checking the argument finds it holds rather than finds it self-contradicting in the same sentence.

## The defect

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md), which was asked to test the claim and read the tickets.

ADR 0106's Context reads:

> **Fact — the evidence that a component is missing rather than a file being homeless is that the work does not share a scope set.** [...] counts five open conformance tickets and no two share one: three are `implementation/compiler`, one adds `implementation/reference`, `contracts/numerics`, and `research/scheduling`, and one adds `implementation/runtime`.

**The clause "no two share one" is falsified by the clause after it, and by the tickets.** Read from the tickets on `main`:

- `route-the-contraction-conformance-through-the-staged-oracle` — `scopes: implementation/compiler`, `shared: project/tickets`
- `route-the-index-region-conformance-through-the-staged-oracle` — `scopes: implementation/compiler`, `shared: project/tickets`
- `retain-the-selected-semantic-candidate-for-the-conformance-oracle` — `scopes: implementation/compiler`, `shared: project/tickets`

Three of the five carry **identical** scope sets. Reproduce with `tkt show <id>` on each.

## Why it is worth correcting rather than ignoring

The sentence is the ADR's stated ground for admitting a workspace member. A reader auditing the admission finds the evidence self-contradicting at the point it is offered, which is exactly the failure `AGENTS.md` names when it says comments and examples are claims about current behaviour.

## The correction is a rewording, not a retraction — the conclusion survives

What is true, and what the surrounding sentences already say, is that the five tickets **span** five distinct scopes across three crates and two documentation contracts with no scope common to all of them, and that the three sharing a set are three tickets about **one compiler-resident file** rather than three independent pieces of scattered work. The survey found the underlying claim understated rather than overstated: `grep -ril conformance tickets/` returns 289 files of which 283 are tickets, 76 of those are non-terminal, and the crates holding conformance-named source are `tiler-ir`, `tiler-reference`, `tiler-compiler`, and `tiler-conformance`.

## Closes when

The Context's scope-set sentence states what the five tickets' scopes actually are, the three-identical-sets fact is stated rather than contradicted, and the conclusion it supports is preserved with its rationale intact per `AGENTS.md`'s supersession rule.
