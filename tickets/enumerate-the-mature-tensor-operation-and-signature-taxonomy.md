---
id: enumerate-the-mature-tensor-operation-and-signature-taxonomy
title: Enumerate the mature tensor operation and signature taxonomy
status: done
priority: p0
dependencies: []
related: [own-operation-family-support-matrix, enumerate-the-mature-tensor-dtype-taxonomy, numerical-policy-contract]
scopes: [research/semantic-graph, research/numerics, contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, signatures, taxonomy, foundation]
---
## User-visible outcome

Tiler has a durable end-state inventory of tensor operation semantics and exact
typed signatures, comparable in scope and honesty to the mature dtype taxonomy. It
states what the project intends eventually to express without claiming that any row
is implemented.

## Research boundary

Start from mature tensor/compiler authorities and primary source APIs, then enumerate
semantic families broadly enough to expose type, shape, numerical, identity, and
lowering consequences. At minimum cover constants and data movement; unary, binary,
ternary, comparison, logical, bitwise, cast and quantize/dequantize; reductions and
scans; contraction/matmul/einsum; reshape/view/transpose/broadcast/slice/pad/gather/
scatter/concat; sorting and selection; indexing updates; random/stateful boundaries;
signal/image-domain candidates; and compound or multi-result operations.

For every family record:

- atomic semantic operation versus composition from smaller governed operations;
- ordered operand and result arity, including multi-result forms;
- admissible exact dtype signatures and promotion/conversion policy;
- rank/shape constraints and strongly typed attributes;
- numerical and exceptional-value contract;
- purity/effects and determinism classification;
- minimum correct reference realization and minimum physical fallback;
- extension/opaque-call interaction and explicit unsupported cases.

Separate **Fact**, **Inference**, **Proposal**, and **Measurement**. Do not derive the
taxonomy solely from PyTorch, Candle, Metal, or a language-model workload. The
existing operation-family support matrix is a current maturity ledger, not evidence
that this end-state enumeration is complete.

## Closes when

Every named family has a stable semantic/signature classification or an explicit
research question with a closure test; cross-products that are intentionally invalid
are distinguishable from merely unsupported combinations; and the document can be
mechanically consumed by the delivery-graph ticket without guessing missing rows.

## Outcome

The record is [`docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md`](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md), `id: tiler.research.semantic-graph.mature-operation-and-signature-taxonomy`, `disposition: pending`, evidence class `primary-source-synthesis` only — the pass took no measurement and says so.

It enumerates **47 families in ten groups**, each carrying all eight dimensions the research boundary above names. **38 families carry a stable classification and 9 are blocked on one of 12 research questions (`RQ-OP-01` … `RQ-OP-12`)**, each with a stated closure test written to be falsifiable rather than aspirational. Twelve intentionally-invalid cross-products are enumerated with the reason each is incoherent rather than early, against a three-class table — intentionally invalid / merely unsupported / `Unknown` — that reuses the corpus's existing `Unknown ≠ unsupported` rule. A join table maps every family onto its [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) row; **25 of 47 families have no matrix row**, which is the record's main practical output and the number the delivery-graph ticket should be sized against. **Correction — 2026-08-10.** An earlier Outcome draft said twenty-three; the taxonomy's 2026-08-05 correction (via [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md)) moved the count to twenty-five for two independent reasons: twenty-three was the with-row count transposed (the join table's no-row cell already listed twenty-four), and F-43 was mis-mapped onto the effectful matrix row and belongs in the no-row set. Rungs are deliberately absent from the record: the matrix stays the sole maturity ledger.

Authorities read at preserved local revisions: `stablehlo-spec-v1.18.0`, `onnx-operators-v1.22.0`, `onnx-ir-v1.22.0`, `tosa-spec-1.0.1`, plus the MLIR, PyTorch, JAX, NumPy, and Arrow records already preserved. **Correction — 2026-08-10.** The close-time Outcome stated that Array API 2025.12 and MLIR Linalg were metadata-only / unpinned. Both are now preserved under the [primary-source record](../docs/research/numerics/sources/README.md): `array-api-2025.12` as fifteen ids at `data-apis/array-api` tag `2025.12` (commit `d016d578040d151707a5b7dd2ba1e55f48a8d511`), and seven MLIR Linalg-related pins at `llvmorg-22.1.8` (commit `ca7933e47d3a3451d81e72ac174dcb5aa28b59d1`). The taxonomy re-checked its Array API and Linalg citations against those preserved copies after the original pass; the authorities boundary is no longer the pre-preservation gap.

## Write restriction observed on this dispatch

This worker was restricted to `docs/research/semantic-graph/**` and `tickets/**` — tighter than the ticket's declared scopes, which also include `research/numerics`, `contracts/foundation`, and `contracts/navigation`. The reason is parallel safety: a sibling p0 worker held `contracts/navigation` and `research/numerics` during the same wave, and an edit inside a scope another live ticket holds is admissible only against a verified file-level disjointness check that this dispatch did not perform.

**Correction — 2026-08-10.** Three consequences of that write restriction were handed to the integrator as ready-to-paste lines and are **already applied** at this base (not live unfinished work):

- the research-catalog entry in `docs/research/README.md` under Foundation, semantics, and extensions — "Mature tensor operation and signature taxonomy" (scope `contracts/navigation`);
- the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) five-axes paragraph ("Five axes are cross-referenced rather than duplicated"), which names the operation taxonomy as owning the end-state operation universe (scope `contracts/navigation`);
- the preservation-record entries for the Python array API standard 2025.12 and the MLIR Linalg pins under `docs/research/numerics/sources/` (scope `research/numerics`) — see Outcome authorities correction above.

The record also documents two adjacent research files that are materially stale on families the roadmap has since advanced, and does not correct them, because both sit in `research/shapes`.
