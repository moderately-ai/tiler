---
id: record-adr-0076-honourability-implementation
title: Record ADR 0076 items 2, 3, and 5 as landed and answer two of its open questions with implementation evidence
status: done
priority: p2
dependencies: []
related: [compose-numerical-honourability-and-retire-the-strict-boolean, expose-the-numerical-contract-preference-list]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, adr]
---
`compose-numerical-honourability-and-retire-the-strict-boolean` landed ADR 0076's item 2 (less its public spelling), item 3, and item 5 in `tiler-compiler`. ADR 0076 is in the `contracts/decisions` scope, which that ticket did not hold, so the record still describes the tree before the change and two of its open questions now have evidence they did not have.

**What landed, so the record can stop saying it is unstarted.** `crates/tiler-compiler/src/honourability.rs` is the per-dimension authority, a peer of `feasibility::CheckedTargetProfile` rather than new `CapabilityAxis` variants: `NumericalDimension`, `DimensionBehaviour`, the four `HonouringMeans`, and a `NumericalHonourabilityFact` carrying the same phase/authority/validity/provenance a `CapabilityFact` does. `CheckedTargetProfile` holds the declaration and encodes it into `canonical_descriptor`, whose domain moved to `tiler.target-profile.descriptor.v3`; the feasibility rule set key moved to `tiler.feasibility.phased-capability-and-numerical-honourability.v1` because the vocabulary widened. `CheckedTargetProfile::assess` composes both predicate kinds into one ADR 0043 outcome under the stated mapping, with the unenumerated-dimension `Unknown` path tested three ways. `PrototypeTargetProfile::supports_strict_f32`, `CapabilityAxis::StrictF32Arithmetic`, and `physical::requires_strict_f32` are gone; axis tag `0x06` is retired rather than reassigned. `explain` gained `ExplainEvent::NumericalHonourability`, which carries the dimension, the required behaviour, the declared means, the honoured alternative, and the declaring profile.

**Open question 2 — list versus retry — has evidence, and it is not a settlement.** The list costs one `Vec` on the request, one resolution loop in `verify_request`, and one length-framed run in the request-subject encoding. What it buys is that the caller's *stated fallback* enters request identity: two requests that resolve to the same contract but declare different alternatives are different requests, which a retry loop cannot achieve because the compiler never sees the alternatives. The record says the alternative was not rejected on evidence; this is evidence for the list and against the retry, and it should be recorded as such rather than left implicit. `expose-the-numerical-contract-preference-list` owns the public spelling, which is where the choice becomes hard to reverse.

**Open question 3 — whether `SupportedOnlyUnderDeclaredRelaxation` earns its place — has evidence too.** It is implemented and independently tested. Its cost is one struct (`RelaxationRequirement`), one extra descriptor field in the conditional arm, and one lookup of the proposal's own requirement set. Its benefit is that a rejection can name *why* a dimension is unhonourable for this request specifically — that the target would honour it had the caller authorized a named relaxation on another dimension — which `Unsupported` cannot say. The record's own analysis is correct that it behaves like `Unsupported` when unauthorized and like `SupportedExactly` when authorized; the finding is that the *explanation* differs even where the verdict does not, and that is what the outcome buys.

**Deliberately not claimed here.** No governed profile constructs `SupportedWithExactEmulation`, `SupportedOnlyUnderDeclaredRelaxation`, or `Unsupported`; the target-neutral baseline declares only `SupportedExactly`, and `declare-metal-numerical-honourability` is the first profile that will declare otherwise. So these are implemented and tested vocabulary, not measured target behaviour, and the record must keep those apart.

## Closes when

ADR 0076's implementation boundary records items 2, 3, and 5 against this change with the exact commit, `implementation_status` reflects what is now implemented, the two open questions above carry the implementation evidence with their answers still explicitly open or explicitly settled, and any stale `at 6555119` paragraph the change invalidated carries an evidence refresh. `uv run --locked python scripts/check_repository.py` passes.

## Outcome

ADR 0076's Implementation boundary now records boundary ticket 2 as done in `6f7d772`, its preamble reports the two claims that changed and the two that did not, the status line records the second landed ticket, and open questions 2 and 3 carry implementation evidence with both still explicitly open. `decision_status` stays `accepted` and `implementation_status` stays `partial`.

### Verified against source before transcription, not taken from the ticket

**The authority and its vocabulary.** `crates/tiler-compiler/src/honourability.rs` carries `NumericalDimension` over `InputSubnormals`, `ResultSubnormals`, `Contraction`, `Reassociation` with a `CANONICAL_DIMENSIONS` constant fixing evaluation and reporting order; `DimensionBehaviour`; `HonouringMeans` with exactly the four variants item 3 names, the third carrying a `RelaxationRequirement`; and `NumericalHonourabilityFact`. Its module documentation states why `profile_key` and the canonical NaN bits are outside the dimension set, which the ADR now repeats because it is a decision and not an omission.

**The retirements are complete.** `grep -rn 'supports_strict_f32\|StrictF32Arithmetic\|requires_strict_f32' crates/` returns eight lines and every one is a doc comment recording the retirement — in `tiler-ir`'s schedule builder and model, and in `tiler-compiler`'s `request.rs`, `explain.rs`, `feasibility.rs`, and `physical.rs`. No code path reads any of the three. `feasibility.rs` states that axis tag `0x06` is retired rather than reassigned so a `v3` descriptor cannot collide with a `v2` one.

**The keys moved as claimed.** `PROFILE_DESCRIPTOR_DOMAIN` is `tiler.target-profile.descriptor.v3` and the rule-set key is `tiler.feasibility.phased-capability-and-numerical-honourability.v1`, whose own comment says it replaces `tiler.feasibility.phased-capability-bounds.v1`.

**The preference list.** `NumericalContractPreference` is ordered, nonempty, and required on the request; the resolved contract and the stated preference are retained together; and the request-subject encoding writes the resolved contract then a length-framed run over the stated preference. That is what makes the caller's fallback part of request identity, which is the specific thing a retry loop cannot buy — and it is now recorded in open question 2 as evidence rather than as a settlement.

**The explain event.** `ExplainEvent::NumericalHonourability { dimension, required, outcome, profile }`. One correction to this ticket's own description, which listed five fields: the declared means and the honoured alternative are not separate fields, they are carried inside `HonourabilityOutcome`. The ADR says so, because a reader checking the record against the enum would otherwise find a field list that does not match.

### The claim this ticket was most at risk of overstating, checked rather than repeated

The ticket says no governed profile constructs `SupportedWithExactEmulation`, `SupportedOnlyUnderDeclaredRelaxation`, or `Unsupported`. That is exactly right and the ADR now carries it with a one-line check: every construction site of those three is inside `crates/tiler-compiler/src/feasibility.rs`'s `#[cfg(test)] mod tests`, which begins at line 1331, and `crates/tiler-metal/src` contains no occurrence at all. The only non-test declaration anywhere is `GOVERNED_TARGET_HONOURABILITY` in `request.rs`, eight `SupportedExactly` behaviours across the four dimensions. Its deliberate omission of `FlushToZero { AlwaysPositive }` is recorded too, because it is the worked case for the unenumerated-behaviour `Unknown` path rather than an oversight.

### Found while verifying, and recorded rather than left for the next worker

**Boundary ticket 3 cannot be done as written.** **Fact.** `crates/tiler-compiler/src/lib.rs:14` declares `mod honourability;` privately and every type in it is `pub(crate)`. **Fact.** `crates/tiler-metal/Cargo.toml` names `tiler-artifact` and `tiler-ir` as its only normal dependencies, and ADR 0077's accepted packaging profile has no edge between `tiler-metal` and `tiler-compiler` in either direction. **Inference.** "Express `MetalSubnormalArithmetic` as a per-dimension honourability declaration in the shared form" is unreachable from `tiler-metal` today: the vocabulary must move to `tiler-ir`, which both crates already depend on, or a new edge must be admitted. Either is an ADR 0075 boundary and neither is decided here. The ADR's boundary entry for that ticket now states it, which is what "each inherits a concrete answer rather than re-deriving one" is supposed to mean. `declare-metal-numerical-honourability` is claimed by another agent on its own branch and was deliberately not edited.

### Gate

`uv run --locked python scripts/docs.py render` and the full `uv run --locked python scripts/check_repository.py` both pass; `git diff --check` is clean and `tkt lint` reports no problems.
