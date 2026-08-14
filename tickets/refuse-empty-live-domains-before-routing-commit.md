---
id: refuse-empty-live-domains-before-routing-commit
title: Refuse empty live domains before routing commit
status: in-progress
priority: p0
dependencies: [accept-the-live-extent-operand-public-surface]
related: [prove-a-schedule-verified-live-contraction-consumes-s, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n]
scopes: [implementation/ir, implementation/artifact, implementation/build, implementation/metal, implementation/runtime, contracts/numerics, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, extents, correctness]
claimed_from: todo
assignee: worker-empty-live-domain
lease_expires_at: 1786679676
---
## User-visible outcome

A live extent of zero cannot make a seeded contraction execute one product or make a live row-major access address an element of an empty axis. The invocation refuses before routing commit whenever the selected kernel requires a nonempty live domain.

## Exact gap and per-Fact audit at `4fb0427319b1504e1549e03ba023ac486343a743`

- **Verified.** `Extent::new` in `crates/tiler-ir/src/shape.rs` states that zero represents an empty axis, and `AbiFactBinder::bind_input_extent` in `crates/tiler-artifact/src/program/facts.rs` accepts zero.
- **Verified.** The static contraction verifier refuses `contracted_points == 0`; `verify_live_contraction` in `crates/tiler-ir/src/schedule/builder.rs` has no equivalent live predicate.
- **Verified.** `emit_contraction` in `crates/tiler-ir/src/kernel/lower.rs` emits the seed product before `serial_loop_range(1, bound)`. At bound zero it therefore performs one product even though the semantic contraction authority says `an unseeded strict fold has no empty result`.
- **Verified.** `live_contraction_loads` uses `bound.saturating_sub(1)`, masking zero as one seed load. The runtime fixture manually adds `1 <= N`, while `metal_entry_declaration` publishes `preconditions: Vec::new()` and generic artifact construction derives no nonzero precondition.
- **False; repaired 2026-08-13.** The current live row-major lowering is the no-work empty-domain neighbour: `emit_live_row_major` places every load and store inside `serial_loop_range(0, columns)`, so an inner extent of zero executes no element access. No selected live row-major kernel at this base can address row 0/column 0 when that extent is empty. A later live access shape that can execute an element independently of its live extent would still owe the conditional nonzero requirement below.

## Required work

- Derive a typed `extent >= 1` precondition for every selected live contraction and for each live row-major access that can execute an element. Do not rely on fixture-authored preconditions.
- Validate that the artifact carries the derived predicate against the same `AbiRoot::InputExtent` as the kernel operand, and enforce it before routing commit.
- Preserve genuinely empty computations that execute no access if the semantic family permits them; do not impose a global nonzero rule on every `Extent`.
- Remove the saturating zero oracle. Model the zero case as a refusal, not one seed load.

## Required evidence

- Bound zero refuses before program work; bound one succeeds and performs exactly one load/product; neighbouring positive bounds retain their load-count oracle.
- Independently remove schedule derivation, artifact validation, and preflight enforcement. Each unchanged negative must fail with quoted text.
- The current live row-major kernel is exercised at zero and remains legal as a proven no-work empty-domain neighbour; a census asserts that no selected live row-major shape at this base can execute an element at zero.
- Targeted IR/artifact/build/Metal/runtime tests, numerical contract review, Clippy, rustdoc, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Changing the meaning of `Extent(0)`, defining seeded empty reductions, or defaulting an absent extent to one.

## Implementation record (2026-08-13)

- `KernelBuilder::build` derives a crate-private required-nonzero extent only from `ReductionTopology::LiveContraction`; every other topology, including `LiveRowMajor`, derives none. The fact is excluded from kernel identity because the already-folded verified schedule identity is its complete authority.
- `KernelProgramBuilder::push_stage` resolves each requirement through the verified kernel buffer and the checked stage access to exactly one distinct program `InputKey` and logical `Axis`. Zero or multiple owners fail as `KernelProgramBuildError::RequiredInputExtentBinding`; no key or axis is guessed.
- `KernelProgramBuilder::build` materializes the canonical existing-ABI formula `1 <= AbiRoot::InputExtent { key, axis }`, sorts and deduplicates multiple requirements, and conjoins them with the producer's guard. Insertion checks reserve the exact interned-node capacity before build, and a recoverable verification failure removes only the derived nodes.
- Artifact construction continues to replay the complete program ABI and runtime selection continues to evaluate the replayed applicability guard before routing commit. The generic backend entry keeps no independently authored launch precondition. No new schema, ABI root, identity field, public type, or public method was introduced; the only additive public spelling is one variant on an already `#[non_exhaustive]` builder-error enum.
- Live-contraction program and artifact identity bytes change because their effective applicability guard now states the missing requirement. Kernel identity, schema domains, static contractions, and `LiveRowMajor` subjects do not change.
- The exact-base scope guard identified the compiler integration oracle as a direct `implementation/compiler` edit. The coordinator authorized that narrow scheduling scope so the duplicate saturating-zero oracle is removed and bound one is pinned at the compiler-facing consumption path; no compiler production code changes.

### Independent S2 review hardening

- A nontrivial producer-authored guard, `2 <= right axis 0`, is conjoined with the derived `1 <= left axis 1` requirement. Its complete two-by-two truth matrix proves neither half overwrites or substitutes for the other.
- Exact ABI-arena boundaries are exercised on the production builder. Four authored size/launch roots, one authored Boolean root, 4,088 shared-DAG levels, and three derived nodes reach exactly 4,096. A failed build restores the authored arena, and repair/retry produces the same 4,096 nodes, guard, and whole-program identity as a fresh twin. One further authored DAG level makes the stage's derived guard demand node 4,097; `push_stage` refuses transactionally, a repeated attempt returns the same typed error, and the still-uncommitted builder admits another authored node.
- The optimized test binary reports 0.05 seconds for the 4,096-node recovery/twin subject and 0.02 seconds for the 4,097-node transactional refusal on this host. `push_abi_node` does clone the intern table while no live requirement is present, but this boundary fixture did not expose material host cost; no production optimization was added to a correctness ticket on that evidence.
- Two independent stages resolving to the same `(InputKey, Axis)` retain exactly one `InputExtent` root. Two stages resolving distinct keys retain the same canonical guard and whole-program identity when inserted in reverse order, because stage identity is canonically ordered by stage key rather than builder insertion.
- More than one distinct owner for one required tensor/axis is a deliberately retained future fail-closed branch, not a claimed current fixture population. Every selected live-contraction kernel in the present vocabulary has exactly one buffer for its live-input role; verified current subjects reach the zero-owner refusal and the one-owner admission, while a future wider buffer vocabulary must refuse rather than guess if it makes the multi-owner branch inhabited.

### Subject perturbations

- **Producer derivation:** replacing the `LiveContraction` arm of `required_nonzero_input_extents` with `Vec::new()` and running `cargo test -p tiler-ir --lib -- program::tests::a_live_contraction_derives_the_same_input_extent_guard_and_refuses_zero --exact` failed: `assertion left == right failed; left: Boolean(true), right: Boolean(false)`.
- **Artifact replay/validation:** replacing the adopted program guard in `ArtifactProgramBuilder::push_variant` with a fresh `BooleanLiteral(true)` and running `cargo test -p tiler-artifact --lib -- program::tests::a_live_contraction_nonzero_guard_round_trips_from_the_same_input_extent --exact` failed artifact verification with `diagnostics: [UnusedExpression]`.
- **Runtime preflight enforcement:** changing `select_variant` to accept any successfully evaluated guard, including `false`, and running `cargo test -p tiler-runtime --test adapter_route -- a_zero_live_contraction_refuses_before_payload_or_program_work --exact` failed: `S=0 must fail applicability before routing commit`.
- **Authored/derived conjunction:** changing the authored guard's left literal from two to one made the unchanged matrix fail at `left axis 1 = 1, right axis 0 = 1`: `left: Boolean(true)`, `right: Boolean(false)`.
- **Exact recovery boundary:** removing one shared-DAG level from the failed-build/recover/retry subject made the exact-node assertion fail with `left: 4095`, `right: 4096`.
- **Transactional first excess:** removing one shared-DAG level from the 4,097 subject made `push_stage` succeed, and the unchanged refusal check failed with `node 4,097 must refuse: StageId { ... index: 0 }`.
- **Same-key deduplication:** swapping the second stage's live input from `left` to `right` made the unchanged root census fail with two roots, `[(InputKey("left"), Axis(1)), (InputKey("right"), Axis(1))]`, instead of the one `left` root.
- **Distinct-key insertion order:** withholding the `right` requirement only from the reverse-insertion twin made the unchanged guard comparison fail with the forward eleven-node arena against the reverse eight-node arena; this reaches the derived fact rather than perturbing the identity assertion.

### Verification

- `cargo check -p tiler-ir -p tiler-artifact -p tiler-build -p tiler-metal -p tiler-runtime -p tiler-compiler --all-targets` passed.
- `cargo nextest run -p tiler-ir -p tiler-artifact -p tiler-build -p tiler-metal -p tiler-runtime -p tiler-compiler` passed 2,676 tests and skipped 2.
- Package-targeted live-contraction, artifact round-trip, `LiveRowMajor` zero-neighbour, Metal live-extent, and runtime preflight subjects passed, including the exact one-load oracle at bound one and moving oracles at 14 and 15.
- Package Clippy with `-D warnings`, package rustdoc with `RUSTDOCFLAGS='-D warnings'`, and package doc-tests passed.
- `cargo fmt --check`, `tkt lint`, `make citations`, and `git diff --check` passed. The exact-base `tkt guard` is rerun on the committed branch so it observes the branch diff rather than the pre-commit empty ref.
- The coordinator deliberately held `make full` and workspace-wide nextest while another branch owned heavy Cargo work; the completed package population above is the review-ready boundary, not a claim that the repository-wide gate ran.
- The S2-review amendment reran the complete `tiler-ir` population: 1,066 library tests and 1,145 nextest tests passed; `cargo check --all-targets`, Clippy with `-D warnings`, rustdoc with `-D warnings`, and 18 doc-tests (one ignored) passed. The delta changes no non-IR production path, so the prior cross-crate package population remains the proportional evidence boundary.

## Closes when

Every live kernel that requires a nonempty domain carries and enforces that requirement from schedule verification through routing commit, and zero cannot execute a seeded product or empty-axis access.
