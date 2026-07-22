---
schema: "tiler-doc/v1"
id: "tiler.contract.correctness-and-testing"
kind: "contract"
title: "Correctness and testing"
topics: ["correctness", "testing", "verification"]
contract_status: "accepted"
implementation_status: "not-started"
evidence: ["tiler.research.numerics.operation-conformance-matrix", "tiler.research.numerics.region-accuracy-contract", "tiler.research.numerics.sound-region-analyzer-spike"]
ticket: "reference-evaluator-slice"
---

# Correctness and testing

**Status:** accepted research contract; implementation pending

Tiler must define semantics before asking a GPU compiler to accept generated
source. Backend compilation is a validation layer, not the type system or the
semantic authority.

## Traceability

This document owns cross-layer verification and evidence requirements. It does
not redefine operation semantics; those are owned by [Numerical semantics](numerical-semantics.md).
Its numerical evidence includes the [operation conformance matrix](research/numerics/operation-conformance-matrix.md),
[region-accuracy contract](research/numerics/region-accuracy-contract.md), and
[bounded sound-analyzer spike](research/numerics/sound-region-analyzer-spike.md).


## Semantic authority

A normative operation specification is the authority. A slow reference
evaluator implements it directly or through an exact verified decomposition;
one of those paths should cover every operation before optimized scheduling is
enabled. Differential tests compare:

```text
frontend or independent compatibility reference
        versus
Tiler reference evaluator
        versus
generated backend program
```

The comparison follows the declared numerical contract and conformance level:
bitwise, toolchain-specific, backend-elementary, bounded-error, or permitted
result set. A runtime such as Candle is an oracle only where its contract
matches the normative operation semantics. The proposed initial integration
adds Candle-versus-Metal comparisons, but those systems do not define core
semantics. Nondeterministic reductions may require repeated runs and
invariant/result-set checks rather than one expected value plus tolerance.

The evaluator is independently tested with hand-authored conformance vectors,
small exhaustive cases, and higher-precision arithmetic where appropriate so a
shared evaluator/lowering bug is not mistaken for agreement.

Bounded transcendental evaluation computes the named immutable reference with
enough precision or certified intervals to decide the exact rational predicate;
it does not round an oracle and then measure against that rounded value. A
named-elementary profile uses its frozen versioned definition and independent
conformance corpus. Running the same live backend as both implementation and
oracle is circular and cannot establish conformance.

## Verification gates

Each lowering verifies its input and output:

| Gate | Primary checks |
| --- | --- |
| Frontend | Axis occurrence, ellipsis, factors, introduced/removed axes, source diagnostics |
| Registry/extension | Key/provider coherence, canonical attributes, capability determinism, trust/budget boundaries |
| Semantic | Shape, dtype, broadcasting, reduction policy, pure DAG, valid outputs |
| Index | Rank, integer types, bounds, overflow, divisors, writer coverage, runtime parameters |
| Schedule | Observational coverage, safe redundancy/tails, resources, convergence, numerical contract, capabilities |
| Kernel | Scope/dominance, types/effects, access modes/address spaces, schedule-refined bounds and ownership, barrier/collective convergence and fences, reduction/order and launch references |
| Program/buffer | Semantic coverage, dependencies, boundary contracts, placement, initialization, allocation/lifetime/alias rules |
| Artifact | Symbols, ABI, hashes, target, launch metadata, guard completeness |

Verification is mandatory during expansion-time generation. Debug APIs may expose
additional expensive proofs, but core safety checks are never optional.

## Property and differential testing

Generate combinations of:

- ranks from scalar through the supported maximum;
- dimensions 0, 1, SIMD width minus/at/plus one, powers and non-powers of two;
- composed and split axes;
- permutations and inverse permutations;
- broadcast axes and unit extents;
- contiguous views with nonzero start offsets;
- supported strided layouts;
- deliberately unsupported layouts that must fall back;
- NaN, infinities, signed zero, and extreme finite values;
- quantization scales at zero, negative, subnormal, normal, maximum finite,
  infinity, and NaN; code and zero-point endpoints; per-axis/block parameter
  grids with distinct sentinel values;
- strict affine quantization rejecting qNaN and sNaN before committing an
  observable result, plus separate conformance vectors for every explicitly
  admitted alternative NaN mapping;
- float-to-integer values on both sides of every rounded destination boundary,
  signed zeros, subnormals, infinities, and qNaN/sNaN; strict/exact rejection,
  ordered saturation, and explicit NaN-to-zero totalization remain distinct;
- checked integer arithmetic returning wrapped low bits plus the correct
  overflow predicate, and widening signatures rejecting every result dtype
  that cannot represent the full mathematical domain;
- IEEE decimal cohort members with equal numerical value but different quantum,
  including DPD/BID transcodes that preserve every admitted observable;
- proof-elided, host-check, device-pre-scan, and transactional validation paths
  producing the same success/error contract; private failed results discarded;
  no dependent publication or fallback after device enforcement begins;
- transcendental clause boundaries, zeros, binade and normal/subnormal
  transitions, overflow thresholds, hard-to-round values, and large
  argument-reduction inputs; every pre-output-policy candidate checked against
  the exact reference and all applicable typed clauses, then the observable
  result checked against the composed subnormal, zero, overflow, and NaN
  policies;
- shape products near index-width boundaries.
- target hard limits at minus/equal/plus one; absent, unknown, stale, and
  dishonest capability providers; fixed/scalable vector legality across
  operation/dtype/mask/address-space/alignment combinations; barrier scope,
  fence, and convergence; deferred checks at their exact preparation phase;
  specialization-specific kernel facts; generic fallback retention; and proof
  that estimates never establish legality.

The cross-operation coverage, adversarial numerical atoms, and backend compiler
verification protocol are maintained in the
[operation conformance matrix](research/numerics/operation-conformance-matrix.md).

Random programs should be small enough to shrink into useful counterexamples.
Every optimizer rule needs positive tests, negative precondition tests, and a
semantic equivalence property.

For curated graphs of at most eight operations, the exhaustive region oracle
enumerates all legal candidates, exact partitions, multi-output alternatives,
and explicitly permitted duplication covers. The bounded production search is
checked for three independent outcomes: every emitted candidate is oracle-
legal, singleton/unfused coverage remains complete, and missed legal
alternatives are reported as bounded search loss. Cost-model comparisons then
measure selection regret separately from enumeration correctness.

The first normative end-to-end evaluator case preserves an explicit
`f32 -> f16 -> f32` rounding boundary before a broadcasted add and returns both
the add result and a row-major reshaped view as ordered graph outputs. Tests
must demonstrate that deleting the cast boundary changes bits, that broadcast
and reshape errors have stable codes, and that both output shapes/bit sequences
match the reference contract.

## Reduction matrix

Reduction tests explicitly cover:

- extents below, equal to, and above SIMD-group width;
- more than one SIMD group;
- ragged and non-power-of-two tails;
- zero and one-length domains under documented identities;
- singleton negative zero under a positive-zero empty result, proving that
  empty results are not automatically legal per-lane padding;
- every supported accumulator dtype;
- serial, SIMD-group, threadgroup, and multi-pass strategies;
- result visibility to consuming lanes;
- barriers and convergence;
- fused prologue and epilogue expressions;
- multiple reductions in one semantic region when introduced.

Benchmarks are not substitutes for these correctness cases.

## IR and canonicalization tests

- Stable serialization round trips.
- Hashes do not depend on construction order or transient IDs.
- Program input/output interface keys participate in identity, while display
  names and source spans do not. Tests cover duplicate-key rejection,
  deterministic ordinal defaults, display-only renames, and two output keys
  intentionally referencing the same value.
- Operation/value graphs preserve use-def relationships, ordered named results,
  sharing, and individually typed multi-result operations.
- Dead pure operations are removed before canonical identity; live operations
  remain when any result is reachable.
- Built-in and third-party operation definitions pass the same mandatory
  capability and deterministic-attribute conformance suite.
- Registry snapshots are identical under shuffled/parallel registration;
  duplicate semantic ownership and provider conflicts are rejected.
- Semantic operation keys, provider-independent definitions, and provider
  revisions affect only their intended identities. Identical graphs admitted
  by different provider revisions have equal `SemanticGraphIdentity` and
  definition projections but unequal admission provenance and registry
  snapshots. No `TypeId`, pointer, vtable, or registration order leaks into
  durable content.
- Reached semantic-authority closure tests cover nested parameterized and
  encoded components, occurrence `Type` and `FloatBits` references, operation
  defaults/facts/conformance, missing definition references, finite cycles,
  and aggregate resource exhaustion at the first item beyond the governed
  limit. Used provider revisions change admission and snapshot subjects;
  unused revisions change only the full snapshot subject.
- Region identity tests distinguish equal semantic content at different graph
  occurrences; index/schedule/KIR structure remains reusable while checked
  refinements and complete-program coverage retain exact occurrence bindings.
- Canonical attributes reject duplicate keys, noncanonical encodings, invalid
  defaults, excessive depth/count/bytes, and checked-size overflow.
- Canonical-attribute vectors cover every integer width, signed zero and NaN
  float-bit payloads, exact UTF-8, empty and boundary-length byte strings,
  ordered sequences, sorted records, unknown fields, and equivalent
  explicit/default representations.
- Provider callbacks are deterministic under repeated/concurrent invocation;
  contradictory capabilities are hard diagnostics and recoverable panics are
  attributed to the provider without committing partial mutations.
- Extension rewrites are transactional, fully reverified, cycle-detected, and
  bounded by per-rule/global budgets.
- Missing extension capabilities conservatively block the corresponding
  rewrite, fusion, or lowering rather than being trusted implicitly.
- Unknown operation keys cannot enter a verified/compilable semantic graph.
- Equivalent canonical programs hash identically where promised.
- Semantically different guards, schedules, ABIs, or numerical contracts hash
  differently.
- Malformed control flow, types, pointers, and effects are rejected.
- Kernel refinement tests reject missing or mismatched bounds/ownership
  witnesses, undeclared invocation coordinates, divergent barriers, nonuniform
  barrier loop counts, insufficient fences, changed reduction order, and
  uncontracted conversions before backend source emission.
- Simplification preserves overflow and division semantics.
- `EXPLAIN` output is deterministic.
- Every normative verifier invariant has at least one negative/rejection test.
- Equivalent normalized schedules hash identically even when produced through
  different transform histories; traces remain separately replayable.
- Schedule verification rejects missing domain coverage, conflicting writes,
  divergent barriers, invalid coordinate maps, and resource overflow before
  backend emission.
- Symbol scopes distinguish equal spellings, reject free/contradictory symbols,
  and prove that every dynamic output, temporary, guard, and launch expression
  has an admitted host-evaluable source.
- Index tests compose identity, permutation, broadcast, split/merge, and static
  or dynamic reshape maps; distinguish read aliasing from exact unique write
  ownership; reject out-of-bounds/data-dependent accesses; and verify
  noncontiguous positive-stride views with nonzero starts.
- The implemented static index-profile gate additionally covers huge
  permutations without enumeration, bounded exhaustive ownership evidence,
  explicit access domains, exact linear normalization, rank-zero output
  ownership, zero-contributor reductions, unused/free reduction rejection,
  tensor-binding identity separation, dead-draft compaction, proof resource
  caps, and compile-time rejection of forged verified regions. Dynamic ShapeEnv
  bindings, predicate exchange, split/merge, semantic-lowering equivalence, and
  physical views remain requirements rather than completed coverage.
- Width tests prove every narrowed coordinate, linearization, element-offset,
  byte/packed-offset, and dispatch intermediate. They include cases where every
  extent fits `u32` but stride multiplication does not, and require the guarded
  variant to select a verified wide path rather than wrap.
- Tail tests at vector width minus/equal/plus one prove inactive scheduled
  points cannot access memory; tail predicates never weaken logical access-map
  totality.
- Program verification rejects data-dependent output shapes in the initial
  profile, cross-device values/stages, noncanonical step order, unauthorized
  concurrency, temporary use outside its lifetime, and allocation aliasing or
  reuse forbidden by the initial buffer plan.
- Every data use and storage reuse is justified by a typed dependency and
  `StorageHandoff`; canonical list/stream order alone is rejected as a lifetime
  or visibility proof. Multi-pass tests preserve accumulator bits through
  scratch and reject narrowing or early reuse.

## Metal and artifact tests

- MSL snapshots plus structural assertions for every structured operation.
- Every scheduled operation compiles with `xcrun metal`.
- Helpers are emitted and deduplicated correctly.
- Each macro-local bundle packages and loads all entry points required by its
  complete one- or multi-step plans.
- Compiler diagnostics identify the originating kernel.
- Canonical IR, MSL, manifest, entry ordering, and cache keys are deterministic.
- Metallib byte identity is tested only within a pinned, verified toolchain and
  environment contract.
- Cache changes when compiler, target, flags, schema, ABI, guards, or schedule
  change.
- Concurrent macro/rustc processes compile an identical cache key once and
  never observe partial artifacts.
- Corrupt or truncated bundles fail validation.
- ABI expression evaluators are fuzzed for overflow, division by zero, invalid
  references, excessive depth, and narrowing.
- Host/MSL metadata layout is checked field by field for offsets, padding,
  signedness, booleans, and binding indices.
- Pipeline reflection is compared with generated bindings where supported.

Metal tests require an eligible macOS runner. Core IR, verifier, evaluator, and
optimizer tests remain platform-independent.

## Proc-macro AOT tests

- An inline invocation cold-compiles and embeds a loadable manifest/metallib
  without consumer `build.rs` or a prebuild command.
- Equivalent warm expansions perform no `xcrun` work, including across rustc
  processes.
- Manifest and metallib are emitted as byte-string literal tokens; generated
  Rust contains no compiler-cache path or `OUT_DIR` dependency.
- Cache deletion, `cargo clean`, incremental compilation, compiler upgrades,
  toolchain changes, lock contention, and stale-lock recovery are safe.
- rust-analyzer and `cargo check` cold/warm behavior is measured and preserves
  the same types and diagnostics as normal compilation.
- Bundle sizes of roughly 10 KiB, 100 KiB, and 1 MiB have explicit rustc
  time/memory and binary-size measurements.
- Many identical invocations establish whether linker constant merging occurs;
  correctness never depends on it.
- Generated consumer-`cfg` tests cover macOS, iOS device, iOS simulator,
  Catalyst, and an unrelated non-Apple target. A selected matching family
  embeds its payload or emits its retained actionable compile error; a
  nonmatching target compiles the semantic fallback; `FallbackOnly` performs
  no backend compiler work.
- A capable macOS host's selected-family work while compiling an unrelated
  target is measured; the content cache bounds it, and correctness never
  depends on proc-macro consumer-target discovery.
- External Metal errors preserve invocation spans and retained canonical MSL.

## Candle integration tests

- Output shape and dtype are correct.
- Element/byte offset convention is applied exactly once.
- Noncontiguous guard and fallback behavior is correct.
- Zero work does not issue an illegal dispatch.
- Buffer/scalar ordering matches the manifest.
- Repeated calls reuse per-device pipelines.
- Separate device instances do not share device-bound objects.
- Chained custom operations remain asynchronous and ordered.
- Autograd behavior matches the documented policy.
- Fallback agrees with the fused result.
- Preflight fallback happens before custom-op application; launch-time failures
  do not execute a second graph after possible device effects.
- Guard failure or artifact validation encodes no kernel and leaks no partially
  initialized output.
- Output allocation and dispatch formulas reject overflow and boundary values.
- Maximum reachable element is checked against allocation bytes.
- Misaligned effective addresses, truncated metadata, duplicate/missing
  bindings, wrong scalar width, and forbidden aliasing are rejected.
- Concurrent first use creates one cache entry safely; buffers and pipeline
  objects remain alive until GPU completion.
- Initial arity limits and partition/failure beyond them are tested.
- Multi-step plans allocate, bind, retain, and release temporaries according to
  the manifest dependency/lifetime contract.
- Routing compares complete one- and multi-kernel plans and never mixes steps
  from different numerical contracts.

## Performance testing

Measure separately:

- cold and cache-hit macro expansion time;
- generated MSL, manifest, metallib, expanded-token, and final-binary size;
- rustc time and peak memory attributable to embedded byte literals;
- first library/function/pipeline creation latency;
- warm dispatch latency;
- kernel count and intermediate allocation count;
- end-to-end latency and effective bandwidth;
- performance cliffs around guards, vector widths, and reduction regimes;
- optimizer estimate versus observed execution.

Performance regressions should retain `EXPLAIN` diffs so changes can be
attributed to a plan, codegen, toolchain, or hardware-profile change.

Metal execution tests run on each supported deployment/device family; source
compilation alone cannot detect races, barrier errors, inactive-lane reduction
bugs, access-map mistakes, or ABI binding errors. Validation-enabled Metal runs
are included where CI hardware permits.
