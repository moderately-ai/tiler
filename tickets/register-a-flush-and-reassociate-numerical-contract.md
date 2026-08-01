---
id: register-a-flush-and-reassociate-numerical-contract
title: Register a flush-and-reassociate numerical contract
status: todo
priority: p1
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal, calibrate-and-activate-parallel-reduction-selection]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A caller can state a numerical contract that both flushes `f32` subnormals to the sign-preserving zero and permits ordered reassociation of one same-operation operand sequence, so a parallel reduction is expressible on Apple hardware.

## Why this exists

**Measurement, `realize-parallel-reduction-strategies-on-metal`, base `d0b8445`.** That ticket landed every *target* fact a parallel reduction needs on the authoritative macOS Metal profile — threadgroup memory at 32,768 bytes, the `threadgroup_barrier(mem_flags::mem_threadgroup)` realization sourced from MSL 4.0 §6.9.1, and both resolutions of the reassociation dimension — and then could not plan a parallel reduction, because the blocker is on the request side rather than the target side.

The four registered contracts are `tiler.strict-f32.v1`, `tiler.flush-f32.v1`, `tiler.relaxed-f32.v1`, and `tiler.reassociate-f32.v1`. **None both flushes subnormals and permits reassociation.** The two granting regrouping are built on the strict reading and require *preserved* subnormals, which the retained Apple measurement records this hardware refusing in every math mode; the one this hardware delivers widens subnormals alone and grants no regrouping. `tiler_compiler::session::CompileRequest` accepts only that four-value preset enumeration, so no caller outside `tiler-compiler` can express the combination.

Every parallel reduction strategy regroups the declared contributor sequence, so on the one measured hardware row all of them are unreachable. The compile-side refusal lands on the `InputSubnormals` dimension and never reaches reassociation at all.

## Implementation keys

Register a fifth preset beside the four above — a governed contract key, its `NumericalPolicyPreset`, and its public `NumericalContract` spelling — widening exactly two dimensions from strict: both subnormal dimensions to the sign-preserving flush, and reassociation to permitted. Widen nothing else: contraction, operand permutation, signed zero, and both exceptional-value assumptions stay at their strict resolution, because each is a freedom an admitted operation could consume and none is carried by the region IR, so two programs differing only there would share one identity.

The presets are deliberately not ordered by strength, so this is a fifth point in the space and never a "relaxed flush". Name it for what it authorizes rather than for a position in an ordering.

## Required evidence

The new contract compiles a reassociating reduction against `BoundMetalCompileDeclaration::first_macos_apple9` and the portfolio retains the multi-pass split and the single-workgroup tree beside the serial fold. The reference oracle agrees with each retained alternative at its own declared order. A contract widening a dimension the target refuses is still a typed refusal naming the dimension.

`crates/tiler-build/src/metal_plan.rs`'s `no_registered_contract_both_flushes_subnormals_and_permits_reassociation` is the activation trigger and **must be updated by this ticket**: it asserts the current absence and fails the moment this lands. Its two halves record what the gap was, so replace it with the positive claim rather than deleting it.

## Closes when

The preset is registered, both parallel strategies reach a portfolio on the authoritative Metal profile, every check is mutation-proved, and targeted tests/Clippy plus `make full` pass.

## Graph maintenance

- This unblocks `realize-parallel-reduction-strategies-on-metal`'s executable backend evidence, which is otherwise complete on the profile-fact half. Re-dispatch that ticket's execution half once this lands.
- Measured crossover and winner activation stay in `calibrate-and-activate-parallel-reduction-selection`.
- `calibrate-and-activate-parallel-reduction-selection`'s stated measurement target must be corrected when this lands: it names the exact qualified Metal environment and the retained alternatives together, and today no contract makes both hold — the environment refuses the contracts that retain the split. Name the new preset as the measurement contract in the same change. (Carried over from the closed duplicate `reach-a-reassociation-permitting-contract-from-a-bound-metal-declaration`, which owned this correction.)
- `package-a-multi-entry-bundle-from-one-expansion` depends on this ticket: under the new preset a selected multi-entry plan becomes expressible, which with the reduction grammar is that ticket's trigger.
- Whether the *public* preset enumeration is the right shape for a fifth entry, or whether a caller should be able to compose dimensions directly, is a public-boundary question for Tom rather than something to settle inside this ticket.
