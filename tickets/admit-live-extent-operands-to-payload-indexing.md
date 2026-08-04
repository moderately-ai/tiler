---
id: admit-live-extent-operands-to-payload-indexing
title: Admit live extent operands to payload indexing
status: todo
priority: p1
dependencies: [admit-symbolic-extents-at-the-compiler-request-boundary]
related: [deliver-an-artifact-family-from-a-symbolic-region, bind-the-kv-cache-through-the-artifact-and-runtime-interface]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/runtime, implementation/build, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, kernel, artifact, runtime, identity, public-boundary]
---
## User-visible outcome

One compiled payload consumes a live symbolic input extent in its address and loop arithmetic, so changing `C` or `S` changes what the kernel indexes without compiling another payload or pipeline.

## Exact gap and ownership

**Fact at `b4e3478d42ce21ed68e23f772b643c6370d36498`.** `AbiRoot::InputExtent` already lets artifact expressions evaluate accessible ranges, guards, preconditions, and launch geometry from runtime-bound extents. `place_bindings` publishes only the evaluated offset/count and launch values. `BufferParameter` plus admitted launch builtins are the complete structured-kernel/Metal parameter population; no live input extent reaches the kernel body. The symbolic compiler-request ticket promises only that a program reaches planning or declines by the right reason, and the artifact-family delivery ticket discusses host-side range/launch evaluation. Neither owns a payload-consumable live scalar. This ticket is the missing prerequisite, not extra layout work.

An input extent is resolved from the program interface's existing `AbiRoot::InputExtent`; it is never derived from a KV storage descriptor. The dynamic-KV layout research selected exact-live dense packing in capacity-sized pooled buffers, so `C` and `S` themselves are the only address operands that path needs; no physical layout-root carrier follows this ticket.

## Required work

- Add a bounded typed structured-kernel operand for an existing governed input-extent root. Prove its symbol/interface/axis association against the scheduled region and refuse undeclared, foreign, late-phase, unused, duplicate, wrong-axis, wrong-type, or unbounded occurrences.
- Lower symbolic address and loop expressions to that operand without specializing its live value. Remove the migrated implicit-static path rather than letting one kernel choose between a baked extent and a parameter.
- Carry the root declaration, use sites, type, phase, canonical order, and read-only parameter transport through kernel identity, Metal emission, payload compilation subject, artifact construction/codec/decode/validation, and routed preflight. Runtime binds it from the same authoritative `AbiFacts` input extent used by range and launch evaluation; callers do not provide another list.
- Freeze canonical parameter bytes before `RoutingCommit`; the committed authority owns them and the backend binds exactly the declared transport. Missing backend support refuses during preparation, while a post-commit mismatch is terminal.
- Execute every kernel/program/artifact/schema identity step whole with ledgers and recomputed pins. The live extent value is excluded from artifact, payload, library, and pipeline identity.
- Make `deliver-an-artifact-family-from-a-symbolic-region` consume this capability rather than claiming that host-side launch/range evaluation makes a symbolic payload executable.

## Required evidence

- One artifact, payload subject, and pipeline handles dense F32 `[2,N]` at `N = 14` and `N = 15`; semantic `(row = 1, column = 0)` addresses bytes 56 and 60 respectively from the bound input extent.
- A bounded direct contraction consumes `S` as its contributor-loop bound and performs exactly `S` loads. Replacing the bound value by the neighbouring extent changes the oracle, while baking either value changes identity and fails the no-specialization assertion.
- Omitted, swapped-symbol, wrong-axis, late-phase, overflowing, misordered, and backend-misbound operands fail at the named layer. Remove each new check and watch its negative fail.
- Existing artifact range and launch expressions resolve from the same bound fact; a deliberate disagreement between host-side and payload use refuses before program work rather than running two meanings of `S`.
- Targeted IR/compiler/artifact/Metal/runtime/build tests, exact identity blast radius, `tkt lint`, `git diff --check`, guard, and the full gate pass.

## Unsupported cases

Only bounded unsigned input-axis extents available by `LiveDevicePreflight`. Dynamic target properties, physical layout facts, arbitrary caller scalars, unbounded loops, ragged per-row extents, negative values, and cursor/capacity specialization are outside this ticket and refuse.

## Public boundary

The exact structured-kernel operand, artifact row/view, routed parameter, and adapter binding methods are consequential public/schema surfaces. Produce the tested draft and put it to Tom; do not self-accept or release dependents before acceptance.

## Closes when

The tested public/schema draft is accepted, the identity step lands whole, every negative is fail-capable, and both the symbolic artifact-family and KV artifact/runtime tickets consume it without introducing a second scalar authority.
