---
id: widen-the-region-realization-to-consumable-dimensions
title: Widen the scheduled-region realization to every consumable numerical dimension
status: todo
priority: p2
dependencies: []
related: [implement-first-profile-numerical-policies, express-metal-honourability-in-the-shared-form]
scopes: [implementation/ir, implementation/compiler, implementation/metal, implementation/artifact]
shared_scopes: []
paths: []
tags: [implementation, numerics, mature-product]
---
Four dimensions of the resolved numerical contract are ones an admitted operation can consume and `tiler_ir::schedule::NumericalRealization` cannot carry: operand permutation, signed-zero elimination, and the NaN and infinity assumptions. `implement-first-profile-numerical-policies` made every one of them expressible in the contract and then refused any contract that resolves one non-strictly, because a scheduled region has nowhere to record which resolution was chosen and two contracts differing only there would reach one region identity.

**Fact — the refusal is implemented and tested.** `crate::policy::unrepresentable_dimension` walks the canonical dimension order, skips `REALIZED_DIMENSIONS`, and refuses a dimension whose stated resolution differs from the one this build realizes and which `operation_capabilities` says some admitted operation can consume. `verify_request` runs it ahead of contract admission, and `RequestError::UnrepresentableNumericalDimension` names the dimension, the arithmetic type, the required behaviour, the behaviour this build realizes, and the first operation that can consume it. Four cases are driven individually in `policy.rs`.

**Fact — why the widening was not done there.** Adding a field to `NumericalRealization` changes `NumericalRealization::new`'s signature, which is source-breaking for `crates/tiler-metal/src/tests.rs` and the `crates/tiler-metal/src/lib.rs` doctest. That is ADR 0075's `a breaking change to an existing public signature` category and it is outside `implement-first-profile-numerical-policies`'s declared scopes, which are `implementation/ir`, `implementation/reference`, and `implementation/compiler`.

**Inference — the widening is not only a signature change.** `crates/tiler-metal/src/emit.rs`'s `realization_requirements` reads the realization by field, so a new field compiles and is silently ignored. A permutation or signed-zero obligation the emitter never checked would be a conformance claim about a dimension nothing assessed, which is the failure mode ADR 0076 item 6 exists to prevent. So the emitter's requirement derivation and `MetalNumericalGap` must widen in the same change, and `tiler-artifact`'s `NumericalFacts`, `push_numerical`, `push_resources`, and `DecodedNumerical` must widen with them or the codec drops dimensions the region carries.

**Inference — the signed-zero dimension is the one with measured evidence behind it.** ADR 0076 measures `(-0.0) + (+0.0)` returning `0x00000000` under `-fmetal-math-mode=safe` and `0x80000000` under `relaxed` and `fast` for the emitter's own `MultiplyThenAdd { scale 1.0, bias +0.0 }` shape. So a contract that permits contraction or reassociation — which selects a relaxed mode — and forbids signed-zero elimination is a contract the measured Apple row does not honour, and today no layer can express that pairing at all.

## Closes when

`NumericalRealization` carries every dimension `crate::policy::operation_capabilities` says an admitted operation can consume; both `push_numerical` encoders and `ResourceRequirements` carry them, exhaustively per dimension; the Metal emitter derives a requirement or records a gap for each; the artifact codec round-trips them; `REALIZED_DIMENSIONS` grows to match, so `policy.rs`'s `every_realized_dimension_is_consumable` and the four refusal cases move together; and the affected identity fixtures are rebaselined on the merged tree rather than taken from either branch. `make full` passes. (Citation corrected at landing: the Python gate this ticket originally named was retired by `e197176`.)
