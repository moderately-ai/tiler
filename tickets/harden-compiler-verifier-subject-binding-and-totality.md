---
id: harden-compiler-verifier-subject-binding-and-totality
title: Harden compiler verifier subject binding and totality
status: done
priority: p0
dependencies: [prototype-canonical-index-region-slice]
related: [prototype-target-neutral-fusion-slice]
scopes: [implementation/compiler, implementation/ir, implementation/artifact, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, optimizer]
---

Turn the current target-neutral proof slice into a verifier boundary whose
evidence is bound to exact subjects and whose public entrypoints are total over
malformed input. The fixed-point audit at
`ad6e9f463de6eabad44af47eaddad9317e0935fd` found multiple mutually consistent
forgeries that pass because copied fields are compared only to one another.

## Required outcome

- Bind numerical-fusion evidence to the exact candidate identity, ordered
  members, boundaries, request, numerical contract, and materialized reference
  capability. Candidate kind alone is not evidence.
- Recompute and verify candidate stable identity, require kind/membership
  consistency, unique region occurrences and witness identifiers, and exact
  request-to-semantic-graph and schedule-to-request occurrence binding.
- Treat the selected target profile as checked authority. Retain and verify its
  exact facts/provenance rather than accepting a forgeable key plus editable
  resource fields.
- Canonically validate schedule axes before using them: reject duplicates and
  out-of-range axes, zero threads per workgroup, invalid contributor products,
  late-zero cardinality mistakes, and scalar numerical realization conflicts.
- Bind structured KIR and whole-program refinement to exact content, including
  constants, bodies, ABI, routing, buffer ownership, capacities, value-to-
  allocation references, launch expressions, resources, and numerical policy.
  Vector-position agreement is not ownership proof.
- Validate lowering-provider outputs for nonempty exact coverage, uniqueness,
  provenance, and subject identity. Distinguish malformed compiler output from
  a valid empty physical frontier.
- Preserve `PhysicalError::Intrinsic` as malformed/intrinsic compiler output;
  do not convert it into `NoFeasiblePlan`, which is reserved for a valid empty
  frontier.
- Make every public compiler entrypoint return a typed error for zero/one-item
  slices, malformed axes, bad contributor indices, stale portfolio selections,
  dead/noncanonical host expressions, and budget excess; no indexing panic or
  post-construction budget escape is permitted.
- Preserve structured explain evidence—subjects, predicates, actual/limit
  budgets, feasibility facts, and evidence class—rather than reducing it to
  strings.
- Derive portfolio selection from verified candidate identity and a
  recomputed, provenanced cost result. A raw vector index, stale selection, or
  caller-editable/manual cost cannot be selection authority.
- Stop describing the hand-enumerated five-singleton/pointwise/full candidate
  set as complete. Keep it as a private bounded recognizer until general region
  enumeration owns completeness.
- Replace self-referential host-arithmetic differential tests with an
  independent reference or explicitly narrow the evidence claim.

This correction hardens the verifier products already present in the bounded
proof and establishes mandatory refinement seams for later stages. It must not
invent the mature KIR, program, ABI, or artifact structures owned by
`prototype-structured-kir-slice` and
`prototype-kernel-program-ir` and `prototype-artifact-program-model`; those
tickets consume and
extend these subject-binding invariants when their fields exist.

## Acceptance

Add field-by-field mutation tests that alter exactly one candidate, request,
schedule, target, KIR, buffer, portfolio, provider, proof, or artifact field and
require rejection with the correct typed class. Add panic-free malformed-input
tests and late-zero/duplicate-axis cases. The compiler must never accept a plan
merely because two forged copies agree.

## Outcome

Completed in `1ae5fcc`: verified requests, schedules, structured kernels,
programs, and artifact plans are opaque receipts bound to authoritative request
subjects; exact reduction shapes, target facts, KIR, ABI/host-expression IDs,
outputs, providers, and portfolio evidence are rederived or checked. Mutation
and malformed-input coverage now includes target, provider, budget, semantic
identity, key, normalized constant, schedule/proof, kernel, program, and
artifact forgeries. Independent immutable reviews of `1ae5fcc` passed, as did
the full Rust gate, documentation validation, Ruff, 143 pytest tests, ticket
lint, strict scope guard, and diff checks.
