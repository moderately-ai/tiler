---
schema: "tiler-doc/v1"
id: "ADR-0046"
kind: "decision"
title: "Separate logical tensor access from storage addressing"
topics: ["indexing", "storage", "ir"]
decision_status: "accepted"
implementation_status: "spike-only"
applies_to: ["tiler.contract.ir", "tiler.contract.fusion-and-scheduling"]
evidence: ["tiler.research.indexing.index-access-model"]
ticket: "index-access-model"
---

# 0046: Separate logical tensor access from storage addressing

**Status:** accepted

## Context

Fusion and reindex composition need to answer which logical input element is
used for each iteration point. Runtime adapters and kernels separately need to
compute where that element resides in an allocation. Combining those questions
would put strides, offsets, storage encodings, target widths, and alias choices
into the canonical tensor indexing relation.

Affine compiler precedents separate iteration/index maps from memory-layout
descriptors. Dynamic reshape also demonstrates that “affine” alone is too
narrow: its logical map may require multiplication, floor division, or modulo
by symbolic extents even though it remains independent of tensor data.

## Decision

An `IndexRegion` owns a bounded exact-integer iteration domain and
`TensorAccessMap`s from iteration coordinates plus admitted host-evaluable
parameters to logical tensor coordinates. It does not own allocation pointers,
runtime strides, byte offsets, memory spaces, thread coordinates, or a target
integer width.

A selected physical implementation separately composes each logical access
with a verified `BufferView`. The view has one allocation-relative element
start, logical shape, element strides, accessible range, and access mode.
Storage encoding and target lowering perform later checked element-to-byte or
packed-address conversion.

The initial expression vocabulary admits affine, constant-divisor
quasi-affine, and guarded semi-affine expressions with symbolic coefficients
or proven-positive symbolic divisors. It rejects iteration-by-iteration
multiplication and tensor-data-derived indices. Expressions have exact signed
mathematical-integer semantics; target-width evaluation is a physical lowering
decision.

Semantic constraints, index-domain predicates, physical variant guards, and
per-point schedule predicates are distinct. Access maps are total over their
declared domain. Tail masks belong to scheduled IR. A future finite piecewise
map requires explicit case coverage and consistency verification.

Read maps may be many-to-one. Ordinary writes require exact output coverage and
unique ownership. Reductions and atomics use explicit contracts. Initial input
views may overlap, while output and temporary allocations do not alias inputs
or each other. A no-kernel view result is a physical alternative requiring a
proof that its view realizes the semantic coordinate relation.

Logical extents retain the portable `u64` shape domain, while canonical access
arithmetic is width-independent. Physical plans separately choose and verify
coordinate, element-offset, byte/packed-offset, and dispatch widths. A `u32`
fast path needs proof or pre-dispatch guards for every relevant intermediate
under the emitted evaluation order, and it retains a target-supported wide
correctness alternative. Extents fitting `u32` alone are not sufficient.

Negative strides are deferred from the initial ABI/profile rather than made
semantically impossible. They require signed reachable-range analysis and
runtime/backend support. Data-dependent gather, scatter, sparse iteration, and
data-dependent cardinality require later explicit IR contracts.

## Consequences

- Broadcast, permutation, and reshape compose without implying storage copies
  or zero-copy aliases.
- General positive-stride inputs and contiguous fast paths share one logical
  index region.
- Layout specialization, materialization, and alias views remain costed
  physical alternatives.
- Target-width narrowing cannot change canonical map meaning or silently wrap.
- Passes may conservatively decline semi-affine maps they cannot analyze.
- Indirect operations remain addable without weakening the verifier for the
  initial direct-access language.

## Alternatives considered

One map directly to byte addresses makes semantic identity depend on runtime
layout and storage encoding. Restricting all maps to constant-coefficient
affine expressions excludes ordinary dynamic reshape composition. Using an
unrestricted scalar expression language admits data-dependent and nonlinear
cases that the verifier cannot bound. Making canonical indices target-width
integers lets overflow and target choice alter semantic transformations.
