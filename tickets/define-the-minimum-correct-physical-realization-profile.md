---
id: define-the-minimum-correct-physical-realization-profile
title: Define the minimum correct physical realization profile
status: review
priority: p1
dependencies: [enumerate-the-mature-tensor-operation-and-signature-taxonomy]
related: [implement-general-dag-partitioning, admit-ordered-multi-output-programs-at-the-compiler-request-boundary, prototype-complete-physical-plan-selection, derive-physical-proposals-from-the-cover-region-subject, assemble-a-kernel-program-from-an-arbitrary-cover, activate-shared-work-duplication-on-the-compile-path]
scopes: [research/program-planning, research/scheduling, contracts/optimizer, contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning, correctness, baseline]
claimed_from: todo
assignee: agent-realization-profile
lease_expires_at: 1785881439
---
## User-visible outcome

Any supported semantic program has a deliberately simple, valid physical route even
when no fusion, tiling, parallel reduction, or calibrated cost model applies. Advanced
physical optimization improves that baseline; it is not a prerequisite for general
correct execution.

Define the minimum profile for arbitrary acyclic MIMO programs over the explicitly
supported operation/signature set. Cover deterministic topological partitioning,
ordered multi-output preservation, conservative materialization, explicit buffers,
placement/transfers, serial/direct kernels where legal, reference/host fallback only
if the architecture explicitly permits it, and fail-closed refusal where no legal
realization exists. Separate hard feasibility from cost and separate semantic
correctness from target availability.

Audit the current Pointwise/SerialSum/Contraction strategy selector and advanced
physical-plan research against this baseline. Identify which existing tickets close
general DAG partitioning, complete covers, output identity, buffer/lifetime planning,
and multi-entry assembly; file the missing bounded work. Do not design a sophisticated
optimizer here and do not claim a fallback for an operation without a defined
reference/numerical contract.

## The audit this ticket asks for has been run against the current tree (2026-08-04)

[`audit-the-general-compiler-pipeline-against-the-semantic-program-model`](audit-the-general-compiler-pipeline-against-the-semantic-program-model.md) traced all eleven compiler stages and the four below them at `c1110ea9`. Its findings are recorded in [the optimizer contract](../docs/compiler/optimizer.md#what-each-stage-is-general-over-today), [the architecture contract](../docs/architecture.md#where-the-post-compiler-half-is-general-today), and [the general compilation boundary](../docs/research/program-planning/general-compilation-boundary.md#the-critical-path-to-a-naive-but-general-compiled-mimo-program). The three that change this ticket's work, so it starts from evidence rather than re-deriving it:

**Fact — the "Pointwise/SerialSum/Contraction strategy selector" this ticket asks to audit is no longer the collapse, and auditing it as one would look at the wrong file.** `select_supported_strategy` in `crates/tiler-compiler/src/request.rs` stopped being a whole-program template match on 2026-08-01: it checks three program-wide properties and then classifies the occurrence producing the output, walking outward at any declared input arity. Region formation and cover enumeration are likewise general over an arbitrary verified DAG, the latter since the general DAG partition search landed 2026-08-04.

**Fact — the collapse is `GovernedPhysicalProvider::propose` in `crates/tiler-compiler/src/frontier.rs`, and this is the component this ticket's baseline most directly owns.** It compares a cover region's exact semantic member set against the member partitions the recognized strategy pre-computed — the pointwise strategy's whole set, the contraction strategy's whole set, or the reduction strategy's prologue, reduction, and whole-program sets — and returns `ProviderOffer::default()` for every other member set. Two consequences a baseline definition must account for. First, a cover the general search enumerates over any other region is dropped at complete-plan selection with *no* explain attribution: nothing rejected it, because nothing proposed for it, so a reader of the trace sees an absence rather than a reason. A minimum correct profile that fails closed with an explainable reason has to say something here. Second, this is what makes the general search unreachable: `CoverPolicy::governed`'s own doc comment names this provider and program assembly as the reason shared-work duplication stays off the compile path.

**Fact — the second half is program assembly, and it is a separate site with a separate failure class.** `build_plan_program` in `crates/tiler-compiler/src/pipeline/planning.rs` and `verify_artifact_refinements` in `crates/tiler-compiler/src/program.rs` each match the ordered scheduled regions against exactly three shapes and classify anything else as *invalid compiler output* (`"unsupported-plan-shape"` / `"artifact-strategy-cardinality"`), not as a refused program. A baseline that partitions topologically will produce four-region and wider plans immediately, so this is the first thing a deterministic topological partitioning would hit, and it would hit it as a compiler fault.

**Inference — the bounded work this ticket owes filing is therefore two implementation tickets, not one.** A region-general physical provider (proposal derived from the region subject it is handed, with a typed decline where it cannot) and a cover-general program assembly (stages, buffers, and materializations derived from the cover rather than matched against a shape). The audit deliberately did not file either: this ticket's stated work is to define what they must guarantee first, and filing an implementation ticket ahead of that definition would fix the interface before the correctness argument.

## Outcome (2026-08-04)

The profile is specified in [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md), written against the tree at `57474a09` with every fact carrying the exact check that reproduces or refutes it. Its shape, and what each part settles:

- **One property, `P`**, decomposed into five obligations — deterministic topological partitioning, ordered multi-output preservation, conservative materialization with explicit buffers, serial or direct kernels wherever legal, and fail-closed refusal with a reason. `P` is deliberately weaker than "every admitted program compiles": what it forbids is the third state, neither a plan nor a reason.
- **A stage-by-stage table over all eleven named stages plus the fusion-legality authority**, stating what the profile requires of each and its status. The result is that stages 1–7, 9's join, and 10 already discharge everything `P` asks; the whole gap is stage 8, stage 9's reporting, and stage 11.
- **Two walls read in full**, with one fact each stage's paragraph did not have: the provider matches member sets because the region builders below it take only the request and read the recognizer's subject — including the *role of the tensor they write* — and the assemblers do the same, consulting none of the cover's materialization edges.
- **A correction to the audit's explain account.** The audit recorded "no explain attribution"; the precise version is three defects. `PlanRejection::RegionUnimplemented` *is* constructed per cover (38 per governed compile, by the site's own count) and `SelectedPortfolio::rejections()` has no production reader — `grep -rn '\.rejections()' crates/ --include='*.rs'` returns seven sites, the three on `SelectedPortfolio` all inside `selection.rs`'s `#[cfg(test)]` module at line 1705. The frontier record that does reach the trace is keyed `region:{role}` and fourteen of seventeen subjects share `unrecognized`. Only the first sighting of a role is recorded, so thirteen emit nothing.
- **The host/reference-fallback question, answered structurally.** The architecture does not permit one and the enforcement is not a policy: `crates/tiler-compiler/Cargo.toml` carries `tiler-reference` under `[dev-dependencies]`, so a compiled program cannot contain a host route. The one "fallback" the architecture names is the consumer-side Candle one.
- **The taxonomy's D7 column consumed mechanically**, sorting all forty-seven families into covered-by-scalar-or-map route, covered-by-fold-or-loop, covered-under-a-stated-precondition, and not-covered-and-the-refusal-is-the-guarantee, with each family appearing exactly once. The rule that decides it is this ticket's own non-goal turned into a test: no minimum physical route or no numerical contract means no fallback, only a typed refusal.
- **Four open questions with closure tests**, including the one that bounds `P` itself (`Q-MPR-03`: whether the fully-materialized cover is feasible whenever any cover is — unresolved, which is why `P` promises a plan *or* a reason rather than a plan whenever any exists).

**Filed, with edges.** [`derive-physical-proposals-from-the-cover-region-subject`](derive-physical-proposals-from-the-cover-region-subject.md) (p1, no dependencies — reachable now) closes the stage-8 half and the explain half. [`assemble-a-kernel-program-from-an-arbitrary-cover`](assemble-a-kernel-program-from-an-arbitrary-cover.md) (p1) closes the stage-11 half and depends on the first, because a plan reaches assembly only when every region of its cover has an admitted implementation — so a generalized assembler's paths would otherwise be unreachable from the compile path and testable only by a fixture constructing a plan the compiler cannot produce. That derivation is stated so it can be refuted rather than only believed.

**Graph corrections made along the way.** [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md)'s closing condition 2 had absorbed the plan-shape generalization; it now depends on the assembly ticket instead, because a *single*-output program partitioned into four regions hits `"unsupported-plan-shape"` just as hard, so the work is general-baseline rather than output-arity work. [The optimizer contract](../docs/compiler/optimizer.md#what-each-stage-is-general-over-today) and [the general compilation boundary](../docs/research/program-planning/general-compilation-boundary.md#the-critical-path-to-a-naive-but-general-compiled-mimo-program) both stated the two walls as unowned; both now name their owners.

**One graph correction deliberately not made, and why.** [`activate-shared-work-duplication-on-the-compile-path`](activate-shared-work-duplication-on-the-compile-path.md) reached these same two walls independently — its "Why this is deferred rather than todo" section derives both from the same two source sites — so its activation triggers 1 and 2 should name the two filed tickets. That edit was written and then reverted: `git diff --name-only origin/main...tkt/sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired` shows that live branch has already committed to `tickets/activate-shared-work-duplication-on-the-compile-path.md`, so file-level disjointness does not hold and the edit is inadmissible under this repository's shared-scope rule even though the two branches' hunks are separate (theirs are the frontmatter `related` line and an appended "Trigger check log"; mine was the trigger list). The naming is a two-line edit owed after that branch merges, and both filed tickets record which trigger they fire in their own graph-maintenance sections so it is not lost. The independent arrival at the same walls is corroboration for this ticket's conclusion rather than duplication of it.

**Scope added autonomously.** `contracts/navigation` was added because a new research record requires its catalog entry in `docs/research/README.md`, which is the only navigation document listing research records individually (`grep -c 'general-compilation-boundary\|kernel-program-buffer-plan' docs/status.md docs/design-map.md docs/README.md docs/roadmap.md` returns 0 for each). The precedent is this ticket's own dependency, `enumerate-the-mature-tensor-operation-and-signature-taxonomy`, which declared the same scope for the same reason. The edit is one line. `correct-the-softmax-divergence-attribution-in-code` is the one live ticket also holding `contracts/navigation`; `git diff --name-only origin/main...tkt/correct-the-softmax-divergence-attribution-in-code` returned empty at the time of this edit, so file-level disjointness holds against that branch as it stood.

**What this ticket deliberately did not do.** It designed no optimizer, proposed no public type, registered no operation, and moved no maturity rung. `docs/compiler/optimizer.md`'s `evidence` frontmatter was left unchanged: the profile record's disposition is `pending`, and listing a pending record as contract evidence would overstate it.

## Closes when

There is a complete stage-by-stage correctness argument for the minimum supported
profile, every required component has a live ticket and dependency, unsupported
cases produce typed explanations, and advanced scheduling work is ordered after—not
substituted for—the generic executable baseline.

All four are discharged by the outcome above: the stage table is the argument, the two filed tickets with their dependency edge are the components, the typed-explanation obligation is stated per stage and carried into both tickets' closing conditions, and the ordering section states which advanced work is deliberately *not* ordered before the baseline and why.
