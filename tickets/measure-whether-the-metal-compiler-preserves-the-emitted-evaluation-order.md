---
id: measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order
title: Measure whether the Metal compiler preserves the emitted evaluation order
status: in-progress
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, admit-a-refutation-only-derived-bound-conformance-oracle]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, apple-targets, conformance, measurement]
claimed_from: todo
assignee: agent-eval-order
lease_expires_at: 1786042476
---
## User-visible outcome

A measured answer to whether a Metal kernel's emitted floating-point evaluation order survives the backend compiler, and a target-profile fact that carries the answer — because today the property is asserted by a flag and declared by nothing.

## Why this exists

**Fact — [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) makes the plan's pinned evaluation order the whole basis of qualifying a permitted-divergence candidate.** That basis holds only if the order the artifact emits is the order the device executes.

**Fact — Tiler pins it today by asserting flags, not by consulting a fact.** `MetalNumericalRequirement::NoFloatingPointContraction` renders `-ffp-contract=off` and `SafeMathMode` renders `-fmetal-math-mode=safe` (`crates/tiler-metal/src/record.rs:76-132`); `crates/tiler-metal/src/tests.rs:1311` records the reason — "`-ffp-contract=off` is a defence against the *compiler* contracting a written multiply and add".

**Fact — no target profile declares the property.** `MetalTargetFacts` (`crates/tiler-metal/src/target.rs:755`) has five fields: language, platform, deployment minimum, per-type subnormal arithmetic, buffer binding limit. `CapabilityAxis` (`crates/tiler-compiler/src/target/feasibility.rs:211`) has seven, none about compiler-preserved evaluation order.

**Inference — so under a contract that permits contraction, Tiler would have no ground to keep asserting the flag that supplies its own pin**, and the executed order would become a property of a compiler nothing declares. `NumericalContract::RELAXED_F32` permits contraction and is registered, so the case is reachable rather than hypothetical.

## The bounded experiment

- **Inputs:** a kernel whose written order and a legal alternative order differ in bits — the four-operand set at `0x3f400000, 0x3e800000, 0x33400000, 0x33000000` already separates a serial fold from a two-by-two split by one ULP and is the natural seed. Compiled at each combination of `-fmetal-math-mode` and `-ffp-contract` the toolchain accepts, including the combinations Tiler does *not* assert.
- **Outputs and metric:** the executed result bits per combination, against the reference value of the written order. The metric is agreement or disagreement, not a magnitude.
- **What it must separate:** whether a disagreement is contraction (the existing golden-compilation work already probes this at `crates/tiler-metal/src/golden_compilation.rs:584`) or reassociation, which is the new question. Design the case so the two are distinguishable, and record it if they cannot be.
- **Unsupported cases and stop condition:** if no flag combination the toolchain accepts reorders the written sequence, the honest result is that this toolchain on this row does not, which is a bounded observation and not a portable guarantee — record it as such and stop rather than searching for a stronger claim.

## What it decides

A `Preserved` answer makes the pinned-order oracle sound on this row and supplies the fact a target profile would declare. A `NotPreserved` answer makes refusal class 3 of the derivation permanent for the affected contracts and fires [`admit-a-refutation-only-derived-bound-conformance-oracle`](admit-a-refutation-only-derived-bound-conformance-oracle.md)'s first clause. **Neither outcome is presumed here.**

## Explicit non-goals

Adding a target-profile field, which is a public boundary and a separate ticket; any performance claim; any change to which flags Tiler asserts.

## Closes when

Every accepted flag combination is measured on a named host with its exact toolchain version, the contraction and reassociation causes are separated or the inseparability is recorded, and the boundary states what the result does not generalize to.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) as the one refusal class with no closer in the graph.
