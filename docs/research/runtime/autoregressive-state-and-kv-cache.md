---
schema: "tiler-doc/v1"
id: "tiler.research.runtime.autoregressive-state-and-kv-cache"
kind: "research"
title: "Autoregressive state and KV-cache ownership"
topics: ["runtime", "kv-cache", "state", "prefill", "decode", "language-model", "identity", "placement", "lifetime", "routing"]
catalog_group: "runtime-integration-placement"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.architecture", "tiler.contract.artifact-abi"]
depends_on: ["tiler.research.program-planning.first-attention-program-vertical", "tiler.research.shapes.sequence-extending-tensor-family", "tiler.research.runtime.execution-contract", "tiler.research.program-planning.first-metal-lm-workload"]
ticket: "design-autoregressive-state-and-kv-cache"
---

# Autoregressive state and KV-cache ownership

**Status:** durable design record for rung L5 of the language-model inference ladder. It is a research outcome, not a capability: nothing here registers an operation, admits a key, fixes a normative contract, or authorizes implementation. It moves no row of the [operation-family support matrix](../../roadmap.md#operation-family-support-matrix). What it delivers is the state and execution contract for prefill followed by repeated token decoding — ownership stated per layer, the four contamination and staleness cases tested against the implemented stack, the eliminations that fix the update discipline and the program shape, and nine dependency-ordered delivery tickets.

## Traceability

- **Work record:** [`design-autoregressive-state-and-kv-cache`](../../../tickets/design-autoregressive-state-and-kv-cache.md).
- **Ladder position:** rung L5 of [the roadmap's language-model ladder](../../roadmap.md#the-ladder). Its trigger reads "L4 delivers a complete transformer block"; Tom fired it on 2026-07-31 under the design-rung reading, recorded in the ladder row and in the ticket's outcome. Every delivered rung so far (L1–L4, L7) fired on record delivery rather than on capability delivery, and L4's own record states that the block itself is its delivery ticket 7 rather than part of its outcome — so holding this design behind the attention implementation chain would buy no evidence the state model needs.
- **Inherited, not re-derived.** [The L4 attention program](../program-planning/first-attention-program-vertical.md) supplies the seam: `k_rope` and `v_heads` as retained program outputs o1 and o2 alongside the residual stream, with `S` a separate extent symbol from `T`, so widening `S` is a binding change rather than a graph change. [The sequence-extending family record](../shapes/sequence-extending-tensor-family.md) supplies the semantic mechanism, which is **decided**: extension is a value-producing `Concatenate`, and the windowed write into preallocated state was eliminated as a *semantics*. It hands this rung three implemented refusals — `MultipleWriters`, `ExternalValueWritten`, and the absence of any proof about the untouched bytes of a partially written value. Since this record was written, the accepted fixed two-addend extent relation can state `S == C + T`; launch-preflight consumption is still absent.
- **Governing contracts read as evidence, not edited:** [the runtime execution contract](runtime-execution-contract.md) for the state machine, the transition table, the preflight order, the retention obligations, and the library and pipeline cache keys; [ADR 0051](../../decisions/0051-make-runtime-routing-commit-one-way.md) for the one-way routing commit; [ADR 0047](../../decisions/0047-model-placement-as-physical-properties.md) for the initial one-affinity, one-device, one-ordered-stream execution profile; [ADR 0033](../../decisions/0033-semantic-validation-enforcement.md) for the three execution boundaries and the five-step device completion observation; [ADR 0090](../../decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 4 and `crates/tiler-runtime/src/adapter.rs` for the independently selected runtime adapter and the `LiveExecutionContext` it mints; [System architecture](../../architecture.md#initial-placement-execution-and-buffer-model) for the conservative buffer model; [Q-PLAN-015](../../open-questions.md#q-plan-015--advanced-buffer-reuse-and-in-place-execution) for the in-place deferral; [the L1 workload profile](../program-planning/first-metal-lm-workload.md) for the C1 and B1 rows and the 229,376 bytes per cached token.
- **Inspected source, at this branch's base commit `03a10ae`:** `crates/tiler-runtime/src/{adapter.rs,load.rs}`, `crates/tiler-runtime/src/load/{host.rs,route.rs}`, `crates/tiler-artifact/src/program/{model.rs,facts.rs}`, `crates/tiler-ir/src/program/{model.rs,verify.rs,abi.rs}`, `crates/tiler-ir/src/shape.rs`, `crates/tiler-ir/src/shape/env/constraint.rs`, `prototypes/serial-sum-run/src/proof.rs`.

Claims are labelled **Fact** when traced to inspected source at that commit or to a merged record, **Inference** when derived from stated facts, and **Proposal** when not yet accepted or tested. **This record contains no measurements and takes none.** Every byte figure is arithmetic over quantities L1 and L4 already state, and is labelled as an inference for that reason.

## The state contract, one line each

**Proposal.** A *KV state* is one runtime-instance object per cached tensor. The ten properties the ticket demands, stated so that each is falsifiable on its own:

- **Identity.** `(program interface key, layer ordinal, the live device and context the adapter bound, generation)`. It is deliberately **not** an artifact subject: no packaged identity, cache key, or canonical descriptor names a state, and two states of one artifact are distinguishable only at the runtime instance that holds them.
- **Logical shape.** The semantic input and retained output shapes are `[8, C, 128]` and `[8, S, 128]` F32. [Dynamic KV physical-layout authority](dynamic-kv-physical-layout.md) selects exact-live head-major packing in two alternating capacity-sized buffer banks per logical K or V member. `C` and `S` govern the two dense head strides; capacity remains allocation policy and no physical-layout schema is required.
- **Capacity.** A fixed logical capacity is chosen when the state is created from the row's declared maximum context — 18 at C1, 8,320 at B1-d — and is bounded above by the checkpoint's `max_position_embeddings = 32768`. A single physical `[8, capacity, 128]` allocation was one candidate, not an established representation.
- **Valid range.** `[0, C)` on the logical sequence axis. `C` is the cursor and is the single authority for how many positions the state holds; storage outside that logical range is not observable state.
- **Growth.** `C` advances by exactly `T` on the observed terminal success of the execution that produced the extended value, and never otherwise. `capacity` does not grow; `C + T > capacity` is a typed refusal raised before any program work.
- **Update.** Logically out of place. The execution reads `[8, C, 128]` as a program input and writes `[8, C + T, 128]` as a distinct program output; publication replaces the state's governed storage population and its cursor together, as one step.
- **Placement.** The one symbolic affinity's memory domain under ADR 0047's initial execution profile — the same domain as every other value of the program. A KV state is not a new memory domain, needs no transfer, and introduces no import edge.
- **Aliasing.** The published input storage and candidate replacement storage obey the survivor's complete resource-population alias law. The rejected singular-allocation candidate required distinct old and new allocations; other representations must derive an equally fail-closed rule.
- **Retention.** Both governed storage populations are retained through their exact final device use. Old resources become releasable only after the extending execution's completion condition, never after their last encoder call.
- **Lifetime.** The state outlives every execution that reads it and is owned by the runtime instance. The consumer destroys it. A post-commit failure retires it to a terminal poisoned status that refuses every later bind rather than leaving a plausible one behind.

## The two semantic interfaces

**Fact — the seam is already named and needs no new graph shape.** L4's program has three ordered named outputs, of which o1 `k_rope` and o2 `v_heads` are "the value a KV cache would retain", and it states that a decode step is that program with `K` and `V` arriving as inputs of extent `S ≥ T` instead of being produced.

**Proposal — one decode step's boundary, as the delta from L4's table.** Two inputs are added and nothing is removed. `T` is the new-position count, `C` the cached-position count, `S = C + T` the total context. Every dtype is `tiler::f32@1`. Bytes are per layer; the pinned model has 28.

| # | Added input | Shape | Bytes at C1 step 8 (`C = 17`) | Bytes at B1-d final step (`C = 8,319`) | Where it comes from |
| --- | --- | --- | --- | --- | --- |
| i11 | `k_cache` | `[8, C, 128]` | 69,632 | 34,074,624 | the previous execution's o1 |
| i12 | `v_cache` | `[8, C, 128]` | 69,632 | 34,074,624 | the previous execution's o2 |

| # | Output | Shape | Bytes at C1 step 8 (`S = 18`) | Bytes at B1-d final step (`S = 8,320`) | Produced by |
| --- | --- | --- | --- | --- | --- |
| o0 | `h_out` | `[T, 1024]` | 4,096 | 4,096 | unchanged from L4 |
| o1 | `k_rope` | `[8, S, 128]` | 73,728 | 34,078,720 | `Concatenate(k_cache, k_new, axis 1)` |
| o2 | `v_heads` | `[8, S, 128]` | 73,728 | 34,078,720 | `Concatenate(v_cache, v_new, axis 1)` |

**Proposal — the operation sequence changes in exactly two places.** L4's steps 13 and 14 permute `k_rope_t` and `v_split` into `[8, T, 128]`; each now feeds a concatenation whose other operand is the cached tensor, and the concatenation's result is the retained output that the score contraction (step 15) and the value contraction (step 19) read. Nothing else in the twenty-two steps moves. **Inference — so the decode step is not a second program design.** It is L4's program with two inputs, two occurrences, and one changed extent binding, which is the whole value of having named the seam.

**Fact — the extension is at the block boundary rather than inside it**, so the two concatenations are the only place the state touches the graph, and the graph stays pure acyclic tensor SSA. `OperationEffect` has exactly one variant and is deliberately not `#[non_exhaustive]`, so nothing else was available anyway.

### Prefill is the same program with `C = 0`

**Proposal — one program serves both phases, and prefill binds an empty cache.** Three candidates were tested.

| Candidate | Survives? | Ground |
| --- | --- | --- |
| **P1** — prefill is L4's cache-free program and decode is a second program | **No** | Two artifact identities for one computation. L4's prefill block and the decode step differ only in bound extents and in whether the concatenation's first operand is empty, so P1 packages the same twenty-two steps twice and then owes a separate numerical conformance surface for each. It also makes "continue from a cached prefix", which every serving path eventually needs, a third program rather than the same one. |
| **P2** — one program, `C = 0` at prefill | **Yes** | **Fact —** a zero extent is expressible: `Extent` wraps a plain `u64` and its constructor's own comment records that "Zero represents an empty axis", so nothing in the shape layer refuses `[8, 0, 128]`. What it costs is two named obligations, both small and both owed anyway: the concatenation family must state its zero-extent rule explicitly rather than inheriting whatever the empty case happens to do, and a zero-byte logical value must follow its explicit allocation and ABI policy rather than an implicit null binding, which [the runtime execution contract](runtime-execution-contract.md) already requires of every zero-byte value. |
| **P3** — one program, prefill runs against a state whose capacity is allocated and whose cursor is zero | **Same as P2 semantically** | It is P2 plus the observation that the runtime object exists before the first execution. Kept as the runtime spelling of P2 rather than as a third candidate. |

**Inference — P2's prefill costs no extra bytes in the plan that exists today.** At `C = 0` the concatenation degenerates to a copy of the new rows into the `[8, T, 128]` output allocation, which is exactly the materialization L4's boundary table already lists for o1 and o2 ("**yes** — it is a program result"). P1's saving over P2 therefore exists only in a fused plan where the permute is composed into the consumer's access map and no output is materialized — and no fusion role exists for any family in the block, so that plan is currently unreachable. Choosing P1 would spend a real identity duplication to buy a saving nothing can realize.

**Updated fact — both prefill and decode relations are statable, but only static/root-bound values are checked today.** At prefill `S = T`, and `ExtentRelation::Equal` states that exactly. At decode the accepted `ExtentRelation::AdditiveEquality` states `S = C + T`; `ShapeEnvBuilder::build` rejects inconsistent statically observed root bindings and retains runtime-bound relations. The remaining stale-state gap is launch-preflight consumption: the live runtime state length must be bound and checked against the retained relation before program work.

## What each layer owns

**Proposal — the fact table.** The right-hand column is the load-bearing one: an ownership claim is only useful if moving the fact one layer breaks something nameable.

| Fact | Owner | Why it cannot move |
| --- | --- | --- |
| That a decode step consumes a cached `[8, C, 128]` and produces `[8, S, 128]` by concatenation on axis 1 | Semantic program | It is what the operation *means*. Moving it to the physical plan is the elimination the sequence-extending record already closed: settling meaning by appeal to how a device would execute it. |
| The extent relation `S = C + T`, and the refusal when it does not hold | Semantic program (shape environment) | An extent relation is a property of one program's interface, not of a plan or a device. Held anywhere else, two executions of one artifact could disagree about what `S` means. |
| The selected physical layout and whether the append has a contiguous byte window | Physical plan | Layout is a delivered physical property under ADR 0047. Row-major `[8, S, 128]` is one rejected/provisional candidate, and a semantic graph that fixed any candidate would make portable meaning depend on one backend's addressing. |
| Which realization the value contraction uses at this `S`, and the guard that selects it | Physical plan, packaged as artifact variants | L4's feasibility predicate 2 refuses a contracted extent that is not a multiple of 16 rather than padding it, and `S` changes every step — so this is a per-execution routing decision over a fixed set of packaged plans, which is exactly what applicability guards are. |
| The accessible byte range and launch geometry of every binding, as formulas over the bound extents | Artifact | **Fact.** They are ABI expressions evaluated against the facts a host binds at preflight, so that an evaluation failure is a refusal rather than a post-commit surprise. Precomputing them per step would put a runtime quantity into a packaged artifact. |
| The artifact's canonical identity | Artifact | **Fact.** `encode_identity` takes `&ArtifactEnvelope` and nothing else. No state, cursor, capacity, or bound extent value reaches it. |
| `capacity`, the governed storage population backing the state, its generation, and its retention leases | Runtime instance | ADR 0047 keeps concrete storage objects below the compiler/runtime boundary, and ADR 0051 makes retention through exact final device use an adapter obligation. |
| The live device and context the state is scoped to | Runtime instance, specifically the adapter | **Fact.** `LiveExecutionContext` carries the target profile, the backend family, and the representation, and its documentation states that what else an adapter discovered — a device handle, a queue — "is the adapter's own to keep and never crosses into this crate". So the crate that could compare a state against a device does not know what a device is, deliberately. |
| The cursor `C`, its advance, and the poisoned status | Runtime instance, published to the consumer | The advance is conditional on an observed terminal success, which only the layer holding the completion receipt can witness. |
| The absolute position of the new tokens, and therefore which rows of the rotary table are bound | Consumer | Nothing below the consumer can check it; see the incorrect-position case. The design's answer is to make the consumer state it *once*. |
| Sampling, termination, and the token sequence | Consumer | L1 already fixes that the Tiler workload boundary begins at token IDs. |

**Inference — one combination is silently wrong and must be named rather than left to taste.** The cursor's granularity must equal the program-boundary granularity. A single model-level cursor with per-layer programs advances on a per-layer completion while twenty-seven other layers have not advanced, so a failure part-way through a token leaves a state that is internally inconsistent and whose inconsistency no layer can observe. Per-layer programs need per-layer cursors; one program per step needs one.

## Cache identity contamination, as three checked invariants

**Inference — there are three caches in this stack and the KV state must stay out of all of them, for three different reasons.** Stating them as one rule would hide that only the third has a plausible-looking way to go wrong.

1. **The expansion cache** (`tiler-cache`, content-addressed, immutable, validated on every hit) is unreachable by construction: compilation happens at macro-expansion time, before any state exists in any process. The invariant is that nothing may move a compilation to run time — which the accepted inline developer experience already forbids, since it rules out runtime source JIT.
2. **Artifact program identity** is a pure function of the packaged envelope. **Fact —** `encode_identity(envelope: &ArtifactEnvelope)` is the whole signature, and the facts a host binds are a separate argument to a separate function. The invariant is therefore *already enforced by a type*, and the way to break it is not to smuggle a value in but to compile a program per decode step. The refusal is a design rule with a negative test: eight decode steps at C1 must produce exactly one artifact identity, and a test that asserts it is a test that can fail.
3. **The runtime library and pipeline caches** are where contamination is actually reachable. [The runtime execution contract](runtime-execution-contract.md) keys a prepared pipeline on the live device, the code-section digest, the resolved entry symbol, **the specialization values**, the canonical pipeline descriptor, and the translation mode. **Inference — so `S`, `C`, and the cursor must be ABI-bound extents and must never become specialization values.** A build that specialized a kernel on `S` would mint a distinct pipeline for every decode step: at C1 that is nine cold pipeline creations where one would do, at B1-d one hundred and twenty-nine, and the cache key would then literally track a mutable inference quantity. The invariant is checkable at the artifact rather than at run time, because the specialization values are packaged.

**Fact — the third invariant is currently unbreakable for a reason that will not last.** No pipeline or library cache exists in `crates/`: `grep -rniE "pipelinecache|librarycache|pipeline_cache|library_cache" crates/` returns nothing, and the positive control `grep -rn "PipelineCacheKey" docs/` returns the contract that specifies one. So the invariant is a requirement on work that has not been done rather than a property of code that exists, and saying so is the difference between a contract and a claim.

### Reproducible checks

Each is one command from the repository root, with the positive control that proves it can return something.

```sh
# 1. Artifact identity is a function of the envelope alone.
grep -n 'pub(super) fn encode_identity' -A 3 crates/tiler-artifact/src/program/model.rs
#    The signature takes `&ArtifactEnvelope` and returns the identity. Positive
#    control: the same file *does* mention AbiFacts, at `ExprRef::evaluate`, so
#    the absence above is a property of the signature rather than of the grep.

# 2. Nothing in the loader compares a device instance.
grep -n 'pub struct ExecutionEnvironment' -A 8 crates/tiler-runtime/src/load/host.rs
#    Three fields: target profile, backend, representation. Positive control:
#    `classify` in the same file compares two of them, so the type is reached.

# 3. An input and an output may not share one allocation.
grep -n 'fn verify_storage' -A 14 crates/tiler-ir/src/program/verify.rs
#    Returns ForbiddenAlias when a shared allocation binds any non-Temporary
#    value. Positive control: the same function admits several Temporaries in
#    one allocation, which the reuse verifier below then constrains.

# 4. No pipeline or library cache exists to be contaminated yet.
grep -rniE 'pipelinecache|librarycache|pipeline_cache|library_cache' crates/
#    Returns nothing. Positive control: `grep -rn 'PipelineCacheKey' docs/`
#    returns the contract that specifies one.

# 5. A zero extent is expressible.
grep -n 'pub struct Extent' -A 6 crates/tiler-ir/src/shape.rs
#    The constructor's comment states that zero represents an empty axis.
#    Positive control: the same read finds the newtype it documents.
```

## Bounds: sequence length, batch, masking, and shape specialization

**Fact — sequence length is bounded per row and the bound is a refusal, not a convention.** `T` and `S` are bounded symbolic extents; an extent symbol with no proved upper bound refuses rather than compiling a generic program. C1 reaches `S = 18`, B1-d reaches `S = 8,320`, and both sit inside `max_position_embeddings = 32768`. A context beyond the declared maximum refuses at the semantic layer; a context beyond the state's own `capacity` refuses at the runtime instance, and the two are different refusals with different remedies.

**Fact — batch is 1 and this record adds no batch axis.** L1 fixes batch 1 for both rows, and a KV state per sequence is what batch 1 means. Batching would make `capacity` and the cursor per sequence within a governed storage population, which is a different state model — ragged valid ranges, per-sequence masking, and a scheduling question about which sequences share a dispatch. It is not reserved here and would be new architectural work.

**Inference — masking degenerates at decode, and that is a useful fact rather than a footnote.** At `T = 1` the single query position attends every cached position and itself, so every entry of the `[1, S]` mask is L4's attended value `0x80000000` and no entry is the finite fill. Two consequences follow. The fully-masked-row case that L4's D-1 needs is unreachable at decode, so the synthetic row remains the only place that choice is tested. And the decode mask carries no information that `S` does not already carry, so it is a candidate for removal — but the removal is a *bit* question, not a shape question, and this record takes no measurement, so it stays an experiment with an exact subject rather than an assumption.

**Inference — absolute position enters the decode program through `cos` and `sin` alone.** The mask is derivable from `T` and `S`; the cache extents are derivable from `C`; the residual stream carries no position. The rotary table's rows are the one input whose correct value depends on *where* the new token sits, and they are `[T, 128]` — 512 bytes at decode — with the same shape, dtype, and accessible range for every position. That is what makes the incorrect-position case below undetectable rather than merely untested.

**Proposal — the only shape specialization is variant selection over `S`, and it is per execution.** L4's feasibility predicate 2 refuses a contracted extent that is not a positive multiple of the tiled realization's width, and structure 3's contracted extent is `S`. Packaging two variants — the tiled plan guarded on `S ≡ 0 (mod 16)` and the direct plan otherwise — makes the choice a guard evaluation at each step under `RoutingPolicy::StablePriority`, which selects the first variant whose guard holds. Across C1's nine executions the tiled guard holds exactly once, at `S = 16`. **Inference — this is the first place in the workload where guards and routing are genuinely both required**, and it is also the reason the design must not specialize a kernel on `S`: the same discrimination is available as a packaged guard over a bound fact, at no cache cost.

## Execution across repeated executions

**Fact — the route is re-entrant and the mechanism is a borrow.** `DecodedProgram::preflight` takes `&mut self`, the returned `Preflight` borrows the program, and `Preflight::commit` carries that borrow into the `RoutedDispatch`. So one authority exists per attempt; dropping an uncommitted `Preflight` ends the borrow and the next attempt may preflight again. **Inference — a decode loop is therefore a sequence of complete routes over one decoded artifact**, and each step's routing commit is its own. ADR 0051's one-way commit is per execution, not per session: eight decode steps at C1 are eight commits, and a fallback taken at step 5 is a fallback for step 5 alone.

**Proposal — one decode step, stage by stage, with the state's obligation at each.**

| Stage | What happens to the state | Failure disposition |
| --- | --- | --- |
| Before binding | The adapter checks that the state's live device and context match the one it bound, and that `C + T ≤ capacity` | Refusal, no artifact obligation decided yet, fallback permitted |
| Fact binding (`LiveDevicePreflight`) | `C`, `T`, and `S` are bound as input-axis extents. **Fact —** input extents become observable exactly at this phase, and the binder refuses a fact offered before its phase | `PhaseNotReached`, `DuplicateInputExtent`, or a structural limit; all pre-commit |
| Loader routing | Identity, variant guard over `S`, profile, backend and representation pair, execution policy, launch geometry, accessible ranges | `LoadRejection`, pre-commit |
| Payload validation, live-device and prepared-entry questions | Nothing state-specific; the pipeline is reused across steps once a cache exists | Adapter refusal, pre-commit |
| `plan_dispatch` | The step is sized: each cache slot names its old governed storage, replacement storage is planned for each retained output under the selected physical representation, and every required resource/range fact is compared against the limits this device declares. Nothing is acquired | Adapter refusal — the last chance; the survivor defines the exact range and declared-capacity comparisons |
| Routing commit | Consuming and infallible | — |
| `allocate_dispatch` | Old storage is bound read-only at the cache slots; replacement resources are acquired according to the selected physical representation; every resource observation must satisfy what the plan sized and addressed | `Failure`. No fallback. An invalid resource population is an adapter defect, not a step to retry — the cursor does **not** advance |
| `dispatch` | Encode in execution order, submit, retain both storage populations against the receipt, observe terminal success | `Failure`. No fallback. The cursor does **not** advance |
| Publication | On observed terminal success only: the state's governed storage and cursor are replaced together, and old resources become releasable | A publication failure poisons the state |

**Inference — the cursor advance costs nothing to make correct, because the sampling dependency already pays for it.** A greedy decode loop cannot form step *n+1*'s input token without reading step *n*'s logits on the host, which crosses ADR 0033's device completion boundary in full: terminal completion, a post-completion status check, error-record visibility and coherence, record validation, and only then interpretation. So the host already observes exact terminal success once per step, and tying the cursor's advance to that same observation adds no synchronization. **Inference — the asynchronous publication the runtime contract permits therefore does not help a decode loop**, and per-token latency includes a full host round trip by construction rather than by an implementation shortcut. Prefill is different: its twenty-eight layers can share one submission and only the final logits need a readback.

**Fact — synchronization inside a step is dispatch ordering and nothing else**, unchanged from L4: no barrier is admitted under the implemented zero-synchronization schedule profile. **Inference — and across steps the ordered command stream is sufficient for the device**, because ADR 0047's initial profile is one affinity, one live device, and one ordered stream; what needs the completion observation is the *host*, for the token and for the cursor, not the device for the state.

**Inference — the repeated-dispatch shape has a precedent in the tree and it is one execution, not two.** `prototypes/serial-sum-run` dispatches one program twice, but the two dispatches are two independent paths — a direct path from locally compiled bytes and an envelope path from a decoded artifact — rather than a second execution against retained state. It exercises decode, route, ABI binding, two-stage qualification, and dispatch on real hardware, and it demonstrates that a route can be driven end to end; it demonstrates nothing about re-entering the route with different bound extents, and no checked-in code does.

## The update discipline

**Historical proposal — three candidates for how the extended value reaches the state.** The logical out-of-place publication rule survives, but the physical allocation/resource spellings and arithmetic in this section are evidence about the rejected singular compact-allocation candidate. They are not the physical-layout authority. **Dated correction, 2026-08-04:** the later [layout record](dynamic-kv-physical-layout.md) compares the alternatives and selects exact-live dense payloads inside two capacity-sized pool banks; the rows below remain historical evidence rather than being rewritten into that result.

| Candidate | Survives? | Ground |
| --- | --- | --- |
| **U-A** — logically out of place: read `[8, C, 128]`, produce `[8, S, 128]`, publish governed replacement storage | **Logical rule survives; physical spelling rejected.** | `verify_storage` establishes that a program input and output cannot share forbidden storage. The earlier one-old-allocation/one-new-allocation realization was only one way to satisfy that rule and cannot address capacity-strided storage correctly with the current ABI. |
| **U-B** — windowed write into retained capacity: write `[8, T, 128]` into the window at offset `C` of an `[8, capacity, 128]` allocation | **No, today** | It owes four implemented refusals rather than the three the sequence-extending record handed over, because that record was scoped to the semantic layer. `ExternalValueWritten` refuses writing a caller-bound input; `MultipleWriters` refuses a second stage writing the rest; nothing proves the untouched bytes of a partially written value; and `ForbiddenAlias` refuses the input and output sharing one allocation in the first place. It additionally owes what U-A gets free: a post-commit failure under U-B leaves the retained state *partially updated*, and ADR 0033 is explicit that initial transactions are out of place and that mutation requires a separate shadow or undo capability. The earlier “exact and reachable” trigger was derived from the rejected singular-allocation arithmetic below; the active trigger is the survivor-specific measurement plus recovery contract stated after that historical evidence. |
| **U-C** — the extension happens outside the compiled program, in the consumer | **No** | Already eliminated by the sequence-extending record on the ground that it moves a required data movement outside identity, cost, explain, and the verifiers. Nothing in this rung's analysis weakens that: a consumer that blits at the wrong offset returns a plausible tensor, and the incorrect-position case below shows how little else would catch it. |

**Historical inference for the rejected singular-allocation candidate.** Under that candidate, U-A copied the full cache per step and `ForbiddenAlias` kept one old and one new allocation live during the execution. Its arithmetic was:

| Program boundary | Peak KV residency at B1-d's final step | Arithmetic |
| --- | --- | --- |
| One program per decode step (56 cache inputs, 56 outputs) | 3,816,587,264 B (3.5544 GiB) | `28 × (68,149,248 + 68,157,440)` |
| One program per layer (2 inputs, 2 outputs) | 1,976,557,568 B (1.8408 GiB) | `28 × 68,157,440 + 68,149,248` |

**Historical inference for that same candidate.** It yielded a 1,840,029,696-byte (1.714 GiB) B1-d difference and the cited copy-traffic figures. Those numbers do not transfer to a segmented, componentized, materialized, or otherwise different representation. L6 retains them as rejected-candidate evidence; [Dynamic KV physical-layout authority](dynamic-kv-physical-layout.md) recomputes the selected capacity-sized resource population, and D-16 still requires that arithmetic to be measured before it becomes a binding cost.

**Proposal — an in-place/windowed reconsideration trigger, stated so it can fire.** A survivor-specific measured decode-latency or peak-residency result at a B1 row where replacement work is the dominant cost, together with a partial-update recovery contract. Both halves are required: the performance evidence could motivate the plan and would not make it safe.

### Layout, and why the obvious optimization does not apply yet

**Fact about the rejected row-major candidate — under `[8, S, 128]`, extending along axis 1 has no contiguous byte window.** The head axis is slowest-varying, so each of the eight heads' rows sit `S · 128 · 4` bytes apart and an append writes eight strided destinations. The sequence-extending record already states the general rule: a contiguous byte window exists only for the slowest-varying axis, and a concatenation along an inner axis cannot use it.

**Historical inference — another candidate layout is real, but arithmetic was not authority.** Storing the cache as `[S, 8, 128]` makes the sequence axis slowest-varying, so an append could be one contiguous window of `T · 8 · 128 · 4` bytes and could remove L4's step 13 permute. What it may cost is locality in the score contraction: for a fixed `(g, t)` the contraction walks `s`, and consecutive `s` are 4,096 bytes apart under `[S, 8, 128]` against 512 bytes apart under `[8, S, 128]`. **Dated correction, 2026-08-04:** the later layout ticket measured all three survivors, selected exact-live `[8,S,128]` packing with stable capacity-sized buffers, and retained the measurement boundary; this paragraph's stride arithmetic alone did not select it.

## Worked example: C1, one layer, nine executions

**Historical arithmetic for the rejected singular dense-allocation candidate.** One logical position of one layer holds a key and a value for eight key/value heads of width 128 in F32: `2 × 8 × 128 × 4 = 8,192` bytes, which is L1's 229,376 bytes per token divided by 28 layers. The `cache in`, `retained out`, and residency columns below assumed one dense allocation for each K and V value and therefore are not survivor-independent resource or peak-residency claims.

| Execution | `C` | `T` | `S` | cache in | retained out | both live | model-wide both live | `tiled` for structure 3 | mask | rotary rows |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| prefill | 0 | 10 | 10 | 0 | 81,920 | 81,920 | 2,293,760 | no | `[10, 10]`, 400 B | 0–9 |
| decode 1 | 10 | 1 | 11 | 81,920 | 90,112 | 172,032 | 4,816,896 | no | `[1, 11]`, 44 B | 10 |
| decode 2 | 11 | 1 | 12 | 90,112 | 98,304 | 188,416 | 5,275,648 | no | `[1, 12]`, 48 B | 11 |
| decode 3 | 12 | 1 | 13 | 98,304 | 106,496 | 204,800 | 5,734,400 | no | `[1, 13]`, 52 B | 12 |
| decode 4 | 13 | 1 | 14 | 106,496 | 114,688 | 221,184 | 6,193,152 | no | `[1, 14]`, 56 B | 13 |
| decode 5 | 14 | 1 | 15 | 114,688 | 122,880 | 237,568 | 6,651,904 | no | `[1, 15]`, 60 B | 14 |
| decode 6 | 15 | 1 | 16 | 122,880 | 131,072 | 253,952 | 7,110,656 | **yes** | `[1, 16]`, 64 B | 15 |
| decode 7 | 16 | 1 | 17 | 131,072 | 139,264 | 270,336 | 7,569,408 | no | `[1, 17]`, 68 B | 16 |
| decode 8 | 17 | 1 | 18 | 139,264 | 147,456 | 286,720 | 8,028,160 | no | `[1, 18]`, 72 B | 17 |

**Inference — the row closes against L1's own table.** The state after decode 8 is `28 × 147,456 = 4,128,768` bytes, which is exactly the 18-position figure L1 records, and the eighteen positions are the ten prompt tokens plus the eight decoded ones. **Inference — the whole C1 run's cache copy traffic is 51,380,224 bytes**, `28 × 8,192 × (10 + … + 17)` read and `28 × 8,192 × (11 + … + 18)` written, against `8 × 2,384,199,680` bytes of weight reads across the eight steps: 0.27%. At C1 the update discipline is unobservable, which is exactly why the decision above was not taken on C1 evidence.

**Inference — the tiled realization is admissible for the value contraction at one of nine executions.** `S ∈ {10, 11, …, 18}` and only `S = 16` is a multiple of 16. So eight of the nine executions route to the direct realization and one routes to tiled, through the same packaged artifact and the same guard. A design that assumed one realization for the whole row would be wrong at one step in nine, and wrong silently unless the tiled plan's own precondition refuses.

### The four cases the design must expose

**Case 1 — incorrect position. Nothing in Tiler refuses it, and this record says so rather than inventing a check.** At decode step 1 the correct rotary input is row 10 of the precomputed table; binding row 0 instead supplies a `[1, 128]` F32 tensor of 512 bytes with the same shape, the same dtype, the same accessible range, and the same launch geometry. **Inference — every layer accepts it:** the envelope decodes, identity matches, the guard over `S` is unaffected, the launch and range expressions evaluate identically, `plan_dispatch`'s byte comparison passes, the kernel verifier sees no difference, and the result is a plausible logit vector with a wrong argmax. ADR 0033's semantic preconditions are typed value predicates over admitted operations, and no operation in this program declares one that could express "these are the rotary rows for position `C`".

**Proposal — the structural answer is one cursor authority, and it splits the case in two.** If the consumer derives the cache extent, the rotary rows, and the mask from a single `C`, then an *inconsistent* position becomes unrepresentable and only a *wrong* `C` survives — and a wrong `C` produces a consistently wrong program that only the conformance oracle detects. **Inference — the spelling that makes the first half structural is a `Slice`**: binding the whole `[max_positions, 128]` table and selecting rows `C … C + T` by an index expression over the same bound extent moves the position from a host convention into a checked coordinate map. That is a new trigger for the sub-tensor-selection row, whose existing trigger is a prefill pass needing only the final position's logits, and it is stronger than that one because it is about correctness rather than about bytes. It is filed rather than assumed, and the claim is bounded: a slice removes the inconsistency mode and does not remove the wrong-cursor mode.

**Case 2 — stale state.** The host binds the allocation from step 4, whose valid range is `[0, 13)`, while binding `C = 14`. The bytes exist, because `capacity ≥ 14`, and nothing reads a byte outside the allocation. **Inference — the only quantity that could refuse this is the state's own valid length**, held at the runtime instance, because the artifact layer sees a well-formed extent. **Fact — the relation is now statable and its static check is implemented.** The accepted `ExtentRelation::AdditiveEquality` spelling makes `S == C + T` part of the shape environment; `ShapeEnvBuilder::build` rejects inconsistent statically observed root bindings and retains runtime-bound relations whose canonical lower-bound model exhibits a solution. **Inference — the stale launch remains unchecked until a runtime consumer evaluates that retained relation against invocation bindings.** The relation-representation gap is closed, but binding `C = 14, T = 1` against a stale runtime state still needs launch preflight to supply and compare the live `S` rather than merely carrying the relation in the artifact.

**Case 3 — partial update.** Step 5's dispatch fails after the routing commit, with some stages encoded and submitted. **Inference — under U-A the state is untouched**, because the old allocation was bound read-only and the new one is unpublished; discarding the new allocation restores nothing because nothing changed. What must *not* happen is a silent continuation: step 5's token was never produced, so a step 6 that binds step 4's state would decode a different sequence than the one the consumer believes it has. **Proposal — the state transitions to a terminal poisoned status that refuses every later bind**, and the consumer resumes by constructing a fresh state from a known prefix. **Inference — under U-B the same failure is unrecoverable without a version or undo mechanism**, because the retained allocation now holds an unknown mixture of old and new bytes with nothing to prove which; that is the second, independent reason U-B is deferred, and it is not fixed by relaxing the three verifier refusals.

**Case 4 — cross-device reuse.** A state created against device A is bound into a route whose adapter bound device B. **Fact — the loader cannot detect it.** `ExecutionEnvironment` has exactly three fields — target profile, backend family, executable representation — and two GPUs of one family in one host classify identically; nothing in `select_route` or `route_entry` compares a device instance. **Fact — and the loader is not the layer that should**: `LiveExecutionContext`'s own documentation records that a device handle is the adapter's to keep and never crosses into `tiler-runtime`, whose forbidden-dependency row lists every platform device API. **Inference — so device scoping of a KV state is necessarily an adapter obligation**, and the runtime execution contract already states the analogous rule for prepared selections: a prepared selection must be scoped by live device and context identity, and artifact identity alone cannot make a binding reusable. The state is the same subject and inherits the same rule; what is missing is that no `LiveDeviceKey` type exists in `crates/`, so the obligation has no carrier. Filed.

## Typed refusals this design owes

Each is a place where a silent approximation would return a plausible tensor.

- A bind whose `C + T` exceeds the state's `capacity` refuses before any program work, naming the required context and the capacity, and is never expressed as a large cost. Hard feasibility, not planning.
- A statically root-bound environment whose `S` does not equal `C + T` refuses with the accepted additive relation. A runtime bind owes the same refusal at launch preflight; that consumer does not exist yet and must evaluate the retained relation before any program work.
- A bind of a state whose live device and context differ from the adapter's refuses, naming both. This refusal has no carrier today.
- A bind of a poisoned state refuses, naming the execution that poisoned it. A poisoned state is never silently reusable and never repairable in place.
- A concatenation whose operands disagree on any axis other than the concatenated one, whose resolved value types differ, or whose result extent cannot be related to its operands' refuses at construction, through the accepted three-outcome shape path. Inherited from the sequence-extending record; restated because this rung is the first occurrence.
- A zero-extent cache operand behaves as the concatenation family's normative definition states, explicitly, rather than inheriting whatever the empty case happens to do. Prefill is the occurrence that makes this reachable rather than hypothetical.
- A plan that writes part of an output boundary without a proof covering the rest refuses. `MultipleWriters` refuses a second writer; a single partial writer passes and nothing proves the remainder. This is U-B's blocker and it is stated as a refusal that must be *added*, not as one that exists.
- A plan binding a program input and a program output to one allocation refuses as `ForbiddenAlias`, which is implemented, and which is what makes U-A's failure semantics free.
- A packaged program that specializes a kernel on `S`, `C`, or any cursor-derived quantity is refused at artifact assembly, because it would put a mutable inference quantity into a runtime cache key.
- A decode step whose value contraction selects the tiled realization at an `S` that is not a positive multiple of the tile width refuses, naming that realization's own precondition, rather than padding. Inherited from L4 and reached at eight of C1's nine executions.

## Unresolved decisions

Continuing L3′'s D-1 … D-5, L3's D-6 … D-8, and L4's D-9 … D-11.

- **D-12 — whether the decode mask may be omitted.** At `T = 1` every mask entry is the attended value `0x80000000` and the mask add is an addition of negative zero to every score. L4 records that the attended entry is value-preserving on every score except one case, so omitting the add is a candidate rewrite and not obviously an identity. **Closes** with a bit comparison over the C1 decode rows against the pinned reference — the same shape of evidence L4 used to settle the mask's fill convention — and it is worth taking because it removes one input, one broadcast, and one elementwise pass from every decode step of every layer. This record takes no measurement and therefore takes no position.
- **D-13 — the program boundary of one decode step.** **Closed by L6.** One per-layer program is reused 28 times inside one token transaction, with one model cursor and atomic publication across all 56 logical members. The earlier 3.5544 GiB versus 1.8408 GiB comparison belonged to a rejected singular-allocation candidate and was not the decision input; [L6](../program-planning/complete-model-ingestion-and-execution.md) chose the axes from artifact reuse and observable-state correctness, while the later layout authority supplied the two-bank arithmetic separately.
- **D-14 — the cache's physical sequence-axis layout.** **Derived proposal selected.** [Dynamic KV physical-layout authority](dynamic-kv-physical-layout.md) retains exact-live head-major packing and separates it from allocation length: old `[8,C,128]` and replacement `[8,S,128]` payloads occupy separate capacity-sized pooled buffers, but their head strides are the governed semantic `C*128` and `S*128`. The retained Metal comparison shows stable pool reuse is identical to capacity-strided storage while avoiding its new physical-root/schema surface; sequence-major is about 3.9% slower at both B1 copy cells and has no compensating property. The existing live-extent carrier is the only payload-address prerequisite.
- **D-15 — layout question resolved; coherent tested boundary next.** [`define-the-runtime-kv-state-boundary`](../../../tickets/define-the-runtime-kv-state-boundary.md) has a concrete survivor-independent draft for logical identity, capacity, cursor, generation, live scope, atomic publication, and poisoning. D-14 now supplies its physical descriptor: two alternating capacity-sized head-major F32 pool buffers per K or V member, an active bank and exact valid extent, with no dynamic physical stride. The ticket must reconcile that descriptor and its layout-dependent errors into one tested public packet before asking Tom; this research text does not accept the surface.

## What this record does not decide

- **Whether the concatenation key is one family or several, and its exact spelling.** A public operation boundary is Tom's, and the sequence-extending record's disposition — one general `Concatenate` rather than a narrow sequence-extend key — is a research disposition rather than an accepted interface.
- **The whole-model program, ingestion, and the vocabulary projection.** L6's. This record supplies the state contract and the cursor-granularity constraint and chooses no model shape.
- **Any cost or latency claim.** No schedule for a concatenation has been measured at any shape, no attention contraction has been measured at any shape, and no decode step has been executed. The residency and traffic figures are arithmetic over L1's and L4's stated quantities.
- **The model-level numeric tolerance and the decode-latency observables.** L8's, under [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md). This record supplies the per-step round-trip derivation as an input to it.
- **Batched or ragged state, speculative decoding, and prefix sharing.** None is reserved. Batched state needs per-sequence valid ranges and a scheduling model; speculative decoding needs two models and a divergence policy, and the roadmap already defers it with a trigger that reads "L5 delivers and measured decode latency is the binding constraint" — which this record does not satisfy, because it measures nothing.
- **Recurrent or convolutional state.** The ticket's own graph-maintenance rule keeps those with the later hybrid work; the first state contract is an ordinary dense-decoder KV cache and this is it.

## Delivery tickets filed from this record

Dependency-ordered, smallest vertical first. Public boundaries remain drafts until Tom reviews their exact implementation.

| Order | Ticket | Outcome | Waits on |
| --- | --- | --- | --- |
| 1 | [`admit-the-sequence-extension-concatenate-family`](../../../tickets/admit-the-sequence-extension-concatenate-family.md) | One general `Concatenate` becomes a registered, reference-evaluated key with its axis attribute, its extent-agreement refusals, and an explicit zero-extent rule. | [`scope-the-sequence-extending-tensor-family`](../../../tickets/scope-the-sequence-extending-tensor-family.md) |
| 2 | [`admit-an-additive-extent-relation`](../../../tickets/admit-an-additive-extent-relation.md) | `S == C + T` becomes statable and inconsistent static/root-bound values refuse; the stale runtime-state case still waits on launch-preflight consumption. | 1 |
| 3 | [`define-the-runtime-kv-state-boundary`](../../../tickets/define-the-runtime-kv-state-boundary.md) | The survivor-independent logical state semantics are drafted and the selected proposal supplies two capacity-sized pool banks with exact-live dense packing per member. The ticket must reconcile that descriptor and every layout-dependent refusal into one tested packet; no public surface is accepted yet. | 1, physical-layout authority |
| 4 | [`bind-the-kv-cache-through-the-artifact-and-runtime-interface`](../../../tickets/bind-the-kv-cache-through-the-artifact-and-runtime-interface.md) | The cache becomes ordered named program inputs and outputs whose extents are bound per execution, with accessible-range expressions over `C` and `S` and no specialization on either. | 1, 3 |
| 5 | [`execute-the-stateful-prefill-path`](../../../tickets/execute-the-stateful-prefill-path.md) | Prefill runs at `C = 0` and publishes a state whose cursor is `T`. | 4, [`integrate-the-attention-block-into-the-runtime`](../../../tickets/integrate-the-attention-block-into-the-runtime.md) |
| 6 | [`execute-the-decode-step-path`](../../../tickets/execute-the-decode-step-path.md) | One decode step runs against a published state, routes its own variant over `S`, and advances the cursor only on observed terminal success. | 5 |
| 7 | [`integrate-the-autoregressive-decode-loop`](../../../tickets/integrate-the-autoregressive-decode-loop.md) | A consumer drives prefill and eight decode steps from one cursor authority that derives the cache extent, the rotary rows, and the mask. | 6 |
| 8 | [`test-the-autoregressive-state-failure-cases`](../../../tickets/test-the-autoregressive-state-failure-cases.md) | Incorrect position, stale state, partial update, cross-device reuse, capacity exhaustion, and specialization contamination are failing tests rather than paragraphs. | 7 |
| 9 | [`prove-the-c1-stateful-attention-vertical`](../../../tickets/prove-the-c1-stateful-attention-vertical.md) | The nine C1 executions of one attention block are compared bit for bit against the pinned reference, with one artifact identity across all nine. | 8 |

Two further tickets are filed as capability work rather than as verticals: [`scope-a-windowed-kv-append-into-retained-capacity`](../../../tickets/scope-a-windowed-kv-append-into-retained-capacity.md) carries U-B with its residency trigger and its four blockers, and [`admit-a-position-selecting-slice-for-the-rotary-table`](../../../tickets/admit-a-position-selecting-slice-for-the-rotary-table.md) carries the incorrect-position case's structural half.

## Consequences for the ladder

**Inference.** L5's stated capability is "stateful prefill and token decoding", and that is not what this rung delivered. What it delivered is the state and execution contract written down — ten properties, a five-layer ownership table, three cache-identity invariants with reproducible checks, four planning decisions of which three are taken here with their eliminations and one is handed to L6 with its arithmetic, the four failure cases tested against the implemented stack, and eleven tickets. Nothing compiles, dispatches, or executes; no operation family moved a rung; the four-claim maturity vocabulary does not apply to a research record. The decode step itself is ticket 6, and it depends on the attention block's own integration rather than on more research.
