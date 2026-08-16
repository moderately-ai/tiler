---
id: decide-the-source-bound-live-row-major-access-surface
title: Decide the source-bound LiveRowMajor access surface
status: in-progress
priority: p1
dependencies: [accept-the-live-extent-operand-public-surface, reconcile-input-ordinal-region-local-and-declared-input-semantics]
related: [admit-symbolic-extents-through-schedule-formation, associate-live-extent-operands-with-symbolic-semantic-interface-axes, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, schedule, shapes, identity]
claimed_from: todo
assignee: worker-source-bound-live-row-major
lease_expires_at: 1786915334
---
## User-visible outcome

After the input-ordinal authority defect is resolved, Tom selects or rejects the exact public schedule relation that lets three same-symbol rank-one inputs share one live extent operand without specializing `n`, inventing an interface authority, or giving one computation duplicate schedule identities. Rejection is typed deferral: the existing schedule-stage `UnsupportedSymbolicExtent` remains fail-closed.

## Decision-readiness stop

This packet compares every material topology, but it must not ask Tom to accept a Rust field type yet. [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](reconcile-input-ordinal-region-local-and-declared-input-semantics.md) must first decide whether the source field names a dense region-local input, a separate declared-input association, or two distinct types joined at stage binding. A bare `InputOrdinal` cannot be described as a program/interface ordinal: its defining docs explicitly deny that authority.

## Exact-current-base Facts — 2026-08-14, `a660ed618446ade55234993b835e75e26d44921c`

- **Fact — source evidence did not move.** The relevant IR/compiler/artifact blobs are byte-identical to the original implementation base `67fc9cac2a53f65fdba7619b9516c6e5e7324f20`. The current-base ticket state did move: main commit `0ebb6879`, anchor `park symbolic schedule implementation`, made the implementation ticket blocked and removed its claim.
- **Fact — the required population is exact.** `request.rs`, anchor `` `(a * b) + c` over three rank-one `f32` inputs ``, constructs three distinct declared inputs whose shapes contain the same `SourcedExtent::Symbol(n)`. Recognition retains exact `SourcedShape` equality; it does not merely prove three runtime values happen to agree.
- **Fact — the current schedule has no multi-input common-source relation.** On an input read, `LogicalAccess::LiveRowMajor { inner_axis }` makes `live_input_extents` derive that input axis as a source. The accepted one-source corpus also uses the same map on an `Intermediate` write, where it addresses the live loop but derives no second extent. Three live input reads therefore declare three `InputExtentParameter`s; lowering consumes one as `columns`, and kernel verification rejects the other two as `UnusedInputExtent`.
- **Fact — rank one does not require sourced `IndexRegion` geometry.** The accepted live-row-major meaning is a static outer product plus a live inner serial loop. Rank one has an empty static outer domain, one work item, and a zero-trip live loop when `n == 0`. `IndexRegion::iteration_shape: Shape`, `KernelSchedule::work_items`, and `LaunchPlan` can remain static for this slice.
- **Fact — ordinal authority is contradictory.** `schedule/handles.rs`, anchor `region-local and positional`, defines `InputOrdinal` as a dense region-local position, not an interface key. `schedule/model.rs`, anchor `Which declared input tensor this access binds`, and `physical.rs`, anchor `The recognized ordinal, not the first declared input`, instead assign declared program-input meaning and admit sparse ordinals. This is a live prerequisite, not wording this packet may choose around.
- **Fact — artifact mapping does not resolve the ordinal against the interface.** `InputExtentParameter` names a region-local input and axis. `derive_extent_operands`, anchor `maps that tensor through the stage access`, matches the kernel buffer role, follows the corresponding stage access to its materialized value, and obtains `MaterializedOrigin::ProgramInput { key }`. That checked `InputKey` is the artifact/interface authority.
- **Fact — compiler request binding owns semantic equality.** `physical.rs`, anchors `verify_region_subject_binding` and `elementwise_reads_match`, binds a verified schedule to the recognized request. The schedule carries no `ShapeEnvIdentity`; exact symbol/environment agreement must be proved here against request/evidence state, never inferred from axis numbers or runtime values.
- **Fact — additive explicit-self is not canonical.** Existing `LiveRowMajor` already spells “use this input axis as its own live bound.” A second variant that permits the same source pair spells that access twice and gives one computation two schedule identities. Any additive survivor must make implicit-self and explicit-other populations disjoint.
- **Fact — the existing output spelling is part of the identity control.** `kernel/tests.rs::live_row_major_region`, `metal/tests.rs::live_row_major_kernel`, and `build/metal_assembly.rs::live_row_major_unit` all pair one input-read `LiveRowMajor` with one `Intermediate`-write `LiveRowMajor`. Claiming the additive option preserves the corpus is true only if both constructions and their schedule/kernel bytes remain unchanged.
- **Fact — downstream artifact association remains separate.** [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md) must bind the selected schedule source through stage access to the symbolic semantic interface. It remains downstream of schedule formation; making it a prerequisite here would create a cycle.

Reproduce:

```sh
git diff --quiet 67fc9cac2a53f65fdba7619b9516c6e5e7324f20 a660ed618446ade55234993b835e75e26d44921c -- crates/tiler-ir/src/schedule/model.rs crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/kernel/model.rs crates/tiler-ir/src/kernel/lower.rs crates/tiler-ir/src/kernel/verify.rs crates/tiler-ir/src/program/builder.rs crates/tiler-compiler/src/physical.rs crates/tiler-compiler/src/pipeline.rs crates/tiler-compiler/src/request.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'region-local and positional|not an interface key|Which declared input tensor this access binds|The recognized ordinal, not the first declared input' crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/schedule/model.rs crates/tiler-compiler/src/physical.rs
rg -n 'fn live_input_extents|fn declare_plan_live_extents|fn emit_live_row_major|fn verify_input_extents|UnusedInputExtent' crates/tiler-ir/src/schedule/model.rs crates/tiler-ir/src/kernel
rg -n 'maps that tensor through the stage access|MaterializedOrigin::ProgramInput|fn verify_region_subject_binding|fn elementwise_reads_match' crates/tiler-artifact/src/program/builder.rs crates/tiler-compiler/src/physical.rs
cargo test -p tiler-compiler request::tests::a_symbolic_elementwise_neighbour_reaches_region_formation -- --exact --nocapture
```

## Invariants every correct answer preserves

- No `ExtentSources::determined` value becomes schedule geometry, work, cost, or identity. The authored symbol remains semantic request state; the runtime extent is an operand only.
- One checked region source supplies the live bound. Every read and write that consumes it is proved against the same exact `SourcedExtent` in the request's exact `ShapeEnv`; matching rank, axis, ordinal, or runtime value is insufficient.
- Schedule intrinsic verification proves its structural source relation. Compiler request binding separately proves that relation realizes the recognized semantic program. Program/artifact construction maps the region-local source through a checked stage access to `InputKey`.
- A hand-built schedule cannot alias unrelated axes by ordering them. No accepted input-read form has both implicit-self and explicit-self spellings, and the retained `LiveRowMajor` output/write map remains the sole output spelling under the additive option.
- `n == 0` launches one static outer invocation and executes a zero-trip inner loop, performing zero loads and stores. No non-emptiness assumption is introduced.
- The literal neighbour retains its old `LinearIdentity` bytes and all downstream identities. A symbolic bound value is absent from schedule, kernel, plan, artifact, and cache identities.
- Reductions, contractions, staged families, structural maps, materializing dynamic intermediates, multi-symbol shapes, rank above one, and output-derived/data-dependent extents remain refused by their existing owning boundaries.

`LiveExtentSource` below is a semantic placeholder, not an accepted Rust type. The prerequisite must replace it with a type whose authority is coherent before this packet reaches Tom.

## Option 1 — completely replace `LiveRowMajor` with a source-bearing relation: survivor

**Candidate topology.** Supersede the accepted variant with one total relation such as `LiveRowMajor { inner_axis, extent_source: LiveExtentSource, extent_axis: Axis }`. Every live-row-major read or write carries the explicit source; the source access itself is explicit too. The existing payload-less variant is removed, so implicit-self and explicit-self cannot coexist.

**Correctness and strictness.** The source is local to every access and total matches force every consumer to interpret it. Intrinsic verification requires all maps in the admitted region to carry the same valid input-axis source; compiler binding proves every accessed symbolic axis exactly equals it. There is no fallback to an access's own tensor.

**Maintenance and compatibility.** One relation is conceptually uniform and leaves no contextual lookup, but repeats identical source fields across every access. This is a complete pre-production replacement of an already accepted public variant, so Tom must explicitly supersede that surface. Every constructor, verifier, encoder, lowering, test, and downstream provider must migrate atomically.

**Host/runtime and identity.** Verification and lowering remain linear in access count; memory and canonical bytes grow by fixed source fields per live access, with no search-population growth. Every existing hand-built live-row-major schedule changes public construction and schedule bytes; bumping the schedule identity domain is the conservative migration because an existing tag's payload changes. Static schedules remain semantically untouched. A one-source control may retain identical kernel `InputExtentParameter` bytes, but that must be recomputed rather than inferred. No artifact schema is selected here.

**Strongest counterargument and reversal evidence.** Repeating one domain source on every access is denormalized and creates more inconsistent forged states to reject. Prefer this option over the others if a consumer audit shows access maps must remain self-contained or region-level context would make equality, identity, or lowering nonlocal and error-prone.

## Option 2 — one required region-level live source consumed by `LiveRowMajor`: survivor

**Candidate topology.** Add one source binding to the region/index schedule subject, for example `IndexRegion::live_extent_source: Option<LiveExtentSource>`, and retain `LiveRowMajor { inner_axis }` as the per-tensor addressing relation. Every verified region containing `LiveRowMajor` must carry exactly one binding; `None` with a live map is invalid. All live maps consume that source, including the write. Thus there is no optional implicit-self spelling inside the verified population.

**Correctness and strictness.** Intrinsic verification proves exactly one valid input-axis source and proves every live map belongs to the one-source domain. Compiler binding proves every mapped axis equals that request source. A future region needing two independent live domains declines until the binding becomes an explicit list; it cannot silently reuse the first.

**Maintenance and compatibility.** The source is stated once and matches the actual one-domain rank-one execution model, reducing repeated fields and inconsistent-map states. The cost is contextual semantics: interpreting `LiveRowMajor` now requires the containing region, and adding a public field changes every `IndexRegion` struct literal. It also centralizes only the current single-source case; a future multi-source region needs a separately designed association rather than widening this field casually.

**Host/runtime and identity.** One fixed record per live region is smaller than per-access repetition and adds constant verification work. Static regions with `None` may retain byte identity if absence is encoded as no appended record and injectivity is reproved. Every formerly valid live region must add the binding and receive new schedule bytes; whether that requires a domain step or a framed appended section is an identity decision for implementation. Derived kernel operands can remain one region-local source. No artifact schema is selected here.

**Strongest counterargument and reversal evidence.** A logical access relation that cannot be interpreted alone weakens locality, and one field may be a dead-end for multiple live domains. Eliminate this option if a complete access-consumer audit requires map-local total meaning, or if the next admitted population already needs independently sourced live axes in one region.

## Option 3 — additive explicit-other input-read relation with disjoint canonical populations: survivor

**Candidate topology.** Keep accepted `LiveRowMajor { inner_axis }` for the one input read whose own axis supplies the source and for the region's existing output/write addressing. Add a source-bearing relation only for a **non-source `TensorRole::Input` read**. It is invalid on `Output`, `Intermediate`, a write, or the source input itself. In the three-input subject, the canonical source read uses existing implicit-self `LiveRowMajor`, the other two input reads use the explicit-other relation, and the output write retains existing `LiveRowMajor`. Intrinsic verification requires exactly one implicit-self input source, requires every explicit-other input read to name it, and treats the old write map as the sole canonical output spelling.

This corrects both retired additive proposals. A `SourceBoundLiveRowMajor` that permits the source input to carry its own explicit source duplicates the existing input meaning. One that permits output/write construction duplicates the accepted output map and either moves the existing one-source corpus or gives the same output two identities. Neither is a survivor.

**Correctness and strictness.** Each input read has exactly one spelling: self-source or other-source; every output/write retains exactly one spelling. Intrinsic verification proves the structural star around one source, the old write belongs to that one live loop, and the new form appears only on non-source reads. Compiler binding proves the source, all reads, and the result share the exact request symbol/environment. Missing source, two self sources, explicit-self, explicit output/write, inconsistent explicit sources, or source not read all fail closed.

**Maintenance and compatibility.** Existing public construction and schedule bytes remain intact, while the new population uses a second explicit relation. This avoids migration but leaves two variants and a mixed-map canonicality rule that every exhaustive internal consumer must implement. The split is semantic rather than cosmetic—self versus other—but is more complex than one total replacement.

**Host/runtime and identity.** Work is linear in accesses and fixed-size fields occur only on non-source input reads. Existing live and static schedule/kernel identities remain byte-identical, including the accepted one-source input-read plus `Intermediate`-write corpus. The new symbolic population receives new schedule bytes under an unused framed tag; lowering derives one source operand after deduplication. No artifact schema or interface ordinal is introduced.

**Strongest counterargument and reversal evidence.** The mixed star relation is harder to explain and verify than one total source field, and preserving the old write keeps output addressing contextual on the region's unique input source. Eliminate it if perturbation shows a consumer can observe the explicit reads or old write without the unique implicit source, or if maintaining two read variants costs more than the pre-production identity migration avoided by it.

## Option 4 — typed deferral: survivor

Retain `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent", extent: n }` and make no public or production change. It is correct and fail-closed, preserves every identity and host cost, and keeps every symbolic schedule unsupported. Its cost is capability: the already recognized three-input `[n]` program cannot compile and delivery remains blocked.

**Strongest counterargument and reversal evidence.** Deferral leaves a known supported-frontier gap and strands downstream symbolic delivery. It ceases to be attractive when Tom values that slice more than avoiding a public schedule source relation, or when one of Options 1–3 is accepted with complete negative and identity evidence.

## Eliminated options

- **Broad sourced `IndexRegion`/launch replacement.** Replacing fixed geometry, work counts, feasibility, cost, frontier handoff, kernel launch, and program assembly is correct only as a broad atomic redesign. Rank-one live-inner needs none of it, so it is worse than all three source-binding survivors on surface, migration, maintenance, and host risk without improving this population.
- **Implicit first-input collapse.** Collapsing live operands onto the first read without representing and verifying a source lets a hand-built schedule alias unrelated axes and turns ordering into authority. Incorrect; eliminated.
- **Symbol/environment copied into schedule maps.** Carrying `ShapeSymbol` and `ShapeEnvIdentity` in structural schedule IR duplicates semantic identity and still needs compiler request binding. The three source-binding survivors obtain the proof from the owning compiler layer with less identity coupling. Dominated.
- **Additive explicit-self plus implicit-self.** Two variants encode the same self-source access under different bytes. It violates canonical identity before any runtime question. Eliminated.

## Pareto frontier

All four survivors are top-tier on correctness and fail-closed strictness after the ordinal prerequisite. They differ materially:

| Survivor | Maintenance / compatibility | Host runtime / memory | Identity and public consequences |
| --- | --- | --- | --- |
| Complete per-access replacement | One total local relation; repeated fields; atomic public migration | Linear checks; fixed fields per live access | Existing live schedules migrate; static subjects unchanged; likely schedule-domain step |
| Required region-level binding | One source record; contextual map meaning; single-domain ceiling explicit | Constant source record plus linear checks; smallest retained state | Public `IndexRegion` construction changes; existing live schedules migrate; static bytes may remain |
| Additive disjoint input reads | Preserves existing read/write surface; two read variants and star canonicality | Linear checks; fields only on non-source input reads | Exact one-source input-read + `Intermediate`-write bytes unchanged; new tag/population only |
| Typed deferral | No maintenance or capability change | No change | No public/schema/identity change; symbolic schedule remains unsupported |

No implementation survivor dominates the others: complete replacement favors one total local relation, region binding favors normalized state, and additive disjoint spelling favors compatibility/identity continuity. Deferral remains incomparable because it adds no surface and no capability. After the ordinal prerequisite names the exact source type, Tom receives one question among these four—not the earlier false claim that the additive spelling is uniquely dominant.

## Required evidence and subject perturbations

- Reproduce the three-input `(a * b) + c` refusal first. After acceptance, prove one rank-zero-outer region, one live source, four consuming maps, exact retained `n`, and exactly one `InputExtentParameter`. Under Option 3 the exact map population is one old source-input read, two new explicit-other input reads, and one old output write.
- Remove the accepted source spelling/binding and quote the restored schedule-stage `symbolic-extent` refusal.
- Forge source position/type, source axis, missing source read, two sources, inconsistent consumers, explicit-self where forbidden, and a determined-literal plan. Quote intrinsic schedule or invalid-compiler-output diagnostics from the owning layer.
- **Environment negative:** keep the same schedule proposal, then perturb the compiler request/evidence binding to a different `ShapeEnvIdentity` or change one accessed `SourcedExtent` to another symbol. The schedule has no environment field to mutate; `verify_region_subject_binding` must reject the reused proposal as invalid compiler output.
- Reorder stage accesses, retain kernel roles, and prove program/artifact binding does not silently map the extent to a different `InputKey`. Repeat with a sparse declared-input subset after the ordinal prerequisite lands.
- Perturb geometry while retaining parametric broadcast, then perturb broadcast while retaining live-source geometry, proving those paths are independent.
- Exercise `n == 0`, `n == 1`, and a larger bound: one outer launch, zero/one/many inner iterations, identical schedule and kernel identities across bound values.
- Recompute literal-neighbour schedule/kernel/program/artifact bytes. Before implementation, retain the exact canonical schedule and kernel bytes plus SHA-256 for the existing one-source controls `kernel/tests.rs::live_row_major_region`, `metal/tests.rs::live_row_major_kernel`, and `build/metal_assembly.rs::live_row_major_unit`, each of which is exactly one input-read `LiveRowMajor` plus one `Intermediate`-write `LiveRowMajor`. Option 3 must reproduce those bytes exactly; Options 1 and 2 must record and govern their migration instead. Perturb only the input read, then only the write, so a control that accidentally drops either access fails.
- Count the admitted population: whole-program same-shape pointwise output, rank one, at least one read, exact shared symbol/environment, one live source, no materialization. Each excluded family receives a named refusal.

## Graph and closing conditions

Graph order is [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](reconcile-input-ordinal-region-local-and-declared-input-semantics.md) → this decision → [`admit-symbolic-extents-through-schedule-formation`](admit-symbolic-extents-through-schedule-formation.md) → [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md). The implementation remains blocked; artifact/source association cannot be mistaken for schedule authority, and there is no cycle.

After the prerequisite lands, replace every `LiveExtentSource` placeholder with the exact derived type, rerun this comparison, and present only the still-nondominated surfaces to Tom. Only Tom accepts a surface or chooses deferral. Do not close this decision on worker recommendation, and do not let its closure authorize artifact schema or unsupported operation families.
