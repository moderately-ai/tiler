---
id: derive-dtype-family-research-tracks-from-the-mature-taxonomy
title: Derive dtype-family research tracks from the mature taxonomy
status: review
priority: p1
dependencies: [enumerate-the-mature-tensor-dtype-taxonomy, own-the-dtype-support-maturity-matrix]
related: [own-operation-family-support-matrix, declare-the-bf16-rows-on-the-authoritative-metal-profile]
scopes: [research/numerics, contracts/navigation, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, roadmap, ticket-graph]
claimed_from: todo
assignee: agent-dtype-tracks
lease_expires_at: 1785880009
---
## User-visible outcome

Every dtype family in the mature taxonomy has a bounded research owner and a route
to explicit support or explicit deferral; no family disappears merely because no
current workload produces it.

Read the taxonomy and support ledger row by row. Partition work by genuinely shared
representation and numerical obligations: booleans; signed and unsigned integers;
IEEE and reduced-precision floats; FP8/FP6/FP4 families; complex; quantized compound
values and scales/zero-points; and any opaque/extension carriers. For each partition
record semantic identity, host/reference carrier, conversion behavior, exceptional
values, constant encoding, artifact ABI, scalar/KIR support, backend capability,
and conformance requirements.

Create research/design/spike tickets for every missing family. Reuse existing BF16,
quantization, and numerical-policy nodes where they are exact owners. Implementation
remains signature-driven: a family track does not claim every operation supports the
dtype. Deferred tracks require measurable activation triggers.

## Closes when

Every dtype taxonomy row maps to an existing exact owner or a newly filed bounded
track; dependency order preserves numerical/reference authority before optimizer and
backend claims; and the support ledger links to those owners without overstating
implemented maturity.

## Outcome (2026-08-04)

[Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md) partitions the [mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md)'s catalog into **fifteen tracks**, derives the partition from shared representation and numerical obligations rather than from topic, records the nine obligations the ticket names per track, joins those nine to the [dtype support ledger](../docs/dtype-support.md)'s thirteen rungs so the two vocabularies stay comparable, and closes with a coverage table mapping every catalog row to a track. Three tracks reuse exact owners — F32 delivered, the live BF16 track, the live affine-quantization track — and twelve are newly filed.

**Twelve track tickets, all `deferred` because no trigger has fired**, each carrying an activation trigger written as a checkable state of the corpus: [`scope-the-predicate-tensor-vertical`](scope-the-predicate-tensor-vertical.md), [`define-the-integer-numerical-contract-and-honourability-subject`](define-the-integer-numerical-contract-and-honourability-subject.md), [`state-the-non-enumerable-float-conformance-profile`](state-the-non-enumerable-float-conformance-profile.md), [`scope-the-ocp-reduced-precision-float-vertical`](scope-the-ocp-reduced-precision-float-vertical.md), [`scope-the-complex-arithmetic-vertical`](scope-the-complex-arithmetic-vertical.md), [`scope-the-ieee-decimal-vertical`](scope-the-ieee-decimal-vertical.md), [`scope-the-block-scaled-compound-value-vertical`](scope-the-block-scaled-compound-value-vertical.md), [`generalize-the-sub-byte-storage-encoding-contract`](generalize-the-sub-byte-storage-encoding-contract.md), [`place-execution-only-numeric-formats-in-the-physical-layers`](place-execution-only-numeric-formats-in-the-physical-layers.md), [`govern-external-dtype-namespace-registration-and-equivalence`](govern-external-dtype-namespace-registration-and-equivalence.md), [`route-the-reserved-numeric-families-through-the-extension-boundary`](route-the-reserved-numeric-families-through-the-extension-boundary.md), and [`scope-the-nonnumeric-tensor-element-domain-vertical`](scope-the-nonnumeric-tensor-element-domain-vertical.md). Filing them `deferred` rather than `todo` is the point: the board must not offer a track whose trigger has not fired.

**Three dependency edges preserve numerical and reference authority before optimizer or backend claims.** The block-scaled track depends on its constituent element track and on [`implement-workload-selected-quantized-parameter-maps`](implement-workload-selected-quantized-parameter-maps.md), because a non-per-tensor map is a prerequisite for MX rather than a substitute for it. The reserved-numeric track depends on external namespace governance, because the extension boundary is the only route its members have. The non-enumerable float track depends on [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md), because deriving from an unconformed pattern would propagate whatever it gets wrong.

**Five of the twelve deferrals rest on a recorded elimination rather than on silence**: the predicate track's trigger is unmet because the first attention vertical binds a host-built *additive* `f32` causal mask; the packing track's because the first quantized profile selected *unpacked* `StorageScalar::U8`; the block-scaled track's because per-block and per-group maps were eliminated on legality with their reopening condition stated; the integer track's because the ledger excludes quantized codes by name; and the decimal track's because weak GPU adoption is a taxonomy finding.

**Ledger links, no cell moved.** Every `Trigger` paragraph in [the dtype support ledger](../docs/dtype-support.md) now names its track's owner, its `## Graph policy` states that a `deferred` owner creates no dispatchable work and no implicit authorization, and the sparse/ragged section points at the off-axis routing table instead of implying a missing dtype reservation. No maturity cell, no evidence claim, and no rung was changed.

**No stop condition fired.** The taxonomy and the ledger were compared row by row and do not contradict each other in a way that changes which tracks exist. Two mappings are non-obvious and are now stated in both documents: five FP8 spellings the taxonomy catalogs (`f8E3M4`, `f8E4M3`, `f8E4M3FNUZ`, `f8E5M2FNUZ`, `f8E4M3B11FNUZ`) are external owner-namespaced candidates whose ledger row is *External or vendor formats* rather than the OCP row — checkable against the ledger's own asserted catalog size of 27 nominal scalars — and bit-packed storage is a ledger *column* rather than a row, which is why the packing track is cross-cutting.

**Stale assertions corrected in scope.** Three `research/numerics` records still said both OCP specifications were `pending-acquisition` after a failed retrieval; both became `metadata-only` on 2026-07-31 with reviewed licences and recorded digests, and `verify-sources.sh` reports `0 pending-acquisition` over 46 records. `mature-dtype-taxonomy.md` (two spans), `quantized-value-and-transform-contract.md`, and `dtype-identity-admission-policy.md` are corrected here; [`correct-the-ocp-source-status-in-adrs-0036-and-0038`](correct-the-ocp-source-status-in-adrs-0036-and-0038.md) carries the two ADR spans, which are outside this ticket's scopes.

**One further drift filed rather than absorbed.** `docs/roadmap.md`'s reduced-precision float row still names [`admit-a-bf16-scalar-arithmetic-subject`](admit-a-bf16-scalar-arithmetic-subject.md) as an open R5/R6 gate; that ticket is `done`, as is [`declare-the-bf16-rows-on-the-authoritative-metal-profile`](declare-the-bf16-rows-on-the-authoritative-metal-profile.md). Correcting the row requires reading the construction sites to decide whether a rung moved, which is a different assertion from the gate list, so it is [`refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings`](refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings.md)'s.

**Operation-axis cross-links, not restatements.** Five of the operation taxonomy's twelve `RQ-OP` questions join a dtype track — `RQ-OP-03` with the predicate track (the two must close together and neither alone), `RQ-OP-01` with the integer track, `RQ-OP-04` with the remaining IEEE float track, `RQ-OP-12` with the complex track, and `RQ-OP-02` with the packing track. The other seven turn on structure, arity, or region support and are independent of dtype.
