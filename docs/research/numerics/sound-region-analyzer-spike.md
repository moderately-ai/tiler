---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.sound-region-analyzer-spike"
kind: "research"
title: "Sound region-accuracy analyzer integration spike"
topics: ["numerics","accuracy","proof"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "informational"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis","sound-proof","bounded-measurement"]
informs: ["tiler.contract.correctness-and-testing"]
ticket: "spike-sound-region-accuracy-analyzer-integration"
---

# Sound region-accuracy analyzer integration spike

**Status:** bounded feasibility gate passed for a trusted-analyzer profile;
independent certificate checking remains unavailable

## Traceability

- **Current disposition:** informational; historical status text below records the report's state when written.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md) and [Correctness and testing](../../correctness-and-testing.md).
- **Adoption:** No ADR directly adopts this bounded evidence.
- **Work record:** [spike-sound-region-accuracy-analyzer-integration](../../../tickets/spike-sound-region-accuracy-analyzer-integration.md).


## Outcome

A small sound profile is practical for fixed, branch-free scalarizations, but
not as an ambient optimizer permission. Tiler can translate a fully bound
candidate into Daisy, accept a result only from a pinned and governed analyzer
profile, and treat every unsupported or ambiguous case as `Unknown`.

The measured profile supports `+`, `-`, `*`, `/`, `sqrt`, explicit FMA,
explicit f32/f16 precision boundaries, relational input assumptions, and
small reductions unrolled in their exact topology. Analysis itself took
8--320 ms in the measured cases; Scala frontend startup made total invocation
time roughly 0.9--1.5 seconds. This is viable for a bounded compile-time proof
portfolio with caching, not for indiscriminate use on every search candidate.

The profile does **not** produce an independently checkable proof certificate.
The soundness authority is the exact Daisy source revision, its analysis
configuration, the adapter, and the admitted semantic profile. This satisfies
the experiment's trusted-analyzer alternative, but it is a materially larger
trusted computing base than a small certificate checker.

## Facts from primary sources

### Daisy

- Daisy describes its static analysis as a sound over-approximation and offers
  dataflow, optimization-based, relative-error, interval-subdivision, and
  mixed-precision modes in its [project documentation](https://github.com/malyzajko/daisy/blob/38a0f33915dde03eeadd34786a920e834c1d9110/doc/documentation.md).
- Its f32 model checks representable range, uses gradual-underflow error for a
  subnormal-only range, and throws on potential overflow in
  [`FinitePrecision.scala`](https://github.com/malyzajko/daisy/blob/38a0f33915dde03eeadd34786a920e834c1d9110/src/main/scala/daisy/tools/FinitePrecision.scala).
- Explicit FMA has a dedicated single-new-rounding propagation case in
  [`RoundoffEvaluators.scala`](https://github.com/malyzajko/daisy/blob/38a0f33915dde03eeadd34786a920e834c1d9110/src/main/scala/daisy/tools/RoundoffEvaluators.scala).
- The repository is active: `master` was updated on 2026-06-17. The tested
  revision is its 2026-04-29 parent line. Current `master` adds the finalized
  Java foreign-memory API, did not compile with the installed JDK 8/17, and
  hit a Scala `Ordering`/Java `Comparator` conflict with JDK 26. We did not
  install another JDK merely to hide this integration constraint.

### FPTaylor

- FPTaylor's [reference manual](https://github.com/soarlab/FPTaylor/blob/b5a77cae348400f21f83512210d9f43c4bffb381/REFERENCE.md)
  admits bounded variables, rational constraints, explicit rounding operators,
  and rigorous absolute/relative roundoff analysis.
- Its `fma(a,b,c)` syntax is deprecated and now means the real expression
  `a*b+c`; a required fused f32 operation would need translation as one
  explicit rounding around the real multiply-add, not use of that deprecated
  name.
- FPTaylor can record HOL Light certificates, but the current configuration
  labels recording deprecated. Its
  [formal setup](https://github.com/soarlab/FPTaylor/blob/b5a77cae348400f21f83512210d9f43c4bffb381/formal/README.md)
  requires HOL Light plus a nonlinear-inequality verifier, documents up to two
  hours to load theories, and says some benchmark checking takes hours. The
  checker also excludes FPTaylor's advanced power-of-two rounding model.
- The host had neither OCaml nor opam. FPTaylor therefore was not executed;
  no host toolchain was installed for this spike.

These facts favor Daisy for an initial trusted-analyzer integration and retain
FPTaylor/HOL Light as a later independent-certificate investigation.

## Reproducible bounded probe

The executable corpus is under
[`spikes/numerics/sound_accuracy`](../../../spikes/numerics/sound_accuracy/).
It contains:

- scalar source covering ordinary arithmetic, square root, explicit FMA,
  equality-constrained division, f16 materialization, and two four-term
  reduction topologies;
- separate overflow and gradual-subnormal cases;
- a mixed-precision assignment that makes the f16 boundary explicit;
- a runner pinned to Daisy revision
  `38a0f33915dde03eeadd34786a920e834c1d9110`;
- standard-library-only adversarial observations using 100-digit `Decimal`
  references; and
- stable unsupported reason examples.

Daisy was cloned with `gwc`, built repository-locally, and invoked with:

```sh
PATH="/path/to/java8/bin:/opt/homebrew/bin:/usr/bin:/bin" \
  spikes/numerics/sound_accuracy/run_daisy.sh /path/to/daisy
python3 spikes/numerics/sound_accuracy/observe.py
```

The generated Daisy runner requires its checkout as the working directory,
because its frontend resolves the bundled language library relative to that
directory. The spike runner enforces that requirement.

## Measurements

Host: arm64 macOS 27.0 build 26A5378n, Daisy
`38a0f33915dde03eeadd34786a920e834c1d9110`, Z3 4.16.0. Timings are single
observations and not performance distributions. The exact recorded values are
in [`measurements.json`](../../../spikes/numerics/sound_accuracy/measurements.json).

| Region/profile | Certified absolute bound | Adversarial observed max | Analysis / total time |
| --- | ---: | ---: | ---: |
| f32 multiply then add | 1.1324882649432766e-6 | 5.9604644775390625e-8 | included in 16 / 1012 ms batch |
| cancellation | 1.0000000596046448 | 1 | included in batch |
| `sqrt(x) / y` | 5.960465330190516e-7 | 4.457063018918261e-8 | included in batch |
| explicit f32 FMA | 8.940696858417141e-7 | 1.1920928955078125e-7 | included in batch |
| explicit f16 materialization | 4.88817720906809e-4 | 4.8828125e-4 | 8 / 1033 ms |
| four-term left reduction | 12.000001072883606 | 1 | included in batch |
| four-term tree reduction | 16.000001072883606 | 2 | included in batch |
| `x / y`, `x == y`, independent ranges | 4.768372292574121e-7 | 0 | 13 / 1171 ms |
| same, Z3 relational ranges | 4.172325844820215e-7 | 0 | 192 / 1350 ms |
| gradual-subnormal add | 2.1019476964872256e-45 | not sampled | 8 / 914 ms |

Interval subdivision reduced the `sqrt`/division bound to
5.456627528741025e-7, while raising the whole batch's analysis time from 16 ms
to 320 ms and total time from 1012 ms to 1451 ms. Other measured bounds were
unchanged. The relational SMT profile tightened the ratio's real range from
`[0.5, 2]` to `[0.998046875, 1.00390625]`; it did not prove the exact singleton
range despite the equality assumption.

The empirical maxima are lower-bound witnesses over small adversarial sets,
not estimates of worst-case error. In particular, the Daisy profile includes
rounding arbitrary real inputs into f32, while the sampled inputs are already
exact f32 values. Bound/observation ratios therefore describe this corpus only
and must not be presented as proof tightness over the full domain.

Potential overflow produced an `OverflowException` diagnostic but Daisy still
returned process status zero. A Tiler adapter must parse a complete structured
result and reject missing results or diagnostics; process success alone is not
evidence.

## Exact initial adapter boundary

### Admission

Admit a candidate only when all of the following are known:

1. fixed scalarization size within a configured operation/input budget;
2. branch-free pure expression DAG;
3. f16, f32, or f64 round-to-nearest-ties-to-even values and conversions;
4. operations from `+`, `-`, `*`, `/`, `sqrt`, or required FMA;
5. finite exact-rational input intervals and a finite reduction contributor
   count;
6. exact reduction topology, contraction points, and materialization casts;
7. gradual subnormal semantics matching the analyzer model;
8. no possible NaN, infinity, invalid square root, division by zero, or
   overflow under analyzer-proved ranges; and
9. every relational assumption either compiler-proved or transactionally
   runtime-validated before plan selection commits.

Anything else returns a stable `Unknown` reason, including
`unsupported_control_flow`, `unbounded_scalarization`,
`unsupported_operation`, `unsupported_rounding`,
`unsupported_subnormal_semantics`, `unproved_finite_range`,
`unproved_domain_precondition`, `unvalidated_assumption`,
`analyzer_timeout`, `analyzer_diagnostic`, `missing_result`, and
`missing_implementation_error_profile`.

Transcendentals other than the specifically modeled real square root remain
outside the profile. A proof about ideal `sin` or `exp` does not bound a Metal
or CUDA intrinsic without a target implementation-error contract.

### Translation

- Emit exact rational bounds and the complete expression without algebraic
  simplification.
- Preserve let-bound sharing and operation order.
- Emit required FMA as Daisy's dedicated `fma`, while leaving separate multiply
  and add separate.
- Encode each semantic precision boundary in Daisy's mixed-precision map.
- Scalarize a fixed reduction into the candidate's exact tree; never translate
  a reduction as an unordered sum.
- Enable Z3 constraints only when the evidence identity names that analyzer
  profile and all assumptions are authorized.

### Evidence identity and checking

The adapter wraps the result in Tiler's `AccuracyEvidence`; Daisy's text is not
itself a portable certificate. The envelope must bind:

```text
schema + adapter version
+ canonical RegionAccuracyGoal digest
+ complete scheduled candidate/numerics digest
+ target numerical profile digest
+ exact assumptions and validation-provenance digest
+ scalarized analyzer-input digest
+ Daisy source revision and executable digest
+ complete analysis flags, budgets, timeout, Java/Z3 identities
+ returned exact bound/result and normalized diagnostic set
```

The result checker validates every binding, rejects duplicate/missing function
results, rejects any analyzer diagnostic or timeout, parses the bound without a
host-float round trip, and verifies `bound <= exact goal tolerance`. It does
**not** independently prove Daisy's analysis. Consequently this evidence class
is `SoundProof` only under an explicitly trusted Daisy profile; otherwise it is
`Unknown`.

## Decision and follow-up

**Proposal:** retain the region-accuracy layer and implement the narrow adapter
interface, but keep delegated numerical freedoms disabled until the adapter,
identity envelope, timeout isolation, result parser, and negative corpus exist.
Run proofs only after ordinary semantic legality and candidate construction,
cache them by the complete evidence identity, and cap them as a search resource.

**Inference:** analyzer startup dominates these small regions, so batching
several named expressions into one pinned invocation or maintaining a governed
worker could materially reduce cost. Either choice is an implementation and
isolation decision; it must not weaken per-candidate identity or failure
attribution.

**Follow-up:** separately spike FPTaylor certificate generation plus HOL Light
checking in a hermetic environment before preferring its smaller trust boundary.
Measure certificate size and checker latency on the same corpus, and verify the
rounding model actually covers casts, required FMA, and gradual subnormals.
