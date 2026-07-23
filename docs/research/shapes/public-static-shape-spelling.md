---
schema: "tiler-doc/v1"
id: "tiler.research.shapes.public-static-shape-spelling"
kind: "research"
title: "Public static-shape evidence spelling"
topics: ["shapes", "rust", "semantics", "api-design"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "superseded"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "executable-model", "bounded-measurement"]
informs: ["tiler.contract.ir"]
depends_on: ["tiler.research.shapes.stable-rust-shape-evidence"]
ticket: "research-the-public-static-shape-evidence-spelling"
---

# Public static-shape evidence spelling

## Question

What public stable-Rust spelling should represent optional exact static-shape
evidence without becoming a second shape authority, fragmenting equivalent
shapes into incompatible types, or committing Tiler to an unbounded type-level
shape algebra?

## Language and compatibility facts

**Fact:** stable Rust const parameters are limited to integer types, `char`, and
`bool`; an extent array or slice cannot be one const argument. Const arguments
used as types or array repeat expressions must also be standalone. General
expressions such as `R - 1` remain under the incomplete, unstable
`generic_const_exprs` feature. See the [Rust Reference on const
generics](https://doc.rust-lang.org/reference/items/generics.html#const-generics)
and the [Unstable Book](https://doc.rust-lang.org/unstable-book/language-features/generic-const-exprs.html).

**Fact:** adding required items to a public trait or changing its item
signatures is a breaking change. Rust's API guidance recommends sealing a trait
when the implementation set is controlled by the defining crate. See Cargo's
[trait-item compatibility rules](https://doc.rust-lang.org/cargo/reference/semver.html#trait-items)
and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed).

**Fact:** a declarative macro may expand in type position, so a future
`static_shape!(2, 3)` convenience is stable-Rust-feasible. Its expansion is
still the public type identity, not merely presentation syntax. See the [macro
invocation grammar](https://doc.rust-lang.org/reference/macros.html#macro-invocation).

## Precedents

**Fact:** dfdx 0.13 represents static dimensions with library-owned `Const<N>`
types and shapes as tuples. Its public `Shape` implementations cover scalar and
tuple families through rank six. This gives structurally canonical types, but
dfdx uses them as the tensor's shape representation rather than optional proof
over a separate graph authority. See dfdx's [shape overview](https://docs.rs/dfdx/0.13.0/dfdx/#shapes--tensors)
and [foreign-type implementations](https://docs.rs/dfdx/0.13.0/dfdx/shapes/trait.Shape.html#implementations-on-foreign-types).

**Fact:** ndarray 0.17.2 deliberately prevents downstream `Dimension`
implementations and supplies fixed ranks zero through six plus a dynamic rank.
See [`Dimension`](https://docs.rs/ndarray/0.17.2/ndarray/trait.Dimension.html).

**Fact:** Burn 0.21 makes rank a tensor const parameter while retaining exact
extents at runtime. See [`Tensor<B, const D: usize,
K>`](https://docs.rs/burn/0.21.0/burn/tensor/struct.Tensor.html). The inspected
local Burn revision was `e5467f02c3cf88eb5d709f190c170005ce26038d`.

**Fact:** nalgebra 0.35 provides a library-owned `Const<N>` dimension, but
type-level arithmetic requires additional typenum bounds and machinery. See
[`Const<N>`](https://docs.rs/nalgebra/0.35.0/nalgebra/base/dimension/struct.Const.html).
Candle instead retains runtime `Vec<usize>` shapes in inspected revision
`31f35b147389700ed2a178ee66a91c3cc25cc80d`.

## Candidate comparison

### Downstream descriptor trait

```rust,ignore
struct Matrix;

impl StaticShapeSpec for Matrix {
    const EXTENTS: &'static [u64] = &[2, 3];
}

type MatrixValue<T> = ShapedValue<T, Exact<Matrix>>;
```

**Point:** this is sound when only the graph can construct `ShapedValue` after
checking the descriptor. It supports arbitrary rank without generating a family
per rank.

**Counterpoint:** it is not canonical. `Exact<crate_a::Matrix>` and
`Exact<crate_b::Matrix>` are different Rust types even when both describe
`[2, 3]`. A shape-preserving API then rejects valid composition or must weaken
and refine again at runtime. The open trait also creates downstream boilerplate
and a permanent semver surface for a capability that grants no authority.

### Library-owned arity families

```rust,ignore
ShapedValue<F32, StaticShape2<2, 3>>
```

**Point:** extents determine one nominal Rust type regardless of which crate
uses it. The vocabulary can remain sealed, uses the canonical `u64` extent
domain, produces compact signatures, and does not imply a recursive shape
algebra.

**Counterpoint:** stable Rust requires one published family per supported
evidence rank. This bounds exact compile-time evidence, but it does not bound
the semantic graph: higher ranks still use `Rank<R>` or `Value<T>`, and adding a
new `StaticShapeN` family is additive.

### Dimension tuples

```rust,ignore
ShapedValue<F32, Exact<(Dim<2>, Dim<3>)>>
```

**Point:** tuples also give canonical structural identity and have mature dfdx
precedent.

**Counterpoint:** they expose more type machinery, produce noisier signatures
and diagnostics, still need a finite set of tuple trait implementations, and
invite mixed static/dynamic type-level shape composition that Tiler's canonical
graph already owns.

### Recursive extent list

**Point:** a cons-list encoding can represent any rank with one recursive
definition.

**Counterpoint:** it commits the public API to typenum-style recursive algebra,
poor diagnostics, and deeper trait evaluation. That complexity is not needed
for optional checked evidence and conflicts with the bounded stable-Rust model
selected by ADR 0061.

## Comparative measurement

**Measurement:** the retained Rust 1.89 harness generated 1, 10, 100, and 1,000
distinct rank-three shapes for each viable spelling. Each result below is the
median of five genuinely isolated target-directory builds on the retained
arm64 macOS 27 host:

| 1,000-shape spelling | Source bytes | Check | Check peak RSS | Release | Release peak RSS | Binary |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Downstream descriptors | 184,693 | 0.22 s | 122.8 MiB | 0.29 s | 137.0 MiB | 404,016 B |
| Owned arity family | 88,011 | 0.23 s | 111.7 MiB | 0.29 s | 130.4 MiB | 404,016 B |
| Dimension tuple | 113,005 | 0.24 s | 113.5 MiB | 0.30 s | 131.6 MiB | 404,016 B |

The owned family used 52% less generated source than open descriptors and
modestly less compiler memory at this bound. Timings and binaries were
materially equivalent. These results reject a scaling concern for the bounded
prototype; they are not a production compile-time guarantee. Full method and
results are in the [spelling measurement](../../../spikes/shapes/shape-evidence/measurements/spelling-summary.json).

## Superseded stable-Rust recommendation

This remains the fallback analysis when stable Rust is mandatory. ADR 0067
instead accepts the follow-up [nightly const-parameter
research](nightly-const-shape-parameters.md) and selects one dependent array
family across arbitrary ranks. The stable arity-family proposal is therefore
superseded, not invalidated as a measurement.

**Inference:** library-owned arity families are the only candidate that
combines canonical cross-crate type identity, sealed authority, compact public
syntax, and a bounded stable-Rust implementation.

**Proposal:** expose the conceptual initial vocabulary as:

```rust,ignore
Value<T>
ShapedValue<T, Rank<R>>
ShapedValue<T, StaticShape2<2, 3>>
```

Publish only the exact-shape arities required by the first governed operation
profile; the current proof exercises ranks zero through three. Higher arities
are additive. Unsupported exact-evidence ranks retain the same semantic graph,
validation, and runtime behavior through `Rank<R>` or `Value<T>`.

Do not initially expose a downstream `StaticShapeSpec`, dimension tuples, a
recursive list, or `static_shape!` macro. The macro is a compatible optional DX
layer if real call sites demonstrate that the nominal family spelling is too
noisy. This proposal does not change the single graph-owned admission path,
checked refinement, evidence-neutral semantic identity, or mandatory canonical
revalidation required by ADR 0061.
