---
id: bind-the-kv-cache-through-the-artifact-and-runtime-interface
title: Bind the KV cache through the artifact and runtime interface
status: todo
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family, admit-live-extent-operands-to-payload-indexing, define-the-runtime-kv-state-boundary, establish-a-dynamic-kv-physical-layout-authority]
related: [design-autoregressive-state-and-kv-cache, assemble-the-causal-self-attention-block-program, expose-the-dispatch-record-on-a-decoded-artifact, evaluate-retained-shape-relations-before-routing-commit]
scopes: [implementation/artifact, implementation/runtime, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, abi, kv-cache, language-model]
---
## User-visible outcome

The cache crosses the program boundary as ordered named inputs and outputs whose extents are bound per execution — so eight decode steps run from **one** artifact identity and one prepared pipeline, not eight of each.

## Required behaviour

`contracts/artifacts` is declared because the new manifest changes the neutral envelope schema and canonical identity, and `docs/artifact-abi.md` is the mapped identity/version ledger that must move in the same whole step. This is required bookkeeping for the already-authorized outcome, not a new product outcome.

Implementation consumes the exact-live dense survivor selected by `establish-a-dynamic-kv-physical-layout-authority`. Old and replacement payloads are packed at the governed semantic `C` and `S` extents inside separate capacity-sized pooled buffers; allocation length is not an address stride or artifact fact. `admit-live-extent-operands-to-payload-indexing` is therefore the only generic address-operand prerequisite. Reuse that semantic-root transport and do not define a KV-specific capacity-stride, layout-root, or second scalar spelling.

- `k_cache` and `v_cache` are named program inputs of shape `[8, C, 128]`; `k_rope` and `v_heads` stay the retained outputs L4 named, at `[8, S, 128]`. Nothing about capacity, the cursor, or the allocation crosses the boundary.
- Encode an authoritative complete ordered state-interface manifest derived from the verified semantic retained-state declarations — never a caller-supplied list. Each row names the cache input and layer, its sequence axis, the step input/axis, retained output/axis, and exact `S = C + T` relation. Decode validates the whole population and exposes it as `DecodedProgram::state_interface`; the runtime derives one owning `KvArtifactStateBindingSet` from that view with no caller population argument and consumes such a binding set both when constructing a `KvStateSet` and when preparing a route, so omitting K or V cannot redefine a smaller set as complete.
- Bind each validated manifest row to the exact routed dispatch output it denotes. `StateTransactionReporter` derives `KvRoutedOutputIdentity` from the authoritative row and routed dispatch for each member ordinal; neither callers nor adapters may restate the member/set/output identity in replacement evidence. Deliberately replayed evidence from another set, execution, member, or output must fail before publication.
- Re-export the existing IR-owned `StorageScalar` and `StorageEncoding` through `tiler_artifact::program`, whose decoded binding accessors already return those types, so the runtime state descriptor can name and compare storage without taking a new `tiler-ir` dependency or defining a duplicate physical vocabulary. Implement the exact transaction-bound input/route views, including committed shared-allocation joins, with no public `RoutedDispatch` escape. The final descriptor records capacity-sized pool ownership plus an exact live valid extent; it carries no physical stride because payload addressing derives from `C` or `S`.
- Treat that manifest as an artifact-schema and canonical-identity change: execute the owning version step whole, recompute every pinned identity on the merged tree, enumerate the blast radius, and update the schema/identity ledgers in the same commit. The manifest contains semantic/interface subjects only; it must not encode live state identity, cursor, capacity, generation, allocation, device/context, or poison status.
- `C`, `T`, and `S` are bound as input-axis extents at `AvailabilityPhase::LiveDevicePreflight`, and every accessible-range and launch expression is a formula over them evaluated during preflight, so an evaluation failure is a refusal rather than a post-commit surprise.
- Route each old and replacement member as an exact dense live span even when its owned buffer is longer: at `capacity = 18, C = 14, S = 15`, head 1 begins at bytes 7,168 and 7,680 and the accessible spans are 57,344 and 61,440 bytes inside separate 73,728-byte pool buffers. A capacity stride producing byte 9,216 is a deliberate failure.
- **No kernel may be specialized on `C`, `S`, or any cursor-derived quantity.** [The runtime execution contract](../docs/research/runtime/runtime-execution-contract.md) keys a prepared pipeline on its specialization values, so specializing on `S` would mint one pipeline per decode step and make a mutable inference quantity part of a cache key. Refuse it at artifact assembly, where the specialization values are packaged and the check is decidable.
- Two variants are packaged for the value contraction and selected per execution by an applicability guard over `S`: the tiled realization guarded on `S ≡ 0 (mod 16)`, the direct realization otherwise. Across C1's nine executions the tiled guard holds exactly once, at `S = 16`.

## Closes when

One assembled artifact routes at every C1 `S` from 10 to 18 with one identity; the guard selects tiled at `S = 16` and direct elsewhere; a program specializing on `S` is refused with its own diagnostic; decode round-trips and validates the complete K/V manifest; deliberate missing, duplicate, unrelated, and non-injective rows fail; construction and route preparation both refuse an omitted, extra, or reordered member; reporter-derived routed-output evidence refuses cross-set, cross-execution, cross-member, and cross-output replay; and a test asserts the single identity across all nine executions so that a per-step compilation would fail rather than pass quietly.
