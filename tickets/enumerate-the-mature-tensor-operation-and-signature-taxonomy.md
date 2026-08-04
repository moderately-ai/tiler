---
id: enumerate-the-mature-tensor-operation-and-signature-taxonomy
title: Enumerate the mature tensor operation and signature taxonomy
status: todo
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
