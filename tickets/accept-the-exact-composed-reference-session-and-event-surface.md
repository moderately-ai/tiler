---
id: accept-the-exact-composed-reference-session-and-event-surface
title: Accept the exact composed-reference session and event surface
status: in-progress
priority: p1
dependencies: []
related: [implement-the-composed-realization-evaluation-driver, define-the-composed-realization-driver-subject-bridge, decide-the-safe-cross-crate-composed-reference-boundary]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, conformance, reference, numerics]
claimed_from: todo
assignee: worker-session
lease_expires_at: 1787427335
---
## User-visible outcome

Tom accepts or redirects the exact Rust surface by which the compiler's already-accepted composed-realization event stream drives the reference-owned, no-internal-tensor session. The implementation then has one unambiguous public spelling for every event descriptor, session transition, owned output, and typed refusal, without colliding with an existing reference type or inventing an unstated work-authority type.

## Dispatch hold

## Hold released — 2026-08-22, its trigger fired

The hold below required "an explicit coordinator observation that LiveRow is no longer active". **That observation is now made.** `.ticketsplease/decision-queue.md` item 6 records *Source-bound live-row-major access — ACCEPTED 2026-08-18*, `RESOLVED … option 1, the fieldless contextual marker`. The queue's companion gate is also clear: item 10's policy question was accepted 2026-08-18 and is recorded on `calibrate-the-physical-frontier-provider-and-outcome-budgets` under `## Accepted policy — 2026-08-18`.

**What this releases is a presentation, not a worker lane.** The next step is re-running the decision-packet readiness gate and moving to `awaiting-decision` for Tom — not implementation. Re-audit the packet's Facts first; it has `dependencies: []` and was written 2026-08-17, before several landings.

**Blocked, 2026-08-17.** Do not queue or present this packet while the current LiveRow decision is active. The trigger is an explicit coordinator observation that LiveRow is no longer active; only then may this move to `awaiting-decision`. This hold changes no technical conclusion below.

## Source-first Fact audit — exact published base `f6f310a47e1def152313120e0beda91ef8219fd8`

- **Verified — the safety architecture is decided, but the language-public reference surface is not spelled.** [`decide-the-safe-cross-crate-composed-reference-boundary`](decide-the-safe-cross-crate-composed-reference-boundary.md), anchors `safe composed-evaluation session` and `Decision — accepted 2026-08-12`, fixes ownership and authority: the session is plan-neutral, owns every internal tensor, accepts the exact retained program, declared inputs, typed value/fold descriptors, an explicit frozen registry and explicit work allowance, and refuses unsupported freedom. It names no Rust session type, constructor, transition method, output type, or error enum. Those choices are consequential language-public boundary, not private implementation detail.
- **False — `IterationStepAllowance` is not an existing type.** The illustrative wrapper in [`define-the-composed-realization-driver-subject-bridge`](define-the-composed-realization-driver-subject-bridge.md), anchor `allowance: IterationStepAllowance`, names a type absent from the repository. `ReferenceEvaluator` instead stores a `usize`; `ReferenceEvaluator::with_iteration_step_allowance(self, usize)` is the accepted public authority spelling, `ReferenceEvaluationRequest::iteration_step_allowance` returns `usize`, and every conformance caller supplies a `usize`. Reproduction: `rg -n 'IterationStepAllowance|with_iteration_step_allowance' crates tickets/define-the-composed-realization-driver-subject-bridge.md`.
- **False — the illustrated wrapper's `ReferenceOutputs` is not an owned whole-program result.** `pub struct ReferenceOutputs` in `crates/tiler-reference/src/registry.rs`, anchor `Host-owned bounded output writer for one reference callback`, is the mutable callback writer passed to `ReferenceOperation::evaluate`; its constructor and `finish` are crate-private. `ReferenceEvaluator::evaluate` returns `Result<Vec<Tensor>, EvaluationError>`. Reusing `ReferenceOutputs` for the wrapper would expose the wrong protocol and collide with the public item already used by every registered reference capability. Reproduction: `rg -n 'pub struct ReferenceOutputs|fn finish|pub fn evaluate' crates/tiler-reference/src/{registry,evaluate}.rs`.
- **Verified — the exact compiler event population is accepted but its public descriptor/accessor surface remains intentionally unspecified.** [`define-the-composed-realization-driver-subject-bridge`](define-the-composed-realization-driver-subject-bridge.md), anchors `Recommended exact boundary` and `Names remain implementation-level`, accepts `PlanAlternative::visit_composed_realization`, a closed `Begin` / `Stage` / `Materialization` / `Split` / `Complete` census, private fields, borrowed accessors, atomic pre-`Begin` validation, and distinct `Visitor(E)`. It does not decide whether variants carry public opaque row types or private-field structs, the exact accessor names and return types, how ordered semantic atoms and consumers are iterated, or the complete compiler-subject error vocabulary. An exhaustive sibling-crate match cannot be written until those spellings exist.
- **Verified — `serial_sum` is the only current `PlanAlternative`-backed expected-value path and the only `strict_partitioned_sum` caller.** In `crates/tiler-conformance/src/serial_sum.rs`, `partitioned_reference` receives a `ContributorPartition` reconstructed by `declared_partition` from a `PlanAlternative`'s ABI and applies it directly to caller operands. That module itself says this is sound only while the pointwise prologue is bit-identity. The new wrapper must replace that conditional provenance for the grouping-sensitive plan comparison. `crates/tiler-conformance/src/loop_carried.rs` is a second grouping oracle: `grouped_bits` calls `tiler_reference::cooperative_grouped_sum` for an explicit participant/round/contributor grouping, but its module header states `The compiler is not in the path`; it builds a `VerifiedScheduledRegion` directly and consumes no `PlanAlternative`. The other `PlanAlternative` consumers in `publication.rs` and `publication/proof.rs` package or classify proof evidence and do not compute a composed expected value. Reproduction: `rg -n 'strict_partitioned_sum|cooperative_grouped_sum|PlanAlternative' crates/tiler-conformance/src --glob '*.rs'`.
- **Verified — no downstream crate currently consumes `tiler-conformance`.** `cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.dependencies[].name == "tiler-conformance") | .name'` prints no package. `crates/tiler-conformance/src/lib.rs`, anchor `There is none`, keeps all modules under `#[cfg(test)]`. The plan-binding wrapper therefore remains `pub(crate)` and test-only; this ticket does not activate the deferred reusable public oracle.
- **Verified — no composed public names already exist to preserve.** `rg -n 'evaluate_composed|ComposedConformance|ComposedRealization' crates --glob '*.rs'` prints no match. This is freedom to choose deliberately, not evidence that any choice is compatible.
- **Verified — the retained candidate and assembly authority are ready.** `ProgramAlternative.semantic` is mandatory and owner-checked under `portfolio-retained-semantic-binding`; `CoverAssembly::from_plan` remains the single derivation of execution-ordered stages, bindings, dependencies, splits, publishing copies, and staged handoffs. This ticket changes neither prerequisite nor their identities.
- **Verified — no identity/schema step follows from the boundary.** The compiler event projection and reference session are on-demand, non-serialized, non-cached values. `ProgramAlternativeIdentity` already folds the retained semantic identity and selected-plan identity. No candidate here changes artifact, cache, request, schedule, KIR, semantic, registry, or canonical identity bytes.

## Decision packet

### Required exact deliverable

Record the complete accepted Rust surface, not pseudocode, including:

1. the `tiler-compiler::session` event enum, all constructor-free row/view types, every accessor and iterator item, and the non-erased compiler-subject/visitor error variants;
2. the `tiler-reference` safe session type, constructor ownership/lifetimes, exact `usize` work-authority parameter (or an explicitly accepted newtype), stage/materialization/split/finalization transitions, owned output type, and complete typed error vocabulary;
3. the `tiler-conformance` crate-private wrapper return/error spelling and its exhaustive event adaptation; and
4. compile-fail evidence that no public reference method accepts an internal `Tensor`, plus exhaustive-match evidence that a new compiler event stops the adapter.

The surface must state how the reference session validates the same `SemanticProgram`, exact arithmetic subject and subnormal modes, stage ordinal, semantic operation/value handles, fold axes, contributor order and partition; how a permitted reassociation is discharged exactly once; and how incomplete coverage, unsupported topology/freedom, registry disagreement, subject disagreement, callback failure, and compiler-subject failure remain distinct.

### Materially distinct candidates

1. **Opaque stateful session plus opaque compiler event rows — recommended.** Keep the accepted one-shot compiler method. Each event variant carries a constructor-free borrowed row with exact accessors. A public `#[doc(hidden)]` reference session is initialized from `&SemanticProgram`, `&FrozenReferenceRegistry`, `usize`, and declared `InputBinding`s; it accepts only scalar handles/descriptors, owns observations and pins internally, and consumes itself at `finish() -> Result<Vec<Tensor>, _>`. The conformance wrapper is the only plan-binding consumer and matches the event enum exhaustively.
2. **One public reference call over an owned descriptor recipe.** The compiler/conformance adapter first accumulates all rows and passes them to one reference function. This can be correct if the recipe is fully validated again, but makes caller-mintable parallel arrays the public reference boundary, duplicates the compiler visitor's completion protocol, retains the whole projection twice, and gives omission/order bugs a second representation. It is dominated by candidate 1 on maintenance and host memory without improving correctness or strictness.
3. **Expose the raw pin/observe primitive or reuse `ReferenceOutputs`.** Rejected. The first admits device-origin tensors into the expected value; the second is an operation-callback writer with the wrong ownership and completion contract. Rustdoc hiding changes neither defect.
4. **Strict/default evaluation or deferral.** Strict, unsubjected, standard-registry, baseline-program, and unsupported-topology fallbacks are rejected because they can silently answer a different numerical question. Deferral is safe and is the current implementation state, but does not deliver the named composed comparison; it remains the fallback if Tom declines every public spelling.

Candidate 1 is the sole nondominated implementation shape found: it is top-tier on correctness and fail-closed strictness, charges only on-demand reference work, introduces no persistent projection or identity, and has one completion protocol rather than two. The decision still belongs to Tom because its exact public types, methods, variants, and errors become callable language surface despite `#[doc(hidden)]`.

### Strongest counterpoint, reversal evidence, and negative controls

The strongest counterpoint is that a stateful session exposes more methods and invalid transition states than a single function. Reverse to candidate 2 only if an exact typestate or complete-recipe design demonstrates fewer public items while retaining atomic validation, exhaustive coverage, no caller tensors, no independently zipped arrays, and no second persistent copy. No such design is present at this base.

Before acceptance, perturb each property independently: omit and duplicate one event; reorder a materialization; pass a foreign same-shape `OperationId` and `ValueId`; mismatch the program owner; fail the callback after `Begin`; stop without `Complete`; alter the arithmetic subject; restore `ReferenceOutputs` as the wrapper output; and attempt to pass a `Tensor` to every public session transition. The expected refusal or compiler diagnostic must be quoted for each. A separate `P' != P` fixture must show that evaluating baseline `P`, the caller operands, or `kernels[0]` changes bits and is refused or disagrees.

## Non-goals

Implementing the surface; making `tiler-conformance` public; adding a plan type to `tiler-reference`; exposing raw pins; artifact-only replay; tolerance comparison; device-produced oracle inputs; a new identity/schema/domain/version; or deciding any LiveRow surface while that round is active.

## Closes when

After the dispatch hold clears, Tom accepts or redirects every exact public item and error/output collision above with provenance; the accepted packet leaves no placeholder type or implementation-level descriptor spelling for the implementation ticket to invent; and that ticket can re-audit the accepted names at its then-current base.
