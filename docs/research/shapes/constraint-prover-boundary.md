# Shape constraint prover boundary

**Status:** research finding with an accepted prototype scope
**Ticket:** `shape-environment-contract`  
**Question:** is shape-constraint reasoning realistically bounded, and what
would an intentionally incomplete initial prover omit?

## Executive finding

A closed expression vocabulary does not create a finite or uniformly easy set
of proof obligations. A finite collection of expression constructors can form
arbitrarily large expressions over arbitrarily many symbols, just as a finite
relational algebra can form arbitrarily large query plans.

There is nevertheless a useful boundary:

- affine/Presburger integer constraints form a decidable core;
- products or division between two symbolic values cross into nonlinear
  integer arithmetic;
- tensor programs need nonlinear expressions for common cases such as dynamic
  element counts, reshape compatibility, and grouped dimensions;
- every admitted expression can still be evaluated exactly for concrete host
  bindings even when the compiler cannot prove a proposition about all
  bindings.

The practical answer is therefore not "solve nothing difficult" or "adopt a
general theorem prover." It is to define a strong decidable lane, add targeted
tensor-specific nonlinear rules, and preserve `Unknown` as a safe result.

## Facts: the mathematical boundary

### Affine and Presburger core

MLIR defines affine expressions over integer dimensions and symbols using
addition, subtraction, multiplication by integer constants, and floor/ceiling
division or modulo by positive integer constants. It explicitly excludes
symbol-by-symbol multiplication, powers, and division or modulo by a symbolic
value. Its Presburger implementation represents affine integer equalities and
inequalities and supports unions, intersections, difference, complement,
sampling, and exact subset checks.

This supports a substantial complete reasoning fragment:

```text
M + 2*N <= 4096
N % 8 == 0
ceil_div(N, 32) <= 64
select(N < 128, N, 128) <= 128  // finite piecewise relation
```

`min`, `max`, and `select` can remain within this fragment when their
predicates and branches are affine: they expand to a finite union of affine
cases. Case growth still requires an explicit resource policy.

Sources:

- [MLIR Affine dialect](https://mlir.llvm.org/docs/Dialects/Affine/)
- [MLIR Presburger relations](https://mlir.llvm.org/doxygen/classmlir_1_1presburger_1_1PresburgerRelation.html)
- [MLIR integer relations](https://mlir.llvm.org/doxygen/classmlir_1_1presburger_1_1IntegerRelation.html)

### Nonlinear tensor constraints

Multiplying two symbolic integer expressions creates nonlinear integer
arithmetic. Z3's official arithmetic guide states that this theory is
undecidable in general: no solver can both terminate and correctly decide
every formula. Division or modulo by a symbolic divisor introduces the same
problem because quotient/remainder semantics relate symbolic values through a
product.

Common tensor examples cross this boundary:

```text
numel([M, N]) = M * N
reshape valid iff M * N == P * Q
C == groups * channels_per_group
floor_div(N, runtime_stride)
```

This does not make all such cases hard. Canonical polynomial ordering proves
`M*N == N*M`; positivity permits some safe cancellation; interval arithmetic
can bound many products. These are useful islands, not a complete nonlinear
decision procedure.

Source: [Z3 arithmetic guide](https://microsoft.github.io/z3guide/docs/theories/Arithmetic/).

### Accepted arithmetic boundary

Tiler has chosen mathematical-integer semantics for semantic `ShapeExpr`
intermediates and checked conversion to a final `Extent(u64)`. Logical
rewrites therefore do not observe overflow from an arbitrary compiler integer
width. The evaluator never wraps or saturates and rejects an unrepresentable
final extent before allocation or device work.

Physical and ABI domains may use explicit fixed-width arithmetic. If every
value in one of those domains is modeled as a fixed-width bit vector, its state
space is finite and its formulas are theoretically decidable. Exact bit-vector
multiplication and division can be encoded by SMT solvers, but the enormous
state space still does not make exhaustive solving a realistic strategy.
Semantic proving likewise retains deterministic resource budgets and
`Unknown(ResourceLimit)`.

Sources:

- [Z3 bit-vector theory](https://microsoft.github.io/z3guide/docs/theories/Bitvectors/)
- [MLIR integer-relation implementation](https://mlir.llvm.org/doxygen/classmlir_1_1presburger_1_1IntegerRelation.html)

## Facts: tensor-system precedent

- JAX export performs useful but incomplete symbolic reasoning. It documents
  `InconclusiveDimensionOperation`, cases such as `b >= b % 3` that it cannot
  currently prove, and runtime checks that concrete arguments satisfy symbolic
  specifications. It also documents an intentionally total but unsound
  equality behavior used for practical object hashing; Tiler's typed
  `Proved`/`Disproved`/`Unknown` result must not copy that shortcut into proof
  semantics.
  [JAX shape polymorphism](https://docs.jax.dev/en/latest/export/shape_poly.html)
- PyTorch `ShapeEnv` propagates symbolic expressions and records guards for
  assumptions made while tracing or optimizing. Export may also retain
  runtime assertions for shape relationships.
  [PyTorch dynamic-shape concepts](https://docs.pytorch.org/docs/stable/user_guide/torch_compiler/compile/dynamic_shapes_core_concepts.html),
  [PyTorch export API](https://docs.pytorch.org/docs/main/user_guide/torch_compiler/export/api_reference.html)
- MLIR Shape represents constraints as witnesses. Unresolved constraints can
  lower to runtime assertions, while dependent IR consumes the witness.
  [MLIR Shape dialect](https://mlir.llvm.org/docs/Dialects/ShapeDialect/)
- StableHLO verifies constraints exposed by static information and provides an
  explicit shape-assertion mechanism for dynamic cases. Tiler should preserve
  typed errors rather than adopt StableHLO's general undefined-behavior default
  for unresolved dynamic mismatch.
  [StableHLO dynamism](https://openxla.org/stablehlo/dynamism),
  [StableHLO shape assertion pass](https://openxla.org/stablehlo/generated/stablehlo_passes)
- ONNX states directly that shape inference is not guaranteed to be complete.
  [ONNX shape inference](https://onnx.ai/onnx/repo-docs/ShapeInference.html)

These systems treat missing proof as an expected compiler state, not evidence
that a condition is false.

## Proof outcomes

For known facts `F` and proposition `P`:

```text
Proved     means F entails P
Disproved means F entails not-P
Unknown    means neither entailment was established
```

`Unknown` needs structured reasons because two important cases differ:

```text
InsufficientFacts:
  query M == N with no relationship between M and N
  // both outcomes are possible for valid runtime bindings

UnsupportedFragment or ResourceLimit:
  query M >= M % 3 with M >= 0
  // true, but the current prover or its budget did not establish it
```

Finding one counterexample to `P` does not establish `Disproved`; `Disproved`
requires proving `not-P` for every binding allowed by `F`.

## Capability stages

The detailed later staging is proposed. The minimally viable initial prover
scope below is accepted.

| Obligation | Exact evaluator | Initial canonical rules | Presburger lane | Targeted nonlinear rules |
|---|---:|---:|---:|---:|
| `M == M` | concrete only | prove | prove | prove |
| `M + 0 == M` | concrete only | normalize/prove | prove | prove |
| `M >= 0` for an extent | concrete only | prove from type | prove | prove |
| `4*M % 4 == 0` | concrete only | divisibility rule | prove | prove |
| `A >= B + 8 => A-B >= 8` | concrete only | possibly unknown | prove | prove |
| affine inequalities | concrete only | limited | decide within budget | decide |
| constant div/mod | concrete only | limited | decide within budget | decide |
| `M*N == N*M` | concrete only | canonical product may prove | nonlinear atom | prove |
| cancel `K` from `M*K == N*K` | concrete only | unknown | nonlinear | prove if `K > 0` |
| bound `M*N` | concrete only | trivial constants | nonlinear | interval rules |
| symbolic divisor | concrete only | unknown | nonlinear | selected identities |
| arbitrary nonlinear formula | concrete only | unknown | unsupported | remains incomplete |

"Exact evaluator" means every well-typed expression has deterministic behavior
once roots are bound; it makes no universal claim. The Presburger lane is
mathematically decidable, but a production implementation may return
`ResourceLimit` rather than permit pathological compile time or case explosion.

## What an initial implementation would postpone

An initial prover need not postpone support for dynamic tensor programs. Under
the already accepted three-outcome validation model, it postpones only proof
elimination and some optimization opportunities:

- an unknown semantic requirement becomes a typed host check before allocation
  or device work;
- an unknown physical precondition can become an explicit host guard, or that
  alternative is not admitted;
- an unknown candidate fact is not inserted into the proof context.

Concrete later improvements should be driven by logged proof misses from the
official operation set:

1. broader affine/Presburger implication;
2. stronger constant-modulus congruence reasoning;
3. canonical factored-product equality for reshape and element counts;
4. cancellation guarded by positivity or nonzero evidence;
5. interval propagation through products;
6. budgeted case splitting for `min`, `max`, `select`, and broadcasting;
7. bounded SMT experiments only if these targeted rules leave valuable gaps.

The overall language will still retain `Unknown` after all of these additions.

## Proposed architecture

Keep three components separate:

```text
ShapeExpr representation
    -> exact checked host evaluator
    -> canonicalizer and proof engine
    -> typed proof outcome with evidence/reason
```

A candidate proof interface is:

```rust
enum ProofOutcome {
    Proved(ProofEvidence),
    Disproved(ProofEvidence),
    Unknown(UnknownReason),
}

enum UnknownReason {
    InsufficientFacts,
    UnsupportedFragment,
    ResourceLimit,
}
```

These illustrative types require the accepted semantic newtype discipline in
an implementation. Proof evidence records the rule or solver lane, premises,
and relevant provenance; derived solver caches remain outside canonical graph
identity.

The proposed staged engine is:

```text
canonicalization and substitution
    -> constants and interval bounds
    -> divisibility/congruence
    -> budgeted Presburger reasoning
    -> targeted nonlinear rules
    -> Unknown
```

A general SMT solver is not recommended for the initial core. It would add a
large dependency and trusted surface, solver-version identity, and potentially
unpredictable compile cost without eliminating genuine runtime-dependent
unknowns or nonlinear incompleteness. If later measured, it should be pinned,
resource-bounded deterministically, and allowed to return `Unknown`.

## Proposed decision boundary

Tiler should promise:

> Shape proving is sound, typed, evidence-bearing, and predictably bounded.
> Lack of proof never becomes truth, and increasing proof capability does not
> change shape-expression semantics or runtime validity.

It should not promise that every true proposition over the admitted shape
language is proved during compilation.

## Accepted prototype scope

**Accepted by Tom on 2026-07-19:** constraint proving is a core part of Tiler,
and the minimally viable prototype must contain a functioning prover. An
interface whose implementation returns `Unknown` for every nonconstant query
is not an adequate proof of concept.

The initial native prover covers canonical equality, constant folding,
equality substitution, nonnegative extent facts, checked intervals, and common
constant-divisibility consequences. It emits typed proof evidence and
structured `Unknown` reasons and exercises compile-time rejection, runtime
semantic checks, physical guards, and guard elimination end to end.

A fuller Presburger engine and targeted nonlinear rules remain incremental
extensions driven by recorded proof misses. The prototype must expose a stable
prover boundary that can admit them without changing expression semantics or
the three-outcome validation contract.

## Accepted resolution of the expression-layer boundary

The proposed ABI contract currently describes checked `u64` arithmetic for its
shared expression evaluator. That no longer fully matches the accepted shape
decision permitting signed intermediate expressions.

**Accepted by Tom on 2026-07-19:** `ShapeExpr` and `AbiExpr` remain distinct,
newtyped domain IRs with an explicit checked lowering boundary. Their
implementations compose shared atomic arithmetic/evaluation components where
semantics coincide; code reuse does not collapse the IR layers. Synthesis must
update the proposed ABI contract to reflect the checked lowering and any
signed-capable primitives it actually needs.
