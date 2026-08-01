---
id: reach-a-reassociation-permitting-contract-from-a-bound-metal-declaration
title: Reach a reassociation-permitting contract from a bound Metal declaration
status: closed
priority: p2
dependencies: []
related: [admit-a-reassociating-contract-without-contraction, calibrate-and-activate-parallel-reduction-selection, package-a-multi-entry-bundle-from-one-expansion]
scopes: [research/apple-targets, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, apple-targets, reductions]
closed_reason: duplicate
closed_note: "Filed from base 2aa0824, before the parallel-reduction landing (merge 39b7ebc) reached main. Its three-way elimination is already settled on the landed tree: option 3 (measurement gap) is refuted by the ledger's retained measurement that this hardware flushes f32 subnormals in every math mode; option 2 (real coupling) is refuted by the fifth-preset design reasoning on register-a-flush-and-reassociate-numerical-contract (presets are points in a space, not strength-ordered, so flush-plus-reassociate is coherent); option 1 (composition artifact) holds and register-a-flush-and-reassociate-numerical-contract is its owner and resolution. The one obligation this ticket carried that register- did not — correcting calibrate-and-activate-parallel-reduction-selection's measurement target, which names an environment/contract pair that cannot both hold — has been added to register-'s graph maintenance."
---
## User-visible outcome

A compilation targeting a measured Metal declaration can state a reassociation-permitting numerical contract, so the reduction split that `admit-a-reassociating-contract-without-contraction` made selectable is reachable from an ahead-of-time compilation rather than only from the compiler's governed prototype profile.

## Why this exists

**Measurement — Apple M4 Max, macOS 27.0 build 26A5388g, `nightly-2026-07-19`, Apple metal 32023.883, base `2aa0824`, 2026-08-01.** Compiling the compiler's recognized serial-sum program (`out = strict_serial_sum(input * 2.0 + 1.0, axis 1)` over `f32[1, 4]`) against `BoundMetalCompileDeclaration::first_macos_apple9`'s profile `tiler.metal.macos-apple9.msl4-0.f32.v1`, once per registered contract:

| contract | outcome |
| --- | --- |
| `StrictF32` | refused — `InputSubnormals { required: Preserve }`, `DeclaredUnhonourable` |
| `FlushSubnormalsToZeroF32` | compiles; retains fused/1-kernel and materialized/2-kernel; **selects the 1-kernel fused plan** |
| `ReassociateF32` | refused — `InputSubnormals { required: Preserve }`, `DeclaredUnhonourable` |
| `RelaxedF32` | refused — `InputSubnormals { required: Preserve }`, `DeclaredUnhonourable` |

Every refusal carries the same measured evidence: the declaration's `InputSubnormals` row is `FlushToZero { zero_sign: PreservesSign }` with `means: Unsupported`, authority `tiler.metal.first-macos-apple9-msl4.measured.v1`.

**Inference — the split is unreachable ahead of time.** `admit-a-reassociating-contract-without-contraction`'s closing evidence, `the_reassociating_contract_reaches_the_split_through_compile`, selects a two-stage plan under `ReassociateF32`. That contract requires preserved input subnormals, and the only bound Metal declaration measures flushing. So the split is selectable only against a profile that no ahead-of-time compilation can name, and `tiler_macros::aot`'s derived `FlushSubnormalsToZeroF32` is not an arbitrary choice but the only survivor — which `only_one_numerical_contract_is_admissible_for_the_bound_declaration` already pins.

**Inference — this constrains a downstream ticket.** `calibrate-and-activate-parallel-reduction-selection` measures the three retained alternatives "on the exact qualified Metal environment". Under the one contract that environment admits, the split is not retained at all; under the contract that retains it, the environment refuses the compilation. The calibration target must be settled before that ticket can measure what it says it measures.

## Implementation keys

Settle which of these is true, with evidence, and record the elimination rather than the conclusion alone:

1. The coupling is a contract-composition artifact — `ReassociateF32` inherits `InputSubnormals: Preserve` from the strict resolution because every non-reassociation dimension sits at strict, and a reassociating contract that tolerates flushing is a coherent fifth preset rather than a relaxation. If so, this is a contract-registration question with a public boundary.
2. The coupling is real — reassociation over flushed subnormals is not soundly specifiable, so the refusal is correct and the split is genuinely unavailable on this hardware. If so, record it as an accepted limit and correct any text implying the split is reachable ahead of time.
3. The declaration is incomplete — a measured row exists or can be measured under which this Apple family preserves input subnormals (for example under different emitted fast-math attributes), making the refusal a measurement gap rather than a hardware fact.

Do not add a preset, widen a row, or relax the honourability check to make the split compile. The refusal is fail-closed behaviour backed by a measurement, and defeating it silently would let an expansion ship a reassociated reduction on a target whose measured arithmetic does not support the contract it claims.

## Required evidence

The chosen resolution names which of the three above it is and why the other two were eliminated. If a contract or row moves, the exact identity domains that move are enumerated and every pinned digest is recomputed on the tree the change lands into. If the outcome is an accepted limit, `calibrate-and-activate-parallel-reduction-selection`'s measurement target is corrected in the same change, because it currently names an environment/contract pair that cannot both hold.

## Closes when

An ahead-of-time compilation against a measured Metal declaration either states a reassociation-permitting contract, or the impossibility is recorded as an accepted limit with the dependent ticket corrected; every new check is perturbation-proved; and targeted tests plus the batch gate pass.

## Graph maintenance

- Reproduce the table above in one line by compiling the recognized serial-sum program against `BoundMetalCompileDeclaration::first_macos_apple9().profile()` once per `NumericalContract` value and reading the `TargetCompileFailure`.
- Public boundary, if a preset is the resolution: a new `NumericalContract` variant and any move of `MAX_NUMERICAL_CONTRACT_PREFERENCES` are session-visible and go to Tom under ADR 0075, exactly as the `ReassociateF32` acceptance did.
