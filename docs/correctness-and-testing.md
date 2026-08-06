---
schema: "tiler-doc/v1"
id: "tiler.contract.correctness-and-testing"
kind: "contract"
title: "Correctness and testing"
topics: ["correctness", "testing", "verification"]
contract_status: "accepted"
implementation_status: "partial"
evidence: ["tiler.research.numerics.operation-conformance-matrix", "tiler.research.numerics.region-accuracy-contract", "tiler.research.numerics.sound-region-analyzer-spike", "tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]
ticket: "prototype-optimizer-conformance-gate"
---

# Correctness and testing

**Status:** accepted contract; bounded semantic/reference/index gates implemented, plus the lowering-capability and index-refinement gates on the ordinary compile path

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

The current reference evaluator covers the admitted semantic profile. The
generic slow evaluator for checked `IndexRegion` values and N-state reductions
**is implemented**: `tiler_reference::oracle` evaluates a `VerifiedIndexRegion`
through its registered scalar capabilities and returns the ordered outputs
beside the scalar-authority evidence it evaluated under, which
[`prototype-index-region-reference-oracle`](../tickets/prototype-index-region-reference-oracle.md)
delivered. The compiler's graph-specific proof is therefore no longer the only
evidence that registered lowering agrees with semantic meaning.

**Its independence from the structural verifier is the property that makes it
evidence.** The verifier decides whether a region is well formed; the oracle
computes what the region *means*, through the same registered capabilities a
backend would resolve. They do not share an arithmetic implementation, and
sharing one would make agreement between them vacuous — the failure mode this
section already names for a shared evaluator/lowering bug, one layer down.

An evaluation names **one** authority. A scalar registry is frozen against a
semantic registry and carries it, so `IndexRegionAuthority` takes the scalar
authority alone and derives the semantic one from it. Accepting both would let a
caller name a semantic authority the scalar authority was never registered
under — two authorities governing one evaluation with nothing comparing them.

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
| Lowering capability | Exactly one resolved capability per recognized occurrence; absent and contended resolution as distinct fail-closed dispositions; recorded provenance re-derived from the installed registry |
| Index refinement and semantic discharge | Candidate-blind construction of the exact canonical region from the exact program registry's immutable typed law, equality with the lowering's emitted region, ordered value-interface realization, reached scalar authority contained in the capability's declaration, agreeing semantic type authority, complete unique-write ownership, proof-budget exhaustion retained as an exact unknown gap, every residual assessed once by IR's closed exact-finite algorithm, only IR-sealed exhaustive proof receipts authorizing residual refinement, and disproved or unsupported unknown claims refusing before cover enumeration |
| Index | Rank, integer types, bounds, overflow, divisors, writer coverage, runtime parameters |
| Schedule | Observational coverage, safe redundancy/tails, resources, convergence, numerical contract, capabilities |
| Kernel | Scope/dominance, types/effects, access modes/address spaces, schedule-refined bounds and ownership, barrier/collective convergence and fences, reduction/order and launch references |
| Program/buffer | Semantic coverage, dependencies, boundary contracts, placement, initialization, allocation/lifetime/alias rules |
| Artifact | Symbols, ABI, hashes, target, launch metadata, guard completeness |

Verification is mandatory during expansion-time generation. Debug APIs may expose
additional expensive proofs, but core safety checks are never optional.

The table is a required gate inventory, not a claim that every row is already implemented. The optimizer conformance owner had to exercise an external operation through the ordinary capability/refinement path, non-isomorphic and fan-out or multi-output graphs, deterministic typed explain records, and identity/provenance assertions at every implemented layer before the public compiler facade was accepted — and that precondition was discharged on 2026-08-05, when Tom accepted the facade at the live decision review with one named exclusion, on the evidence the paragraphs below carry. The exclusion was `session::compile_governed`, held back until [`widen-compile-governed-s-error-to-the-target-compile-failure`](../tickets/widen-compile-governed-s-error-to-the-target-compile-failure.md) stopped it silently discarding the typed refusal detail the general path retains; that widening landed and Tom accepted the returned delta on 2026-08-06, so the facade is accepted in full.

**Evidence.** The `pipeline::conformance` module is that gate, and the capability/refinement half of it is now produced rather than owed. `an_externally_registered_lowering_provider_drives_the_compile_path` composes a lowering-capability registry entirely through the public capability surface — substituting an out-of-crate provider for one governed family and keeping the others — drives it through the compiler's ordinary entry point, and asserts both that every retained alternative's artifact plan records that provider and its capability revision as the occurrence's lowering authority and that the resolution record is attributed at the external provider's own revision rather than the governed one. `a_lowering_cannot_replace_the_semantic_providers_realization_law` deliberately supplies a structurally valid alternate multiply realization and observes fail-closed refusal before planning; `equal_semantic_snapshots_cannot_substitute_the_programs_law` separately proves that equal semantic snapshot identities do not let an installed add-law sidecar replace the exact program registry's multiply law. Sibling cases pin absent and contended capabilities. IR proof tests separately exercise exact finite proof, a deterministic counterexample, symbolic and over-budget `Unknown`, and invalid-budget rejection. The compiler supplies only the budget and retains IR-sealed proof objects rather than minting a second authority.

**Installation is reachable, and the out-of-crate half of it is now its own evidence.** This paragraph used to record the opposite — that the compilation request and its capability field were crate-private, so a registry composed outside the crate could only be installed from inside it. [`prototype-public-compiler-api`](../tickets/prototype-public-compiler-api.md) landed the reviewed `tiler_compiler::session` boundary and closed that gap: `session::InstalledCapabilities::installed` carries a caller's lowering registry with its `FrozenScalarRegistry`, `session::CompileRequest::with_capabilities` installs the pair, and request verification derives the realization-law snapshot from the exact program semantic registry while checking full lowering/scalar/program coherence before `session::compile` consumes it. The check is out-of-crate by construction rather than by assertion: `prototypes/serial-sum-compile` depends on `tiler-compiler` and sees only its public surface, `an_out_of_crate_caller_installs_its_own_capability_registry` composes and installs the registry there, and `an_installed_registry_missing_a_family_fails_closed` omits one family and observes the refusal.

**The multi-output row is now positive, and what the gate still does not cover is stated so it is not read as closed.** Ordered multi-output programs compile: `ordered_multi_output_programs_compile_through_the_ordinary_path` compiles a program declaring two independent ordered named outputs — an elementwise sum over `[2, 2]` and a strict serial reduction to `[2]` — through the ordinary entry point, in both declaration orders, and asserts a complete legal cover, one implementation selected per region, and a published interface whose keys follow the caller's declaration order with each key carrying its own producing occurrence's domain. This paragraph asserted the opposite until 2026-08-05, and both `output_count() != 1` guards were relaxed in one change: `select_supported_strategy`'s `output-arity` and `verify_artifact_refinements`'s `semantic-output-coverage` arity check, the latter widened into an ordered per-output key comparison rather than deleted. The wall had been traced to three different layers before it fell, and each tracing was right when it was made. `tiler-ir` was never the wall: `KernelProgramBuilder::push_output` is bounded by `MAX_PROGRAM_OUTPUTS` rather than by one, `KernelProgramDiagnostic::MissingNamedOutput` already rejects a plan naming fewer outputs than the program declares, and `program::tests::storage_reuse_is_admitted_only_with_an_explicit_handoff` builds and verifies a two-output program. The compiler's *planner* was the wall next, and [`implement-general-dag-partitioning`](../tickets/implement-general-dag-partitioning.md), [`assemble-a-kernel-program-from-an-arbitrary-cover`](../tickets/assemble-a-kernel-program-from-an-arbitrary-cover.md), and [`carry-artifact-program-output-order-into-kernel-program-identity`](../tickets/carry-artifact-program-output-order-into-kernel-program-identity.md) removed it. The request-boundary *recognition* was the wall after that, and [`recognize-several-ordered-named-outputs-at-the-compiler-request-boundary`](../tickets/recognize-several-ordered-named-outputs-at-the-compiler-request-boundary.md) replaced the single walk from `outputs().next()` with one walk per declared output. **Measurement (2026-08-05, at `23a3562d`):** with both guards relaxed and nothing else changed, an independent two-output program reached `phase: "program-assembly", rule: "cover-named-output-attribution"` — every stage through complete-plan selection succeeded and only the pairing of declared outputs to publishing regions was missing. Supplying it is what [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](../tickets/admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) landed: `CoverRegion` now carries the named-result value ordinals its candidate retains, `verify_cover` checks that projection against the authoritative candidate, and `CoverAssembly::from_plan` attributes each declared output by value. Reverting that attribution to the execution-order pairing it replaced makes the gate row fail with `InvalidCompilerOutput(StageElementCount)`, which is the measurement that it is load-bearing rather than defensive. **What the row still does not cover.** Two declared outputs whose recognition walks share an occurrence refuse under `output-partition-overlap` — two keys naming one value, and the published-and-consumed intermediate `a_published_and_consumed_intermediate_refuses_by_name` states — because one region's owning write would have to serve both a materialization edge and a publication; [`admit-elementwise-epilogues-over-a-materialized-intermediate`](../tickets/admit-elementwise-epilogues-over-a-materialized-intermediate.md) owns the copy stage that lifts it. Two independent outputs reading *disjoint* subsets of the declared inputs refuse under `elementwise-reads`, which multi-output admission made reachable for the first time and [`admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`](../tickets/admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs.md) owns. `crates/tiler-compiler/tests/multi_output_boundary.rs` is the executable form of this paragraph, including the ordering obligation at the semantic layer. The admitted program subject remains bounded — `f32` throughout, and each declared output produced by a strict serial sum, a strict tensor contraction, or an expression the general elementwise walk can spell — so this gate reports what an installed authority does inside that subject and nothing about a wider one; neither input nor output cardinality is part of that bound any more, because [`admit-multi-input-elementwise-programs-at-the-compiler-boundary`](../tickets/admit-multi-input-elementwise-programs-at-the-compiler-boundary.md) and this ticket landed. Scalar-lowering providers register and resolve but no compile stage resolves that family, so no installation evidence exists for it. And the session facade is accepted in full: `session::compile`, `CompileRequest` and its installation methods, `InstalledCapabilities`, `Compilation` and its accessors, and the `CompileFailureClass` vocabulary on 2026-08-05, and `session::compile_governed` on 2026-08-06 as the returned delta once its error type widened to `Result<Compilation, TargetCompileFailure>` (the packet and both provenance records live in [`accept-the-public-compiler-facade-boundary`](../tickets/accept-the-public-compiler-facade-boundary.md)); acceptance is still not stabilization — the surface is accepted pre-alpha vocabulary, not a published API with compatibility obligations.

**A per-dtype numerical contract is evidenced by refusal, and the refusal has to be watched failing.** `crates/tiler-compiler/tests/bf16_numerical_contract.rs` states a pure-BF16 constant/multiply/add program under `NumericalContract::STRICT_BF16` against a profile declaring the measured sign-preserving BF16 flush, and asserts the whole refusal rather than its variant: the subject is `ArithmeticType::Bf16` paired with `tiler::bf16@1`, the requirement is the caller's `SubnormalMode::Preserve`, the disposition is the profile's own `DeclaredUnhonourable` rather than silence, the declared means is `Unsupported`, and the behaviour reported as honoured is the measured flush. Two perturbations establish that those assertions can say no: substituting the `f32` subject for the contract's own makes the *preserving* `f32` row answer the BF16 question and the refusal disappears entirely, and rendering the BF16 key under the `f32` domain collapses two contracts onto one identity. Both were run and reverted.

**What that evidence bounds.** It proves the compiler boundary answers correctly for the measured behaviour; it does not bind the answer to the authoritative ledger's own rows, because `FIRST_MACOS_APPLE9` lives in `tiler-build`, which depends on the compiler and cannot be reached from its tests. It also does not imply BF16 execution: the flush-accepting request clears numerical feasibility and is then refused by the recognizer's `dtype-f32` rule, which the same file asserts so a positive numerical answer cannot be read as support. And the ledger's BF16 rows currently cover the two subnormal dimensions only, so a flush-accepting contract meets `Unknown` on the first undeclared consumable dimension — recorded as its own case, so widening those rows changes a test rather than passing silently.

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
- strict affine U4 and U8 Quantize producing both ordered residual predicates for runtime-unknown expressed value and scale; exact governed constants independently proving each predicate with a sealed typed proof basis; a provider merely self-identifying as the standard provider remaining unable to mint proof; zero, negative, ±infinite, qNaN, and sNaN scales rejecting; ±infinite expressed values and the smallest positive subnormal scale succeeding; simultaneous disproof using deterministic stable-code/ordinal priority; malformed typed result contracts taking diagnostic precedence over input disproof; and no operation, result, or canonical-work mutation after rejection;
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

**A corpus that reports uniform agreement has usually failed to ask the question.** Two formulas that a reader would call the same operation typically agree on almost every input and differ in a narrow band, so an unrestricted agreement rate mostly measures how often the corpus avoided the boundary. The [transformer non-linear derivation](research/numerics/transformer-nonlinear-normalization-and-reductions.md) produced both failure modes while separating lookalikes: two SiLU spellings reported identical over a corpus with no input near the exponential's overflow threshold and one ULP apart once one was added, and a softmax's divide-versus-reciprocal question was decidable only after the count was restricted to elements where the two forms actually differ and stratified by a row width narrow enough that the denominator's own accumulation order could not contribute. So a differential corpus separating two candidate contracts states which inputs discriminate and counts only those, and a comparison that cannot name a discriminating input has not yet established agreement.

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

**Under a contract that permits reassociation, the oracle is the strategy's own declared grouping, and it stays bitwise.** A reassociating contract admits a *set* of results, so a serial-fold reference is the wrong question — disagreement with it is the expected outcome for a legally regrouped strategy, and comparing against it would refuse a correct implementation for being correct. The answer is not to widen the comparison but to narrow the question: a physical plan does not pick from the permitted set at run time, it *declares* one grouping, so what it is checked against is the exact value that grouping produces. `tiler_reference::strict_partitioned_sum` is that second exact oracle and its own documentation carries the derivation. This sharpens rather than contradicts the "nondeterministic reductions" sentence in [Semantic authority](#semantic-authority): the implemented strategies are deterministic, so neither repeated runs nor a result-set membership check is needed, and membership would in any case be too weak — it accepts a strategy that produced some *other* legal grouping than the one it declared, which is precisely the defect a declared-grouping oracle exists to catch. A tolerance is weaker still and remains refused: "within a bound" cannot separate a strategy that grouped as it declared from one that did not, and this document's standing position is that a difference is attributed to a named cause or it is a defect.

**Measurement, and the boundary it does not exceed.** On the qualified host — Apple M4 Max, macOS 27.0 build `26A5388g`, `arm64`, Apple9, offline compiler `Apple metal version 32023.883`, toolchain `nightly-2026-07-19` — a `1x4` reduction under `FLUSH_AND_REASSOCIATE_F32` was driven through `prototypes/serial-sum-run` on operands `0x3f400000, 0x3e800000, 0x33400000, 0x33000000`, chosen so the declared groupings differ by exactly one rounding step. The serial fold returned `0x3f800000` and both parallel strategies returned `0x3f800001`, each matching its own declared grouping bit for bit; the difference is attributed to `governed_partition`'s two-by-two blocked split rather than to a tolerance. This is the first observation in the corpus of a reassociation-permitting program producing a different-but-permitted answer from the serial fold. It does **not** generalize past four contributors, past one row, past this contract, or past this host, and it says nothing about which strategy is faster — and note what it cannot separate: at four contributors both parallel strategies declare the *same* partition, so the case discriminates the parallel strategies from the serial fold and not the tree from the split. Discriminating those two needs a contributor count at which their declared partitions differ, and no such count exists on this profile. That used to be attributed to the shape's grid-axis bound, which admitted four contributors and nothing wider; the bound is now a measured 268,435,456 and widening it changed nothing, because `single_workgroup_tree_region` and `split_reduction_regions` both take their partition from one `governed_partition(contributors)` and `workgroup_tree_tile` fixes `rounds: 1`, which makes the two declared groupings identical at *every* count. [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](../tickets/separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md) holds that evidence gap, and its one surviving activation trigger is a cooperative tile whose rounds exceed one rather than a wider shape.

**A grouping-sensitive operand set does not subsume an exact one.** The two answer different questions and neither replaces the other, which is why both run. Over four contributors there are five order-preserving groupings; on operands whose every grouping is exact all five produce one value, so the comparison has an empty refusal population among legal answers and cannot observe rounding at all — but its subset sums are distinct, so no dropped or double-counted contributor escapes it. The grouping-sensitive set inverts both properties: it separates the groupings, and it leaves one of the sixteen single-contributor corruptions undetected. Both counts are pinned by a device-free case beside the operands, so a later edit that dropped either set would have to change them.

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
- Semantic-precondition tests distinguish declaration identity from occurrence obligation identity. They cover predicate and invalid-code changes, declaration bounds and duplicates, the explicit logical-view tag, exact operand selectors, shape, complete resolved type, canonical occurrence and subject coordinates after compaction, dead-operation removal, insertion-order independence, provider-revision provenance without obligation-identity churn, the aggregate cached-identity byte boundary and one-over refusal, and retained proved/residual records with typed proof basis.
- Reached semantic-authority closure tests cover nested parameterized and
  encoded components, occurrence `Type` and `FloatBits` references, operation
  defaults/facts/conformance, missing definition references, finite cycles,
  root-iterator and unique-closure resource exhaustion at the first item beyond
  each governed limit, and no polling of a root iterator's remaining tail.
  Program-level tests vary providers reached only through composite value types
  and occurrence attributes. Used provider revisions change admission and
  snapshot subjects; unused revisions change only the full snapshot subject.
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
  differently. The guard and the ABI are proven at the kernel-program layer and
  not only at the artifact layer, because both are folded into complete program
  identity; two programs differing only in their applicability guard, only in
  the *expression* computing an accessible byte range, or only in whether
  fallback is permitted before commit must produce different identity bytes.
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

- An inline invocation cold-compiles and embeds a loadable artifact envelope
  without consumer `build.rs` or a prebuild command.
- Equivalent warm expansions perform no *compilation* work — neither `metal` nor `metallib` runs, and nothing is published — including across rustc processes. Resolving the Apple toolchain is not compilation work — it is identity work every expansion pays, warm or cold, for the structural reason stated after this list.
- The artifact envelope carrying the manifest and every built family's metallib is emitted as exactly one byte-string literal token; generated Rust contains no compiler-cache path or `OUT_DIR` dependency.
- Cache deletion, `cargo clean`, incremental compilation, compiler upgrades,
  toolchain changes, lock contention, and stale-lock recovery are safe.
- rust-analyzer and `cargo check` cold/warm behavior is measured and preserves
  the same types and diagnostics as normal compilation.
- Bundle sizes of roughly 10 KiB, 100 KiB, and 1 MiB have explicit rustc
  time/memory and binary-size measurements.
- Many identical invocations establish whether linker constant merging occurs;
  correctness never depends on it.
- Generated consumer-`cfg` tests cover macOS, iOS device, iOS simulator, Catalyst, and an unrelated non-Apple target. The envelope's bytes are embedded once and unconditionally, so what a selected matching family's `#[cfg]` decides is its payload's *position* within them — or, when that family did not build, its retained actionable compile error. A nonmatching target selects no position and compiles the semantic fallback; `FallbackOnly` performs no backend compiler work.
- A capable macOS host's selected-family work while compiling an unrelated
  target is measured; the content cache bounds it, and correctness never
  depends on proc-macro consumer-target discovery.
- External Metal errors preserve invocation spans and retained canonical MSL.

**Why the warm requirement states compilation rather than `xcrun`.** [Frontend and proc-macro integration](integration/frontends.md) corrected that bullet's frontend counterpart on 2026-08-01 and carries the derivation, the two eliminated alternatives, and the measured cost; only the consequence for this list is restated here. The compiler fingerprint is an *input* to compilation identity, so the toolchain must be observed before a lookup exists to skip — an entry reached without observing it would be keyed on something other than the compiler that would build a miss, the incomplete-key failure [ADR 0050](decisions/0050-use-immutable-self-validating-expansion-cache-entries.md) exists to exclude. The invariant that replaces "avoid `xcrun`" is narrower and stronger: identity folds a fingerprint read by executing the binaries the same prepared token will execute. A resolution on every expansion is therefore structural, and a test demanding its absence would be demanding a broken key.

**Evidence.** `a_checked_plan_publishes_then_hits_without_recompiling` (`tiler-build`) is the direct assertion: a launcher shim whose fake `metal` and `metallib` append one line per *compile* invocation and none for a `--version` probe, three resolutions of one subject, and exactly two lines required — so the two hits ran the fingerprint probes and no compilation. `the_second_expansion_of_one_subject_compiles_nothing` (`tiler-macros`) makes the same claim against the real Apple toolchain by asserting the resolutions are `["published", "hit"]`, which a design computing identity after compiling could not produce. `concurrent_processes_on_one_key_compile_once` (`tiler-cache`) is the cross-process half: racing OS processes are held at a barrier until every one has looked and found nothing, and one compilation is then the lock excluding something rather than the scheduling. The one-envelope requirements above are pinned by `one_built_family_emits_its_gated_selector_and_a_total_catch_all` and `the_emitted_arms_select_exactly_one_payload_per_consumer_target`, the second of which evaluates the emitted predicates against `rustc`'s own `cfg` answer for exactly the five targets that bullet names.

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
