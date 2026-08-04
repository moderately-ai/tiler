---
id: audit-the-general-compiler-pipeline-against-the-semantic-program-model
title: Audit the general compiler pipeline against the semantic program model
status: done
priority: p1
dependencies: []
related: [implement-general-dag-partitioning, admit-ordered-multi-output-programs-at-the-compiler-request-boundary, accept-the-public-compiler-facade-boundary]
scopes: [research/program-planning, research/region-search, research/scheduling, contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, compiler-pipeline, audit, mimo]
---
## User-visible outcome

The documented and planned compiler pipeline demonstrably accepts the public typed
MIMO semantic program rather than only the three shapes recognized by the first
prototype.

Trace the complete construction and consumption path for the eleven intended stages:
semantic verification, normalization, logical exploration, region enumeration,
lowering-capability resolution, index-region lowering, complete-cover enumeration,
scheduled-region exploration, complete physical-plan selection, structured-kernel
refinement, and kernel-program assembly. Then trace backend emission, artifact build,
runtime preflight/routing commit, and execution.

For each boundary record its typed input/output, governing identity, validation and
explain obligations, unsupported cases, and whether merged code, a contract, only a
proposal, or no owner exists. Pay special attention to general DAGs, ordered named
outputs, multi-result operations, symbolic extents, materialization, transfers,
memory lifetimes, and the current premature `select_supported_strategy` collapse.

This is a read/design audit. It may create and repair tickets and documentation but
does not authorize implementation or acceptance of a public compiler facade.

## Closes when

Every stage has a maturity classification and exact owner; missing bridges are filed
with dependency-correct edges; no physical choice has leaked into semantic identity;
and the critical path to a naive but general compiled MIMO program is explicit.

## Outcome, 2026-08-04 — the audit ran; the collapse moved and is now named where it lives

Traced at `c1110ea9` by reading the compile path in full: `pipeline.rs`, `pipeline/planning.rs`, `request.rs`'s recognizer, `cover.rs`'s policy, `frontier.rs`'s governed provider, `program.rs`'s artifact refinement, and the four stages below the compiler. Every claim below cites a file read at that revision.

**The premature `select_supported_strategy` collapse this ticket names no longer exists at that function.** The recognizer stopped being a whole-program template match on 2026-08-01: it checks three program-wide properties — at least one declared input, exactly one output, `f32` throughout — and then classifies the occurrence producing the output, walking outward through the occurrences feeding it, at any declared input arity and over the general `PointwiseF32Expression` vocabulary. `normalize_serial_sum`, the function four documents still cited as the arity wall, does not exist; `grep -rn 'normalize_serial_sum' crates/` returns one historical mention in a test's doc comment.

**The collapse moved downstream, to two sites, and naming them is this audit's main result.** `GovernedPhysicalProvider::propose` (`crates/tiler-compiler/src/frontier.rs`) compares a cover region's exact semantic member set against the partitions the recognized strategy pre-computed and offers nothing otherwise; `build_plan_program` (`crates/tiler-compiler/src/pipeline/planning.rs`) and `verify_artifact_refinements` (`crates/tiler-compiler/src/program.rs`) each implement exactly three plan shapes and classify anything else as invalid compiler output. So stages 1–4, 7, and the per-subject stages 5, 6, 9, 10 are general over an arbitrary verified DAG, and stages 8 and 11 are not. The general DAG partition search that landed 2026-08-04 runs on every compile and produces partitions nothing downstream can realize — which is exactly why `CoverPolicy::governed` states the exact-partition admission, and its own doc comment names these two sites as the reason.

**One consequence a reader most needs stated: the request boundary's `output-arity` refusal is load-bearing.** It is what keeps the mismatch between a general search and a three-shape assembly from surfacing as a mid-pipeline `"unsupported-plan-shape"` compiler fault instead of a typed refusal. Relaxing it ahead of stages 8 and 11 would be strictly worse than refusing.

**No physical choice has leaked into semantic identity, checked rather than assumed.** `crates/tiler-ir/src/semantic/identity.rs` encodes declared inputs with key, resolved type, and shape; operations in canonical traversal order with operands as canonical value ids and every result's `result_index`, type, and shape; and outputs in *declaration* order with no sort. Multi-result operations and ordered named outputs are both in the encoding, and no region, cover, schedule, target, or plan value reaches it. The one ordering defect in the corpus is at the artifact layer, not the semantic one — the kernel-program identity encoder sorts its output records — and it is `carry-artifact-program-output-order-into-kernel-program-identity`'s, `in-progress` at this writing.

**Two stage-vocabulary defects in the contract, both corrected.** `ExplainStage::CandidateEnumeration` carries documented stages 3 and 7 and never stage 4 despite the resemblance of its name, and fusion-legality derivation is an authority on the compile path with its own explain stage that the eleven-stage list omits entirely.

**Where each finding landed.** Contract corrections in `docs/compiler/optimizer.md` (the stale `normalize_serial_sum` reachability caveat, the two stage-vocabulary defects, and a new per-stage generality section), `docs/compiler/fusion-and-scheduling.md` (duplication is a stated legality contract now, off for a downstream reason), and `docs/architecture.md` (the post-compiler half's maturity, which is general for multi-entry and mixed for multi-output). Research corrections in `docs/research/program-planning/general-compilation-boundary.md` (stale admitted subject, plus the critical path this ticket asks to make explicit), `complete-model-ingestion-and-execution.md`, `first-attention-program-vertical.md`, and `docs/research/region-search/exhaustive-region-oracle.md` (status raised to `partial`; the ninth budget that never became real). Ticket repairs on `admit-ordered-multi-output-programs-at-the-compiler-request-boundary` and `define-the-minimum-correct-physical-realization-profile`.

**No bridge was filed as a new ticket, and that is a decision rather than an omission.** The two unowned bridges — a region-general physical provider and a cover-general program assembly — are precisely what `define-the-minimum-correct-physical-realization-profile` exists to specify before anyone builds, and it is `todo` with its dependency `done`. Filing implementation tickets ahead of that definition would fix an interface before the correctness argument that shapes it. The evidence they need is recorded on that ticket instead.
