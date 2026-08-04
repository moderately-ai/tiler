---
id: define-the-minimum-correct-physical-realization-profile
title: Define the minimum correct physical realization profile
status: in-progress
priority: p1
dependencies: [enumerate-the-mature-tensor-operation-and-signature-taxonomy]
related: [implement-general-dag-partitioning, admit-ordered-multi-output-programs-at-the-compiler-request-boundary, prototype-complete-physical-plan-selection]
scopes: [research/program-planning, research/scheduling, contracts/optimizer, contracts/foundation]
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

## Closes when

There is a complete stage-by-stage correctness argument for the minimum supported
profile, every required component has a live ticket and dependency, unsupported
cases produce typed explanations, and advanced scheduling work is ordered after—not
substituted for—the generic executable baseline.
