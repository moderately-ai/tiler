---
id: scope-a-value-domain-precondition-for-squaring-overflow
title: Scope a value-domain precondition for a squaring overflow
status: deferred
priority: p2
dependencies: []
related: [admit-the-rms-normalization-family, admit-the-softmax-family]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, normalization, preconditions]
---
## User-visible outcome

A program can state, and a compiler can discharge or validate, a precondition that a normalized row's elements stay below the magnitude at which squaring overflows — so that the family's silent-wrongness case becomes a typed refusal for the callers who can prove or pay for it, without making every occurrence pay.

## Why this is deferred rather than done

`admit-the-rms-normalization-family` settled decision **D-3** of the [L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) as **define, not refuse**, and the elimination is what this ticket inherits.

**Fact — the wrongness is real.** `RMS_NORM_F32_SQUARING_OVERFLOW_BITS` (`0x5f7fffff`, ≈ 1.845 × 10¹⁹) names the largest binary32 whose square is still finite — the last finite-square magnitude, not the first overflowing value. At that constant the square is finite and a row normalizes nonzero; magnitudes **strictly above** it give a mean of squares of `+inf`, a reciprocal square root of `+0.0`, and a row of signed zeros. Finite, plausible, and wrong, with no NaN or infinity to reveal it. The conformance corpus already tests the threshold, its successor, and an above-threshold row.

**Correction — 2026-08-10.** An earlier wording equated *reaching* `0x5f7fffff` with overflow. That was one step too inclusive: `the_squaring_overflow_threshold_is_the_last_argument_whose_square_is_finite` asserts the constant's square is finite and the successor's is not, and the reference threshold/successor asserts show nonzero normalize at the constant and signed zeros only beyond it.

**Fact — none of the three refusal routes was available.** Construction sees shapes and attributes, never element values, so there is nothing to check there. A proved value domain would need an upper bound on `|x|` carried by the operand; `ExceptionalValueAssumption` and `ValueDomainProvenance::CompilerProven` name that class of evidence and no program input supplies it. A runtime scan is a *costed operation*: a second full pass over the 144,384·`T` contributors of one forward pass, whose answer must then be acted on through either a host readback per occurrence — 113 synchronization points per forward pass — or a device-side validation mechanism the bounded profile does not have.

**Inference — so the deferral is about a missing mechanism, not a missing decision.** Defining the behaviour is what the pinned formula already means, and Tiler reproducing the reference model exactly is the correct outcome for the workload as it stands. What is absent is any way for a *different* caller — one that can prove a bound, or one that would rather pay for a scan than receive zeros — to say so.

## Activation trigger

Any one of:

- a workload whose normalized inputs are not bounded below the threshold by construction, so the case stops being unreachable in practice;
- a caller that asks for the precondition explicitly, which needs a public way to state a value-domain assumption on a tensor;
- the arrival of a runtime-validation authority for a tensor-contents precondition, which would make the scan route costable rather than unimplementable.

## Closes when

1. The precondition is statable as a typed semantic precondition on the occurrence, with its provenance (`CompilerProven`, `RuntimeValidated`, or `CallerDeclaredUnvalidated`) explicit rather than inferred.
2. A validated route exists and its cost is measured rather than assumed — the extra pass, and whatever synchronization the answer needs.
3. An occurrence carrying the precondition refuses with a typed, explainable reason naming the threshold and the element that violated it; an occurrence not carrying it keeps today's defined behaviour unchanged.
4. The explain output distinguishes "precondition discharged", "precondition validated at runtime", and "no precondition stated", because the three are different claims about the same result.

## Trigger check log

- 2026-08-04 — **not fired.** None of the three: the pinned workload's normalized inputs stay bounded below `RMS_NORM_F32_SQUARING_OVERFLOW_BITS` by construction; no caller can state a value-domain assumption, because no public way to do so exists; and no runtime-validation authority for a tensor-contents precondition has arrived. `ValueDomainProvenance` and `ExceptionalValueAssumption` are consumed only by Metal emission's safe-math decision (`crates/tiler-metal/src/emit.rs:70-71,884-887`) and by nothing that carries a caller-supplied bound. Recheck: `grep -rn 'ValueDomainProvenance' crates/ --include='*.rs'`.
- 2026-08-09 — **not fired; the old consumer census is retired.** `ValueDomainProvenance` now also participates in compiler request-coherence checks, IR/artifact identity, and conformance tests, so it is no longer consumed only by Metal. That vocabulary still states provenance for an exceptional-value assumption, not a per-tensor magnitude bound such as `|x| < RMS_NORM_F32_SQUARING_OVERFLOW_BITS`; no caller asks for that precondition and no runtime contents-validation authority can discharge it. The three activation conditions therefore remain unmet for substantive reasons rather than because the provenance enum is unused.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `grep -rn 'ValueDomainProvenance' crates/ --include='*.rs'`, and run at this base it returns **120** lines. A result other than the 120 recorded here is the changed answer. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
