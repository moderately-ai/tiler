# Reduction semantics and legality

**Status:** proposed normative contract and research synthesis  
**Ticket:** `reduction-semantics-contract`

## Outcome

A reduction is a semantic operation over an explicitly ordered logical
contributor sequence. Its axes, combiner family, input-to-accumulator
conversion, accumulator dtype, optional seed, empty-domain behavior,
accumulator-to-result conversion, order permissions, and determinism scope are
semantic identity. A concrete serial, SIMD-group, threadgroup, atomic, or
multi-pass tree remains physical-plan identity.

The first executable slice should support strict, plan-deterministic `Sum` with
one input, one result, one or more statically named axes, explicit accumulator
and result dtypes, no mask, and serial-per-output execution. This small slice
exercises all semantic boundaries without claiming that floating-point tree
reduction is exact. Parallel and multi-pass alternatives become additive once
their topology verifier can prove the order and partial-state rules below.

This record refines, rather than replaces, accepted ADRs 0009, 0011--0015,
0018--0025, and 0039. In particular:

- the seed contributes exactly once;
- the empty result, algebraic identity, and replicable physical padding are
  separate facts;
- reassociation and contributor permutation are independent permissions;
- topology is physical, not semantic; and
- determinism always names a stability scope.

## Primary-source facts

### StableHLO and XLA

StableHLO `reduce` accepts one or more inputs and scalar init values, removes a
set of dimensions, and applies a region. Its schedule is an
implementation-defined full binary tree. The in-order leaves contain the input
slice in ascending lexicographic index order, but the specification permits an
implementation-defined number of init values at implementation-defined
positions. StableHLO consequently says the body and init values must form a
monoid to guarantee equal results across implementations, while explicitly
noting that floating-point addition does not.

This is useful evidence for separating semantic reduction from physical tree
selection, but its init rule is intentionally incompatible with Tiler's
seed-exactly-once contract.

- [StableHLO `reduce`](https://openxla.org/stablehlo/spec#reduce)

### LLVM

LLVM's floating vector add and multiply reductions distinguish a sequential
form from a form carrying the `reassoc` fast-math flag. Without `reassoc`, the
operation begins with the start value and combines elements in increasing
vector-index order. With `reassoc`, the scalarized association need not be
preserved. LLVM's partial floating reduction goes farther: its reduction method
is unspecified and it assumes reassociation and contraction.

LLVM also exposes distinct floating extrema reductions. `fmaximum` and
`fminimum` propagate NaNs and order signed zeros, while legacy number-preferring
forms have different signaling-NaN and order behavior. This reinforces that a
reduction does not own one generic `nan_policy`; it repeatedly invokes a
specific scalar combiner contract.

- [LLVM floating vector-add reduction](https://llvm.org/docs/LangRef.html#llvm-vector-reduce-fadd-intrinsic)
- [LLVM floating vector partial reduction](https://llvm.org/docs/LangRef.html#llvm-vector-partial-reduce-fadd-intrinsic)
- [LLVM floating-point min/max comparison](https://llvm.org/docs/LangRef.html#floating-point-min-max-intrinsics-comparison)

### MLIR Linalg

`linalg.reduce` names reduction dimensions in increasing order and expresses
the combiner in a region with explicit init/output operands. More generally,
Linalg makes iterator kinds and indexing maps explicit so transformations can
reason about loops and dependencies. Its documentation also makes the
frontend responsible for ensuring that region behavior matches declared
iterator semantics; the structural `reduction` label alone is not an
associativity or commutativity proof.

- [MLIR Linalg dialect](https://mlir.llvm.org/docs/Dialects/Linalg/)

### Array frameworks

NumPy accepts an integer or tuple of axes, exposes accumulator/result `dtype`,
and treats `initial` as the starting value. It returns zero for an empty sum.
NumPy may use pairwise summation depending on which axis is reduced, so its
surface `sum` does not imply one portable bitwise order.

JAX's low-level `lax.reduce_sum` accepts zero or more unique axes and returns the
input dtype; unlike `jax.numpy.sum`, it does not widen narrow integers. This is
evidence that accumulator typing belongs to the imported operation contract,
not to the spelling `sum`.

PyTorch warns that reproducibility is not guaranteed across releases or CPU
and GPU platforms, and its deterministic-algorithm mode either selects a known
deterministic implementation or errors. This supports Tiler's scoped
plan-determinism rather than an unqualified boolean.

- [NumPy `sum`](https://numpy.org/doc/stable/reference/generated/numpy.sum.html)
- [NumPy `ufunc.reduce`](https://numpy.org/doc/stable/reference/generated/numpy.ufunc.reduce.html)
- [JAX `lax.reduce_sum`](https://docs.jax.dev/en/latest/_autosummary/jax.lax.reduce_sum.html)
- [PyTorch reproducibility](https://docs.pytorch.org/docs/stable/notes/randomness.html)

### GPU execution

Metal exposes SIMD-group sum, product, minimum, and maximum operations and
shows a two-level reduction through SIMD-group results and threadgroup memory.
The example requires a barrier before the last SIMD group consumes partials.
It demonstrates a useful topology, not portable scalar evaluation order.

CUDA specifies its ordinary atomic functions as relaxed read-modify-write
operations. Conflicting writes are serialized, but their order is undefined.
Atomicity prevents lost updates; it does not establish a deterministic
floating-point contributor order.

- [Apple SIMD-group reductions](https://developer.apple.com/videos/play/tech-talks/10858/)
- [CUDA atomic functions](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/cpp-language-extensions.html#atomic-functions)

## Inferences

1. Axis membership and contributor traversal order must be separate. Sorting
   an axis set can canonicalize identity only if the traversal order is defined
   independently.
2. A scalar combiner's machine semantics apply at every combine step. NaN,
   signed-zero, subnormal, rounding, overflow, and exception-value behavior do
   not become reduction-global defaults.
3. A parallel partial is not necessarily an accumulator value alone. Without
   a proven neutral padding value, it needs a `has_value` state or an equivalent
   mask so an empty partition contributes nothing.
4. Reassociation permits changing parentheses, not changing leaf order.
   Noncontiguous lane assignment therefore also needs permutation permission.
5. Plan determinism and strict left-fold equivalence are independent. A fixed
   parallel tree may be plan-deterministic while differing from the strict
   serial result.
6. Multi-pass materialization is not a new logical rounding boundary. Scratch
   storage must preserve the selected accumulator value contract; narrowing,
   flushing, or NaN rewriting needs a separately authorized semantic
   relaxation or makes the plan illegal.
7. Atomic floating accumulation has an arrival-order-dependent tree. It is not
   plan-deterministic merely because the instruction is atomic.

## Proposed semantic contract

The following is descriptive schema, not a committed Rust API:

```text
Reduction {
  input: TensorValue,
  axes: NonEmptyCanonicalAxisSet,
  reducer: ReductionOperatorKey,
  input_conversion: ConversionContract,
  accumulator_dtype: DTypeKey,
  initial: None | ScalarValue,
  empty_without_initial: EmptyResult,
  result_conversion: ConversionContract,
  result_dtype: DTypeKey,
  result_value_policy: ResultValuePolicy,
  order: ReductionOrderContract,
  determinism: DeterminismScope,
}

EmptyResult = Value(TypedResultScalar) | Error(EmptyReductionErrorKey)

ReductionOrderContract {
  reassociation: Forbid | Permit,
  permutation: Forbid | Permit,
}
```

`ReductionOperatorKey` resolves to one registered binary scalar operation over
accumulator values plus reduction capabilities. Built-ins use specialized
families such as `Sum`, `Product`, `Minimum`, `Maximum`, `MinimumNumber`,
`MaximumNumber`, `All`, and `Any`; arbitrary binary graph regions are not the
initial extension mechanism. The descriptor states whether the scalar operation
is associative or commutative under the exact resolved dtype/numerical
contract, and whether a bitwise-neutral replicable padding value is proven.
Those capabilities do not grant the matching user permission.

### Axes and output shape

- Frontends resolve negative axes against input rank before graph admission.
- Canonical axes are unique, in range, and sorted by original input-axis
  number. Duplicates are invalid rather than silently deduplicated.
- A canonical reduction has at least one axis. A frontend request with no axes
  lowers to explicit elementwise conversion/seed combination rather than a
  degenerate reduction.
- The result axes are unreduced input axes in their original order.
- `keepdims` is frontend sugar for a reduction followed by explicit singleton
  axis insertion/reindexing; it is not a second reduction shape convention.
- For each result coordinate, contributors are input coordinates matching all
  unreduced coordinates. The canonical contributor sequence is ascending
  lexicographic order over reduced coordinates in original input-axis order;
  the greatest-numbered reduced axis varies fastest.
- If a surviving result extent is zero, the result contains no elements and no
  empty-domain scalar is evaluated. Otherwise, if any reduced extent is zero,
  each logical result has an empty contributor sequence.

Example: reducing shape `[2, 3, 4]` over axes `{0, 2}` produces shape `[3]`.
For output `j`, contributor order is
`(0,j,0), (0,j,1), ..., (0,j,3), (1,j,0), ..., (1,j,3)`.

### Scalar evaluation

For each logical output independently:

1. convert every input contributor to `accumulator_dtype` using
   `input_conversion`;
2. if `initial` exists, convert it to the accumulator contract and place it
   exactly once before all input contributors;
3. evaluate a legal tree over that sequence using the resolved scalar reducer;
4. convert the final accumulator exactly once with `result_conversion` to
   `result_dtype`; and
5. apply the reduction operation's resolved result-value policy, including
   canonical arithmetic NaN normalization when portable-bitwise conformance
   requires it.

When both order permissions are forbidden, the legal tree is the left fold in
canonical contributor order. With an initial value `s`, it is
`(((s op x0) op x1) ... op xn)`. Without an initial value, a nonempty sequence
starts from `x0`; `x0` is not combined with an implicit identity.

If there is no input contributor:

- with `initial`, the result is `result_conversion(convert(initial))`;
- without `initial`, `empty_without_initial` supplies the already resolved,
  typed result value or raises its semantic error. The declared value includes
  the operation's final NaN/zero policy and is not implicitly converted through
  the accumulator.

The empty value is not passed through the reducer and does not prove a neutral
padding value. An identity-less reducer without an initial requires a statically
proven or runtime-validated nonempty domain. Failure is a semantic precondition
error, not an optimization-guard miss.

### Allowed trees

Let the canonical leaves be the optional seed followed by converted input
contributors.

| Reassociation | Permutation | Allowed evaluation |
|---|---|---|
| forbid | forbid | canonical left fold only |
| permit | forbid | any full binary tree whose in-order leaves are canonical |
| forbid | permit | any left fold over a permutation of input contributors; the seed remains first |
| permit | permit | any full binary tree over a permutation of input contributors; the seed occurs once |

Permission is necessary but not sufficient. Reassociation also requires the
reducer's applicable associativity capability; permutation requires its
commutativity capability. A plan consuming neither freedom need not prove those
capabilities.

The seed remains first even when permutation is allowed. Moving a non-neutral
seed is a separate transformation for which the initial contract provides no
permission.

### NaN, zero, overflow, and rounding

Reduction inherits each combine step from its exact scalar reducer:

- floating `Sum` uses the resolved floating `Add` contract, including
  round-to-nearest ties-to-even in the initial profile;
- integer `Sum` names a wrapping, saturating, checked, widening, or other
  explicit addition family;
- `Minimum`/`Maximum` propagate NaN and order `-0 < +0`;
- `MinimumNumber`/`MaximumNumber` prefer a numeric operand when exactly one is
  NaN and use the same deterministic zero ordering; and
- portable-bitwise arithmetic canonicalizes produced NaNs according to ADR
  0018 after every scalar combine whose result is NaN and at the reduction
  result boundary. Thus even a singleton or empty arithmetic reduction cannot
  leak target-selected NaN payload behavior.

Reassociation or permutation never silently implies permission to ignore NaNs,
zeros, subnormals, intermediate overflow, or rounding. For example, signed
saturating addition lacks a general associativity capability, and checked
addition cannot be regrouped when it changes whether an intermediate failure
occurs.

### Determinism

The initial reduction guarantee is plan deterministic as defined by ADR 0013.
The artifact records the selected topology, partitioning, scratch format,
compiler contract, and target compatibility identity. Identical input bits and
runtime bindings routed to the same plan must produce identical output bits.

This permits a fixed parallel tree when the order contract admits it, but
rejects timing-dependent atomic accumulation. Portable-bitwise scope is
stronger: every selected realization must produce the same required bits across
the declared target set. An operation whose result varies across its admitted
trees cannot claim portable-bitwise merely because every individual tree is
repeatable.

## Physical partial and multi-pass contract

A physical partial is:

```text
PartialAccumulator {
  has_value: bool,
  value: accumulator_dtype, // meaningful only when has_value
  covered_contributors: OrderedSubset,
}
```

An implementation may erase `has_value` only when it proves every partial is
nonempty or supplies a replicable padding value that is observably neutral
under the complete scalar contract.

For reassociation without permutation, each first-pass partial covers one
contiguous interval of the canonical contributor sequence, and later merges
preserve interval order. Strided lane assignment such as lane `l` consuming
`l, l+W, l+2W` creates noncontiguous subsets; merging lane totals generally
permutes contributors and therefore needs permutation permission or a proof of
equivalence for the concrete reducer.

The seed is attached once at the root-facing boundary, never once per lane,
threadgroup, tile, or pass. Empty partials are skipped rather than combined.
Every scratch write/read preserves accumulator dtype, representation,
subnormals, signed zero, and required NaN bits. A deliberately narrower partial
format or additional quantization is a different semantic contract, not a
cost-only schedule choice.

A multi-pass topology is legal only if its complete composed tree belongs to
the semantic order class. Splitting a strict ordered fold into independently
reduced parallel chunks is illegal even when each chunk is internally ordered,
because merging the chunks reassociates the sequence. A serial chain of passes
that carries the prior accumulator can preserve the strict fold, although it is
unlikely to be profitable.

## Legality versus cost

| Question | Hard semantic/physical feasibility | Cost only |
|---|---|---|
| Axes | unique, in range, normalized; shape and contributor mapping correct | axis order chosen for locality after preserving mapping |
| Types | exact input/seed/result conversions and supported accumulator reducer | wider accumulator throughput and register footprint |
| Empty domain | declared value, initial, or enforced nonempty precondition | guard/validation latency |
| Order | composed tree and leaf order fit permissions and reducer capabilities | serial depth versus parallel fan-in |
| Seed | present exactly once and first | placement traffic |
| Padding | full-contract bitwise-neutral proof, or masked/nonempty partial state | masked-lane overhead |
| NaN/zero/subnormal | every combine and scratch boundary refines scalar contract | fixup instruction cost |
| Integer overflow | selected overflow family preserved at every combine | widening/emulation cost |
| Determinism | topology satisfies declared stability scope; no timing-dependent choice | deterministic alternative may be slower |
| Multi-pass scratch | lossless accumulator representation, valid lifetimes and dependencies | bytes, allocation, and extra dispatches |
| Synchronization | race-free merge, uniform barriers, valid collective scope | barrier/atomic latency |
| Target resources | group sizes, local memory, bindings, launch and prepared-kernel limits | occupancy above feasibility, bank conflicts, coalescing |

No estimated speedup can legalize a failed hard row. Conversely, a legal
serial plan is not rejected merely because a tree plan is expected to be
faster.

## Normative examples

Assume binary32 round-to-nearest ties-to-even unless stated otherwise.

1. **Strict sum:** reducing `[1e20, -1e20, 3.25]` without a seed by canonical
   left fold produces `3.25`. The tree `1e20 + (-1e20 + 3.25)` produces `0.0`
   and requires reassociation permission.
2. **Permutation is separate:** canonical `[1e20, 3.25, -1e20]` left-folds to
   `0.0`. Permuting to `[1e20, -1e20, 3.25]` produces `3.25` without changing
   the left-deep shape; it requires permutation permission.
3. **Seed once:** `Sum([1, 2], initial=10)` is `13`. Seeding two partials gives
   `23` and is illegal.
4. **Signed-zero padding:** an unseeded strict sum of singleton `[-0.0]` returns
   the input `-0.0`. Combining it with an injected `+0.0` yields `+0.0`; the
   empty-sum value is therefore not strict neutral padding.
5. **Empty versus zero output count:** reducing shape `[0, 3]` over axis `0`
   evaluates three empty reductions. Reducing the same shape over axis `1`
   has result shape `[0]` and evaluates no scalar reductions.
6. **Extrema families:** `Maximum([qNaN, 1])` produces the canonical arithmetic
   NaN; `MaximumNumber([qNaN, 1])` produces `1`. A backend native `fmax` cannot
   implement both contracts without checking its exact semantics.
7. **Contiguous partials:** for contributors `[a,b,c,d]`, partials `(a op b)`
   and `(c op d)` merged in that order require reassociation but not
   permutation. Partials `(a op c)` and `(b op d)` also permute leaves.
8. **Narrow scratch:** an f32 accumulator written as f16 between passes adds an
   observable conversion and is illegal unless the semantic contract already
   permits that boundary.

## Required adversarial tests

Every supported reducer/dtype/order cell has positive and negative tests.

- ranks 0 through the supported maximum; first, middle, last, multiple, all,
  duplicate, out-of-range, and dynamically bound axes;
- extent zero on a reduced axis versus extent zero on a surviving axis;
- no seed, neutral seed, non-neutral seed, runtime seed, and seed-conversion
  halfway/overflow cases;
- singleton signed zeros, both zero signs in both orders, subnormals, infinities,
  qNaN/sNaN in every contributor position, and several NaN payloads;
- three-element reassociation and permutation witnesses;
- serial, balanced, skewed, SIMD, threadgroup, contiguous multi-pass,
  noncontiguous lane, and atomic-arrival trees;
- empty partials with masks and `has_value`, plus invalid replicated empty
  values and replicated seeds;
- integer wrapping, saturating, checked, and widening boundary vectors;
- f16/bf16 inputs accumulated in f32 and finalized to same or narrower result;
- scratch round trips at normal/subnormal boundaries and NaN canonicalization;
- repeated execution under the exact artifact/variant/target identity claimed
  by plan determinism; and
- verifier rejection reasons naming the missing permission, algebraic
  capability, target capability, nonempty proof, or lossless scratch contract.

The executable spike at
[`spikes/numerics/reduction_contract_probe.py`](../../../spikes/numerics/reduction_contract_probe.py)
checks the core bit-level witnesses without third-party dependencies.

## First vertical slice

Included:

- built-in one-input/one-result `Sum`;
- f32 input, accumulator, and result;
- one or more statically named nonempty axes;
- statically known shapes, including zero extents;
- no explicit seed in the initial API, but the internal contract and reference
  evaluator implement seed-once;
- empty sum `+0.0` as a result, never inferred as padding;
- strict canonical left fold;
- plan-deterministic serial-per-output physical schedule; and
- out-of-place output.

Explicitly excluded from the first slice, without making them impossible:

- parallel, SIMD-group, threadgroup, atomic, and multi-pass reductions;
- reassociated or permuted floating reductions;
- product, extrema, logical, integer, complex, quantized, and user-defined
  reducers;
- masks/`where`, runtime axes, empty axis sets, `keepdims` inside the reduction,
  per-output seeds, and data-dependent output shapes;
- argmin/argmax, mean, variance, norms, log-sum-exp, scans, windows, segments,
  histograms/scatters, distributed collectives, and multiple inputs/results;
- backward kernels and in-place/aliased output; and
- portable-bitwise claims beyond the exact f32 scalar-operation and target
  conformance actually established.

The next additive step is a fixed contiguous balanced-tree f32 `Sum` variant
behind explicit reassociation permission, followed by a permutation-requiring
lane-strided variant. A multi-pass variant should wait until partial-state,
scratch-preservation, and program-level dependency verification exist.

## Decision record for later ADR promotion

### Context

Existing ADRs separate semantic order from physical topology but leave the
canonical contributor sequence, axis normalization, seed position, partial
state, and initial supported subset unspecified. Those omissions permit two
implementations to claim the same semantic identity while producing different
strict results.

### Proposed decision

Adopt the schema and rules in this record: canonical nonempty axis sets;
ascending-lexicographic contributors; seed exactly once and first; explicit
input/seed/result conversions and result policy; a two-bit reassociation versus
permutation order class; scalar-combiner inheritance for all exceptional-value
behavior; masked partial state unless padding neutrality is proven; and
route-independent plan determinism. Admit only the serial f32 `Sum` slice
listed above initially.

### Consequences

- The reference evaluator has one target-independent strict answer.
- Parallel and multi-pass schedules remain possible, but must expose and prove
  their full composed tree and partial representation.
- Importers must lower framework defaults for axes, widening, seeds, masks,
  and empty behavior explicitly.
- Some profitable native reductions are rejected until a matching numerical
  permission and target conformance record exist.
- Later reducer families and determinism scopes are additive versioned
  contracts rather than reinterpretations of `Sum`.

### Alternatives rejected

- **StableHLO-compatible arbitrary init injection:** conflicts with Tiler's
  seed-once invariant and admits non-monoidal surprises.
- **Concrete semantic tree:** specifies strict results but makes tensor meaning
  target-schedule dependent and conflicts with ADR 0012.
- **One unordered flag:** conflates regrouping with permutation and conflicts
  with ADR 0014.
- **Empty result as padding:** changes strict signed-zero and other observable
  results, as established by ADR 0025.
- **Backend-defined accumulation dtype/order:** makes frontend, reference, and
  fallback meaning target-dependent.

## Measurements

No target performance or numerical conformance measurements were made in this
ticket. The executable spike is a host-side semantic witness, not evidence that
Metal, CUDA, or another backend realizes the same contract. Backend admission
still requires generated-code inspection and target/device conformance under
the selected compiler and numerical flags.

## Remaining genuinely non-obvious questions

1. Should a later public generic reduction accept any registered pure binary
   scalar operation with a capability descriptor, or only named reduction
   operation families whose complete contracts are audited centrally?
2. Is plan determinism sufficient for every initial user-facing mode, or is a
   weaker explicitly nondeterministic execution scope worth adding for atomic
   reductions? The latter would expand fallback, testing, and artifact
   contracts and should not be inferred from a fast mode.
3. Which exact target/toolchain tuples preserve f32 accumulator bits across
   scratch storage, especially subnormal and NaN payload behavior? This is a
   measurement question, not a semantic default.
