---
id: design-model-level-qualification-and-optimization
title: Design model-level correctness and performance qualification
status: review
priority: p2
dependencies: [define-first-metal-lm-workload, design-model-ingestion-and-complete-execution]
related: [implement-analytical-component-cost-model, calibrate-device-cost-models, scope-first-quantized-lm-profile, land-the-model-level-qualification-record, measure-the-model-level-comparison-envelope-under-the-target-realization, define-the-model-level-conformance-corpus, build-the-model-level-measurement-harness, qualify-the-model-level-claims-per-apple-device-and-toolchain-row, supply-the-model-level-benchmark-protocol-to-cost-calibration, define-the-model-level-regression-policy, measure-b1-d-peak-residency-on-a-named-host]
scopes: [research/cost-model, research/apple-targets, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [design, testing, performance, conformance, language-model, metal]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785611639
---
## User-visible outcome

"This model is correct and optimized on this Metal target" becomes four separately-supported claims — correctness against reference outputs, feasibility, estimated cost, measured performance — with comparison tolerances derived from the effective numerical contract *before* results are observed, never after.

Define how Tiler will establish that a supported language model is both correct
and optimized on its declared Metal target. Correctness, feasibility, estimated
cost, and measured performance remain separate claims.

## Required design

- Select model-level reference outputs and adversarial inputs for prompt,
  prefill, decode, exceptional values, sequence bounds, and persistent state.
- Define exact or tolerance-based comparison from the effective numerical
  contract rather than choosing thresholds after observing results.
- Define the Apple device-family and toolchain matrix for each claim.
- Specify cold and warm time to first token, decode latency, tokens per second,
  peak and persistent memory, artifact preparation, dispatch count,
  materialization count, and cache behavior.
- Distinguish correctness gates, performance measurements, cost-model
  calibration data, and regression thresholds.
- Define how failures remain attributable to frontend, compiler, backend,
  artifact, runtime, or consumer boundaries.

## Ticket-producing outcome

File separate tickets for the conformance corpus, measurement harness, device
qualification, cost-model calibration, kernel or schedule improvements exposed
by evidence, and regression policy. Do not turn an unmeasured performance goal
into a normative guarantee or file optimization work without a measured
bottleneck.

## Closes when

The complete-model vertical has a reproducible correctness and performance
qualification plan; every metric names an environment and procedure; baseline
and quantized paths can be compared without conflating claims; and justified
follow-up work is represented by scoped tickets.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L8** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L1 and L6 both deliver.

**Rests on:** L1 and L6.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **The two bounded rows this rung qualifies are already fixed by [`docs/research/program-planning/first-metal-lm-workload.md`](../docs/research/program-planning/first-metal-lm-workload.md)** — a 10-token/8-step conformance row whose every logit is retainable, and a four-point prompt/decode benchmark matrix bounded at 8,320 of the checkpoint's declared 32,768 positions because F32 residency and materialized-score workspace cross unmeasured thresholds above it. Extending the matrix upward is legitimate and needs a residency measurement on a named host first.
- **This rung owns the comparison bound, which L1 deliberately left `Unknown` rather than guessing.** The conformance level is `bounded error`, and the profile records why a model-level constant cannot be composed from per-operation tolerances. It also names the measurable half: the reference's own F32 sensitivity envelope for the exact conformance prompt, obtained by evaluating the pinned reference under two independently legal orderings and retaining the per-position deviation. [`retain-the-qwen-conformance-reference-logit-fixture`](retain-the-qwen-conformance-reference-logit-fixture.md) produces that envelope, so this rung derives a budget from retained evidence rather than starting the measurement itself — and must still state the declared realization's subnormal and elementary-function behaviour as the other half, since the qualified Apple9 row flushes F32 subnormals where a CPU reference preserves them.
- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Qualify that exact model and reference path rather than a generic transformer. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).

- **Measured-performance claims are M3-host measurements** — the qualification design must name the bench host discipline (serial runs, interleaved A/B), and its measured rows are `Measurement`s tied to exact environments, per the research standards.

## Stop condition — the record's home is outside this ticket's scopes (2026-08-01)

**Fact — the scope map, read from `ticketsplease.toml` rather than recalled.** This ticket declares `research/cost-model`, `research/apple-targets`, and `contracts/navigation`, plus the shared `project/tickets`. `research/cost-model` covers `docs/research/cost-model/**` and `spikes/cost-model/**`; `research/apple-targets` covers `docs/research/apple-targets/**` and `spikes/apple-targets/**`; `contracts/navigation` is an explicit file list containing `docs/roadmap.md`, `docs/research/README.md`, `docs/status.md`, `docs/open-questions.md`, and eight others. `research/program-planning` — `docs/research/program-planning/**` and `spikes/program-planning/**` — is a separate scope this ticket does not hold.

**Inference — the qualification record's home is `docs/research/program-planning/`, and the elimination leaves one survivor rather than a fork.** Four candidates were tested against where a reader looking for "how is this model qualified" would go, and against whether the record's spine — four separately supported claims — survives the placement.

| Candidate home | Survives? | Ground |
| --- | --- | --- |
| `docs/research/program-planning/model-level-qualification.md` | **Yes** | Its subject is the qualification of one complete model *program*, which is what L1 and L6 already own in this directory and in the `physical-planning-lowering` catalog group. [L7's own "Why this record lives in `research/numerics`" section](../docs/research/numerics/first-quantized-lm-profile.md) is the precedent for placing a ladder rung by its subject rather than by the ladder: L7 sits in `research/numerics` because its subject is a quantized value scheme. L8's subject is a model program's qualification. |
| `docs/research/cost-model/` | No | Estimated cost is *one* of this rung's four claims and the weakest-supported of them — eight of the analytical model's nine components are `Unknown` today. Filing the record here names it by its weakest quarter, and buries the conformance corpus, the comparison bound, and the regression policy in a directory whose one existing record is a costing plan. |
| `docs/research/apple-targets/` | No | The device-and-toolchain matrix genuinely belongs to this scope's subject, and nothing else in the record does. A qualification record here puts correctness-gate design and regression policy inside a target-behaviour measurement record, and leaves the reader of `numerical-behaviour.md` — a 554-line measurement corpus — to discover a design document folded into it. |
| Split across the two held research scopes | No | The record's spine is that correctness, feasibility, estimated cost, and measured performance stay four claims. A split files the cost claim in one directory and the device claim in another, and leaves the conformance corpus, the comparison bound, the attribution ladder, and the regression policy without a home in either — which is most of the rung. |

**So this is a dispatch-scope error rather than an architecture fork, and it is reported rather than repaired here.** One candidate survives, so there is no decision for Tom in the placement itself; what there is, is a scope this ticket does not hold and must not self-grant. The ticket's own `scopes` list sits in `tickets/**` and is therefore editable from the shared `project/tickets` scope — which is exactly why editing it would be a silent scope expansion rather than an accident, and it was not done.

**Fact — no live ticket holds `research/program-planning`, so the carrier is dispatchable immediately.** The check, reproducible in one line, is `tkt list --status in-progress` followed by reading each result's `scopes` line: at this branch's base commit `2aa0824` the seven in-progress tickets hold `implementation/compiler`, `implementation/ir`, `research/scheduling`, `implementation/frontend`, `contracts/decisions` + `research/runtime`, `implementation/metal` + `implementation/build` + `implementation/runtime` + `implementation/artifact` + `contracts/artifacts`, and this ticket's own three. None is `research/program-planning`.

**What was done instead, following the corpus's own discipline for a record that cannot reach its destination.** `AGENTS.md` fixes the shape: a research record whose scopes cannot reach a destination drafts the destination body verbatim-landable inside the record and files a carrier ticket, and the transfer is byte-identical. The work record here is this ticket, so the complete record body is drafted below and [`land-the-model-level-qualification-record`](land-the-model-level-qualification-record.md) carries the transfer. The drafted span's relative links resolve from `docs/research/program-planning/` and therefore not from `tickets/`; per the same convention that condition is stated beside the span rather than repaired inside it, because repointing forks the transfer and spends the byte-identity that makes the span quotable.

## Outcome — 2026-08-01

**The design is derived and drafted; nothing is measured and nothing executes.** What this rung delivers is: the four claims kept apart with what each may and may not say; the model-level comparison bound converted from L1's deliberate `Unknown` into a bounded experiment with exact inputs, outputs, and a stop condition, whose budget is derived from accepted contracts and measured target facts *before* any Tiler result exists; the adversarial corpus derived from the refusals L4, L5, and L6 already own; the per-claim Apple device and toolchain matrix; the decomposition of "time to first token" into four separately attributable terms the architecture forces; the bench-host discipline with the amendment L3's own cache-residency measurement forces on it; the attribution ladder with the portability constraint that follows from the correctness host and the bench host being two different machines; the regression policy; and eight dependency-ordered tickets.

**No optimization work is filed, because no bottleneck is measured.** Nothing in this rung establishes a latency, a throughput, or a device-optimal claim, and the record says so in its own terms rather than leaving it to a reader.

**Tickets filed from this rung.**

| Order | Ticket | Outcome | Waits on |
| --- | --- | --- | --- |
| 0 | [`land-the-model-level-qualification-record`](land-the-model-level-qualification-record.md) | The drafted body below lands byte-identically at `docs/research/program-planning/model-level-qualification.md` and the research catalog gains its row. | this ticket |
| 1 | [`measure-the-model-level-comparison-envelope-under-the-target-realization`](measure-the-model-level-comparison-envelope-under-the-target-realization.md) | The admissible model-level comparison bound for C1 is measured from the pinned reference alone, under the joint perturbation the qualified target's realization actually applies. Closes the `Unknown` L1 left. | [`retain-the-qwen-conformance-reference-logit-fixture`](retain-the-qwen-conformance-reference-logit-fixture.md) (done) |
| 2 | [`define-the-model-level-conformance-corpus`](define-the-model-level-conformance-corpus.md) | The adversarial corpus beyond C1 exists as named rows with the exact refusal each must produce, including the consistently-wrong-cursor case no other suite can reach. | 0, 1 |
| 3 | [`build-the-model-level-measurement-harness`](build-the-model-level-measurement-harness.md) | One harness produces every measured row with a three-state record schema, counted populations, and a demonstrated failing perturbation per check. | 0, [`drive-the-complete-forward-pass-over-three-artifacts`](drive-the-complete-forward-pass-over-three-artifacts.md) |
| 4 | [`qualify-the-model-level-claims-per-apple-device-and-toolchain-row`](qualify-the-model-level-claims-per-apple-device-and-toolchain-row.md) | The per-claim device and toolchain matrix becomes a maintained record with `Unknown` where a row is unmeasured. | 3 |
| 5 | [`supply-the-model-level-benchmark-protocol-to-cost-calibration`](supply-the-model-level-benchmark-protocol-to-cost-calibration.md) | The one activation input [`calibrate-device-cost-models`](calibrate-device-cost-models.md) still names as missing — a reproducible benchmark protocol — exists against this workload. | 3 |
| 6 | [`define-the-model-level-regression-policy`](define-the-model-level-regression-policy.md) | Correctness gates, performance reports, and the environment-change rule are separated, with no latency threshold set before a baseline exists. | 2, 3 |
| 7 | [`measure-b1-d-peak-residency-on-a-named-host`](measure-b1-d-peak-residency-on-a-named-host.md) | L1's own condition for extending the benchmark matrix above 8,320 positions is met or the bound is confirmed. | 3 |

**Nothing is filed for** a model-level latency target, a tokens-per-second guarantee, a kernel or schedule improvement, a second conformance host, or a quantized-versus-F32 acceptance threshold — each is either eliminated in the record below with its ground, or owned by a rung that already has it.

## Drafted record body — verbatim-landable at `docs/research/program-planning/model-level-qualification.md`

**How to transfer it, and the two mechanical steps that are named rather than hidden.** The span below the rule is the destination file's complete content. Two transformations apply and nothing else changes: the fenced YAML block becomes the file's delimited frontmatter, restoring the `---` delimiters the fence stands in for; and every `###` heading in the span is promoted to `##`, since the span is nested one level under this heading and the destination is not. A transfer that edits anything else is a fork rather than a transfer.

**The link condition, stated beside the span rather than repaired inside it.** Every relative link in the span is written to resolve from `docs/research/program-planning/`, which is where it will live and where it does not resolve from `tickets/`. That is the standing convention `AGENTS.md` records for a drafted body, and repointing them here would trade a reader's inconvenience for the byte-identity that makes the span landable at all. Every link in this ticket *outside* the span resolves from `tickets/`.

---

```yaml
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.model-level-qualification"
kind: "research"
title: "Model-level correctness and performance qualification"
topics: ["program-planning", "conformance", "qualification", "performance", "measurement", "language-model", "metal", "numerics", "regression"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.correctness-and-testing", "tiler.contract.numerical-semantics"]
depends_on: ["tiler.research.program-planning.first-metal-lm-workload", "tiler.research.program-planning.complete-model-ingestion-and-execution", "tiler.research.runtime.autoregressive-state-and-kv-cache", "tiler.research.numerics.first-quantized-lm-profile", "tiler.research.apple-targets.numerical-behaviour", "tiler.research.scheduling.first-metal-contraction-realizations", "tiler.research.cost-model.bootstrap-cost-model"]
ticket: "design-model-level-qualification-and-optimization"
```

### Model-level correctness and performance qualification

**Status:** durable design record for rung L8 of the language-model inference ladder. It is a research outcome, not a capability: nothing here registers an operation, admits a key, widens a budget, fixes a normative contract, sets a threshold, or authorizes implementation. It moves no row of the [operation-family support matrix](../../roadmap.md#operation-family-support-matrix) and no cell of the [dtype support ledger](../../dtype-support.md). **It contains no measurement of any Tiler execution, because none exists.** What it delivers is how "this model is correct and optimized on this Metal target" becomes four separately supported claims, the derivation of the model-level comparison bound L1 deliberately left `Unknown`, the corpus and matrix each claim needs, the bench-host discipline, the attribution ladder, the regression policy, and eight dependency-ordered tickets.

### Traceability

- **Work record:** [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md), which holds the scope derivation that placed this file here and the drafted body it was transferred from.
- **Ladder position:** rung L8 of [the roadmap's language-model ladder](../../roadmap.md#the-ladder). Its trigger reads "L1 and L6 deliver"; L1 delivered on 2026-07-31 and L6 on 2026-08-01, both under the **design-rung** reading of capability wording that Tom fired for L5 on 2026-07-31 and that every delivered rung has followed since.
- **Inherited, not re-derived.** [L1's workload profile](first-metal-lm-workload.md) supplies the pinned checkpoint and manifest, the two bounded rows, the oracle's five observables, the effective numerical policy, the qualified target row, and the measured F32 sensitivity envelope. [L5's state contract](../runtime/autoregressive-state-and-kv-cache.md) supplies the ten state properties, the four failure cases, the three cache-identity invariants, and the per-step host round trip. [L6's ingestion and execution record](complete-model-ingestion-and-execution.md) supplies the three programs and thirty executions, the three artifact identities, the model-level peak-residency arithmetic, the attribution surface, and the five failure classes. [L7's quantized profile](../numerics/first-quantized-lm-profile.md) supplies the quantized comparison path and its measured C1 observables. [L3's realization record](../scheduling/first-metal-contraction-realizations.md) supplies the bench host and its timing procedure.
- **Governing contracts read as evidence, not edited:** [Correctness and testing](../../correctness-and-testing.md) for the differential-testing shape, the requirement that every oracle name a conformance level, and the performance-testing list; [Numerical semantics](../../numerical-semantics.md#conformance-levels) for the conformance-level vocabulary and for the rule that cost may never rank contracts against each other; [Region accuracy contracts and analyzable error budgets](../numerics/region-accuracy-contract.md) for why a model-level bound is not the sum of per-operation tolerances; [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md) for the qualified row's measured F32 subnormal flush and for what remains unmeasured on it; [the initial cost model and calibration plan](../cost-model/bootstrap-cost-model.md) for the component vector and the rule that compile time and artifact size are separate objectives; [ADR 0033](../../decisions/0033-semantic-validation-enforcement.md) for the five-step device completion observation; [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md) for the rule that an empirical qualification is not a normative guarantee; [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) for target-honourable realizations; [ADR 0086](../../decisions/0086-require-attributable-or-attested-native-translation.md) for the unknown native translation identity.
- **Retained evidence this record reads:** [the Qwen3-0.6B-Base C1 conformance and attribution reference fixture](../../../spikes/program-planning/qwen3-conformance-fixture/README.md).

Claims are labelled **Fact** when traced to inspected source, primary documentation, or a merged record, **Inference** when derived from stated facts, **Measurement** when tied to an exact environment and procedure, and **Proposal** when not yet accepted or tested.

### The four claims, and what each may say

**Proposal — the separation, stated so that a reader can tell in one line which claim a sentence belongs to.** The ticket's user-visible outcome is that "this model is correct and optimized on this Metal target" stops being one sentence. It becomes four, and the point of the separation is that each fails differently and each is repaired by different work.

| Claim | Question it answers | Verdict form | What it may never be |
| --- | --- | --- | --- |
| **Correctness** | Does the executed program agree with the pinned reference, under a conformance level named before results were observed? | pass / fail, per named observable, per position | traded for speed; softened after a result is seen; reported without naming the operations it covers |
| **Feasibility** | Can this plan run at all on the declared target? | admitted / refused, with a typed reason | expressed as a large or infinite cost; conflated with a correctness failure |
| **Estimated cost** | What does the analytical model predict, and with what evidence class per component? | a component vector with `Exact`, `Bounded`, and `Unknown` kept distinct | quoted as a latency; used to rank two different numerical contracts |
| **Measured performance** | What did this exact build do on this exact host under this exact procedure? | a `Measurement` naming host, toolchain, procedure, and spread | generalized past its host; taken before the correctness oracle passed on the same build |

**Inference — the fourth column is the load-bearing one.** Each entry names a failure this corpus has already seen or already forbids somewhere else: the numerical contract forbids treating a flush-tolerant plan as a cheaper alternative to a preserving one; the architectural contract forbids hiding infeasibility behind a cost; the cost model's own plan forbids converting compile time into GPU nanoseconds; and L6 already owes a refusal for a conformance comparison reported without naming the operations it covers. This record adds no new prohibition — it states which of the four claims each existing one binds.

**Inference — two of the ticket's requested metrics are correctness gates wearing performance clothes, and classifying them wrongly would lose them.** Dispatch count and materialization count are not measurements on this workload: L6 fixes one forward pass at exactly **30 executions over exactly 3 artifact identities**, and the complete C1 row at **270 executions**, and records that "a build that produced a fourth would have specialized something it must not". Those are exact invariants a test asserts and that can fail; reporting them as performance numbers would let a fourth identity appear as a regression to be triaged rather than as a build that must not ship. They are listed under correctness below and appear in the performance section only as inputs to a cost-per-dispatch reading.

### Correctness: the oracle, and the bound this rung owns

#### The oracle is L1's and is not re-derived

**Fact.** Five observables after prefill and after every decode step: logit agreement under a `bounded error` level; greedy-token equality; the declared tie policy, where the greedy token is the lowest vocabulary index attaining the maximum and a bit-identical top-two pair is recorded as a tie rather than resolved; termination as EOS 151643 or the row's fixed budget and never implicitly; and plan determinism on the Tiler side alone. L1 also fixes the method: an error bound is a relation between two complete computations and is not the sum of per-operation tolerances, so nothing here composes one.

#### The bound: three named perturbations, derived before any Tiler result exists

**Fact — what is already measured, and exactly what it covers.** The retained fixture evaluated the pinned reference under two independently legal F32 orderings and retained the per-position deviation. Across all 18 C1 positions the largest whole-vocabulary deviation is **2.048e-4** (`f64_unmodified`) and **2.007e-4** (`f64_promoted`); restricted to the reference's own top-32 entries it is 7.82e-5 and 7.44e-5, at most 78 ULP under both. The greedy token agrees at every position under both, and between 483 and 3,863 of each position's 151,936 logits are bit-identical between orderings — under 3%.

**Inference — that envelope is one of three terms and is not the bound.** It measures what *reordering* does. The qualified Metal realization differs from the CPU reference in two further ways that the envelope cannot contain, and a budget built from the envelope alone would be a number that looks derived and is not.

**Proposal — the three perturbations, each with its authority named, and each fixed before observation.**

| Perturbation | What it models | Authority it is derived from |
| --- | --- | --- |
| **P-reorder** — evaluate under an independently legal F32 ordering | reduction order, which L1 measured to be the dominant term on this row | the retained envelope; promoting the reference's three float32-pinned stages moved it by about 2%, so the contractions are what the deviation is made of here |
| **P-flush** — flush F32 subnormal inputs and results to sign-preserving zero at every arithmetic site | the qualified target's own realization, which the CPU reference does not share | **Measurement.** On `apple9-f32-unified-msl4-macos26` both compilation paths flush F32 input *and* result subnormals, the flush preserves the sign of zero, and arithmetic-free materialization is unaffected — findings 2, 3, and 4 of [the numerical-behaviour record](../apple-targets/numerical-behaviour.md), each admitted under its execution-witness guard |
| **P-elem** — perturb each subordinate elementary function to the worst result its *registered* accuracy contract admits | elementary-function results, the third divergence source L1 names | the contracts Tiler itself registers: `Ulp(tiler::ulp-reference-gap@1, 12)` for the exponential subordinate to both `tiler::softmax-f32@1` and `tiler::silu-f32@1`, and `Faithful` for the reciprocal square root subordinate to `tiler::rms-norm-f32@1` |

**Inference — the authority for P-elem is the registered contract and not Table 8.1, and the difference is a factor of three.** The Metal Shading Language Specification bounds `exp` at 4 ULP *under Apple's own ULP definition*, and the corpus crosses that metric gap once, through the single `RegisteredImplication::ScaledMetric` whose derivation carries the factor of three; the contract the compiler actually promises is therefore 12 ULP under `tiler::ulp-reference-gap@1`. A perturbation sized at 4 would be measuring a bound Tiler does not claim, and would under-state the admissible band by three times.

**Proposal — the admissible bound is the *joint* perturbation, measured once, not the sum of three maxima.** Evaluate the pinned reference on the exact C1 prompt with P-reorder, P-flush, and P-elem applied together, and retain the per-position and per-top-32 deviation against the plain F32 pass. That single quantity is the bound. **Inference — why jointly and not termwise.** [Region accuracy contracts](../numerics/region-accuracy-contract.md) establishes that an error bound is a relation between two complete computations and is not generally the sum of per-operation tolerances, because cancellation, correlated reuse, deleted materialization rounding points, and exceptional-value discontinuities all break the sum. Three separately measured maxima added together is that same forbidden composition at a coarser granularity, and it would be simultaneously unsound — the terms are not independent — and needlessly loose.

**Inference — the bound is admissibility, not proof, and the asymmetry is what makes it usable.** A Tiler result *outside* the band is a defect: no legal realization of this program under this contract on this target could produce it. A Tiler result *inside* the band is not thereby proven correct — it is only indistinguishable, at the model boundary, from a legal realization. That is why the bound is one of five observables rather than the whole oracle, and why the attribution ladder below exists.

**Inference — greedy-token equality stays an exact gate, independent of the bound, and that is derived rather than asserted.** L1 measured the smallest runner-up gap across all 18 C1 positions at **0.266**, about 1,300× the reordering envelope. Under the joint perturbation the band may widen, and the gate holds as long as the measured band stays below the smallest runner-up gap — a condition the measurement itself checks. If it does not hold, the correct response is to record that C1's margin is no longer wide enough to carry an exact greedy gate, not to keep the gate and hope. **Inference — and this is a property of this row.** L7 measured a candidate that reproduces the C1 sequence exactly while disagreeing with the baseline's argmax at three of eighteen positions, so sequence equality is not a substitute for either observable.

**Fact — the tie branch is unexercised by C1 and this is a fact about the prompt.** At all 18 positions exactly one index attains the maximum and no top-two pair is bit-identical. The tie policy is declared and must be implemented; C1 does not demonstrate it, and the corpus below is where a demonstrating row would have to come from.

**Measurement boundary — what the bound qualifies.** One prompt, one checkpoint revision, one reference revision, 18 positions, batch 1, greedy, F32. It qualifies nothing about a B1-length row, another prompt, another checkpoint, or the quantized path. Extending it to a B1 row is possible and costs what L1's retention policy says it costs: at 512 prompt tokens the complete logit set is 296.8 MiB, so a B1 comparison retains a bounded summary rather than every logit, and its bound is a different measurement rather than this one applied further.

#### The reference-side qualification needs no device, and that is the schedule's most useful property

**Inference.** P-reorder is measured. P-flush and P-elem are perturbations of a CPU reference. So the entire bound is obtainable today, on the correctness host, with no Tiler execution, no Metal compilation, and no live device — while L6's five refusals still stand between this design and a compiled model. The measurement that closes L1's `Unknown` is therefore not blocked by the ladder, and it is filed as the first ticket for that reason.

**Proposal — the mechanism, with its own verification, because a flush that does not flush would silently return the reordering envelope again.** The candidate mechanism for P-flush is the host FPU's own flush-to-zero mode, which on this Apple-silicon correctness host sets ARM `FPCR.FZ`; `torch.set_flush_denormal` is the reachable spelling and reports whether it took effect. **The verification is mandatory and is a positive control rather than a return value**: a hand-built tensor expression whose exact result is F32-subnormal must return a sign-preserving zero with the mode on and the subnormal with it off, in the same process, and the BLAS-backed contraction path must be checked separately from the elementwise path because they need not share the mode. If either control fails, the mechanism does not establish P-flush and the term stays `Unknown` rather than being approximated — the stop condition is stated that way in the ticket.

### The adversarial corpus, derived from refusals that already exist

**Proposal — the rows, and where each comes from.** The ticket asks for prompt, prefill, decode, exceptional values, sequence bounds, and persistent state. Every row below is derived from a boundary L4, L5, or L6 already named, so the corpus tests the design rather than inventing hazards for it.

| Row | Subject | Expected outcome | Derived from |
| --- | --- | --- | --- |
| C1 | the pinned 10-token prompt, 8 decode steps, 18 positions | five observables pass; 270 executions; exactly 3 artifact identities | L1, L6 |
| A-prompt-1 | a 1-token prompt | prefill at `T = 1`, `C = 0`; the zero-extent cache path still taken | L5's P2 |
| A-token-low / A-token-high | token IDs `0` and `151935` | admitted; gathered in range | L2's bounds obligation |
| A-token-out | token ID `151936` | **refuses** at the gather's named enforcement boundary; never clamps, wraps, or reads out of bounds | L6's typed refusals |
| A-eos-in-prompt | EOS 151643 supplied as a prompt token | admitted; termination is a decode-side rule, not a prefill one | L1's termination rule |
| A-tie | a prompt reaching a position with a bit-identical top-two pair | recorded as a tie; greedy token is the lowest attaining index | L1's tie policy, unexercised by C1 |
| A-tiled-guard | a decode step at `S ≡ 0 (mod 16)` and one at `S ≢ 0` | both route through one artifact identity; the tiled variant is selected exactly at the multiple | L4's feasibility predicate 2; C1 hits it once in nine, at `S = 16` |
| A-mask-value | an additive mask entry that is neither `0xff7fffff` nor `0x80000000` | detected; a mask is a host computation with exactly two admitted values | L4, L6's host-computation surface |
| A-capacity | `C + T = capacity`, then `capacity + 1` | admitted, then **refuses at the runtime instance** naming required context and capacity | L5's growth rule |
| A-position-range | a declared context exceeding `max_position_embeddings = 32768` | **refuses at the semantic layer** — a different refusal with a different remedy | L5, L6 |
| A-cursor-consistent | a *consistently* wrong cursor: cache extent, rotary rows, and mask all derived from the same wrong `C` | **only the conformance oracle detects it**; every Tiler layer accepts it | L5 case 1 |

**Inference — A-cursor-consistent is the row this rung alone can supply, and it is why the corpus is not a duplicate of the state-failure suite.** [`test-the-autoregressive-state-failure-cases`](../../../tickets/test-the-autoregressive-state-failure-cases.md) covers incorrect position, stale state, partial update, cross-device reuse, capacity exhaustion, and specialization contamination — every one of which is either refusable or an *inconsistency* between the cursor and something derived from it. L5 states exactly what survives after a single cursor authority removes the inconsistency mode: "a wrong `C` produces a consistently wrong program that only the conformance oracle detects." That failure has no refusal, no ordinal, and no intermediate that disagrees with the reference's own intermediates at the same wrong position — it is a correct execution of a different program, and the model-level oracle is the only layer that can see it.

**Inference — two rows a reader would expect are deliberately absent, and their absence is derived.** A *subnormal weight* row is unreachable from this checkpoint: L1 records that BF16 is a truncated F32, so even a BF16 subnormal widens to an F32 **normal** and the target's flush cannot touch it. A row supplying one would test a path this workload does not have, and would invite the reading that the workload has subnormal weight inputs. A *NaN or infinite weight* row is likewise not a corpus case but a one-line check against bytes the fixture already digests — the retained `host.tsv` carries one digest over all 310 widened F32 tensors — and it belongs to the ingestion ticket rather than to a conformance corpus.

### Feasibility is a separate verdict, and the harness must be able to say it

**Fact — a qualification run today refuses at five places, and they are five different remedies.** L6 enumerates them with their exact sites: the deterministic budgets refuse all three programs (P2 exceeds three of four); the whole-program recognizer refuses all three and a budget widening does not admit a transformer; the inline route refuses a symbolic region; the facade cannot dispatch; and the Candle custom-op path cannot carry a decoder layer at all.

**Proposal — so the harness's record schema has three outcomes per row and not two.** `refused` names a typed reason and a phase; `failed` names a post-commit failure with the execution ordinal and the token in flight; `disagreed` names a conformance observable and a position and deliberately carries no ordinal, because the model boundary has none. Collapsing `refused` into `failed` would report a budget limit as a numerical defect; collapsing either into a missing row would let a population that never ran read as a population that passed — the exact shape `AGENTS.md` names, where "a survey once reported forty-three worktrees uniformly clean" because an unresolvable command produced empty output.

**Proposal — and every row states its population.** The harness reports counted populations rather than pass rates: 18 positions, 9 passes, 270 executions, 3 identities, 310 bound tensors, 28 layers × 18 positions of attribution slices. A check that cannot say how many answers it expected cannot distinguish silence from success. The retained fixture's own validator is the model — 2,721 counted checks needing no model, each demonstrated failing under a named perturbation — and this harness inherits that discipline rather than restating it.

**Inference — a model-level fallback is all-or-nothing per forward pass and the corpus must contain the negative.** L6 fixes that the preflight decision is taken once before the first routing commit over the bound facts of all thirty executions, and that after it no execution may fall back. The corpus row is a build in which one execution's route is made to fail after another's commit: the required outcome is a refusal naming both ordinals, never a completed pass. **Inference — and for this workload the fallback arm is empty by construction**, because the effective policy is subnormal-flushing, contraction-free, safe-math F32 and a strict realization has no valid fallback under the Candle contract's own numerical-scope rule. The negative test is still required, because "empty by construction" is a property of this workload's contract selection rather than of the mechanism.

### Estimated cost: what the model says today, and what it may not be asked

**Fact — the analytical model produces no model-level latency and none may be quoted from it.** `tiler.cost.analytical.v1` reports nine governed components in canonical order with `Exact`, `Bounded`, and `Unknown` kept apart; exactly one, `Allocation`, is computed exactly, and eight are `Unknown`. It never enters dominance — the frontier refuses an estimate claiming the analytical key — so structural cost remains the sole pruning input and the analytical numbers are, today, an explain-only report. `Unknown` is deliberately not zero, because a caller substituting zero would report a plan as free.

**Fact — the calibration ticket's own activation is three-quarters satisfied and it names what is missing.** [`calibrate-device-cost-models`](../../../tickets/calibrate-device-cost-models.md) is `deferred` behind representative kernels, exact target profiles, devices, and a reproducible benchmark protocol; its own trigger 3 records that L1 "supplies three of the four activation inputs at once — representative kernels, the exact target profile, and the device — leaving only the benchmark protocol". **Inference — that protocol is this rung's, and it is one ticket rather than a section of the calibration work**, because the protocol is a property of the workload's shape and its host discipline, and the fitting, provenance, uncertainty, and drift policy are properties of the model.

**Inference — the cost model must keep at least four alternatives separately costed for this workload, and each pairing is already named.** L4 leaves D-A and D-B unmeasured on both sides; L6 shows the choice between them moves B1-d prefill peak residency from 26.1462 GiB to 10.1472 GiB and, with final-position logits, to 5.5111 GiB; L7 requires the packed-fused, explicit-dequantize, and F32 weight candidates to stay separately costed rather than assuming the first wins; and L5's variant selection over `S` is a per-execution guard rather than a costed choice at all. A cost model that collapsed any of these would be selecting on an unmeasured assumption, and "optimal means the lowest-cost valid plan, not the largest fused kernel" is what forbids it.

**Proposal — and hard feasibility never enters the cost.** A row that L6's five refusals reject is reported as infeasible with its reason. The temptation this rung is most exposed to is the one the numerical contract names by spelling: treating a flush-tolerant plan as a cheaper alternative to a preserving one. The qualification harness must therefore never rank two plans that resolved different numerical contracts, and its record schema carries the resolved contract on every row so that a comparison across two of them is detectable rather than plausible.

### Measured performance: the metrics, decomposed as the architecture forces

#### Time to first token is four terms, not one

**Inference — this is the decomposition, and it is forced rather than chosen.** A single TTFT number for this system is not attributable, because the four costs it would sum are paid at different times, by different layers, and against different caches.

| Term | When it is paid | What it is, exactly | Cold/warm axis |
| --- | --- | --- | --- |
| Artifact preparation | **build time**, at macro expansion | offline `metal` and `metallib` runs, MSL and metallib bytes, embedded byte-literal size, rustc time and peak memory | the **expansion** cache: cold publishes, warm hits and performs no compilation work |
| Runtime preparation | first execution, per live device | library load, function resolution, and compute-pipeline creation — **exactly 3 times cold and 0 times warm**, because L6 fixes three artifact identities | the **runtime** library and pipeline caches, which are per live device and context |
| Model load | once per model | the BF16→F32 widening of 596,049,920 parameters, producing 2,384,199,680 bytes of resident F32 weights, plus the weight-manifest verification | not cached; paid once per process |
| Prefill | per request | 30 executions over 3 identities at the request's `T`, ending in a host readback of the logits | not cached |

**Inference — three consequences follow and only the first is obvious.** "Cold TTFT" and "warm TTFT" differ by the runtime-preparation term alone, because artifact preparation is not at run time under the accepted inline AOT flow and model load is not repeated. The two caches are different caches with different keys and different lifetimes — one content-addressed and immutable on disk at build time, one scoped to a live device and context — so a single cold/warm axis over "the cache" would conflate them, and a report that did would be unable to say which one a change moved. And the runtime-preparation term is *exactly three pipeline creations*, which makes it an invariant to assert rather than a number to watch: a cold run creating four pipelines has specialized something, and L5's third cache-identity invariant says exactly what — a build that specialized on `S` would mint a distinct pipeline per decode step, nine at C1 and one hundred and twenty-nine at B1-d.

#### Decode latency carries an irreducible host round trip, by construction

**Fact.** L5 derives it: a greedy decode loop cannot form step *n+1*'s input token without reading step *n*'s logits on the host, which crosses ADR 0033's device completion boundary in full — terminal completion, a post-completion status check, error-record visibility and coherence, record validation, and only then interpretation. The cursor's advance is tied to that same observation and adds no synchronization.

**Proposal — so decode latency is reported as two numbers with the gap stated.** GPU time from the command buffer's own timestamps, and wall-clock per token. **Inference — a GPU-time-only report under-states per-token latency by an amount that is a property of the design rather than of the implementation**, and a reader comparing it against a system that batches or that samples on device would be comparing different quantities. Prefill is different and the report must say so: its twenty-eight layer executions share one submission and only the final logits need a readback.

**Proposal — tokens per second is admitted only as the reciprocal of a measured per-token latency, and is labelled as that.** L1 already excluded aggregate throughput because batch is 1 and a throughput figure would need batching the profile deliberately excludes. Restated here because "tokens per second" is the phrase most likely to reappear as a batched number once a harness exists.

#### Peak and persistent memory: the measurement falsifies the arithmetic

**Fact — L6 supplies the figures and labels every one an Inference.** C1 prefill 2,394,286,488 B (2.2299 GiB); C1 decode 8 ≤ 2,393,069,056 B (2.2287 GiB); B1-d final decode ≤ 6,203,791,360 B (5.7777 GiB); B1-d prefill 28,074,307,592 B (26.1462 GiB) unfused, 10,895,486,984 B (10.1472 GiB) under D-B, and 5,917,459,976 B (5.5111 GiB) under D-B with final-position logits.

**Proposal — the measured quantity is peak resident bytes on a named host, and the qualification gate is agreement with the arithmetic rather than a threshold.** A measured peak *above* the row's stated sum means a plan allocated something the design did not account for; a measured peak materially *below* it means the transient column's bound is loose in a way worth recording. **Inference — so this measurement's primary value is not performance at all.** It is the only check that can falsify L6's residency arithmetic, and the arithmetic is what three separate design decisions rest on: D-16's 1.714 GiB, the D-A-versus-D-B choice, and the final-position projection's 4,978,027,008-byte saving.

**Fact — and it is the condition L1 attached to extending the benchmark matrix.** L1's exclusion table reads "Contexts beyond 8,320 tokens … A residency measurement on a named host, under L8", and its B1 section says the same: "Extending the matrix upward is legitimate work; it needs a residency measurement on a named host first, and it belongs to L8." That measurement is filed, and until it exists no row above 8,320 positions may be added to the matrix.

#### Dispatch, materialization, and cache behaviour

**Proposal — the exact counts are correctness assertions and appear here only as denominators.** 30 executions per forward pass; 270 for the C1 row; 3 artifact identities; one weight-set binding per layer per pass with the tied `[151936, 1024]` matrix bound twice per pass; the tiled value-contraction realization selected at exactly one of C1's nine executions, at `S = 16`. A per-dispatch cost reading divides a measured pass time by these; a *change* in any of them is a build defect and not a performance movement.

**Proposal — materialization count is reported per program and per plan, because the two decompositions differ in it.** L4's D-A and D-B differ precisely in which intermediates are materialized, and L6's residency table shows the model-level consequence. A single model-wide materialization count would be a number that changed for two unrelated reasons.

### The bench-host discipline, and the amendment the model level forces

**Fact — two hosts, and the split is not incidental.** Every retained numerical, conformance, and attribution digest in this corpus was produced on an **Apple M4 Max**, macOS 27.0 build 26A5388g, and the qualified target row `apple9-f32-unified-msl4-macos26` is measured there. Every retained timing in this corpus was produced on the **Apple M3 Pro** bench host, macOS 27.0 build 26A5378n, Xcode 26.6 build 17F113, offline compiler `metalfe-32023.883`. **Fact — the two report the same highest GPU family and identical threadgroup limits**, differing only in buffer and working-set size, so nothing about the split is visible in a capability query and a number from the wrong host is not detectable by inspecting the device.

**Proposal — the procedure, inherited from L3 unchanged.** One process; five interleaved A/B rounds; one warm-up per round; "settled" is the minimum over rounds 1–4 with round 0 reported separately because one warm-up does not remove a pair's first-encounter cost; the settled spread across rounds is reported alongside every figure. Interleaving rather than sequential blocks is what keeps a thermal or clock drift from being attributed to whichever variant ran second.

**Inference — and here is the amendment, which L3's own measurement forces.** L3 recorded that its `w_decode_kv` cell's 15.5 µs implies 270 GB/s, "above this host's DRAM bandwidth", because the harness reuses one buffer across dispatches and the `[1024, 1024]` operand is cache-resident — and it stated the consequence exactly: "A real decode step walks 28 layers of distinct weights totalling about 1.76 GiB and cannot be." **A model-level harness that reused L3's protocol unchanged would reproduce that artefact at 28× the scale**, because the A run of an interleaved pair would leave the second run's weights warm. Three amendments follow, and each names what it protects:

- **Interleave whole forward passes, never dispatches.** A dispatch-level A/B inside one forward pass has A warming exactly the operands B is about to read.
- **Never reuse one weight allocation across the interleaved variants.** The 2.22 GiB weight set is the thing being walked; sharing it makes the second variant's read a cache hit that the first paid for.
- **Report the achieved bytes-per-second beside every latency, and check it against the host's DRAM bandwidth.** L3's artefact was caught by exactly that division. A reported rate above the host's memory bandwidth is a residency artefact, not a fast kernel, and the harness should say so rather than leaving it to a reader who happens to divide.

**Proposal — the correctness oracle passes first, on the same build, or the timing is not a performance measurement.** `AGENTS.md`'s performance loop requires preserving a correctness oracle and comparing outputs before comparing speed. At the model level this is stronger than usual, because L6 shows a wrong weight binding produces a correctly shaped, correctly typed, plausible logit vector that every layer accepts — so a build that is timing a *different computation* is not visibly different from one that is not. The harness's record therefore carries the conformance verdict of the same build on every timing row, and a timing row without one is malformed rather than merely unlabelled.

**Fact — one field on every measured row is `Unknown` and must stay so.** ADR 0086 and L1 both record that exact native translation identity is unavailable through the AOT route: the source-JIT compiler `metalfe-32023.921` qualifies the `newLibraryWithSource` comparison rows and is not evidence about the AOT route, and substituting it would certify a relationship no measurement established. A measured-performance row taken through the AOT route records the offline compiler `metalfe-32023.883` as artifact provenance and the native translator as `Unknown`.

### The Apple device-family and toolchain matrix, per claim

**Proposal — the matrix is per claim because the claims have different device dependencies**, and the precedent for splitting it that way is the numerical-behaviour record's own compile-side/device-side table, which reaches three families unevenly and says so.

| Claim | Device requirement | Toolchain identity that qualifies it | State on the qualified row |
| --- | --- | --- | --- |
| Reference-side bound (P-reorder, P-flush, P-elem) | none — a CPU and the pinned reference | Python, torch, transformers, and the host row in `environment.tsv` | P-reorder measured; P-flush and P-elem measurable today |
| Correctness, Tiler side (the five observables) | live Apple9 device, macOS | offline `metalfe-32023.883`; native translator `Unknown` | blocked by L6's five refusals |
| Feasibility, compile side | none | offline `metalfe-32023.883` | available today; this is what refuses now |
| Feasibility, device side (pipeline creation, dispatchability) | live device | the execution environment's own runtime compiler | macOS/Apple9 only |
| Estimated cost | none | the target profile identity | 8 of 9 components `Unknown` |
| Measured performance | the M3 Pro bench host | offline `metalfe-32023.883`; native translator `Unknown` | blocked |

**Fact — the family coverage, and why one row of it is not evidence about what it looks like.** macOS is measured. A physical iOS device is unmeasured, because none is attached. The iOS Simulator **dispatches on the host Mac GPU** under a different device name, with the same registry ID within every run that dispatched both — so a simulator result is a measurement about the simulator and not about iOS-device hardware, and it additionally refuses `bf16` pipeline creation after compiling and linking the module successfully. **Inference — that refusal is the shape of failure this matrix exists to keep visible:** a device can compile, link, and then decline to run, so "it compiled for this target" is not a dispatchability claim, and a qualification matrix that inferred one would be wrong on a row this corpus has already measured.

**Fact — `registryID` is not a row key.** It is a within-run correlation handle whose measured lifetime on the correctness host is bounded from below and which changed at least once between retained records for the same named Apple M4 Max; ADR 0086 eliminates it by name as an applicability predicate. The matrix's row key is the named profile — `apple9-f32-unified-msl4-macos26` — plus the host, OS build, and toolchain builds.

**Proposal — every unmeasured cell reads `Unknown` and is never predicted from a neighbour.** The corpus has already refuted one such prediction with a measurement: the subnormal flush was inferred dtype-independent from a module-level declaration and measured to be false, `f16` preserving what `f32` flushes. The same rule applies across device families, across toolchain builds, and inside the integer domain.

### The attribution ladder, and the constraint that the two hosts impose on it

**Fact — L6 built the surface and deliberately did not compare against it.** The retained fixture holds per-layer `h_out` digests, per-layer post-RoPE `k_rope` and `v_heads` digests, the rotary table in full, the mask in full, the four host computations, and a digest over the widened F32 weights — with the digest unit one *position* of one tensor rather than one tensor, "which is what resolves a disagreement to a pass as well as to a layer". L6 states that nothing has been compared against it and that the propagation question is this rung's.

**Proposal — the ladder, in bisection order, each rung naming exactly what it eliminates.**

| Rung | Compare | Eliminates |
| --- | --- | --- |
| 0 | N repeated executions of one artifact, same inputs, same device, against each other | "disagrees with itself" — needs no reference, and separates plan non-determinism from every reference disagreement below |
| 1 | the four host computations against `host.tsv`, `rotary.tsv`, `mask.tsv` | the rotary table, the additive mask, the widened weight set, and the token IDs — all checkable in full and the cheapest rung |
| 2 | the weight binding manifest against the pinned safetensors header | the `57! · 56!² · 28!⁴` shape-preserving permutation class that every Tiler check is indifferent to |
| 3 | P1's `x0 [T, 1024]` against the reference's embedding output | the gather, including the consistently-wrong-index case |
| 4 | per layer and per position: `h_out`, `k_rope`, `v_heads` against `hidden.tsv` and `cache.tsv` | 27 of the 28 layer executions, and the attention half from the MLP half within the failing one |
| 5 | P3's logits against `positions.tsv` and `top32.tsv` | the final normalization and the vocabulary projection |
| 6 | the five observables at the model boundary | nothing — this is the pass/fail, and it names a position rather than an ordinal |

**Inference — and here is the constraint composing L1's boundary with the bench-host split produces.** Every digest in the fixture is bound to the correctness host's exact CPU, thread count, and BLAS; the fixture's own boundary says a mismatch on another host is *expected* and is not by itself a defect. The one portable column is `l2_norm`, computed by exactly-rounded summation over exact float64 squares, which depends on neither summation order nor host. **So an attribution run on the M3 Pro bench host cannot use the digests at all.** Three responses exist and only one is free:

- **Attribute on the correctness host and time on the bench host, never mixing.** Costs nothing, and is what this record recommends: the attribution ladder is a diagnosis tool for a conformance failure, and a conformance failure is found on the correctness host.
- **Retain a second host record beside the first.** L1 already names this as open — "a second host would turn that expectation into evidence; no ticket owns doing so, because nothing yet needs it" — and this rung does not make it needed, so it stays unowned.
- **Restrict bench-host attribution to `l2_norm` and the bounded top-*k* comparisons at the reference's own coordinates.** Available if a bench-host failure ever has to be diagnosed in place, and weaker than the digest ladder by exactly the per-lane resolution.

**Inference — the surface answers *where* and not *how far*, and that gap is deliberate.** The fixture carries no envelope on the attribution surface: the float64 passes are not hooked, so nothing says how far an intermediate may legally deviate. Whether a per-execution disagreement implies a model-level one, and by how much, is error propagation over 30 executions — and by the same rule that forbids composing the model bound from per-operation tolerances, it may not be composed from per-execution ones. **So the ladder localizes a failure and does not grade an intermediate.** An intermediate that differs is a lead, not a verdict; the verdict is at rung 6.

### Regression policy

**Proposal — correctness regresses as a gate, performance regresses as a report, and the two never share a threshold.**

- **Pinned, and a change is a failure:** the retained C1 record's checked-in files; the 18-token sequence; the greedy token and runner-up gap per position; the top-32 entries; the envelope; the three artifact identities; 30 executions per forward pass and 270 for the row; the cold pipeline-creation count of three; the peak-residency arithmetic; and the exact refusal each corpus row must produce.
- **Reported with an `EXPLAIN` diff retained, and triaged rather than failed:** every latency, every achieved bandwidth, every measured peak, artifact size, and expansion time. [Correctness and testing](../../correctness-and-testing.md) already requires the retained diff so a change is attributable to a plan, codegen, toolchain, or hardware-profile change; this rung adds only that a model-level report names which of the four it attributed to, or says it could not.
- **No latency threshold is set before a baseline exists.** A threshold chosen now would be the ad hoc number this ticket's own outcome forbids. The policy is that a performance row becomes a *gate* only after N recorded baselines on one host establish its spread, and the gate is then stated as a multiple of that measured spread rather than as a round percentage.
- **An environment change announces and declines to compare.** When the host, OS build, toolchain build, checkpoint revision, or reference revision differs from the retained record's, the harness reports the difference and refuses the comparison rather than accepting new values as the baseline. This is the rule the numerical probe's harness already enforces, and it is the one policy that stops a toolchain bump from silently rebaselining a conformance record.
- **An intermittent conformance failure is a defect in the mechanism.** It is root-caused and fixed; it is never re-run until green, never loosened, and never labelled flaky.
- **Every check is demonstrated able to fail.** The retained fixture is the standard: an altered logit digest failed against a stale manifest on both the manifest re-hash and the logit-byte re-hash, and an altered greedy token with a *consistently* re-hashed manifest still failed on the top-32 head and the decode-chain cross-checks. A model-level check without a recorded failing perturbation is not yet evidence.

### The quantized path: comparable without conflating claims

**Fact — L7 handed this question here and supplied its measured inputs.** Its record states that "whether the quantized program is an acceptable approximation of the F32 model is an ingestion and qualification question that `design-model-level-qualification-and-optimization` owns, and this record supplies its measured inputs rather than a budget."

**Proposal — the answer is structural: the quantized path carries *two* claims and only the first is a compiler-correctness gate.**

1. **Exactness against the quantized program's own reference, at zero tolerance.** L7 derives it: the decode's evaluation order is fully determined and its only rounding is one correctly rounded F32 multiply, so a backend disagreeing by one bit is wrong rather than approximate. This is the compiler's obligation and it fails a build.
2. **Acceptability against the F32 model.** L7 measured it on C1 for the selected per-output-channel strict-affine U8/F32 profile: the 18-token sequence reproduced exactly, greedy agreement 17 of 18 with the projections quantized and 17 of 18 with the tied embedding included, median whole-vocabulary logit deviation 1.08e-1. **This is a profile-qualification result and never a build gate.**

**Inference — folding the second into the first is the error this separation prevents, and L7 already proved why the F32 bound cannot be reused.** A quantized program is a *different computation*, not a different realization of the same one. The gentlest surviving candidate's median whole-vocabulary deviation, 1.08e-1, is roughly 500× L1's whole-vocabulary reordering envelope of 2.048e-4, so a tolerance derived from the F32 bound would be unachievable; a tolerance widened to admit the quantized path would be vacuous for the F32 path. Two computations, two references, two claims — which is what lets the two paths be *compared*, on the one axis they share: the model observable, measured the same way for both.

**Proposal — so the comparison table the qualification harness produces has one row per path and one column per claim**, and the F32 row's correctness column cites the joint-perturbation bound while the quantized row's cites zero tolerance against its own reference. Neither path's correctness column ever cites the other's number.

### What this rung does not decide

- **Any latency, throughput, or device-optimal claim.** Nothing here measures a Tiler execution, and L7's own analytical projection for a fused quantized decode is explicitly a hypothesis with a named experiment attached.
- **Any performance threshold.** No baseline exists on any host for any row of this workload.
- **Whether D-A or D-B is the prefill decomposition.** L4's, unmeasured on both sides. This record states what each costs at the model level and selects neither.
- **D-16, D-17, and D-18.** L6's, and untouched. D-16's trigger explicitly requires a measured decode-latency or peak-residency result at a B1 row where the 1.714 GiB is the binding constraint *and* a per-layer recovery contract; the residency measurement filed here supplies at most the first half.
- **Whether the quantized profile is adopted.** This record fixes how it is qualified, not whether its measured 17-of-18 agreement is acceptable — which is a product judgement.
- **Any second measurement host, and any B1-length conformance row.** Both are possible and neither is needed by anything today.
- **The exact public surface of anything.** The harness's record schema, the corpus's spelling, and any measurement API are drafts; acceptance of a public boundary is Tom's regardless of how the derivation ran, and this record requests no crate admission.

### Consequences for the ladder

**Inference.** L8's stated capability is "model-level correctness and performance qualification", and that is not what this rung delivered. What it delivered is the qualification design written down: four claims with what each may and may not say, the model-level comparison bound converted from L1's `Unknown` into a joint-perturbation measurement whose three terms each name an accepted authority, an adversarial corpus every row of which is derived from a refusal L4, L5, or L6 already owns, the per-claim device and toolchain matrix, TTFT decomposed into the four terms the architecture forces, the bench-host discipline with the amendment L3's own cache-residency artefact forces on it, the attribution ladder with the portability constraint the two-host split imposes, the regression policy, and the structural separation that lets the baseline and quantized paths be compared without conflating claims. Nothing compiles, dispatches, executes, or is measured on the Tiler side; no operation family moved a rung; the four-claim maturity vocabulary does not apply to a research record.

**Inference — the honest sequencing, which is the one useful thing this rung says about the ladder's shape.** The reference-side half of the bound is obtainable *today*, with no device and no Tiler execution, while L6's five refusals still stand. So the measurement that closes L1's `Unknown` does not wait on the ladder, and the qualification design is available before the thing it qualifies exists — which is the correct order, because a tolerance derived after a result is seen is not a tolerance.
