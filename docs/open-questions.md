---
schema: "tiler-doc/v1"
id: "tiler.questions.open"
kind: "questions"
title: "Open design questions"
topics: ["decisions", "research", "roadmap"]
questions_status: "active"
related: ["tiler.roadmap"]
---

# Open design questions

This file contains only unresolved work. Accepted invariants live in contracts
and ADRs; ordinary implementation tasks live in the roadmap. Each question has
one owner and an explicit way to close or reconsider it.

## Genuine product decisions

The initial checked shape-evidence spelling is no longer open: ADR 0067 selects
one pinned-nightly dependent-array family. Its conformance harness and
implementation are tracked work rather than product decisions.

### Q-ART-011 — Apple deployment floors

- Owner/tracking: [Metal backend](backends/metal.md), after its compatibility
  experiment below.
- Close when: old/new macOS and real/simulated iOS library-load and
  pipeline-creation evidence exists and Tom selects the supported floors.

## Milestone-owned implementation contracts

These have a correctness-derived direction. They require implementation and
tests, not a product-level choice unless their evidence exposes a new tradeoff.

Ergonomic artifact-family profiles are no longer open: Tom accepted the
consumer-visible spelling on 2026-07-31 under
[`accept-the-inline-artifact-family-profile-syntax`](../tickets/accept-the-inline-artifact-family-profile-syntax.md),
closing what was Q-ART-008. A region states `deliver <profile>;` or
`deliver <family> <minimum>, …;` in its declaration block, the profile vocabulary
is `fallback-only`, `macos`, `ios`, and `macos-and-ios`, and each spelling
resolves through the one canonical `ArtifactFamilySelection` constructor.
[The frontend contract](integration/frontends.md) states the accepted spelling
and what a stated selected family produces while nothing compiles a payload for
it; how the profiles expand is implementation and tests rather than a remaining
choice.

Five further questions closed on 2026-08-04 under [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](../tickets/re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md), which re-ran the ownership audit against the tree rather than trusting the 2026-07-31 list and found each of these answered rather than merely unowned. Each is recorded with the durable authority that now carries its answer, because a question removed without naming where the answer went is indistinguishable from one that was dropped.

- **Q-SEM-001 — numerical-policy presets**, closed by supersession rather than by delivery, so the expansion table it asked for has no subject. [Numerical semantics](numerical-semantics.md) records that the four-value preset enumeration was eliminated on 2026-08-01 and that a caller now resolves the contract one dimension at a time from a strict base; what replaced it is eleven governed dimensions in canonical order, a declaration keyed by dimension *and* scalar-arithmetic subject with silence about a neighbouring width failing closed as `Unknown`, the per-operation intersection against `operation_capabilities`, and a contract key that is the canonical **injective** encoding of the dimension vector under a versioned domain *per arithmetic type* — `tiler.contract.f32.v2` and, since 2026-08-05, the sibling `tiler.contract.bf16.v1` — rather than one of four hand-written names. Naming both is not a widening of the close: the two grammars are mutually closed, so a contract stated in one width never answers for the other, and the `f32` keys the close was recorded against are byte-identical. The round-trip half of the close condition is that injectivity, checked exhaustively over the whole statable space rather than sampled, in `crates/tiler-compiler/src/request.rs`; the rejection half is `RequestError::NoResolvableNumericalContract`, which names the contract key, the dimension, the arithmetic type, the required behaviour, the means the profile declares, and the declaring profile's versioned identity.
- **Q-PLAN-001 — initial bounded search representation.** [The optimizer contract](compiler/optimizer.md) carries both halves: region enumeration is general rather than a narrow builder — `EnumerateRegionCandidates` proposes every connected convex region of an arbitrary verified DAG within the declared budgets, with separate content and occurrence identities and typed budget-stops — and it **is checked against an exhaustive subset oracle**, which is the comparison this question required. The memo half was recorded here as a standing reservation — the contract reserved the term `memo` for an implementation that groups equivalence classes and performs goal-directed property search, and stated that a Cascades-style memo was one possible technique and not a committed architecture. **Corrected 2026-08-05 — the reservation is now an answer, and it moved further than "still not open".** [The rewrite-search formalism record](research/region-search/rewrite-search-formalism.md) selected a staged, alternative-retaining search against the primary literature and eliminated three alternatives including a Cascades memo as the whole search; the contract's [bounded hierarchical search](compiler/optimizer.md#bounded-hierarchical-search) section states the selection, places the memoized level at physical enumeration, and keeps the `memo` reservation for the goal-directed-property-search sense. The oracle requirement survives the selection and is stated without the "before a memo architecture is chosen" clause it used to carry, because an oracle is what makes a retention claim checkable regardless of which formalism produced it. One question opened where this one closed and it is narrower: whether semantic exploration adopts an e-graph over the semantic algebra alone, held at `deferred` by [`probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary`](../tickets/probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary.md) and [`decide-whether-stage-one-semantic-exploration-adopts-an-e-graph`](../tickets/decide-whether-stage-one-semantic-exploration-adopts-an-e-graph.md) with stated triggers, so it is board work rather than an entry here. General memo search in the reserved sense and calibrated cost estimation remain unimplemented and are Q-PLAN-002 and Q-PLAN-005's, not this question's; partitioning is no longer among them, having landed 2026-08-04.
- **Q-ART-002 — private lockstep serialization.** [The artifact contract](artifact-abi.md)'s "Implemented envelope profile" section is the durable record and states exactly what this question asked for without promising a public stable format: the fixed framing header and canonical manifest, the measurement that declaring the same payloads and providers in reversed order produces byte-identical envelopes, the refusal of a well-formed but non-canonical encoding by re-encode-and-compare rather than by normalization, the governed schema and component versions with their stepping rules, and the typed rejection vocabulary over corruption, truncation, trailing bytes, identity mismatch, and unsupported formats. The layout stays `pub(crate)` behind ADR 0074 convention 7 and only the codec's *capability* is accepted, which is the question's own "does not promise a public stable format" clause holding rather than a gap.
- **Q-ART-004 — expansion-cache root, accounting, and GC policy**, both halves. The root half closed on 2026-07-31 under [ADR 0089](decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md). The collection half closed on 2026-08-04: Tom decided the schedule — automatic eviction, configured by environment variables, with no maintenance command — and [the frontend contract](integration/frontends.md)'s "Compiler cache" section now states the whole policy, including the `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE` spelling and its refusal of an unsuffixed count, the documented default, the `off` opt-out, the rule that eviction runs only after a successful publication and at most once per process, and the deliberate decision that the report is *not* surfaced because automatic hygiene is silent while `ExpansionCache::collect` stays public for a caller that wants the detail. The race half is measured rather than asserted, at 1, 8, and 32 concurrent writer processes. The two aggregate ceilings are deliberately not configurable and that exclusion is a decision with its own derivation and activation triggers, held by [`configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction`](../tickets/configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction.md) at `deferred`. One residue survives the closure and is named rather than absorbed: `ExpansionCache::preflight` is still not called on a resolved root, so the filesystem-capability report `crates/tiler-macros/src/cache_root.rs` cites is never taken — reproduce with `grep -rn preflight crates/tiler-macros/src/`, which returns that doc comment and no call site. It is a diagnostic gap rather than a correctness one and is owned by [`call-the-expansion-cache-preflight-on-the-resolved-root`](../tickets/call-the-expansion-cache-preflight-on-the-resolved-root.md). **Corrected 2026-08-05 — that residue is closed and its owner ticket is `done`, so Q-ART-004 carries none.** The sentence above is preserved rather than deleted because the reproduction it offered is what dates it: `grep -rn preflight crates/tiler-macros/src/` now returns the `preflight` module, the call site in `aot::open_cache`, and the tests, so it refutes the claim it was cited for, and `crates/tiler-macros/src/cache_root.rs`'s doc comment now points at that probe rather than at nothing. `crates/tiler-macros/src/preflight.rs` probes the root `open_cache` resolved, once per build process and before the expansion uses it, reporting a root that did not answer for every property as one attributable line on standard error — refuted and unprobed distinguished, the build never refused, and `TILER_EXPANSION_CACHE_DIR=off` probing nothing and spending no probe. `grep -rn report_unsuitable_root crates/tiler-macros/src/` is the narrower check that passes, and [the frontend contract](integration/frontends.md)'s "Compiler cache" section is where the probe is now stated for a consumer.
- **Q-PKG-003 — proc-macro to Metal-AOT visibility.** [ADR 0088](decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) is the accepted record: the `tiler-macros` → `tiler-metal-aot` edge is deliberate and the facade is forbidden to hold one, so a process-spawning Apple toolchain driver never enters the build graph of a consumer that declared only `tiler`. The audit is mechanical rather than by inspection — `crates/tiler/tests/dependency_direction.rs` reads what Cargo actually resolved and fails on any inward edge to a frontend package or on a `tiler` → `tiler-metal-aot` edge, and it names its population first so "no offending edge" and "the check did not run" cannot be confused. The compile/UI half is `crates/tiler/tests/facade/`, and the lockstep clause is Q-ART-002's closure above.

The prototype tickets below all reached `done`. The list is therefore a record of what landed and **not** a live mapping of contracts to open work — reading it as one is the failure mode this file's ownership audit exists to catch. The live work graph is the ticket board; the roadmap's operation-family support matrix is what states delivered capability per family.

- semantic/index lowering and fusion search: [capability registration](../tickets/prototype-operation-capability-registry.md),
  [checked refinement](../tickets/prototype-semantic-index-refinement.md),
  [canonical index regions](../tickets/prototype-canonical-index-region-slice.md),
  [generic index oracle](../tickets/prototype-index-region-reference-oracle.md),
  [generic region formation](../tickets/prototype-generic-region-formation.md),
  [legality evidence](../tickets/prototype-fusion-legality-and-numerical-proof.md),
  and [complete cover enumeration](../tickets/prototype-region-cover-enumeration.md);
- mature symbolic indexing: [ShapeEnv-backed index bindings](../tickets/implement-shapeenv-index-bindings.md)
  followed by [typed index-domain predicates and proof exchange](../tickets/implement-index-domain-predicates.md);
- physical/kernel/program layers: [target feasibility](../tickets/prototype-target-feasibility-authority.md),
  [checked schedules](../tickets/prototype-scheduled-region-ir.md),
  [physical implementations](../tickets/prototype-physical-implementation-frontier.md),
  [complete physical-plan selection](../tickets/prototype-complete-physical-plan-selection.md),
  [structured KIR](../tickets/prototype-structured-kir-slice.md), and
  separate [kernel-program](../tickets/prototype-kernel-program-ir.md) and
  [artifact-program](../tickets/prototype-artifact-program-model.md) models;
- artifact and Metal AOT: [neutral codec](../tickets/prototype-neutral-artifact-codec.md),
  [MSL lowering](../tickets/prototype-metal-kir-lowering.md),
  [numerical realization](../tickets/prototype-metal-numerical-realization.md),
  [offline driver](../tickets/prototype-apple-aot-driver.md), and
  [bundle assembly](../tickets/prototype-metal-bundle-assembly.md);
- runtime safety: [artifact validation](../tickets/prototype-runtime-artifact-validation.md),
  [preflight](../tickets/prototype-metal-runtime-preflight.md),
  [routing commit](../tickets/prototype-runtime-routing-commit.md), and
  [execution mechanics](../tickets/prototype-metal-runtime-execution.md); and
- inline delivery: [proc-macro frontend](../tickets/prototype-inline-proc-macro-frontend.md),
  [expansion cache](../tickets/prototype-expansion-content-cache.md),
  [artifact-family selection](../tickets/prototype-artifact-family-delivery.md),
  and the [complete inline proof](../tickets/prototype-inline-aot-integration-proof.md).

### Q-SEM-002 — Built-in algebraic capability declarations

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestone 1.
- Close: complete operation/dtype/signature reassociation and commutativity
  matrix, plus ADR 0101 decision 3's parameterized elementary-identity
  capability law: an operation-owned functional equation together with that
  equation's real-domain side condition, all with verifier tests.

### Q-SEM-003 — First-profile operation and dtype support

- Owner/track: [Numerical semantics](numerical-semantics.md) owns tuple meaning; [dtype support maturity](dtype-support.md) owns delivered state by layer; Milestones 1 and 2Q own profile progression. Built-in recognition policy is settled by ADRs 0026–0038, while registration and execution remain separate implementation claims. The bounded governed-F32 and strict-affine U4/U8 slices do not select a first production profile.
- Close: a named first production consumer has an explicit operation/dtype/signature allowlist, and every tuple that profile requires has delivered reference evaluation, optimizer legality, backend execution, runtime semantic enforcement where required, target dispatchability, and bounded conformance evidence. Recognized but unselected families remain visible in the ledger without blocking closure.

### Q-SEM-004 — First-profile transcendental tuples

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestones 1–2.
- Close: operation/dtype/accuracy allowlist with reference and backend
  conformance evidence.
- **Restated 2026-08-04 — both reasons this question gave for staying open were discharged on 2026-08-01, and the remainder is narrower and different in kind.** The [Metal elementary-function accuracy guarantee](research/numerics/metal-elementary-function-accuracy.md) record quotes Apple's normative Table 8.1 for `exp` (≤ 4 ULP under Apple's own ULP definition), `rsqrt`, and division at F32 under the governed compile flags. The two derivations that guarantee needed are now supplied and are recorded in [the operation-family support matrix](roadmap.md#operation-family-support-matrix)'s transcendental row: the cross-metric gap is crossed by `RegisteredImplication::ScaledMetric` in `crates/tiler-compiler/src/target/accuracy.rs`, registered once with its derivation attached and reused by a second operation rather than duplicated; and Metal §8.2's unfixed rounding mode does not bind an entry stated as a ULP bound at all, because a correctly rounded result under either admitted mode is a member of the faithful pair, so the promised set is exactly `AccuracyContractForm::Faithful` and a faithful contract is metric-free. The reference half is likewise no longer wholly open: `crates/tiler-reference/src/accuracy.rs` supplies the certified enclosures and the three-way conformance decision ADR 0042's exact comparison needs, and three families — `tiler::silu-f32@1`, `tiler::rms-norm-f32@1`, `tiler::softmax-f32@1` — are reference-evaluated against exact rational enclosures for their subordinate transcendentals.
- **What remains, exactly.** Every one of those three declares the accuracy of a *subordinate* elementary function under its own operation key and mints no general one; `no_general_exponential_or_sigmoid_key_is_registered`, `no_layer_normalization_rsqrt_mean_or_bias_key_is_registered`, and `no_general_exponential_maximum_reduction_or_log_softmax_key_is_registered` are the checks that hold that line. So the machinery on both sides is delivered and exercised, and what this question still owns is the *selection* it always named — which general operation, dtype, and accuracy tuples enter the first profile, and the exceptional-value contract each owes. A general `Exp`, `Log`, `Sin`, or `Gelu` key has no reference evaluator for the same reason it has no backend row: it is not registered. The matrix row is the tracking record and states the same thing from the delivery side.

### Q-SEM-005 — First-profile float-to-integer tuples

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestones 1 and
  2Q.
- Close: family/source/destination/rounding allowlist with exceptional and
  boundary tests.

### Q-SEM-007 — Concrete transactional rewrite API

- Owner/track: [Operation extensions](operation-extensions.md). **Retargeted 2026-08-04: the engine half is delivered and only the public boundary is open.** The previous owner [`implement-transactional-rewrite-engine`](../tickets/implement-transactional-rewrite-engine.md) closed `done`, as did its two children [`generalize-the-normalize-transaction-to-alternatives`](../tickets/generalize-the-normalize-transaction-to-alternatives.md) and [`implement-first-algebraic-rewrite-portfolio`](../tickets/implement-first-algebraic-rewrite-portfolio.md), which left this question owned by terminal tickets — unowned in fact, the same way Q-ART-008 and Q-ART-004 were.
- **What landed, and it is every property this question's close condition named.** `crates/tiler-compiler/src/rewrite.rs` carries a governed `RewriteRuleIdentity` with an output-affecting revision under a length-prefixed encoding no two provider/rule pairs can collide in, a whole-candidate-program `RewriteProposal`, the `RewriteRuleProvider` trait at one rule per provider, an attribution check that fails the whole batch rather than filtering it, and a `RuleRegistry` that refuses a duplicate identity and iterates in canonical identity order so the alternative set is reproducible. The transaction is the normalize stage generalized, so termination, budget exhaustion, rollback, and revalidation through the checked `SemanticProgramBuilder` are the properties that stage already proved rather than new ones. [Operation extensions](operation-extensions.md) states the same contract normatively — rewrites are transactional, reverified, cycle-bounded, and budgeted — and the live algebraic portfolio derives every `RuleRef` and identity fact from the complete `RewriteRuleIdentity`, including provider, rule key, and revision, in deterministic rule order.
- **Open, and it is a public boundary rather than an implementation.** `crates/tiler-compiler/src/lib.rs` declares `mod rewrite;` without `pub`, so there is no *API*: an out-of-crate authority cannot register a provider at all, and the seam is a staging state under ADR 0074 convention 7 rather than a statement of intent. Reproduce with `grep -n 'mod rewrite' crates/tiler-compiler/src/lib.rs`.
- Trigger: the first rule provider that must live outside `tiler-compiler`. Closure is then Tom's under [ADR 0075](decisions/0075-scope-public-boundary-approval-by-change-category.md) — the exact facade over rule identity, proposal, provider trait, and registry — and not a worker's, because promoting a `pub(crate)` authority is on that decision's always-ask list. The one residue that is not a boundary question: CSE's canonical normalization records still emit stage-owned `&'static str` constants rather than deriving their explain key from `RewriteRuleIdentity`, so those particular records carry no provider identity.

### Q-SEM-009 — Decomposition versus direct access lowering

- Owner/track: [Operation extensions](operation-extensions.md), Milestone 1.
- Close: per-built-in capability/decomposition table with equivalence tests.

### Q-SHAPE-001 — Runtime extent specialization policy

- Owner/track: [IR](ir.md), Milestones 2–3. Runtime ABI parameters remain the
  default unless specialization is deliberate.
- Close: first-profile policy with identity, guard, and routing tests.

### Q-SHAPE-002 — First-profile composed-axis factor bindings

- Owner/track: [IR](ir.md), Milestone 2.
- Close: static/runtime binding allowlist and complete sourceability tests.

### Q-PLAN-002 — Shared-work duplication

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  [`implement-general-dag-partitioning`](../tickets/implement-general-dag-partitioning.md).
- Close: legality plus an uncertainty-bearing analytical cost rule checked
  against the exhaustive oracle. Calibrated device selection becomes
  authoritative only after the separate calibration ticket's activation
  conditions and measurements pass.

### Q-PLAN-004 — Coexisting reductions in one kernel

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  [`implement-parallel-reduction-strategies`](../tickets/implement-parallel-reduction-strategies.md).
- Close: topology/order/resource compatibility matrix with positive and
  negative verifier cases.

### Q-PLAN-005 — Physical multi-output kernels

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  [`implement-general-dag-partitioning`](../tickets/implement-general-dag-partitioning.md).
  Semantic multi-result programs are already accepted.
- Close: schedule, ABI, runtime profile, and measured value proof.

### Q-PLAN-007 — First Metal capability keys and feasibility rules

- Owner/track: [Metal backend](backends/metal.md), Milestone 2. **Retargeted 2026-08-04.** The [`target-neutral baseline`](../tickets/prototype-target-neutral-baseline-slice.md) and [`Metal AOT proof`](../tickets/prototype-metal-aot-slice.md) both closed `done` having implemented one private named prototype fixture and said so explicitly, which left this question owned by terminal tickets while the mature profile stayed open. The grid-axis row's authority landed: [`establish-an-upper-bound-authority-for-the-metal-grid-axis-row`](../tickets/establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md) (`done`) moved `FIRST_MACOS_APPLE9`'s bound from a conservative representability floor to a retained measurement at 268,435,456, and [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](../tickets/raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md) (`done`) carried its boundary-test consequence. The live owners of what remains are [`declare-metal-subgroup-realization-facts-in-the-target-profile`](../tickets/declare-metal-subgroup-realization-facts-in-the-target-profile.md) for the subgroup facts a feasibility predicate reads, [`reconcile-the-operation-identity-and-governed-key-grammars`](../tickets/reconcile-the-operation-identity-and-governed-key-grammars.md) for whether a key composed from a legally registered operation is always a legal governed key, and [`activate-measured-reduction-selection-from-a-target-cost-row`](../tickets/activate-measured-reduction-selection-from-a-target-cost-row.md) for whether a profile may carry a *cost* row at all. **Updated 2026-08-07.** [`calibrate-and-activate-parallel-reduction-selection`](../tickets/calibrate-and-activate-parallel-reduction-selection.md) ran the crossover sweep the widened domain admitted and it landed a measured contour rather than an open question — parallel reduction plans win by up to 50.7 times and lose by up to 1.78 — but it also found that acting on the result needs a profile row whose silence would render a profile unexecutable for a quantity no feasibility predicate reads, which is the *opposite* failure direction from every capability key this question is about. That is what the successor ticket has to settle before this question can close on a profile carrying one.
- Close: governed profile/schema with boundary tests and stable explain reasons. A row established by a representability floor rather than by a real upper-bound authority does not satisfy this, because a bound nothing measured is not a capability key.

### Q-PLAN-009 — First-profile capability providers and phases

- Owner/track: [Architecture](architecture.md), Milestones 2–3. The general phases are settled by ADR 0043. **Retargeted 2026-08-04.** The target-neutral baseline, Metal AOT, and [`runtime proof`](../tickets/prototype-metal-runtime-proof.md) supplied the bounded enabled-key/provider/phase subset their proofs required and all closed `done`, which left this question owned by terminal tickets. **Corrected 2026-08-09 after the retargeted owners themselves moved terminal.** The composition, installation, offered-versus-selected disclosure, and scalar-seam disposition are now delivered history rather than live ownership: [`expose-explicit-backend-provider-and-selection-policy-composition`](../tickets/expose-explicit-backend-provider-and-selection-policy-composition.md) closed after splitting its one surviving family-policy key; [`drive-an-external-physical-implementation-provider-through-compilation`](../tickets/drive-an-external-physical-implementation-provider-through-compilation.md) and [`disclose-offered-and-selected-physical-provider-sets-separately`](../tickets/disclose-offered-and-selected-physical-provider-sets-separately.md) landed the provider path and disclosure; and [ADR 0105](decisions/0105-retire-the-scalar-lowering-provider-seam.md) retired the scalar provider family. **Corrected 2026-08-11 after Tom resolved that split.** Provider-family policy is no longer open: [`decide-whether-a-loading-host-may-state-several-backend-families`](../tickets/decide-whether-a-loading-host-may-state-several-backend-families.md) fixes one explicitly selected backend environment per routing attempt, and [`express-the-typed-backend-family-selection-policy`](../tickets/express-the-typed-backend-family-selection-policy.md) is closed `wontdo` because neither the loader nor a consumer facade may silently fall back across families. What remains here is the phase question, owned by [`name-a-host-process-availability-phase`](../tickets/name-a-host-process-availability-phase.md), and the conformance population after the explicitly routed portfolio, owned by [`publish-the-backend-provider-conformance-suite`](../tickets/publish-the-backend-provider-conformance-suite.md).
- Close: complete enabled-key/provider allowlist and preflight tests.

### Q-PLAN-011 — CPU execution and vector profile

**Moved out of "Deferred until an explicit trigger" on 2026-08-04, because its trigger fired.** It sits here, among the milestone-owned implementation contracts, since ADR 0093 gives it a correctness-derived direction and what remains is implementation, tests, and the public-boundary acceptances that follow them.

- Owner: [CPU backend](backends/cpu.md). **The trigger fired and this question is re-owned rather than left as a deferral (2026-08-04).** Its stated trigger was "the CPU backend enters the active roadmap", and it has: [`prototype-a-bounded-scalar-cpu-backend-vertical`](../tickets/prototype-a-bounded-scalar-cpu-backend-vertical.md) executed one bounded scalar CPU implementation from a declared CPU target profile to bit-for-bit agreement with `tiler-reference` against `crates/` unmodified, and [ADR 0093](decisions/0093-bind-vector-lanes-to-the-map-or-the-contributor-partition.md) was accepted by Tom on 2026-08-01. That spike's own graph-maintenance note recorded `docs/open-questions.md` as deliberately unchanged, which is why this entry stood while the trigger was already spent.
- **What is accepted and what is not, kept separate.** ADR 0093 accepts a *model* — seven numbered decisions binding vector lanes to the map or the contributor partition and never to the combine order — and not one line of public Rust; the two research records it cites enumerate thirteen distinct public-boundary items between them, each of which returns to Tom under [ADR 0075](decisions/0075-scope-public-boundary-approval-by-change-category.md) when its implementation does. [ADR 0110](decisions/0110-split-the-bounded-scalar-cpu-backend-at-the-production-process-boundaries.md) separately accepts the bounded scalar production package, refusal, and resource-policy boundary. [The CPU backend contract](backends/cpu.md) is therefore **mixed**: scalar ownership is accepted and spike-only, while vector and threaded execution remain proposed.
- Live owners of the delivery: [`declare-cpu-vector-realization-facts-in-the-target-profile`](../tickets/declare-cpu-vector-realization-facts-in-the-target-profile.md) for the profile declaration a lane-bound schedule composes against, [`admit-vector-lane-bindings-into-the-schedule-vocabulary`](../tickets/admit-vector-lane-bindings-into-the-schedule-vocabulary.md) for the schedule half, and [`admit-fixed-vector-ssa-and-unmasked-memory-into-kernel-ir`](../tickets/admit-fixed-vector-ssa-and-unmasked-memory-into-kernel-ir.md) for the first fixed-width kernel-IR half. Four vocabularies the spike measured as having no CPU referent — a host-process availability phase, an execution policy that can say "interpreted image", an Apple-shaped payload provenance, and a capability axis set with no target-triple, ABI, data-layout, or vector-width axis — are seams rather than blockers; the first is Q-PLAN-009's, and the CPU profile answers the GPU-only workgroup and local-memory axes with `1` and `0` until the rest are widened.
- Close: a target profile declares the corrected operation-specific vector realization requirements, lane bindings and lane-typed masked memory are admitted into the schedule and kernel vocabularies with contributor padding identity proved rather than declared, an eligible live host earns the required ISA/features, one CPU vector program executes and agrees bitwise with `tiler-reference`, and the proposed SIMD/threaded part of [the CPU backend contract](backends/cpu.md) becomes accepted. Public-boundary items reach Tom along the way; none is self-accepted, and an accepted scalar package boundary or vector model is not an implemented vector backend.

### Q-PLAN-013 — Replayable schedule transforms

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  Milestone 3.
- Close: versioned transform vocabulary with deterministic replay/golden tests.

### Q-KIR-001 — Conservative uniformity analysis

- Owner/track: [IR](ir.md), Milestone 4.
- Close: scope-sensitive rules with reduction, barrier, convergence, and
  negative-control tests.

### Q-RUNTIME-002 — Affine-strided Candle layouts

- Owner: [Candle integration](integration/candle.md). **Restated 2026-08-04 as a demand-triggered widening.** [`prototype-candle-metal-adapter`](../tickets/prototype-candle-metal-adapter.md) closed `done` having delivered the contiguous first profile exactly as scoped, which left this question owned by a terminal ticket; no live ticket owns affine-strided support, and that is correct rather than an omission. The boundary is delivered rather than merely intended: the adapter refuses `AffineStridedLayout` and `BroadcastView` by name with typed refusals, never copying, relayouting, or approximating, and it accepts a contiguous view at a **nonzero** start offset, which is the positive half the refusal probes cannot supply. That contract already states the ordering a widening would follow — an aligned vectorized contiguous variant, then a scalar tail-capable contiguous variant, then a general affine-stride variant.
- One finding is worth keeping beside the trigger, because the first probe written for it tested the wrong refusal: Candle's `Layout::is_contiguous` ignores the stride of any extent-1 axis, so transposing a 1×N tensor yields an N×1 view Candle still calls contiguous. A genuinely non-contiguous case narrows the inner axis of a multi-row tensor.
- Trigger: a selected region whose Candle operand is non-contiguous and cannot be soundly handled by the refusal — that is, where falling back to Candle's own kernels is not available or not acceptable. Closure then needs exact stride/offset/alias predicates and guarded differential tests, which is what the original close condition named.

### Q-PKG-002 — Rust data APIs and operation capability traits

- Owner/track: [Operation extensions](operation-extensions.md), Milestone 0A. ADRs 0005 and 0044 settle the conceptual split, and [ADR 0078](decisions/0078-name-the-intended-public-extension-seams.md) — accepted 2026-07-25 — settles which surfaces are *intended* seams at maturity, so intent is no longer being decided case by case at promotion time. **Retargeted 2026-08-04.** The [`resolved-type registry`](../tickets/prototype-resolved-value-type-registry.md), [`typed handles`](../tickets/prototype-typed-value-handles.md), and bounded [`shaped-value API`](../tickets/prototype-shaped-value-api.md) all closed `done` with integrated compile/UI proofs, which left this question owned by terminal tickets while the half it names as its close condition — concrete *visibility* — stayed open.
- **What is open is the promotion, not the design.** ADR 0074 convention 7 keeps an authority crate-private until Tom accepts its exact facade, so a cross-crate-ready trait behind a private module is a staging state rather than a statement of intent; that record names `frontier::PhysicalImplementationProvider` as exactly such a case. Every promotion is therefore Tom's under [ADR 0075](decisions/0075-scope-public-boundary-approval-by-change-category.md), and the live promotion node is [`accept-the-public-route-requirement-answer-boundary`](../tickets/accept-the-public-route-requirement-answer-boundary.md) at `deferred`; [`resolve-or-retire-the-scalar-lowering-provider-seam`](../tickets/resolve-or-retire-the-scalar-lowering-provider-seam.md) closed on 2026-08-06, when the elimination retired the seam under [ADR 0105](decisions/0105-retire-the-scalar-lowering-provider-seam.md), so ADR 0078's one open capability-trait question is answered; and [`audit-dead-code-admissions-after-public-boundary-promotions`](../tickets/audit-dead-code-admissions-after-public-boundary-promotions.md) is current `todo` work for the full sweep after its trigger fired.
- Close: concrete visibility and trait ergonomics with compile/UI tests. Reviewed visibility is not stabilization, and this question does not close on a reviewed draft.

## Bounded evidence gates

### Q-PLAN-008 — Multi-family target-profile compatibility

- Owner/track: [Architecture](architecture.md), Milestone 7.
- Close: versioned capability-intersection rules backed by cross-family,
  device, and OS measurements; unmeasured guarantees remain unknown.

### Q-ART-003 — Additional embedding-platform matrices

- Owner/track: [Artifact ABI](artifact-abi.md), Milestone 7.
- Run when: proposing new delivery platforms or changing the current 1 MiB per invocation and 32-invocation/3.2 MiB package gates. **Evaluated 2026-08-04 and unfired, with the headroom measured rather than assumed:** the largest real artifact is 47,803 bytes, 4.56% of the per-invocation ceiling, so nothing is near a gate. **Re-evaluated 2026-08-06 and still not fired, with the headroom three and a third times smaller than that:** the largest real artifact is **159,037 bytes, 15.17%** of the 1,048,576-byte per-invocation ceiling, and the member [the embedding note](research/embedding/self-contained-embedding.md#5-the-gates-as-numbers) embedded is 146,324 bytes, 13.96%. This is a re-evaluation of a recorded number and **not** a trigger firing, and the distinction is the run-when condition above read literally: no delivery platform has been proposed and neither gate has changed, so nothing this question watches has happened. What moved is the artifact encoding — the band was re-derived at `8bd720b8` by re-running the producer that set it, with every carried `metallib` byte-identical to the 2026-07-31 record, so all of the growth is what the canonical manifest describes rather than backend output ([the hot-path note's Section 9](research/cache/hot-path-efficiency.md#9-the-re-run-at-the-re-derived-band-2026-08-06) attributes it). The 4.56% above is retained as the 2026-07-31 measurement it was. A reader carrying it forward would be reading a figure three and a third times out of date and, more consequentially, would be inferring an order of magnitude of headroom that no longer exists: roughly two thirds of one more threefold growth exhausts the per-invocation gate. Whether the encoding owes a budget is [`attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget`](../tickets/attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget.md)'s, not this question's. Note what does *not* fire this: an Apple artifact *family* is not a delivery platform in this question's sense, because two declarations differing only in `MetalTargetFacts::platform` share a profile key and produce a byte-identical canonical descriptor. iOS delivery is therefore Q-ART-011's axis, not this one, and the ticket that would exercise it is parked on hardware regardless.

**Q-ART-006 — rust-analyzer cold and warm expansion costs — closed 2026-08-04**, into [the frontend contract](integration/frontends.md), which holds the matrix rather than this index. The availability blocker was resolved by the [build-tool exercise](research/cache/build-tool-exercise.md), which drove real expansions through the pinned toolchain's own `rust-analyzer-proc-macro-srv` — the process that expands, and one that ships with the pin even though the LSP binary is not a pinned component — under both drivers. The *edit* column this question was still waiting on was supplied on 2026-08-01 by [`avoid-toolchain-resolution-on-a-warm-expansion-cache-hit`](../tickets/avoid-toolchain-resolution-on-a-warm-expansion-cache-hit.md) over a real LSP session — initialize, `didOpen`, then `didChange` edits each followed by a `textDocument/semanticTokens/full` round trip, with expansions counted exactly because a resolution is exactly five shim lines. That ticket closed `done` carrying no graph-maintenance section, so nothing propagated the result, which is why this entry stood after it was answered. The contract now states the whole matrix: a settled in-region edit costs one expansion and a 137–217 ms round trip against 10–16 ms for the same region with no `deliver` statement; a warm `cargo check` is 170–190 ms whole-crate; `Toolchain::resolve` is 44–97 ms of it; and the compiler-invocation column is four `xcrun` calls plus two direct `--version` executions per resolution, with the warm build's fifth call attributed to rustc's own linker SDK query rather than to Tiler. One cell is absent and is parked in that contract with its own trigger rather than here: no wall-clock number exists for the **cold interactive** round trip — the first expansion of a region against an empty cache inside the IDE — and the trigger is an analysis-stub proposal, which would have to carry that number.

### Q-ART-007 — Apple cross-machine, patch-toolchain, and runtime-compiler evidence

- Owner/track: [Metal backend](backends/metal.md), Milestone 7.
- Close: a reproducibility and compatibility matrix over four independent axes — the machine and GPU, the Xcode toolchain patch version, the **OS build**, and the **installed simulator runtime version**. The last two are axes in their own right because a host that never changes Xcode can still change two of its three Metal compilers: the offline driver ships with Xcode, the macOS runtime compiler with the OS, and a booted simulator's with that runtime, as [Metal backend](backends/metal.md#compiler-provenance-and-the-runtime-compiler) records. Read without them, this question is satisfied by a matrix that holds the OS constant, and the numerical harness then announces an environment-row difference and declines to compare rather than confirming agreement.
- Closing measurement: re-run [`numerical_probe.py`](../spikes/apple-targets/numerical_probe.py) on a host whose OS build differs while its Xcode build does not, and again against a second installed simulator runtime version, comparing the resulting `environment.family.<name>.runtime_compiler_build` rows with the retained record. A run whose rows are unchanged has not exercised the axis and does not close it.

### Q-ART-011-E — Apple deployment-minimum compatibility experiment

- Owner/track: [Metal backend](backends/metal.md), prerequisite to Q-ART-011.
- Close: record whether incompatibility fails at library load or pipeline
  creation across old/new macOS, iOS devices, and simulators.

## Deferred until an explicit trigger

### Q-SEM-006 — Additional quantization schemes

- Owner: [Numerical semantics](numerical-semantics.md).
- Trigger: strict affine Milestone 2Q is complete and a named workload requires
  another exact scheme.

### Q-SEM-011 — Semantic effects and resource tokens

- Owner: [Operation extensions](operation-extensions.md).
- Trigger: the first stateful, mutating, or hidden-random operation proposal;
  closure requires ordering, liveness, verification, ABI, and failure rules.

### Q-SEM-012 — Semantic modules, calls, and control flow

- Owner: [IR](ir.md).
- Trigger: a workload requires reusable graph functions, interprocedural
  optimization, recursion, or structured control flow.

### Q-SEM-013 — Differentiation ownership

- Owner: [Architecture](architecture.md).
- Trigger: backward-kernel compilation enters the roadmap; closure requires a
  product-layer and semantic/autograd decision.

### Q-SEM-015 — Tensor contraction: matmul, batched matmul, and einsum

- Owner/tracking: the [Milestone 6 framing](roadmap.md#framing-what-a-tensor-contraction-family-would-impose). **Retargeted 2026-08-04**: [`scope-einsum-contraction-support`](../tickets/scope-einsum-contraction-support.md) filed this question and closed `done` the moment it existed, which is correct for a scoping ticket and wrong as a standing owner — the original audit missed it because the line also named the framing and so did not read as unowned. The framing is the live authority for the planning half, and [`decide-whether-a-contraction-may-consume-more-than-two-operands`](../tickets/decide-whether-a-contraction-may-consume-more-than-two-operands.md) at `deferred` is the live owner of the one reserved semantic choice named below. The [operation-family support matrix](roadmap.md#operation-family-support-matrix) records this family at R6 for a whole-program contraction occurrence since 2026-08-01 — a registered identity with a host reference evaluator, all three of the pinned workload's index structures admitted as structure values, an eighth governed lowering capability, and the `direct` realization's schedule constructs and Metal emission — with no fusion role and no execution row. What remains of the planning half is what this question still owns: contraction-order exploration, GEMM recognition, layout-conversion costing, and the `tiled` schedule. "Contraction" here always means the tensor sense — summation over indices shared by two or more operands — and never ADR 0015's fused-multiply-add permission, which is a separate field of the numerical contract that happens to govern a tensor contraction's own per-contributor step.
- Trigger: a named workload or frontend lowering requires a tensor contraction — fired by the pinned L1 workload. Closure of the semantic half needs an accepted decision fixing what establishes a contraction's identity, what its operation definition rejects at construction, and which access relation it emits; none of those depends on a backend. That decision must settle three choices, and two of the three are now settled. The first: [ADR 0087](decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) accepts one keyed family carrying a renaming-invariant index-structure attribute, on the L2 derivation's three-structure evidence. The third: [ADR 0095](decisions/0095-decline-a-distributivity-permission.md) **declines** a distributivity permission, so [Numerical semantics](numerical-semantics.md#distributivity-is-outside-the-order-contract) continues to define the dimension, admit no permission for it, and reject contraction-chain regrouping — now as a decided position rather than a reserved one, with contraction ordering remaining a planning question within one semantic contraction. Its reopening trigger is the first workload whose natural spelling is a directly regroupable chain, and its dependent question, [`decide-whether-distributivity-directions-share-one-permission`](../tickets/decide-whether-distributivity-directions-share-one-permission.md), does not arise under a decline and stays parked. Still reserved from the framing, and the only one of the three left: whether a semantic contraction node may consume more than two operands, owned by [`decide-whether-a-contraction-may-consume-more-than-two-operands`](../tickets/decide-whether-a-contraction-may-consume-more-than-two-operands.md). The three are independent: the distributivity derivation, and therefore ADR 0095's decline, holds under either answer to the multi-operand choice.
- Gate: no contraction *planning* work — contraction-order exploration, GEMM recognition, layout-conversion costing, or direct and tiled schedules — may be scheduled until [`prototype-optimizer-conformance-gate`](../tickets/prototype-optimizer-conformance-gate.md) closes and a backend has executed a compiled program, which [`prototype-metal-aot-slice`](../tickets/prototype-metal-aot-slice.md) and [`prototype-metal-runtime-proof`](../tickets/prototype-metal-runtime-proof.md) own. **All three are `done`, so the gate is open, and `realize-the-contraction-through-the-appendable-direct-path` was the first work to pass through it on 2026-08-01.** The two limits below are the evidence the gate rested on, and both have since been lifted deliberately rather than eroded: `normalize_contraction` in `crates/tiler-compiler/src/request.rs` is a third recognized whole-program strategy admitting exactly two inputs, and `governed_index_access_capabilities` registers an eighth capability covering a contraction occurrence. What the gate still holds back is everything the `direct` path did not deliver — contraction-order exploration, GEMM recognition, layout-conversion costing, and the `tiled` schedule, the last of which additionally waits on [`admit-the-first-typed-synchronization-point-and-atomic-target-authority`](../tickets/admit-the-first-typed-synchronization-point-and-atomic-target-authority.md). The original statement of the two limits follows, preserved because it is what the gate's derivation cited: `normalize_serial_sum` rejected any program that did not have exactly one input, so a binary contraction could not reach the compiler at all, and no registered lowering capability covered a contraction occurrence, so resolution would fail closed rather than lower one. **Corrected by `correct-the-surviving-stale-one-contract-claims`.** The second limit previously read that `crates/tiler-compiler/src/capability.rs` and `crates/tiler-compiler/src/legality.rs` are draft authorities with no in-crate production caller. That was true when written and was falsified by `wire-capability-and-refinement-into-compile-path`: `pipeline::compile` calls `resolve_lowering`, which consumes both modules, and governed index-access providers are registered — four then, and six since `admit-the-reindex-and-broadcast-operation-families` added one for each structural family. [The Milestone 6 framing](roadmap.md#framing-what-a-tensor-contraction-family-would-impose) records the same correction and is the surviving statement of it. The gate is unaffected, because what closes it is the absence of a capability for a *contraction* occurrence rather than the absence of a caller — and stating it that way is the invariant, which the fifth and sixth registered providers did not silently falsify. Both limits belong to that gate. [Fusion and scheduling](compiler/fusion-and-scheduling.md) independently requires contraction planning to follow, not precede, the boundary-contract and cost infrastructure.

### Q-SHAPE-004 — Dynamic-rank semantic values

- Owner: [IR](ir.md).
- Trigger: a concrete workload cannot be represented as static-rank variants.

### Q-SHAPE-005 — Device-produced shapes and indirect dispatch

- Owner: [IR](ir.md).
- Trigger: a selected operation requires device-produced extents; closure needs
  a host/device `ShapeProgram`, synchronization, publication, and guard contract.

### Q-SHAPE-006 — Finite piecewise access maps

- Owner: [IR](ir.md).
- Trigger: a named workload is not expressible in the admitted access language. **Evaluated 2026-08-04 and unfired**, and the near misses are named so the next reader does not have to re-derive them. A tensor contraction needs no piecewise map: each operand map is a pure projection and permutation of the coordinate vector using no index arithmetic at all. The sub-tensor selection family's *symbolic-offset* half is blocked on a **carrier** gap rather than an expressiveness class — `SourcedExtent` is the only `IndexNode` variant that carries a possibly-symbolic extent and it appears in no other position, so `t + k` for a literal `k` is expressible and `t + C` for a bound symbol is not. Its literal-offset half needed no piecewise map and landed on 2026-08-04 as `tiler::slice-f32@1`, which refuses the symbolic form by name under `slice.selection.symbolic-offset-unsupported` rather than approximating it; the rotary-slice occurrence still needs the carrier and is what would widen it. Elementwise epilogues are blocked in the physical layer rather than the index language.
- The one live piecewise *pressure* is resolved and does not fire this trigger. [Concatenate fusion role and lowering](research/indexing/concatenate-fusion-role-and-lowering.md) ran the elimination on 2026-08-05 at `d5960e81` and selected the partitioned write over the piecewise read, so the concatenate lowering asks nothing of the coordinate-expression language: an operand's write coordinate on the concatenated axis is `t + offset` for a literal offset, and `IndexNode::LinearCombination`'s exact-integer constant already carries it. The piecewise read was eliminated as **insufficient rather than merely expensive** — the case selects a different operand *tensor* per coordinate, which `AccessData`'s single `tensor` field does not express and which ADR 0046's piecewise reservation, being over the map rather than over the tensor, does not reserve; the alternative spelling that reads every operand and selects is refused by the bounds proof and additionally needs a predicate dtype `RQ-OP-03` owns. What the surviving alternative owed was a write-ownership contract rather than an access class, and that contract has since landed: [`admit-sub-range-write-domains-for-unequal-partitions`](../tickets/admit-sub-range-write-domains-for-unequal-partitions.md) admits a write domain that is any subset of the region's parallel dimensions (`InvalidWriteDomain` survives meaning only that a domain names a non-parallel dimension), several roots may partition one output, and joint coverage and disjointness are decided by interval reasoning at verification under `OutputPartitionUncovered`, `OutputPartitionRangesOverlap`, and `OutputPartitionDoubleWritten`. Of the four sites that refused the partitioned write when the elimination ran, the two construction-time refusals are discharged, the total-coverage ownership walk governs only a root that owns its output alone, and `MultipleWriters` still refuses one value written by two *stages* at the program layer, which partitioned accesses within one region do not trigger; the research record's refusal list carries the dated correction. The family's own rungs are owned rather than unassigned. **Restated trigger:** this question fires on the first family whose *read* map is genuinely case-split over one tensor — padding and cropping, track O-24, is the named candidate and its physical route is already recorded as "a guarded read" — and no longer on the concatenate.

**Corrected 2026-08-08 by [`correct-the-symbolic-coefficient-era-index-vocabulary-claims`](../tickets/correct-the-symbolic-coefficient-era-index-vocabulary-claims.md): the first bullet's symbolic-slice near miss is no longer an index-language carrier gap. `IndexRegionBuilder::sourced_linear_combination` now admits a declared `ShapeSymbol` as a coordinate coefficient or addend, so `t + C` is expressible; its stored `LinearCombination` constant remains exact because the addend normalizes to `C * 1`. The literal slice continues to refuse `symbolic-window` because `SliceAxisSelection::Window` has only a `u64` offset and `decode_axis` rejects the reserved relation before it parses its fields; no constructible semantic slice path carries the addend. Q-SHAPE-006 therefore stays unfired — a symbolic slice is not a case-split read map — and its restated padding/cropping trigger is unchanged.

### Q-SHAPE-007 — Indirect gather/scatter relations

- Owner: [IR](ir.md). **The gather half's trigger has fired, and it is re-owned rather than left standing as a deferral (2026-08-04).** [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](../tickets/admit-an-indirect-gather-family-for-tied-embedding-lookup.md) owns the tensor-data-derived index class, states its own reconsideration trigger as active now, and already enumerates exactly this question's four closure items. The evidence is the pinned workload's *first* operation: one gather per forward pass, `[T]` token IDs into a `[151936, 1024]` F32 table, and with no admitted access class it is not expressible at all. This is a missing access class rather than a missing key over an existing one — [IR](ir.md)'s Layer 2 rejects tensor-data-derived indices by name — which is what makes it this question's subject and not a registration task.
- Trigger, for the half that has **not** fired: scatter. The gather ticket's non-goals exclude scatter and any data-dependent output shape; [`scope-the-scatter-and-indexed-update-family`](../tickets/scope-the-scatter-and-indexed-update-family.md) is the deferred owner and activates when a named workload requires an indexed update. Until then duplicate-write and write-determinism rules stay reserved. Closure of the whole question still needs bounds, duplicate-write, determinism, and validation rules; the gather ticket supplies the first, third, and fourth for reads and states the duplicate-write rule without implementing it.
- **Delivered for reads, 2026-08-07, and the delivery is narrower than closure.** `tiler::gather-f32@1` is registered and reference-evaluated under [ADR 0107](decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md) — `proposed` when this bullet was written, **accepted by Tom on 2026-08-07** — supplying exactly the three rules the bullet above promised for reads plus the stated duplicate-write rule. **What it does not supply is an access class below the semantic layer.** `AccessData` carries one tensor ordinal and `IndexNode` has no variant reading tensor data, so no index region expresses the access and an occurrence fails closed at the request boundary. That no-admission boundary preserves [ADR 0046](decisions/0046-separate-logical-access-from-storage-addressing.md)'s current direct-access guarantees; it does not decide the future representation. **This question therefore stays open on two distinct halves rather than one:** scatter, unfired; and whether the index layer admits a data-dependent access class at all. The second is a new half this delivery *created* by drawing the boundary explicitly, and naming it is the point — before the family landed, "the gather is not expressible" covered both.
- **The proposed answer to the index-layer half was returned for revision, 2026-08-08.** A source audit rejected [ADR 0108](decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md)'s selection of an expression route and its claimed fourth-unknown-reason prerequisite. A fresh access tag can preserve old bytes; `IndexRegionBuilder::prepare_access` establishes rank equality before the verifier's `zip` consumers; the three current unknown reasons do not promise eventual closure; and ADR 0107 permits a data-dependent bound to be statically proved or host-validated through the reusable `decide_gather_index` rule. The proposed expression node was also incomplete as a nested logical read, and its public-boundary census counted private `IndexNode` while omitting authoring and validation surfaces. [`revise-adr-0108-with-a-complete-data-dependent-index-vertical`](../tickets/revise-adr-0108-with-a-complete-data-dependent-index-vertical.md) now owns a comparison of a first-class verified nested read/value expression with an append-only tagged access representation. The current 5/3/3 no-admission boundary stays pinned. The graph trigger is deliberately not Metal emission: design and acceptance precede a separate IR admission, and only admitted IR plus the integer carrier may unblock [`emit-the-indirect-gather-on-metal`](../tickets/emit-the-indirect-gather-on-metal.md).
- **Direction accepted for revision, 2026-08-11.** Static proof remains the first lane; the only initial dynamic lane is a host-visible U32 input validated during explicit preflight into a sealed receipt over an immutable snapshot. That receipt is invocation-bound and cannot become artifact identity or timeless program proof. Mutable zero-copy bindings, device-resident validation, and generalization beyond gather are named deferred work rather than silent omissions. The revision still owns the nested-read versus tagged-access decision and exact ADR 0109 supersession; acceptance, IR admission, the integer carrier, validation receipt, and its public-surface review are separate graph nodes before Metal emission.

### Q-SHAPE-008 — Negative-stride ABI support

- Owner: [IR](ir.md), after Milestone 3.
- Trigger: signed reachable-range proof and backend/runtime layout support.

### Q-PLAN-015 — Advanced buffer reuse and in-place execution

- Owner: [Architecture](architecture.md), after Milestones 3/5.
- Trigger: memory/performance data shows the conservative allocation plan is
  insufficient.

### Q-PLAN-016 — Multi-device and sharded planning

- Owner/tracking: [Architecture](architecture.md),
  [`multi-device-and-sharding-scope-gate`](../tickets/multi-device-and-sharding-scope-gate.md).
- Trigger: a selected product workload requires multiple devices or sharding.

### Q-PLAN-018 — External storage and out-of-core orchestration

- Owner/tracking: [Architecture](architecture.md),
  [`external-storage-resource-scope-gate`](../tickets/external-storage-resource-scope-gate.md).
- Trigger: a selected workload requires file-backed, mapped, evicted, or
  out-of-core tensor resources.

### Q-ART-009 — Binary archives and dynamic Metal libraries

- Owner: [Metal backend](backends/metal.md), Milestone 7.
- Trigger: measured startup or bundle-size cost exceeds a documented gate.

### Q-ART-010 — Public serialized-IR compatibility

- Owner: [Artifact ABI](artifact-abi.md), Milestone 7.
- Trigger: a stable external reader/writer use case exists and IR boundaries
  have settled.

### Q-ART-012 — Catalyst artifact support

- Owner: [Metal backend](backends/metal.md).
- Trigger: an integration requires Catalyst; closure needs an explicit family,
  deployment, `cfg`, compile, and runtime compatibility profile.

### Q-KIR-002 — Asynchronous copies and split-phase barriers

- Owner: [IR](ir.md).
- Trigger: a selected pipelined workload needs overlap not expressible by total
  phases.

### Q-KIR-003 — Target-specific lowering IR

- Owner: [IR](ir.md).
- Trigger: a target operation cannot faithfully lower from common structured
  KIR without polluting target-independent semantics.

### Q-KIR-004 — General CFGs, pointers, calls, and aliasing

- Owner: [IR](ir.md).
- Trigger: a demonstrated workload falls outside bounded structured tensor
  kernels and justifies the larger verifier surface.

### Q-RUNTIME-001 — Candle input arity beyond `CustomOp3`

- Owner: [Candle integration](integration/candle.md), Milestone 5.
- Trigger, **sharpened 2026-08-04 because the loose reading is already satisfied and the question is still correctly deferred**: a region *selected to run through the Candle wrapper* exceeds Candle arity and cannot be soundly partitioned. A region that exceeds the arity and is routed elsewhere does not fire this. The first complete-model program needs eighteen inputs and three outputs and no partitioning fixes it — a partition that fits `CustomOp1` is one operation per dispatch, and Candle's custom-op return type has no position for the retained outputs at all — but that record answers it by *not choosing Candle*: the route it targets is the inline one with its storage seam, and it deliberately files nothing to move the arity. The adapter prototype is `CustomOp1` today, so nothing has yet needed even the three inputs the contract admits.

### Q-RUNTIME-004 — Tracked/autograd fusion

- Owner: [Candle integration](integration/candle.md).
- Trigger: backward support enters an explicitly authorized phase.
