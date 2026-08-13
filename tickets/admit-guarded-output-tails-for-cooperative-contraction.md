---
id: admit-guarded-output-tails-for-cooperative-contraction
title: Admit guarded output tails for the cooperative contraction
status: done
priority: p1
dependencies: [admit-a-cooperative-tile-over-shared-operands]
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, realize-the-strict-contraction-on-metal, admit-predicated-fixed-vector-map-tails]
scopes: [implementation/ir, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, physical-planning, public-boundary, decision, needs-tom]
---
## User-visible outcome

A cooperative contraction can represent a partial output block without reading outside either operand or writing outside the output, while every launched participant still reaches every required synchronization point.

## Source-first Fact audit — 2026-08-13, exact base `4275c14bb3c5fb1d73f8ae41cdc803d871742481`

Re-read at this base. The 2026-08-12 audit at `9144e4e1` is stale in the identity tags, the exact-sibling implementation claim, and the Activation paragraph.

1. **Verified.** [`contract_tiled`](../spikes/scheduling/metal_contraction_vertical/kernels.metal), anchor `const bool writes`, still computes `m` and `n` from a 16×16 workgroup, substitutes `0.0f` for inactive operand coordinates, leaves every barrier outside every predicate, and predicates only the output store. Left load uses `m < dims.m_extent`; right load uses `n < dims.n_extent`; store uses `writes`. [`host.m`](../spikes/scheduling/metal_contraction_vertical/host.m), anchor `dispatchThreadgroups:MTLSizeMake((n_extent + 15) / 16`, still rounds both free extents up. The six-cell record still contains `M = 1` and `M = 10`.
2. **Verified (decision-changing, unchanged).** There is not one complete active-output predicate governing all three boundary effects. For `mk,nk->mn` the left load is safe when `m < M`, the right load is safe when `n < N`, and the owning store is safe only when both hold.
3. **Verified.** [`TailPolicy`](../crates/tiler-ir/src/schedule/model.rs), anchor `pub enum TailPolicy`, still has only `Exact`. `verify_intrinsic`, anchor `schedule.tail != TailPolicy::Exact`, still requires `work_items`, `grid_threads`, and the logical iteration count to agree. [`BoundsProofKind::LinearRange`](../crates/tiler-ir/src/schedule/model.rs) still proves operand coordinates only over the logical iteration domain.
4. **Verified.** [`OperationKind::Load`](../crates/tiler-ir/src/kernel/model.rs) is unconditional. [`OperationKind::Predicated`](../crates/tiler-ir/src/kernel/model.rs) has no results. [`KernelBuilder::predicated`](../crates/tiler-ir/src/kernel/builder.rs) still cannot return a loaded value. There is still no `GuardedLoad`, select, or value-yielding conditional.
5. **Verified.** `guard_values`, anchor `Collects the values that denote a schedule-derived governed predicate`, still recognizes one flat global-prefix guard and one cooperative commit-prefix guard. `Walk` still retains only `guarded: bool`.
6. **Verified.** [`OwnershipProofKind::OneGlobalInvocationPerOutput`](../crates/tiler-ir/src/schedule/model.rs) still states the right theorem. No new ownership-proof kind is needed.
7. **Verified (was imprecise, now exact).** Shapes and launch values are still static. This remains intrinsic construction and verification. Checked ceiling division and multiplication may refuse while constructing the proposal.
8. **Verified (policy).** `admit_exact_cooperative_contraction` still never substitutes the direct contraction. A guarded proposal must not normalize into Exact or direct. Independent complete alternatives may still participate in ordinary selection.
9. **False.** KIR operation tags do **not** end at `0x1e`, and `0x1f` is **not** free. `push_operation` at this base: `StagedStore` is `0x1d`, `StagedLoad` is `0x1e`, `SerialLoopRange` is `0x1f`, `InputExtent` is `0x20`. The next unused append-only KIR tag is `0x21`. `push_schedule` still has `let TailPolicy::Exact = schedule.tail; bytes.push(0x01);` — tail tag `0x02` is still free. `OperationView` is still `#[non_exhaustive]`. No artifact record decodes the KIR operation tree.

**Additional stale claim in Activation.** The paragraph "The exact-divisible cooperative-contraction boundary is accepted but still unimplemented" is **false** at this base. [`admit_exact_cooperative_contraction`](../crates/tiler-ir/src/schedule/blocked.rs) and `verify_cooperative_contraction` are implemented. KIR lowering of `ReductionTopology::CooperativeContraction` still returns [`KernelDiagnostic::CooperativeLoweringShape`](../crates/tiler-ir/src/kernel/lower.rs), anchor `Representable, not lowered`. This ticket lands Predicated schedule admission and the guarded KIR vertical on top of that exact schedule sibling; it does not reconstruct the exact sibling.

## Recommended complete boundary

Accept one generic iteration-tail policy and one narrow value-producing KIR effect, then admit exactly their cooperative-contraction composition.

### Schedule relation

Add `TailPolicy::Predicated` with no payload. It means that the launch may be a strict superset of the logical iteration domain, while the execution binding derives which launched coordinates are logically active and the verifier proves that only active coordinates own observable boundary effects. It does not mean contributor padding, inactive subgroup lanes, scalar peeling, or a backend-chosen mask. Those are different axes.

The first admitted combination is the accepted blocked-workgroup cooperative-contraction binding only. For logical output `[M, N]` and block `[B_m, B_n]`:

- `work_items = M × N` remains the logical output population;
- `grid_threads = ceil(M / B_m) × ceil(N / B_n) × B_m × B_n` is the padded launch population, with every operation checked for overflow;
- zero output work still skips dispatch;
- `K` remains positive and exactly divisible by its contracted tile width; and
- `TailPolicy::Exact` retains its equality rules and byte-for-byte identity.

The active predicates are derived, never caller-authored or duplicated in the schedule:

- `row_active := m < M` authorizes the left boundary load;
- `column_active := n < N` authorizes the right boundary load; and
- `output_active := row_active ∧ column_active` authorizes the owning store.

The source of `m`, `n`, `M`, and `N` is the same verified blocked execution binding and logical output shape used by the exact sibling. No free predicate AST, callback, stored extent copy, or default is introduced.

### KIR relation

Add one scalar operation, illustratively `GuardedLoad { predicate, buffer, offset, bounds, inactive }`, with these exact semantics: when `predicate` is true it performs the bounds-witnessed load and returns that value; when false it performs no memory access and returns `inactive`. The builder requires a Boolean predicate and an inactive value whose type equals the buffer element type. The operation is not a vector masked load and carries no lane-mask or target-ISA claim.

For this first cooperative F32 topology, canonical lowering supplies `+0.0f` as the inactive value, performs the left and right guarded loads under their distinct derived predicates, and then performs every staged store unconditionally. Every participant therefore initializes its two staging slots and reaches every phase and round barrier. The accumulator and staged loads may run for inactive outputs; their result is unobservable because only `output_active` may enclose the final store.

The zero is a canonical inert filler, not a reduction contributor or algebraic identity. The verifier proves that an active output reads left slots populated by `row_active` participants and right slots populated by `column_active` participants, so no active output can observe an inactive filler. Consequently this decision consumes no reassociation, contraction, signed-zero, or padding-neutrality permission and does not derive the filler from an empty-domain identity.

The KIR verifier must replace the current undifferentiated `guarded: bool` treatment for this path with role-sensitive schedule-derived guard facts. It verifies each guarded load against the operand axis it protects, verifies the final store under both predicates, rejects any barrier nested under either predicate, and compares the complete staged access/round structure against the scheduled topology. A random Boolean, a row guard on the right load, a column guard on the left load, or only one store guard is a named refusal.

### Strictness, consumption, and identity

The guarded schedule is a distinct physical proposal. Construction refuses a wrong binding, unsupported topology, zero block dimension, overflowed padded launch, non-exact `K`, missing or swapped operand guard, incomplete output guard, conditional staged store, uninitialized staging slot, guarded barrier, wrong inactive value, or a mismatch between logical ownership count and `M × N`. It never rewrites itself to `Exact`, pads `K`, switches to `direct`, or asks a backend to infer a guard.

The later Metal ticket owns translation of the verified guarded load to an MSL conditional expression or branch with non-evaluated inactive memory access and owns the six-cell device evidence. A backend without an exact translation declines the operation; it does not need a quantitative target-profile row or runtime query. The CPU vector tail remains separate: its lane-shaped masked load and fault-suppression fact are target/ISA obligations, whereas this operation is scalar structured control over one invocation.

Encode `TailPolicy::Predicated` at schedule tag `0x02` and the guarded load at KIR operation tag `0x21` (the next unused append-only tag after `InputExtent` at `0x20`; `0x1f` is already `SerialLoopRange`), including predicate, buffer, offset, bounds witness, and inactive value references. Existing exact schedules and kernels remain byte-identical. The new schedule and KIR bytes flow through the identities already carried by plans and artifacts, with no duplicate artifact field.

## Ranked options

1. **Generic `TailPolicy::Predicated` plus the role-checked scalar guarded load above.** Best correctness and fail-closed behavior, best long-term separation of iteration coverage from execution binding, and the smallest KIR effect that can express the retained algorithm. Host validation is linear in the small fixed guard/effect population; device work matches the retained one-load/one-staged-store structure.
2. **A general value-yielding conditional region.** It can express the same kernel and may eventually serve more consumers, but it immediately admits branch result arity, merge typing, nested effects, and convergence cases no current second consumer needs. It is equally capable and materially harder to verify and maintain, so it is dominated until another accepted value-yielding branch consumer exists.
3. **Unconditionally stage zero, then predicated-load and overwrite the slot.** This can be made correct without a value-yielding load, but it changes every tile load into two staged writes, requires a new same-participant staged-rewrite proof, and adds device traffic to avoid one narrow KIR operation. It is worse in performance and proof complexity.
4. **Keep exact cooperative contraction only and let `direct` cover partial shapes.** Correct and simplest, but it cannot realize the retained tiled correctness population and leaves the accepted physical alternative unavailable at common decode shapes.
5. **Reuse one `output_active` predicate for all loads, guard only the store, physically pad inputs, or reconstruct the ternary in Metal.** Reject. The first loses operand data active peers need, the second reads out of bounds, the third changes placement/storage and adds a different physical plan, and the fourth bypasses structured-kernel verification.

## Strongest counterpoint and reversal evidence

The guarded-load operation expands the public KIR for one physical family, and a general value-yielding conditional would be more expressive. Reverse to that broader construct only when another accepted consumer needs a value from arbitrary conditional computation and a bounded design proves branch effects, merge typing, convergence, identity, and backend lowering without weakening this load's fault-suppression rule. Reverse to the double-staged-write form only if measurement shows the extra writes are optimized away on every selected backend and its staging proof is simpler in source, neither of which is established now.

## Required evidence

- Admit exact and partial `[M, N]` blocks under the same blocked binding while retaining distinct `Exact` and `Predicated` identities.
- Perturb row, column, and output guards independently and show the specific left-load, right-load, and write refusal text.
- Show that replacing either guarded load with an ordinary load fails bounds refinement and that guarding either staged store leaves a named incomplete-staging refusal.
- Show that placing any phase or round barrier under a predicate fails convergence.
- Exercise `M = 1`, `M = 10`, a partial `N`, both extents partial, exact neighbours, zero work, launch overflow, and nondivisible `K`.
- Prove every active output has exactly one writer, every inactive invocation has none, every staging slot is initialized before its read, and no active output observes an inactive filler.
- Perturb tail and guarded-load tags and operands for identity inequality while retaining every old exact schedule and kernel pin.
- Keep the guarded KIR unexecutable in a backend that declines `GuardedLoad`; no source-emission fallback or implicit direct kernel may appear.

## Decision request

Accept the generic predicated iteration-tail plus role-checked scalar guarded-load boundary above; revise it; or keep cooperative contraction exact-only. Acceptance chooses the schedule/KIR public surface and its strict proof relation. It does not authorize the Metal implementation, cost selection, contracted-axis padding, a general value-yielding conditional, or vector masked-memory support.

## Accepted decision — 2026-08-12

Tom accepted the recommended complete boundary in the live Codex coordination thread by replying `okay agreeed, next decision`. The relay source is Tom's direct response in that thread.

The accepted public surface is `TailPolicy::Predicated` with no payload plus the narrow scalar guarded-load KIR operation and the role-sensitive proof relation above. The first admitted composition is the blocked-workgroup cooperative F32 contraction only. Row activity authorizes the left load, column activity authorizes the right load, their conjunction authorizes the output store, staged stores and barriers remain unconditional, and the inactive `+0.0f` value is an unobservable canonical filler rather than contributor padding or a numerical identity.

The acceptance does not authorize a general value-yielding conditional, vector masked memory, contracted-axis padding, implicit direct substitution, backend-inferred guards, or Metal emission. Exact tails remain byte-for-byte and behaviorally unchanged. Independent complete direct and tiled alternatives may still participate in ordinary physical selection and ADR 0051 pre-commit routing; the guarded proposal itself may never normalize into another approach.

## Activation and closure

The exact-divisible cooperative-contraction **schedule** sibling is implemented at this base (`admit_exact_cooperative_contraction`). KIR lowering of that topology is still refused as representable-not-lowered. This ticket owns the atomic Predicated schedule/KIR vertical on top of that sibling and must not reconstruct the exact admission path. [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md) remains the consumer that proves the emitted Metal body and device results.
