---
id: refuse-empty-live-domains-before-routing-commit
title: Refuse empty live domains before routing commit
status: todo
priority: p0
dependencies: [accept-the-live-extent-operand-public-surface]
related: [prove-a-schedule-verified-live-contraction-consumes-s, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n]
scopes: [implementation/ir, implementation/artifact, implementation/build, implementation/metal, implementation/runtime, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, extents, correctness]
---
## User-visible outcome

A live extent of zero cannot make a seeded contraction execute one product or make a live row-major access address an element of an empty axis. The invocation refuses before routing commit whenever the selected kernel requires a nonempty live domain.

## Exact gap and per-Fact audit at `f3e1efd3b3b4f896976b326e6a3d993147206cd3`

- **Verified.** `Extent::new` in `crates/tiler-ir/src/shape.rs` states that zero represents an empty axis, and `AbiFactBinder::bind_input_extent` in `crates/tiler-artifact/src/program/facts.rs` accepts zero.
- **Verified.** The static contraction verifier refuses `contracted_points == 0`; `verify_live_contraction` in `crates/tiler-ir/src/schedule/builder.rs` has no equivalent live predicate.
- **Verified.** `emit_contraction` in `crates/tiler-ir/src/kernel/lower.rs` emits the seed product before `serial_loop_range(1, bound)`. At bound zero it therefore performs one product even though the semantic contraction authority says `an unseeded strict fold has no empty result`.
- **Verified.** `live_contraction_loads` uses `bound.saturating_sub(1)`, masking zero as one seed load. The runtime fixture manually adds `1 <= N`, while `metal_entry_declaration` publishes `preconditions: Vec::new()` and generic artifact construction derives no nonzero precondition.
- **Verified.** The same missing guard lets a live row-major access address row 0/column 0 when the live inner axis is empty.

## Required work

- Derive a typed `extent >= 1` precondition for every selected live contraction and for each live row-major access that can execute an element. Do not rely on fixture-authored preconditions.
- Validate that the artifact carries the derived predicate against the same `AbiRoot::InputExtent` as the kernel operand, and enforce it before routing commit.
- Preserve genuinely empty computations that execute no access if the semantic family permits them; do not impose a global nonzero rule on every `Extent`.
- Remove the saturating zero oracle. Model the zero case as a refusal, not one seed load.

## Required evidence

- Bound zero refuses before program work; bound one succeeds and performs exactly one load/product; neighbouring positive bounds retain their load-count oracle.
- Independently remove schedule derivation, artifact validation, and preflight enforcement. Each unchanged negative must fail with quoted text.
- A live row-major kernel that addresses an element at zero refuses, while a proven no-work empty-domain neighbour remains legal if one exists.
- Targeted IR/artifact/build/Metal/runtime tests, numerical contract review, Clippy, rustdoc, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Changing the meaning of `Extent(0)`, defining seeded empty reductions, or defaulting an absent extent to one.

## Closes when

Every live kernel that requires a nonempty domain carries and enforces that requirement from schedule verification through routing commit, and zero cannot execute a seeded product or empty-axis access.
