---
id: admit-symbolic-extents-through-schedule-formation
title: Admit symbolic extents through schedule formation
status: done
priority: p1
dependencies: [admit-symbolic-extents-through-compiler-region-formation, accept-the-live-extent-operand-public-surface, decide-the-source-bound-live-row-major-access-surface, refuse-mixed-pointwise-live-row-major-access-relations-before-lowering]
related: [deliver-an-artifact-family-from-a-symbolic-region, carry-live-extent-operands-through-the-artifact-envelope]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, implementation/artifact, implementation/build, implementation/metal, implementation/runtime, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, ir, shapes, public-boundary]
---
## User-visible outcome

`compile()` of a recognized same-shape symbolic elementwise program produces a
verified source-bound live schedule whose retained compiler request still names
the authored symbols, or declines with a typed reason that is not the current
schedule-geometry refuse. Shared schedule IR names the exact source access and
runtime input-axis operand, not `ShapeSymbol` or `ShapeEnv`. Specializing the
plan on a representative literal extent remains forbidden.

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

**Historical stop, partially cleared at the 2026-08-16 base.** The exact
source/equality relation remains a consequential public schedule boundary and
[`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md)
remains this ticket's hard dependency. The ordinal prerequisite is no longer a
blocker: [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](reconcile-input-ordinal-region-local-and-declared-input-semantics.md)
landed fieldless `TensorRole::Input`, public full-list `AccessOrdinal`, and the
compiler-private checked projection. Production remains stopped only until Tom
chooses or rejects the source-bound surface. The existing typed schedule refusal
remains the correct fail-closed result meanwhile.

## Exact-current-base re-audit — 2026-08-14, `a660ed618446ade55234993b835e75e26d44921c`

**Historical audit.** Its ordinal stop described that base and is superseded by
the current-base correction below; it must not be used as live authority.

- **Verified — production evidence is unchanged.** `git diff --quiet 67fc9cac2a53f65fdba7619b9516c6e5e7324f20 a660ed618446ade55234993b835e75e26d44921c -- crates/tiler-ir/src/schedule/model.rs crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/kernel/model.rs crates/tiler-ir/src/kernel/lower.rs crates/tiler-ir/src/kernel/verify.rs crates/tiler-ir/src/program/builder.rs crates/tiler-compiler/src/physical.rs crates/tiler-compiler/src/pipeline.rs crates/tiler-compiler/src/request.rs crates/tiler-artifact/src/program/builder.rs` exits zero. Facts 1–3 and 5–8 above therefore still describe the executable boundary.
- **Verified — ticket custody changed.** Main commit `0ebb6879`, anchor `tickets: park symbolic schedule implementation`, changed this ticket to `status: blocked` and removed its assignee and lease. The rebase preserves that state; this ticket remains stopped behind public decisions.
- **Imprecise — Fact 4 needs an authority split.** `InputExtentParameter` does name a scheduled input axis, but `InputOrdinal`'s defining docs say it is dense, region-local, positional, and not an interface key. `TensorRole::Input` docs and physical compiler construction instead call the same value a declared program-input ordinal. Artifact construction follows the former model in practice: kernel buffer and stage-access position resolve the parameter to `MaterializedOrigin::ProgramInput { key }`. [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](reconcile-input-ordinal-region-local-and-declared-input-semantics.md) is the blocking P1 defect; no new live-source field may assign either authority until it resolves the contradiction.
- **False — the first packet's sole-dominance claim is withdrawn.** An additive explicit source variant overlaps existing implicit-self `LiveRowMajor` unless verification makes their populations disjoint. Complete replacement and a region-level binding are materially distinct options and must be compared. The repaired decision packet does so; the implementation purpose is unchanged, but no exact production spelling is authorized yet.

## Current-base correction — 2026-08-16, `88c7c2181ac9a73de56598411915f176c50c3645`

- **False now — production evidence is not unchanged.** The AccessOrdinal
  reconciliation changed the schedule, kernel, compiler, program, and artifact
  paths named above. `AccessOrdinal` is the complete access-list coordinate;
  `InputExtentParameter { access, axis }` direct-indexes it; public
  `InputOrdinal` is gone; `VerifiedScheduledRegion::declared_input_at` projects
  to compiler-private `DeclaredInputOrdinal`; artifact construction follows the
  exact stage access to `MaterializedOrigin::ProgramInput { key }`.
- **Verified and more exact — the fixture's semantic source is not an arbitrary
  equal input.** All three inputs have exact structural `[program/0::n]`, and
  the decoded subject of retained `SemanticIdentity::shape_environment` binds `n` specifically to
  `BindingSource::InputDimension { input: InputKey("a"), axis: Axis(0) }`.
  Schedule construction must represent the access that projects to that root;
  it may not default to the first access or choose `b`/`c` merely because the
  shapes compare equal.
- **False current outcome wording repaired above — schedule IR does not name
  `n`.** The checked compiler wrapper retains the exact normalized request and
  `SemanticIdentity`, not the authored `ShapeEnv`/`ExtentSources` object. The
  compiler revalidates the identity's canonical environment bytes through
  public `decode_shape_env_subject`. Shared schedule IR carries only the source relation and the
  runtime input-axis operand, preserving ADR 0070's boundary and keeping the
  live value out of identity.
- **Verified after frontier repair — the decision dependency contains three
  nondominated exact surfaces and awaits Tom.** They are the contextual marker,
  total-local, and explicit-source/referenced-consumer form whose consumer
  carries only `source_access`. The earlier redundant-axis hybrid is dominated
  for this exact same-shape rank-one contract. No missing ordinal, interface, or
  mixed-map authority remains a prerequisite; this implementation stays blocked
  only on the public choice.
- **False at the former base — current mixed pointwise maps now fail closed.**
  The P0
  [`refuse-mixed-pointwise-live-row-major-access-relations-before-lowering`](refuse-mixed-pointwise-live-row-major-access-relations-before-lowering.md)
  landed at `f568467b` and closed at `48088dfb`. Current intrinsic verification
  requires all accesses to be static or every read and write to be live on the
  same axis. Its dependency edge remains truthful history, but the dependency is
  `done`; the accepted source implementation will replace that temporary broad
  refusal with the selected dedicated source rule.

## Exact-current-base correction — 2026-08-17, `a51305ce5b78628f9fbcbce78fd5cbbdfd43512e`

- **Verified — the implementation purpose and all production Facts are
  unchanged.** The exact live-row-major construction, verification, lowering,
  compiler binding, identity, and artifact owners are byte-identical to
  independently audited current main `e8141d7d`; this correction changes no
  production authority.
- **False — the 2026-08-16 frontier sentence retained a redundant consumer
  axis.** The exact three survivors are now the fieldless contextual marker
  `LiveRowMajorSource { inner_axis }` plus `LiveRowMajor`, total-local, and the
  referenced consumer. The former contextual `LiveRowMajor { inner_axis }` is
  dominated for the same reason as the redundant-axis referenced hybrid. The
  decision packet now recommends the fieldless marker, subject to independent
  review, and this implementation remains blocked until Tom accepts one exact
  surface.

Reproduce:

```sh
rg -n 'pub struct AccessOrdinal|pub enum TensorRole|pub struct InputExtentParameter|fn live_input_extents' crates/tiler-ir/src/schedule crates/tiler-ir/src/kernel
rg -n 'fn declared_input_at|fn derive_extent_operands|MaterializedOrigin::ProgramInput' crates/tiler-compiler/src/physical.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'fn request_environment|draft.bind\(&declared|fn symbolic_three_input_elementwise' crates/tiler-compiler/src/request.rs
rg -n 'pub fn decode_shape_env_subject|enum ShapeEnvSubjectError|fn shape_environment' crates/tiler-ir/src/shape/env/subject.rs crates/tiler-ir/src/semantic/identity.rs
```

## Required work

- Re-audit `IndexRegion`, `ScheduledRegionBuilder`, `LiveRowMajor`, `pipeline.rs` `first_symbolic_extent` / `carries_parametric_broadcast`, and the frontend compile path at the exact base before editing.
- Form a scheduled region with a static empty outer geometry and the accepted
  exact source relation. The verified compiler wrapper, not schedule geometry,
  retains the exact normalized symbolic shape and the environment's canonical
  identity bytes. Decode and revalidate those bytes to prove the root; do not
  retain or reconstruct an `ExtentSources` object, fold
  `ExtentSources::determined` into the logical plan, or bake a bound value into
  plan or artifact identity.
- Do not change production until [`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md) is accepted. If accepted, implement only its source-bound rank-one live-inner slice; if rejected, retain the typed schedule refusal and defer.
- Implement the selected packet's exact public
  `ScheduledRegionDiagnostic::LiveRowMajorSource` and
  `LiveRowMajorSourceRule` population: four rules for the recommended
  fieldless marker, seven for total-local, and six for the
  explicit-source/referenced-consumer replacement. For the recommended
  fieldless marker, retire old `0x09`, use fresh `0x0A` for
  `LiveRowMajorSource { inner_axis }` and fresh `0x0B` for unit
  `LiveRowMajor`, and apply marker count, marker role/mode, then complete access
  relation precedence. The consumer has no duplicate axis or source handle;
  `AxisMismatch` belongs only to total-local. The referenced survivor instead
  adds consumer-reference range before marker count and unique-marker reference
  consistency after complete coverage. Once one selected live relation
  appears, every pointwise read and the final write must carry that relation. The first
  access that does not is
  `ConsumerMissingRelation { access }` with stable rule
  `live-row-major-source-consumer-missing-relation`. Missing, multiple,
  out-of-range, inconsistent, and missing-consumer source relations do not
  collapse into `AccessContract` or `NumericalOrAccessRefinement`; semantic
  decode/root/shape mismatch remains compiler `request-binding`.
- For the recommended fieldless marker, thread the verified source axis or
  owning schedule relation into `kernel::lower::addressing`, which currently
  receives a detached `Access` plus `ReductionTopology`. Do not default a
  fieldless consumer axis. `kernel::builder::scheduled_access_rank` and
  `kernel::verify::access_rank` already receive the schedule, while request
  sizing declines this schedule-only relation and physical sizing uses the
  region element count; preserve those fail-closed/current owners.
- `IndexRegion.iteration_shape: Shape` need not change for that slice: the live inner dimension is outside the static outer domain. Any later sourced-geometry replacement is a separate broad public decision, not an implementation fallback here.
- Keep reductions, contractions, staged families, and structural maps refused by name until each has its own admitted geometry. Do not silently reuse the elementwise path.
- Leave Metal emission and the `deliver` identity-across-extents hash to [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## Required evidence

- The existing `sym n` `(a * b) + c` fixture that today declines at schedule now
  yields a verified source-bound schedule whose retained request still names
  `n`; decoding `semantic_identity.shape_environment().as_bytes()` proves its
  root is input `a`, axis 0, the schedule source projects there, and the literal
  neighbour still compiles with unchanged identity bytes.
- Truncated and bad-domain identity-subject bytes exercise the production decode
  mapping and fail as existing compiler `request-binding`, without a panic,
  empty-environment default, or fallback source.
- Perturb every selected intrinsic source rule independently and quote its exact
  `live-row-major-source-*` identifier and coordinate payload.
- For the recommended fieldless marker, prove the exact four-rule population
  and precedence, positively construct the public source and unit consumer from
  outside the crate, and compile-fail attempts to add either `inner_axis` or
  `source_access` to the consumer. No axis-mismatch or reference failure may be
  representable on that surface.
- With a valid source fixed, independently change one read and then the final
  write to `LinearIdentity`; both fail intrinsically as exact
  `ConsumerMissingRelation` coordinates before kernel lowering. Preserve an
  all-static neighbour and prove its exact identity bytes remain unchanged.
- Removing the new path restores `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }`. Quote that failure text.
- A rewrite or formation step that would mint a launch over a determined representative extent fails as invalid compiler output.
- Perturb the new geometry independently of the parametric-broadcast exception so a missing broadcast cannot be the only way a symbol reaches a plan.
- Targeted compiler and IR tests, rustdoc, Clippy with warnings denied, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Lifting the frontend refuse again (already gone). Artifact-envelope rows. `N = 14` / `N = 15` pipeline evidence. Teaching `deliver` to embed and hash one artifact across bound extents — that remains the parent ticket after this lands.

## Closes when

`compile()` of the admitted same-shape symbolic elementwise population returns a
verified source-bound schedule attached to the exact symbolic request, or a
narrower typed decline than `symbolic-extent` at schedule, without specializing
on a bound value or copying semantic symbols into shared schedule IR.

## Dependency correction — 2026-08-13

The former dependency on [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) was too broad. This schedule ticket needs the already-accepted `LiveRowMajor` / kernel live-operand spelling, not the later artifact/backend proof that has now been reopened. It therefore depends directly on [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md). This avoids a false cycle: [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md) must consume the schedule carrier produced here before a symbolic artifact interface can be validated.

## Forced scopes — 2026-08-18

Five scopes added as scheduling metadata required by the authorized work, not outcome expansion. The accepted complete replacement retires tag `0x09`, so every crate that hand-builds the live fixture had to move to the accepted spelling: `implementation/artifact` (`crates/tiler-artifact/src/program/tests.rs` fixture), `implementation/build` (`crates/tiler-build/src/metal_assembly.rs` fixture), `implementation/metal` (`crates/tiler-metal/src/tests.rs` fixture), `implementation/runtime` (`crates/tiler-runtime/tests/adapter_route/fixture.rs`). `implementation/frontend` covers the ticket-owned relay recording: the `deliver macos;` next-wall rendering (`crates/tiler-macros/src/aot.rs` + its test) and the byte-compared facade golden (`crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.stderr`).

## Exact-base Fact re-audit — 2026-08-18, `236753a3b4ae8fa42da0a67d7f4f4d3c9a864a48`

Worker `worker-symbolic-extents`, before any edit. Every file cited below was read in full at this base. The 2026-08-14 public-boundary STOP is resolved: Tom accepted the fieldless contextual marker on 2026-08-18 under [`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md) (its "Accepted decision — 2026-08-18" section is this implementation's exact spec).

1. **Verified with named drift — boundary and failure.** The schedule-stage gate stood at `crates/tiler-compiler/src/pipeline.rs`, anchors `A sourced broadcast must reach physical selection` and `IndexRegion requires a fixed geometry`, returning `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }`; `a_symbolic_elementwise_neighbour_reaches_region_formation` passed and observed it after region formation.
2. **Verified — frontend relay.** `crates/tiler-macros/src/aot.rs`, anchors `let batch = compile` and the `symbolic-extent` `rendered_refusal` arm; the `deliver macos;` consumer saw the compiler's typed decline.
3. **Verified — fixed schedule geometry suffices.** `IndexRegion::iteration_shape` public `Shape`; the rank-one live plan needs only the rank-zero static outer domain (product one, one work item), exactly as the packet's Fact 4 states.
4. **Verified — the old carrier and its dispersal.** `LogicalAccess::LiveRowMajor { inner_axis }` at tag `0x09`; `live_input_extents` derived one operand per live input read, `emit_live_row_major` consumed one `columns`, `verify_input_extents` rejected the rest as `UnusedInputExtent`. No compiler live construction existed.
5. **Verified — specialization forbidden.** `NormalizedPointwise::shape` stayed `SourcedShape`, symbolic `elements` zero, `first_symbolic_extent` never consulting `ExtentSources::determined` (anchor `Later schedule construction that needs a fixed`).
6. **Verified — the pre-announced drift is real and material.** The schedule domain is `tiler.schedule.v7` (`crates/tiler-ir/src/domains.rs`, anchor `tiler.schedule.v7`); `IndexRegion` carries the `RegionProgram` sum with `PartitionedCopy`; `push_numerical` writes the two elementary dimensions between signed-zero and the exceptional assumptions; the encoder is the re-read grammar, not the packet's `v6` base.
7. **Verified — tag reconciliation exactly as briefed.** At this base the access-map tag space wrote `0x01`–`0x09` and `0x0D` (partitioned-copy source, landed), with `0x0A`/`0x0B` reserved by the accepted live decision and `0x0C` reserved for the accepted gather surface — all recorded verbatim in the `TAG_PARTITIONED_COPY_SOURCE` doc, anchor `a gap is preferable to colliding reviewed identities`. No incoherence; the STOP condition did not fire.
8. **Rederived at v7 — the packet's no-version-step conclusion holds.** Every field before the access-map tag position is framed and self-delimiting under the v7 grammar, so `0x0A` and `0x0B` are bytes no earlier `v7` region wrote at that position: fresh injective tags, no reinterpretation, no schedule/kernel/explain domain step. Every live identity value moves (source: five bytes `0x0A`+axis; each consumer: one byte `0x0B`; the old spelling was five bytes `0x09`+axis per access); static values stay byte-identical. `map_schedule_build_error` projects the four new stable rules into `PhysicalError::Intrinsic`, appending explain reason vocabulary without a schema step.
9. **Unverified, marked.** The packet's Rust-layout probe values (`LogicalAccess` 208/8, diagnostic 12/4) were measured at its own base, before `VectorLaneBinding` and `PartitionedCopy` widened `ScheduledRegionDiagnostic`; they were not re-probed here and no delivered claim rests on them.

## Delivery record — 2026-08-18

Implemented the accepted fieldless contextual marker surface end to end through schedule formation; the population's `compile()` outcome is the Closes-when's second arm — a narrower typed decline past schedule — with the packaging remainder split into its own ticket.

**Shared IR (the accepted surface).**

- `LogicalAccess::LiveRowMajorSource { inner_axis }` (tag `0x0A`) and unit `LogicalAccess::LiveRowMajor` (tag `0x0B`) replace the retired contextual relation whole; `0x09` is permanently retired and documented as never-reused (`crates/tiler-ir/src/schedule/model.rs`). Compile-fail doctests (`E0559`) prove the consumer can acquire neither `inner_axis` nor `source_access` without a build error.
- `ScheduledRegionDiagnostic::LiveRowMajorSource { rule: LiveRowMajorSourceRule }` with exactly the four accepted rules and stable identifiers `live-row-major-source-{missing,multiple,not-input-read,consumer-missing-relation}`; census sized from the type (`variant_count == 4`) in `the_live_row_major_source_rule_census_is_exactly_the_accepted_four`.
- `verify_live_row_major_source` runs before the broad access-contract/refinement gates at the accepted precedence — marker count, marker role/mode, complete live-relation coverage — replacing `pointwise_accesses_choose_one_addressing_regime`; an all-static region has no source obligation. The landed mixed-map P0 closure is preserved under the dedicated rule: mixed static read and static write now fail as exact `ConsumerMissingRelation { access }` coordinates; the disagreeing-axis state is unrepresentable (compile-fail evidence above).
- `live_input_extents` derives the marker's one operand; crate-private `live_source_axis` is the checked-context derivation the fieldless consumer is interpreted through in `kernel::lower::addressing` (threaded, never defaulted), `access_rank`, and `scheduled_access_rank`.

**Compiler (real schedule formation).**

- The pipeline gate narrows: `crate::physical::admits_source_bound_live_schedule` admits exactly one whole-program rank-one single-symbol `f32` pointwise output whose decoded root is an input dimension realized by a dense read; everything else keeps `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }`.
- `live_pointwise_region` builds the accepted spelling: rank-zero outer, `work_items == 1`, marker on the exact root read with the decoded axis, fieldless consumers on every other read and the write, zero-linear-range live bounds proofs. The root arrives only through public `decode_shape_env_subject` over `semantic_identity().shape_environment().as_bytes()` (`decode_live_extent_root`); every decode failure, absent symbol, or non-`InputDimension` root is existing `PhysicalError::Intrinsic { rule: "request-binding" }`. No `ExtentSources` object is retained; no `ShapeSymbol`/`ShapeEnvIdentity` enters shared schedule IR.
- Request binding gained the independent live arm (`live_pointwise_binding_matches`): exact source position from the decoded root, consumer/write coverage, rank-zero domain — a forged marker position and a `[4]`-specialized launch against the symbolic subject both fail as `request-binding`.
- Occurrence refinement now realizes the population: `GovernedPointwise` and the law-side `emit_pointwise`/`realize_pointwise` emit sourced dimensions and sourced tensors (the existing crate-internal path the parametric broadcast and source-bearing slice already used), and both environment gates widened symmetrically by the same boundary-names-a-symbol condition (`subject_boundaries_name_a_symbol` in `tiler_ir::index::law`; mirrored in `capability.rs::occurrence_needs_shape_environment`) — a static neighbour in an environment-carrying program keeps the environment-free builder and its identity.
- `published_shape` is now `Option` (fail-closed for symbolic domains at its two callers).

**The boundary decision and the next wall.** With schedule formation, binding, feasibility, selection, and kernel lowering all real for the population, `compile()` declines at program packaging: `CoverAssembly::from_plan`'s `named-output-symbolic` (typed `UnsupportedCapability { phase: "program-assembly" }`), behind which the shared kernel-program builder's `SymbolicInterfaceExtent` states the identity property "a symbolic program … cannot ship with its shape-environment subject unrepresented in the artifact's three carried subjects". Lifting that is an identity consequence beyond retire-0x09-plus-append and squarely the envelope/delivery chain's scope, so it was not lifted as an implementation fallback; the remainder is [`package-the-admitted-live-schedule-into-a-symbolic-kernel-program`](package-the-admitted-live-schedule-into-a-symbolic-kernel-program.md) (new, p1), now a dependency of [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md). The `deliver macos;` relay's next true wall, recorded and rendered: `CompileFailureClass::UnsupportedCapability { rule: "named-output-symbolic" }` — "the compiler formed and verified a live schedule over this region's symbolic extent and cannot yet package it" (`rendered_refusal` arm added; trybuild golden `deliver_selects_an_artifact_family.stderr` updated; `a_symbolic_region_reaches_the_compilers_typed_decline` asserts the retired schedule-geometry text is gone).

**Moved-pin enumeration (retire-0x09 identity movement).** The repository's only pinned live identity values are `crates/tiler-ir/src/kernel/tests.rs::{LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX, LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX}`; both rebaselined, with the exact prior values retained as `RETIRED_CONTEXTUAL_LIVE_ROW_MAJOR_{SCHEDULE,KERNEL}_IDENTITY_HEX` and inequality asserted in the pin test, so the movement is a checked fact. The read's `0109…` run became `010a…` and the write's `0209…` became the bare `020b`, shrinking the fixture's schedule bytes by four; the kernel frame length moved `0x0184 → 0x0180`. Static pins are byte-identical and still asserted: `ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX`, `STRICT_F32_REGION_IDENTITY_HEX` (+`_V6`), `ONE_COMMITTER_COOPERATIVE_IDENTITY_HEX`. The unpinned live fixtures in `tiler-metal/src/tests.rs`, `tiler-build/src/metal_assembly.rs`, `tiler-artifact/src/program/tests.rs`, and `tiler-runtime/tests/adapter_route/fixture.rs` moved to the new spelling and stay green.

## Perturbation evidence — 2026-08-18 (subject perturbations, quoted)

- **Each of the four rules reached independently** (fixture maps perturbed, never assertions): no marker → `LiveRowMajorSource { rule: Missing }` / `live-row-major-source-missing`; two markers → `Multiple { first: AccessOrdinal(0), second: AccessOrdinal(1) }`; marker on the owning write → `SourceNotInputRead { source: AccessOrdinal(2) }`; marker on an intermediate read → `SourceNotInputRead { source: AccessOrdinal(0) }`; static read inside the live loop (both widths) → `ConsumerMissingRelation { access: AccessOrdinal(0) }`; static write → `ConsumerMissingRelation { access: AccessOrdinal(2) }`; and marker-count-before-coverage precedence held (`the_marker_count_rule_precedes_the_coverage_rule`).
- **Removing the new path restores the old refusal.** Reverting the gate to the unconditional condition made `the_admitted_symbolic_population_declines_at_program_assembly_not_schedule` fail with, quoted: `got compile.schedule.symbolic-extent: program/0::n is a symbolic extent this capability cannot plan over`. Restored.
- **Consumer tag reused as `0x0A`** (production collision with the source): the pin test failed quoting the write map bytes `020a000000` where `020b000000` is pinned — `the source-bound all-live schedule bytes must match their rebaselined pin`. Restored.
- **Consumer tag restored to retired `0x09`**: the pin test failed quoting `0209000000` at the write — a fieldless consumer under `0x09` is byte-indistinguishable from the retired axis-carrying relation's leading byte, which is why `0x0B` is load-bearing. Restored.
- **Forged marker position** (marker on `b` where the decoded root is `a[0]`) and a **`[4]`-specialized launch** against the symbolic subject both fail, quoted: `schedule.intrinsic.request-binding: region 0 rejected`.
- **Truncated, bad-domain, and absent-symbol identity bytes** through the production decoder each fail, quoted: `schedule.intrinsic.request-binding: region 0 rejected` (`corrupted_identity_subject_bytes_fail_as_request_binding`).

## Commands — 2026-08-18

From this worktree (results recorded at the pre-commit tree):

- `cargo nextest run --workspace` — 3787 passed, 8 skipped.
- `cargo test --workspace --doc` — all doc-tests pass, including the two new `E0559` compile-fail proofs on `LogicalAccess::LiveRowMajor`.
- `cargo clippy --workspace --all-targets --locked --exclude tiler-prototype-run --exclude tiler-prototype-compile --exclude tiler-prototype-candle -- -D warnings` — clean (the `lint` target's own scope; prototype lint findings predate this branch and are excluded by the Makefile's stated policy).
- `cargo fmt --check` — clean.
- Key single tests: `the_admitted_symbolic_population_forms_a_verified_source_bound_live_schedule`, `the_source_marker_follows_the_environment_root_not_the_first_input`, `a_forged_source_marker_position_fails_request_binding`, `a_specialized_representative_launch_fails_request_binding`, `corrupted_identity_subject_bytes_fail_as_request_binding`, `unsupported_symbolic_populations_keep_the_named_schedule_refusal`, `a_proved_equal_symbol_does_not_widen_the_admitted_population`, `a_compiled_plan_does_not_fold_a_bound_extent_value`, `the_admitted_symbolic_population_declines_at_program_assembly_not_schedule`, `a_symbolic_region_reaches_the_compilers_typed_decline`, `static_and_same_axis_live_pointwise_identities_remain_exact` — all green.
- `make full` at the code tree of the delivery commit — green end to end: citations (1166 pinned citations resolved), fmt, check, clippy `-D warnings`, workspace nextest 3787 passed / 8 skipped, workspace doc-tests all ok, rustdoc `-D warnings` with and without `--document-private-items`, release numerical suite 1288 passed / 3 skipped, `tkt lint`, shellcheck. Only output flagged: Cargo's pre-existing future-incompat note for third-party `block v0.1.6` (informational, not gated). This ticket-record commit is a `tickets/`-only delta over that gated code tree, carrying the gate per the repository delta rule; `tkt lint` and `make citations` rerun after the edit.

## Unsupported populations (counted, refused by name)

Reductions, contractions, staged families, and structural maps keep their existing recognition-level refusals; within recognized symbolic pointwise, the schedule-stage `symbolic-extent` refusal remains for: rank above one or mixed static/symbolic axes; a non-`f32` width; several outputs; a static, interface-parameter, or target-property root; a differently spelled or proved-equal symbol (refused earlier, at `strategy.elementwise-shape`); and a root input the region never reads densely. The admitted population — whole-program same-shape rank-one single-symbol `f32` pointwise with a dense-read `InputDimension` root — forms the verified live schedule and declines only at packaging.
