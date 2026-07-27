---
schema: "tiler-doc/v1"
id: "tiler.research.documentation.open-ticket-audit-2026-07-27"
kind: "research"
title: "Open-ticket accuracy, scope, and outcome-language audit"
topics: ["tickets", "planning", "architecture", "documentation"]
catalog_group: "documentation-governance"
research_status: "complete"
disposition: "pending"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.portal.status"]
---

# Open-ticket accuracy, scope, and outcome-language audit

## Question

Does every nonterminal ticket describe still-open work accurately, declare the
areas its outcome can change, and lead with the user or system outcome rather
than prematurely fixing an implementation mechanism?

## Snapshot and population

The initial board query on 2026-07-27 returned 82 tickets whose status was
neither `done` nor `closed`: 59 `todo`, 9 `in-progress`, 2 `review`, 11
`deferred`, and 1 `blocked`.

One ticket, `carry-the-stage-execution-order-in-the-envelope`, completed on
`main` while the audit was running. The remaining 81 initial tickets were read
against their current construction sites, accepted ADRs, normative contracts,
dependencies, scopes, and closing conditions. Six remediation tickets were
created from gaps with no live owner. One further ticket,
`carry-the-data-flow-of-a-stage-dependency`, appeared concurrently and was also
reviewed. The final nonterminal population is 85.

The review used three non-overlapping parallel reading tracks and a central
synthesis pass. Agents returned claims and exact source locations; the
coordinator read the surfaced ticket and source before editing. No ticket was
classified from its title alone.

## Rubric

A ticket passes when:

- its facts reproduce at current construction sites;
- its status, dependency edges, and scopes describe executable upcoming work;
- it states the behavior, evidence, or decision the user needs before naming
  one implementation;
- a mechanism is prescribed only when that mechanism is the ticket's purpose
  or when alternatives fail correctness, authority, or maintainability;
- its close condition proves the outcome rather than a historical file shape;
- accepted decisions and completed prerequisites are not presented as open;
  and
- deferred work has a concrete activation trigger.

Implementation tickets may be technically specific. “User language” does not
mean deleting correctness invariants or type names that define the work; it
means the observable capability or failure boundary comes first and incidental
file shapes remain choices.

## Principal findings

### Physical enforcers had crossed the semantic boundary

`implement-boundary-property-enforcers` authorized dtype conversion as a
physical enforcer even though the accepted optimizer contract makes dtype a
semantic property. The ticket now permits only value-preserving changes to
storage, layout, encoding, placement, synchronization, and delivery.

### Cache reporting denied an already completed publication

The atomic rename is the cache publication point, but current outcome language
can report a later directory-sync or cleanup failure as `Uncached` with a
publication refusal. That is false once another process can observe the entry.

`report-cache-publication-state-after-the-rename-boundary` now owns the
correction. The cache public-boundary review and deterministic I/O-failure
ticket depend on it and distinguish pre-rename absence from post-rename
publication with weakened durability or cleanup.

### Runtime preflight was repeatable authority, not one-shot commitment

Several tickets assumed that consuming a non-clone `Preflight` makes fallback
authority unrecoverable. `DecodedProgram::preflight` is callable through a
shared reference and the decoded program is clonable, so several preflights can
be minted.

`make-runtime-routing-commit-authority-one-shot` now owns the route-level
authority. Multi-stage preflight, the Metal runtime proof, and the inline AOT
integration proof depend on it.

### Metal provenance could describe a tool that did not produce the bytes

Metal AOT preflight records resolved absolute tools, while compilation selects
bare tools through `xcrun` again. Promotion of the compilation identity is now
blocked on `bind-recorded-metal-toolchain-to-the-tools-that-execute`.

### ShapeEnv's proposed resolution depended on values the model does not carry

`widen-shapeenv-factorization-fragment` treated availability phase as though it
made a caller parameter known. Non-static bindings name a future source and
carry no supplied value. The ticket is now one atomic product decision:
continue refusing fully launch-dynamic factorizations, or admit explicit
specialization input/undecided-state machinery. It recommends refusal until a
real single-artifact dynamic requirement exists.

### Public reviews mixed several independent decisions

The delivered-realization review now asks one boundary question. Cache
maintenance was split from key-oriented lookup/publication into
`accept-the-expansion-cache-maintenance-boundary`. The distributivity direction
question is now dependent on first admitting distributivity at all.

### Several tickets described completed or obsolete work

- `expose-the-built-artifact-canonical-identity` is `done`; the approved
  canonical accessor exists and is exercised.
- `revisit-kernel-lowering-placement` is `done`; the landed Metal and compiler
  paths provide no evidence against the approved placement.
- `probe-the-expansion-cache-filesystem-properties-on-linux` is `closed`;
  Linux is outside the current support policy.
- `reconcile-cache-filesystem-claims-with-macos-support-policy` owns removal of
  stale “supported Linux” product rows while preserving them as future
  research.
- `add-subgroup-memory-scope-when-collectives-land` is `deferred` until its
  stated collective trigger fires.

### Historical implementation detail obscured live work

The compiler public-boundary ticket was mostly a record of settled explain
questions and an already approved facade. It now states the remaining outcome:
external construction of a consumer-independent request and provider
installation, while preserving `compile_governed` as the bounded convenience
path.

Large historical appendices were removed or replaced where they instructed a
future worker from obsolete bases, schema versions, call-site counts, deleted
Python gates, or superseded decisions.

## Scope and dependency corrections

Every final nonterminal ticket has an exclusive or intentionally shared mapped
scope. The only ticket with only `project/tickets` is
`supersede-the-multiply-by-one-subnormal-claim`, whose entire outcome is an
append-only correction to a historical completed ticket.

Notable corrections include:

- layered identity, numerical-profile ownership, and digest-selection tickets
  now declare the code, contract, research, and workspace areas they name;
- quantized backend, durability, multi-device, identity-namespace, recorded
  identity, and standard-scalar tickets now include their missing consumers;
- semi-affine expressions depend on the public-enum hardening that makes the
  addition compatible;
- proc-macro/Cargo measurement depends on the actual admitted frontend and
  production cache prerequisites;
- multi-stage runtime work depends structurally on execution-order and
  one-shot-authority work; and
- public cache acceptance depends on truthful publication semantics.

## Board-state corrections

Five stale branchless claims initially made the board disagree with Git.
Branchless implementation/research work returned to `todo`; an owner-reserved
public API review moved to `awaiting-decision`. The unsafe-site gate ticket also
moved to `awaiting-decision` because the implementation it described was
deleted and the remaining question is whether mechanical inventory should be
restored.

Existing worktrees and branches remain represented as `in-progress`; expired
lease metadata was removed from the ticket text without deleting or mutating
those worktrees.

## Tickets corrected in place

The audit corrected 57 tickets from the initial population. Changes fall into
the categories above: factual premise, status, dependency, scope, atomicity,
outcome language, or removal of stale implementation prescription.

The affected tickets are the tracked ticket changes in the audit diff,
excluding the six newly created remediation tickets and the concurrently
created `carry-the-data-flow-of-a-stage-dependency`. Git is the authoritative
per-ticket patch; this record explains the classification rather than
duplicating every rewritten paragraph.

## Tickets reviewed without a substantive text correction

The following initial tickets remained accurate and sufficiently
outcome-oriented:

- `bound-the-target-profile-descriptor-by-its-declaring-authority`
- `calibrate-device-cost-models`
- `carry-the-honourability-fact-provenance-into-the-artifact-record`
- `decide-the-expansion-cache-collection-schedule`
- `external-storage-resource-scope-gate`
- `harden-kernel-vocabulary-recognizer-completeness`
- `implement-analytical-component-cost-model`
- `implement-first-algebraic-rewrite-portfolio`
- `implement-general-dag-partitioning`
- `implement-index-domain-predicates`
- `implement-opaque-physical-call-providers`
- `implement-parallel-reduction-strategies`
- `implement-transactional-rewrite-engine`
- `prototype-candle-metal-adapter`
- `prototype-inline-proc-macro-frontend`
- `prototype-quantized-value-vertical`
- `report-per-target-compilation-outcomes`
- `retire-the-metal-first-use-buffer-binding-workaround`
- `revisit-kernel-body-single-spelling-gate`
- `spike-cuda-multi-device-transfers`
- `spike-hermetic-fptaylor-certificate-checking`
- `spike-metal-multi-device-transfers`
- `supersede-the-multiply-by-one-subnormal-claim`
- `test-two-revisions-of-one-provider-as-a-capability-ambiguity`

`carry-the-data-flow-of-a-stage-dependency`, created concurrently during the
audit, also passes: it leads with the silent-wrong-result risk, derives the
slot-pair mechanism from existing verified data, and does not require a new
public value identity.

## New remediation tickets

- `report-cache-publication-state-after-the-rename-boundary`
- `bind-recorded-metal-toolchain-to-the-tools-that-execute`
- `make-runtime-routing-commit-authority-one-shot`
- `accept-the-expansion-cache-maintenance-boundary`
- `reconcile-cache-filesystem-claims-with-macos-support-policy`
- `decide-whether-distributivity-directions-share-one-permission`

Each was reviewed under the same rubric before inclusion in the final board.

## Verification

At the end of the audit:

- `tkt lint --format json` reported no diagnostics;
- `tkt reconcile --format json` reported no board/Git findings;
- `git diff --check` reported no whitespace errors; and
- all Markdown links in the initial nonterminal ticket population resolved to
  existing local targets.

No production code was changed. Cargo execution was not required to validate
ticket prose. One parallel reader accidentally invoked the repository's default
`make` target while constructing a shell search; it completed without changing
tracked files, and the audit did not treat that run as verification evidence.

## Disposition

The board is now a materially more reliable statement of upcoming work, but the
ticket edits are planning corrections rather than implementation evidence.
Public-boundary tickets tagged `needs-tom` remain proposals or
`awaiting-decision` until Tom accepts the exact boundary. Newly surfaced
correctness prerequisites should be completed before their dependent public
surfaces or end-to-end proofs are treated as stable.
