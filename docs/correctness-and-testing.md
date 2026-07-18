# Correctness and testing

**Status:** proposed

Tiler must define semantics before asking a GPU compiler to accept generated
source. Backend compilation is a validation layer, not the type system or the
semantic authority.

## Semantic authority

A normative operation specification is the authority. A slow reference
evaluator implements it and should support every operation before optimized
scheduling is enabled. Differential tests compare:

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

## Verification gates

Each lowering verifies its input and output:

| Gate | Primary checks |
| --- | --- |
| Frontend | Axis occurrence, ellipsis, factors, introduced/removed axes, source diagnostics |
| Registry/extension | Key/provider coherence, canonical attributes, capability determinism, trust/budget boundaries |
| Semantic | Shape, dtype, broadcasting, reduction policy, pure DAG, valid outputs |
| Index | Rank, integer types, bounds, overflow, divisors, writer coverage, runtime parameters |
| Schedule | Observational coverage, safe redundancy/tails, resources, convergence, numerical contract, capabilities |
| Kernel | Scope/dominance, types, access modes, address spaces, barriers, store bounds |
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
- shape products near index-width boundaries.

Random programs should be small enough to shrink into useful counterexamples.
Every optimizer rule needs positive tests, negative precondition tests, and a
semantic equivalence property.

## Reduction matrix

Reduction tests explicitly cover:

- extents below, equal to, and above SIMD-group width;
- more than one SIMD group;
- ragged and non-power-of-two tails;
- zero and one-length domains under documented identities;
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
- Operation/value graphs preserve use-def relationships, ordered named results,
  sharing, and individually typed multi-result operations.
- Dead pure operations are removed before canonical identity; live operations
  remain when any result is reachable.
- Built-in and third-party operation definitions pass the same mandatory
  capability and deterministic-attribute conformance suite.
- Registry snapshots are identical under shuffled/parallel registration;
  duplicate semantic ownership and provider conflicts are rejected.
- Semantic operation keys and provider revisions affect only their intended
  identities; no `TypeId`, pointer, vtable, or registration order leaks into
  durable content.
- Canonical attributes reject duplicate keys, noncanonical encodings, invalid
  defaults, excessive depth/count/bytes, and checked-size overflow.
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
- Program verification rejects data-dependent output shapes in the initial
  profile, cross-device values/stages, noncanonical step order, unauthorized
  concurrency, temporary use outside its lifetime, and allocation aliasing or
  reuse forbidden by the initial buffer plan.

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
- Native macOS embeds Metal; non-Apple builds select fallback without Apple
  tools; unsupported cross-Apple builds fail with an actionable diagnostic.
- A macOS-host non-Apple target's unnecessary Metal work is measured until
  better target discovery or multi-platform generation is implemented.
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
