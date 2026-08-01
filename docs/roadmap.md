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

The [operation-family support matrix](#operation-family-support-matrix) below and the separate [dtype support maturity ledger](dtype-support.md) record how narrow the delivered surface currently is on both axes, so recognition, implementation, and execution cannot collapse into one claim.

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

ADR 0072 corrects the prototype identity boundary before generic compilation: graph meaning, reached provider-independent definitions, admission-provider provenance, and the full registry environment have independent identities. The implemented region/index/schedule/KIR/program/artifact layers preserve that separation rather than nesting whole-program or provider identity into reusable structural content.

The bullets below describe Milestone 0A's authorized scope. Some are now delivered for the bounded profile; the [status page](status.md) and live ticket graph, rather than this proposed progression, own current implementation state.

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

The bullets below describe the Milestone 0B exit rather than the state of every component. The offline Metal producer, expansion cache, neutral artifact path, and bounded device proof now exist; inline composition and consumer integration do not.

- Compose the proposed proc macro with the implemented deterministic `xcrun` producer and emit manifest/metallib byte-string literals without consumer setup.
- Integrate the implemented immutable self-validating content-addressed cache into that complete inline workflow.
- Retain the completed embedding, Cargo freshness, cache deletion, and Apple
  family/toolchain probes. Measure rust-analyzer cold/warm behavior when the
  component is available, plus the actual native macOS and non-Apple fallback
  paths.

**Exit criterion:** cold inline macro AOT produces a loadable bundle, warm
equivalent expansions invoke no external compiler, and the proposed Rust DX
works without build scripts or prebuild commands. Failure does not invalidate
Milestone 0A's consumer-independent compiler boundary.

The completed Metal AOT and runtime proof establishes bounded backend artifact and device-execution evidence but intentionally excludes the proc macro, complete inline orchestration, and consumer integration. It is a prerequisite and evidence for this milestone, not its complete exit.

The live ticket graph deliberately gates those proofs on a backend-consumable
target-neutral compiler path. That path lowers a verified semantic program
through semantic normalization, generic fusion-region formation and legality,
checked semantic-to-index refinement, region covers, target feasibility and
scheduling, physical-implementation planning and complete-plan selection,
structured kernel IR, and artifact-facing programs, closed by the
[optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md)
and the reviewed [public compiler API](../tickets/prototype-public-compiler-api.md)
that the intended inline frontend will consume. These are independent authorities with real
dependencies, not a single linear pipeline, and their dependency-satisfied
ordering shifts as work lands. `tkt rollup` and `tkt ready` — not a chain
enumerated here — report which authorities are complete, dispatchable, or
blocked.

Opaque physical calls are part of this bounded compiler path. Their [provider ticket](../tickets/implement-opaque-physical-call-providers.md) and [frontier integration](../tickets/integrate-opaque-calls-into-the-physical-frontier.md) are both done: the frontier admits checked `ScheduledKernel` and `KernelSubprogram` bodies and registered `OpaqueCall` proposals, rejecting only the reserved `View` variant, and this sentence previously deferred all of that behind gates those tickets have since passed. What remains absent is out-of-crate registration — `OpaqueCallDeclaration` and `OpaqueCallRegistry` are crate-private, so no external provider can supply an opaque call — and that gap is a public-boundary question, not a missing contract.

The bounded Metal path now includes independently verified KIR lowering, numerical-realization refusal, Apple offline compilation, bundle assembly, a public reviewed-draft neutral artifact codec carrying real emitted and compiled payloads, runtime validation and preflight, one-way routing commit, and device execution. The inline proc macro, complete inline cache/AOT/embedding orchestration, artifact-family delivery, and Candle adapter remain explicit downstream tickets rather than implicit promises of that proof.

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

The target-neutral portion of that proof is now integrated into the ordinary compiler path: the same verified request produces a two-stage materialized program and a one-stage fused program, checked generic authorities carry it through refinement, legality, scheduling, KIR, verified program, and artifact construction, and a public reviewed-draft session facade exposes compilation and caller-installed index-access lowering capabilities. Complete cover enumeration is proved for the bounded recognized profile. The request recognizer and candidate corpus remain graph-specific; this does not establish general occurrence discovery, unbounded partition search, or workload breadth. The Metal producer and retained runtime prototype carry the selected path through source, artifact, guarded routing, and device execution for the measured corpus.

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

**Inference — a contraction is not one operation in the way `Add` is one operation.** `matmul` over `[M, K] × [K, N]`, batched `matmul` over `[B, M, K] × [B, K, N]`, and an attention-shaped einsum `bhqk,bhkd->bhqd` differ in rank, in which axes are batched, in which are contracted, and in the output axis order. They are one family only if a canonical attribute carries that index structure. If instead each shape class takes its own key, the standard registry grows one governed key per rank-and-batching combination and semantic normalization must relate them; the registry admits four scalar F32 operation identities, one tensor-contraction identity, and three strict-affine quantization operation identities, so this is a real growth decision rather than a bookkeeping one.

**Fact — an index-notation attribute is not automatically canonical.** [IR](ir.md) fixes the attribute data model: `CanonicalValue::Utf8String` holds "exact valid UTF-8 bytes with no implicit Unicode normalization", `Sequence` order is semantic, and `Record` fields are sorted by unique ID. **Inference.** Storing an authored subscript string therefore makes `ij,jk->ik` and `ab,bc->ac` two different operations with different semantic graph identities, different artifact identities, and no cache reuse between them, even though they denote the same computation. ADR 0074's conventions bind the fix as well as the hazard: the encoder writes a versioned NUL-terminated `tiler.<subject>.v<N>` domain tag before any content, a fixed-width length before every variable-length run, excludes transient identifiers "wherever the represented semantics are equivalent without them", and matches every encoded enum exhaustively. An author's choice of index letters is exactly such a transient identifier. **Proposal.** The canonical attribute is a *structure* — which operand positions each index visits, which indices are free, which are summed, and the output index order — rather than the authored labelling, and the operation definition normalizes to that structure before storing or hashing, in the same way the schema already normalizes a field equal to its declared default.

**Fact — there is a precedent for where that normalization lives.** `tiler::strict-serial-sum-f32@1` carries its reduced axes as a required `Sequence` attribute, and `StrictSerialSumF32::infer` in `crates/tiler-ir/src/semantic/registry.rs` rejects an empty sequence, a non-`u32` element, an out-of-range axis, and any axis that is not strictly ascending and unique — as typed provider diagnostics, at construction. A contraction's index structure is the same kind of obligation at greater complexity.

#### Validation

**Fact.** Every registered operation validates its own operands. `BinaryF32::infer` accepts a rank-zero operand under the narrow scalar admission [IR](ir.md) owns, and otherwise requires `left == right` exactly. **Inference — a contraction is the first family whose operands are *required* to disagree.** `[M, K]` and `[K, N]` share one extent and differ everywhere else. **Corrected by `admit-the-contraction-semantic-profile`:** this previously read that nothing in the admitted vocabulary expressed it, which was true when written. `tiler::strict-tensor-contraction-f32@1` now does, by carrying the index structure that says which operand axis binds which iteration coordinate; its inference routine resolves the shared extent through the three-outcome path below and names both observed operand axes when the equality is disproved.

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

**Inference — this is the reason a contraction is architecturally different from a pointwise family, and it is a sharper difference than "it reduces".** A pointwise family's access maps are functions of the output coordinates alone. The registered strict serial sum introduces a reduction domain, but it has one operand, so exactly one access map consumes it. A contraction introduces a reduction domain that *two or more operand access maps share, while each drops a different subset of the free coordinates*. For `C[m, n] = sum over k of A[m, k] * B[k, n]`, the iteration domain is `(m, n)`, the reduction domain is `(k)`, `A`'s map is `(m, n, k) -> (m, k)` and never mentions `n`, and `B`'s map is `(m, n, k) -> (k, n)` and never mentions `m`. **Corrected by `admit-the-contraction-semantic-profile`:** this previously read that no admitted operation produced two such maps, which was true when written. `a_contraction_emits_two_operand_projections_dropping_different_coordinates` in `crates/tiler-ir/tests/index_region.rs` now emits exactly that region, and both projections discharge their bounds by interval propagation with no exhaustive proof. The emission is an index-layer region, not a lowering capability; the sentence below about the absence of any such capability is unaffected.

**Inference — the index language already admits those maps; the gap is elsewhere.** [IR](ir.md) bounds the initial index vocabulary to addition and negation, multiplication by a parameter-only expression, and Euclidean floor division or modulo by a proven-positive parameter-only expression, and rejects iteration-by-iteration multiplication and tensor-data-derived indices. Each contraction operand map above is a pure projection and permutation of the coordinate vector, using no index arithmetic at all; the multiplication is between two *loaded values* in the reducer body, which is ordinary scalar computation and not index arithmetic. A contraction therefore needs no new access class, no piecewise map (Q-SHAPE-006), and no indirect relation (Q-SHAPE-007). What it needs is an operation capability that emits those maps, and the [preconditions](#preconditions) below are about the absence of any such capability rather than about the expressiveness of the index layer.

**Inference — expressing the same computation as broadcast, multiply, and reduce is a different semantic identity, not an alternative spelling.** Composing `Broadcast` to `[M, N, K]`, an elementwise `Multiply`, and a `Sum` over `K` denotes the same values, but it is a different canonical graph, and it is the physical planner rather than the semantic layer that must then discover the contraction inside it. Keeping the contraction atomic is what makes GEMM recognition an operation-specific rewrite over a named node — which is what [Optimizer](compiler/optimizer.md) already assumes when it proposes a "supported prologue/epilogue around a semantic operation with an opaque library implementation" as a region-candidate rule. Neither `Broadcast` nor a general `Reduce` is registered today, so the alternative spelling is not currently available either.

#### Lowering and physical planning

**Fact — the two implementation alternatives named in this milestone have different prerequisites and must not be scheduled as one item.** [Optimizer](compiler/optimizer.md) lists "direct or GEMM-backed contraction" among implementation rules, and both it and [Fusion and scheduling](compiler/fusion-and-scheduling.md) fix the mature `ImplementationFrontier` body as an additive sum type over `ScheduledKernel`, `KernelSubprogram`, `OpaqueCall`, and `View`. The frontier now admits the first three and rejects only the reserved `View` — [`implement-opaque-physical-call-providers`](../tickets/implement-opaque-physical-call-providers.md) and [`integrate-opaque-calls-into-the-physical-frontier`](../tickets/integrate-opaque-calls-into-the-physical-frontier.md) delivered the opaque contract and its admission, and an earlier revision of this passage quoted the since-corrected "admits only checked `ScheduledKernel` proposals" sentence. A direct or tiled contraction is a `ScheduledKernel`; an optimized library matmul is an `OpaqueCall`. **Inference, restated on the corrected premise.** "Direct/tiled contractions and fusible epilogues" and "GEMM recognition and library-call alternatives" are now separated by a different pair of gaps than deferral: a library GEMM needs an out-of-crate provider seam that does not exist (`OpaqueCallDeclaration` and `OpaqueCallRegistry` are crate-private) and a per-shape numerical guarantee the L3 record measured no library supplying — `MPSMatrixMultiplication` was refuted against all twenty-two named topologies, and the optimizer admits an implementation candidate only when its guarantee refines every effective operation contract. So the second alternative is inadmissible on numerical-evidence grounds today regardless of the seam, which is the argument the passage after this one carries and nothing here falsifies.

**Fact — hard feasibility and estimated cost stay separate, and a library GEMM tests that boundary first.** [Optimizer](compiler/optimizer.md) requires each implementation candidate to advertise "a machine-checkable numerical guarantee, realization/provider identity, and scoped evidence", admitted "only when that guarantee refines every effective operation contract", and states that numerical conformance is checked *before* the dominance relation because "accuracy is a hard semantic dimension, not a Pareto cost". [IR](ir.md) states the feasibility half of the same discipline, where an "`Unknown` *feasibility* verdict keeps its candidate in explain and search state only" and "such a candidate cannot enter an executable `ImplementationFrontier` or manifest" — a rule IR scopes to that verdict and explicitly declines to generalize to every `Unknown` in the corpus. **Inference.** A vendor GEMM that does not publish its accumulation order, input precision, and contraction behaviour offers unknown numerical evidence, which the optimizer's own rule already makes inadmissible rather than merely expensive; IR's feasibility verdict is the parallel case under a different assessment, not that rule applied twice. Milestone 6's exit criterion — that contraction planning use the same properties, enforcers, and cost framework rather than backend-specific exceptions — is exactly the requirement that a library call not be granted an exemption here.

**Fact — layout conversion is already an enforcer, not a new mechanism.** [Optimizer](compiler/optimizer.md) lists contiguous materialization, layout conversion, and encoding repacking as enforcers that supply a missing required property at a cost, defines satisfaction, subsumption, child-requirement derivation, dominance, and cycle-checked insertion over boundary contracts, and retains interesting properties such as useful unit-stride axes on a bounded Pareto frontier even when they are not locally cheapest. Every entry is value-preserving; a dtype cast is a semantic operation and is deliberately not on that list. **Inference.** "Layout-conversion costing" is therefore the first family that exercises that machinery under real pressure rather than a separate subsystem, which is what [Fusion and scheduling](compiler/fusion-and-scheduling.md) means by requiring contraction planning to follow the boundary-contract and cost infrastructure.

**Fact — contraction-order exploration is a numerical permission question, not only a search question.** [Optimizer](compiler/optimizer.md) places the tensor-contraction association rewrite under logical exploration, where `ExploreLogicalAlternatives` "adds only proved contract-preserving forms", and admits it "only when the effective distributivity, reassociation, and operand-permutation permissions all authorize the regrouping". **Fact.** [Numerical semantics](numerical-semantics.md#distributivity-is-outside-the-order-contract) owns the derivation and states why all three are named: regrouping `(AB)C` into `A(BC)` forms different rounded products rather than regrouping one reduction's contributors, so the identity it consumes is distributivity — a third numerical dimension, independent of the two order-contract dimensions, for which that contract admits no permission. **Inference.** Reassociation is necessary and never sufficient. Each of the three demands both an operation capability declaring the algebraic property and an effective numerical permission to use it, as ADR 0014 requires of the two order-contract dimensions, and no contract Tiler can express grants distributivity because the dimension is absent — so contraction-order exploration is *illegal* as a settled legality position rather than unexplored or merely unimplemented, and its rejection must name the missing distributivity dimension rather than a forbidden reassociation. It would still fail closed on a future compiler that accepted a tensor contraction under a contract permitting both reassociation and permutation; that no registered contract currently permits operand permutation, and that `normalize_serial_sum` admits only one input, are incidental limits rather than the reason. Whether to admit the dimension at all is a product choice reserved under [Q-SEM-015](open-questions.md) and owned by [`decide-whether-to-admit-a-distributivity-permission`](../tickets/decide-whether-to-admit-a-distributivity-permission.md).

#### Numerical contract

**Fact — a contraction is a reduction and inherits every reduction obligation.** [Numerical semantics](numerical-semantics.md) requires a reduction definition to state input dtype, accumulator dtype, output dtype, identity and empty-domain behaviour, operation-order guarantee, NaN and signed-zero behaviour, and its deterministic-or-implementation-dependent result policy; requires canonical reduced axes to be a nonempty, unique, sorted set with ordered-fold contributors in ascending lexicographic order of reduced coordinates; and holds reassociation and permutation as independent permissions, each requiring both an operation capability and an effective numerical permission, proved separately by any physical schedule. The concrete reduction topology belongs to the selected physical plan and participates in artifact identity.

**Fact — K-padding is not free, and the contract already says so.** [Numerical semantics](numerical-semantics.md) keeps empty result, algebraic identity, and safe physical padding as three separate facts, and gives the exact counterexample: strict floating sum may return `+0.0` for an empty domain, yet adding `+0.0` to the singleton `-0.0` under round-to-nearest produces `+0.0`, so `+0.0` is not bitwise-neutral padding for that reduction even though it is its empty result. **Inference.** Padding the contracted extent to a tile multiple with zeros — the ordinary way a tiled GEMM handles a ragged `K` — is a schedule choice that owes a neutrality proof under the selected conformance contract, or must track nonempty partials, or must use another proven construction. It is not admitted by the fact that zero is the additive identity.

**Fact — this is the single point where the two senses of "contraction" meet, and it is a bit-level difference.** A tensor contraction's per-contributor step is `accumulator + a * b`. Whether that may become one rounding is precisely ADR 0015's permission. The compiler's registered strict and flush-to-zero contracts forbid it and require `tiler-metal` to emit `-ffp-contract=off`; the registered relaxed contract permits it. **Inference.** A device or library GEMM built on fused multiply-add accumulation is incompatible with either forbidding contract and may be considered under the relaxed contract only after satisfying its other operation, realization, and evidence obligations. ADR 0076's rule still applies: no authority may narrow, weaken, or substitute the caller's stated numerical contract in order to make a target feasible. A reader who conflates the two senses would conclude that a *tensor* contraction permits fused multiply-add by virtue of its name. It does not; the permission is a separate, independently resolved field of the numerical contract.

**Fact — the shape of a contraction's numerical signature is already an accepted decision rather than an open question.** [Dtype resolution and mixed-precision precedent](research/numerics/dtype-resolution-precedents.md) records that contractions expose more than input and result dtype — StableHLO's `DotAlgorithm` separates input precision types, accumulation type, component decomposition, primitive-operation count, and permission for imprecise accumulation, and Triton's `dot` separately exposes an accumulator input, result dtype, and input precision whose defaults differ by vendor — and its decision, accepted on 2026-07-19, is that reductions and contractions "use specialized signatures that explicitly identify applicable computation/input precision, accumulator dtype, result value dtype, conversion behavior, and order or algorithm contract", with the exact fields being operation capabilities rather than one universal bag. [IR](ir.md) carries that rule normatively. **Inference.** A contraction admitted with only an operand dtype and a result dtype would be underspecified against an already-accepted decision.

**Fact — nothing would fuse.** `FusionNumericalCapabilities::governed` in `crates/tiler-compiler/src/fusion_legality.rs` registers three fusion roles — `ValueSource`, `ElementwiseArithmetic`, and `OrderedReduction` — and its own contract is that "an operation family with no registered role yields no fusion legality at all". `OrderedReduction` is held by `tiler::strict-serial-sum-f32@1` alone and names a strict lexicographic left fold. A contraction is neither elementwise nor a strict left fold over one operand, so it would resolve to `FusionLegality::Unknown`, and the fusible prologues and epilogues Milestone 6 names are exactly what that failure closes off.

#### Preconditions

**Fact — a matmul cannot currently be presented to the compiler at all.** `normalize_serial_sum` in `crates/tiler-compiler/src/request.rs` rejects any program whose `input_count()` is not exactly `1`, whose `output_count()` is not exactly `1`, or whose operation count is outside four-to-five, and then requires the operations to be precisely a strict serial `Sum` over an `Add` of a `Multiply` of the single input by constants, with `check_recognized_operation_cover` demanding that those recognized operations exhaust the reachable graph. A binary contraction has two inputs and fails at the first check. **Fact.** `crates/tiler-compiler/src/capability.rs` implements the lowering-capability registry that would resolve an index-access provider for such an occurrence, and `crates/tiler-compiler/src/legality.rs` implements the refinement authority that would bind its emitted region to the occurrence, and both are now reached from `pipeline::compile` — `wire-capability-and-refinement-into-compile-path` resolves a capability for every recognized occurrence and refines its emitted region, with four governed index-access providers registered at that landing and six since `admit-the-reindex-and-broadcast-operation-families` added one per structural family. **Corrected by `draft-public-extension-seam-ownership-adr`:** this previously read that neither was reached and that no governed provider registered a capability, which was true when written and is not now. The surrounding claim is unaffected and is the reason a contraction still cannot be presented: no capability exists for a contraction occurrence, so resolution fails closed rather than lowering one. That limit is owned by the [optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md), whose own ticket names `capability` and `legality` among the draft authorities it must wire onto the ordinary path.

**Fact — only the bounded governed-F32 prototype has executed.** The retained runtime proof dispatched thirty bit-compared cases under its explicitly selected `FlushSubnormalsToZeroF32` contract on one Apple M4 Max host; it is not a production runtime or portable backend guarantee. **Inference.** A contraction could be given an identity, a validated index structure, and an access relation without any backend — those are semantic obligations, and `admit-the-contraction-semantic-profile` has since delivered all three. It could not be *planned*, costed, scheduled, or realized, which is the whole of Milestone 6 and remains true. That asymmetry is why [Q-SEM-015](open-questions.md) states a demand trigger for the semantic half and a structural gate for the planning half rather than one conjoined condition.

#### Decisions reserved for Tom

Two questions in this framing are genuine product and architecture choices where the alternatives encode different valid priorities, and this section deliberately does not settle them. Each belongs in the eventual decision record, presented one at a time with a worked tensor program. A third choice belongs in that same record but is framed elsewhere: whether to admit a distributivity permission at all, defined by [Numerical semantics](numerical-semantics.md#distributivity-is-outside-the-order-contract) and owned by [`decide-whether-to-admit-a-distributivity-permission`](../tickets/decide-whether-to-admit-a-distributivity-permission.md). [Q-SEM-015](open-questions.md) indexes all three.

1. **Whether a contraction is one keyed family carrying an index-structure attribute, or a set of fixed-arity keys per shape class — decided.** [ADR 0087](decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) accepts the single keyed family on 2026-07-31, on the L2 workload evidence: the identity carries a renaming-invariant canonical index structure, the five structural rules reject at construction under named rules, frontends never choose among keys, and an unsupported structure fails closed at capability resolution. The fixed-key costs both ways are preserved in that record, including the disclosed partial recoverability of the transpose costs that deliberately did not carry the decision.
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

## Optimized Metal language-model inference

The durable capability map from the current bounded Metal proof to a compiled, optimized language-model inference pipeline. **This section authorizes nothing.** Like the support matrix below it, it is a visibility artefact: a rung listed here is a claim about what evidence exists today, and every step is separately scheduled work gated on the evidence beneath it.

**The goal is inference, not training**, and that is a scope position rather than an omission. Training needs a gradient program, an optimizer state, and a mutation model that none of the contracts here admit; nothing below reserves a seam for it. Reconsider only if Tom explicitly broadens the product goal.

### The ladder

Each rung presupposes the one below it. The maturity column uses the same four `AGENTS.md` claims the support matrix uses, so a reader can compare the two without translating.

| Step | Capability | Owning ticket | Activation trigger | Maturity today |
| --- | --- | --- | --- | --- |
| L1 | A representative workload is named, with its exact model, shapes, and dtypes — the [workload profile](research/program-planning/first-metal-lm-workload.md) | [`define-first-metal-lm-workload`](../tickets/define-first-metal-lm-workload.md) | this map is accepted | workload named and bounded; nothing executes |
| L2 | The tensor operation and shape surface that workload requires is derived — the [derivation record](research/shapes/transformer-operation-and-shape-surface.md) | [`derive-transformer-operation-and-shape-surface`](../tickets/derive-transformer-operation-and-shape-surface.md) | L1 names a workload | surface derived and dispositioned; nothing executes |
| L3 | One contraction runs end to end on Metal | [`spike-first-metal-contraction-vertical`](../tickets/spike-first-metal-contraction-vertical.md) | L2 lists the contraction shapes, and milestone 6 settles the keyed-family question below — both fired, the second by ADR 0087 on 2026-07-31 | first realization profile bounded and measured — the [realization record](research/scheduling/first-metal-contraction-realizations.md); six candidates eliminated to one permission-free survivor over an exact-arithmetic topology corpus, seven delivery tickets filed; nothing compiles or executes |
| L3′ | Transformer non-linearities, normalization, and reductions are scoped — the [derivation record](research/numerics/transformer-nonlinear-normalization-and-reductions.md) | [`scope-transformer-nonlinear-normalization-and-reductions`](../tickets/scope-transformer-nonlinear-normalization-and-reductions.md) | L2 lists them; runs beside L3 | non-linear, normalization, and reduction contracts derived; three capability verticals filed; nothing executes |
| L4 | A complete attention program and transformer block | [`design-attention-program-vertical`](../tickets/design-attention-program-vertical.md) | L3 and L3′ both deliver — both fired on 2026-07-31 | fixed-profile prefill attention program derived — the [program design](research/program-planning/first-attention-program-vertical.md); twenty-two typed steps over exact C1 and B1 shapes, two decompositions surviving on legality and feasibility with an exact residency predicate, eight candidates rejected with grounds, ten delivery tickets filed; nothing compiles or executes |
| L5 | Stateful prefill and token decoding | [`design-autoregressive-state-and-kv-cache`](../tickets/design-autoregressive-state-and-kv-cache.md) | L4 delivers a block — fired on 2026-07-31 under the **design-rung** reading of that wording, recorded here because nothing else records it: every delivered rung so far (L1–L4, L7) fired on record delivery rather than on capability delivery, and L4's own record states that the block itself is its delivery ticket 7, so holding a *design* behind the attention implementation chain would buy no evidence the state model needs | state and execution contract derived — the [state contract](research/runtime/autoregressive-state-and-kv-cache.md); ten state properties, a five-layer ownership table, three cache-identity invariants with reproducible checks, three planning decisions taken with their eliminations and one handed to L6 with its arithmetic, four failure cases tested against the implemented stack, eleven tickets filed; nothing compiles or executes |
| L6 | Complete supported-model execution, including ingestion | [`design-model-ingestion-and-complete-execution`](../tickets/design-model-ingestion-and-complete-execution.md) | L2 and L5 deliver | none |
| L7 | A selected quantized model profile | [`scope-first-quantized-lm-profile`](../tickets/scope-first-quantized-lm-profile.md) | L1 and L3 deliver; milestone 2Q supplies the quantized-value proof — all three fired | one profile selected from measured evidence — the [selection record](research/numerics/first-quantized-lm-profile.md); per-output-channel strict-affine U8/F32 over the workload's 196 weighted projections, every U4 form eliminated on a measured model observable and every per-block form on contraction legality, seven delivery tickets filed; no dtype ledger cell moved and nothing compiles or executes |
| L8 | Model-level correctness and performance qualification | [`design-model-level-qualification-and-optimization`](../tickets/design-model-level-qualification-and-optimization.md) | L1 and L6 deliver | none |

**Every rung through L5, and L7, now carries a record and no capability, and that is the honest state.** The support matrix records four governed F32 operations as the narrow implemented profile; a transformer needs contraction, softmax, normalization, and a residual add, and of those only the residual add is executable and only the contraction is statable at all — it sits at R3, a registered identity with no evaluator, no fusion role, and no lowering. Nothing in this ladder is partially built.

L1, L2, L3, L3′, L4, L5, and L7 are the rungs whose deliverable so far is a record rather than a capability, and their cells say what was delivered. L7 ran ahead of L5 and L6 because its trigger depends on L1, L3, and the quantized-value proof rather than on the rungs directly beneath it, and what it delivered is a selection: one profile chosen against the pinned workload's own measured behaviour, with the cheapest and most nearly built candidate — per-tensor U4, the one quantized profile that already has a target-neutral vertical — eliminated because it agrees with the F32 baseline's greedy token at zero of eighteen conformance positions. L3 delivered a record, not the capability its row names — the end-to-end remainder is [`integrate-the-contraction-vertical-into-the-runtime`](../tickets/integrate-the-contraction-vertical-into-the-runtime.md). L1 named the pinned `Qwen/Qwen3-0.6B-Base` workload, bounded it into a conformance row and a benchmark matrix, and manifested it by digest. L2 turned that trace into families, extent classes, and capability tickets: 253 contraction occurrences over exactly three index structures, an atomic-versus-composition disposition for every family, two varying extents that need no capability beyond the symbolic index promotion already in flight, and four filed capability tickets. L4 wrote the fixed-profile prefill attention block down as twenty-two typed steps with three ordered named outputs, eliminated its decompositions to a materialized baseline and a recomputing alternative on legality and feasibility rather than on cost, and left the block itself as [`integrate-the-attention-block-into-the-runtime`](../tickets/integrate-the-attention-block-into-the-runtime.md). L5 attached to that seam and derived the state and execution contract: the cache is two ordinary program inputs and two retained outputs per layer with the cursor, capacity, and allocation owned by the runtime instance, so mutable inference state reaches no cache identity in the stack; it fixed the update as out-of-place with the cursor advancing only on observed terminal success, kept prefill and decode one program by binding an empty cache, and recorded that the incorrect-position case is accepted by every layer of Tiler today. All are research outcomes and not implementation maturity claims — no part of the workload compiles, dispatches, or executes, no operation family moved a rung, and the four-claim vocabulary the other rungs use does not apply to any of them.

### What the ladder rests on that is already scheduled elsewhere

These are prerequisites, not duplicates, and the ladder does not restate them:

- **Contraction as a keyed family.** Milestone 6 owned the question of whether a contraction is one keyed family carrying an index-structure attribute or a set of fixed-arity keys per shape class; [ADR 0087](decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) settled it on the single keyed family, and `admit-the-contraction-semantic-profile` registered that key for the workload's first index structure. What L3 still rests on is the planning half of [Q-SEM-015](open-questions.md), not the identity: an attention einsum needs one governed key and three attribute values, and none of the three can yet be planned.
- **The sequence-extending tensor family.** [`scope-the-sequence-extending-tensor-family`](../tickets/scope-the-sequence-extending-tensor-family.md) and its [record](research/shapes/sequence-extending-tensor-family.md) own what it means to extend a tensor along one axis — the operation each decode step performs twice per layer. L5 depends on it and inherits the physical realization, the semantic mechanism having been eliminated to a value-producing family there.
- **The dtype universe.** [`enumerate-the-mature-tensor-dtype-taxonomy`](../tickets/enumerate-the-mature-tensor-dtype-taxonomy.md) and the [mature dtype taxonomy](research/numerics/mature-dtype-taxonomy.md) own it. L7 depends on it and does not re-derive it.
- **Quantized values.** Milestone 2Q owns the quantized-value vertical proof that L7 needs.
- **Numerical contracts per family.** [Numerical semantics](numerical-semantics.md) and the [operation conformance matrix](research/numerics/operation-conformance-matrix.md) own what each family's contract must answer.

### Explicitly deferred, with reconsideration triggers

Recorded so that absence is a tracked position rather than an oversight. None has a reserved seam; each would be new architectural work.

| Deferred capability | Why | Reconsideration trigger |
| --- | --- | --- |
| Training | Needs a gradient program, optimizer state, and a mutation model no contract here admits | Tom broadens the product goal beyond inference |
| Distributed execution | The physical contracts model one device's placement, memory domains, and transfers; multi-device is a different feasibility problem | A workload at L1 does not fit one device's memory |
| Speculative decoding | Needs two models and a divergence policy; L5 assumes one autoregressive state | L5 delivers and measured decode latency is the binding constraint |
| Unconstrained dynamic shapes | The sourced-extent profile bounds symbolic extents deliberately; unconstrained shapes reopen bounds proofs and index-domain feasibility | A workload at L1 requires an extent no `ShapeEnv` bound can express |

## Operation-family support matrix

Wide operation support is a durable project goal. The only backend-executed profile is the bounded four-operation F32 prototype under `FlushSubnormalsToZeroF32`, while semantic/reference admission also includes the F32 `Reindex` and `Broadcast` families and the U4/F32 and U8/F32 `AssembleStrictAffine`, `QuantizeStrictAffine`, and `DequantizeStrictAffine`. This section exists so that operation breadth stays tracked; the separate [dtype support maturity ledger](dtype-support.md) prevents either slice from being generalized across dtypes or layers. Listing a family authorizes nothing, and widening any row is separate, explicitly scheduled work.

Three axes are cross-referenced rather than duplicated. The [mature tensor dtype taxonomy](research/numerics/mature-dtype-taxonomy.md) owns the dtype universe, the [dtype support maturity ledger](dtype-support.md) owns delivered state by dtype and layer, and [Numerical semantics](numerical-semantics.md) plus the [initial operation conformance matrix](research/numerics/operation-conformance-matrix.md) own required contract content. This section adds only operation-family maturity.

### Maturity ladder

`AGENTS.md` requires a type-system reservation, an architectural seam, implemented support, and a tested guarantee to remain four different maturity claims. The rungs below preserve all four and decompose implemented support by the layer that owns it. Each rung presupposes every rung below it for the same family.

| Rung | Name | What must already exist | `AGENTS.md` maturity claim |
| --- | --- | --- | --- |
| R1 | Type-system reservation | An extension point can express the family. No contract fixes its meaning. | type-system reservation |
| R2 | Architectural seam | An accepted ADR or normative contract fixes the family's obligations, but no *operation* identity is registered and no program can construct one. A registered dtype identity is a different axis and sits in the [dtype support ledger](dtype-support.md); it never lifts an operation family off this rung. | architectural seam |
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
| Pointwise `f32` constants and separate-rounding arithmetic: `constant-f32`, `add-f32`, `multiply-f32` | R6, with R7 bounded to checked target-neutral layers and one prototype execution row | **Fact.** `StandardSemantics` in `crates/tiler-ir/src/semantic/registry.rs` registers these three operations plus `strict-serial-sum-f32`; `StandardReferenceProvider` in `crates/tiler-reference/src/standard.rs` registers an evaluator for each; `FusionNumericalCapabilities::governed` gives them `ValueSource` and `ElementwiseArithmetic` roles; `BinaryOp::F32Add` and `BinaryOp::F32Multiply` are emitted by `tiler-metal`. **Measurement.** The retained runtime proof dispatched thirty bit-compared cases on one Apple M4 Max host. The governed Metal profile still rejects strict subnormal-preserving arithmetic where the measured target flushes F32 subnormals, so that execution row is bounded to the proof's exact realization, host, toolchain, program, and corpus. | Revisit per target and numerical realization when a production runtime or a second device family enters; neither inherits the prototype's R7 boundary. |
| Strict serial `f32` `Sum` reduction | R6, with R7 bounded to checked target-neutral layers and one prototype execution row | **Fact.** `tiler::strict-serial-sum-f32@1` is registered, reference-evaluated, and carries the sole `OrderedReduction` fusion role; ADR 0055 selects it as the first Metal value proof. Its lexicographic contributor order and result-boundary NaN canonicalization are part of its registered definition facts. **Measurement.** The retained runtime proof executed the strict sum in all thirty bit-compared cases on one Apple M4 Max host under `FlushSubnormalsToZeroF32`; that row is bounded to the proof's exact realization, host, toolchain, program, and corpus. | Milestone 4 broadens the exact serial baseline; revisit with [`implement-parallel-reduction-strategies`](../tickets/implement-parallel-reduction-strategies.md), which must not reuse this row's rung for a tree topology. |
| Remaining pointwise float algebra: `Subtract`, `Divide`, negation, required `Fma` | R2 | **Fact.** ADR 0024 fixes round-to-nearest ties-to-even for `Add`, `Subtract`, `Multiply`, and `Divide`, and ADR 0015 makes `Fma` a dedicated single-rounding operation that may not be lowered to separate roundings. ADR 0024 is `partial` and ADR 0015 is `not-started`; no key for `Subtract`, `Divide`, negation, or `Fma` exists in the standard registry. | A named workload or frontend lowering that needs one. Each entering operation requires a key, an evaluator, a fusion role, and a backend realization before it may be claimed above R2; `Divide` additionally needs its reciprocal permission resolved under Q-SEM-001. |
| Reductions beyond strict sum: product, logical `any` and `all`, extrema reductions, seeded and empty-domain forms, tree and multi-pass topologies | R2 | **Fact.** ADRs 0012, 0022, 0023, and 0025 accept physical reduction topology, reduction identities and initial values, the extrema families, and the empty-result-versus-padding split; all four are `implementation_status: not-started`. The only registered reduction is `tiler::strict-serial-sum-f32@1`, and `OrderedReduction` is the only reduction fusion role, so any other reduction resolves to no fusion legality at all. | Milestone 4, via [`implement-parallel-reduction-strategies`](../tickets/implement-parallel-reduction-strategies.md). Q-PLAN-004 must close before two reductions may coexist in one kernel. A non-identity seed and an identity-less extrema reduction are separate obligations from a new scalar family. |
| Pointwise transcendentals: `Exp`, `Log`, `Sin`, `Gelu`, and similar | R2 | **Fact.** ADR 0016 and ADR 0042 accept a complete typed accuracy-contract vocabulary, exact rational tolerances, a versioned ULP metric, and a refinement relation. **The carrier now exists; no operation does, and the rung is unmoved.** `crates/tiler-ir/src/semantic/accuracy/` implements the four discriminated contract forms kept distinct by construction, exact rational tolerances, `tiler::ulp-reference-gap@1` with the dtype-capability check that rejects rather than guesses, the normalized predicate algebra and the accuracy-domain clause language with decided coverage and intersection semantics, canonical serialization into identity with a decode path that refuses every non-canonical spelling, the conservative refinement relation over an open registered-implication registry, and the five classified conformance-evidence classes; `crates/tiler-reference/src/accuracy.rs` supplies the certified enclosures and the three-way conformance decision that ADR 0042's exact comparison needs. It registers no operation key, no reference evaluator for one, and no structured-kernel construct, so nothing here is executable and nothing here is a family. Both ADRs' frontmatter still reads `implementation_status: not-started`, which this landing makes stale — the carrier is implemented and the initial supported subset is not, so `partial` is the value they now describe; correcting it is a `contracts/decisions` edit this row does not hold. `docs/ir.md` names `Gelu` illustratively and requires an admitted key to pin its exact formula, so erf-GELU and a tanh approximation are not interchangeable. | Q-SEM-004 selects the first operation, dtype, and accuracy tuples. Milestone 1 forbids admitting any such operation before its accuracy contract is canonically serialized and reference-evaluated end to end; that precondition is now *buildable* rather than blocked, which moves the gate from the vocabulary to the tuple selection and to D-4's backend evidence. The first transcendental remains a vertical slice rather than one more pointwise key. |
| Integer data arithmetic: wrapping, saturating, checked, and widening add, subtract, and multiply | R2 | **Fact.** ADR 0039 accepts explicit overflow-specialized families with required-no-overflow as a discharged proof or runtime-validation obligation; it is `not-started`. The standard registry registers every accepted integer identity, `i2` through `i64` and `u2` through `u64`, but admits U4 and U8 in an operation signature only as strict-affine code and zero-point roles; no general integer arithmetic operation is registered, and no other width reaches any signature. `KernelType::Index`, `KernelType::U8`, and `KernelType::I32` are address, carrier, and widened-subtract machinery and are not integer tensor arithmetic support. | A named integer tensor workload requiring one exact width and overflow family. Quantized code tensors do not trigger general arithmetic, and no neighboring width or operation may be inferred from shared machinery. |
| Integer division and remainder: signed truncating, floor, Euclidean, ceiling, canonical unsigned, exact | R2 | **Fact.** ADR 0040 accepts the specialized families together with their quotient rounding, matched remainder sign and range, zero-divisor behavior, signed quotient overflow, and the standalone `MIN rem -1` result; it is `not-started`. `BinaryOp::IndexDivide` and `BinaryOp::IndexModulo` are truncating index division and remainder by a positive constant for address computation only. | The same trigger as integer arithmetic, plus a validated divisibility precondition mechanism before exact division may be admitted. The enabling gate is the value-assumption machinery of ADR 0021 together with Q-SEM-001. |
| Cast and convert: floating widening and narrowing, float to integer, integer to float, integer widening and narrowing, bit reinterpretation | R2, over one realized construct that is not a dtype conversion | **Fact.** ADR 0010 makes conversion a typed semantic contract and ADR 0041 separates the strict rounded, exact, ordered saturating, and total saturating NaN-to-zero float-to-integer families; both are `not-started`. `docs/ir.md` lists `Cast` among illustrative built-ins, and no `Cast` key exists. The only conversion the structured-kernel vocabulary realizes is `ConvertOp::CanonicalizeF32Nan`, an `f32`-to-`f32` NaN canonicalization whose own definition records that representation, narrowing, and rounding conversions are versioned extensions that must name their own rounding, overflow, and exceptional-value behavior. **Proposal.** The structured-kernel-IR verifier research sketches `Convert`, `Bitcast`, and `CheckedNarrow` as kernel-level operations; that research is `spike-only` and none of the three is implemented. | Q-SEM-005 selects the first float-to-integer tuples. Admitting any second dtype into a profile forces this row, because a mixed-dtype program cannot be expressed without an explicit conversion operation and no implicit promotion exists after semantic admission. |
| `QuantizeStrictAffine`, `DequantizeStrictAffine`, `Requantize`, `AssembleStrictAffine`, and integer `Rescale` | R4 for per-tensor strict-affine U4/F32 and U8/F32 semantics/reference; separately tested non-monotone physical evidence exists for U4 dequantization without promoting the family to R5 or R6; the remainder is R2 | **Fact.** `register_standard_quantization` registers U4, U8, both strict-affine schemes, and `AssembleStrictAffine`, `QuantizeStrictAffine`, and `DequantizeStrictAffine`; the reference provider evaluates all three and tests typed no-NaN and positive-finite-scale preconditions. Target-neutral U4 dequant schedule, KIR, program, artifact, and Metal-translation construction is tested, but compiler region-subject verification refuses that scalar program, Metal rejects the strict numerical realization, runtime semantic enforcement and dtype dispatchability are absent, and U8 has no physical vertical. The ladder is monotone, so those later structural fixtures do not skip the absent optimizer and executable-target rungs. `Requantize`, integer `Rescale`, non-per-tensor maps, and other schemes remain absent. | The exact cells and structural dependencies are owned by the [dtype support maturity ledger](dtype-support.md). Q-SEM-006 gates any non-affine scheme, and no selected backend may generalize the U4 fixture into U8 or another map. |
| Arithmetic over reduced-precision floats: f16, bf16, OCP OFP8 E4M3 and E5M2, MX FP6, FP4, and E8M0 | R2 for identity recognition, R1 for arithmetic | **Fact.** ADR 0036 recognizes IEEE binary16, binary32, binary64, binary128, BF16, the OFP8 pair, and the MX constituents as built-in logical formats, and ADR 0038 recognizes OCP MX compound schemes. ADR 0026 separates representability from operation support. Every one of those logical identities is now registered by standard semantics with its complete descriptor, and no operation signature admits any of them; Apple F16/BF16 measurements and `ArithmeticType` variants remain contract vocabulary and bounded target evidence, and registration did not turn either into support. E8M0 is scale data, not ordinary arithmetic. | A selected exact dtype, operation, workload, target, conversion/accumulation policy, physical ABI, runtime predicates, and corpus. Q-SEM-003 tracks the per-layer state rather than assuming registration from recognition. |
| `Minimum` and `Maximum`, `MinimumNumber` and `MaximumNumber` | R2 | **Fact.** ADR 0023 accepts the propagating and number-preferring families with deterministic `-0.0 < +0.0` ordering as separate semantic operations rather than one mode-selected operation; it is `not-started` and no key exists. [Numerical semantics](numerical-semantics.md) records that Metal `fmin` and `fmax` are number-preferring with an order-dependent signed-zero result, so neither family lowers to the obvious intrinsic without a fixup or a matching authorized relaxation. | Clamp or ReLU recognition, or an extrema reduction, entering a profile. The elementwise and reduction forms name one scalar family but retain separate identity, seed, and order contracts, so admitting one does not admit the other. |
| Structural and data-movement families: `Reindex`, `Broadcast`, views, bit-preserving copies | R5 for the two admitted families; views and bit-preserving copies stay R2 | **Fact.** `StandardSemantics` registers `tiler::reindex-f32@1` and `tiler::broadcast-f32@1`. A `Reindex` carries one named mapping form — `permute-axes`, `split-axis`, `merge-axes`, `insert-unit-axis`, `remove-unit-axis`, `reverse-axis` — as a strongly typed attribute, *derives* its result shape from the operand rather than accepting a declared one, and refuses every other coordinate map by name; a split whose factors overshoot its axis is refused as non-total and one that falls short as a **slice**, under separate diagnostic codes. A `Broadcast` carries an explicit axis mapping with exactly one entry per result axis, keeps a rank pad and an extent-one stretch as separate relations, and refuses a mapping that reorders or drops an operand axis, that widens without saying so, or that states no many-to-one relation at all. **Fact — decision D-10 is settled in the registered normative reference** rather than in a research record: a within-axis coordinate permutation is admitted in the `reverse-axis` form `i -> extent − 1 − i` and in no other, because the affine within-axis bijections of an axis are exactly the identity and the reversal, while a general within-axis permutation is a tensor-data-derived index the accepted index vocabulary rejects. **Fact.** `StandardReferenceProvider` registers a bit-preserving evaluator for each, which deliberately does not apply the arithmetic NaN canonicalization, and `crates/tiler-reference/tests/structural_conformance.rs` covers all six forms and all three relations at ranks one through four against hand-derived materialized results with retained perturbations. **Fact.** `FusionNumericalCapabilities::governed` gives both the `CoordinateRelation` role, so a region containing them derives legality instead of failing closed, and `governed_index_access_capabilities` registers an index-access lowering for each — a read whose coordinates are affine, quasi-affine, or omitted, and a write over the whole result domain, with no scalar operation applied and no new access class. `governed::tests` executes each emitted region on the independent index-region oracle. **Fact — and no further rung.** Neither family emits a structured-kernel construct, by design, and `crates/tiler-compiler/src/request.rs`'s whole-program recognizer admits no program shape containing one, so no program containing either reaches a `VerifiedKernel`. | Milestone 2's own exit criterion, which requires an einops-derived chain and names reindex plus pointwise fusion. R6 arrives with [`reach-a-verified-kernel-through-the-structural-families`](../tickets/reach-a-verified-kernel-through-the-structural-families.md). Views and bit-preserving copies have no key and no contract and stay at R2. Gather and scatter stay out until Q-SHAPE-007 triggers, and finite piecewise access maps until Q-SHAPE-006. A within-axis rotation is expressible in the index vocabulary and deliberately unadmitted; admitting one needs a modulus in canonical identity, a positivity proof, and a conformance row of its own. |
| Sequence extension: `Concatenate` along one axis | R1 | **Fact.** No normative contract says what extending a tensor along an axis means and no key exists: `StandardSemantics::register` constructs four scalar F32 operation definitions, `register_standard_contraction` one, `register_standard_reindex` and `register_standard_broadcast` one each, and `register_standard_quantization` three, and none of the ten is a concatenation. [The sequence-extending family record](research/shapes/sequence-extending-tensor-family.md) derives the family's obligations on six axes and eliminates the in-place windowed write as its *semantics*, but a research record is not a normative contract, so the rung stays R1 — the mechanism is dispositioned, nothing is fixed. **Fact.** Its two implemented blockers sit outside the semantic layer and would bind either mechanism: `ExtentRelation` admits no additive relation over its symbol-or-constant terms, so the defining extent equality `S == C + T` is inexpressible, and whole-program verification refuses a second writer of one value (`MultipleWriters`) while proving nothing about the untouched bytes of a partially written one. | The KV cache reaches a compiled program — 56 appends per forward pass of the pinned workload, at [rung L5](#the-ladder). [`design-autoregressive-state-and-kv-cache`](../tickets/design-autoregressive-state-and-kv-cache.md) inherits the physical realization, not the semantic mechanism; the in-place alternative stays on the effectful row below and behind Q-PLAN-015, and a concatenation along an inner axis loses the contiguous-window realization, which is an applicability predicate rather than a second family. **Updated 2026-07-31:** that rung delivered [its state contract](research/runtime/autoregressive-state-and-kv-cache.md) and filed [`admit-the-sequence-extension-concatenate-family`](../tickets/admit-the-sequence-extension-concatenate-family.md) as the R3 step, with the zero-extent rule made reachable rather than hypothetical by prefill binding an empty cache, and [`admit-an-additive-extent-relation`](../tickets/admit-an-additive-extent-relation.md) as the separate ticket the earlier record declined to file — L5 is the consumer that makes the gap load-bearing, because `S == C + T` is the only check that refuses a stale cache bind. The rung does not move: nothing is registered. |
| Sub-tensor selection: `Slice` and other non-surjective coordinate maps | R1 | **Fact.** No contract defines a slice and no key exists. `tiler::reindex-f32@1` admits bijective permutation, split, merge, unit-axis insertion or removal, and the within-axis reversal; a slice is injective but not surjective, so it is outside that family and outside `Broadcast`, and the registered `reindex.split.not-surjective` refusal names it as such rather than admitting it as a narrow reindex. **Fact.** No pinned workload row needs one. The [L2 derivation](research/shapes/transformer-operation-and-shape-surface.md) reduces `rotate_half` to a split, a within-axis permutation, a broadcast multiply, and a merge, and L1 records that the conformance row's `logits_to_keep=0` becomes `slice(0, None)`, so the vocabulary projection selects nothing. **Measurement.** [The L4 program](research/program-planning/first-attention-program-vertical.md) compared that composition with the pinned reference's `rotate_half` on a `[1, 16, 10, 128]` operand: 0 of 20,480 elements differ, while dropping the coordinate swap differs at all 20,480. | A prefill pass that needs only the final position's logits, which otherwise projects all `T` positions to 151,936 values — 4,978,634,752 F32 bytes at the B1-d row against 607,744 for one position. The second trigger this row carried — [D-10](research/program-planning/first-attention-program-vertical.md#unresolved-decisions) resolving *against* a within-axis coordinate permutation, which would have sent `rotate_half` to a slice-plus-concatenate spelling — **did not fire**: the structural row above records the resolution, which admits the reversal form the composition needs. **A third trigger was added on 2026-07-31 by [the L5 state contract](research/runtime/autoregressive-state-and-kv-cache.md), and it is about correctness rather than bytes.** At batch 1 with a contiguous cache, absolute position enters a decode program through `cos` and `sin` alone — the mask is derivable from `T` and `S`, the cache extents from `C`, the residual stream carries none — and a wrong rotary row is a `[1, 128]` F32 tensor with the same shape, dtype, accessible range, and launch geometry as the right one, so every layer accepts it and the result is a plausible logit vector with a wrong argmax. Selecting rows `C … C + T` from a bound `[max_positions, 128]` table would make the *inconsistency* mode unrepresentable, though not the wrong-cursor mode; [`admit-a-position-selecting-slice-for-the-rotary-table`](../tickets/admit-a-position-selecting-slice-for-the-rotary-table.md) owns it, and it must cost the fact that no `IndexNode` variant carries an extent symbol outside a `FloorDiv` or `Modulo` divisor. |
| Tensor contraction: matmul, batched matmul, einsum | R3 | **Fact.** `StandardSemantics` in `crates/tiler-ir/src/semantic/registry.rs` registers `tiler::strict-tensor-contraction-f32@1` — one governed key per [ADR 0087](decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md), carrying its index structure as a strongly typed attribute whose canonical encoding is renaming-invariant by first appearance, with all five structural admission rules refusing at construction under their own named provider diagnostics and a mutation proof demonstrating the collision the encoder's framing and renumbering prevent. Its numerical signature states computation/input precision, accumulator dtype, result dtype, conversion behaviour, contributor sequence, seed, empty-contracted-domain behaviour, both order permissions, the absent distributivity dimension, ADR 0015's contraction permission, the canonical NaN payload and where it is installed, and determinism — the shape [dtype resolution precedent](research/numerics/dtype-resolution-precedents.md) requires of a contraction. `crates/tiler-ir/tests/index_region.rs` emits the access relation, two operand projections dropping *different* iteration coordinates, so "no new access class" is exercised rather than assumed. **Fact — and no further rung.** No reference evaluator, no fusion role, no lowering capability, and no backend construct exists: `capability::tests::a_contraction_occurrence_resolves_to_no_installed_index_access_capability` pins that an occurrence still fails closed at resolution, and `compile()` refuses a contraction program earlier still, at the request boundary. A program can state a contraction and it verifies; nothing plans, costs, schedules, or executes one. The word also carries an unrelated second meaning in the numerical contracts, where ADR 0015's contraction is the FMA fusion permission; the two senses must not be read as one another, and this family declares that permission forbidden while `policy::operation_capabilities` still asks a target about it, because a contraction's per-contributor step is the one place the two meet. | R4 arrives with [`admit-the-contraction-normative-reference`](../tickets/admit-the-contraction-normative-reference.md); R5 and R6 stay behind the planning half of [Q-SEM-015](open-questions.md), which the [optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md) closes. The consequences are framed by [Milestone 6](#framing-what-a-tensor-contraction-family-would-impose). Index structures 2 and 3, the multi-operand form reserved behind rule five, and the distributivity permission are separately gated, and none of the three is admitted by this rung. |
| `Select` and bit-selecting operations | R1 | **Fact.** A tensor `Select` is named only in one row of the adopted [operation conformance matrix](research/numerics/operation-conformance-matrix.md); no ADR or normative contract section defines it and no key exists. Three other `Select`s in the corpus are different constructs and must not be counted as support; the [glossary](glossary.md) separates all four, and only one of them exists in compiled code — `ExprNode::Select`, the host-side ABI expression in `crates/tiler-ir/src/program/abi.rs`, which is what every hit of `grep -rnw Select crates/` is. **Proposal.** The structured-kernel-IR verifier research that proposes a kernel-level `Select` is `disposition: adopted` but `implementation_status: spike-only`, and the implemented structured-kernel vocabulary in `crates/tiler-ir/src/kernel/model.rs` has no `Select`. | The first predicated or masked workload. Closure needs an admitted predicate value type, which does not exist because the registry admits no boolean dtype, plus an explicit rule for speculating unselected arms. |
| Effectful and stateful operations: hidden randomness, floating-point environment observation, in-place mutation | R1 | **Fact.** `OperationEffect` has exactly one variant, `Pure`, and is deliberately **not** `#[non_exhaustive]`: its doc comment records that three encoders outside `tiler-ir` map the vocabulary totally onto an identity tag, so adding a variant is a build error at each of them rather than a silent re-encoding. **Corrected by `scope-the-sequence-extending-tensor-family`**, which read the declaration while scoping the KV-cache append; this cell previously asserted the attribute. The rung is unaffected and the fail-closed property is stronger than the cell claimed — mutation is unrepresentable rather than merely unimplemented. [Operation extensions](operation-extensions.md) and [Numerical semantics](numerical-semantics.md) reserve a separately versioned effect signature and resource or effect-token value kinds while implementing none of the required ordering, liveness, verification, ABI, or partial-execution rules. ADR 0020 fixes the initial value-only floating-point exception contract. | Q-SEM-011, the first stateful, mutating, or hidden-random operation proposal. Q-SEM-013 separately gates differentiation and Q-PLAN-015 in-place execution; none of the three may be satisfied by widening `OperationEffect` alone. |

### Absence checks

An absent operation family is asserted above only where the exact check is reproducible. Each command is run from the repository root and its result is the evidence, not the expectation.

```sh
# 1. No transcendental operation *family* is registered. This check no longer
#    asserts emptiness, and it should not have since `762ba34`: that commit gave
#    `crates/tiler-ir/src/semantic/broadcast/tests.rs` a comment naming the
#    workload's rotary `cos`/`sin` tables, so the "returns no output at all" this
#    block used to claim was already false when it was written. The accuracy
#    vocabulary adds the rest of the current hits — `exp`, `sqrt`, and `rsqrt`
#    under `crates/tiler-ir/src/semantic/accuracy/` and
#    `crates/tiler-reference/src/accuracy*`, where they are the subject of a
#    *contract carrier and a certified enclosure* rather than of an operation.
#
#    So read the hits, do not count them: the absence of a family rests on check
#    3's enumeration of the registry, exactly as `Log`'s and `Cast`'s already do.
#    `log` was in this alternation and has been removed, because the pattern
#    matched twenty-one ordinary logging identifiers plus one `O(n log n)`
#    comment and none of them is an operation family. A word that names a
#    transcendental and a commonplace English verb cannot discriminate here.
grep -rniE '\b(exp|sin|cos|tanh|sqrt|rsqrt|gelu|erf|sigmoid)\b' crates/ --include='*.rs'

# 2. No `Cast` operation family is defined. This check no longer asserts the
#    absence of `Reindex` or `Broadcast`, which are now registered families with
#    their own modules — the alternation below returns roughly three hundred
#    lines, and nearly all of them are those two families. `Cast`'s absence
#    rests on check 3's enumeration of the registry instead, for the same reason
#    `Log`'s does in check 1: a word that names a family and a commonplace
#    English verb ("broadcast", "cast") cannot discriminate here once one of the
#    families exists.
grep -rn 'reindex-f32\|broadcast-f32' crates/ --include='*.rs' -l

# 3. Enumerate standard F32 and quantized construction without relying on a brittle count.
#    Read StandardSemantics::register and register_standard_quantization after locating them.
rg -n 'register_standard_(quantization|contraction|reindex|broadcast)|register_integer|register_marked_value_type::<|register_operation\(' crates/tiler-ir/src/semantic/{registry.rs,quantization.rs}
```

Two structural limits bound every rung above R4 and are easy to overstate. First, the compilation request path in `crates/tiler-compiler/src/request.rs` recognizes exactly two one-input/one-output F32 shapes — a four-operation pointwise add or multiply over one input plus constants, and a four- or five-operation strict serial `Sum` over a scale-bias prologue — and explicitly refuses the strict-affine U4 scalar program, so admitted semantic operations are not compilable in arbitrary combinations. Second, `crates/tiler-compiler/src/lowering.rs` resolves an index-access lowering capability for every recognized occurrence, so a program whose occurrences no installed capability covers fails closed rather than compiling. The first limit is owned by the [optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md). **Corrected by `draft-public-extension-seam-ownership-adr`.** This previously read that the registry had no in-crate production caller and that no governed provider registered a capability. Both were true when this matrix was written and were falsified by `wire-capability-and-refinement-into-compile-path`, which put `resolve_lowering` on the ordinary `compile()` path and shipped four governed index-access providers; `admit-the-reindex-and-broadcast-operation-families` brought that count to six. The rungs above are unaffected: capability resolution constrains which programs compile, not how far any operation family was built. The first limit is what holds the structural row at R5 — a lowering capability exists for both families and no program shape the recognizer admits contains an occurrence of either, so the capability is never resolved on the compile path.

## Deferred until justified

- Generated backward kernels.
- In-place or aliasing kernels.
- Arbitrary user-authored kernel language.
- Cross-threadgroup atomics as a general scheduling tool.
- Runtime autotuning.
- Stable public serialization compatibility before IR boundaries settle.
