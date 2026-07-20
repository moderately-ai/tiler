---
schema: "tiler-doc/v1"
id: "tiler.research.indexing.index-access-model"
kind: "research"
title: "Symbolic index and access model"
topics: ["indexing", "access", "storage"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.ir", "tiler.contract.fusion-and-scheduling"]
adopted_by: ["ADR-0046"]
ticket: "index-access-model"
---

# Symbolic index and access model

**Status:** research basis for ADR 0046

**Ticket:** `index-access-model`

## Conclusion

Tiler should not use one expression to mean both “which logical tensor
element?” and “which allocation address?” The first question belongs to the
target-independent index region. The second depends on a selected physical
view, layout contract, storage encoding, and target index representation.

The initial model therefore has two composed maps:

```text
iteration coordinates + shape parameters
    -> TensorAccessMap
    -> logical tensor coordinates
    -> BufferView address derivation
    -> allocation-relative element offset
    -> storage encoding / target lowering
    -> byte or target address
```

This split represents broadcast, permutation, split/merge, and reshape without
claiming that any of them is a zero-copy storage view. A physical planner may
later prove that a particular `BufferView` realizes the same map as an alias;
otherwise it materializes.

## Primary precedents

MLIR's [Affine dialect](https://mlir.llvm.org/docs/Dialects/Affine/) separates
iteration dimensions from symbols, represents maps and integer sets, and
extends affine expressions with floor/ceiling division and modulo by positive
constants. Its semi-affine form additionally permits symbolic coefficients and
divisors. This is useful terminology, but Tiler should not call every dynamic
reshape map affine: multiplication or division by a runtime extent is
semi-affine, and a tensor-value gather is data-dependent.

MLIR [Linalg](https://mlir.llvm.org/docs/Dialects/Linalg/) associates iterator
types and logical indexing maps with structured computations, then lowers them
to loops, vector code, library calls, or target-specific implementations.
This supports keeping the iteration/access relation independent of a chosen
thread mapping.

MLIR [MemRef](https://mlir.llvm.org/docs/Dialects/MemRef/) separately models
strided storage, offsets, subviews, reinterpretation, and metadata extraction.
Its `subview` composition is relative to the source view, while
`reinterpret_cast` is relative to underlying memory. That distinction is a
warning against an unqualified `offset` field. Tiler uses one allocation-
relative element offset convention and makes view composition explicit.

OpenXLA's
[`IndexingMap`](https://github.com/openxla/xla/blob/main/xla/hlo/analysis/indexing_map.h)
combines a symbolic map with dimension, range, and runtime variables, intervals,
and constraints. Its
[`indexing_analysis`](https://github.com/openxla/xla/blob/main/xla/hlo/analysis/indexing_analysis.h)
composes output-to-input maps and models reshape/bitcast by linearization
followed by delinearization. It validates the utility of composable maps plus
domains, while also showing why variable roles and runtime-symbol provenance
must be explicit.

The [Integer Set Library](https://libisl.sourceforge.io/manual.pdf) represents
sets and relations of integer points bounded by affine constraints using exact
integer arithmetic. Tiler does not need to expose ISL or adopt a full
polyhedral optimizer initially, but exact mathematical integers and the
set/map distinction are the right semantic basis.

MLIR's [Index dialect](https://mlir.llvm.org/docs/Dialects/IndexOps/) instead
models a target-dependent pointer-width `index`. Tiler deliberately does not
use that contract for canonical index identity: it would make a semantic map's
folding behavior target-dependent. Width is selected and proved later.

## Proposed records

The notation is structural, not a committed Rust API.

```text
IndexRegion {
  semantic_region,
  domain: IterationDomain,
  scalar_program,
  accesses: [TensorAccess],
  required_semantic_facts,
}

IterationDomain {
  dimensions: [DomainDimension { id, role, extent }],
  constraints: CanonicalPredicateSet,
}

DomainRole = Parallel | Reduction

TensorAccess {
  tensor_value,
  mode: Read | Write | ReductionUpdate(contract),
  map: TensorAccessMap,
}

TensorAccessMap {
  domain_signature,
  parameter_signature,
  result_coordinates: [IndexExpr],
}

BufferView {
  allocation,
  logical_shape,
  allocation_relative_start: ElementOffset,
  element_strides: [ElementStride],
  accessible_element_range,
  access,
}
```

Every domain dimension is half-open: `0 <= d < extent`. Rank and variable
roles are static. A zero extent makes the domain empty. Parameters are scoped,
typed shape or interface values whose provenance comes from `ShapeEnv`; tensor
element data is never an index parameter in the initial model.

`TensorAccessMap` returns one coordinate per logical tensor axis. It does not
contain an allocation, base pointer, byte offset, runtime stride, memory space,
thread ID, vector lane, or target integer type.

For an initial nonnegative strided view, address derivation is:

```text
element_offset = allocation_relative_start
               + sum(logical_coordinate[k] * element_stride[k])
```

The offset is allocation-relative and measured in elements. Storage encoding
performs the later checked element-to-byte or packed-code conversion. A view's
accessible range must cover every derived access; allocation length alone is
not logical tensor length.

## Expression classes

Canonical evaluation uses exact signed mathematical integers. The durable
expression language is bounded and typed:

- constants;
- iteration variables;
- admitted nonnegative shape/interface parameters;
- addition and negation;
- multiplication by a parameter-only expression;
- Euclidean `FloorDiv` and `Mod` by a proven-positive parameter-only
  expression; and
- explicitly versioned extensions added only with verifier and
  canonicalization support.

Iteration-variable-by-iteration-variable multiplication is rejected. A tensor
load is not an `IndexExpr`. `FloorDiv` and `Mod` use Euclidean semantics, so a
positive divisor yields `0 <= x mod divisor < divisor`; host and generated code
must not inherit a language's signed remainder accident.

Each expression is classified after canonicalization:

```text
Affine              constant coefficients, no div/mod
QuasiAffine         constant positive div/mod
SemiAffine          symbolic coefficients or positive symbolic div/mod
DataDependent       tensor-derived coordinate or indirect relation
```

The first slice admits the first three when their divisors, bounds, and
host-evaluable parameters are proven. A pass may support only a subset and
must return `Unknown` or decline when its analysis cannot reason about the
class. `DataDependent` accesses require a later gather/scatter/indirection
contract; they are not smuggled through an opaque symbol.

## Canonicalization

Canonical identity includes variable roles and binding identities, not source
spellings. Canonicalization:

- recursively substitutes composed maps and then normalizes;
- flattens associative addition, combines exact constant terms and like
  coefficients, removes zero terms, and orders terms structurally;
- normalizes negation and parameter-only products without applying a rewrite
  that needs an unproved sign, nonzero, divisibility, or no-overflow fact;
- requires positive divisors and uses one Euclidean div/mod spelling;
- folds constants with exact integers;
- removes unused parameters and renumbers local iteration variables by
  canonical domain order;
- canonicalizes predicates as a sorted duplicate-free set; and
- retains proof provenance separately from the normalized expression.

Canonicalization never evaluates in `u32`, `u64`, `usize`, or a target's
pointer width. For example, canceling `(A * B) / B` requires `B > 0`; it is not
just a wrapping-integer peephole.

## Worked maps

For output domain `[B, H, W, C]`:

```text
identity:       (b, h, w, c) -> (b, h, w, c)
permutation:    (b, h, w, c) -> (b, c, h, w)
broadcast read: (b, h, w, c) -> (c)
```

Broadcast is intentionally many-to-one for a read. The same map is illegal for
an ordinary write because multiple iterations would own one output element.

For a reshape from logical `[2, 3]` to `[3, 2]`, an output coordinate `(i, j)`
first linearizes in the output logical order and then delinearizes in the input
logical order:

```text
linear = i * 2 + j
input  = (linear floordiv 3, linear mod 3)
```

With dynamic extents, coefficients and divisors are parameters and the map is
semi-affine. Equal element count and positive divisors are semantic facts. This
logical map says nothing about whether the source's physical strides allow a
view-only implementation.

For a noncontiguous physical input view:

```text
shape   = [2, 3]
start   = 2 elements
strides = [5, 2] elements
offset(i, j) = 2 + i*5 + j*2
```

The logical access remains `(i, j)`. The physical view verifier proves the
reachable offsets are within its accessible allocation range. The planner can
compare this general-stride realization with a materialized contiguous one.

## Predicates and guards

Four predicate classes must not be conflated:

1. **Semantic constraints** make the tensor operation meaningful, such as
   equal reshape element counts. Failure is a semantic error.
2. **Index-domain constraints** restrict the mathematical points in an
   `IndexRegion`. They are part of the map/domain truth, not a plan fallback.
3. **Variant guards** require runtime layout, alignment, or narrowed-width
   facts. Failure selects another semantically equivalent physical plan.
4. **Per-point schedule predicates** mask tails or boundary lanes after tiling.
   They are verified against coverage and write ownership in scheduled IR.

The initial `TensorAccessMap` is total over its declared domain. Finite
piecewise/guarded maps may be added as an explicit ordered-or-disjoint case set,
but only with a verifier proving cases cover the domain and overlap consistently.
Tails are not represented by weakening logical totality: a schedule maps an
over-approximated launch point to a logical point and predicates the load/store.

## Aliasing and write ownership

Logical aliasing and allocation aliasing are different facts.

- Read maps may be many-to-one; broadcast is the standard example.
- An ordinary output write map must assign each required output coordinate one
  writer. Injectivity alone is insufficient unless coverage is also proven.
- Reduction updates and atomics use separate contracts naming the combine
  semantics, order permissions, visibility, and synchronization.
- Initial input views may share an allocation or overlap. They are read-only
  unless a later effect/alias contract says otherwise.
- Initial outputs and temporaries use fresh nonaliasing allocations. Therefore
  the index-region verifier need not perform dependence analysis for in-place
  writes.
- A view-only result is a physical program alternative. It requires a proof
  that the proposed output view implements the semantic coordinate relation,
  stays in range, and obeys the runtime's return/alias contract.

Negative strides are representable mathematically but deferred in the initial
ABI/profile. Supporting them requires signed stride and reachable-range
analysis, a precise allocation-relative start convention, runtime adapter
support, and target lowering tests. Zero strides are permitted only where the
view/access contract allows read aliasing; they do not authorize overlapping
ordinary writes.

## Index width

Logical extents remain portable `u64` values under the shape contract, but
canonical index expressions use exact mathematical integers. Physical lowering
chooses widths independently for:

- iteration and logical-coordinate arithmetic;
- allocation-relative element offsets; and
- byte/packed-storage address arithmetic.

A generic correctness path uses a target-supported wide representation and
checked conversions. A guarded `u32` fast path is legal only when proof or a
host pre-dispatch guard covers every relevant input and intermediate under the
emitter's fixed evaluation order, including:

- domain extents and maximum iteration coordinates;
- linearization/delinearization intermediates;
- strides and allocation-relative starts where computed in `u32`;
- maximum reachable element offset;
- element-to-byte or packed-code calculations where computed in `u32`; and
- dispatch/grid arithmetic that shares the narrowed representation.

“Every extent fits `u32`” is insufficient: `i * stride` may overflow even when
both values fit. Coordinate arithmetic may be `u32` while address arithmetic
remains `u64`; the manifest and verifier record each role rather than one
ambiguous index-width flag. A failed narrowing guard selects the wide plan.
Overflow or an unsupported wide path is a hard failure, never wrapping behavior.

## Verifier boundaries

### Index-region verifier

- domain extents and parameter sources are typed, scoped, nonnegative, and
  host-evaluable where required;
- expression graphs are acyclic, bounded, and use admitted operators;
- divisors are proven positive;
- map domain arity and parameter bindings are exact;
- each access result rank matches the logical tensor rank;
- every access coordinate is proven in bounds over the complete domain;
- scalar loads/stores reference declared accesses and compatible value types;
- ordinary writes prove exact output coverage and unique ownership;
- read aliasing, reduction updates, and atomics are classified explicitly; and
- all required semantic facts are proved or retained as semantic obligations,
  never converted into variant guards.

### Physical view/address verifier

- rank, shape, offset, stride units, access mode, and allocation provenance
  agree with the binding contract;
- checked min/max reachable element ranges fit the accessible view/allocation;
- storage encoding converts element coordinates to bytes/bits without overflow;
- selected alignment and layout requirements are proved or guarded;
- narrowed coordinate, element-offset, byte-offset, and dispatch widths each
  satisfy their own proof/guard; and
- alias/view implementations refine the semantic map and program alias policy.

### Schedule/kernel verifier

- schedule coordinates cover the logical domain;
- over-approximated launch points are predicated before access;
- vector lanes and tails do not introduce missing or duplicate forbidden writes;
- derived address arithmetic matches the selected `TensorAccessMap` and
  `BufferView`; and
- target lowering preserves Euclidean division/modulo and checked-width facts.

## Counterexamples beyond the initial model

- `input[index_tensor[i]]` is a data-dependent gather. Bounds depend on tensor
  values and need an indirect-access plus validation contract.
- scatter writes require collision, atomic/reduction, determinism, and effect
  semantics; a function-valued coordinate map alone is insufficient.
- boolean-mask selection and `nonzero` have data-dependent output cardinality,
  crossing the host-evaluable shape boundary.
- sparse/compressed formats require metadata-driven iteration and access, not a
  dense strided `BufferView` fiction.
- pointer chasing, hash-table lookup, and paged/external storage require
  resource and failure models outside pure index arithmetic.
- a finite piecewise pad/clamp map is not one affine map; it needs explicit case
  coverage and boundary-value semantics.

These cases are future extensions, not reasons to make the first map language
an unrestricted scalar program.

## Required tests

- identity, permutation, broadcast, split/merge, static and dynamic reshape
  composition, including zero extents and rank-zero tensors;
- contiguous and noncontiguous positive-stride views with nonzero starts;
- read alias acceptance and overlapping ordinary-write rejection;
- exact write coverage, missing points, duplicate writers, reduction exception;
- divisor zero/negative, out-of-bounds maps, free parameters, rank mismatch,
  expression/depth budgets, and canonical-equivalent construction histories;
- tail points at width minus/equal/plus one and proof that inactive lanes never
  access memory;
- `u32` boundaries for every intermediate, including an extent-fitting but
  stride-product-overflowing counterexample, with successful wide fallback;
- element-to-byte and packed-storage overflow;
- negative-stride rejection in the initial profile; and
- gather/scatter/data-dependent examples rejected with a typed unsupported
  classification rather than mislabeled affine maps.

## Measurements and open extensions

The initial implementation should measure canonicalization growth under long
reshape/reindex chains and cap expression nodes, terms, predicate count, and
composition depth. A later experiment can compare the bounded native prover
with MLIR Presburger or ISL for implication and injectivity. That substitution
must not change expression semantics or canonical identity.

Non-obvious choices intentionally left for later are the first finite
piecewise-map surface, negative-stride ABI support, and the representation of
indirect gather/scatter relations. None requires changing the two-map boundary.
