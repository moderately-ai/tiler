---
id: admit-symbolic-extents-through-schedule-formation
title: Admit symbolic extents through schedule formation
status: blocked
priority: p1
dependencies: [admit-symbolic-extents-through-compiler-region-formation, accept-the-live-extent-operand-public-surface, decide-the-source-bound-live-row-major-access-surface]
related: [deliver-an-artifact-family-from-a-symbolic-region, carry-live-extent-operands-through-the-artifact-envelope]
scopes: [implementation/ir, implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, ir, shapes, public-boundary]
---
## User-visible outcome

`compile()` of a recognized same-shape symbolic elementwise program produces a scheduled region that still names its symbols — typically a `LiveRowMajor` plan over the declared `[n]` — or declines with a typed reason that is not the current schedule-geometry refuse. Specializing the plan on a representative literal extent remains forbidden.

## Why this exists

[`admit-symbolic-extents-through-compiler-region-formation`](admit-symbolic-extents-through-compiler-region-formation.md) deliberately stopped at schedule. Same-shape symbolic elementwise now reaches region formation; `crates/tiler-compiler/src/pipeline.rs` then returns `RequestError::UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }` unless the program carries a parametric broadcast. Durable anchors: `A sourced broadcast must reach physical selection` and `IndexRegion requires a fixed geometry`.

[`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md) lifted the frontend-local `AotRefusal::SymbolicExtent` gate at `bd9c65dd` so that refusal is what a `deliver macos;` consumer now sees. That ticket cannot form the scheduled region: `IndexRegion.iteration_shape` is a fixed `Shape` (`crates/tiler-ir/src/schedule/model.rs`). Live-extent operands already exist on the hand-built `ScheduledRegion` / `LiveRowMajor` path, not on `session::compile`, but the accepted carrier is narrower than this ticket originally implied: it names one live axis per accessed input and carries no common-source relation for three inputs whose semantic shapes all name `n`.

## Exact-base Fact audit — 2026-08-14, `67fc9cac2a53f65fdba7619b9516c6e5e7324f20`

The ticket's purpose survives, but its implication that accepted `LiveRowMajor` might already express the required three-input subject is false and triggers the public-boundary stop. No production file was edited before this audit.

1. **Verified — current boundary and failure.** [`admit-symbolic-extents-through-compiler-region-formation`](admit-symbolic-extents-through-compiler-region-formation.md) admits same-shape whole-program elementwise through recognition, normalization, and region formation. `pipeline.rs`, anchors `first_symbolic_extent` and `IndexRegion requires a fixed geometry`, then returns `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent", extent: n }`. `cargo test -p tiler-compiler request::tests::a_symbolic_elementwise_neighbour_reaches_region_formation -- --exact --nocapture` passes and observes that refusal after region formation.
2. **Verified — frontend relay.** Commit `bd9c65dd` removed the frontend-local symbolic gate. `crates/tiler-macros/src/aot.rs`, anchors `let batch = compile` and `Same-shape symbolic elementwise is recognized and formed`, calls the public compiler and renders its typed decline.
3. **Verified — fixed schedule geometry.** `IndexRegion::iteration_shape` is public `Shape`; `ScheduledRegionBuilder` holds `Option<Shape>` and accepts `iteration_shape(Shape)`; `encode_identity` writes that shape first. A rank-one live-inner plan could retain this field as a rank-zero static outer domain with one work item, so replacing the field is not intrinsically required for this slice.
4. **Verified, and narrower than the ticket stated — accepted live carrier.** `LogicalAccess::LiveRowMajor { inner_axis }` means a static outer product plus one live inner input-axis loop. `live_input_extents` derives one `(TensorRole, Axis)` for every input access carrying that map. `InputExtentParameter` names only that scheduled input and axis; neither carrier names a `ShapeSymbol`, a `ShapeEnvIdentity`, or an equality source shared by several inputs.
5. **False implication — repaired.** The required fixture is `(a * b) + c` over three distinct rank-one inputs whose sourced shapes are all `[n]`. Giving all three reads the existing map yields three `InputExtentParameter`s. `declare_plan_live_extents` declares all three, `emit_live_row_major` consumes one `columns` value, and `verify_input_extents` rejects the other two as `UnusedInputExtent`. Silently choosing the first input would change the public relation and let a hand-built schedule equate unrelated axes. The exact source must be represented and the compiler must prove every accessed axis names the same authored `SourcedExtent` in the request's one environment.
6. **Verified — specialization remains forbidden.** `NormalizedPointwise::shape` remains `SourcedShape`, symbolic `elements` is zero rather than a representative size, and `first_symbolic_extent` never consults `ExtentSources::determined`. The bound-symbol fixture retains `n` even when the environment proves `n == 4`.
7. **Verified — unsupported populations remain separate.** Reductions, contractions, staged families, and the still-static structural relations continue through `static_shape` / `static_shape_ref` or fixed shape payloads. None can inherit a live-inner pointwise spelling without its own derivation.
8. **Verified — graph order is acyclic.** [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md) depends on this schedule carrier and cannot be made its prerequisite. The public schedule decision is therefore split ahead of this ticket; semantic-interface/artifact association remains after it.

Searchable reproductions:

```sh
rg -n 'A sourced broadcast must reach physical selection|IndexRegion requires a fixed geometry|first_symbolic_extent|carries_parametric_broadcast' crates/tiler-compiler/src/pipeline.rs crates/tiler-compiler/src/request.rs
rg -n 'pub struct IndexRegion|iteration_shape: Shape|iteration_shape: Option|pub enum LogicalAccess|fn live_input_extents' crates/tiler-ir/src/schedule/model.rs crates/tiler-ir/src/schedule/builder.rs
rg -n 'fn declare_plan_live_extents|fn emit_live_row_major|fn verify_input_extents|UnusedInputExtent|pub struct InputExtentParameter' crates/tiler-ir/src/kernel
rg -n 'fn symbolic_three_input_elementwise|fn a_symbolic_elementwise_neighbour_reaches_region_formation|a_compiled_plan_does_not_fold' crates/tiler-compiler/src/request.rs
```

## Public-boundary stop — 2026-08-14

The exact source/equality relation is a consequential public schedule boundary. [`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md) is the Pareto-complete topology packet and a hard dependency; its exact Rust field type remains blocked by [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](reconcile-input-ordinal-region-local-and-declared-input-semantics.md). Until that defect lands and Tom accepts an exact surface, this ticket authorizes no production change. The existing typed schedule refusal remains the correct fail-closed result.

## Exact-current-base re-audit — 2026-08-14, `a660ed618446ade55234993b835e75e26d44921c`

- **Verified — production evidence is unchanged.** `git diff --quiet 67fc9cac2a53f65fdba7619b9516c6e5e7324f20 a660ed618446ade55234993b835e75e26d44921c -- crates/tiler-ir/src/schedule/model.rs crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/kernel/model.rs crates/tiler-ir/src/kernel/lower.rs crates/tiler-ir/src/kernel/verify.rs crates/tiler-ir/src/program/builder.rs crates/tiler-compiler/src/physical.rs crates/tiler-compiler/src/pipeline.rs crates/tiler-compiler/src/request.rs crates/tiler-artifact/src/program/builder.rs` exits zero. Facts 1–3 and 5–8 above therefore still describe the executable boundary.
- **Verified — ticket custody changed.** Main commit `0ebb6879`, anchor `tickets: park symbolic schedule implementation`, changed this ticket to `status: blocked` and removed its assignee and lease. The rebase preserves that state; this ticket remains stopped behind public decisions.
- **Imprecise — Fact 4 needs an authority split.** `InputExtentParameter` does name a scheduled input axis, but `InputOrdinal`'s defining docs say it is dense, region-local, positional, and not an interface key. `TensorRole::Input` docs and physical compiler construction instead call the same value a declared program-input ordinal. Artifact construction follows the former model in practice: kernel buffer and stage-access position resolve the parameter to `MaterializedOrigin::ProgramInput { key }`. [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](reconcile-input-ordinal-region-local-and-declared-input-semantics.md) is the blocking P1 defect; no new live-source field may assign either authority until it resolves the contradiction.
- **False — the first packet's sole-dominance claim is withdrawn.** An additive explicit source variant overlaps existing implicit-self `LiveRowMajor` unless verification makes their populations disjoint. Complete replacement and a region-level binding are materially distinct options and must be compared. The repaired decision packet does so; the implementation purpose is unchanged, but no exact production spelling is authorized yet.

## Required work

- Re-audit `IndexRegion`, `ScheduledRegionBuilder`, `LiveRowMajor`, `pipeline.rs` `first_symbolic_extent` / `carries_parametric_broadcast`, and the frontend compile path at the exact base before editing.
- Form a scheduled region whose launch geometry names the program's symbols. Do not fold `ExtentSources::determined` into the logical plan and do not bake a bound value into plan or artifact identity.
- Do not change production until [`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md) is accepted. If accepted, implement only its source-bound rank-one live-inner slice; if rejected, retain the typed schedule refusal and defer.
- `IndexRegion.iteration_shape: Shape` need not change for that slice: the live inner dimension is outside the static outer domain. Any later sourced-geometry replacement is a separate broad public decision, not an implementation fallback here.
- Keep reductions, contractions, staged families, and structural maps refused by name until each has its own admitted geometry. Do not silently reuse the elementwise path.
- Leave Metal emission and the `deliver` identity-across-extents hash to [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## Required evidence

- The existing `sym n` `(a * b) + c` fixture that today declines at schedule now yields a scheduled region that still names `n`, and its literal neighbour still compiles with unchanged identity bytes.
- Removing the new path restores `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }`. Quote that failure text.
- A rewrite or formation step that would mint a launch over a determined representative extent fails as invalid compiler output.
- Perturb the new geometry independently of the parametric-broadcast exception so a missing broadcast cannot be the only way a symbol reaches a plan.
- Targeted compiler and IR tests, rustdoc, Clippy with warnings denied, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Lifting the frontend refuse again (already gone). Artifact-envelope rows. `N = 14` / `N = 15` pipeline evidence. Teaching `deliver` to embed and hash one artifact across bound extents — that remains the parent ticket after this lands.

## Closes when

`compile()` of the admitted same-shape symbolic elementwise population returns a scheduled region that names its symbols, or a narrower typed decline than `symbolic-extent` at schedule, without specializing on a bound value.

## Dependency correction — 2026-08-13

The former dependency on [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) was too broad. This schedule ticket needs the already-accepted `LiveRowMajor` / kernel live-operand spelling, not the later artifact/backend proof that has now been reopened. It therefore depends directly on [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md). This avoids a false cycle: [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md) must consume the schedule carrier produced here before a symbolic artifact interface can be validated.
