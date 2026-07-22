---
id: prototype-typed-explain-infrastructure
title: Implement typed optimizer explain infrastructure
status: in-progress
priority: p0
dependencies: [reconcile-implementation-work-graph-after-authority-audit, harden-compiler-verifier-subject-binding-and-totality]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, explain, authority]
claimed_from: todo
assignee: gpt-sol-explain
lease_expires_at: 1784743250
---
Implement one bounded typed explain authority shared by normalization, region,
feasibility, costing, selection, and refinement stages. Stable stage,
disposition, reason/rule/provider keys, subject references, evidence classes,
predicates, and exact budget stops are data; rendered strings are presentation.
Require deterministic ordering, bounded retention, causal errors, and stable
positive and negative conformance fixtures.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Review handoff

The implementation draft deliberately keeps `tiler_compiler::explain` private.
It adds a sealed, request-qualified `VerifiedExplainTrace` to each successful
target compilation product and migrates the current normalization, fusion,
feasibility, costing, selection, kernel, program, and artifact-plan flow to
typed records. Canonical identity uses an explicit schema encoding; the
versioned text renderer is presentation only. Retention is bounded across the
complete canonical trace, preserves the current portfolio's terminal selection
records, and emits a typed truncation record.

Tom still needs to review these public-boundary choices before this ticket can
be completed:

- compiler-owned public module versus a future `tiler-explain` crate;
- how successful and failed compilations return partial or complete reports;
- whether canonical traces are serialized or embedded in artifacts;
- which renderer guarantees, retention controls, and provider-detail/redaction
  policy form part of the public contract.
- whether public enums are non-exhaustive, versioned schema views, or both;
- which component may mint trusted evidence receipts for external providers;
- whether the public identity is canonical bytes, a specified digest, or both;
- how much of the request-qualified renderer header is stable versus redacted.

No provider emission trait is proposed yet. The draft keeps emission
compiler-owned and exposes only private read accessors, avoiding a premature
extension contract while preserving typed provider and rule identities.

The request-qualified trace boundary begins only after a
`VerifiedTargetRequest` exists. Malformed requests, request-budget failures,
unsupported semantic signatures, and semantic-output preflight failures retain
their existing typed errors and do not forge a target-qualified
`VerifiedExplainTrace`. A future public facade may carry separate unverified
attempt evidence, but that is a distinct authority and API decision.

The correction pass after immutable review also keeps terminal decisions in a
typed ledger: successful traces require one selected alternative and one
selection disposition per feasible or target-rejected considered alternative;
failed target compilations retain their original typed source plus one terminal
failure record. Configured limits govern optional detail only. Mandatory
decisions and the typed truncation summary are retained independently under the
compiler's hard aggregate bound, so detail retention cannot change planning.

The second correction pass makes omission and failure authority explicit.
Truncation is a non-causal sibling summary; a terminal whose direct detail was
omitted instead points to a mandatory typed bridge carrying that detail's exact
rule, subject, stage, and disposition. Pending terminal entries retain compact
keys rather than cloned compilation subjects and are transactionally bounded by
count and aggregate variable bytes. A maximum-ledger fixture demonstrates that
the hard trace record and byte formulas cover one bridge plus one terminal per
entry and the truncation summary.

Target compilation failures now carry an internal phase-local context beside
the original typed source. Numerical proof failures, scheduling and target
failures, kernel/program/artifact verification failures, and selection failures
are tagged at their failing call sites rather than reconstructed from the last
retained explain record. Feasibility records also enforce admitted
`required <= available`, rejected `required > available`, and budget-stop
`actual > limit`; equality is explicitly an admission rather than exhaustion.
The private conformance suite checks all current typed count emitters, all eight
target predicates and units, dominated versus Pareto-tradeoff selection,
zero/maximum retention, omitted-cause routing, and paired multi-alternative
target rejection context.

The final causal-integrity correction makes `TerminalCause` an opaque,
writer-minted token. Sibling compiler modules can wrap only an
`ExplainRecordId` returned by a writer; only `push_causal_detail` can mint the
omitted-detail form. That form now retains its validated, bounded predecessor
handles, and its mandatory bridge restores those exact edges instead of
severing the trace DAG at the retention boundary.

Target infeasibility is accumulated canonically for every considered
alternative under the same 16-cause bound enforced by terminal records. A
`NoFeasiblePlan` terminal depends on every materialized and fused rejection;
the existing typed source remains the deterministic first representative
because the error enum does not yet aggregate physical errors. Fixtures cover
two alternatives with distinct rejection predicates and assert that both are
direct terminal causes. Physical-error stage attribution is centralized and
used by both initial construction and portfolio rederivation: target errors map
to target feasibility, intrinsic and shape-product errors to intrinsic
scheduling, and refinement errors to kernel refinement.

Failure descriptors admit causes only through a bounded canonical cause set.
The set orders opaque tokens by their semantic identity and rejects duplicates
before the writer appends truncation, materializes an omitted bridge, or emits
the terminal failure. This matters for omitted tokens: cloning one token can no
longer manufacture two distinct bridge record IDs for the same logical cause.
Fixtures cover duplicate retained handles and duplicate cloned omitted tokens,
then reuse each unchanged writer successfully to prove admission failure is
transactional.
