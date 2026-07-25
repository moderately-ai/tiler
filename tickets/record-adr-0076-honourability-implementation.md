---
id: record-adr-0076-honourability-implementation
title: Record ADR 0076 items 2, 3, and 5 as landed and answer two of its open questions with implementation evidence
status: todo
priority: p2
dependencies: []
related: [compose-numerical-honourability-and-retire-the-strict-boolean, expose-the-numerical-contract-preference-list]
scopes: [contracts/decisions]
shared_scopes: []
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
