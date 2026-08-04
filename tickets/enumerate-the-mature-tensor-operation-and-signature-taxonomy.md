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

It enumerates **47 families in ten groups**, each carrying all eight dimensions the research boundary above names. **38 families carry a stable classification and 9 are blocked on one of 12 research questions (`RQ-OP-01` … `RQ-OP-12`)**, each with a stated closure test written to be falsifiable rather than aspirational. Twelve intentionally-invalid cross-products are enumerated with the reason each is incoherent rather than early, against a three-class table — intentionally invalid / merely unsupported / `Unknown` — that reuses the corpus's existing `Unknown ≠ unsupported` rule. A join table maps every family onto its [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) row; **23 of 47 families have no matrix row**, which is the record's main practical output and the number the delivery-graph ticket should be sized against. Rungs are deliberately absent from the record: the matrix stays the sole maturity ledger.

Authorities read at preserved local revisions: `stablehlo-spec-v1.18.0`, `onnx-operators-v1.22.0`, `onnx-ir-v1.22.0`, `tosa-spec-1.0.1`, plus the MLIR, PyTorch, JAX, NumPy, and Arrow records already preserved. Two sources are metadata-only and the record states that boundary rather than implying a pinned copy: the Python array API standard 2025.12, and the MLIR Linalg dialect page, for which the preservation record pins no Linalg source at all.

## Write restriction observed on this dispatch

This worker was restricted to `docs/research/semantic-graph/**` and `tickets/**` — tighter than the ticket's declared scopes, which also include `research/numerics`, `contracts/foundation`, and `contracts/navigation`. The reason is parallel safety: a sibling p0 worker held `contracts/navigation` and `research/numerics` during the same wave, and an edit inside a scope another live ticket holds is admissible only against a verified file-level disjointness check that this dispatch did not perform.

Three consequences are therefore *not* applied here and are handed to the integrator as ready-to-paste lines in the dispatch report:

- the research-catalog entry in `docs/research/README.md` (scope `contracts/navigation`);
- the cross-reference from the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s "three axes are cross-referenced rather than duplicated" paragraph, which currently names the dtype taxonomy as the only owned universe and now has an operation-axis peer (scope `contracts/navigation`);
- the preservation-record entry for the Python array API standard 2025.12 (scope `research/numerics`), which is a genuine gap rather than bookkeeping — the record cites the standard and no local copy exists.

The record also documents two adjacent research files that are materially stale on families the roadmap has since advanced, and does not correct them, because both sit in `research/shapes`.
