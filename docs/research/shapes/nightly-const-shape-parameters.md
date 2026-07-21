---
schema: "tiler-doc/v1"
id: "tiler.research.shapes.nightly-const-shape-parameters"
kind: "research"
title: "Nightly arbitrary-rank const shape parameters"
topics: ["shapes", "rust", "const-generics", "api-design"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "adopted"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.ir"]
adopted_by: ["ADR-0067"]
supersedes: ["tiler.research.shapes.public-static-shape-spelling"]
ticket: "research-nightly-const-shape-parameters"
---

# Nightly arbitrary-rank const shape parameters

## Question

Are Rust's unstable const-parameter features intended to support the exact
capability Tiler wants: one canonical, value-based Rust type carrying an exact
extent vector of arbitrary rank as optional checked evidence?

This report resolves the feature-premise question. The follow-up spike owns
cross-crate identity, diagnostics, compiler stability, and cost measurements.

## The const-generics roadmap

**Fact:** the original const-generics RFC defines const parameters as values
known at type-checking time and explicitly motivates values as a simpler and
more accurate alternative to marker types. It requires a const parameter's type
to have a compiler-defined equality suitable for type identity. See [RFC
2000](https://rust-lang.github.io/rfcs/2000-const-generics.html).

**Fact:** Rust's accepted 2026 [Full Const Generics project
goal](https://rust-lang.github.io/rust-project-goals/2026/const-generics.html)
calls the current stable subset incomplete. It separates two directions:

1. additional const-parameter types, beginning with structs, enums, tuples, and
   arrays; and
2. additional const arguments, such as associated constants and generic
   expressions.

The project explicitly intends to stabilize ADT const parameters. References,
slices, strings, and const-parameter types that depend on other generics are
named as later areas of exploration, not part of the first stabilization set.

## Relevant features

### `min_adt_const_params`

**Fact:** [`min_adt_const_params`](https://doc.rust-lang.org/nightly/unstable-book/language-features/min-adt-const-params.html)
allows arrays and user-defined structural values as const-parameter types. It
is the narrowed stabilization candidate split from the broader
`adt_const_params`. User-defined fields cannot be more private than their type
in this subset.

**Compatibility with Tiler:** this feature's premise is directly aligned with
representing extents by their array value rather than by a downstream marker
type. By itself, however, `[u64; N]` still has a different Rust type for every
`N`; another feature must allow `N` to parameterize the following const
parameter's type.

### `generic_const_parameter_types`

**Fact:** [`generic_const_parameter_types`](https://doc.rust-lang.org/beta/unstable-book/language-features/generic-const-parameter-types.html)
exists specifically to allow the type of a const parameter to depend on an
earlier generic parameter. The Rust const-generics design document uses the
same form Tiler needs:

```rust,ignore
fn foo<const LEN: usize, const ARR: [u8; LEN]>() -> [u8; LEN];
```

See the project group's [generic const parameter types design
record](https://github.com/rust-lang/project-const-generics/issues/28).

**Fact:** this is still an in-tree experiment without an accepted RFC. Its
[tracking issue](https://github.com/rust-lang/rust/issues/137626) lists variance,
inference-bearing const arguments, an RFC, and stabilization work as open.
Compiler guidance also explains that the older anonymous-const representation
makes dependent const-parameter types difficult and that the newer generic
const-argument machinery may provide part of the eventual solution. See the
[rustc const-generics guide](https://rust.googlesource.com/rust/+/7ee6257470b6bc6fc0cbc43430f7cef63e23f571/src/doc/rustc-dev-guide/src/const-generics.md).

**Compatibility with Tiler:** the premise is exactly aligned. Tiler wants rank
to determine the type of the extent array. Syntax and implementation may
change, but that relationship is not an accidental compiler loophole.

### `unsized_const_params`

**Fact:** [`unsized_const_params`](https://doc.rust-lang.org/unstable-book/language-features/unsized-const-params.html)
permits unsized const-parameter types and currently also gates references such
as `&'static [u64]`. That makes this spelling compile today:

```rust,ignore
pub struct StaticShape<const DIMS: &'static [u64]>;
```

**Fact:** this reference-based spelling is not the favored long-term premise.
The Rust change that split the feature says the intended experiment is to make
direct unsized values work and potentially forbid references in const generics.
Reference const parameters have unresolved pointer-identity and padding
semantics because the compiler lowers their contents into structural value
trees. See [rust-lang/rust#127722](https://github.com/rust-lang/rust/pull/127722)
and the open [reference-identity question](https://github.com/rust-lang/rust/issues/120961).

**Compatibility with Tiler:** arbitrary rank is attractive, but the current
borrowed-slice syntax relies on precisely the behavior Rust may remove. It is a
useful comparison case, not the leading public contract.

### Generic const arguments and `type const`

**Fact:** [`min_generic_const_args`](https://doc.rust-lang.org/nightly/unstable-book/language-features/min-generic-const-args.html)
introduces `type const` items and paths to const values in type positions. The
broader [`generic_const_args`](https://doc.rust-lang.org/nightly/unstable-book/language-features/generic-const-args.html)
extends this using definitional equality.

**Compatibility with Tiler:** these features may later improve aliases,
associated shape constants, or operation-derived evidence. They are not needed
for a literal dependent array and do not determine whether arrays can be const-
parameter types.

## Candidate public forms

### Dependent extent array — leading spike candidate

```rust,ignore
pub struct StaticShape<const RANK: usize, const DIMS: [u64; RANK]>;

type Matrix = ShapedValue<F32, StaticShape<2, { [2, 3] }>>;
```

**Point:** one family supports every rank; equal arrays are structural values;
no allocation or reference identity participates; and the two feature premises
are exactly “arrays as const values” plus “one const parameter determines the
next parameter's type.”

**Counterpoint:** `RANK` is syntactically repeated, and the dependent-parameter
feature is earlier in its design process than ADT const parameters. A future
type-position macro could derive the rank without changing the underlying
canonical type:

```rust,ignore
static_shape![2, 3] // expands to StaticShape<2, { [2, 3] }>
```

### Borrowed slice

```rust,ignore
pub struct StaticShape<const DIMS: &'static [u64]>;
```

**Point:** this is the most compact arbitrary-rank spelling currently accepted
by nightly.

**Counterpoint:** its equality traverses a reference-backed compiler value, and
Rust's own implementation notes preserve the option of forbidding references.
That is a premise-level mismatch, not merely nightly instability.

### Fixed-length array family

```rust,ignore
pub struct StaticShape3<const DIMS: [u64; 3]>;
```

**Point:** it depends only on the nearer-term ADT const-parameter work and is
more compact than one const parameter per extent.

**Counterpoint:** it still publishes one family per rank, so it does not resolve
the motivating limitation.

### Tiler-owned padded structural value

```rust,ignore
pub struct ShapeValue {
    pub rank: u8,
    pub dims: [u64; MAX_EVIDENCE_RANK],
}

pub struct StaticShape<const SHAPE: ShapeValue>;
```

**Point:** one family uses only the nearer-term ADT feature.

**Counterpoint:** it replaces a finite number of public families with a finite
maximum rank. The current minimum feature also requires public fields, allowing
multiple structural encodings for the same logical extents unless every caller
uses a canonical constructor. That undermines the cross-crate type-identity
goal.

## Current compiler observations

**Measurement:** `rustc 1.97.0-nightly (8b03437a8 2026-05-12)` accepted the
borrowed-slice form, but rejected the dependent-array declaration with E0770.

**Measurement:** `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, installed as
`nightly-2026-07-19`, accepted all of the following direct probes:

- `[u64; 3]` under `min_adt_const_params`;
- `&'static [u64]` under `adt_const_params` plus `unsized_const_params`;
- `[u64; RANK]` under `min_adt_const_params` plus
  `generic_const_parameter_types`; and
- a Tiler-owned structural shape value under `min_adt_const_params`.

These observations establish that the accepted language form exists on the
governed compiler, but they are not retained conformance evidence. The spike
ticket still requires reproducible cross-crate fixtures, negative tests,
diagnostics, compiler-pin comparison, and compile-cost measurements before the
workspace adopts the pin and the shaped-value API is implemented.

## Interim assessment

**Inference:** the general Rust const-generics premise is what Tiler wants:
shape extents are values that should participate structurally in optional Rust
type identity, not nominal marker descriptions.

**Inference:** the dependent extent array is better aligned than the borrowed
slice. Its language premises are arrays as const values and generic-dependent
const-parameter types, both explicitly intended capabilities. The borrowed
slice depends on reference behavior the Rust project may forbid.

**Proposal:** make the dependent extent array the primary spike candidate and
retain the slice form only as a comparison. ADR 0067 subsequently accepted this
proposal and an exact nightly policy. The spike now establishes conformance and
upgrade behavior rather than choosing the product contract.

## Research outcome

The premise is compatible with Tiler. Rust intends const parameters to carry
structural compile-time values, intends arrays to become const-parameter types,
and explicitly intends const-parameter types to depend on earlier generics. The
dependent-array form composes those capabilities without inventing a separate
Tiler type algebra.

That conclusion does not establish implementation readiness. The dependent-type
feature remains experimental and lacks an accepted RFC. ADR 0067 accepts that
risk; `spike-nightly-arbitrary-rank-shape-evidence` is the required conformance
and compiler-upgrade harness.
