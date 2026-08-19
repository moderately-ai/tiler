---
id: replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges
title: Replace zero live-bounds sentinels with ABI-derived accessible ranges
status: in-progress
priority: p0
dependencies: [associate-live-extent-operands-with-symbolic-semantic-interface-axes, package-the-admitted-live-schedule-into-a-symbolic-kernel-program]
related: [bind-repeated-invocations-over-caller-retained-tensors, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n]
scopes: [implementation/ir, implementation/artifact, implementation/runtime, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, abi, correctness]
claimed_from: todo
assignee: worker-live-bounds
lease_expires_at: 1787159586
---
## User-visible outcome

A backend receives the routed accessible span derived from the live ABI fact. It neither receives a zero sentinel nor reconstructs reach from private row/column state, so a longer allocation cannot silently change the payload's address meaning.

## Exact gap and per-Fact audit at `f3e1efd3b3b4f896976b326e6a3d993147206cd3`

- **Verified.** `crates/tiler-ir/src/kernel/verify.rs` `access_elements` returns zero for live row-major and live contraction accesses; the schedule proof therefore carries a zero-length linear range rather than a dynamic witness.
- **Verified.** `crates/tiler-runtime/tests/adapter_route/fixture.rs` `live_extent_program` constructs zero-length byte windows and `let accessible = CheckedMultiply(zero, live_n)`. `one_live_extent_artifact_indexes_dense_f32_at_two_n_without_baking` explicitly asserts `accessible_bytes == Unsigned(0)` while claiming the range uses `N`.
- **Verified.** The scalar adapter's `plan_dispatch` replaces zero with `prepared.rows * parameter.value() * 4`; `ScalarImage::entry_for` replaces the routed read/write reach with locally computed values, so its supposed comparison is self-comparison. A conforming adapter that honors `RoutedBinding::accessible_bytes()` receives zero.
- **Verified.** Existing wrong-stride examples calculate helper values but do not perturb the routed range. They do not prove the route's authority is load-bearing.

Reproduce the wrong-positive with `cargo test -p tiler-artifact --lib one_live_extent_artifact_indexes_dense_f32_at_two_n_without_baking -- --nocapture`; the passing test asserts the zero range.

**Stale — do not run (2026-08-19).** That test was deleted at `2cb7c83c`; the command now selects nothing. The block above is preserved as history; the audit below supersedes it.

## Fact audit at base `441f321583ee08856b2b8f87e056ebabf487277b` — 2026-08-19 by worker-live-bounds, before any edit

The audit above was taken at `f3e1efd3`, 492 commits back. The prerequisite [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md) landed at `2cb7c83c` (an ancestor of this base) and **withdrew the entire worked instance this ticket's evidence is written against**. Verdicts, then the consequence.

- **Fact 1 — verified, unchanged.** `crates/tiler-ir/src/kernel/verify.rs "pub(super) fn access_elements("` still returns `Ok(0)` for `LiveRowMajorSource`/`LiveRowMajor` and for a `ContractionOperand` under `ReductionTopology::LiveContraction`, on the `BoundsProofKind::LinearRange` arm only.
- **Fact 2 — half verified, half false.** *Verified:* `crates/tiler-runtime/tests/adapter_route/fixture.rs ".push_abi_binary(AbiBinaryOp::CheckedMultiply, zero, live_n)"` and the two zero-length `ByteWindow`s are still there, and the tiler-artifact twin `crates/tiler-artifact/src/program/tests/support/live.rs "push_abi_binary(AbiBinaryOp::CheckedMultiply, zero, live_n)"` too. *False:* `one_live_extent_artifact_indexes_dense_f32_at_two_n_without_baking` **does not exist at this base** and asserts nothing. `2cb7c83c` deleted it. The stated reproduce command reports `Summary [0.000s] 0 tests run: 0 passed, 341 skipped` / `error: no tests to run` under nextest. There is no passing test asserting the zero range.
- **Fact 3 — verified, imprecise in wording.** `crates/tiler-runtime/tests/adapter_route/adapter.rs ".filter(|_| binding.accessible_bytes() == 0)"` still substitutes `self.prepared[position].rows * parameter.value() * ELEMENT_BYTES` (the ticket writes `prepared.rows`; the site is indexed per entry). `crates/tiler-runtime/tests/adapter_route/image.rs "let live = !routed.extent_parameters().is_empty();"` still sets `routed_read`/`routed_write` from the image's own arithmetic on the live path, discarding the routed value — a self-comparison. **But both arms are now unreachable** (see below), so "a conforming adapter receives zero" has no reachable subject.
- **Fact 4 — false.** The wrong-stride oracle it names is `a_capacity_pool_addresses_the_exact_live_span_and_the_wrong_stride_fails`, present at `f3e1efd3` in `crates/tiler-runtime/tests/adapter_route/main.rs` and **deleted by `2cb7c83c`**. `grep -rn "wrong_stride\|wrong stride" crates/` returns one unrelated comment in `crates/tiler-compiler/src/governed.rs`. There is no oracle to retain, so Required-evidence item 3 has no subject either.

### The blocking finding: no live-extent artifact can exist at this base, by construction or by decode

Both routes to a routed live range are refused by name, deliberately, as the accepted fail-close `2cb7c83c` landed:

- **Construction.** `crates/tiler-artifact/src/program/builder.rs "pub(super) fn check_extent_operand_association("` refuses a live operand over a *fixed* semantic axis as `ExtentOperandStaticAxis`; the *symbolic* alternative is refused earlier still by the same file's `"fn static_interface_shape("` as `SymbolicSemanticInterface`. Its own doc states the symbolic arms "are production-reachable only once a symbolic semantic interface can open a builder at all".
- **Decode.** `crates/tiler-artifact/src/program/codec/validate.rs "fn check_extent_operand_static_axes("` refuses **every** declared extent-operand row: "Until a symbolic interface representation exists on the wire (the packaging decision), every declared row is therefore refused by name". Forged bytes fail closed too.

Consequently `crates/tiler-runtime/src/load.rs "fn bind_extent_parameters("` iterates a list a decode has proved empty, and `RoutedEntry::extent_parameters()` is empty on every reachable route. **Measurement:** `cargo nextest run -p tiler-artifact -p tiler-runtime -E 'test(live)'` runs 28 tests at this base and every one is a *refusal* test — `a_live_operand_over_a_fixed_semantic_axis_refuses_at_artifact_construction`, `the_live_extent_member_refuses_at_artifact_construction`, `a_well_placed_live_extent_row_over_the_static_interface_is_refused`, and siblings. No live-extent execution, routing, or addressing test exists.

Both `crates/tiler-runtime/tests/adapter_route/main.rs "fn the_live_extent_member_refuses_at_artifact_construction()"` and its tiler-artifact twin record the withdrawal explicitly, and both name `package-the-admitted-live-schedule-into-a-symbolic-kernel-program` as the ticket that lifts it.

### What this does to the Required work and evidence

- **Required evidence 1** ("the routed range moves at two neighbouring extents and the backend-observed range is byte-for-byte the routed value") — **unobtainable**. Needs a routable live artifact.
- **Required evidence 2** (four perturbations, each flipping an unchanged assertion) — the *routed range* and *adapter local dimensions* perturbations have no reachable subject; only the ABI-fact and encoded-expression perturbations survive, and they would only exercise refusals already landed by `2cb7c83c`.
- **Required evidence 3** (retained wrong-stride oracle) — **the oracle was deleted**; nothing to retain.
- **Required work 2** (remove the adapter reconstruction and self-comparison) is mechanically doable but is now dead-code removal that evidences nothing — and doing it *alone* is worse than the status quo: the fixture's `accessible_bytes` is still the statically-zero `CheckedMultiply(zero, live_n)`, so an adapter that consumed the routed value "unchanged" would plan a zero-byte live binding rather than fail closed.

### The zero is not a local sentinel; it is load-bearing for three coupled static invariants

Reading the construction path rather than the symptom: `crates/tiler-ir/src/program/builder.rs` `push_stage` requires `buffer.element_count * element_bytes == view.window.length`, and then requires `evaluate_static_abi(accessible_bytes) == view.window.length` (`"KernelProgramBuildError::AccessibleBytesDisagreement"`). Because `access_elements` yields 0, the window length must be 0, so the ABI expression must *statically evaluate* to 0 — which is exactly why the fixtures spell it `CheckedMultiply(zero, live_n)`: it mentions the live root while being pinned to zero. `image.rs` says so outright: "the ABI expression cannot name 8N without failing the static window check."

Worse, `crates/tiler-ir/src/program/builder.rs "fn static_facts(&self) -> AbiFacts {"` binds only the semantic program's **declared static** extents, so on a genuinely symbolic `[2, N]` subject that same agreement check cannot evaluate at all. Publishing `rows * N * 4` therefore requires a symbolic element count in the bounds witness, a symbolic `ByteWindow` length, **and** a symbolic rather than static ABI agreement rule in the `tiler.kernel-program.v11` contract — together, not one at a time.

### Two authority steps this ticket does not carry

1. **Public boundary.** A dynamic range witness means a new `BoundsProofKind` variant in `tiler_ir::schedule`. That enum is accepted public vocabulary, and the accepted `LiveRowMajorSource` record states the opposite intent in `crates/tiler-ir/src/schedule/model.rs "consumed in the payload"` — the live extent is deliberately *not* a schedule value. The ticket's own phrasing ("derived from the artifact's existing `AbiRoot::InputExtent` authority") inverts the layer order: the schedule sits below the program and artifact and cannot read them. The workable direction is the reverse — the schedule publishes a symbolic witness that the program layer lowers into the ABI expression — which is a different design from the one this ticket specifies.
2. **Identity domain.** `BoundsProofKind` is written into the canonical scheduled-region identity encoding beside `TAG_LINEAR_RANGE = 0x11`. A new tag moves every live region's schedule identity and cascades to kernel, kernel-program, and artifact identity. The neighbouring tag comments show these assignments are reconciled *across* accepted decision packets (the `0x0C`/`0x0D` gap); claiming one unilaterally is not a worker step.

### Recommendation

Add the missing edge: this ticket depends on [`package-the-admitted-live-schedule-into-a-symbolic-kernel-program`](package-the-admitted-live-schedule-into-a-symbolic-kernel-program.md), which is `todo`, `ready`, and has a fully `done` dependency closure (no cycle: it does not reach this ticket). Until it lands there is no subject to route, no oracle to retain, and no perturbation to run. The public-boundary and identity-domain steps above should be split out and put to Tom rather than folded in here.

## Required work

- Replace the zero placeholder with a typed dynamic bounds/range witness derived from the artifact's existing `AbiRoot::InputExtent` authority. For the dense F32 `[rows,N]` worked instance, publish exactly `rows * N * 4`, with checked arithmetic.
- Make the runtime adapter consume the routed offset and accessible bytes unchanged when validating storage and preparing work. Remove `prepared.rows`/allocation-length reconstruction and live-path self-comparisons.
- Preserve the distinction between exact live reach and allocation capacity: longer storage is legal only when the published range fits; it does not redefine stride or reach.
- Refuse an unrepresentable range, an unbound fact, a zero sentinel on a live access, and any disagreement before routing commit. Do not default or repair malformed artifact data in the adapter.

## Required evidence

**Blocked as written (2026-08-19)** — items 1–3 have no reachable subject at this base; see the Fact audit's "What this does to the Required work and evidence".

- The routed range moves at two neighbouring extents and the backend-observed range is byte-for-byte the routed value.
- Independently perturb the ABI fact, encoded range expression, routed range, and adapter's local dimensions. Each unchanged assertion must fail with quoted text.
- The retained longer-allocation wrong-stride oracle fails when capacity is substituted for live reach.
- Targeted IR/artifact/build/runtime tests, Clippy, rustdoc, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Binding the scalar parameter bytes to their backend transport is [`bind-frozen-live-extent-bytes-at-declared-backend-transports`](bind-frozen-live-extent-bytes-at-declared-backend-transports.md). Associating the operand with symbolic semantic meaning is the prerequisite [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md).

## Closes when

The artifact route is the sole authority for accessible offset/range and no adapter can turn the former zero sentinel or its own allocation metadata into a second meaning of the live extent.
