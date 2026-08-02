---
id: draft-an-adr-for-the-inline-delivery-statement
title: Draft a proposed ADR recording the accepted inline `deliver` statement
status: in-progress
priority: p2
dependencies: [accept-the-inline-artifact-family-profile-syntax]
related: [generate-cfg-gated-artifact-family-delivery, prototype-inline-proc-macro-frontend]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, inline-dx, frontend, apple-targets]
claimed_from: todo
assignee: agent-deliver-adr
lease_expires_at: 1785696202
---
## Why this exists

Tom accepted a consumer-visible spelling on 2026-07-31 under
[`accept-the-inline-artifact-family-profile-syntax`](accept-the-inline-artifact-family-profile-syntax.md):
an inline region states its artifact-family delivery policy with a `deliver`
statement in the declaration block, as `deliver <profile>;` or
`deliver <family> <major>.<minor>, …;`, with the profile vocabulary
`fallback-only`, `macos`, `ios`, `macos-and-ios`. The acceptance and its grounds
live in that ticket's Decision section; [the frontend
contract](../docs/integration/frontends.md) states the resulting spelling and
what each production resolves to.

That is a durable contract, so the question it closed is closed. What it is not
is an ADR, and the precedent says one belongs here: the *neighbouring* accepted
consumer-visible spelling — the expansion cache root, from the other half of the
same contract — got [ADR
0089](../docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md)
on the same day. A ticket's Decision section is not indexed by
`docs/decisions/README.md`, is not reachable from a contract's frontmatter, and
becomes hard to find once the ticket is terminal, which is exactly the failure
mode `AGENTS.md` names when it says a decision recorded in a ticket outcome
should be lifted into an accepted ADR when it outlives the ticket.

**This is a records question, not a reopening.** The syntax is accepted and
implemented; nothing here may change what a consumer writes.

## The work

Draft a **proposed** ADR recording the decision as taken. It must state:

- the two accepted productions and the exact profile and family vocabularies;
- that the statement's absence is `fallback-only` rather than an unstated
  policy, and why that keeps a region without one token-identical;
- why `ios` names the device and the simulator together rather than publishing
  the driver's `ios-device` and `ios-simulator` identifiers;
- why the escape hatch exists at all — a profile fixes each family to its
  governed floor, so a consumer needing a higher one would otherwise wait for a
  new profile to be minted;
- the eliminated alternatives and their grounds, taken from the deciding
  ticket rather than re-derived: a profile alone (no floor override), a family
  list alone (publishes Apple vocabulary on the mandatory path and restates
  floors the driver governs), and the `#[tiler::deliver(macos)]` attribute form
  (a `#[proc_macro]` cannot see attributes outside its own token stream, so it
  would need a second macro entry point and would break the accepted
  self-contained-invocation property);
- that the surface leaves room for the second axis the frontend contract
  reserves, a separate explicit "acceleration required" policy; and
- the measurement boundary: a stated selected family is refused today, because
  no expansion compiles a payload, so nothing about *delivering* one is
  evidence yet.

Then update `docs/decisions/README.md` and the frontend contract's frontmatter
in the same change, and file the paired `accept-adr-NNNN-…` ticket so nothing
depends on a record that is merely written.

## Closes when

A proposed ADR exists with the content above, the decision catalog and the
frontend contract's metadata agree with it, and its acceptance ticket exists.
