---
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.precision-schedule-co-search-envelope"
kind: "research"
title: "First precision-schedule co-search envelope"
topics: ["optimizer", "precision", "quantization", "scheduling", "accuracy", "cost-model"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.optimizer", "tiler.contract.numerical-semantics", "tiler.contract.cost-model", "tiler.contract.fusion-and-scheduling"]
depends_on: ["tiler.research.numerics.first-quantized-lm-profile", "tiler.research.program-planning.model-level-qualification", "tiler.research.program-planning.flash-class-capability-set"]
ticket: "scope-precision-schedule-co-search-under-accuracy-obligations"
---

# First precision-schedule co-search envelope

**Status:** complete bounded scope; pending adoption. This record chooses a research envelope and no production surface. It registers no precision profile, admits no parameter map, creates no accuracy-obligation type, changes no identity domain, and activates no selector.

## Decision in one page

**Proposal — the first precision population is two named whole-workload assignments, not an independent choice at every value or operation.** `P0` is the pinned workload's F32 baseline. `P1` is the already selected per-output-channel strict-affine U8-to-F32 profile over all 196 weighted projection operands, excluding the tied embedding. `P1` uses one per-axis map over weight axis 0 and is therefore conditional on Tom accepting and the repository delivering the exact public map boundary owned by [`implement-workload-selected-quantized-parameter-maps`](../../../tickets/implement-workload-selected-quantized-parameter-maps.md). The delivered per-tensor U4/U8 vertical is not a substitute: its values are caller-stated, its parameter map has one inhabitant, and the model-profile measurements reject per-tensor U8.

**Proposal — precision assignments are explicit semantic candidates.** An optimizer may propose `P1` from the closed profile above, but it may not reinterpret an F32 program through a physical-plan attribute, mint parameter payloads without an authority, infer a map from a schedule, or introduce an unregistered dtype or scheme. The proposed candidate must be a verified semantic program carrying its exact value types, scheme, map, component producers, conversions, and observable rounding boundaries. `P0` is always retained.

**Proposal — model accuracy is candidate-level hard feasibility; implementation accuracy remains target- and schedule-level hard feasibility.** A precision candidate first passes semantic verification and target-independent reference construction, then the named model-level obligation against the pinned corpus. Only a candidate with a satisfied obligation reaches physical enumeration. Every schedule for that candidate must separately prove that it realizes the candidate's own exact numerical contract. Neither result can be bought by a lower estimate.

**Proposal — the first schedule population retains both materialization and fusion.** `P1` admits a materializing `DequantizeStrictAffine` plus strict contraction and a fused decode in the strict contraction's operand access, crossed with the exact-order contraction schedules the target can prove. `P0` uses the corresponding F32 contraction schedules and retains the checkpoint's exact BF16-storage/F32-ingestion control once that storage route exists. A schedule-dependent quantization map is not a third schedule of one semantic candidate: if the map changes selected parameters, it is a new precision candidate and returns to the model-level gate.

**Proposal — existing layered identity is the carrier; this record invents no bytes.** The complete plan-selection subject must fold the exact semantic candidate identity and the selected physical-plan identity, as `ProgramAlternativeIdentity` already does. Therefore two candidates with the same physical schedule and different precision assignments are distinct at complete-plan, explain, cache, program, and artifact layers, while structurally identical lower schedule content may still be shared. The implementation decision that admits a precision-candidate origin owns any required domain/version step and exact encoding.

**Proposal — the first automatic cross-precision winner waits for an applicable measured or calibrated selector.** Structural cost may retain its exact Pareto view, and uncalibrated analytical cost may be reported for every accuracy-feasible plan, but neither may discard a semantic candidate. The current measured fold-step row does not cover the U8 decode/contraction trade. Until a measured or calibrated row names a domain covering both `P0` and `P1`, co-search may enumerate, validate, and explain the portfolio but may not automatically prefer one precision assignment on performance grounds. An explicit caller preference is a separate meaning choice, not a measured co-search result.

## Why this record lives in `research/program-planning`

**Inference.** The selected quantized scheme already lives with its numerical evidence in [First quantized language-model profile](../numerics/first-quantized-lm-profile.md), and the cost authorities already live in [Cost model](../../compiler/cost-model.md). The new conclusion is the ordering and ownership of candidate construction, accuracy admission, physical enumeration, identity composition, and selection. Those are program-planning questions governed primarily by [Optimizer model](../../compiler/optimizer.md) and [Fusion and scheduling](../../compiler/fusion-and-scheduling.md), so `docs/research/program-planning/` is the narrow landing home.

## Existing authorities and the gap they leave

**Fact — resolved precision is semantic meaning.** [ADR 0009](../../decisions/0009-resolved-numerical-typing.md) requires every value and operation to have a resolved numerical signature before semantic optimization; casts and quantization boundaries remain observable even when fusion removes materialization. [ADR 0030](../../decisions/0030-first-class-quantized-values.md) gives a quantized tensor one static versioned contract and ordinary component-producing dataflow. A lower precision, different parameter map, or different conversion boundary therefore denotes another semantic program, not another backend spelling of the same program.

**Fact — schedule remains physical.** [ADR 0001](../../decisions/0001-separate-semantic-and-physical-plans.md) separates semantic planning from target-aware scheduling, and [ADR 0007](../../decisions/0007-first-class-kernel-schedules.md) makes the normalized schedule an identity-bearing physical value. Different schedules may realize one precision assignment. A schedule that changes which quantization parameters apply has crossed the boundary and must instead construct a different semantic assignment.

**Fact — local and model-level accuracy are different obligations.** [ADR 0017](../../decisions/0017-local-vs-region-accuracy.md) makes a future region/output accuracy goal additive, evidence-labelled, and hard feasibility, and forbids it from silently overriding local operation semantics. The implemented `require_elementary_accuracy` path in [`request.rs`](../../../crates/tiler-compiler/src/request.rs) asks each target to refine every registered elementary operation before numerical-contract resolution and asks again when a semantic candidate is readmitted. [`target/accuracy.rs`](../../../crates/tiler-compiler/src/target/accuracy.rs) owns that closed refinement relation and explicitly owns no cost. It proves whether a target realization implements an operation already in the candidate; it does not decide whether a quantized candidate is an acceptable approximation of an F32 model.

**Fact — the model-level evidence exists but does not authorize every assignment.** [Model-level correctness and performance qualification](model-level-qualification.md) defines thirteen named rows with exact inputs and required dispositions, including the explicitly `Unknown` `A-tie` row and the stage-dependent `A-fallback-after-commit` row. [First quantized language-model profile](../numerics/first-quantized-lm-profile.md) separately selects and measures the `P1` profile against the pinned Qwen3 workload. Neither record establishes an arbitrary mixture of F32 and U8 weights, a per-tensor candidate, or an attention activation/KV-cache candidate.

**Fact — cost authorities are already separated.** [`PlanStructuralCost`](../../../crates/tiler-compiler/src/selection.rs) is an exact four-dimensional structural record whose dominance is a view over retained valid plans. [`component_cost.rs`](../../../crates/tiler-compiler/src/component_cost.rs) emits the typed analytical model as reported evidence and does not feed dominance. [`measured_cost.rs`](../../../crates/tiler-compiler/src/measured_cost.rs) may prefer retained valid plans only when a target profile declares its measured row. [Optimizer model](../../compiler/optimizer.md) prohibits estimated-cost pruning of semantic alternatives at every stage.

**Inference — caller-stated encoded inputs alone do not close the gap.** The existing strict-affine vertical can schedule a program whose encoded value and parameter map the caller already stated. That is valuable physical search inside one precision assignment, but it is not precision assignment search. Conversely, an unconstrained optimizer that chooses a code width while leaving the scheme, map, scale production, and conversions implicit would make meaning a physical default. The first envelope must therefore install one closed semantic-candidate producer before physical co-search is meaningful.

## The first finite population

### Precision assignments

The population is sized from two named profiles rather than from 196 independent booleans.

| Key | Exact assignment | Model-level evidence | Admission |
| --- | --- | --- | --- |
| `P0` | The pinned Qwen3 workload's 196 weighted projection operands remain F32; the tied embedding remains outside the changed population. | The pinned F32 workload/reference rows. | Always retained as the baseline when the request itself is valid. |
| `P1` | All 196 weighted projection operands use `tiler::strict-affine@1`, `tiler::u8@1` codes, F32 expression/computation, a positive-normal F32 scale and U8 zero point per output channel, and one map over weight axis 0; the tied embedding remains F32. | The selected profile's retained weight-error and C1 model-visible measurements, under their exact checkpoint, calibration, prompts, target and evidence class. | Conditional on the per-axis map and payload-producing/ingestion authority being delivered; then admitted only when its named model-level obligation is satisfied. |

**Proposal — the assignment unit is the complete named profile.** The first population has cardinality two. It does not contain `2^196` independent weight choices, layer-by-layer bit widths, U4, per-tensor U8, per-block/group maps, activation precision, accumulator precision, attention Q/K/P/V precision, or KV-cache precision. Adding one assignment requires one independently specified semantic profile and one model-level obligation whose evidence covers that exact assignment.

**Inference — why per-axis is inside and per-tensor is outside.** The selected-profile evidence reports per-tensor U8 failing the pinned model-visible row, while per-output-channel U8 survives and adds only 0.3% bytes over per-tensor in that workload. A first population restricted to the delivered map would therefore contain only the F32 baseline after hard accuracy admission and would exercise no co-search. The per-axis map is consequently a conditional prerequisite, not optional width for this envelope.

**Proposal — optimizer-minted does not mean optimizer-invented.** The candidate producer selects the closed `P1` profile and constructs an explicit semantic program from an authoritative workload artifact. The artifact must supply or reproducibly derive the exact code and parameter payloads under a versioned producer; the optimizer may not guess them, hide calibration state in a provider, or treat runtime payload bytes as static type identity. No `P1` candidate exists until that producer and the per-axis map can make the full program verifiable.

**Fact — `P1` is selected for study, not accepted as a model approximation.** Its retained C1 evidence is one checkpoint, one prompt and eighteen positions: the sequence agrees, the greedy argmax agrees at 17 of 18 positions, and the median whole-vocabulary deviation is `1.08e-1`. [Model-level correctness and performance qualification](model-level-qualification.md) explicitly leaves whether that is acceptable as a product judgement. No accepted obligation or threshold currently admits `P1`, so this record places it in the population and does not claim it passes stage 3.

**Fact — exact BF16 storage is a control inside `P0`, not a third semantic assignment.** The checkpoint's BF16 weights widen to F32 bit-exactly on the retained C1 evidence while using half the F32 weight bytes. The arithmetic program remains `P0`; only ingestion/storage realization changes. A valid cost comparison must retain this control, because comparing `P1` only with F32-stored `P0` would attribute an ingestion-storage saving to model approximation that exact BF16 storage already supplies.

### Physical schedules

For every precision candidate that passes its candidate-level gate, enumerate the legal schedules rather than assigning one schedule to the profile by convention.

| Candidate | First schedule families | Required relation |
| --- | --- | --- |
| `P0` | The target-feasible exact-order F32 contraction schedules already admitted for the workload; once delivered, both F32 storage and exact checkpoint-BF16 storage widened at ingestion. | Each schedule refines the same F32 semantic candidate and its local numerical contract; the BF16 route must prove exact widening rather than consume model-accuracy tolerance. |
| `P1` | Materialize exact F32 decode before contraction; fuse exact decode into the contraction operand access; for either form, retain every exact-order contraction topology the target proves. | Materialized and fused results agree bit-for-bit with the same quantized semantic reference. Fusion may remove storage, never the semantic F32 decode boundary or its rounding. |

**Inference — this is the smallest population that exposes a precision/schedule interaction.** `P1`'s stored-weight benefit disappears if F32 decode is kept resident, while decoding and materializing per dispatch adds both a write and a read of the F32 weight. The fused schedule can retain the compressed traffic. Keeping the materialized plan makes that conclusion a cost comparison over two valid realizations instead of a fusion axiom.

**Proposal — schedule-coupled maps split the semantic candidate.** SageAttention2's per-thread INT4 groups are derived from an MMA instruction's thread/layout mapping. If Tiler later admits such a map, the selected parameter coordinate relation must be explicit before schedule identity is formed. Changing the thread tile so the selected parameter groups change creates another semantic assignment and re-runs candidate-level accuracy; it cannot be hidden as a schedule transform over `P1`.

## Accuracy and feasibility phase ordering

The word “accuracy” names three different questions in this envelope. They keep separate subjects, evidence, and failure classes.

| Order | Subject and authority | Required answer | Cost visibility |
| --- | --- | --- | --- |
| 1 | The proposed semantic candidate: ordinary semantic verification, exact resolved types/schemes/maps/conversions, local operation preconditions, and reference construction. | Valid or typed invalid/missing capability. | None. |
| 2 | Each requested target: existing dtype dispatch, `require_elementary_accuracy`, numerical-contract applicability and honourability during candidate readmission. | Proven/refined or a typed target-local refusal/`Unknown`. | None. |
| 3 | The whole named precision assignment: its model-level obligation over exact checkpoint, input corpus, F32 reference, metric/tolerance, exceptional behaviour, and evidence class. | Satisfied. `Unknown`, disagreed, refused, missing evidence, or a corpus miss excludes the candidate from executable physical search while retaining the explanation. | None. |
| 4 | Each precision-candidate × schedule realization: intrinsic schedule verification, exact local numerical refinement, target feasibility, and any permitted-divergence realization oracle. | Proven, or safely deferred only under the existing pre-routing rules; otherwise rejected/`Unknown`. | None. |
| 5 | Complete plans that passed all four gates. | Retained and costed. | Structural, analytical, then applicable measured/calibrated evidence. |

**Proposal — stage 3 runs once per distinct semantic assignment, before region and schedule enumeration.** The model output is a function of the semantic candidate. Re-running the same corpus for materialized and exactly fused schedules would confuse model approximation with compiler conformance and multiply expensive work without changing the question.

**Proposal — stage 4 still runs for every schedule.** Exact agreement with a quantized program's reference does not establish that the quantized program is acceptable relative to F32, and model-level acceptance does not establish that a kernel correctly implements the quantized program. Both proofs are required and neither substitutes for the other.

**Proposal — a schedule-affecting numerical choice returns to stage 1.** Changing code type, expressed/computation/accumulator type, scheme, map, smoothing transform, conversion order, materialization rounding, or contributor order is not a cheaper realization of the same candidate unless the candidate's existing numerical contract already permits and identifies it. Otherwise the candidate is reconstructed, re-identified, and re-qualified from the beginning.

**Proposal — evidence classes remain exact.** A named empirical corpus can satisfy an empirical model-level obligation with that exact test definition; it cannot satisfy a `SoundProof` obligation or make claims about other inputs. A schedule's local exact-refinement proof remains a separate stronger statement about its governed domain. The first envelope declines `A-tie` because its required outcome is still `Unknown`, and preserves the pre-commit refusal/post-commit failure split of `A-fallback-after-commit` rather than flattening it into one pass result.

## Identity without a new encoding in this record

**Fact — the complete alternative already has the required composition shape.** `ProgramAlternativeIdentity::new` in [`pipeline.rs`](../../../crates/tiler-compiler/src/pipeline.rs) folds the semantic alternative origin, the semantic graph, reached definitions, admission provenance, registry snapshot, shape environment, resolved numerical-contract key, and `SelectedPlanIdentity`. The physical receipt in [`selection.rs`](../../../crates/tiler-compiler/src/selection.rs) separately folds the cover, selected implementation identities, handoffs, guards, honoured numerical facts, and structural cost. This follows [ADR 0072](../../decisions/0072-separate-semantic-meaning-from-provider-provenance.md): pure schedule content may be shared, while a complete program binds meaning, occurrence, realization, and provenance.

**Proposal — the identity requirement is transitive rather than duplicative.** `P0` and `P1` must produce different semantic graph/type identities because their value types, components, maps and conversion boundaries differ. A complete plan or artifact then binds that semantic identity to the selected physical plan. `SelectedPlanIdentity` need not embed the whole semantic program a second time merely to make the lower-layer physical receipt globally unique; the complete alternative/program/artifact subject is the equality used for selection, cache and publication.

**Proposal — the future implementation decision owns the missing origin.** The current origin vocabulary distinguishes a baseline from registered algebraic rewrites and has no precision-profile origin. Adding one is identity-sensitive work: the owning decision must choose its canonical identity, version/domain movement, provenance versus meaning placement, explain subject, and pin updates. This record supplies only the invariant: no two distinct precision assignments may compare equal at complete-plan, program, artifact, cache, or explain identity, and no history-only difference may falsely change semantic graph meaning.

**Proposal — required perturbations.** Hold the schedule fixed and change only the code width, parameter-map axis, scheme key, conversion boundary, or one identity-bearing constant parameter; complete-plan and artifact identity must change. Hold the full semantic candidate fixed and change only a runtime-bound parameter payload; static identity must not change, while binding/version/conformance evidence must prevent stale reuse. Hold semantic and physical content fixed while changing only an unused provider; semantic graph and selected artifact identity must remain stable under ADR 0072's reached-only rule. The identity owner must perturb the producer, not these assertions, and show each failure message.

## Ranking without semantic cost pruning

**Proposal — retention is unconditional on estimate.** `P0` is retained, and `P1` is retained whenever its hard obligations pass. No semantic exploration, readmission, contract grouping, or model-level accuracy stage receives structural, analytical, or measured cost. A precision candidate may disappear only through semantic invalidity, missing capability, hard accuracy/feasibility disposition, or an explicit deterministic search budget that reports what it stopped.

**Proposal — structural cost keeps its current narrow role.** Exact dispatch, launched-thread, temporary-byte and materialization counts may form a Pareto view over complete physical plans, and that view never deletes the retained portfolio. For the first envelope, structural comparison is safe for eliminating no complete semantic candidate: compare physical realizations within `P0` and within `P1`, retain the cross-precision trade-off, and report the structural relation. Those four dimensions omit code/parameter traffic and low-precision throughput, so a cross-precision structural winner would not be a latency conclusion.

**Proposal — uncalibrated analytical cost reports but does not choose precision.** Emit the existing typed analytical assessments for every admitted plan, including code bytes, parameter bytes, decoded work, F32 materialization traffic, and schedule work where the model can state them. Keep `Reported` attribution, intervals/unsupported interactions, and model identity. It may order an exploratory report, but it may neither feed dominance nor determine the product winner between `P0` and `P1`.

**Proposal — measured/calibrated preference activates the automatic winner.** A row may select between `P0` and `P1` only when its declared target, workload/shape domain, dtype/scheme/map and schedule feature coverage include both candidates and every decisive term, including the exact BF16-storage control for `P0` once available. Silence means no automatic cross-precision preference. The existing measured fold-step row covers a reduction saturation quantity and says nothing about U8 component traffic, per-axis scale reads, decode work, strict contraction throughput, or ingestion storage, so it cannot activate this envelope.

**Proposal — “first co-search” has two milestones.** The research/enumeration milestone constructs the two candidates, runs hard gates, retains their schedules, and reports structural plus analytical assessments. The automatic-selection milestone additionally supplies an applicable measured/calibrated row and may then prefer any retained valid plan, including a structurally dominated one, exactly as the current measured selector does. Calling the first milestone a measured performance optimizer is refused.

## Primary sources and eliminations

Only primary papers and official proceedings/repository copies were used. Their empirical results remain bounded to their programs, workloads and hardware.

| Primary source | What it establishes in its own scope | What this envelope retains | What this envelope eliminates |
| --- | --- | --- | --- |
| Rubio-González et al., [*Precimonious: Tuning Assistant for Floating-Point Precision*](https://people.eecs.berkeley.edu/~ksen/papers/precimonious.pdf), SC 2013, DOI `10.1145/2503210.2503296` | Searches type assignments for floating-point variables under developer accuracy/performance criteria; uses representative program inputs and explicitly makes no guarantee for other inputs. | Precision assignments are semantic candidates, and search effort is worth bounding. | Free variable-level assignment as the first population; representative-input testing presented as a proof outside its corpus; a locally found configuration presented as globally optimal. |
| Guo and Rubio-González, [*Exploiting Community Structure for Floating-Point Precision Tuning*](https://huiguoo.github.io/files/hifptuner-issta18.pdf), ISSTA 2018, DOI `10.1145/3213846.3213862` | Uses dependence/community structure for hierarchical variable-level search; every configuration executes to test accuracy and measure performance; distinguishes search effort from achieved speedup. | Future grouping can shrink a larger population, and search-work budgets stay separate from plan cost. | Importing variable communities before a Tiler semantic/profile population exists; treating fewer explored configurations as faster generated code; using dynamic accuracy runs as an unlabeled universal guarantee. |
| Wang et al., [*HAQ: Hardware-Aware Automated Quantization with Mixed Precision*](https://arxiv.org/abs/1811.08886v3), arXiv v3 (2019-04-06), DOI `10.48550/arXiv.1811.08886`, CVPR 2019 | Selects layerwise bit widths using hardware latency/energy feedback; reports hardware-specific policies and empirical accuracy after quantization/fine-tuning. | Target-specific measured feedback is needed to choose between accuracy-feasible precision assignments. | Reinforcement-learning reward as a correctness authority; accuracy as a soft reward rather than a hard gate; sequentially mutating bit widths until a resource constraint passes; proxy FLOPs/model size as sufficient ranking truth. |
| Xiao et al., [*SmoothQuant: Accurate and Efficient Post-Training Quantization for Large Language Models*](https://proceedings.mlr.press/v202/xiao23c.html), PMLR 202:38087–38099, ICML 2023 | W8A8 requires an offline, mathematically equivalent smoothing transform that moves quantization difficulty between activations and weights, followed by approximate quantization and calibration. | A precision assignment includes transforms, parameter production and granularity, not only a scalar dtype label. | A “choose INT8” candidate with implicit smoothing/calibration; transferring W8A8 accuracy claims to the selected weight-only U8 profile; treating calibration state as hidden provider data. |
| Zhang et al., [*SageAttention: Accurate 8-Bit Attention for Plug-and-play Inference Acceleration*](https://arxiv.org/abs/2410.02367v9), arXiv v9 (2025-10-01), DOI `10.48550/arXiv.2410.02367`, ICLR 2025 | Couples FlashAttention tiling with Q/K INT8 quantization, K smoothing, full-precision online softmax, multiple P/V realizations, and empirically adaptive per-layer selection. It reports that direct uniform quantization degrades model metrics. | The motivating fact that useful precision choices and schedules can be coupled, and that a model-level gate is indispensable. | SageAttention itself from the first population: Tiler lacks its attention semantic candidate, smoothing operation, Q/K/P/V map families, target instruction profile, and corpus; its empirical adaptive threshold is not a portable hard obligation. |
| Zhang et al., [*SageAttention2: Efficient Attention with Thorough Outlier Smoothing and Per-thread INT4 Quantization*](https://arxiv.org/abs/2411.10958v7), arXiv v7 (2025-10-01), DOI `10.48550/arXiv.2411.10958`, ICML 2025 | Maps INT4 quantization groups to the PTX `mma.m16n8k64` thread/layout assignment, combines Q/K smoothing with FP8 P/V and two-level accumulation, and evaluates the resulting kernels empirically. | A schedule can expose the need for a new semantic map/profile; precision and scheduling cannot be optimized as independent bags of flags. | Per-thread, per-block, FP8 and attention-activation maps from the first envelope; inferring a semantic parameter map from a selected target schedule; transferring NVIDIA PTX-specific grouping or accuracy to Apple Metal or Tiler's per-axis weight profile. |

**Inference — the sources converge on one negative result.** None licenses a compiler to choose an untyped width and repair accuracy after the fact. Each successful system binds precision to a concrete assignment, transformation, input corpus and performance environment. The Tiler-specific contribution of this scope is to keep those bindings explicit while preserving hard feasibility and layered identity.

## Refutable boundaries and expansion triggers

This envelope should be replaced, not silently widened, if any of the following evidence arrives:

- a second model-qualified assignment over the same workload, with exact semantic profile and corpus obligation, justifies a population larger than `{P0, P1}`;
- a measured/calibrated target row covers the decisive F32-versus-U8 contraction/decode terms and can activate automatic selection;
- the per-axis map is refused by Tom or its implementation evidence contradicts the selected profile, in which case `P1` is unavailable and this envelope has no non-baseline member;
- a delivered attention semantic/profile/corpus vertical supplies the Q/K/P/V transforms, maps, local contracts and target facts SageAttention-class search requires;
- a schedule-coupled map obtains a target-neutral semantic definition and model qualification, in which case it enters as another semantic assignment rather than by widening the schedule fields of `P1`; or
- a sound region analyzer can discharge a stronger model-level obligation than the empirical corpus, in which case the evidence class changes explicitly rather than upgrading the existing measurements in prose.

## Implementation dependency boundary

**Proposal — no implementation starts from this record alone.** The earliest implementation packet must name and obtain Tom's acceptance for the public per-axis map surface; define the closed precision-candidate producer and payload authority; define the model-level obligation object and its relation to the existing corpus; decide the precision-origin identity/domain change; widen the physical component access and strict-contraction fusion; and supply an applicable selection cost row before automatic cross-precision preference is enabled. Each is a separately reviewable public-, identity-, numerical-, or target-sensitive step.

**Fact — current maturity remains below that packet.** The profile is selected but `implementation_status: "not-started"`; its measured 17-of-18 greedy agreement has no accepted product threshold; the map ticket is `awaiting-decision`; no compiler stage constructs `P1`; `ProgramAlternativeIdentity` has no precision origin; the current elementary-accuracy authority does not compare whole models; and the active measured row covers fold steps rather than quantized contraction. This record is therefore a scope for future work, not evidence that precision-schedule co-search exists.

## Conclusion

**Proposal.** The first precision-schedule co-search is a closed two-assignment semantic portfolio with `P0` preserved and `P1` conditionally proposed, a model-level hard gate before physical enumeration, exact realization gates for every schedule, transitive complete-plan identity through the existing semantic-plus-physical composition, and no automatic cross-precision performance winner until applicable measured/calibrated evidence exists. That is sufficient to test the architecture without importing SageAttention's unowned semantic and target surface or pricing meaning before its accuracy obligation is satisfied.
