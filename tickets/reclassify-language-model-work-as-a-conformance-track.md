---
id: reclassify-language-model-work-as-a-conformance-track
title: Reclassify language-model work as a consumer conformance track
status: in-progress
priority: p0
dependencies: [reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission]
related: [supersede-the-runtime-owned-kv-state-design, retain-the-c1-attention-block-conformance-evidence, retain-the-qwen-conformance-reference-logit-fixture]
scopes: [contracts/navigation, contracts/integrations, research/program-planning, research/runtime, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, conformance, language-model, roadmap]
claimed_from: todo
assignee: agent-reclassify-lm
lease_expires_at: 1785876199
---
## User-visible outcome

Language-model examples remain valuable end-to-end evidence, but they test the
generic compiler as consumer-owned tensor programs. They do not define Tiler's
product goal, semantic model, runtime state, or public API.

Inventory every language-model, attention, rotary, normalization, quantization,
prefill, decode, KV, and Candle-specific roadmap node. Classify each as one of:

1. a generic atomic operation or compiler/runtime capability, renamed and specified
   without workload ownership;
2. a consumer integration/conformance fixture that composes generic capabilities;
3. a performance study whose result may motivate a generic optimization; or
4. obsolete work whose premise is superseded.

Preserve exact numerical fixtures and measurements with their bounded profiles.
Ensure integration tests do not become alternate semantic authorities. Correct the
roadmap ladder and dependency graph, filing generic prerequisite tickets where a
workload currently hides a missing atomic building block.

## Closes when

Every affected node has one classification, core capabilities have consumer-neutral
names and contracts, application loops/state remain in consumer integration scope,
and the roadmap presents language models as one demanding conformance track among
many possible tensor workloads.

## Outcome

Delivered at `6fac3caf`. Forty-two nodes classified, one per node, recorded as a ticket tag so the classification is queryable rather than prose: **15** `class-generic-capability`, **19** `class-conformance-fixture`, **6** `class-performance-study`, **2** `class-obsolete` (the two boundary tickets [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md) had already closed, tagged here only so the inventory is complete). The population is the 41 tickets the mission reconciliation gated on this one plus every other open `language-model`-tagged ticket; reproduce it with `grep -l 'class-' tickets/*.md`. Delivered rungs' records are covered by the rung classification in the roadmap rather than tagged individually, and that boundary is stated there rather than left to inference.

[`docs/roadmap.md`](../docs/roadmap.md) gains three things the classification needed: a candidate-track table that makes "one demanding conformance track among many" concrete with four other workload classes and their status, the disambiguation between *authoring* tracks (which own contracts) and *conformance* tracks (which own none), and the classification scheme itself with its counts, its reproduction command, and the rule that a category-2 node's declared core-crate scopes are legitimate for tests and never for a public type whose name comes from the workload. The ladder's Capability column is now explicitly a description of a consumer program, with the delivered rungs' generic yield named separately. A conformance fixture is stated to be bounded evidence and never a semantic authority: when a fixture and an operation contract disagree, the contract wins and the fixture is the defect.

**Three findings changed something rather than only being recorded.**

1. **A generic operation family was scheduled behind a consumer loop.** Sub-tensor selection was designed inside [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md), which depends on [`integrate-the-autoregressive-decode-loop`](integrate-the-autoregressive-decode-loop.md), and [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md) depended on that in turn. Filed [`admit-the-sub-tensor-selection-family`](admit-the-sub-tensor-selection-family.md) dependency-free; both workload tickets now depend on it and keep their own triggers, and it is dispatchable immediately rather than after the whole decode chain. Its `IndexNode` premise was verified rather than inherited at `crates/tiler-ir/src/index/model.rs:94`–`109`: `SourcedExtent` appears in no variant except the `FloorDiv` and `Modulo` divisors, so a literal-offset selection is expressible today and a symbolic one is not.
2. **A ticket that reads as a runtime capability is not one.** [`name-the-execution-ordinal-in-model-level-failures`](name-the-execution-ordinal-in-model-level-failures.md) is classified as a fixture, on the fact that `route_with_adapter` at `crates/tiler-runtime/src/adapter.rs:496`–`555` is synchronous and returns every stage's refusal to its own call site — so the driver already knows which invocation failed, and an execution ordinal on a `tiler-runtime` type would be a caller's loop position in a consumer-agnostic public surface.
3. **Two capability tickets were named after the workload occurrence.** The gather ticket is retitled to "Admit an indirect gather access family" and its outcome sentence restated over the access class. **No ticket id was renamed**, deliberately: five records outside this ticket's scopes link to that file by name, `tkt rename` repoints ticket dependents and not documentation links, and no gate reports a broken one — so a rename would trade a workload-flavoured identifier for reader-visible breakage.

**Six rung status lines** moved from "rung Lx of the language-model inference ladder" to "… conformance track", matching the wording the supersession had already given L5: L1, L3′, L4, L6, L7, L8.

## Findings outside this ticket's scopes

**The two remaining rung signposts** are `docs/research/shapes/transformer-operation-and-shape-surface.md:19` (L2) and `docs/research/scheduling/first-metal-contraction-realizations.md:19` (L3). `grep -rn 'language-model inference ladder' docs/` returns exactly those two.

**Ten sentences survived the KV supersession** outside every correction marker, across `research/runtime`, `research/shapes`, `research/scheduling`, and `research/program-planning`. They are filed as [`complete-the-kv-ownership-supersession-sweep`](complete-the-kv-ownership-supersession-sweep.md) (p1) with every line quoted and cited, **undivided**: seven sit in scopes this ticket held and three do not, and a corpus where seven sentences say "the consumer" while three still say "the runtime instance" reads as a live disagreement rather than a finished correction. The sharpest is `docs/research/scheduling/first-metal-contraction-realizations.md:180` — "the attention score and value contractions wait on L5's state model" — which is not stale prose but the recorded reason two of the workload's three contraction index structures are unscheduled, against a model that no longer exists.

**Orphan sweep.** `tkt lint` clean. No live dependent waits on any of the 25 `closed` tickets (check positively controlled against a known closed-ticket dependent). Two `deferred` nodes block live dependents — `realize-the-tiled-contraction-schedule-and-its-metal-emission` (4) and `first-authoritative-ios-metal-compile-declaration` (2) — and both carry explicit activation triggers naming their own dependencies, so they are parked work rather than orphans and were deliberately left.

**No stop condition fired.** Every node classified from its own text plus inspected source; none required Tom to choose a product outcome, and no evidence contradicted the four-category scheme. Two product decisions already staged inside their own tickets are untouched and still Tom's: the `StorageScalar` carrier width in [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md), and D-18's budget widening in [`widen-the-deterministic-budgets-to-the-decoder-layer-program`](widen-the-deterministic-budgets-to-the-decoder-layer-program.md).
