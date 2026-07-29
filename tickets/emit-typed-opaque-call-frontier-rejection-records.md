---
id: emit-typed-opaque-call-frontier-rejection-records
title: Emit typed opaque-call frontier rejection records
status: done
priority: p2
dependencies: []
related: [emit-analytical-costs-through-the-typed-cost-vocabulary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, explain, opaque-calls]
---
## User-visible outcome

A reader of the explain trace can distinguish an unregistered opaque call, a malformed named binding, a numerical-contract mismatch, an underivable boundary contract, unresolvable work scaling, and target infeasibility. Each refusal retains the opaque-call identity, provider attribution, and its typed or stable reason rather than collapsing into `frontier.rejected-count`.

## Why this is separate from analytical cost reporting

These records are refusals, not costs. `ExplainEvent::CostAssessment` has no rejection subject or reason and widening it would make a calibration record assert a frontier disposition it never computes.

The existing rejection events also cannot represent the complete set truthfully. `UnregisteredCall` and `MalformedBinding` can be intrinsic checked refusals, and `numerical-contract-mismatch` can be a numerical-legality refusal, but `CallNotAdmissible("target-infeasible")` retains only a reason string. `ExplainEvent::Feasibility` requires typed `required` and `available` quantities and validates their relation, so constructing it from the current frontier would require inventing data. Labelling the same refusal intrinsic would put a target verdict at the wrong stage. The construction site is `enumerate_frontier` in `crates/tiler-compiler/src/frontier.rs`: `assess_resources` returns `ResourceVerdict`, whose capability rejection, numerical rejection, intrinsic error, and unresolved evidence are four different outcomes, but the closure currently reduces all four to `"target-infeasible"` before storing the rejection.

## Implementation keys

- Replace the three partial opaque-call rejection shapes and the opaque use of generic `NotApplicable` with one exhaustive opaque-call rejection carrying the complete `OpaqueCallProposal` and a typed cause. The proposal, including ordered named bindings, is part of the refusal because two calls with the same local name but different owner, revision, or role binding are not the same subject.
- The cause vocabulary covers target-profile non-applicability, unregistered identity, each `BindingError`, each boundary `GuaranteeError`, numerical-contract mismatch, typed work-resolution failure, capability infeasibility with its `ResolvedPredicate`, and numerical unhonourability with its `UnhonouredDimension`.
- Classify `ResourceVerdict` without a catch-all: `Rejected(Capability)` and `Rejected(Numerical)` are ordinary typed frontier refusals; `Intrinsic` and `Unresolved` are compiler errors and fail frontier enumeration closed.
- Replace the work-scaling `Option` with a typed result that distinguishes an unknown declaration parameter from an unavailable intermediate shape.
- Encode opaque-call rejection order from the full typed proposal and cause. Do not order by rendered diagnostics, omit bindings, hash or truncate identity components, or reconstruct quantities from strings.
- Govern opaque-call identity components tightly enough that the exact owner/name/revision can become a collision-free bounded explain subject; delimiter ambiguity or an identity that is valid but unreportable is not acceptable.
- Govern ABI parameter names, complete ordered proposal subjects, and physical-provider explain identities at construction with typed lexical and bound errors. A valid ABI, proposal, or provider must not become unreportable only after frontier work has begun.
- Emit one typed explain record per opaque-call rejection from `record_frontier`, with the exact call and provider as subjects/provenance.
- Attribute the admission decision to a compiler-owned rule. The provider is the proposer and a subject of the verdict, not the authority that decided its own proposal was valid.
- Represent a disproved applicability predicate as not applicable. It is not an intrinsic defect, numerical illegality, target infeasibility, or unknown capability; if the explain vocabulary lacks that disposition, widen the vocabulary and its schema rather than misclassifying the event.
- Keep frontier counts only if they remain useful as a summary; they cannot substitute for the records.
- Do not route any rejection through `CostAssessment`, and do not let explain reporting alter frontier admission, dominance, or retained plans.
- Exercise every binding fault and every opaque-call admission refusal path, including both target verdict classes and a capability-infeasible fixture whose exact required/available quantities and relation are validated.
- Carry the arithmetic dtype through numerical honourability events, rendering, and canonical identity. The same dimension can be honourable for one dtype and unhonourable for another, so omitting it aliases different target claims.

## Closes when

- Every opaque-call `FrontierRejection` variant reaches explain with a typed stable reason and exact subject/provenance.
- The target-infeasible record carries the quantities produced by the feasibility authority.
- Rejection identity and canonical order change when owner, call, revision, named binding, role, cause, or typed cause payload changes; registration order does not change them.
- An intrinsic or unresolved resource assessment cannot appear as an ordinary target rejection.
- A perturbation that drops or misclassifies each refusal makes its check fail.

## Graph maintenance

- Update the frontier explain census once, naming whether per-rejection records replace or accompany the summary count.
- Remove the split-remainder note from `emit-analytical-costs-through-the-typed-cost-vocabulary` when this ticket closes.

## Implemented outcome

Opaque-call refusals now use one exhaustive typed `FrontierRejection::OpaqueCall` carrying the complete proposal and one of eight typed causes. The frontier preserves ordered named bindings, every binding and boundary fault, typed work-resolution failure, exact target capability quantities, and dtype-qualified numerical unhonourability. Capability and numerical target refusals remain ordinary frontier outcomes; intrinsic and unresolved resource assessments fail enumeration closed as typed compiler errors.

Explain schema 5 and renderer 4 add append-only opaque-call/provider subjects and a truthful not-applicable disposition. Each opaque refusal emits one compiler-attributed detail record alongside the unchanged admitted/rejected summaries; unregistered identities and impossible work parameters are checked intrinsic failures, unavailable intermediate shapes remain deferred, and no refusal is represented as cost. Numerical honourability now includes `ArithmeticType` in its event, rendering, and canonical encoding across both opaque and scheduled-region paths.

Exact reporting is enforced before work begins. Opaque call and ABI parameter components use a delimiter-safe governed grammar; complete ordered proposal subjects and physical-provider subjects must fit the explain bound; oversized or ambiguous values receive typed construction/provenance errors rather than hashing, truncation, or a late reporting failure. Provider provenance carries the validated exact subject through admission and rejection instead of dropping back to an unchecked string.

The focused compiler suite passes 414 tests with one configured skip, package Clippy passes with warnings denied, doc-tests pass, and formatting, diff, and ticket checks are clean. Fault perturbations independently proved rejection identity, oversized-provider preflight, exact proposal/binding subjects, binding/work payloads, target quantities, dtype-qualified honourability, and per-rejection emission can make their checks fail.
