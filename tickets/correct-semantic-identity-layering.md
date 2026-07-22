---
id: correct-semantic-identity-layering
title: Correct semantic identity layering
status: done
priority: p0
dependencies: []
related: [harden-semantic-registry-and-program-construction]
scopes: [implementation/ir, implementation/compiler, project/tickets, contracts/foundation, contracts/navigation, contracts/decisions, contracts/artifacts, contracts/numerics, research/indexing, implementation/reference, research/artifacts, research/extensions]
shared_scopes: []
paths: []
tags: []
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

The initial correction replaced the overloaded semantic identity with explicit
graph-meaning, reached-definition, admission-provenance, and full-registry-
snapshot public identity types, but adversarial review found that its reached
authority projection is not yet transitively complete. This ticket remains in
progress until all of the following hold:

- one deterministic, cycle-safe, resource-bounded semantic authority closure
  starts from every committed value type, operation key, and occurrence
  attribute value;
- the closure follows nested parameterized and encoded types, `Type` and
  `FloatBits` attribute values, type-definition facts, and operation schema
  defaults, facts, and conformance values;
- freezing validates every authority referenced by a registered type
  definition, including finite cyclic definition graphs without recursion or
  nontermination;
- `SemanticProgram` owns or exposes the complete reached definition and
  admission projections so compiler callers cannot silently omit roots;
- the compilation request retains graph meaning, reached definitions,
  admission provenance, and full registry snapshot provenance as distinct
  subjects; and
- tests cover nested constructors, encoded components, attribute-only type
  references, `FloatBits`-only references, missing definitions, finite cycles,
  resource exhaustion, and provider-only revisions.

ADR 0072 remains the accepted contract. Later region, refinement, schedule,
KIR, program, artifact, and runtime identities remain work for their owning
tickets; this correction must not add placeholder forms for them.

The correction now uses one iterative ordered-worklist closure with separately
governed, incrementally enforced root-ingestion and unique-subject bounds.
Their values are private implementation policy; the public non-exhaustive
resource class and typed `{ resource, limit, actual }` diagnostic remain stable
inspection surfaces.
Borrowed builder validation runs the same reachable authority projection as
commitment and preserves typed registry failures. Completed programs own one
non-forgeable `SemanticIdentity` with named borrowed accessors for the four
typed subjects; compiler requests, target requests, and artifact-construction
plans retain that bundle atomically. Registry caller-root projections are not public program-evidence
APIs. Tests cover every dependency class above at registry and completed-program
boundaries, both sides of finite cycles, the first item beyond each bound
without polling the root tail, used and unused provider revisions, compile-fail
construction of the public bundle, and rejection of a cross-program
semantic/request mix. Later identity layers remain deferred to
their owning tickets without placeholder types.
