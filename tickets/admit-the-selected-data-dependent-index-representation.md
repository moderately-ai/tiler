---
id: admit-the-selected-data-dependent-index-representation
title: Admit the selected data-dependent index representation
status: todo
priority: p1
dependencies: [accept-adr-0108-data-dependent-index-coordinate-siting, decide-the-data-dependent-index-representation-public-surface]
related: [revise-adr-0108-with-a-complete-data-dependent-index-vertical, admit-an-invocation-scoped-gather-index-validation-receipt, emit-the-indirect-gather-on-metal]
scopes: [implementation/ir, implementation/reference, implementation/compiler, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, verification, identity, decision, needs-tom, public-boundary]
---
## Status repair — 2026-08-19, this ticket was `blocked` with no surviving ground

Found by the ticket-population sweep and confirmed by the coordinator at `1c56a977`. Both declared dependencies are `done`: `accept-adr-0108-data-dependent-index-coordinate-siting` (ADR 0108 carries `decision_status: "accepted"`, accepted 2026-08-12) and `decide-the-data-dependent-index-representation-public-surface` (which records Tom's acceptance under its own `## Accepted decision — 2026-08-18` heading). The body below stated no blocking reason at all, and `git log -S "status: blocked"` attributes the status to `f01c1c92 tickets: gate data-dependent index public surface` — it was gating on exactly the decision that has since been accepted.

`.ticketsplease/decision-queue.md` item 12 already recorded the consequence on 2026-08-18 — "the p1 carrier `admit-the-selected-data-dependent-index-representation` is unblocked and joins the solo identity-migration queue" — but **the ticket's status was never flipped**, so a p1 sat parked for a day and held `emit-the-indirect-gather-on-metal` and the receipt tickets behind it. Status moved to `todo`; no other field changed.

**What is accepted, and what is still not.** Tom accepted option B — literal-only with the source-side `index_access` field — as the exact reviewed packet at `a25f4268b768f1b0391db34798676f910d4f1660`. That acceptance covers the public surface this ticket implements. It does **not** cover sourced boundary/domain gather support, which stays a separate future decision, and it authorizes no kernel, artifact, Metal, cache, or dispatch route past the KIR `body-refinement` wall. The `needs-tom` and `public-boundary` tags stay on the frontmatter because the *surface* is Tom's and must not be re-spelled by a worker; they are not a second unresolved gate.

**Scheduling.** This is a solo identity migration — it moves the `LogicalAccess` grammar and carries the named narrow ADR 0108 schedule-clause amendment — so it takes the solo migration slot and must not run beside other identity-moving work.

## User-visible outcome

The representation ADR 0108 ultimately accepts is admitted as a complete verified logical index form, while every existing direct-access byte and verifier guarantee remains unchanged.

## Required boundary

- Implement only the accepted nested-read or tagged-access form; do not blend the candidates.
- Carry the outer coordinate, nested source tensor, complete source coordinates, U32 value semantics, rank and reachability checks, exact bounds obligation, compaction/remapping, alpha-equivalence, canonical ordering, encoding, views, errors, reference evaluation, compiler recognition, and explanation as one coherent population.
- Preserve all old canonical bytes and pin every identity-domain step the accepted ADR requires.
- Retain the gather bound as either a static proof or one exact mandatory invocation-validation obligation. This ticket does not mint a runtime receipt and cannot treat an obligation as discharged.
- Keep direct access verification and ADR 0046 unchanged; scatter and data-dependent output shapes remain absent.

## Closes when

The selected form is constructed and inspected through the reviewed surface, all exhaustive consumers are updated, static proof reaches executable coverage, the dynamic form remains pending on the named receipt, subject perturbations independently fail, and targeted plus full gates pass.

## Exact-base Fact audit — 2026-08-22, `3cca543807be9951fd9ced9cd31cad4db2736a36`

Every Fact below was re-read in its named source file at this base. The audit covers this ticket's `Required boundary` and the accepted packet's numbered Facts and its `Identity, schema, registry, and cache consequences` block, because the packet is the specification this ticket implements.

| # | Packet claim | Verdict at this base |
|---|---|---|
| 1 | ADR 0108 selects an append-only tagged gather access; ADR 0075 reserves the boundary for Tom | **verified** |
| 2 | The read/scalar-leaf association owner is `verify_pointwise_region` | **imprecise** — the function exists and behaves as described, but lives in `crates/tiler-ir/src/schedule/builder/elementwise.rs`, not `schedule/builder.rs` |
| 3 | No current type states the extra read's owner, order, or multiplicity | **verified** |
| 4 | Two closed proof classes are derivable; vacuity is a property of the complete result domain, not the index shape | **verified**, and now executable |
| 5 | `PendingIndexRefinementReceipt`/`IndexRefinementUnknown` are not truthful dynamic-gather results | **verified** |
| 6 | The accepted ADR selects no law, access, schedule, proof, registry, or diagnostic tags | **verified** |
| 7 | Census remains `IndexNode` 5, `IndexExprClass` 3, `IndexDomainUnknownReason` 3 | **verified** |
| 8 | Both dependencies are `done` | **verified** |
| 9a | Index-region access tags are `0x01`/`0x02` | **verified** (`builder/identity.rs`, `encode_region`) |
| 9b | `IndexRealizationLaw` uses `0x01`-`0x0D` | **verified** — 13 variants, encoder tags 1..=13 |
| 9c | `LogicalAccess` uses `0x01`-`0x09`, with `0x0A`/`0x0B` merely *reserved* by a pending packet | **false / stale** — the live-row-major and partitioned-copy packets have landed. `LogicalAccess` now has 11 variants; `TAG_LIVE_ROW_MAJOR_SOURCE = 0x0A`, `TAG_LIVE_ROW_MAJOR_CONSUMER = 0x0B`, `TAG_PARTITIONED_COPY_SOURCE = 0x0D`, and `0x09` is retired-and-never-reused. **`0x0C` is still free and the source explicitly reserves it for `GatherSource`**, so the packet's assignment survives — but for a different reason than the packet gives. |
| 9d | Bounds-proof tags `LinearRange`/`ReductionDomain` are at `0x01`/`0x02` | **false, and false when written** — they are `TAG_LINEAR_RANGE = 0x11` and `TAG_REDUCTION_DOMAIN = 0x12` in `crates/tiler-ir/src/schedule/model.rs`, and `git log -S "TAG_LINEAR_RANGE: u8 = 0x11"` attributes them to `912bb110`, well before the packet. The schedule tag space is nibble-partitioned (`0x0x` access relations, `0x1x` bounds proofs, `0x2x` scalar programs, `0x3x` reductions), so **the fresh gather bounds-proof tag must be `0x13`, not `0x03`**; `0x03` is `TAG_SCALAR_BROADCAST`. |
| 10 | Sixteen standard realization rows; request subject is v6 | **verified** — rows enumerated in `crates/tiler-ir/src/semantic/registry.rs`; `b"tiler.compiler.request-subject.v6` in `request/subject.rs` |
| 11 | `GatherF32` is `ShapeInferenceParticipation::LiteralOnly` and calls `static_operand_shape` | **verified** |
| 12 | `DTypeNotDispatchable`/`dtype-recognized` live in `crates/tiler-compiler/src/request.rs` | **imprecise** — the semantics hold, but the variant is defined in `request/refusal.rs` and the rule string is raised from `request/graph.rs`, `request/elementwise.rs`, and `request/recognize.rs` |
| - | `INDEX_REGION_IDENTITY_DOMAIN` remains version 11 | **verified in substance, imprecise in name** — the constant is `INDEX_REGION_DOMAIN` in `crates/tiler-ir/src/index/builder.rs`, value `b"tiler.index-region.v11\0"` |
| - | `NormalizedOutput::epilogue()` returns `None` for Gather | **false** — there is no `epilogue()` method on `NormalizedOutput`. It has 5 variants and the accessors `serial_sum`, `try_serial_sum`, `pointwise`, `contraction`, `staged`. `fused_prologue_constants` and `output_subject` are free functions (`physical.rs`, `request/subject.rs`), not methods. |
| - | `RegionSpellingKind` 7 to 8, `NormalizedOutput` 5 to 6 | **verified** as starting counts |
| - | `PendingInvocationIndexValidation` is to be added | **verified absent** |
| - | `LogicalAccess` measures 208 bytes, `Shape` 24, `SourcedShape` 32 | **re-derived at this base and unchanged**: `RUSTFLAGS='-Zprint-type-sizes' cargo +nightly check -p tiler-ir` prints `LogicalAccess: 208 bytes`, `Shape: 24 bytes`, `SourcedShape: 32 bytes`, all alignment 8. `BoundsProofKind` currently measures 72 bytes. |
| - | Governed index-access capability rows | **not stated by the packet; measured here** — `GOVERNED_INDEX_ACCESS_CAPABILITIES = 21` (14 fixed rows plus one per admitted concatenate arity), so a gather row makes 22. |

Two of these are consequential rather than cosmetic. Fact 9d changes an identity assignment the packet fixed: implementing the schedule bounds proof at `0x03` as written would collide with `TAG_SCALAR_BROADCAST`. Fact 9c changes the *reason* `0x0C` is correct without changing the value. Neither re-spells the accepted public surface, so neither is a stop condition; both are recorded so the next worker does not re-derive them.

## Outcome so far - the index layer

Landed on this branch and gated: `make full` green, `cargo nextest run --workspace` 3847 passed, `cargo test --workspace --doc` green, `tkt lint` ok, `git diff --check` clean.

- `AccessData` and `VerifiedAccessData` are now the checked sums `Direct` / `GatherRead`, with a separate `CompactedAccess` staging type that makes the proof-binds-identity ordering unrepresentable to get wrong.
- Public `TensorAccessRef` keeps `id`, `mode`, `domain`, and gains the exhaustive `view()`; `tensor()`, `coordinates()`, `bounds_proof()`, and `write_ownership_proof()` moved to `DirectTensorAccessRef`, and `GatherReadAccessRef` exposes `source`, `index`, `axis`, `source_coordinates`, `index_coordinates`, `bounds_resolution`.
- `IndexRegionBuilder::gather_read` with the accepted refusal precedence and all thirteen `IndexBuildError` variants; whole-region revalidation under `IndexRegionDiagnostic::GatherAccess` with the fifteen-rule `GatherAccessRule`.
- `GatherIndexBoundsProof` / `GatherIndexValidationRequirement`, their opaque identities, `GatherIndexBoundsResolution`, `GatherIndexBoundsProofKind`, and the single verifier-private deriver that mints them. The fact source is derived from the complete subject before the kind short circuit, and the empty-result argument takes precedence over the U32-universe argument.
- Index-region access tag `0x03`; `tiler.index-region.v11` deliberately does **not** step.
- `tiler-reference` evaluates a gather and refuses out-of-range indices through the semantic family's own `decide_gather_index`, under the new `IndexRegionEvaluationError::GatherIndexOutOfBounds`.

New pinned identity domains, sorted into `PINNED_IDENTITY_DOMAINS`: `tiler.gather-index-bounds-proof.v1\0`, `tiler.gather-index-validation-requirement.v1\0`, `tiler.index.access-gather-read.v1\0`, `tiler.index.access-gather-read.alpha.v1\0`.

**No previously encodable region's bytes moved.** Every existing identity pin and golden passes unchanged; the direct arms of every encoder keep their exact former tags, field layouts, and domain literals.

### Subject perturbations, each driven separately with its failure text

| Perturbation | Failure |
|---|---|
| gather access tag reuses the direct write tag `0x02` | `the direct read and write tags are frozen and the gather takes the next free value` / `left: [1, 2, 2]  right: [1, 2, 3]` |
| source and index ordinals swapped in the access frame | `the gather frame is tag, source, index, axis, domain, source coordinates, index coordinates, in exactly that order` |
| the two coordinate runs written in the wrong order | same pin, with the two runs transposed in `left` |
| axis frame dropped | `encode_region` capacity assertion: `left: 141  right: 145` |
| U32 threshold loosened from `>=` to `>` | `a_source_axis_reaching_the_u32_universe_is_proved` fails at its `statically_proved` expectation |
| vacuity decided from the index shape alone | `a domain that visits no point places no obligation on any value, so attributing the conclusion to the source axis would name the wrong premise` / `left: U32RangeContainedBySourceExtent  right: VacuousEmptyResultDomain` |
| empty-result precedence removed | same message, from the precedence control alone |
| fact source ignores coordinate expressions | `a declared symbol participated in the subject even though the argument did not need it` / `left: Program  right: ShapeEnvironment` |
| sourced source boundary admitted when the environment determines it | `each_sourced_boundary_and_domain_extent_is_refused_before_the_domain_shape` fails |
| aliasing check removed | `one_tensor_cannot_play_both_gather_roles` fails |

A negative control is retained for both static arguments: a source axis of exactly `2^32 - 1` is **not** proved, because `u32::MAX` itself is then out of range.

## Remaining work - not landed here

The accepted packet's layers above the index region are untouched, and no scope beyond the declared `implementation/{ir,reference,compiler}` was needed for what landed. These are the named remainder:

- `IndexRealizationLaw::Gather { axis_attribute }` at fresh tag `0x0E`, `gather_f32()`, the seventeenth standard realization row, and the frozen realization-registry identity move it causes.
- `LogicalAccess::GatherSource` at reserved tag `0x0C`, `BoundsProofKind::GatherSource` at **`0x13`** (see Fact 9d - *not* the `0x03` the packet states), `GatherAddressReadRule`, `ScheduledRegionDiagnostic::GatherAddressRead`, and schedule association verification.
- `kernel::lower::addressing`'s exhaustive `LogicalAccess::GatherSource => Err(KernelDiagnostic::BodyRefinement)` arm and the `body-refinement` wall behind it.
- `NormalizedOutput::Gather` / `NormalizedOutputSubject::Gather`, the `gather-f32.v1` output subtag, compiler access-relation tag `0x06`, and every total consumer named in the packet.
- `InvocationGatherIndexValidationRequirement`, the two `InvocationValidationRequired` outcomes, `tiler_compiler::legality::PendingInvocationIndexValidation`, and the `gather-invocation-validation-required` reason.
- The governed lowering capability row (21 to 22) and the ADR 0108 schedule-clause amendment with its catalog and contract sweep.

No runtime receipt was minted and no obligation was treated as discharged. No artifact, manifest, cache, or Metal surface was touched.
