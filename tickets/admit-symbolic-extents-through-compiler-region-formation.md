---
id: admit-symbolic-extents-through-compiler-region-formation
title: Admit symbolic extents through compiler region formation
status: review
priority: p1
dependencies: [admit-symbolic-extents-at-the-compiler-request-boundary]
related: [admit-live-extent-operands-to-payload-indexing, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, shapes, extents]
claimed_from: todo
assignee: worker-admit-symbolic-region-formation
lease_expires_at: 1786640549
---
## User-visible outcome

`compile()` of a symbolic semantic program is no longer stopped at the first strategy-selection refuse. A recognized symbolic program reaches region formation and either produces a scheduled region that still names its symbols or declines with a typed reason for the unsupported case.

## Exact gap

**Fact at `209e0f9fd5a18486039d859a5f47ccf260f0f8cf`, re-read this session.** [`admit-symbolic-extents-at-the-compiler-request-boundary`](admit-symbolic-extents-at-the-compiler-request-boundary.md) made a symbolic program reach strategy selection. Recognition still declines the first non-static extent as `RequestError::UnsupportedSymbolicExtent { phase: "strategy", rule: "symbolic-extent", extent }` from `static_shape` in `crates/tiler-compiler/src/request.rs`. Durable anchor: `A symbolic extent is refused here rather than resolved through the environment`.

**Fact, same base.** Later fail-closed gates still refuse a symbolic shape if reached: normalization `NormalizeError::Structure { rule: "symbolic-extent" }` in `crates/tiler-compiler/src/normalize.rs`, and region-graph construction `RegionError::Structure { rule: "symbolic-extent" }` from `value.shape().as_static()` in `crates/tiler-compiler/src/region.rs`. The normalization helper documents that it is not the compile path's first refusal.

**Imprecise, repaired from the live-extent review comment.** The comment on [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) said `compile()` declines at region formation. That is the later gate, not the first. The first refuse on today's compile path is strategy selection. Region formation would refuse if a recognizer ever returned a symbolic region. The working live-extent draft path is `ScheduledRegionBuilder` + `lower_scheduled_region`, which bypasses `compile()`.

This ticket owns the compile path, not the labelled kernel operand.

## Required work

- Re-audit `static_shape` / `static_shape_ref` in `request.rs`, every strategy recognizer that requires a fixed `Shape`, `normalize.rs` `static_shape`, and `RegionGraph::from_program` at the exact base before editing.
- Let a named, bounded population of recognizers accept a symbolic input or result shape without folding `ExtentSources::determined` into the logical plan. Unrecognized or unlowerable symbolic cases still decline as `UnsupportedSymbolicExtent` naming the extent, never as a mis-attributed handle or signature rule.
- Carry the program's own environment through any rewrite so a rebuilt symbolic value cannot lose or swap the environment its identity folds.
- Teach region-graph construction to record a symbolic value instead of requiring `as_static()`. A hole in the graph is not an answer.
- Keep later payload compilation allowed to decline until the live-extent operand and envelope exist. This ticket does not invent those surfaces and does not claim a symbolic payload is executable.
- If a recognizer, region-graph record, or compile facade needs a new public type, produce the labelled draft and stop for Tom.

## Required evidence

- A symbolic elementwise neighbour that today's strategy refuses now reaches region formation, and its literal neighbour still compiles.
- A still-unsupported symbolic case declines with `UnsupportedSymbolicExtent` naming the extent. Remove the new path and watch the old strategy refuse return.
- A rewrite perturbation that would mint a symbolic value without the program's environment fails as invalid compiler output.
- `RegionGraph::from_program` no longer dies at `as_static()` for the admitted population; a value whose shape is still unrepresentable fails with a named rule.
- Targeted compiler tests, rustdoc, Clippy, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Live-extent kernel operands, Metal `eN` ABI, artifact envelope rows, `N = 14` / `N = 15` pipeline evidence, and lifting `AotRefusal::SymbolicExtent`. Those belong to the sibling remainders and to [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## Closes when

`compile()` of the admitted symbolic population reaches a scheduled region or a typed decline past strategy selection, specialization remains forbidden at the request boundary, and every new check is fail-capable.

## Fact audit — 2026-08-13 at base `1dc1c9d78c3a35b9c61993f970774f6afdd991bd`

Re-read this session: `crates/tiler-compiler/src/request.rs` (`static_shape`, `static_shape_ref`, `recognize_elementwise_output`, `plan_elementwise`, `recognize_reduction`, `normalize_contraction`, `recognize_staged_family`, `recognize_structural_read`), `crates/tiler-compiler/src/normalize.rs` (`static_shape`, `detect_shared_values`, `rebuild`, `rebuild_ordered_reassociation`, `revalidate_structurally`), `crates/tiler-compiler/src/region.rs` (`RegionGraph::from_program`, `GraphValue`, `encode_value_facts`), `crates/tiler-ir/src/semantic/program.rs` (`try_new`, `try_standard_with_shape_environment`, `input_resolved_sourced`), `crates/tiler-ir/src/schedule/model.rs` (`IndexRegion::iteration_shape` is `Shape`). Purpose unchanged.

- **Verified.** First compile-path refuse is strategy selection. Durable anchor: `A symbolic extent is refused here rather than resolved through the environment`. `static_shape` returns `RequestError::UnsupportedSymbolicExtent { phase: "strategy", rule: "symbolic-extent", extent }` when `sourced.as_static()` is `None`. `rg -n "UnsupportedSymbolicExtent|rule: \"symbolic-extent\"" crates/tiler-compiler/src/request.rs`.
- **Verified.** Normalization is a later fail-closed gate, not the first refuse. Durable anchor: `Strategy selection already declines a symbolic extent`. `rg -n "symbolic-extent" crates/tiler-compiler/src/normalize.rs`. CSE detection called `static_shape` on every value even when there were no merges, so admitting through strategy without teaching detection `SourcedShape` would have turned a valid program into `InvalidCompilerOutput`.
- **Verified.** Region-graph construction required `value.shape().as_static()` and failed as `RegionError::Structure { rule: "symbolic-extent" }`. Durable comment: `Every access, tile, and boundary derived below is stated over fixed extents`. `rg -n "rule: \"symbolic-extent\"" crates/tiler-compiler/src/region.rs`.
- **Verified (coordinator-unverified this session).** Recognizers that called `static_shape` / `static_shape_ref`: `recognize_elementwise_output` (output), `plan_elementwise` (leaf and result compare), `recognize_structural_read` (operand and result), `recognize_staged_family` (operands and result), `recognize_reduction` (contributor), `normalize_contraction` (operands). The honest population that can be admitted without a live-extent payload or a public `IndexRegion` change is **same-shape whole-program elementwise** (the existing `(a * b) + c` over `f32[n]` fixture). Reductions, contractions, staged families, and structural maps still need fixed extents for axis decode or element counts. `IndexRegion.iteration_shape` is `Shape`; producing a `ScheduledRegion` that names symbols would be a public IR type change. This ticket declines at schedule with `UnsupportedSymbolicExtent` rather than inventing that surface. No rewrite minted a symbolic value: every rebuild opened `SemanticProgramBuilder::try_new` with no environment.

## Implementation record — 2026-08-13

Admitted population: same-shape whole-program elementwise. Recognition compares `SourcedShape` equality and does not fold `ExtentSources::determined`. `NormalizedPointwise.shape` is `SourcedShape`; a wholly literal boundary still answers `as_static()` and keeps the previous request-subject bytes. CSE detection keys on `SourcedShape`. Rewrites open `SemanticProgramBuilder::try_new_with_shape_environment` on the program's own registry and cloned `ShapeEnv`, and mint symbolic inputs through `input_resolved_sourced`. `RegionGraph` records `SourcedShape`; `encode_value_facts` keeps the previous static encoding when `as_static()` answers. `compile()` records region formation, then declines as `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent", extent }` because `IndexRegion` still requires a fixed launch geometry. Payload compilation is not claimed.

`SemanticProgramBuilder::try_new_with_shape_environment` is the registry-taking counterpart of `try_standard_with_shape_environment`. It is not a new compile-facade type. `CompilationRequest` stays `pub(crate)` with no new public field.

### Measurement boundary at this commit

- **Request and strategy admit same-shape symbolic elementwise.** The program still names its symbols.
- **Normalization and region formation accept that population.** A rewrite carries the program's environment.
- **Scheduled-region construction declines**, naming the extent. `IndexRegion.iteration_shape` remains `Shape`.
- **Reductions, contractions, staged families, and structural maps** still refuse a symbolic extent at the first `static_shape` they hit.
- **A bound symbol is still a symbol.** An environment that pins `n == 4` does not emit a `[4]` plan.

### Support matrix

No operation-family support-matrix row and no dtype-maturity cell moves. Same-shape elementwise now reaches region formation; that is a compile-path seam, not a family-admission widening.

### Evidence

- `a_symbolic_elementwise_program_is_recognized_with_its_symbols` — `verify_planned_request` succeeds; the recognized shape is `[n]`.
- `a_symbolic_elementwise_neighbour_reaches_region_formation` — `RegionGraph::from_program` and `form_region_candidates` succeed; `compile()` declines at `phase: "schedule"` naming `n`; the literal `[4]` neighbour compiles.
- `an_unsupported_symbolic_case_names_the_extent_and_the_literal_neighbour_compiles` — `compile()` of the symbolic elementwise names `n`, not a handle or vocabulary rule.
- `a_compiled_plan_does_not_fold_a_bound_extent_value` — an environment that pins `n == 4` still authors the symbol; `compile()` still refuses that symbol.
- `a_symbolic_elementwise_program_records_its_sourced_shape` — every graph value keeps `SourcedExtent::Symbol(n)`.
- `a_symbolic_common_subexpression_rewrite_keeps_the_program_environment` — CSE of `(a * b) + (a * b)` rebuilds with the same environment identity; `try_new` plus `input_resolved_sourced` fails.

### Perturbations

Subject, not assertion, each watched failing once:

1. **Remove the new strategy path** (`sourced_shape` replaced by `static_shape` in `recognize_elementwise_output`):
   ```
   thread 'request::tests::a_symbolic_elementwise_program_is_recognized_with_its_symbols' panicked at crates/tiler-compiler/src/request.rs:12295:14:
   same-shape symbolic elementwise must pass strategy selection: UnsupportedSymbolicExtent { phase: "strategy", rule: "symbolic-extent", extent: Symbol(...) name: "n" }
   ```
2. **Mis-attribute the schedule refuse** (`UnsupportedCapability { phase: "planning", rule: "region-vocabulary" }`):
   ```
   assertion `left == right` failed: an unsupported symbolic case must name the extent, got compile.unsupported.planning.region-vocabulary: no installed capability can compile this valid semantic program
     left: None
    right: Some(Symbol(...) name: "n")
   ```
3. **Mint a symbolic rewrite without the environment** (`rewrite_builder` always `try_new`):
   ```
   thread 'normalize::tests::a_symbolic_common_subexpression_rewrite_keeps_the_program_environment' panicked at crates/tiler-compiler/src/normalize.rs:1911:44:
   the rewrite must carry the program's environment: Rebuild { rule: "input" }
   ```
4. **Restore `as_static()` in `RegionGraph::from_program`**:
   ```
   thread 'region::tests::a_symbolic_elementwise_program_records_its_sourced_shape' panicked at crates/tiler-compiler/src/region.rs:5028:14:
   a sourced elementwise program has a graph record: Structure { rule: "symbolic-extent" }
   ```

### Identity blast radius

None on previously encodable subjects. Static `encode_value_facts` and `encode_explain_shape` keep the previous rank-plus-raw-`u64` path whenever `as_static()` answers. Symbolic subjects had no planned request or region identity before this commit. Existing request-subject, region-identity, and compile-path tests stayed green.

### Commands

From this worktree, after the implementation:

- `cargo test -p tiler-compiler` — 797 lib tests passed, 1 ignored; integration tests passed; rustdoc compile-fail tests passed.
- `cargo test -p tiler-ir` — 978 lib tests passed; integration and compile-fail tests passed.
- `cargo clippy -p tiler-compiler --all-targets -- -D warnings` — clean.
- `cargo clippy -p tiler-ir --all-targets -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler -p tiler-ir --no-deps` — clean.
- `tkt lint` — `ok: no problems found`.
- `git diff --check` — clean.

Did not run `make full`. Coordinator gates at integration.
