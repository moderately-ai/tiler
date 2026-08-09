---
id: name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set
title: Decide the stable diagnostic key for a materialized reduction prologue
status: awaiting-decision
priority: p3
dependencies: []
related: [admit-a-recognized-chain-more-than-one-materialization-boundary-deep]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

A caller receives a stable rule that truthfully classifies the materialized producer a reduction prologue cannot retain, without pretending the compiler has distinguished a producer kind it discarded.

## Per-Fact audit — 2026-08-09

- **Verified mechanism.** `recognize_reduction` is the production caller of `recognize_elementwise`; `ElementwiseRefusal::Folded(ValueId)` reaches `impl From<ElementwiseRefusal> for RequestError`, whose source-safe anchor is `Flattens a discovered materialization boundary into the rule a caller`. That arm reports `operation-set` because `NormalizedSerialSum` has no producer field.
- **False proposed classification.** `Folded` carries only a `ValueId`, not the producing family or whether the producer is a chain. `materializes_its_result` covers reductions, contractions, and every staged family realization. Renaming the single arm to `reduction-prologue-chain` would therefore classify direct materialized producers and staged families as chains without evidence.
- **Impossible required control.** `sum(rms_norm(x))` reaches the same `Folded` arm as `sum(sum(x) * 2.0)`. A one-arm rename necessarily changes both. The original requirement that the first remain `operation-set` while the second receives the new key cannot pass without first widening the internal refusal to retain producer classification.
- **Public-boundary consequence.** `session::CompileFailureClass::UnsupportedCapability { rule }` documents `rule` as the stable diagnostic key. This is an intentional caller-visible observed-value change, not a comment-only rename.
- **Imprecise documentation finding.** The existing “same general walk” statement is correct about the shared planner. What differs is how the two callers consume `Folded`: output recognition builds an epilogue, while reduction recognition flattens it. Amend with that distinction rather than deleting the shared-walk claim.

The original implementation prescription is withdrawn. It would make the diagnostic more specific in spelling and less accurate in meaning.

## Decision boundary

Tom chooses the stable public classification:

1. **One key for the actual retained fact.** Rename every flattened `Folded` result to a key such as `materialized-reduction-prologue`, covering any producer whose value would have to cross into `NormalizedSerialSum`.
2. **Producer-specific keys.** Extend `ElementwiseRefusal::Folded` or the reduction recognizer to retain a governed producer class, then decide separate keys for reduction, contraction, and staged-family producers.
3. **Keep `operation-set`.** Preserve the broad key and correct only the explanatory documentation.

**Recommendation: option 1.** It names exactly what the recognizer knows and what the normalized form lacks, without widening the recognizer merely to improve a diagnostic. **Strongest counterpoint:** callers may care whether the missing admission is a nested fold, a contraction result, or a staged family; one broad key gives them no more remediation detail than `operation-set` unless the explain path carries the producer separately.

The optimizer contract's stable-reason-code promise is now in scope so the accepted classification can be stated where callers are told what a key means.

## Required evidence after the decision

- Keep `sum(sum(x) * 2.0)`, a direct materialized contraction prologue, and `sum(rms_norm(x))` as separate subjects. State the intentional key for each; do not require an impossible distinction from one unextended arm.
- Keep the accepted neighbour — the same fold over the same scaling of a declared input — compiling end to end.
- Keep a genuinely unspellable elementwise family reporting `operation-set` so the new classification does not absorb the vocabulary wall.
- Perturb the production subject, not the expected string: remove or bypass the materialized producer and show the classified refusal changes or disappears.
- File admission of a materialized producer into `NormalizedSerialSum` separately; the existing staged-operand-depth deferral owns a different boundary.

## Explicit non-goals

No admission change, request identity change, schedule change, or silent reuse of the deeper-chain ticket. A producer-kind split is implementation work only if Tom selects option 2.

## Closes when

Tom selects the public classification; the source docs, optimizer contract, and complete affected test population agree with it; the accepted neighbour and genuinely unspellable control remain distinct; and the separate admission owner is filed.
