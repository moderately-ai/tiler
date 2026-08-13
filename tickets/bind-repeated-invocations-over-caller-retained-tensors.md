---
id: bind-repeated-invocations-over-caller-retained-tensors
title: Bind repeated invocations over caller-retained tensors from one artifact identity
status: review
priority: p1
dependencies: [admit-live-extent-operands-to-payload-indexing, establish-a-dynamic-kv-physical-layout-authority, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [admit-the-sequence-extension-concatenate-family, design-autoregressive-state-and-kv-cache, assemble-the-causal-self-attention-block-program, expose-the-dispatch-record-on-a-decoded-artifact, evaluate-retained-shape-relations-before-routing-commit]
scopes: [implementation/artifact, implementation/runtime, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, abi, consumer-neutral, language-model, class-generic-capability]
claimed_from: todo
assignee: worker-bind-repeated
lease_expires_at: 1786664533
---
## User-visible outcome

A caller that holds a tensor between invocations and rebinds it at a longer
extent each time runs every invocation from **one** artifact identity and one
prepared pipeline — not one of each per extent — and each invocation addresses
exactly the live payload it bound, never the allocation that happens to contain
it.

## Why this ticket was rewritten

**Superseded scope, 2026-08-04, under
[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md).**
This ticket previously read "Bind the KV cache through the artifact and runtime
interface" and required a KV-specific artifact-schema extension: an encoded
state-interface manifest naming cache inputs and layers, a
`DecodedProgram::state_interface` view, `KvArtifactStateBindingSet`,
`KvStateSet`, `KvRoutedOutputIdentity`, and a `StateTransactionReporter`, plus
an artifact canonical-identity version step to carry them.

All of it is withdrawn, and the withdrawal removes work rather than deferring
it. The manifest existed to stop a caller from supplying a partial, reordered,
or duplicated cache-binding population. A program's ordered named inputs and
outputs *already* state that population, and binding already refuses a wrong
count, a wrong rank, a wrong stored scalar, and a wrong literal extent on the
positional `bind_region` path — for every program, not for caches. Keyed
lookup on `RegionRequest` / artifact `BindingTarget::ProgramInput` is a separate
path with its own tests; it is not a universal wrong-key refusal on every bind
surface. A second, KV-named authority over the same subject would have been the
duplication the corpus keeps eliminating, and it would have put workload
vocabulary into the neutral envelope schema.
**The artifact-schema and canonical-identity version step is therefore no longer
required by this ticket**, and `contracts/artifacts` is dropped from its scopes
for that reason.

What survives is the part that was always generic, and the KV workload is only
the occurrence that found it.

## Required behaviour

Consume the exact-live dense representation measured by
[`establish-a-dynamic-kv-physical-layout-authority`](establish-a-dynamic-kv-physical-layout-authority.md).
The caller owns the allocation and its reuse policy; Tiler sees one bound value
per invocation at one bound extent.

- **A routed accessible span is derived from the bound extents, never from the
  allocation length.** A caller may bind a dense payload that occupies a prefix
  of a longer resource, and the invocation must address exactly that payload.
  Today the paths disagree: adapter `plan_dispatch` already accepts storage
  longer than the published live reach (`supplied < reach` →
  `UndersizedStorage`; longer storage passes), while the public
  `DenseRowMajorStorage` facade in `tiler` `checked_length` requires
  `declared == actual` and returns `BindError::StorageLengthMismatch` otherwise.
  On the path this ticket owns, that equality must be relaxed or bypassed so a
  capacity-sized pool bound at a shorter live extent is legal and still
  range-checked (`reach ≤ storage`, not `storage == dense(extents)`). Relaxing
  facade equality is a consequential bind-surface behaviour change; treat as
  draft until Tom accepts if it widens `BindError` / adapter contracts. The
  retained oracle is the layout record's: at a caller pool of 73,728 bytes
  holding `[8, 14, 128]` and `[8, 15, 128]` F32 payloads, head 1 begins at byte
  7,168 and 7,680 and the accessible spans are 57,344 and 61,440 bytes. An
  implementation that derives a stride from the resource length addresses byte
  9,216, stays in bounds, reads the wrong head, and **must fail** the oracle.
- **Extents are bound at `AvailabilityPhase::LiveDevicePreflight`**, and every
  accessible-range and launch expression is a formula over them evaluated during
  preflight, so an evaluation failure is a refusal rather than a post-commit
  surprise.
- **No kernel may be specialized on a value that is a per-invocation binding.**
  [The runtime execution contract](../docs/research/runtime/runtime-execution-contract.md)
  keys a prepared pipeline on its specialization values, so specializing on a
  bound extent would mint one pipeline per invocation and make a caller's
  mutable quantity part of a cache key. Refuse it at artifact assembly, where the
  specialization values are packaged and the check is decidable. This is a
  general rule about bound extents; it names no workload quantity.
- **Guarded variants discriminate on bound extents at route time, not at build
  time.** Package **two complete variants** (a synthetic multi-variant fixture
  or any already-landed realizations) whose applicability guards discriminate on
  a bound contracted extent `≡ 0 (mod 16)`, selected per invocation under
  `RoutingPolicy::StablePriority`, with the guard true only at extent 16 across
  the nine C1 extents. This discharges "one artifact, several plans" without
  requiring [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md)
  (status `deferred`; cooperative-tile public boundary not yet accepted). Scopes
  here do not include `implementation/metal`; if the real tiled Metal body
  becomes the close criterion later, that ticket must be added as a dependency
  and readiness accepted as deferred with it.
- **Reuse the live-extent operand transport.**
  [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md)
  is the only generic address-operand prerequisite. Do not define a capacity
  stride, a second physical-layout root, a storage-descriptor grammar, or a
  workload-named scalar spelling.

## Non-goals

Any type naming a cache, a cursor, a capacity, a generation, a layer ordinal, a
decode step, or a session. Any artifact-schema row describing retained state.
Any runtime object that outlives one invocation.

## Closes when

One assembled artifact routes at every extent of the C1 conformance row's nine
invocations with one identity and one prepared pipeline; the StablePriority
guard selects the extent-`≡ 0 (mod 16)` variant at extent 16 and the other
variant elsewhere (fixture or already-landed realizations; not the deferred
Metal tiled contraction body); a program specializing a kernel on a bound extent
is refused at artifact assembly with its own diagnostic, watched failing against
a deliberate perturbation; an invocation binding a payload inside a longer
resource addresses the exact live span on the consumer path this ticket owns
(after facade equality is relaxed or bypassed so capacity-sized pools at shorter
live extents are legal), and the wrong-stride interpretation is exercised and
fails the retained oracle; and a test asserts the single artifact identity
across all nine invocations so that a per-invocation compilation fails rather
than passes quietly.

## Fact audit — 2026-08-10

Ticket-audit wave B5 residual repair against report
`docs/research/documentation/ticket-audit-2026-08-10/reports/bind-repeated-invocations-over-caller-retained-tensors/e1e32eeca509_c99ac54950f2.md`.

**Correction — 2026-08-10.** `admit-the-sequence-extension-concatenate-family` moved from `dependencies` to `related`. Required behaviour and Closes when never construct a concatenate occurrence after the generic rewrite; concatenate remains a related capability for later stateful verticals, not a bind/route/identity prerequisite for nine-extent rebinding of one artifact.

**Correction — 2026-08-10.** Close criterion packages two complete multi-variant realizations with extent-rooted guards under `RoutingPolicy::StablePriority`, not the real Metal tiled contraction body owned by deferred `realize-the-tiled-contraction-schedule-and-its-metal-emission`. Prior wording ("Package the tiled realization … and the direct realization") could send a worker into that deferred chain despite scopes excluding `implementation/metal`.

**Correction — 2026-08-10.** Longer-resource live-span obligation now names the facade/adapter split: adapter `plan_dispatch` already allows `supplied ≥ reach`; `tiler` `checked_length` / `BindError::StorageLengthMismatch` still requires equality under `DenseRowMajorStorage` and must change on the path this ticket owns.

**Correction — 2026-08-10.** Supersession rationale no longer claims binding refuses "a wrong key" as a universal program-interface rule. Positional `bind_region` refuses wrong count/rank/stored scalar/literal extent; keyed request/artifact paths are separate.

Reproduce anchors: `rg -n 'if declared == actual' crates/tiler/src/route.rs`; `rg -n 'if supplied < reach' crates/tiler-runtime/tests/adapter_route/adapter.rs`; `rg -n 'status: deferred' tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md`; `rg -n 'OperandCountMismatch|RankMismatch|StorageScalarMismatch|LiteralExtentMismatch' crates/tiler/src/value.rs`.

## Fact audit — 2026-08-13 at `0b3ca334793e3975a2057f18424def2c251b1202`

Re-read this session at the dispatch base before editing. The 2026-08-10 wave is kept above; these verdicts replace any of its claims that have since moved.

- **Verified.** Facade `checked_length` still requires `declared == actual` and returns `BindError::StorageLengthMismatch`. Reproduce: `rg -n 'if declared == actual' crates/tiler/src/route.rs`.
- **Verified.** Adapter `plan_dispatch` still refuses only `supplied < reach` (`UndersizedStorage`); longer caller storage passes. Reproduce: `rg -n 'if supplied < reach' crates/tiler-runtime/tests/adapter_route/adapter.rs`.
- **Verified.** `realize-the-tiled-contraction-schedule-and-its-metal-emission` is `status: deferred`. This ticket's two-variant close is the synthetic guard fixture, not that body.
- **Verified.** Positional `bind_region` refuses `OperandCountMismatch`, `RankMismatch`, `StorageScalarMismatch`, `LiteralExtentMismatch`. Reproduce: `rg -n 'OperandCountMismatch|RankMismatch|StorageScalarMismatch|LiteralExtentMismatch' crates/tiler/src/value.rs`.
- **Verified, prerequisite now landed.** `AbiRoot::InputExtent`, `RoutedExtentParameter`, and `DecodedExtentOperand` exist. The envelope view remains a labelled draft (`Draft surface, not yet accepted` on `DecodedExtentOperand` / `EntryRef::extent_operands`). Not self-accepted.
- **Verified.** C1's nine invocations are `S ∈ {10, …, 18}`; only `S = 16` is `≡ 0 (mod 16)`.
- **Verified.** Oracle arithmetic: pool `18 × 8 × 128 × 4 = 73,728`; live spans `8 × {14,15} × 128 × 4 = {57,344, 61,440}`; exact-live head 1 at `{7,168, 7,680}`; capacity-strided head 1 at `18 × 128 × 4 = 9,216`.
- **Stale in the 2026-08-10 report, not in this ticket's Required behaviour.** Live-extent operands were `todo` at `c99ac549`; they are on this base. No KV-named second authority or artifact-identity step is required.

**Facade decision.** Relaxing `checked_length` equality would widen the public `BindError` / `DenseRowMajorStorage` contract and needs Tom's acceptance. This ticket owns the runtime adapter path, which already admits `reach ≤ storage`. The close is discharged there; the facade equality is unchanged and still draft if later accepted.

## Outcome

One assembled artifact routes at every C1 extent `S = 10 … 18` with one identity. Two complete live-extent variants are packaged: the first applicability guard is `IsMultipleOf(InputExtent(input, 1), 16)`, the second is constantly true. `RoutingPolicy::StablePriority` selects the aligned variant only at `S = 16` and the direct variant elsewhere. Across the nine invocations the artifact identity is identical; the selected kernel-program identities collapse to two values (one per variant), not nine. `DecodedExtentOperand` was not self-accepted.

A program whose launch or accessible-range formula names `AbiRoot::InputExtent` while the kernel baked a nonzero `element_count` for that input and declared no matching live operand is refused at `ArtifactProgramBuilder::push_variant` as `ArtifactBuildError::BoundExtentSpecialization`. Perturbing the subject — omitting the assembly check — fails as:

`a baked kernel must not assemble over a bound extent: VariantId { owner: ArtifactBuilderId(1), index: 0 }`

A 73,728-byte caller pool bound at live `S = 14` and `S = 15` dispatches on the adapter path (`reach ≤ storage`). Head 1 is bytes 7,168 and 7,680 and the live spans are 57,344 and 61,440, derived from the bound extent. Deriving the sequence from the allocation length yields capacity 18 and byte 9,216, which fails the retained oracle. No capacity stride, second physical-layout root, KV-named type, or artifact-schema identity step was added. `tiler.artifact-program.v16` does not step.

**Identity.** Artifact identity is `encode_identity(&ArtifactEnvelope)` and excludes the bound value. Payload, library, and pipeline subjects of the live-extent unit are unchanged across nine prepares and unequal to a baked `[2, 14]` / `[2, 15]` neighbour. The live MSL still contains `constant ulong& e0 [[buffer(2)]]` and neither `14ul` nor `15ul`.
