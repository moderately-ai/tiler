---
id: spike-a-red-yellow-first-full-conformance-suite
title: Spike a red-yellow-first full conformance suite
status: todo
priority: p1
dependencies: []
related: [shape-the-conformance-corpus-for-target-multiplication, survey-what-belongs-in-the-conformance-crate, derive-the-conformance-evidence-ledger-cells-from-executed-runs, replace-host-kir-simulator-claims-with-authoritative-evidence, delete-the-two-host-kir-simulators, inventory-the-closed-world-conformance-claim-universe-by-owner, cost-protected-review-versus-signed-conformance-authority, define-the-conformance-obligation-and-evidence-requirement-algebra, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles, decide-how-owner-private-conformance-inventories-cross-crate-boundaries, define-the-canonical-conformance-receipt-join-and-freshness-model, derive-the-optimizer-and-planner-capability-obligation-manifest, derive-the-five-family-structural-conformance-manifest, classify-machine-compilation-and-execution-outcomes-by-stage, design-the-conformance-audit-regress-and-qualify-command-contracts, spike-the-serial-sum-canonical-receipt-spine, assemble-the-first-versioned-conformance-goal-profile, design-the-machine-readable-and-explorable-conformance-report, design-the-conformance-denominator-and-receipt-perturbation-suite, produce-the-conformance-duplicate-equivalence-ledger, authorize-the-pkmt-conformance-authority-mechanism-implementation, bind-protected-review-and-signed-conformance-authority, define-retained-performance-claim-authority-and-identity, define-the-owner-emitted-performance-claim-manifest-contract, derive-artifact-proof-and-publication-conformance-obligations, derive-runtime-route-completion-and-cache-obligations, design-protected-review-authority-for-conformance-policy, design-the-exact-source-pk-conformance-authority-composition, design-the-external-mixed-diff-conformance-attestation, design-threshold-signed-five-class-conformance-authority, design-witnessed-conformance-authority-history-and-recovery, establish-external-mixed-diff-conformance-attestation, establish-protected-review-authority-for-conformance-policy, establish-threshold-signed-five-class-conformance-authority, establish-witnessed-conformance-authority-history, migrate-retained-performance-evidence-to-owner-claim-identities, record-the-pkmt-conformance-authority-architecture]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, spike, conformance, testing, conformance-progress]
---
# Spike a red-yellow-first full conformance suite

## User-visible outcome

A bounded design and evidence spike for one versioned conformance system that can report Tiler's semantic, reference, optimizer, schedule, kernel, artifact, compilation, runtime, device, numerical, and performance capability obligations without turning incompleteness into an omitted test or a permanently broken ordinary gate.

The intended first result is deliberately mixed **red / yellow / green**, not an all-green migration. The spike must make an incomplete profile honest and explorable before it tries to make the implementation more complete.

## Asked for

Tom asked on 2026-08-24 for one in-progress spike containing the current investigation, proposed path, and duplicate-retirement discipline. He specifically called out the failure mode in which LLM coding agents optimize for every commit looking green and thereby delete, skip, weaken, relabel, or narrow tests that were meant to expose unsupported capability.

Tom clarified later on 2026-08-24 that the spike's final output should be a defensible family of immediately actionable research, design, decision, and eventually implementation tickets. That instruction supersedes the temporary prohibition on descendants below. It does **not** authorize premature implementation: unresolved authority, schema, identity, oracle-independence, profile-scope, threat-model, and migration-safety questions become bounded research or decision tickets first. Movement from an accepted design into implementation remains Tom's decision.

## Exact-base note

**Measurement — 2026-08-24.** The read-only audit was performed on clean local `main` at `cc9b082dbc34db98e7d7fd677254baaae56112ba`. `git rev-list --left-right --count origin/main...main` returned `0 1`: local `main` was one commit ahead of `origin/main`.

**Measurement — coordination state.** `tkt reconcile --format json` returned exit 3 with six pre-existing findings: three `branch-without-active-ticket` findings and three orphan review/measurement branches. This ticket records that state rather than treating the board as synchronized. The spike's factual counts and file classifications must be re-audited at the base from which later work is dispatched, per repository policy.

**Measurement — review base on 2026-08-24.** The design review was performed on `main` at `50a3d132dfd25245d194dfb0d45e1d2431351cb5`, with `git rev-list --left-right --count origin/main...main` returning `0 0`. The ticket and its opening comment were still untracked in the main worktree, the Codex claim was live, and no matching `tkt/spike-a-red-yellow-first-full-conformance-suite` branch or worktree existed. `tkt reconcile --format json` still reported the same six unrelated findings. This is safe for review and ticket-family authoring, but it is not a dispatchable branch state until the ticket graph is committed and a ticket branch/worktree is created from that commit.

## Problem

The repository has extensive tests, typed receipts, exact oracles, rejection cases, and several real device verticals, but no one finite denominator answers:

> For this accepted profile, which exact capabilities and obligations exist and are required, which were exercised, at what layer and under which evidence kind, authority, scope, and freshness, on which target and environment, and what is missing or failing?

Counting tickets is not such a denominator. Counting `#[test]` attributes is not one either: loops hide expanded cases, multiple tests can protect the same fixture at different layers, and one green construction test does not imply production reachability, optimizer retention, selection, lowering, execution, or semantic preservation.

An ordinary test runner also creates the wrong incentive if every unsupported capability is represented as a failing unit test. Agents will be rewarded for converting red into `#[ignore]`, `#[cfg]`, early return, `XFAIL`, a smaller corpus, a weaker oracle, or an `N/A` declaration. Conversely, keeping the whole ordinary branch red makes unrelated changes hard to evaluate.

## Governing facts already established

**Fact — the conformance crate is an evidence consumer, not a second semantic authority.** The crate header in `crates/tiler-conformance/src/lib.rs` says it owns cross-layer executed evidence, not semantic definitions, benchmarks, or layer-local tests. A host that cannot perform a measured half reports `Measured::Unavailable`; it does not establish support.

**Fact — target multiplication already has an accepted shape.** [`shape-the-conformance-corpus-for-target-multiplication`](shape-the-conformance-corpus-for-target-multiplication.md) rejects a flat Cartesian generator and accepts one lifecycle protocol, algebraic family-specific case declarations, a separately supplied run context, a singular reference authority under the selected realization, and structured evidence reports.

**Fact — migration boundaries were previously surveyed.** [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md) establishes that cross-layer executed evidence moves, layer-local evidence stays, and oracle plumbing remains with its owning layer. A test does not become a duplicate merely because another layer uses the same input.

**Fact — a run cannot assign maturity authority.** [`derive-the-conformance-evidence-ledger-cells-from-executed-runs`](derive-the-conformance-evidence-ledger-cells-from-executed-runs.md) finds that a run can derive or report an evidence qualifier and can check a claimed tested guarantee for a matching receipt, but cannot decide that a reserved seam has become implemented support or stamp a normative guarantee.

**Fact — the accepted correctness contract requires more than agreement.** [`docs/correctness-and-testing.md`](../docs/correctness-and-testing.md) requires independent semantic/reference/backend comparisons, positive and negative optimizer-rule tests, bounded exhaustive frontier checks for small graphs, exact grouping-specific reduction oracles, and subject perturbations that show checks can say no.

## Working hypothesis

One conformance system should have **two different kinds of green**:

1. The **harness-integrity gate** stays green when it emitted a complete and honest report, including red and yellow capability cells.
2. A **profile-qualification command** stays red until every obligation required by that profile has sufficient passing evidence.

The repository's ordinary gate runs the first. Explicit qualification runs the second. A third regression command rejects unexplained loss of previously established evidence.

This keeps normal development usable without converting unsupported capability into a skipped or expected-success test.

## Proposed model

The design review found that the denominator must not originate in the goal profile alone. A profile is policy and can omit a feature; it cannot be the discovery authority for everything the system implements. Keep three identities separate:

1. a **system-universe snapshot**, derived from owner-side typed registries, vocabularies, accepted contracts, and explicit manifests, answering which feature claims exist in this source revision;
2. a **goal profile**, accepted policy selecting obligations from that universe and answering which claims are required for one qualification target; and
3. an **evidence snapshot**, joining immutable owner-produced receipts to feature, obligation, case, target, and environment identities and answering what was actually established.

Adding a feature claim without a disposition fails `audit` against the universe. Changing what the goal requires moves the goal-profile identity. Gaining, losing, weakening, or contradicting evidence moves the evidence snapshot. None may masquerade as another.

```text
Owner-derived system universe          exact feature/claim denominator
└── capability and invariant families
    └── stable feature claims          owner, identity, revision
        └── family-specific obligations
            ├── semantic admission and refusal
            ├── reference evaluation
            ├── production reachability
            ├── optimizer retention and selection
            ├── schedule verification
            ├── KIR and program verification
            ├── artifact, ABI, cache, and publication identity
            ├── offline compilation
            ├── runtime routing and fallback phase
            ├── device execution and completion
            ├── independent oracle comparison
            └── bounded performance evidence
                └── owner receipts    derived observations, not edited status

Versioned goal profile                 accepted requirement authority
└── selects obligations from the system universe; never discovers them

Evidence snapshot                      joined observation authority
└── binds receipts to exact feature, obligation, case, source, and run context
```

### The full profile's axes

The profile must be able to declare, without pretending every cross-product cell is meaningful:

- operation family and semantic key;
- operand/result signatures and dtypes;
- shape, rank, extent boundary, and shape class;
- layout, view, offset, stride, alias, and placement class;
- numerical contract, realization, and comparison rule;
- optimizer rewrite, fusion, alternative, and selection obligation;
- schedule topology, synchronization, memory, and resource obligation;
- KIR/program/ABI/artifact identity obligation;
- target profile, deployment family, device, OS, SDK, compiler, and linker;
- compile, eligibility, preparation, execution, completion, comparison, and fallback phases;
- cache/publication and cross-process delivery obligations;
- source-maturity claim from its real owner, evidence requirement, applicability, availability, and retained-record identity;
- performance workload, metric, baseline, variance, and bounded environment when a cost claim is made.

Family-specific declarations choose the meaningful points. Dtype and operation are independent high-level views over the same cases, not parents of one another and not multiplicative progress scores.

### Separate observation, verdict, and evidence

Store neither a manually editable `green = true` nor one scalar status. Record three distinct subjects:

- **Observation:** the stage result that actually occurred, such as compiled, refused with a typed code, executed, mismatched, unavailable, or not observed.
- **Obligation verdict:** satisfied, contradicted, insufficient, or not applicable. A required typed refusal can therefore be satisfied even though the observed route refused.
- **Evidence requirement:** a predicate over typed evidence kinds, authorities, scope, context, and freshness. Semantic equivalence, reference evaluation, compilation, execution, measurement, exhaustive finite evidence, proof, and normative authority are not one total maturity ladder; several are orthogonal or incomparable.

The visualization derives:

- **green** — the obligation is satisfied by evidence meeting its exact requirement;
- **red** — the required route ran and failed, refused unexpectedly, produced wrong bits, or violated an invariant;
- **yellow** — not run, unavailable, expired, too expensive for this lane, or missing some required evidence kind, authority, scope, or freshness;
- **gray** — outside this exact accepted profile, with a reason and authority.

A yellow-to-red transition can be increased knowledge: an unavailable route finally ran and exposed failure. The regression logic must not reward preserving yellow by avoiding execution.

### Three commands with different contracts

Names are provisional; the separation is not.

| Command | Contract | Exit behavior |
| --- | --- | --- |
| `conformance audit` | Every goal cell is classified; counts, identities, receipts, and applicability are valid; no silent omission | Zero for an honest mixed-color report |
| `conformance regress` | Previously sufficient evidence did not disappear or weaken without an accepted profile/authority change | Nonzero on unexplained green loss or denominator manipulation |
| `conformance qualify <profile>` | Every required obligation has sufficient passing evidence | Nonzero until the requested profile is fully qualified |

`make full` should eventually run `audit` and `regress`, not require all of `qualify` to be green. The qualification command and its report remain runnable and retain their nonzero state.

## Anti-greenwashing requirements

The spike must test whether these controls are sufficient and identify which require authority outside the repository:

1. Status is derived from typed receipts; it cannot be edited directly.
2. The goal profile declares requirements, never `expected failure` or current implementation outcome.
3. Stable `CapabilityId`, `CaseId`, and `ObligationId` values make deletions visible.
4. Removing a goal cell leaves a tombstone and changes the denominator identity.
5. Adding a registered operation, provider, strategy, or target without a disposition fails the audit with the missing key printed.
6. Every run prints expected, discovered, executed, passed, failed, unavailable, and not-applicable populations. Zero execution is never a silent success.
7. `#[ignore]`, platform `#[cfg]`, conditional early return, unavailable hardware, timeout, and missing retained evidence cannot produce green support.
8. Required-to-optional or required-to-`N/A` is an authority change, not an implementation fix. It is reviewed separately from the code being qualified.
9. An implementation change cannot change its own goal profile, oracle, exception ledger, and evidence baseline in the same work item.
10. Every harness check is demonstrated by perturbing the **subject**, not its assertion, and the exact failure is retained.
11. Evidence binds source revision, profile identity, case manifest, oracle/reference authority, target profile, toolchain, device/environment, selected plan, artifact, and output.
12. A green-to-yellow/red transition under the same subject and context fails the regression command; a newly observed yellow-to-red result is retained honestly.
13. A corpus expansion may lower displayed completion without being called a regression. Denominator change and evidence change are reported separately.
14. No in-repository mechanism is claimed to defend against an agent authorized to rewrite the profile, verifier, baseline, and tests together. The spike must cost protected human review versus a signed external goal-profile/exception root.

## Discoveries so far

### Semantic and reference layers

**Measurement — source inventory, not progress.** Anchored `#[test]` counts found 484 semantic tests in 24 files under `crates/tiler-ir/src/semantic`, 182 reference unit tests in 13 files, and 152 reference integration tests in 14 files. Loops mean these are not expanded case counts.

**Fact — missing exact correspondence census.** `StandardSemantics::register` currently owns 19 semantic operation keys. `StandardReferenceProvider::register` expands those into 28 exact reference operation/signature rows because concatenate has seven arities and strict-affine operations have U4/U8 signatures. Existing checks are family-local; no one fail-loud manifest requires an explicit reference disposition for every semantic key.

**Fact — the cleanest shared-declaration seam is structural.** Gather, slice, concatenate, reindex, and broadcast semantic tests own inference, canonical encoding, and typed refusal. Their reference conformance tests repeat much of the same case enumeration to prove payload behavior. The obligations are distinct, but the case declarations and fixture construction are duplicated.

**Fact — independent expected data must survive.** The reference tests' hand-written payload permutations are independent oracles and must not be recomputed by the mapping implementation under test.

**Fact — exceptional-bit transport is incomplete.** Concatenate, slice, and gather carry overlapping F32 bit-preservation values. Reindex and broadcast currently use ordinary ascending bit patterns and do not establish transport of NaN payloads, signed zero, subnormals, or infinities.

**Fact — one isolated vertical is missing.** SiLU has rich scalar accuracy and workload tests but lacks the isolated public `SemanticProgramBuilder` to standard `ReferenceEvaluator` case already present for RMS and softmax.

### Compiler, optimizer, schedule, and KIR

**Measurement — source inventory, not progress.** The audit counted 115 focused compiler-pipeline tests, 94 external compiler integration tests, 265 schedule tests, and 137 kernel tests.

**Fact — several strong seams already exist.** Production frontier tests retain serial, split, and tree reductions beside one another; selection tests move the winner by perturbing declared cost; `SelectedPlan` and `SelectedPortfolio` are re-verifiable and tamper-tested; verified scheduled/KIR/program values retain layer identities; typed explain coverage is extensive.

**Fact — preservation is sampled rather than required per optimizer capability.** Among 25 focused pipeline test files, 20 mention direct compilation, 21 mention selection/portfolio, 18 mention explain, but only 10 mention a reference/oracle/interpreter and 8 mention KIR lowering/body. There is no denominator requiring each advertised rewrite or strategy to prove production reachability, retention, selection when claimed, exact selected identities, and independent semantic preservation.

**Fact — a few green-only tests overstate their evidence if counted as conformance.** Examples found include a same-shaped epilogue test that says it proves distinct stage owners but only checks compilation success, a general contraction-axis case that only checks `Ok(())`, and a softmax control that only proves more than zero requests compile. These may remain smoke tests, but they cannot satisfy optimizer-preservation cells without stronger receipts.

**Fact — host KIR simulators are already scheduled for retirement rather than promotion.** [`replace-host-kir-simulator-claims-with-authoritative-evidence`](replace-host-kir-simulator-claims-with-authoritative-evidence.md) and [`delete-the-two-host-kir-simulators`](delete-the-two-host-kir-simulators.md) require old simulator assertions to become semantic/reference, structural compiler, real execution, or explicitly unsupported evidence; the new suite must not invent another interpreter to make those rows green.

### Artifact, backend, runtime, and device

**Measurement — corrected source inventory, not progress.** The stated backend-side population contains **954** explicit `#[test]` / `#[tokio::test]` attributes across Metal, Metal AOT, artifact, runtime, build, conformance, the two serial-sum prototypes, and the Candle Metal prototype. The original `1,001` was false at both the original `cc9b082dbc34db98e7d7fd677254baaae56112ba` base and the review base `50a3d132dfd25245d194dfb0d45e1d2431351cb5`; the relevant trees did not change between them. Reproduce the corrected occurrence count with `rg -o '#\[(tokio::)?test\]' crates/tiler-metal/src crates/tiler-metal-aot/src crates/tiler-artifact/src crates/tiler-runtime crates/tiler-build/src crates/tiler-conformance/src prototypes/serial-sum-compile/src prototypes/serial-sum-run/src prototypes/candle-metal-adapter/src -g '*.rs' | wc -l`. The correction changes no design conclusion because this number was never the denominator.

**Fact — most receipt ingredients already exist.** Relevant owners include `PreparedCompilation`, `ArtifactProvenance`, `CompiledArtifact`, `VerifiedArtifactProgram`, `DeliveredRealizationRecord`, `PayloadPlanDeterminismReceipt`, `ProofSidecarBuilder`, `LiveExecutionContext`, `MeasurementBoundary`, and `Measured::{Ran, Unavailable, Failed}`. The missing piece is a canonical retained execution receipt joining them, not wholesale replacement of those authorities.

**Fact — compile-only absence can currently look green.** Ten Metal golden compilation tests and five Metal-AOT driver tests return early when no qualified toolchain resolves. They do not use the conformance crate's typed `Unavailable` result and therefore cannot be trusted as machine-evidence cells until migrated.

**Fact — execution evidence is lossy.** Current device verticals print plan, environment, launch, and comparison details but do not retain one canonical record linking every case, phase, terminal completion, artifact realization, and oracle result.

**Fact — the serial-sum prototype and conformance corpora have forked.** Thirteen identically named tests and the serial-sum/contraction proof-sidecar inputs appear in both. The conformance copy now derives its oracle contract from the packaged plan while the old prototype retains its earlier evaluator shape. The prototype pair still uniquely exercises independently built producer and consumer executables across a process/file boundary, so it cannot yet be deleted wholesale.

## Proposed first experiment

The first spike artifact should be test-only and private. It should not add a public crate boundary.

### A. Freeze a bounded universe/profile pair

Define a candidate owner-derived `SystemUniverseV1` with stable feature and obligation identifiers and exact or explicitly unknown populations. Define `GoalProfileV1` separately as accepted policy selecting requirements from that universe, with applicability, evidence predicates, and its own identity. Populate both from current accepted authorities even where evidence is missing. Produce the first mixed-color report without implementing new capability.

### B. Build one complete serial-sum receipt spine

Use serial sum because it already crosses semantic construction, exact reference evaluation, multiple physical alternatives, cost-based selection, schedule/KIR/program construction, proof sidecar, artifact publication, runtime routing, Metal dispatch, completion, and grouping-specific comparison.

Dual-write one canonical receipt beside current assertions. Include one serial case, one tree case, one split case, the four-versus-twelve contributor grouping distinctions, an unavailable-host outcome, and a subject perturbation that breaks the joined receipt. Delete nothing in this phase.

### C. Build the finite structural breadth pilot

Use five structural families: reindex, broadcast, concatenate, slice, and gather. The exact first manifest contains the structural subset's 11 reference signatures: one each for reindex, broadcast, slice, and gather plus concatenate arities 2 through 8.

One descriptor drives two independent runners:

- semantic inference and exact typed refusal;
- reference payload evaluation against literal expected outputs.

Add exceptional-bit transport for reindex and broadcast, exact expanded counts, and one true subject perturbation per family.

### D. Emit the optimizer obligation report

For every declared strategy/rewrite in the chosen bounded profile, report:

```text
construction
production reachability
retained beside valid neighbour
selectable
selection changes under target/cost perturbation
schedule receipt
KIR receipt
program receipt
independent oracle
typed refusal
explain coverage
measured cost, if claimed
```

Initially missing cells are yellow in the report and do not break `audit`. After the denominator is reviewed, `qualify` treats required missing cells as nonzero.

### E. Make machine absence typed

Prototype test-support outcomes `Compiled`, `Unavailable(reason)`, and `Failed(stage, detail)` for the fifteen conditional compile tests. Split compilation, eligibility, preparation, execution, completion, and comparison into distinct receipt stages. Convert ignored heavy correctness lanes into declared required-heavy obligations whose lack of a current receipt remains yellow.

## Duplicate-retirement rule

The sequence is **shadow, prove, then prune**. No mass deletion belongs in the first spike result.

A test may retire only when:

1. every old case maps to a stable new `CaseId`;
2. the subject, obligation, layer, and oracle are the same;
3. positive, negative, unavailable, and refusal populations are preserved;
4. each unique subject perturbation and exact failure survives;
5. old and new paths dual-run with equivalent dispositions at a named revision;
6. the replacement reaches at least the same construction and consumption sites;
7. moving it does not destroy a dependency-closure, public-boundary, or independent producer/consumer claim;
8. expected and observed population counts are reported before and after removal.

Likely early candidates, subject to re-audit at the working base:

- duplicated structural case declarations and local fixture helpers, while retaining separate semantic and reference drivers and literal expected arrays;
- construction-only reference negatives already owned by exact semantic refusal tests;
- three schedule-verifier replicas at the bottom of the compiler materialized-intermediate integration test, while retaining the compiler end-to-end rows;
- the repeated single-operation BF16 selected-plan assertion, while retaining its distinct contract-derived schedule/KIR assertions;
- copied serial-sum applicability, preflight, dispatch, and proof tests only after the cross-process receipt runner exists.

Explicitly not candidates merely because their inputs overlap:

- semantic fact/encoding tests versus independent reference mathematics;
- pipeline selection versus schedule/KIR identity versus executed numerical comparison;
- runtime tests whose subject is the consumer's negative dependency closure;
- `ProofSidecar`, retained-record parsing, Candle integration, independent-backend composition, or the serial-sum producer/consumer process boundary until equivalent evidence exists;
- BF16, contraction, strict-affine, RMS, SiLU, and softmax hand-derived or certified mathematical corpora.

## Spike work

1. Re-audit every fact above at the exact spike base and repair this ticket if any changed fact alters its purpose.
2. Produce a closed-world inventory plan covering owner-side feature vocabularies, including the optimizer, planner, schedule, verifier, KIR, artifact, runtime, numerical, target, and performance claims that are not user-visible.
3. Separate system-universe, goal-profile, and evidence-snapshot authority and file bounded research or decision tickets wherever any owner, identity, schema, or change policy remains unresolved.
4. Decide whether evidence requirements form a set/predicate algebra rather than an ordered maturity enum, and preserve source/maturity claims their real owners make without letting a run stamp them.
5. Compare the feasible owner-to-conformance observation boundaries without assuming that compiler-private structures should become public.
6. Decompose the serial-sum receipt spine, structural manifest, optimizer manifest, typed machine outcomes, report commands, perturbations, and duplicate-retirement ledger into dependency-complete descendants.
7. Cost the external-authority options: protected review over the goal profile, an independently signed profile/exception root, and the status quo. State which threats each does and does not address.
8. End with a Pareto-complete architecture and ticket graph: exact dependencies, authorities, scopes, non-goals, stop conditions, immediately ready work, and decision- or evidence-blocked work. Do not implement a descendant merely to make the root spike appear complete.

## Non-goals

- Do not make `make full` require a completely green capability profile.
- Do not encode unsupported capability as `XFAIL`, `#[ignore]`, early return, platform omission, or `N/A` merely to make a command succeed.
- Do not build a Cartesian operation-by-dtype-by-shape-by-target generator.
- Do not invent a second semantic, numerical, target-profile, maturity, or evidence authority.
- Do not create a new host KIR interpreter or treat source compilation as device execution.
- Do not migrate every layer-local test into `tiler-conformance`.
- Do not use test count, ticket count, frontier size, or one scalar percentage as the capability denominator.
- Do not delete existing tests before the replacement's scope and negative controls are demonstrated.
- Do not add a public conformance API without Tom accepting its exact included and excluded surface.
- Do not create follow-up **implementation** tickets until the research and decision prerequisites expose one dominant safe boundary. Tom has authorized the research/design family; that is not blanket implementation authority.

## Stop conditions

Stop and return to Tom if:

- one profile cannot identify a singular authority for any outcome it calls green;
- a required obligation cannot be represented without a new public boundary or maturity authority;
- the proposed receipt must duplicate rather than reference an accepted identity owner;
- an implementation and its oracle cannot be made independently perturbable;
- preserving the denominator requires weakening an accepted target or numerical contract;
- a proposed test retirement removes a dependency-closure, independent-process, or other negative architectural claim;
- protecting the goal profile requires authority Tom does not want outside ordinary repository writes.

## Acceptance

The spike is complete only when one reviewable result contains:

- the corrected per-Fact audit at the exact final base;
- the nondominated system-universe / goal-profile / evidence-snapshot architecture and its strongest counterarguments;
- a complete owner and visibility inventory, with every currently unenumerable internal feature family made explicit rather than omitted;
- a dependency-complete family of bounded research, decision, design, pilot, migration, and eventual implementation tickets, each naming authority, scope, non-goals, stop conditions, and readiness;
- explicit descendants for the serial-sum receipt spine, five-family structural manifest, optimizer obligation manifest, typed machine outcomes, report/command semantics, perturbation suite, anti-greenwashing authority, and duplicate-equivalence ledger, without pretending unresolved ones are implementation-ready;
- a threat analysis stating which guarantees require external human or cryptographic authority;
- the proposed first profile's denominator decision path, not an invented denominator ahead of its authorities;
- no test retirement and no new capability claimed merely because the harness or ticket family exists.

## Reproduction commands used for the initial audit

These commands count source attributes, not expanded cases or progress:

```sh
rg -o '^\s*#\[test\]' crates/tiler-ir/src/semantic -g '*.rs' | wc -l
rg -o '^\s*#\[test\]' crates/tiler-reference/src -g '*.rs' | wc -l
rg -o '^\s*#\[test\]' crates/tiler-reference/tests -g '*.rs' | wc -l
rg -o '#\[test\]' crates/tiler-compiler/src/pipeline/tests -g '*.rs' | wc -l
rg -o '#\[(tokio::)?test\]' crates/tiler-compiler/tests -g '*.rs' | wc -l
rg -o '#\[test\]' crates/tiler-ir/src/schedule -g '*.rs' | wc -l
rg -o '#\[test\]' crates/tiler-ir/src/kernel -g '*.rs' | wc -l
rg -n 'resolved_toolchain\(\)|resolved_system_toolchain\(\)|return;' crates/tiler-metal/src/golden_compilation.rs crates/tiler-metal-aot/src/driver.rs
git rev-list --left-right --count origin/main...main
tkt reconcile --format json
```

Every future census must state its searched spellings and why they cover the intended population. Where a typed registry exists, derive the denominator from it rather than freezing a hand-written count.

## External precedents used for orientation, not authority

- [LLVM lit test status results](https://llvm.org/docs/CommandGuide/lit.html#test-status-results) separates pass, expected failure, unexpected pass, unresolved, unsupported, and timeout; Tiler should retain the distinction but reject expected failure as the representation of required support.
- [PyTorch device-type test instantiation](https://github.com/pytorch/pytorch/blob/main/torch/testing/_internal/common_device_type.py) demonstrates reusable operation metadata driving dtype/device cases; Tiler's sparse non-monotone matrix still requires family-specific declarations rather than a universal Cartesian product.
- [Vulkan Conformance Tests](https://docs.vulkan.org/guide/latest/vulkan_cts.html) illustrate profile qualification as a conformance result rather than ordinary unit-test completeness.
- [NIST combinatorial testing guidance](https://csrc.nist.gov/pubs/sp/800/142/final) is relevant to bounded interaction coverage where exhaustive cross-products are infeasible.
- [Alive2](https://users.cs.utah.edu/~regehr/alive2-pldi21.pdf) is precedent for bounded translation validation of optimizer transformations without claiming an unbounded universal proof.

## Related repository records

- [`shape-the-conformance-corpus-for-target-multiplication`](shape-the-conformance-corpus-for-target-multiplication.md)
- [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)
- [`derive-the-conformance-evidence-ledger-cells-from-executed-runs`](derive-the-conformance-evidence-ledger-cells-from-executed-runs.md)
- [`replace-host-kir-simulator-claims-with-authoritative-evidence`](replace-host-kir-simulator-claims-with-authoritative-evidence.md)
- [`delete-the-two-host-kir-simulators`](delete-the-two-host-kir-simulators.md)
- [`docs/correctness-and-testing.md`](../docs/correctness-and-testing.md)
