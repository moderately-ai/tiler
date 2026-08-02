---
id: integrate-the-contraction-vertical-into-the-runtime
title: Run one profile contraction end to end through the AOT and runtime route
status: done
priority: p1
dependencies: []
related: [design-attention-program-vertical, prototype-metal-runtime-proof, prototype-metal-aot-slice, realize-the-tiled-contraction-schedule-and-its-metal-emission]
scopes: [implementation/runtime, implementation/metal-aot, implementation/artifact, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifacts, contraction, language-model]
---
## User-visible outcome

Rung L3's stated capability — "one contraction runs end to end on Metal" — becomes true through the accepted AOT and runtime route rather than through a spike's own dispatch host. This is the remainder the L3 record deliberately did not claim.

## What is already true, and what is not

**Fact.** The [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) measured six realizations under a hand-written Objective-C host that loads a metallib, dispatches, checks `MTLCommandBufferStatusCompleted`, and reads back. That is a spike, not the route: it produces no artifact, has no identity, resolves no capability, and answers no applicability predicate.

**Fact — the route it must use instead.** An offline-produced metallib loaded through the accepted AOT path, with artifact identity carrying the offline compiler's provenance and exact native translator identity remaining `Unknown` per [ADR 0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md). The source-JIT compiler build measured elsewhere is not an input to this route and must not be substituted into its identity.

## Required delivery

- Artifact planning, ABI derivation, and buffer planning for a two-input one-output contraction — the first program in the project with two tensor inputs, so every place that assumed one is a place to check rather than to trust.
- Preflight before routing commit, and no fallback after allocation, partial encoding, submission, or semantic validation failure.
- Exact command-buffer terminal success before host validation readback.
- Bit-comparison of the executed result against the reference evaluator, with the spike's retained `result_sha256` values at the profile's cells available as an independent cross-check on a matching host row.
- Retention of asynchronous resources through their final device use.

## Non-goals

A transformer block, an attention program, the KV cache, batching, or more than one contraction in one program. L4 owns the block; this ticket owns making one contraction reach the device through the real route.

## Closes when

One contraction of the L3 profile executes through the accepted route with a terminal-success check before readback, its result is bit-identical to the reference, and a deliberately corrupted artifact is refused rather than executed.

**Satisfied 2026-08-02, with the extent boundary stated rather than implied.** One contraction of the L3 profile's index structure `td,od->to` runs end to end through the accepted AOT and runtime route at `activations[2,3] × weights[2,3] → projected[2,2]`: every device-decidable obligation discharged before `Preflight::commit`, exact `MTLCommandBufferStatusCompleted` before any readback, buffers retained across the wait, all five operand cases bit-identical to `tiler-reference`. The corrupted-envelope refusal was watched failing at the sidecar guard — `artifact.integrity: SectionDigestMismatch { section: 2 }`, never decoded or executed.

**What is not covered, and it is an extent limit rather than a route limit.** The L3 profile's own correctness cells each publish ≥ 1024 output elements while `direct` launches one invocation per element and the declared profile's `grid_axis_threads` is `4`, so `w_decode_kv` resolves `target.grid-axis` / `Rejected("target-infeasible")` before any plan composes. That row lives in `crates/tiler-build`, outside this ticket's scopes, and is owned by [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md). The host matched the spike's retained correctness row on every recorded field, so the `result_sha256` cross-check is withheld **for extent alone** and retained as an unavailable predicate rather than converted into a claim.

**Two results worth keeping.** `negative-zero-fold` returned `80000000` — the L3 record's own unseeded-fold counterexample, now reproduced through an artifact rather than the spike host — and `contraction-sensitive` returned `1.0` where a reassociated fold gives `0.0`, so `direct`'s strict-fold attribution holds on this route.

**Six sites assumed one tensor input and were widened to read the artifact's declared interface**: `bind_interface`, `plan_route`, `device_preflight`, `prove_member`, `run`, and the producer's sidecar. `allocate_alternative` now refuses a multi-input program on the direct path instead of filling both buffers from one slice. `tiler-artifact`'s proof layer already carried N inputs and enforced arity — verified by reading `place`, `verify_cases`, and `project_interface` — so it is unchanged. The `operands[0]`-for-`operands[ordinal]` perturbation was watched failing **while every one-input member still passed**, which is the defect a one-input proof could not have caught.

## Outcome (2026-08-02)

**Measurement — the contraction runs end to end through the accepted route.** `prototypes/serial-sum-compile` publishes a seventh member, `<base>.contraction.selected`: the L3 profile's index structure `td,od->to` at `activations[2, 3] x weights[2, 3] -> projected[2, 2]`, compiled through the ordinary entry point against `BoundMetalCompileDeclaration::first_macos_apple9`, carried through Metal emission and `xcrun` by `tiler-build`, and packaged as a neutral artifact with a two-operand proof sidecar. `prototypes/serial-sum-run` routes it through `tiler-runtime`: preflight and every device-decidable obligation before `Preflight::commit`, exact `MTLCommandBufferStatusCompleted` before any readback, and bit-comparison against the sidecar's reference-evaluated expectations. All five operand cases agree bit for bit, over 2 declared operands, 3 bindings, 4 threads, 1 dispatch.

The measured bits, on Apple M4 Max / macOS 27.0 `26A5388g` / Xcode 26.6 `17F113` / SDK 26.5 `25F70` / `metalfe-32023.883`:

| Case | `projected[2, 2]` |
| --- | --- |
| `ordinary` | `40c00000 41400000 41700000 41f00000` (6, 12, 15, 30) |
| `negative-zero-fold` | `80000000 80000000 00000000 00000000` |
| `non-canonical-nan` | `7fc00000 7fc00000 40c00000 40c00000` |
| `infinity` | `7f800000 7f800000 40c00000 40c00000` |
| `contraction-sensitive` | `3f800000 3f800000 40400000 40400000` |

**Measurement — the unseeded fold is now observable through the accepted route.** `negative-zero-fold` returns `0x80000000` on the two outputs whose every product is `-0.0`. That is the L3 record's own counterexample — a `+0.0`-seeded kernel returns `0x00000000` — reproduced through an artifact rather than through the spike's dispatch host. `contraction-sensitive` returns `1.0` where a reassociated fold returns `0.0`, so the `direct` realization's strict-fold attribution holds on this route.

**Fact — the L3 profile's own cells are unreachable at this profile, and the field is named.** Every correctness cell publishes at least 1,024 output elements (`w_decode_kv` is `M=1, N=1024`), and `direct` launches one invocation per output element. The declared profile's `grid_axis_threads` row is `4` (`crates/tiler-build/src/metal_declaration.rs`, `FIRST_MACOS_APPLE9`), a deliberately conservative compile guarantee the macOS 26.5 SDK contract does not bound above. Compiling `w_decode_kv` against that declaration resolves rule `target.grid-axis`, predicate `grid-axis`, `Rejected("target-infeasible")`, `required: Threads(1024), available: Threads(4)` — `NoFeasiblePlan`, before any plan composes. `2x3x3` (six outputs) refuses identically at `required: Threads(6)`. Reproduce by compiling `projection(1, 1024, 1024)` under `first_macos_apple9` and reading the explain trace's `target.grid-axis` record. Raising that row is a target-fact change in `tiler-build`, which this ticket does not hold; the remainder is filed as [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md).

**Fact — the host row matches and the `result_sha256` cross-check is still unavailable.** This host matches the spike's retained correctness row on every recorded field: Apple M4 Max, macOS 27.0 build `26A5388g`, SDK 26.5 build `25F70`, Xcode 26.6 build `17F113`, offline compiler `metalfe-32023.883`. The cross-check is nonetheless not taken, because the retained `result_sha256` values exist only at the profile's own cells, and those cells are refused above. The predicate is retained as unavailable rather than converted into a claim about a run that did not happen.

**Fact — every place that assumed one tensor input.** `bind_interface`'s `let [input] = inputs.as_slice()` refused two operands and bound one shape into the ABI facts; `plan_route` matched `ProgramInput(key)` against one key constant and produced a `Placement::Input` carrying no ordinal; `device_preflight` wrote one operand slice into whichever buffer it placed; `prove_member` and `run` read `case.inputs().next()` and ignored the rest; the producer's sidecar hardcoded a single `(InputKey, payload)` pair. Each was widened to read the artifact's own declared interface. `allocate_alternative` on the *direct* path still binds one operand slice by local knowledge and now refuses a multi-input program with `ProofError::DirectPathMultiInput` rather than silently filling both buffers from one slice. `crates/tiler-artifact`'s proof layer already carried N inputs and enforced arity, so it needed no change — verified by reading `program/builder.rs`'s `place`, `verify_cases`, and `project_interface`.

**Failure-path evidence, watched failing rather than asserted.** A byte flipped in the published contraction envelope makes the run exit 1 at the sidecar association guard — `the proof sidecar does not describe this envelope: the supplied envelope bytes are not the ones this sidecar names` — before any decode or dispatch. In-process, `probe_damaged_section_content` against the contraction's exact bytes yields `artifact.integrity: SectionDigestMismatch { section: 2 }`, paired with `probe_accepted_baseline` requiring the unperturbed subject to route. Substituting `operands[0]` for `operands[ordinal]` in `device_preflight` makes the contraction return `[58000000, 3f800000, 3f800000, 40400000]` against the reference's `[3f800000, 3f800000, 40400000, 40400000]` while every one-input member still passes — the defect a one-input proof could not have caught. Substituting `Placement::Input(0)` for `Placement::Input(ordinal)` fails the gate-reachable `a_two_operand_route_places_each_declared_input_at_its_own_ordinal` with `left: [0, 0], right: [0, 1]`.

## Dependency corrected at the third tiled stop (2026-08-01)

The supersede recipe re-pointed this ticket from `realize-the-strict-contraction-on-metal` onto the deferred tiled chain, and the coordinator reversed that edge on reading this ticket's own outcome: "one contraction runs end to end on Metal" is a claim about the accepted AOT and runtime route, not about which realization rides it, and the `direct` realization — compiled through the ordinary entry point, bit-compared at the profile cells — is a complete vehicle for it. The tiled realization arrives later as the performance-selected alternative behind `realize-the-tiled-contraction-schedule-and-its-metal-emission` (kept as related), and integrating on `direct` first is exactly the multi-kernel-may-be-correct-and-faster posture the architectural contract states.
