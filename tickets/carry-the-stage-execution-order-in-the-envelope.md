---
id: carry-the-stage-execution-order-in-the-envelope
title: Carry a multi-stage variant's execution order in the envelope
status: done
priority: p1
dependencies: []
related: [carry-reconstructable-kernel-programs-in-the-neutral-envelope, expose-the-dispatch-record-on-a-decoded-artifact, route-the-runtime-loader-through-the-dispatch-record, carry-the-byte-offset-of-a-partial-binding-view]
scopes: [contracts/artifacts, implementation/artifact, implementation/runtime, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, runtime]
---
A variant that dispatches more than one stage encodes, and this build's reader refuses it. That refusal is correct and it is not the end state: it is the last gap between the dispatch record Tom decided on [`carry-reconstructable-kernel-programs-in-the-neutral-envelope`](carry-reconstructable-kernel-programs-in-the-neutral-envelope.md) and what the envelope actually carries.

**Fact — the gap, reproducible in one line.** `grep -n "FEATURE_MULTI_STAGE_PROGRAM\|SUPPORTED_FEATURES" crates/tiler-artifact/src/program/codec/model.rs` shows the key derived at the projector and absent from `SUPPORTED_FEATURES`. Its own doc comment states the reason: the neutral program section carries a program's canonical *identity* and not its dependency graph, so entries reach a reader in canonical stage-key order — identity's order, not execution order. Emitting the feature and refusing to read it is the fail-closed form of that gap; treating declaration order as execution order would be the silent one.

**Fact — the owner this inherited from is closed, and nothing live replaced it.** `carry-reconstructable-kernel-programs-in-the-neutral-envelope` is `done`. Its decision named "carrying execution order and dependency obligations explicitly" as a consequence of choosing the dispatch record, and [`expose-the-dispatch-record-on-a-decoded-artifact`](expose-the-dispatch-record-on-a-decoded-artifact.md) implemented every other part of that record. `grep -rln "multi-stage" tickets/` names seven ticket files and no live owner for this gap, which is why it is filed rather than assumed to be tracked.

**Why it is load-bearing rather than a nicety.** [`route-the-runtime-loader-through-the-dispatch-record`](route-the-runtime-loader-through-the-dispatch-record.md) records that its loader "genuinely cannot sequence a multi-entry variant" and keeps an `UnroutableEntries` refusal that is unreachable only because the decoder rejects such an envelope one layer earlier. A loader correct only by another layer's refusal is not correct. `tiler-compiler`'s materialized plans do produce multi-stage variants, so today a program's fused alternative can travel in an envelope and its materialized alternative cannot.

## Scope

Decide and encode what a reader needs in order to sequence a multi-stage variant: the stages' execution order and the dependency obligations between them, as encoded facts a decoder validates, in the same posture as every other dispatch-record row. This is a new encoded fact, so it is a manifest schema step and an identity-domain step — each stating its reason at its own site — rather than an accessor over rows that already exist.

**Do not close it by ordering entries at the encoder and declaring declaration order authoritative.** That is exactly the silent form the current refusal exists to avoid: the order a producer wrote is not a fact a decoder can check, and a consumer sequencing by position would have no way to verify the position meant what it assumed. Whatever is carried must either be checkable against something the envelope already proves, or be derived by the builder from the program's own dependency graph the way `binding_target` is derived from its stage access.

## Closes when

A consumer holding only encoded bytes can sequence a multi-stage variant's entries and name the dependency each edge rests on; `tiler.artifact.feature.multi-stage-program` is either supported or replaced by a key naming what actually remains unsupported; the refusal this build relies on is removed or restated as a narrower one with its reason; `docs/artifact-abi.md`'s required-feature table and item 3 of "Where the implemented profile is narrower than this contract" are updated to match; and `make full` passes.

## Design, verified against the code — 2026-07-27

Read in full: `crates/tiler-artifact/src/program/model.rs`, and `codec/model.rs` through `project_entries`. Two claims that seemed obvious before reading turned out false, and both are recorded because each would have produced a wrong change.

### The IR already publishes what the envelope lacks

**Fact — `crates/tiler-ir/src/program/model.rs`.** `VerifiedKernelProgram::execution_order()` returns the stages in "a deterministic topological order of the dependency graph, broken by canonical stage content rather than by insertion", and `dependencies()` returns typed edges with `predecessor()`, `successor()`, and `reason()`, where `DependencyReasonView` is `Data(value)` or `StorageHandoff(allocation)` and its doc states "every edge names the obligation it discharges".

That is exactly this ticket's closing condition — sequence the entries *and* name the dependency each edge rests on. The facts exist; they are simply not encoded.

### So the fact to carry is the edge set and the order, not a name for each intermediate

The tempting design is to give `BindingTarget::Internal` an identity so a reader can see that stage B's input is stage A's output. **It is the wrong one, and `BindingTarget`'s own doc says why**: "the shared IR's own canonical *value* key is crate-private to `tiler_ir::program` with no read view publishing it". Naming intermediates would force a new public surface on `tiler-ir` to encode a fact the program already exposes a better spelling of.

Derive instead, in `ArtifactEnvelope::project`, from the `VerifiedKernelProgram` the variant already carries — the same posture as `binding_target`, whose doc is the precedent: "the fact a decoded envelope cannot re-derive, so it is the one fact most worth taking from the program instead of accepting as a claim." A producer states nothing and so can contradict nothing.

### The mapping is by stage key, and needs no `tiler-ir` change

**Fact — `codec/model.rs:805`.** `project_entries` projects entries "into canonical stage-key order": the ordinal a producer pushed an entry at is presentation, the stage it realizes is identity. So the envelope's entries are already sorted by `stage_key`.

Each `StageRef` from `execution_order()` and `dependencies()` therefore maps to an entry position by computing its `stage_key` and finding that key among the sorted entries. No `StageRef::index()` accessor and no other `tiler-ir` addition is required.

### What to add

- `VariantRow` gains an execution order — entry positions in the canonical table — and a dependency edge list of `{predecessor, successor, reason tag}`. `DependencyReasonView`'s two arms get governed tags in the same adjacent forward/inverse pair style `model.rs` uses for every other shared-IR vocabulary.
- The decoder validates rather than trusts: the order is a permutation of the variant's entries, every edge's predecessor precedes its successor in it, and edges are canonically ordered without duplicates. Each is its own refusal in the existing taxonomy, not one collapsed "bad order".
- `FEATURE_MULTI_STAGE_PROGRAM` joins `SUPPORTED_FEATURES` (`codec/model.rs:98`), and `derived_features` keeps emitting it unchanged.

### Correction — why the identity domain steps

An earlier draft of this plan said `ARTIFACT_DOMAIN` must step because two artifacts differing only in stage order would otherwise share an identity. **That is false.** `push_variant` (`model.rs:1537`) folds the variant's program-section bytes, and `SectionKind::KernelProgramSubject` is the kernel program's canonical identity — which already differs between two stage orders.

The domain steps for the reason its own doc gives for `v2` and `v3`: a new field landing *inside* a per-variant record means a `v3` and a `v4` encoding of two *different* artifacts could produce equal bytes, and two artifacts that are not the same artifact must never share an identity. That is collision avoidance, not compatibility. `MANIFEST_DOMAIN` steps for the same reason at its own layer.

Backward compatibility is not a consideration: Tiler has no external consumers and no artifacts in the wild.

### Consequence to expect, and why it is correct

After this lands, an envelope decodes that `tiler-runtime` still cannot execute — it dispatches one entry. The refusal moves from the decoder to the loader and names a narrower thing, which is the point: `route-the-runtime-loader-through-the-dispatch-record` recorded that a loader correct only by another layer's refusal is not correct. `preflight-every-entry-of-a-multi-stage-route` owns the runtime half, and `prototype-metal-runtime-proof` needs both.

## Outcome

A consumer holding only encoded bytes can sequence a multi-stage variant and name the dependency each edge rests on, at `eb56c55`.

### What is carried, and why it is this and not a name for each intermediate

`VariantRow` gained an execution order — a permutation of the variant's entries — and the typed dependency edges that order discharges. Both are derived in `ArtifactEnvelope::project` from the packaged `VerifiedKernelProgram`'s own `execution_order()` and `dependencies()`, never stated by a producer, which is the `binding_target` posture: a producer cannot assert a correspondence its own plan contradicts.

The design the ticket warned against — ordering entries at the encoder and calling declaration order authoritative — is not what this does. The design that *looks* right and is not is giving `BindingTarget::Internal` an identity so a reader can see that stage B's input is stage A's output. `BindingTarget`'s own doc says why it fails: the shared IR's canonical value key is crate-private with no read view publishing it, so naming intermediates would force a new public surface onto `tiler-ir` to encode a fact the program already spells better.

### Carried *and* checked, which is the whole difference

An order alone cannot be checked. It says *an* order and not *why*, so a consumer could not distinguish a required sequence from an incidental one, and a decoder could not refuse an order that contradicts the program. With the edges present the decoder proves the order is a permutation of the entries and that every edge's predecessor precedes its successor in it.

Three refusals, each its own diagnostic rather than one collapsed "bad order": `StageOrderNotAPermutation`, `StageDependencyOutOfOrder`, and `StageDependencyOnItself`. All classify as `ArtifactCodecFailure::Invalid`.

### Correction — why `ARTIFACT_DOMAIN` steps

To `v4`, for the reason its own doc gives for `v2` and `v3`: the rows landed *inside* a per-variant record, so a `v3` and a `v4` encoding of two different artifacts could produce equal bytes.

**Not** because two stage orders would otherwise share an identity. They already differ under `v3` — `push_variant` folds the variant's program-section bytes, and that section is the kernel program's canonical identity, which the shared IR derives over its own dependency graph. An earlier draft of this ticket's design asserted the opposite; it was wrong, and writing it at the constant would have left a false justification where the next reader trusts it. The new rows make the order *readable*, not newly distinguishable.

`MANIFEST_DOMAIN` is unchanged: the manifest's own framing is untouched, and the rows sit inside the variant record the artifact domain already covers.

### Evidence

- `a_multi_stage_variant_encodes_and_this_reader_refuses_it` inverted into `a_multi_stage_variant_round_trips_with_a_recoverable_sequence`, against the real materialized serial-sum plan. It asserts the recovered *sequence* and that each edge is a `Data` dependency whose predecessor precedes its successor — not merely that it decoded, which is what a test checking only for `Ok` would have missed and is exactly the silent behaviour the old refusal existed to prevent.
- `an_execution_order_that_is_not_a_permutation_is_rejected` covers both directions of "not a permutation", because an omitted entry and a repeated one fail for different reasons. `a_stage_dependency_on_itself_is_rejected` covers the malformed edge.
- **Both new checks were confirmed able to fail** — neutered in `decode.rs`, observed failing, reverted.
- 961 workspace tests pass; `make full` green; the hardware run still ends in bit-for-bit agreement on both paths, with the envelope 40 bytes larger for the new rows.

### Measurement boundary

`StageDependencyOutOfOrder` is **not** reached from the codec's own tests, and the reason is structural rather than an omission: it needs two entries whose stated order contradicts an edge between them, and every codec fixture packages a single-stage program, where an edge with distinct endpoints cannot be built. The producer's multi-stage case exercises the satisfied direction against a real two-stage plan. A two-stage codec fixture is owed, and the refusal is currently proven only by the code path rather than by a case that reaches it.

### The surface this adds

`DecodedVariant::execution_order`, `DecodedVariant::stage_dependencies`, the `DecodedStageDependency` view, and the `StageDependencyReason` vocabulary are all new `pub` items under `tiler_artifact::program`, which is a reviewed **draft** boundary (ADR 0074 §7). Flagged for Tom rather than assumed accepted.

### The consequence to expect, which is correct

An envelope now decodes that `tiler-runtime` still cannot execute. `LoadRejection::UnroutableEntries` is reachable rather than shadowed by a decoder that refused one layer earlier — `route-the-runtime-loader-through-the-dispatch-record` recorded that a loader correct only by another layer's refusal is not correct — and its reason is restated: the limit is that this loader dispatches one entry, not that a sequence cannot be recovered. `preflight-every-entry-of-a-multi-stage-route` owns lifting it.

Both are now recorded dependencies of `prototype-metal-runtime-proof`, whose materialized-plan requirement was unreachable while the graph showed it ready.
