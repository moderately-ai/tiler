---
id: reconcile-implementation-work-graph-after-authority-audit
title: Reconcile the implementation work graph after the authority audit
status: in-progress
priority: p0
dependencies: [prototype-canonical-index-region-slice]
related: []
scopes: [contracts/navigation, contracts/optimizer, implementation/workspace, contracts/foundation, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [maintenance, architecture, implementation]
claimed_from: todo
assignee: gpt-sol-reconcile
lease_expires_at: 1784735955
---

Apply the adversarial ticket-DAG, implementation-coverage, and ticket-quality
audit performed at `01a223b7670b2251c5c6e1b3c0e18e2db7891716` after the
semantic-authority and canonical-index corrections are accepted. This is a
work-graph correction, not authority to implement the compiler components.

An eleven-wave fixed-point audit of the actual codebase at
`ad6e9f463de6eabad44af47eaddad9317e0935fd` found additional correctness and
evidence gaps. This ticket must integrate the resulting vertical corrective
work rather than treating the earlier ticket-only audit as exhaustive:

- `harden-semantic-registry-and-program-construction`;
- `correct-reference-value-and-authority-contracts`;
- `harden-compiler-verifier-subject-binding-and-totality`;
- `enforce-repository-validation-gate-integrity`;
- `repair-numerical-witness-integrity`;
- `repair-cache-experiment-harness-integrity`;
- `repair-apple-target-experiment-integrity`;
- `repair-macro-and-embedding-harness-integrity`;
- `repair-shape-and-runtime-experiment-integrity`; and
- `reconcile-research-evidence-provenance`.

The two already-open owners, `correct-semantic-identity-layering` and
`prototype-canonical-index-region-slice`, retain their transitive-authority and
index-canonicality findings respectively. Do not duplicate those obligations
into a competing ticket.

## Required outcome

### Correct the P0 dependency and ownership graph

- Record only non-transitive dependencies after the accepted chain
  `shared compiler IR -> semantic identity -> canonical index`.
- Introduce one typed explain infrastructure owner. Stable stages,
  dispositions, reason/rule/provider keys, subject references, evidence
  classes, predicates, and budget stops are typed authority; rendered strings
  are presentation only.
- Introduce a bounded semantic-normalization stage before generic region
  formation. The P0 slice may admit only identity or a deliberately tiny
  proved rule set, but must establish deterministic traversal, termination,
  budgets, semantic revalidation, transactional failure, and explain records.
  Track the broader external transactional rewrite engine and first algebraic
  rule portfolio separately rather than hiding them in region formation.
- Split operation capability registration/resolution from checked semantic-to-
  index refinement. Refinement binds exact occurrences, values, accesses,
  numerical/effect evidence, scalar authority, and provider provenance after
  both capability and region authorities exist.
- Add a generic slow `IndexRegion` oracle in `tiler-reference`, including
  registered scalar evaluators and N-state reductions, before fusion legality
  is accepted.
- Split target-profile feasibility, checked scheduled-region IR, and physical
  implementation-frontier enumeration into separate verifier authorities.
  Keep hard feasibility distinct from cost and malformed proposals distinct
  from a valid empty frontier.
- Correct the planning inversion: enumerate legal region covers and local
  physical frontiers before choosing a compatible complete program. Structured
  KIR refinement follows selection; it does not precede the schedule or select
  the complete cover.
- Split verified target-neutral kernel-program IR from the artifact-facing
  manifest/model. The codec consumes the artifact model.
- Add the reviewed general public compiler boundary required by ADR 0069 and
  make the inline proc-macro frontend consume it. Backend feasibility work need
  not wait for that ergonomic facade unless its actual dependency requires it.
- Strengthen the optimizer conformance gate with an external operation through
  the ordinary path, non-isomorphic and fan-out or multi-output graphs, stable
  explain, and identity/provenance assertions for every implemented layer.
- Gate operation capabilities and fusion legality on the semantic/reference
  corrections, and gate every schedule/KIR/program/artifact milestone on the
  compiler verifier subject-binding correction. A mutually consistent forged
  plan is not a verified plan.
- Treat the repository-gate correction as implementation infrastructure, not
  optional cleanup: no later conformance claim may rely on a gate that can
  disable tests, lints, documentation, toolchain selection, or locked Python
  resolution while remaining green.

### Correct later work ownership

- Split physical boundary-property modeling from executable enforcers,
  transfers, synchronization, and resource-lifetime verification.
- Split ShapeEnv core ownership from index binding and predicate support.
- Rename the presently uncalibrated cost-model ticket to describe an
  analytical component model; track device calibration behind explicit
  measurements and an activation condition.
- Make general DAG partitioning depend on the cost authority it uses for
  shared-work duplication. Make reduction strategy *selection* depend on that
  authority if the ticket selects rather than only enumerates.
- Narrow the quantized vertical to a target-neutral compound-value,
  reference/index, and ABI proof. Track the first backend execution profile
  separately when its format and target are selected.
- Make Metal numerical realization consume the Metal KIR-to-MSL lowering
  surface. Keep the Metal AOT, optimizer, runtime, and inline integration
  tickets as integration gates rather than duplicating component ownership.

### Repair scheduling metadata and corpus status

- Enable ticketsplease's Rust backend and map every existing implementation
  scope to its owning workspace crate so reverse-dependency collisions are
  visible.
- Give every nonterminal ticket additive `project/tickets` coverage using a
  shared scope; do not make all implementation tickets mutually exclusive
  merely because each updates its own file.
- Add a coordination scope for `Cargo.lock`, include `.gitignore` in the
  workspace scope, and replace path-only allowances where this improves
  collision detection.
- Keep historical empty scopes only with an explicit coordination-only
  explanation or migrate their references to real contract scopes.
- Reconcile `docs/status.md`, operation-extension status, correctness/testing
  status, roadmap language, and terse ticket acceptance criteria with the
  resulting graph.
- Remove the stale local `tkt/prototype-operation-compilation-capabilities`
  branch only after re-verifying that it has no unique commits and is already
  contained in `origin/main`.

## Non-goals

- Do not delete completed or closed historical tickets. Preserve superseded
  work as provenance.
- Do not bulk-delete old worktrees or branches; three historical branches have
  non-obvious patch relationships and require individual inspection.
- Do not make the P0 proof implement the mature rewrite portfolio, calibrated
  device models, general DAG partitioning, quantized backend execution, or
  opaque physical calls.

## Validation

Run `tkt lint`, `tkt reconcile`, `tkt ready`, `tkt tracks`, critical-path
queries for the optimizer and inline milestones, the full documentation gate,
and `git diff --check`. The resulting immediate frontier must expose only work
whose prerequisites and public verifier authorities actually exist.

## Reconciliation evidence boundary

The implementation audit was performed from integrated base
`92b9b37b92fb3c2e6c13fde48c9ed499edb6ced6`. All registered worktrees were
inspected individually and were clean. No required content was found stranded
on the three non-patch-equivalent historical heads; completed worktree and
branch cleanup remains a separate coordinated maintenance action. The named
stale `tkt/prototype-operation-compilation-capabilities` branch was already
absent locally and remotely and had no unique commit, so this ticket performs
no destructive branch or worktree operation. Local `main` was ahead of
`origin/main`; new dispatch must use the exact integrated base until remote
coordination catches up rather than pretending stale remote state is current.
