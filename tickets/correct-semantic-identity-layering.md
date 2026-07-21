---
id: correct-semantic-identity-layering
title: Correct semantic identity layering
status: review
priority: p0
dependencies: []
related: []
scopes: [implementation/ir, implementation/compiler, project/tickets, contracts/foundation, contracts/navigation, contracts/decisions, contracts/artifacts, contracts/numerics, research/indexing, implementation/reference, research/artifacts, research/extensions]
shared_scopes: []
paths: []
tags: []
claimed_from: todo
assignee: codex
lease_expires_at: 1784655074
---
Separate graph meaning, reached semantic-definition requirements, and
provider-attributed admission provenance. Remove reached provider authority
from `SemanticProgram` identity, give each identity subject an unambiguous
public type, and update the existing target-neutral compiler proof to retain
the components independently.

Record the complete layered identity contract through an accepted ADR and
cross-document updates. The correction must preserve graph identity across a
provider-only revision while changing provider/admission provenance; semantic
keys, types, attributes, shapes, sharing, and ordered interfaces must continue
to affect graph identity. This ticket also specifies the later separation of
region content, region occurrence, pure index structure, checked refinement,
schedule/KIR content, complete plan coverage, and artifact/runtime provenance.

This is a P0 prerequisite for operation compilation capabilities and generic
region formation. It must not add placeholder region or schedule types before
their owning tickets implement the corresponding verifier authority.

## Outcome

- Replaced the overloaded semantic identity with explicit graph-meaning,
  reached-definition, admission-provenance, and full-registry-snapshot public
  identity types.
- Made graph identity provider-independent and retained the definition and
  admission projections independently through the bounded compilation request
  and artifact-construction plan.
- Added a regression proving a provider-only revision preserves graph meaning
  and reached definitions while changing admission and registry provenance.
- Accepted ADR 0072 and reconciled the IR, architecture, extension, artifact,
  correctness, roadmap, status, research, and downstream ticket contracts.
  Later region/refinement/schedule/KIR/program identities remain explicit work
  for their owning tickets; no placeholder types were added.
