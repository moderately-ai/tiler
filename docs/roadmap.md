---
schema: "tiler-doc/v1"
id: "tiler.roadmap"
kind: "roadmap"
title: "Roadmap"
topics: ["roadmap", "implementation"]
roadmap_status: "proposed"
---

# Roadmap

**Status:** proposed

The roadmap favors narrow end-to-end slices over implementing a broad IR with
no verified runtime contract.

The [operation-family support matrix](#operation-family-support-matrix) below records how narrow the operation surface currently is, family by family, so breadth stays a tracked position rather than an accidental one.

ADR 0055 authorizes a bounded, unstable implementation prototype whose first
Metal value proof fuses a resolved `f32` pointwise prologue into a strict serial
`f32` `Sum`. ADR 0065 fixes the current prototype crate layout, and ADR 0067
supersedes the Rust 1.89 floor with the exact `nightly-2026-07-19` toolchain for
dependent static-shape evidence. Broader work below remains proposed
progression rather than blanket implementation authority.

## Milestone 0A: semantic graph and extension feasibility

**Research-contract status:** complete. ADRs 0005, 0006, 0008, 0044, and 0045
fix the graph, shape, registry, and proc-macro visibility boundaries. The
semantic type, operation, typed-handle, and reference boundaries are now
compile-checked, and the retained dependent-array conformance harness passes.
The checked shaped-value layer is implemented with compile-fail and identity
coverage, and the assembled semantic/reference slice passes through a
downstream-style public construction and evaluation test. The bounded
Milestone 0A integration proof is complete; production stabilization and the
broader operation/dtype surface remain future work.

ADR 0072 corrects the prototype identity boundary before generic compilation:
graph meaning, reached provider-independent definitions, admission-provider
provenance, and the full registry environment now have independent identities.
The later region/index/schedule/KIR/program/artifact tickets must preserve that
layering rather than nesting whole-program or provider identity into reusable
structural content.

The bullets below are the implementation scope authorized only after the
research-readiness decision; they are not claims that the implementations
already exist.

- Define axes, reindexing, broadcasting, dtype, reduction, empty-domain,
  overflow, alias, and numerical policies.
- Define operation conformance vectors and oracle precedence.
- Implement the public semantic operation/value graph and deterministic graph-
  meaning identity needed for executable examples, separately retaining
  reached definitions and provider-attributed admission provenance.
- Implement and exercise the public experimental extension path through the ordinary compiler
  API with one built-in and one statically linked external operation definition
  using the same capability interfaces. Separately record which providers the
  proposed proc macro can see across its compilation boundary.
- Implement an explicit per-session registry and continuously test collision rejection,
  canonical attribute encoding, and separation of semantic keys from provider
  revisions.
- Define the consumer-independent `CompilationRequest`, scoped shape symbols,
  and sourceability of every dynamic output/temporary/guard/launch expression.
- Establish multiple named results, sharing, and multi-result representation
  invariants even if the first runtime profile executes a narrower subset.
- Review and integrate ADR 0067's implemented arbitrary-rank
  `StaticShape<RANK, EXTENTS>` evidence family; reuse the retained conformance
  harness for every compiler-pin migration.

**Exit criterion:** tensor meaning and graph invariants have a reviewed
contract, mandatory operation-extension capabilities are explicit, and a small
semantic graph verifies and evaluates without any frontend, backend, or runtime
dependency. Registry, canonical-data, semantic/provider identity, and dynamic
shape-source invariants are tested.

## Milestone 0B: Rust/Metal integration vertical feasibility

**Research-contract status:** complete. ADRs 0002–0004, 0049–0053 and the
artifact/cache/runtime spikes fix the AOT, inline-DX, family-selection,
publication, and fallback boundaries. The actual Tiler macro-to-dispatch
vertical remains implementation work.

The bullets below are remaining vertical implementation and integration checks,
not completed production capabilities.

- Build a proc-macro spike that compiles fixed deterministic MSL with `xcrun`
  and emits manifest/metallib byte-string literals without consumer setup.
- Implement the accepted immutable self-validating content-addressed cache and
  reproduce the completed process-level crash/race harness against it.
- Retain the completed embedding, Cargo freshness, cache deletion, and Apple
  family/toolchain probes. Measure rust-analyzer cold/warm behavior when the
  component is available, plus the actual native macOS and non-Apple fallback
  paths.

**Exit criterion:** cold inline macro AOT produces a loadable bundle, warm
equivalent expansions invoke no external compiler, and the proposed Rust DX
works without build scripts or prebuild commands. Failure does not invalidate
Milestone 0A's consumer-independent compiler boundary.

The currently authorized Metal AOT and runtime tickets prove backend artifact
and device-execution boundaries but intentionally exclude the proc macro,
generalized cache, and consumer integration. They are prerequisites and
evidence for this milestone, not its complete exit.

The live ticket graph deliberately gates those proofs on a backend-consumable
target-neutral compiler path. That path lowers a verified semantic program
through semantic normalization, generic fusion-region formation and legality,
checked semantic-to-index refinement, region covers, target feasibility and
scheduling, physical-implementation planning and complete-plan selection,
structured kernel IR, and artifact-facing programs, closed by the
[optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md)
and the reviewed [public compiler API](../tickets/prototype-public-compiler-api.md)
that the inline frontend consumes. These are independent authorities with real
dependencies, not a single linear pipeline, and their dependency-satisfied
ordering shifts as work lands. `tkt rollup` and `tkt ready` — not a chain
enumerated here — report which authorities are complete, dispatchable, or
blocked.

Opaque physical calls are not part of this bounded compiler path. Their reviewed
[provider ticket](../tickets/implement-opaque-physical-call-providers.md) is
deferred behind the optimizer conformance gate and the mature boundary-property
and analytical-cost authorities.

Metal is then split into independently verifiable KIR lowering, strict
numerical realization, Apple offline compilation, bundle assembly, and
proof-evidence work before the existing Metal AOT integration ticket. The
neutral artifact codec half of that split has landed, behind an unaccepted
crate-private facade; filling its carried-payload shape from a real emission
and a real compilation has not. Runtime validation, preflight, one-way routing commit, and execution
mechanics similarly precede the device integration proof. The inline proc
macro, cache, artifact-family delivery, embedding measurements, complete inline
workflow, and Candle adapter remain explicit downstream tickets rather than
implicit promises of the Metal proof.

## Milestone 1: canonical semantic graph and index IR

- Build the typed operation/value semantic graph.
- Lower output coordinates through composed reindexes into access maps.
- Add symbolic extents, strides, and offsets.
- Implement semantic/index verifiers in `tiler-ir` and the slow executable
  oracle in downstream `tiler-reference`.
- Canonically serialize and reference-evaluate every enabled transcendental
  accuracy contract before admitting such an operation to the vertical slice.
- Establish randomized differential testing against normative semantics and
  independent compatibility cases; Candle cases belong to the first
  integration suite.
- Add deterministic serialization, hashing, and textual `EXPLAIN`.
- Add the conservative one-allocation-per-output/temporary `BufferPlan` and
  single-device, single-ordered-stream `KernelProgram` verifier.
- Implement constant folding, index CSE, and conservative dimension coalescing.

**Exit criterion:** programs within the implemented view/map normalization
theory produce verified canonical access maps independent of transient IDs and
construction order.

## Milestone 2: conservative Metal vertical slice

- One input, one newly allocated output, F32, statically known rank.
- Contiguous layout with arbitrary valid start offset.
- Reindex plus pointwise fusion.
- Implement the accepted prototype strict serial `f32` `Sum` profile and
  compare one fused map/reduce dispatch with a deliberately materialized
  reference.
- Initially limit pointwise operations to fully resolved algebraic semantics;
  any transcendental or GELU enters only with its formula, reference evaluator,
  accuracy contract, and conformance evidence implemented end to end.
- Scalar one-thread-per-output and rank-aware schedules.
- Minimal conservative Metal target profile for correctness and launch limits.
- Deterministic MSL, one metallib bundle, and complete lockstep experimental
  manifest.
- Expansion-time `xcrun`, global content cache, and direct byte embedding.
- Candle custom-op adapter, per-device pipeline cache, and fallback.
- A trivial single-pipeline region builder; general memo/DAG planning is not
  implemented yet.

**Exit criterion:** a useful einops-derived chain executes correctly with fewer
dispatches or intermediates than the reference path.

The strict serial-Sum architectural proof exercises the core compiler and Metal
path before this broader milestone. It intentionally does not claim the Candle
adapter, inline macro, general fallback, or einops-derived workload required by
the milestone exit.

The target-neutral portion of that proof is now implemented as a private
graph-specific conformance fixture: the same verified request produces a
two-stage materialized program and a one-stage fused program, the fused
structured kernel preserves atomic multiply/add and strict contributor order,
and a fixed structural policy selects it while retaining the baseline. It does
not yet establish generic occurrence lowering, region enumeration, legality
derivation, complete partition search, a public compiler boundary, Metal
source, artifacts, or runtime execution. ADRs 0070 and 0071 now establish the
shared `tiler-ir` ownership and checked-builder/verified-wrapper lifecycle into
which the dependency-ordered implementation tickets lower. The ordinary
compiler library target is active, but proof-specific structures remain
private until replaced rather than being promoted as provisional public IR.

## Milestone 2Q: quantized-value vertical proof

- Verify and reference-evaluate strict affine `i4/u4/i8/u8` code tensors with
  `f32` expressed, scale, computation, and requantization-intermediate values.
- Cover per-tensor, per-axis, and per-block parameter maps with constant and
  runtime graph operands.
- Implement `AssembleQuantized`, `Quantize`, `Dequantize`, and `Requantize`
  contracts independently of physical packing.
- Lower at least one 8-bit path and one packed 4-bit block path, with complete
  component-role ABI and storage-encoding validation.
- Exercise proof-elided semantic validation; measure runtime enforcement
  separately rather than weakening strict semantics for an integration.

**Exit criterion:** logical code type, quantized interpretation, parameter map,
and packed storage remain independently verified while a representative 8-bit
and 4-bit program agree with the strict reference evaluator.

## Milestone 3: physical properties and alternatives

- Required/provided layout, alignment, vector width, and materialization.
- Scalar, vectorized, collapsed, and general-stride candidates.
- Explicit contiguous/layout enforcers.
- Bounded alternative search and first analytical cost model.
- Add richer device-family profiles and symbolic/guarded routing.
- Implement governed capability keys and all `CompileProfile`,
  `ArtifactEvidence`, `LiveDevicePreflight`, `PreparedKernelPreflight`, and
  `LaunchPreflight` fact phases, with aggregate
  `Proven`/`Deferred`/`Rejected`/`Unknown` feasibility and `RoutingCommit`.
- Keep hard resource proofs distinct from register, occupancy, cache, and
  throughput estimates; validate fixed and scalable vector legality.
- Typed explain data for rejection reasons and plan comparison, with
  deterministic text rendering as presentation.

**Exit criterion:** the optimizer chooses among several valid region
implementations and complete `KernelProgram`s and explains the choice.

## Milestone 4: reductions

- Broaden the exact serial baseline beyond any narrow Milestone 2 proof.
- SIMD-group and threadgroup strategies.
- Fused pointwise prologues and epilogues.
- Explicit accumulator and empty-domain policy.
- Ragged-tail and multi-SIMD-group coverage.
- Multi-pass fallback for large domains.

**Exit criterion:** at least one rearrange/map/reduce chain is safely fused and
outperforms or reduces traffic relative to the reference pipeline.

## Milestone 5: graph partitioning

- Candidate regions across DAGs.
- Costed fuse versus split decisions.
- Fan-out recompute versus materialize.
- Multi-output candidates.
- Live-value/register and intermediate-memory estimates.

**Exit criterion:** fusion is a costed global decision rather than a linear
pipeline heuristic.

## Milestone 6: einsum contractions

- Contraction-order exploration.
- GEMM recognition and library-call alternatives.
- Layout-conversion costing.
- Direct/tiled contractions and fusible epilogues.

**Exit criterion:** contraction planning uses the same properties, enforcers,
and cost framework rather than backend-specific exceptions.

### Framing: what a tensor-contraction family would impose

The four bullets above are physical-planning intent. They presuppose a semantic operation that does not exist, and this section frames what admitting one would fix. It is a framing, not a design: it authorizes nothing, commits to no einsum surface, registers no key, and does not schedule work. Its entry in the durable question index is [Q-SEM-015](open-questions.md), its rung in the [support matrix](#operation-family-support-matrix) is R1, and the two questions it cannot answer for itself are named at the end. Claims are labelled **Fact** when supported by inspected source or an accepted decision's own recorded status, **Inference** when derived from stated facts, and **Proposal** when they remain to be accepted.

**The word "contraction" carries two unrelated meanings in this corpus and they must never be read as one another.** ADR 0015's contraction is a *numerical permission*: whether a separately rounded multiply and add may fuse into a single rounding. It is what `NumericalRealization::contraction` in `tiler-ir`, `StrictF32NumericalContract::contraction` in `tiler-compiler`, and `MetalNumericalRequirement::NoFloatingPointContraction` (which emits `-ffp-contract=off`) in `tiler-metal` all mean. *Tensor contraction* is the operation family framed here: summation over indices shared by two or more operands, of which `matmul`, batched `matmul`, and general einsum are instances. Everything below means the tensor sense unless it names ADR 0015. The two senses meet at exactly one point, described under [Numerical contract](#numerical-contract) below, and that intersection is why the collision is a correctness hazard rather than a naming curiosity.

**Fact — the planning layer already anticipates a contraction; nothing fixes its meaning.** [Optimizer](compiler/optimizer.md) reserves "alternative associations of a future multi-input einsum contraction" as an equivalence group that no expressible numerical policy currently admits, and "direct or GEMM-backed contraction" as an implementation rule. [Fusion and scheduling](compiler/fusion-and-scheduling.md) sketches contraction-order choices, GEMM canonicalization, optimized library matmul, layout-conversion enforcers, batching and split-reduction strategies, and fusible prologues and epilogues, and states that contraction planning "should follow, not precede, the boundary-contract and cost infrastructure". [IR](ir.md) names contractions once, in the accepted numerical-typing rule that operations with intrinsic mixed precision "such as reductions and contractions, declare computation precision, accumulator/result types, and relevant order or algorithm contracts through their specialized semantics". [Shape environment contract](research/shapes/shape-environment-contract.md) uses `MatMul` as the worked example for two accepted decisions. No ADR, no normative contract section, and no registered operation definition says what a contraction *is*.

#### Identity

**Fact.** [IR](ir.md) models an operation as an `OpKey` (dialect, name, semantic version), ordered operands, canonical attributes, and ordered results; canonical identity uses that content and never arena handles. A contraction is therefore identified by whatever the chosen `OpKey` and attribute schema encode, and by nothing else.

**Inference — a contraction is not one operation in the way `Add` is one operation.** `matmul` over `[M, K] × [K, N]`, batched `matmul` over `[B, M, K] × [B, K, N]`, and an attention-shaped einsum `bhqk,bhkd->bhqd` differ in rank, in which axes are batched, in which are contracted, and in the output axis order. They are one family only if a canonical attribute carries that index structure. If instead each shape class takes its own key, the standard registry grows one governed key per rank-and-batching combination and semantic normalization must relate them; the registry admits exactly four operations today, so this is a real growth decision rather than a bookkeeping one.

**Fact — an index-notation attribute is not automatically canonical.** [IR](ir.md) fixes the attribute data model: `CanonicalValue::Utf8String` holds "exact valid UTF-8 bytes with no implicit Unicode normalization", `Sequence` order is semantic, and `Record` fields are sorted by unique ID. **Inference.** Storing an authored subscript string therefore makes `ij,jk->ik` and `ab,bc->ac` two different operations with different semantic graph identities, different artifact identities, and no cache reuse between them, even though they denote the same computation. ADR 0074's conventions bind the fix as well as the hazard: the encoder writes a versioned NUL-terminated `tiler.<subject>.v<N>` domain tag before any content, a fixed-width length before every variable-length run, excludes transient identifiers "wherever the represented semantics are equivalent without them", and matches every encoded enum exhaustively. An author's choice of index letters is exactly such a transient identifier. **Proposal.** The canonical attribute is a *structure* — which operand positions each index visits, which indices are free, which are summed, and the output index order — rather than the authored labelling, and the operation definition normalizes to that structure before storing or hashing, in the same way the schema already normalizes a field equal to its declared default.

**Fact — there is a precedent for where that normalization lives.** `tiler::strict-serial-sum-f32@1` carries its reduced axes as a required `Sequence` attribute, and `StrictSerialSumF32::infer` in `crates/tiler-ir/src/semantic/registry.rs` rejects an empty sequence, a non-`u32` element, an out-of-range axis, and any axis that is not strictly ascending and unique — as typed provider diagnostics, at construction. A contraction's index structure is the same kind of obligation at greater complexity.

#### Validation

**Fact.** Every registered operation validates its own operands. `BinaryF32::infer` accepts a rank-zero operand under the narrow scalar admission [IR](ir.md) owns, and otherwise requires `left == right` exactly. **Inference — a contraction is the first family whose operands are *required* to disagree.** `[M, K]` and `[K, N]` share one extent and differ everywhere else. Nothing in the admitted vocabulary expresses that today, because no admitted operation needs it.

**Fact — the shape layer already anticipated exactly this, using `MatMul` as its example.** Two accepted decisions in [Shape environment contract](research/shapes/shape-environment-contract.md) are stated over it. Equality does not erase source identity: with `K_left <- InputDimension(A, 1)` and `K_right <- InputDimension(B, 0)`, "MatMul contributes `SemanticRequirement(K_left == K_right)`" and both bindings survive, so a failure reports both observed sources. And validation is three-outcome: `MatMul([M, K1], [K2, N]) requires K1 == K2` resolves to proved (accept and retain the derived fact), disproved (reject during compilation with a typed diagnostic), or unresolved (emit a typed host-side pre-dispatch requirement evaluated after root extent binding and before allocation or device work). **Inference.** A contraction's extent agreement is therefore already an answered question; what remains unanswered is the *structural* well-formedness that precedes it.

**Proposal — the structural rejections a contraction spec owes at construction**, each independent of any extent value and none of them expressible as a shape comparison:

- an output index that appears in no operand, which has no source to read;
- an index summed over but appearing in only one operand, which is a reduction of that operand rather than a contraction and has a different meaning and a different access relation;
- an index repeated within a single operand, which is a diagonal or trace extraction — a many-to-one read inside one operand that [IR](ir.md) would require an explicit construct for, not an einsum convenience;
- a duplicated output index, or an output index order that is not a permutation of the free indices; and
- for a multi-operand form, an index appearing in more than two operands, whose admission is itself one of the reserved decisions below.

Extent agreement across every occurrence of one index is then the shape-requirement class above and resolves through the accepted three-outcome path.

#### Access relation

**Fact.** [IR](ir.md) Layer 2 defines an access map as `(output coordinates, reduction coordinates, shape/interface parameters) -> logical tensor coordinates`, and lists `IterationDomain` and `ReductionDomain` as separate core concepts. Reduction is a structural region form with ordered bound dimensions, ordered initial state, ordered contributor values, a checked nested scalar body, and ordered results; its first supported traversal is an exact lexicographic left fold.

**Inference — this is the reason a contraction is architecturally different from a pointwise family, and it is a sharper difference than "it reduces".** A pointwise family's access maps are functions of the output coordinates alone. The registered strict serial sum introduces a reduction domain, but it has one operand, so exactly one access map consumes it. A contraction introduces a reduction domain that *two or more operand access maps share, while each drops a different subset of the free coordinates*. For `C[m, n] = sum over k of A[m, k] * B[k, n]`, the iteration domain is `(m, n)`, the reduction domain is `(k)`, `A`'s map is `(m, n, k) -> (m, k)` and never mentions `n`, and `B`'s map is `(m, n, k) -> (k, n)` and never mentions `m`. No admitted operation produces two operand maps that project away different iteration coordinates.

**Inference — the index language already admits those maps; the gap is elsewhere.** [IR](ir.md) bounds the initial index vocabulary to addition and negation, multiplication by a parameter-only expression, and Euclidean floor division or modulo by a proven-positive parameter-only expression, and rejects iteration-by-iteration multiplication and tensor-data-derived indices. Each contraction operand map above is a pure projection and permutation of the coordinate vector, using no index arithmetic at all; the multiplication is between two *loaded values* in the reducer body, which is ordinary scalar computation and not index arithmetic. A contraction therefore needs no new access class, no piecewise map (Q-SHAPE-006), and no indirect relation (Q-SHAPE-007). What it needs is an operation capability that emits those maps, and the [preconditions](#preconditions) below are about the absence of any such capability rather than about the expressiveness of the index layer.

**Inference — expressing the same computation as broadcast, multiply, and reduce is a different semantic identity, not an alternative spelling.** Composing `Broadcast` to `[M, N, K]`, an elementwise `Multiply`, and a `Sum` over `K` denotes the same values, but it is a different canonical graph, and it is the physical planner rather than the semantic layer that must then discover the contraction inside it. Keeping the contraction atomic is what makes GEMM recognition an operation-specific rewrite over a named node — which is what [Optimizer](compiler/optimizer.md) already assumes when it proposes a "supported prologue/epilogue around a semantic operation with an opaque library implementation" as a region-candidate rule. Neither `Broadcast` nor a general `Reduce` is registered today, so the alternative spelling is not currently available either.

#### Lowering and physical planning

**Fact — the two implementation alternatives named in this milestone have different prerequisites and must not be scheduled as one item.** [Optimizer](compiler/optimizer.md) lists "direct or GEMM-backed contraction" among implementation rules, and both it and [Fusion and scheduling](compiler/fusion-and-scheduling.md) fix the mature `ImplementationFrontier` body as an additive sum type over `ScheduledKernel`, `KernelSubprogram`, `OpaqueCall`, and `View`, of which "the bounded P0 frontier admits only checked `ScheduledKernel` proposals and rejects opaque-call proposals explicitly". A direct or tiled contraction is a `ScheduledKernel`. An optimized library matmul is an `OpaqueCall`, and opaque physical calls are deferred behind [`implement-opaque-physical-call-providers`](../tickets/implement-opaque-physical-call-providers.md), which is itself gated on the optimizer conformance gate and the mature boundary-property and analytical-cost authorities. **Inference.** "Direct/tiled contractions and fusible epilogues" is reachable without opaque-call support; "GEMM recognition and library-call alternatives" is not.

**Fact — hard feasibility and estimated cost stay separate, and a library GEMM tests that boundary first.** [Optimizer](compiler/optimizer.md) requires each implementation candidate to advertise "a machine-checkable numerical guarantee, realization/provider identity, and scoped evidence", admitted "only when that guarantee refines every effective operation contract", and states that numerical conformance is checked *before* the dominance relation because "accuracy is a hard semantic dimension, not a Pareto cost". [IR](ir.md) states the feasibility half of the same discipline, where an "`Unknown` *feasibility* verdict keeps its candidate in explain and search state only" and "such a candidate cannot enter an executable `ImplementationFrontier` or manifest" — a rule IR scopes to that verdict and explicitly declines to generalize to every `Unknown` in the corpus. **Inference.** A vendor GEMM that does not publish its accumulation order, input precision, and contraction behaviour offers unknown numerical evidence, which the optimizer's own rule already makes inadmissible rather than merely expensive; IR's feasibility verdict is the parallel case under a different assessment, not that rule applied twice. Milestone 6's exit criterion — that contraction planning use the same properties, enforcers, and cost framework rather than backend-specific exceptions — is exactly the requirement that a library call not be granted an exemption here.

**Fact — layout conversion is already an enforcer, not a new mechanism.** [Optimizer](compiler/optimizer.md) lists contiguous materialization, layout conversion, and encoding repacking as enforcers that supply a missing required property at a cost, defines satisfaction, subsumption, child-requirement derivation, dominance, and cycle-checked insertion over boundary contracts, and retains interesting properties such as useful unit-stride axes on a bounded Pareto frontier even when they are not locally cheapest. Every entry is value-preserving; a dtype cast is a semantic operation and is deliberately not on that list. **Inference.** "Layout-conversion costing" is therefore the first family that exercises that machinery under real pressure rather than a separate subsystem, which is what [Fusion and scheduling](compiler/fusion-and-scheduling.md) means by requiring contraction planning to follow the boundary-contract and cost infrastructure.

**Fact — contraction-order exploration is a numerical permission question, not only a search question.** [Optimizer](compiler/optimizer.md) places the tensor-contraction association rewrite under logical exploration, where `ExploreLogicalAlternatives` "adds only proved contract-preserving forms", and admits it "only when the effective distributivity, reassociation, and operand-permutation permissions all authorize the regrouping". **Fact.** [Numerical semantics](numerical-semantics.md#distributivity-is-outside-the-order-contract) owns the derivation and states why all three are named: regrouping `(AB)C` into `A(BC)` forms different rounded products rather than regrouping one reduction's contributors, so the identity it consumes is distributivity — a third numerical dimension, independent of the two order-contract dimensions, for which that contract admits no permission. **Inference.** Reassociation is necessary and never sufficient. Each of the three demands both an operation capability declaring the algebraic property and an effective numerical permission to use it, as ADR 0014 requires of the two order-contract dimensions, and no permission Tiler can express grants distributivity — so contraction-order exploration is *illegal* as a settled legality position rather than unexplored or merely unimplemented, and its rejection must name the missing distributivity dimension rather than a forbidden reassociation. It would still fail closed on a future compiler that accepted a tensor contraction under a contract permitting both reassociation and permutation; that no such contract is expressible today, and that `normalize_serial_sum` admits only one input, are incidental limits rather than the reason. Whether to admit the dimension at all is a product choice reserved under [Q-SEM-015](open-questions.md) and owned by [`decide-whether-to-admit-a-distributivity-permission`](../tickets/decide-whether-to-admit-a-distributivity-permission.md).

#### Numerical contract

**Fact — a contraction is a reduction and inherits every reduction obligation.** [Numerical semantics](numerical-semantics.md) requires a reduction definition to state input dtype, accumulator dtype, output dtype, identity and empty-domain behaviour, operation-order guarantee, NaN and signed-zero behaviour, and its deterministic-or-implementation-dependent result policy; requires canonical reduced axes to be a nonempty, unique, sorted set with ordered-fold contributors in ascending lexicographic order of reduced coordinates; and holds reassociation and permutation as independent permissions, each requiring both an operation capability and an effective numerical permission, proved separately by any physical schedule. The concrete reduction topology belongs to the selected physical plan and participates in artifact identity.

**Fact — K-padding is not free, and the contract already says so.** [Numerical semantics](numerical-semantics.md) keeps empty result, algebraic identity, and safe physical padding as three separate facts, and gives the exact counterexample: strict floating sum may return `+0.0` for an empty domain, yet adding `+0.0` to the singleton `-0.0` under round-to-nearest produces `+0.0`, so `+0.0` is not bitwise-neutral padding for that reduction even though it is its empty result. **Inference.** Padding the contracted extent to a tile multiple with zeros — the ordinary way a tiled GEMM handles a ragged `K` — is a schedule choice that owes a neutrality proof under the selected conformance contract, or must track nonempty partials, or must use another proven construction. It is not admitted by the fact that zero is the additive identity.

**Fact — this is the single point where the two senses of "contraction" meet, and it is a bit-level difference.** A tensor contraction's per-contributor step is `accumulator + a * b`. Whether that may become one rounding is precisely ADR 0015's permission, and it is `NumericalPermission::Forbidden` in the only numerical contract the compiler registers today (`StrictF32NumericalContract::governed` in `crates/tiler-compiler/src/request.rs`), which `tiler-metal` realizes by requiring `-ffp-contract=off`. **Inference.** Under that contract a device or library GEMM built on fused multiply-add accumulate is producing a different result from the declared semantics, and ADR 0076 proposes the rule that closes the remaining direction: no authority may narrow, weaken, or substitute the caller's stated numerical contract in order to make a target feasible. A reader who conflates the two senses would conclude that a *tensor* contraction permits fused multiply-add by virtue of its name. It does not; the permission is a separate, independently resolved field of the numerical contract.

**Fact — the shape of a contraction's numerical signature is already an accepted decision rather than an open question.** [Dtype resolution and mixed-precision precedent](research/numerics/dtype-resolution-precedents.md) records that contractions expose more than input and result dtype — StableHLO's `DotAlgorithm` separates input precision types, accumulation type, component decomposition, primitive-operation count, and permission for imprecise accumulation, and Triton's `dot` separately exposes an accumulator input, result dtype, and input precision whose defaults differ by vendor — and its decision, accepted on 2026-07-19, is that reductions and contractions "use specialized signatures that explicitly identify applicable computation/input precision, accumulator dtype, result value dtype, conversion behavior, and order or algorithm contract", with the exact fields being operation capabilities rather than one universal bag. [IR](ir.md) carries that rule normatively. **Inference.** A contraction admitted with only an operand dtype and a result dtype would be underspecified against an already-accepted decision.

**Fact — nothing would fuse.** `FusionNumericalCapabilities::governed` in `crates/tiler-compiler/src/fusion_legality.rs` registers three fusion roles — `ValueSource`, `ElementwiseArithmetic`, and `OrderedReduction` — and its own contract is that "an operation family with no registered role yields no fusion legality at all". `OrderedReduction` is held by `tiler::strict-serial-sum-f32@1` alone and names a strict lexicographic left fold. A contraction is neither elementwise nor a strict left fold over one operand, so it would resolve to `FusionLegality::Unknown`, and the fusible prologues and epilogues Milestone 6 names are exactly what that failure closes off.

#### Preconditions

**Fact — a matmul cannot currently be presented to the compiler at all.** `normalize_serial_sum` in `crates/tiler-compiler/src/request.rs` rejects any program whose `input_count()` is not exactly `1`, whose `output_count()` is not exactly `1`, or whose operation count is outside four-to-five, and then requires the operations to be precisely a strict serial `Sum` over an `Add` of a `Multiply` of the single input by constants, with `check_recognized_operation_cover` demanding that those recognized operations exhaust the reachable graph. A binary contraction has two inputs and fails at the first check. **Fact.** `crates/tiler-compiler/src/capability.rs` implements the lowering-capability registry that would resolve an index-access provider for such an occurrence, and `crates/tiler-compiler/src/legality.rs` implements the refinement authority that would bind its emitted region to the occurrence, and both are now reached from `pipeline::compile` — `wire-capability-and-refinement-into-compile-path` resolves a capability for every recognized occurrence and refines its emitted region, with four governed index-access providers registered. **Corrected by `draft-public-extension-seam-ownership-adr`:** this previously read that neither was reached and that no governed provider registered a capability, which was true when written and is not now. The surrounding claim is unaffected and is the reason a contraction still cannot be presented: no capability exists for a contraction occurrence, so resolution fails closed rather than lowering one. That limit is owned by the [optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md), whose own ticket names `capability` and `legality` among the draft authorities it must wire onto the ordinary path.

**Fact — no backend has executed anything.** The support matrix records that `tiler-metal` emission exists for the first profile while its strict Apple conformance claim fails closed on every governed family, and that no device execution exists yet. **Inference.** A contraction could be given an identity, a validated index structure, and an access relation without any of this — those are semantic obligations and need no backend. It could not be *planned*, costed, scheduled, or realized, which is the whole of Milestone 6. That asymmetry is why [Q-SEM-015](open-questions.md) states a demand trigger for the semantic half and a structural gate for the planning half rather than one conjoined condition.

#### Decisions reserved for Tom

Two questions in this framing are genuine product and architecture choices where the alternatives encode different valid priorities, and this section deliberately does not settle them. Each belongs in the eventual decision record, presented one at a time with a worked tensor program. A third choice belongs in that same record but is framed elsewhere: whether to admit a distributivity permission at all, defined by [Numerical semantics](numerical-semantics.md#distributivity-is-outside-the-order-contract) and owned by [`decide-whether-to-admit-a-distributivity-permission`](../tickets/decide-whether-to-admit-a-distributivity-permission.md). [Q-SEM-015](open-questions.md) indexes all three.

1. **Whether a contraction is one keyed family carrying an index-structure attribute, or a set of fixed-arity keys per shape class.** One key makes every index-structure question attribute validation inside a single operation definition and gives normalization and region formation one family to reason about, at the cost of a large attribute schema and an inference routine that must reject every malformed structure. Separate keys keep each inference routine small and each signature exactly checkable, at the cost of registry growth per rank-and-batching combination and of normalization having to relate keys that denote the same computation.
2. **Whether a semantic contraction node may consume more than two operands.** [Optimizer](compiler/optimizer.md) reserves alternative associations of "a future multi-input einsum contraction", and contraction-order exploration only has a subject if either the node is multi-operand or the optimizer may reassociate a chain of binary nodes. The first makes association a choice over one node's own declared semantics; the second makes it a logical-exploration rewrite over a chain. Neither answer changes the numerical paragraph above: [Numerical semantics](numerical-semantics.md#distributivity-is-outside-the-order-contract) settles that a multi-operand node defined as a flat sum has contributors that no binary association ever forms, so recovering an association from it consumes distributivity exactly as the chain rewrite does. This choice therefore determines what an association is a choice *about*, not whether it needs a permission Tiler cannot currently express.

## Milestone 7: artifact maturity

- Stable versioned artifact schema.
- Compatibility policy beyond the earlier lockstep experimental schema.
- Mature macro-local multi-entrypoint packaging and deterministic expansion.
- Concurrent expansion locking and compiler-cache diagnostics.
- Embedded-byte size budgets and, if justified by measurement, linker-level
  deduplication that does not change call-site DX.
- Compile/search/artifact-size budgets.
- Platform and toolchain compatibility policy.

## Milestone 8: calibration

- Device-family microbenchmarks.
- Predicted-versus-observed plan tracking.
- Cost coefficient calibration and candidate pruning.
- Optional offline or profile-guided schedule selection.

The proposed Rust/Metal integration does not require runtime source JIT.

## Operation-family support matrix

Wide operation support is a durable project goal, and the first supported profile is four strict-`f32` operations. This section exists so that the narrowness is a tracked position rather than an accidental one. It is a visibility artefact: listing a family authorizes nothing, and a rung recorded here is a claim about evidence that exists today rather than about intent. Widening any row is separate, explicitly scheduled work.

Two axes are owned elsewhere and are cross-referenced rather than restated here. The dtype universe belongs to [`enumerate-the-mature-tensor-dtype-taxonomy`](../tickets/enumerate-the-mature-tensor-dtype-taxonomy.md) and its [mature tensor dtype taxonomy](research/numerics/mature-dtype-taxonomy.md), which deliberately claims no reference, optimizer, or backend support. The numerical dimensions each family's contract must answer belong to [Numerical semantics](numerical-semantics.md) and the [initial operation conformance matrix](research/numerics/operation-conformance-matrix.md), which likewise records required contract content rather than delivered support. This section adds only the maturity axis: how far each family has actually been built.

### Maturity ladder

`AGENTS.md` requires a type-system reservation, an architectural seam, implemented support, and a tested guarantee to remain four different maturity claims. The rungs below preserve all four and decompose implemented support by the layer that owns it. Each rung presupposes every rung below it for the same family.

| Rung | Name | What must already exist | `AGENTS.md` maturity claim |
| --- | --- | --- | --- |
| R1 | Type-system reservation | An extension point can express the family. No contract fixes its meaning. | type-system reservation |
| R2 | Architectural seam | An accepted ADR or normative contract fixes the family's obligations, but no identity is registered and no program can construct it. | architectural seam |
| R3 | Recognized identity | A governed `OpKey` is registered in `tiler-ir`'s standard semantic registry with a schema, inference, and normative reference, so a program using it verifies. | implemented support |
| R4 | Reference-evaluated | `tiler-reference` registers an evaluator for the exact signature. | implemented support |
| R5 | Optimizer-supported | `tiler-compiler` resolves a fusion role and legality for the family instead of failing closed to `Unknown`. | implemented support |
| R6 | Backend-realized | The structured-kernel vocabulary carries the construct, a backend emits it, and the target's declared numerical realization does not reject it. | implemented support |
| R7 | Tested guarantee | Checked-in tests or conformance evidence exercise the family's normative contract, bounded by exactly what they exercise. | tested guarantee |

R7 is scoped to an operation, dtype, target, and layer, never to a family as a whole. Under ADR 0042 an empirical qualification is not a normative guarantee, so a measurement never promotes a row to R7 for an unmeasured input.

### Family state and reconsideration triggers

Every row carries a trigger. A row without one is a note; a row with one is a tracked position. Claims are labelled **Fact** when supported by inspected source or an accepted ADR's own recorded status, and **Measurement** when tied to an exact environment.

| Operation family | Rung | Evidence | Reconsideration trigger |
| --- | --- | --- | --- |
| Pointwise `f32` constants and separate-rounding arithmetic: `constant-f32`, `add-f32`, `multiply-f32` | R6, with R7 bounded to the target-neutral layers | **Fact.** `StandardSemantics` in `crates/tiler-ir/src/semantic/registry.rs` registers exactly these three operations plus `strict-serial-sum-f32`; `StandardReferenceProvider` in `crates/tiler-reference/src/lib.rs` registers an evaluator for each; `FusionNumericalCapabilities::governed` in `crates/tiler-compiler/src/fusion_legality.rs` gives them `ValueSource` and `ElementwiseArithmetic` roles; `BinaryOp::F32Add` and `BinaryOp::F32Multiply` in `crates/tiler-ir/src/kernel/model.rs` are emitted by `crates/tiler-metal/src/emit.rs`. **Measurement.** `tiler-metal` rejects any kernel performing `f32` arithmetic under a subnormal-preserving realization on every governed Apple family, because Apple GPU `f32` arithmetic flushes subnormals in every math mode, so emission succeeds while the strict Apple conformance claim fails closed. No device execution exists yet. | This is the first profile; revisit when the [optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md) closes and Metal AOT plus runtime execution land, at which point R7 must be restated per target rather than per layer. |
| Strict serial `f32` `Sum` reduction | R6, with R7 bounded to the target-neutral layers | **Fact.** `tiler::strict-serial-sum-f32@1` is registered, reference-evaluated, and carries the sole `OrderedReduction` fusion role; ADR 0055 selects it as the first Metal value proof. Its lexicographic contributor order and result-boundary NaN canonicalization are part of its registered definition facts. | Milestone 4 broadens the exact serial baseline; revisit with [`implement-parallel-reduction-strategies`](../tickets/implement-parallel-reduction-strategies.md), which must not reuse this row's rung for a tree topology. |
| Remaining pointwise float algebra: `Subtract`, `Divide`, negation, required `Fma` | R2 | **Fact.** ADR 0024 fixes round-to-nearest ties-to-even for `Add`, `Subtract`, `Multiply`, and `Divide`, and ADR 0015 makes `Fma` a dedicated single-rounding operation that may not be lowered to separate roundings. ADR 0024 is `partial` and ADR 0015 is `not-started`; no key for `Subtract`, `Divide`, negation, or `Fma` exists in the standard registry. | A named workload or frontend lowering that needs one. Each entering operation requires a key, an evaluator, a fusion role, and a backend realization before it may be claimed above R2; `Divide` additionally needs its reciprocal permission resolved under Q-SEM-001. |
| Reductions beyond strict sum: product, logical `any` and `all`, extrema reductions, seeded and empty-domain forms, tree and multi-pass topologies | R2 | **Fact.** ADRs 0012, 0022, 0023, and 0025 accept physical reduction topology, reduction identities and initial values, the extrema families, and the empty-result-versus-padding split; all four are `implementation_status: not-started`. The only registered reduction is `tiler::strict-serial-sum-f32@1`, and `OrderedReduction` is the only reduction fusion role, so any other reduction resolves to no fusion legality at all. | Milestone 4, via [`implement-parallel-reduction-strategies`](../tickets/implement-parallel-reduction-strategies.md). Q-PLAN-004 must close before two reductions may coexist in one kernel. A non-identity seed and an identity-less extrema reduction are separate obligations from a new scalar family. |
| Pointwise transcendentals: `Exp`, `Log`, `Sin`, `Gelu`, and similar | R2 | **Fact.** ADR 0016 and ADR 0042 accept a complete typed accuracy-contract vocabulary, exact rational tolerances, a versioned ULP metric, and a refinement relation; both are `implementation_status: not-started` and no ticket implements one. No transcendental operation, evaluator, or structured-kernel construct exists; see absence check 1 below. `docs/ir.md` names `Gelu` illustratively and requires an admitted key to pin its exact formula, so erf-GELU and a tanh approximation are not interchangeable. | Q-SEM-004 selects the first operation, dtype, and accuracy tuples. Milestone 1 forbids admitting any such operation before its accuracy contract is canonically serialized and reference-evaluated end to end, so the first transcendental is a vertical slice rather than one more pointwise key. |
| Integer data arithmetic: wrapping, saturating, checked, and widening add, subtract, and multiply | R2 | **Fact.** ADR 0039 accepts explicit overflow-specialized families with required-no-overflow as a discharged proof or runtime-validation obligation; it is `not-started`. No integer data dtype is admitted: the standard registry registers exactly one value type, `tiler::f32@1`. `KernelType::Index` and `BinaryOp::IndexAdd` and `IndexMultiply` are index-role address arithmetic in the structured-kernel vocabulary and are not integer tensor support. | Whenever an integer tensor value enters a profile, most likely with Milestone 2Q, whose code tensors are `i4`, `u4`, `i8`, and `u8`. Closure needs an admitted integer dtype key plus one explicit overflow family per operation; no family may be inferred from a width. |
| Integer division and remainder: signed truncating, floor, Euclidean, ceiling, canonical unsigned, exact | R2 | **Fact.** ADR 0040 accepts the specialized families together with their quotient rounding, matched remainder sign and range, zero-divisor behavior, signed quotient overflow, and the standalone `MIN rem -1` result; it is `not-started`. `BinaryOp::IndexDivide` and `BinaryOp::IndexModulo` are truncating index division and remainder by a positive constant for address computation only. | The same trigger as integer arithmetic, plus a validated divisibility precondition mechanism before exact division may be admitted. The enabling gate is the value-assumption machinery of ADR 0021 together with Q-SEM-001. |
| Cast and convert: floating widening and narrowing, float to integer, integer to float, integer widening and narrowing, bit reinterpretation | R2, over one realized construct that is not a dtype conversion | **Fact.** ADR 0010 makes conversion a typed semantic contract and ADR 0041 separates the strict rounded, exact, ordered saturating, and total saturating NaN-to-zero float-to-integer families; both are `not-started`. `docs/ir.md` lists `Cast` among illustrative built-ins, and no `Cast` key exists. The only conversion the structured-kernel vocabulary realizes is `ConvertOp::CanonicalizeF32Nan`, an `f32`-to-`f32` NaN canonicalization whose own definition records that representation, narrowing, and rounding conversions are versioned extensions that must name their own rounding, overflow, and exceptional-value behavior. **Proposal.** The structured-kernel-IR verifier research sketches `Convert`, `Bitcast`, and `CheckedNarrow` as kernel-level operations; that research is `spike-only` and none of the three is implemented. | Q-SEM-005 selects the first float-to-integer tuples. Admitting any second dtype into a profile forces this row, because a mixed-dtype program cannot be expressed without an explicit conversion operation and no implicit promotion exists after semantic admission. |
| Quantize, Dequantize, Requantize, AssembleQuantized, and integer Rescale | R2, over an implemented type-system reservation | **Fact.** ADRs 0029 through 0033 accept affine parameter index maps, first-class quantized values, NaN rejection in strict affine `Quantize`, the exact evaluation order, and the semantic-validation-versus-physical-enforcement split; all are `not-started`. The encoded-numeric reservation is implemented: `ResolvedValueType` carries an `EncodedNumeric` variant keyed by `QuantSchemeKey`, and `ValueTypeDefinitionKey::EncodedNumeric` participates in registry identity. No governed scheme is registered; `tiler::affine@1` appears only inside the test module of `crates/tiler-ir/src/semantic/registry.rs`, and no quantization operation exists. | Milestone 2Q, via [`prototype-quantized-value-vertical`](../tickets/prototype-quantized-value-vertical.md); [`implement-first-quantized-backend-profile`](../tickets/implement-first-quantized-backend-profile.md) is deferred behind it. Q-SEM-006 gates any non-affine scheme. |
| Arithmetic over reduced-precision floats: f16, bf16, OCP OFP8 E4M3 and E5M2, MX FP6, FP4, and E8M0 | R2 for identity recognition, R1 for arithmetic | **Fact.** ADR 0036 recognizes IEEE binary16, binary32, binary64, binary128, BF16, the OFP8 pair, and the MX constituents as built-in logical formats, and ADR 0038 recognizes the OCP MX schemes as compound values; both are `not-started`. ADR 0026 separates representability from operation support, so recognition never implies arithmetic. In code the standard registry admits one dtype, `tiler::f32@1`, so no reduced-precision identity is constructible and no operation signature admits one. The inventory itself is owned by [`enumerate-the-mature-tensor-dtype-taxonomy`](../tickets/enumerate-the-mature-tensor-dtype-taxonomy.md). | An admitted dtype key is the precondition; arithmetic then requires its own operation and dtype signature per ADR 0026. Q-SEM-003 closes when every admitted tuple has explicit reference, optimizer, and backend support state, which is the point at which this row splits per format. |
| `Minimum` and `Maximum`, `MinimumNumber` and `MaximumNumber` | R2 | **Fact.** ADR 0023 accepts the propagating and number-preferring families with deterministic `-0.0 < +0.0` ordering as separate semantic operations rather than one mode-selected operation; it is `not-started` and no key exists. [Numerical semantics](numerical-semantics.md) records that Metal `fmin` and `fmax` are number-preferring with an order-dependent signed-zero result, so neither family lowers to the obvious intrinsic without a fixup or a matching authorized relaxation. | Clamp or ReLU recognition, or an extrema reduction, entering a profile. The elementwise and reduction forms name one scalar family but retain separate identity, seed, and order contracts, so admitting one does not admit the other. |
| Structural and data-movement families: `Reindex`, `Broadcast`, views, bit-preserving copies | R2 | **Fact.** `docs/ir.md` gives `Reindex` and `Broadcast` normative semantics, including totality of the output-to-input coordinate function, explicit axis mapping for every many-to-one relation, and the narrow rank-zero operand admission that is a shape rule rather than an implicit `Broadcast` node. No `Reindex` or `Broadcast` key exists; see absence check 2 below, which matches only doc comments, one diagnostic rule name, and index-region test fixtures. | Milestone 2's own exit criterion, which requires an einops-derived chain and names reindex plus pointwise fusion; this row is that milestone's largest unstated prerequisite. Gather and scatter stay out until Q-SHAPE-007 triggers, and finite piecewise access maps until Q-SHAPE-006. |
| Tensor contraction: matmul, batched matmul, einsum | R1 | **Fact.** The planning layer anticipates contraction: Milestone 6 names contraction-order exploration and GEMM recognition, [Optimizer](compiler/optimizer.md) reserves alternative associations of a future multi-input einsum contraction as an equivalence group no expressible numerical policy admits, and a direct-or-GEMM-backed implementation rule, and [Fusion and scheduling](compiler/fusion-and-scheduling.md) sketches contraction-order choices and GEMM canonicalization. [IR](ir.md) uses the same sense once, requiring reductions and contractions to declare computation precision, accumulator and result types, and an order or algorithm contract. No ADR, semantic contract section, or registered key defines a contraction operation, so nothing fixes its identity, validation, or access relation. The word also carries an unrelated second meaning in the numerical contracts, where ADR 0015's contraction is the FMA fusion permission; the two senses must not be read as one another. | The consequences are framed by [Milestone 6](#framing-what-a-tensor-contraction-family-would-impose) and indexed as [Q-SEM-015](open-questions.md), which states the demand trigger and the structural gate. This row records only the rung; it does not restate that framing. |
| `Select` and bit-selecting operations | R1 | **Fact.** A tensor `Select` is named only in one row of the adopted [operation conformance matrix](research/numerics/operation-conformance-matrix.md); no ADR or normative contract section defines it and no key exists. Three other `Select`s in the corpus are different constructs and must not be counted as support: the shape-environment research `Select` is shape-metadata computation and explicitly not a tensor `where`; `ExprNode::Select` in `crates/tiler-artifact/src/program/expr.rs` is a host-side ABI expression with lazy branch evaluation; and the `Select` in the structured-kernel-IR verifier research is a proposed kernel-level operation. **Proposal.** That research is `disposition: adopted` but `implementation_status: spike-only`, and the implemented structured-kernel vocabulary in `crates/tiler-ir/src/kernel/model.rs` has no `Select`. | The first predicated or masked workload. Closure needs an admitted predicate value type, which does not exist because the registry admits no boolean dtype, plus an explicit rule for speculating unselected arms. |
| Effectful and stateful operations: hidden randomness, floating-point environment observation, in-place mutation | R1 | **Fact.** `OperationEffect` has exactly one variant, `Pure`, and is `#[non_exhaustive]`. [Operation extensions](operation-extensions.md) and [Numerical semantics](numerical-semantics.md) reserve a separately versioned effect signature and resource or effect-token value kinds while implementing none of the required ordering, liveness, verification, ABI, or partial-execution rules. ADR 0020 fixes the initial value-only floating-point exception contract. | Q-SEM-011, the first stateful, mutating, or hidden-random operation proposal. Q-SEM-013 separately gates differentiation and Q-PLAN-015 in-place execution; none of the three may be satisfied by widening `OperationEffect` alone. |

### Absence checks

An absent operation family is asserted above only where the exact check is reproducible. Each command is run from the repository root and its result is the evidence, not the expectation.

```sh
# 1. No transcendental operation family is named anywhere in the workspace.
#    This currently returns no output at all.
grep -rniE '\b(exp|log|sin|cos|tanh|sqrt|rsqrt|gelu|erf|sigmoid)\b' crates/ --include='*.rs'

# 2. No Reindex, Broadcast, or Cast operation family is defined. The current matches
#    are doc comments on the rank-zero scalar admission, one index-builder doc comment,
#    one diagnostic rule name, and index-region test fixture lines.
grep -rniE '\breindex\b|\bbroadcast\b|\bcast\b' crates/ --include='*.rs'

# 3. The standard semantic registry admits one value type and four operations. The
#    governed registrations are the calls inside StandardSemantics::register; the other
#    matches are the two registrar methods themselves and test-module fixtures. Read the
#    function rather than trusting the count.
grep -n 'register_marked_value_type\|register_operation' crates/tiler-ir/src/semantic/registry.rs
```

Two structural limits bound every rung above R4 and are easy to overstate. First, the compilation request path in `crates/tiler-compiler/src/request.rs` recognizes exactly one program shape — one input, one output, one strict serial `Sum` over an add-of-multiply-by-constants chain — so even the four admitted operations are not compilable in arbitrary combinations. Second, `crates/tiler-compiler/src/lowering.rs` resolves an index-access lowering capability for every recognized occurrence, so a program whose occurrences no installed capability covers fails closed rather than compiling. The first limit is owned by the [optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md). **Corrected by `draft-public-extension-seam-ownership-adr`.** This previously read that the registry had no in-crate production caller and that no governed provider registered a capability. Both were true when this matrix was written and were falsified by `wire-capability-and-refinement-into-compile-path`, which put `resolve_lowering` on the ordinary `compile()` path and shipped four governed index-access providers. The rungs above are unaffected: capability resolution constrains which programs compile, not how far any operation family was built.

## Deferred until justified

- Generated backward kernels.
- In-place or aliasing kernels.
- Arbitrary user-authored kernel language.
- Cross-threadgroup atomics as a general scheduling tool.
- Runtime autotuning.
- Stable public serialization compatibility before IR boundaries settle.
