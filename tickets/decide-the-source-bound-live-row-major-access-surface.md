---
id: decide-the-source-bound-live-row-major-access-surface
title: Decide the source-bound LiveRowMajor access surface
status: in-progress
priority: p1
dependencies: [accept-the-live-extent-operand-public-surface, reconcile-input-ordinal-region-local-and-declared-input-semantics]
related: [admit-symbolic-extents-through-schedule-formation, associate-live-extent-operands-with-symbolic-semantic-interface-axes, deliver-an-artifact-family-from-a-symbolic-region, repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation, refuse-mixed-pointwise-live-row-major-access-relations-before-lowering]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, schedule, shapes, identity]
claimed_from: todo
assignee: worker-live-row-major-packet
lease_expires_at: 1786966097
---
## User-visible outcome

Tom chooses among the three exact public schedule relations that can make the
recognized three-input rank-one `(a * b) + c` program consume the one live
extent which its `ShapeEnv` binds to input `a`, axis 0. All three choices leave the
normalized symbol and the environment's canonical identity bytes in
compiler-owned semantic state, carry only the runtime input-axis source into
schedule/kernel construction, and keep every unsupported symbolic population
fail-closed. Rejecting all three retains the current schedule-stage
`UnsupportedSymbolicExtent` refusal.

## Exact-base Fact audit — 2026-08-16, `88c7c2181ac9a73de56598411915f176c50c3645`

This supersedes the 2026-08-14 packet completely. No production or decision
document was edited before this audit.

1. **Verified historical correction — the source evidence did move.** The old
   packet said the relevant paths were byte-identical to `67fc9cac`; the accepted AccessOrdinal
   reconciliation and later work changed all ten named files by 867 insertions
   and 511 deletions between old packet base `a660ed61` and this base. Current
   source, not that historical comparison, governs this decision.
2. **Verified after the earlier precision repair — one authored shape and root
   survive as checked identity bytes, not as the authored environment object.**
   `request.rs`, anchors
   `three_input_elementwise_with` and `` `(a * b) + c` ``, creates distinct
   `InputKey`s `a`, `b`, and `c`, each with
   the same structural `SourcedShape([SourcedExtent::Symbol(program/0::n)])`.
   `plan_elementwise`, anchor `sourced_shape_ref(program, value) != Some(shape)`,
   compares exact `SourcedShape`; it does not admit differently spelled symbols
   merely because `ExtentSources::proves_equal` can prove their values equal.
   `request_environment`, anchor `draft.bind(&declared,
   request_axis_binding("a", 0))`, gives `n` exactly one root
   `BindingSource::InputDimension { input: "a", axis: 0 }`. The pointer test at
   anchor `the request must carry the program's own environment` proves only the
   pre-verification `CompilationRequest` borrows the program's exact `ShapeEnv`.
   `VerifiedTargetRequest` and `VerifiedRequestSubject` instead retain
   `NormalizedProgram`/`NormalizedProgramSubject` plus `SemanticIdentity`, not
   `Arc<ShapeEnv>` or `ExtentSources`. `SemanticIdentity::shape_environment`
   exposes a `ShapeEnvIdentity`; its canonical bytes are the complete root and
   constraint subject which public `decode_shape_env_subject` revalidates.
3. **Verified — no shared live source is
   represented today.** `live_input_extents` emits one
   `(AccessOrdinal, Axis)` for every input read whose map is `LiveRowMajor`.
   Three live reads therefore produce accesses 0, 1, and 2. Kernel lowering's
   `emit_live_row_major` chooses one matching operand as `columns`, while
   `verify_input_extents` requires the complete derived list and rejects the two
   unused operands as `UnusedInputExtent`. No compiler live-row-major
   construction exists at this base.
4. **Verified — rank one needs no sourced schedule
   geometry.** The accepted relation is a static outer product plus a live inner
   loop. For rank one the outer `Shape` is empty, its product is one, and
   `KernelSchedule::{work_items, launch.grid_threads}` can both be one. At
   `n == 0` that one invocation enters a zero-trip loop; all element loads and
   stores remain inside the loop, so it executes none. “One launch” means one
   static outer invocation, not a nonempty live domain.
5. **Verified after the former prerequisite was retired — ordinal authority is
   resolved.** Current `TensorRole::Input` is fieldless. `AccessOrdinal`, anchor `The exact position
   in a scheduled region's complete ordered access list`, is the sole shared
   local coordinate, including intermediate reads and the final write. Public
   `InputOrdinal` no longer exists; retained declared association uses
   compiler-private `DeclaredInputOrdinal`.
6. **Verified after the earlier precision repair — artifact mapping resolves
   the exact coordinate.**
   `InputExtentParameter { access: AccessOrdinal, axis: Axis }` directly indexes
   the complete schedule access and corresponding kernel-buffer list.
   `derive_extent_operands`, anchor `maps that position through the matching
   stage access`, follows that exact position to
   `MaterializedOrigin::ProgramInput { key }` and emits the existing artifact
   row `(InputKey, Axis, AbiType)`. It neither filters inputs nor searches for a
   first matching role.
7. **Verified after the earlier precision repair — compiler request binding
   owns the proof through two retained subjects.**
   `VerifiedScheduledRegion::declared_input_at`, anchor
   `Projects one local input access back to the declared program interface`,
   uses the already-verified request subject to project `AccessOrdinal` to
   private `DeclaredInputOrdinal`; `CoverAssembly` consumes it. The schedule
   carries no `ShapeSymbol` or `ShapeEnvIdentity`. The compiler must compare the
   exact `NormalizedPointwise::shape`, then decode
   `request.semantic_identity().shape_environment().as_bytes()` with public
   `decode_shape_env_subject` and match its exact root binding. Current
   `elementwise_reads_match` compares the recognized static access maps only;
   the implementation must add that source-bound live realization check.
   `verify_schedule_with_feasibility` already holds the `VerifiedTargetRequest`;
   the implementation can decode there and pass the decoded environment subject
   into compiler binding before storing the checked `VerifiedRequestSubject`
   clone, so no second retained association or new public accessor is needed. Any
   `ShapeEnvSubjectError`, absent symbol/root, non-`InputDimension` root, or
   source/key/axis mismatch fails under existing compiler rule
   `request-binding`; it never defaults an environment or source.
8. **Imprecise classification — duplicate explicit-self is an inference, not a
   source Fact.** It is nevertheless a binding canonicality requirement: a
   source access cannot have both an implicit-self and an explicit-self
   spelling. Nor may the compiler select `b` or `c` merely because their shapes
   equal `[n]`; this fixture's root source is `a[0]`, and another valid source
   denotes a different runtime-fact authority.
9. **False at this base — multi-input construction and exact identity pins have
   landed.** `kernel/tests.rs`, anchor `fn two_input_pointwise_builder`, now
   constructs two reads plus one write for the mixed-map refusal evidence.
   `LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX` and
   `LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX`, asserted by
   `static_and_same_axis_live_pointwise_identities_remain_exact`, pin the exact
   all-live schedule and kernel bytes. An accepted source migration must capture
   those values before editing and prove the intended moved values plus unchanged
   static neighbour; that is no longer missing current evidence.
10. **Verified — symbolic artifact association remains downstream.** Current
    fixed-shape artifact construction can project an already-selected access to
    `InputKey`; it does not yet prove that the key/axis is the root binding of
    the semantic symbol. [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md)
    remains downstream of schedule formation and owns that artifact/semantic
    validation. Making it a prerequisite here would create a cycle.
11. **False at this base — the directly related documentation repair is done.**
    The former `docs/architecture.md` anchor `distinguishes inputs by ordinal and
    carries none` and the matching program/request comments were repaired by
    [`repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation`](repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation.md)
    at `6b0ccec4` and closed at `340f8e5f`; that ticket is `done`. Broader
    retired-ordinal prose remains owned by its separately ticketed follow-ups
    and supplies no authority for this surface.
12. **False at this base — mixed pointwise maps now fail closed before
    lowering.** The independent P0
    [`refuse-mixed-pointwise-live-row-major-access-relations-before-lowering`](refuse-mixed-pointwise-live-row-major-access-relations-before-lowering.md)
    landed at `f568467b` and closed at `48088dfb`. Current
    `pointwise_accesses_choose_one_addressing_regime` requires all pointwise
    reads and the write to be static, or all to carry `LiveRowMajor` on the same
    axis. Mixed static/live reads, a static write, and disagreeing live axes now
    return `NumericalOrAccessRefinement` before a verified schedule exists. The
    accepted source implementation will replace that temporary broad refusal
    with the selected dedicated source rule; P0 is no longer an unmet graph
    edge.
13. **False former Pareto-completeness claim — the redundant-axis hybrid hid a
    narrower referenced-consumer replacement.** The earlier packet added
    `LiveRowMajorSource { inner_axis }` plus
    `LiveRowMajor { inner_axis, source_access }`, but did not examine
    `LiveRowMajorSource { inner_axis }` plus
    `LiveRowMajor { source_access }`. In the admitted exact same-shape rank-one
    population, every consumer axis is necessarily the marker axis, so the
    repeated consumer `inner_axis` carries no current information. The narrower
    relation retains exact map-local source lookup, makes source self-reference
    unrepresentable, uses the same fresh tags and Rust layout, and removes one
    public field, one invalid state, and four canonical bytes per consumer. It
    dominates the redundant-axis hybrid for the current contract. Future
    independent consumer axes are a refused population that would require a new
    execution/validation decision, not state this slice should pre-authorize.

## Exact-current-base Fact re-audit — 2026-08-17, `a51305ce5b78628f9fbcbce78fd5cbbdfd43512e`

No production source changed between the independently reviewed current-main
commit `e8141d7decbb8204e7930421d0b1acedef9b4dd5` and this exact repair base:
`git diff --quiet e8141d7decbb8204e7930421d0b1acedef9b4dd5
a51305ce5b78628f9fbcbce78fd5cbbdfd43512e -- crates/tiler-ir/src/schedule
crates/tiler-ir/src/kernel crates/tiler-compiler/src/physical.rs
crates/tiler-compiler/src/pipeline.rs crates/tiler-compiler/src/request.rs
crates/tiler-artifact/src/program/builder.rs` exits zero. The intervening commit
only pulled this ticket back to `in-progress`. No file was edited before this
re-audit.

1. **Verified — Fact 1 remains exact historical evidence.** Re-running its
   ten-path `git diff --stat` still reports 867 insertions and 511 deletions
   between `a660ed61` and `88c7c218`; the current owners remain the authority.
2. **Verified — Fact 2 remains exact.** `request_environment`,
   `symbolic_three_input_elementwise`, `plan_elementwise`,
   `SemanticIdentity::shape_environment`, and `decode_shape_env_subject` retain
   the same authored `[program/0::n]` and exact `a[0]` root evidence.
3. **Verified — Fact 3 remains exact.** Current `live_input_extents`,
   `declare_plan_live_extents`, `emit_live_row_major`, and
   `verify_input_extents` still derive three input operands and consume only one;
   no compiler `LiveRowMajor` constructor exists.
4. **Verified — Fact 4 remains exact.** The rank-one outer iteration shape is
   empty with one static work item, and every element access remains inside the
   live zero-trip-capable loop.
5. **Verified — Fact 5 remains exact.** `TensorRole::Input` is fieldless,
   `AccessOrdinal` owns the complete access-list coordinate, and
   `DeclaredInputOrdinal` remains compiler-private.
6. **Verified — Fact 6 remains exact.** `InputExtentParameter` directly names
   an access and `derive_extent_operands` follows the corresponding stage access
   to `MaterializedOrigin::ProgramInput`; no role-filtered fallback exists.
7. **Verified — Fact 7 remains exact for this subject.**
   `VerifiedScheduledRegion::declared_input_at` retains the checked projection,
   `elementwise_reads_match` still has no source-bound live case, and every
   malformed or mismatched semantic/root subject remains compiler
   `request-binding`.
8. **Verified — Fact 8 remains the correctly labelled inference.** An explicit
   and implicit self-source would still be two spellings, and equal runtime
   values still cannot replace the exact `a[0]` authority.
9. **Verified — Fact 9 remains exact current evidence.** The two-input builder
   and exact live schedule/kernel pins are unchanged.
10. **Verified — Fact 10 remains exact.** Symbolic artifact association is
    still downstream and its ticket still depends on schedule formation.
11. **Verified — Fact 11 remains exact.** The directly related documentation
    repair is `done`; later declared-input prose work supplies no runtime or
    source-binding authority.
12. **Verified — Fact 12 remains exact.** The P0 is `done`, and
    `pointwise_accesses_choose_one_addressing_regime` still rejects mixed or
    disagreeing live maps before lowering.
13. **Verified but incomplete — Fact 13 correctly eliminates the
    redundant-axis referenced consumer.** It does not complete the census:
    the same proof that makes a referenced consumer's `inner_axis` redundant
    also makes it redundant on the contextual marker consumer.
14. **False — the resulting three-survivor and recommendation claims are not
    Pareto-complete.** The packet omitted the complete replacement
    `LiveRowMajorSource { inner_axis }` plus fieldless contextual
    `LiveRowMajor`. With exactly one marker, the containing verified region is
    the consumer's sole axis authority. Relative to the retained-axis marker it
    preserves correctness, the contextual one-domain ceiling, linear
    verification, and map/access layout while removing one public field,
    `AxisMismatch`, four bytes per consumer, and one representable invalid
    state. The retained-axis marker has no surviving advantage under the same
    pre-production and refused-future-population rules that eliminate the
    redundant-axis referenced hybrid, so it is dominated. The frontier,
    diagnostics, tags, byte counts, recommendation, dependent implementation,
    and queue item must be re-derived below.

Reproduce:

```sh
git rev-parse HEAD
git merge-base HEAD main
git diff --stat a660ed618446ade55234993b835e75e26d44921c 88c7c2181ac9a73de56598411915f176c50c3645 -- crates/tiler-ir/src/schedule/model.rs crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/kernel/model.rs crates/tiler-ir/src/kernel/lower.rs crates/tiler-ir/src/kernel/verify.rs crates/tiler-ir/src/program/builder.rs crates/tiler-compiler/src/physical.rs crates/tiler-compiler/src/pipeline.rs crates/tiler-compiler/src/request.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'fn request_environment|draft.bind\(&declared|fn symbolic_three_input_elementwise|fn plan_elementwise|sourced_shape_ref\(program, value\) != Some\(shape\)' crates/tiler-compiler/src/request.rs
rg -n 'pub struct SemanticIdentity|fn shape_environment|pub fn decode_shape_env_subject|enum ShapeEnvSubjectError' crates/tiler-ir/src/semantic/identity.rs crates/tiler-ir/src/shape/env/subject.rs
rg -n 'struct VerifiedTargetRequest|struct VerifiedRequestSubject|fn semantic_identity|request-binding' crates/tiler-compiler/src/request.rs crates/tiler-compiler/src/physical.rs
rg -n 'pub struct AccessOrdinal|pub enum TensorRole|fn live_input_extents|pub struct InputExtentParameter|fn declare_plan_live_extents|fn emit_live_row_major|fn verify_input_extents' crates/tiler-ir/src/schedule crates/tiler-ir/src/kernel
rg -n 'fn declared_input_at|fn elementwise_reads_match|fn derive_extent_operands|MaterializedOrigin::ProgramInput' crates/tiler-compiler/src/physical.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'LiveRowMajor \{' crates --glob '*.rs'
rg -n 'TAG_LIVE_ROW_MAJOR|0x0A|0x0B|fn push_logical_access' crates/tiler-ir/src/schedule/model.rs
rg -n 'fn two_input_pointwise_builder|LIVE_ROW_MAJOR_(SCHEDULE|KERNEL)_IDENTITY_HEX|static_and_same_axis_live_pointwise_identities_remain_exact' crates/tiler-ir/src/kernel/tests.rs
rg -n 'fn pointwise_accesses_choose_one_addressing_regime|every access is static, or every access is' crates/tiler-ir/src/schedule/builder.rs
rg -n -C 12 'LogicalAccess::LinearIdentity \| LogicalAccess::LiveRowMajor|any\(\|addressing\| matches!\(addressing, ReadAddressing::LiveRowMajor|fn emit_live_row_major|if data != &canonical' crates/tiler-ir/src/schedule/builder.rs crates/tiler-ir/src/kernel/lower.rs crates/tiler-ir/src/kernel/verify.rs
rustc --version --verbose
cargo test -p tiler-compiler request::tests::a_symbolic_elementwise_neighbour_reaches_region_formation -- --exact --nocapture
cargo test -p tiler-ir kernel::tests::static_and_same_axis_live_pointwise_identities_remain_exact -- --exact --nocapture
```

## Fixed invariants and exact projection

- The admitted population is whole-program same-shape pointwise F32, rank one,
  at least one read, no materialization, one exact shared
  `SourcedExtent::Symbol`, and an `InputDimension` root binding for that symbol.
  A static root, interface parameter, target property, a differently spelled
  proved-equal symbol, multiple live domains, or a source absent from the
  region remains unsupported.
- Intrinsic schedule verification proves the structural source relation.
  Compiler request binding independently proves that the source access maps to
  the exact `BindingSource::InputDimension` and that every consuming read and
  the result carry the same exact normalized `SourcedShape`. It obtains the
  root only by decoding and revalidating
  `request.semantic_identity().shape_environment().as_bytes()` with
  `decode_shape_env_subject`; it does not recover an `ExtentSources` object.
  Rank, axis number, equality of runtime values, or declaration order alone
  proves none of that.
- Shape-environment decoding is fail-closed compiler binding. A decode error,
  missing matching symbol, non-`InputDimension` root, or disagreement between
  decoded `(InputKey, Axis)` and the access projected through
  `declared_input_at` returns `PhysicalError::Intrinsic { rule:
  "request-binding", region }`. No empty environment, first binding, first
  input, or proved-equal symbol is a fallback.
- In the three-input fixture, canonical read order is `[a, b, c]`, so the root
  source projects exactly as follows:

```text
AccessOrdinal::FIRST
  -> VerifiedScheduledRegion::declared_input_at
  -> DeclaredInputOrdinal(0)
  -> CoverAssembly::AssemblyBinding::Input(0)
  -> stage MaterializedOrigin::ProgramInput { key: InputKey("a") }
  -> ExtentOperandData { key: InputKey("a"), axis: Axis(0), Unsigned }
```

  A program whose root binding names a different declared input must project to
  that input's exact access instead; no “first input” rule is accepted.
- `ExtentSources::determined` never supplies schedule geometry, work, cost, or
  identity. A binding that also proves `n == 4` still compiles or refuses as
  symbolic `n`; it never becomes the literal neighbour.
- One source yields exactly one `InputExtentParameter`. Every live read and the
  write consumes that bound; `n == 0` remains a valid no-element-execution
  neighbour. No nonzero applicability guard is added for row-major pointwise.
- Once any selected live relation appears, every pointwise access driven by its
  loop — every read and the final write — carries the selected live relation.
  A `LinearIdentity`, structural, parametric-broadcast, or other access mixed
  into that loop fails intrinsically at its exact `AccessOrdinal`; lowering
  never silently applies a live offset to a map that did not state it.
- The source access and every consumer have one canonical spelling. The
  schedule contains no interface key or environment identity; local-to-program
  association remains compiler/program authority.
- Reductions, contractions, staged or materialized families, structural maps,
  parametric broadcast, dynamic intermediates, rank above one, multiple
  symbols, output-derived extents, and data-dependent extents retain their
  existing refusals.

## Exact intrinsic and compiler refusal ownership

The source relation needs one dedicated public intrinsic diagnostic arm:

```rust
ScheduledRegionDiagnostic::LiveRowMajorSource {
    rule: LiveRowMajorSourceRule,
}
```

`LiveRowMajorSourceRule` is re-exported from `tiler_ir::schedule` beside
`ScheduledRegionDiagnostic` and has the same public
`Clone + Copy + Debug + Eq + Hash + Ord + PartialEq + PartialOrd` and
`#[non_exhaustive]` shape as the existing nested public schedule-rule enums.

`AccessContract` is not the owner. It describes one access's read/write mode,
map, or ownership contract, while missing, multiple, and inconsistent sources
are cross-access referential-integrity failures even when each access is locally
well formed. A `LinearIdentity` consumer is also locally well formed on its own;
its defect is that the containing region executes it inside a selected live
loop, so `ConsumerMissingRelation` cannot truthfully be `AccessContract`.
Splitting only `SourceNotInputRead` into that broad bucket would also give one
relation two diagnostic authorities and precedence rules.
`NumericalOrAccessRefinement` is less truthful: no scalar program, numerical
permission, or reduction topology disagrees in these cases, and the missing
consumer relation has an exact access coordinate the payload-free broad variant
would discard. The dedicated arm is not cosmetic; each rule below has a
different producer repair, carries the coordinates needed to make that repair,
and is independently perturbable.

If Tom chooses the fieldless-marker survivor, the exact public
`#[non_exhaustive] LiveRowMajorSourceRule` population is:

```rust
Missing,
Multiple { first: AccessOrdinal, second: AccessOrdinal },
SourceNotInputRead { source: AccessOrdinal },
ConsumerMissingRelation { access: AccessOrdinal },
```

If Tom chooses the explicit-source/referenced-consumer survivor, its consumer
stores no axis of its own, so `AxisMismatch` is not representable. Its exact
six-rule population is the fieldless marker's four rules plus the two invalid
states a consumer reference can construct:

```rust
ReferenceOutOfRange { access: AccessOrdinal, source: AccessOrdinal },
InconsistentReference {
    access: AccessOrdinal,
    expected: AccessOrdinal,
    actual: AccessOrdinal,
},
```

If Tom chooses total-local, the exact population is all seven rules: its every
map stores both an axis and a source reference, so both `AxisMismatch` and the
two reference failures remain representable.

The stable rules are respectively
`live-row-major-source-missing`, `live-row-major-source-multiple`,
`live-row-major-source-not-input-read`,
`live-row-major-source-consumer-missing-relation`,
`live-row-major-source-reference-out-of-range`, and
`live-row-major-source-inconsistent-reference`; total-local additionally owns
`live-row-major-source-axis-mismatch`. Fieldless-marker verification recognizes
whether any marker/consumer live relation is present, counts markers, validates
the unique marker's role/mode, requires every pointwise access including the
final write to carry either the unique source marker or the consumer relation,
and then succeeds: the consumer has no axis or reference field to validate.
No marker is `Missing`; the second marker is `Multiple`; the first static or
other relation inside the live-driven access list is
`ConsumerMissingRelation { access }`. Its exact precedence is marker count,
marker role/mode, then complete live-relation coverage; an all-static region
has no source obligation.
Referenced-consumer verification has this exact precedence: reject the first consumer whose
reference is out of range; count source markers and report `Missing` or the
first/second `Multiple`; validate the unique marker's input/read role and mode;
report the first pointwise access, including the final write, which carries
neither selected live relation as `ConsumerMissingRelation`; report the first
consumer whose in-range reference does not name the unique marker as
`InconsistentReference { access, expected: marker, actual }`. A consumer which
points to itself, a non-marker, or a
different marker is therefore inconsistent rather than a second source. The
source has no reference field, and the consumer has no duplicate axis field,
so non-self source and consumer-axis-disagreement states are unrepresentable.

Total-local verification first rejects the first out-of-range reference, counts
self-referencing candidates, validates the unique candidate's role/mode,
requires every pointwise access including the final write to carry the
source-bearing relation, requires every live map to name the unique source, then
checks axes. A live relation with no self-reference is `Missing`; a second
self-reference is `Multiple`; the first non-live map is
`ConsumerMissingRelation { access }`; a live consumer naming another in-range
access after one self-source is established is `InconsistentReference`.
`Missing` applies in total-local even when consumers form a cycle or point to an in-range
non-live access: what is absent is the required self-declaring source. A region
with no selected live relation has no source obligation. For all three survivors
this source gate runs before the pre-existing broad access/refinement gates, so
source role/mode, missing consumer relation, and source integrity never collapse
into `AccessContract` or `NumericalOrAccessRefinement`. Local failures outside
this source relation retain their existing owner.

This additive diagnostic vocabulary changes no identity preimage. Every
survivor is a complete live-relation replacement and retires `0x09`, so no old
tag is reinterpreted under a changed payload. Fieldless-marker and
referenced-consumer both use fresh `0x0A` for `LiveRowMajorSource` and fresh
`0x0B` for their distinct consumer payload; total-local uses `0x0A` for its
single replacement. Every valid live schedule and nested identity value moves,
while static values remain byte-identical. A constructed fieldless consumer
without a marker reports `Missing`; an old all-`0x09` byte run is never decoded
as that new state. Exact source reading confirms
`0x0A` and `0x0B` are unused in the logical-access tag space; tags in other
framed vocabularies are irrelevant to this one. `map_schedule_build_error`
already projects any intrinsic diagnostic's
stable rule into `PhysicalError::Intrinsic`, so the new strings append
explain-reason vocabulary without changing an existing trace's bytes or
spelling. Under `explain.rs`'s explicit append-only rule, neither explain schema
nor renderer steps; schedule, kernel, request, proposal, artifact, and manifest
versions do not step for the diagnostic addition or fail-closed narrowing.
The public diagnostic enum and stable-rule census do grow. An exact replica
probe on the repository toolchain (`rustc 1.99.0-nightly
(eff8269f7 2026-07-18)`, `aarch64-apple-darwin`) copied every current
`LogicalAccess` variant and field type, changed only the live arms to each
candidate, and first checked the replica against the real type. Current,
fieldless-marker, total-local, and referenced-consumer `LogicalAccess` are all
exactly 208 bytes with alignment 8; `Access` is 232/8 in all four cases. The
four-rule fieldless-marker enum is 12/4 and projects the complete diagnostic to
12/4. The six-rule referenced-consumer and seven-rule total-local enums are
16/4 and project the diagnostic to 16/4. The current
`ScheduledRegionDiagnostic` is 2/1. This growth enlarges entries in the
failure-only diagnostics vector; verified schedules retain no diagnostic, and
kernel runtime memory is unchanged. Rust layout is not a stable ABI promise,
so implementation must pin these sizes on the same toolchain rather than
extrapolate them to another compiler.

Reproduce from this exact base with a clean worktree-local `target/`. Run the
first two commands, supply the following Rust block to the second command's
stdin, send EOF, then run the emitted probe. The real-type assertions execute
before the candidate comparisons, so a stale rlib or changed layout cannot
quietly validate the replicas.

```sh
cargo build -p tiler-ir
rustc --edition=2024 -Awarnings -L dependency=target/debug/deps --extern tiler_ir=target/debug/deps/libtiler_ir-05b60e461ca9956c.rlib -o /tmp/tiler-live-complete-layout-probe -
/tmp/tiler-live-complete-layout-probe
```

```rust
use std::mem::{align_of, size_of};
use tiler_ir::schedule::{
    Access, AccessMode, AccessOrdinal, ArithmeticType, AxisDecode,
    BlockedWorkgroupRule, BoundsWitnessId, ContractionAxisSource,
    ContributorCoverageRule, ContributorOrder, CooperativeTileRule,
    LogicalAccess, OwnershipWitnessId, ScheduleComponent,
    ScheduledRegionDiagnostic, SynchronizationRule, TensorRole,
};
use tiler_ir::semantic::{BroadcastAxisMapping, EncodedComponentRole};
use tiler_ir::shape::{Axis, Shape, ShapeEnvIdentity, SourcedShape};

macro_rules! candidate {
    ($name:ident, $($live:tt)*) => {
        enum $name {
            LinearIdentity,
            ScalarBroadcast,
            PackedU4LsbZeroTail { logical_elements: u64 },
            ReductionContributor {
                input_shape: Shape,
                output_shape: Shape,
                axes: Vec<Axis>,
                order: ContributorOrder,
            },
            ContractionOperand {
                operand_shape: Shape,
                output_shape: Shape,
                contracted_shape: Shape,
                sources: Vec<ContractionAxisSource>,
                order: ContributorOrder,
            },
            ReindexBijection {
                operand_shape: Shape,
                result_shape: Shape,
                axes: Vec<AxisDecode>,
            },
            BroadcastReplication {
                operand_shape: Shape,
                result_shape: Shape,
                axes: Vec<AxisDecode>,
            },
            ParametricBroadcast {
                operand_shape: SourcedShape,
                mapping: BroadcastAxisMapping,
                environment: ShapeEnvIdentity,
            },
            $($live)*
        }
    };
}

candidate!(FieldlessMarker,
    LiveRowMajorSource { inner_axis: Axis },
    LiveRowMajor,
);
candidate!(Total,
    LiveRowMajor { inner_axis: Axis, source_access: AccessOrdinal },
);
candidate!(Referenced,
    LiveRowMajorSource { inner_axis: Axis },
    LiveRowMajor { source_access: AccessOrdinal },
);

struct AccessProbe<M> {
    tensor: TensorRole,
    component_role: Option<EncodedComponentRole>,
    mode: AccessMode,
    map: M,
    bounds: BoundsWitnessId,
    ownership: Option<OwnershipWitnessId>,
}

enum FieldlessMarkerRule {
    Missing,
    Multiple { first: AccessOrdinal, second: AccessOrdinal },
    SourceNotInputRead { source: AccessOrdinal },
    ConsumerMissingRelation { access: AccessOrdinal },
}

enum ReferencedRule {
    Missing,
    Multiple { first: AccessOrdinal, second: AccessOrdinal },
    SourceNotInputRead { source: AccessOrdinal },
    ConsumerMissingRelation { access: AccessOrdinal },
    ReferenceOutOfRange { access: AccessOrdinal, source: AccessOrdinal },
    InconsistentReference {
        access: AccessOrdinal,
        expected: AccessOrdinal,
        actual: AccessOrdinal,
    },
}

enum TotalRule {
    Missing,
    Multiple { first: AccessOrdinal, second: AccessOrdinal },
    SourceNotInputRead { source: AccessOrdinal },
    ConsumerMissingRelation { access: AccessOrdinal },
    AxisMismatch { access: AccessOrdinal, source_axis: Axis, access_axis: Axis },
    ReferenceOutOfRange { access: AccessOrdinal, source: AccessOrdinal },
    InconsistentReference {
        access: AccessOrdinal,
        expected: AccessOrdinal,
        actual: AccessOrdinal,
    },
}

enum Diagnostic<R> {
    IncompleteRegion { component: ScheduleComponent },
    LaunchCoverage,
    AccessCount,
    AccessContract,
    BoundsProofCount,
    ProofReference,
    BoundsProof,
    NumericalOrAccessRefinement,
    AccumulationWidth { declared: ArithmeticType, required: ArithmeticType },
    ShapeProductOverflow,
    CooperativeTile { rule: CooperativeTileRule },
    Synchronization { rule: SynchronizationRule },
    ContributorCoverage { rule: ContributorCoverageRule },
    BlockedWorkgroup { rule: BlockedWorkgroupRule },
    LiveRowMajorSource { rule: R },
}

fn layout<T>() -> (usize, usize) {
    (size_of::<T>(), align_of::<T>())
}

fn main() {
    assert_eq!(layout::<LogicalAccess>(), (208, 8));
    assert_eq!(layout::<Access>(), (232, 8));
    assert_eq!(layout::<FieldlessMarker>(), layout::<LogicalAccess>());
    assert_eq!(layout::<Total>(), layout::<LogicalAccess>());
    assert_eq!(layout::<Referenced>(), layout::<LogicalAccess>());
    assert_eq!(layout::<AccessProbe<FieldlessMarker>>(), layout::<Access>());
    assert_eq!(layout::<AccessProbe<Total>>(), layout::<Access>());
    assert_eq!(layout::<AccessProbe<Referenced>>(), layout::<Access>());
    assert_eq!(layout::<ScheduledRegionDiagnostic>(), (2, 1));
    assert_eq!(layout::<FieldlessMarkerRule>(), (12, 4));
    assert_eq!(layout::<ReferencedRule>(), (16, 4));
    assert_eq!(layout::<TotalRule>(), (16, 4));
    assert_eq!(layout::<Diagnostic<FieldlessMarkerRule>>(), (12, 4));
    assert_eq!(layout::<Diagnostic<ReferencedRule>>(), (16, 4));
    assert_eq!(layout::<Diagnostic<TotalRule>>(), (16, 4));
    println!(
        "real map/access = {:?}/{:?}; candidates equal; real diagnostic = {:?}; fieldless-marker/referenced/total/projected diagnostics = {:?}/{:?}/{:?}/{:?}/{:?}/{:?}",
        layout::<LogicalAccess>(),
        layout::<Access>(),
        layout::<ScheduledRegionDiagnostic>(),
        layout::<FieldlessMarkerRule>(),
        layout::<ReferencedRule>(),
        layout::<TotalRule>(),
        layout::<Diagnostic<FieldlessMarkerRule>>(),
        layout::<Diagnostic<ReferencedRule>>(),
        layout::<Diagnostic<TotalRule>>(),
    );
}
```

All semantic disagreement remains compiler-owned `request-binding`: malformed
shape-environment identity bytes, absent decoded symbol/root, the wrong root
source class, decoded key/axis disagreement, or a normalized `SourcedShape`
mismatch maps to that existing rule. None is renamed as an intrinsic source
failure because a standalone schedule cannot see the semantic subject.

## Complete option census

### Status quo and typed deferral

The exact-current status quo is a valid typed deferral: the landed P0 rejects
every mixed addressing regime before lowering. It keeps
`LiveRowMajor { inner_axis }` and
the compiler schedule-stage `UnsupportedSymbolicExtent`, changes no valid
identity bytes or host cost, and supplies none of this ticket's capability. It
is the result if Tom rejects all three survivors, not a completed source-bound
implementation outcome. The landed repair is topology-neutral: it narrowed the
existing surface without selecting or eliminating one of the three source-bound
relations below.

### Fieldless contextual marker replacement — survivor and recommendation

Retire the accepted live relation and replace it with a source marker plus a
fieldless consumer:

```rust
pub enum LogicalAccess {
    LiveRowMajorSource { inner_axis: Axis },
    LiveRowMajor,
    // existing non-live variants unchanged
}
```

Exactly one `TensorRole::Input`/`AccessMode::Read` in an admitted live
pointwise region uses `LiveRowMajorSource`; it both reads that tensor and marks
its axis as the extent operand. Every other pointwise read and the final write
uses `LiveRowMajor` as a consumer; no access executed by the live loop may keep
`LinearIdentity` or another map. A source marker on an intermediate/write, no
marker, two markers, or any access missing the selected live relation fails
intrinsic verification. The consumer stores no axis: the exact same-shape
rank-one compiler proof makes the unique marker's axis the only current
authority. `live_input_extents` derives the marker's exact `AccessOrdinal` and
`inner_axis`; it never searches for the first input.

Compiler binding then decodes the shape-environment subject from retained
`SemanticIdentity` bytes, proves that the marker projects to the symbol's exact
`InputDimension` root, and requires all consumers to carry the exact normalized
shape. Thus the marker cannot silently nominate `b[0]` for the fixture whose
authority is `a[0]`.

**Correctness and strictness.** The source position is explicit by variant and
cannot be out of range or name a detached access. Exactly-one validation makes
missing/multiple authority fail closed, and
`ConsumerMissingRelation { access }` preserves the landed mixed-map closure
under its exact owning source diagnostic instead of the P0's temporary broad
refusal. A consumer-axis disagreement and a dangling or inconsistent consumer
handle are unrepresentable. The cost is contextual meaning: interpreting a
consumer requires the containing verified region's unique marker, and the
representation deliberately has a one-live-domain ceiling.

**Maintenance, compatibility, and host cost.** It adds one map arm but no
retained axis or ordinal per consumer. Verification/lowering scan the bounded
access list once, as `live_input_extents` already does. The measured Rust layout
stays `LogicalAccess` 208/8 and `Access` 232/8. The one source is five canonical
bytes — tag plus axis — and every consumer is its one-byte tag, so the
three-read-plus-write fixture uses eight bytes. Its four-rule diagnostic and
projected outer diagnostic are 12/4 rather than the other survivors' 16/4.
Context must be threaded through one presently detached internal lowering
helper: `kernel::lower::addressing` currently receives only an `Access` and
`ReductionTopology`, so implementation must pass the verified source axis (or
the owning schedule relation) instead of reading an axis from each consumer.
`kernel::builder::scheduled_access_rank` and `kernel::verify::access_rank`
already receive the schedule; `schedule::live_input_extents` already scans it.
The request `read_tensor_elements` wildcard declines this schedule-only map,
the physical builder's `addressed_elements` uses the region element count for
it, and the identity encoder needs no detached semantic interpretation. Thus
the migration is bounded but not zero-cost.

**Identity/schema/public consequences.** Retire `0x09`; changing its payload
from an axis to no fields would reinterpret old bytes. Use fresh unused `0x0A`
for `LiveRowMajorSource` and `0x0B` for fieldless `LiveRowMajor`. Every live
schedule and nested identity value migrates; static values remain
byte-identical. Schedule v6 and kernel v8 domains need not step because both are
fresh injective tags and no enclosing grammar changes; the kernel identity
value moves because it frames the moved schedule identity.
`InputExtentParameter`, kernel-program grammar, artifact extent row, and
artifact schemas remain unchanged. Public diagnostics add
`ScheduledRegionDiagnostic::LiveRowMajorSource` and the four marker rules
above; malformed semantic/root binding stays compiler `request-binding`. This
is a breaking existing public signature and requires Tom under ADR 0075.

**Strongest counterargument.** The consumer map is not self-contained and the
one-marker grammar must be replaced if one region later consumes two independent
live domains; referenced consumers pay more state now but already name the
association a multi-source grammar would need.

**Reversal evidence.** Eliminate this survivor if a public or identity-bearing
consumer must interpret one `LogicalAccess` without its verified region and
cannot instead take checked context, or if the next admitted population
requires two live sources in one region. Prefer it while detached helper
plumbing remains internal and bounded and no accepted next population benefits
from map-local source identity.

**Negative controls.** Remove the marker, duplicate it, place it on the write,
and change one read and then the final write to `LinearIdentity`; require the
four exact dedicated rules in their stated precedence. Independently attempt
to construct `LiveRowMajor { inner_axis: Axis::new(0) }`; the public compile-fail
fixture must prove the consumer is fieldless rather than accepting a hidden
second axis authority.

### Complete total local replacement — survivor

Replace the accepted payload with one total source-bearing relation:

```rust
pub enum LogicalAccess {
    LiveRowMajor {
        inner_axis: Axis,
        source_access: AccessOrdinal,
    },
    // existing non-live variants unchanged
}
```

The source input read carries the one canonical self-reference. Every consumer
carries the same `source_access`. A separate `source_axis` is deliberately
absent: the referenced source map's `inner_axis` is the exact source axis and is
what becomes `InputExtentParameter::axis`; storing it again would create two
authorities that can disagree. Intrinsic verification requires the referenced
access to exist, be `Input`/`Read`, use this relation, and self-reference; every
pointwise access in this admitted region must use the relation and name it.
Compiler binding applies the same decoded-identity root-source and exact-shape
proof as the fieldless-marker survivor.

**Correctness and strictness.** Each map states its complete relation locally.
Out-of-range, non-input, non-read, non-self-referencing, missing consumer
relation, or inconsistent source handles all fail before lowering. The repeated
field makes forged inconsistent states representable, but none verifies.

**Maintenance, compatibility, and host cost.** One relation and one total match
arm serve sources, other reads, and writes, and a later multi-source execution
model already has an explicit association seam. The cost is one repeated
`AccessOrdinal` per live map and validation of equality across them. Canonical
bytes grow by four bytes per live map relative to tag-plus-axis. Measured
`LogicalAccess` and `Access` layout remains 208/8 and 232/8 because larger
existing variants govern both enums. Verification and lowering remain linear;
kernel runtime and kernel memory are unchanged because they still consume one
input-extent operand.

**Identity/schema/public consequences.** Encode the replacement under fresh
unused tag `0x0A` and retire `0x09`; changing the payload under `0x09` would silently
reinterpret old bytes. Existing live schedule and every nested identity value
migrate; static schedule bytes remain identical. As with the marker, schedule
v6, kernel v8, proposal, kernel-program, artifact-stage/program, explain, and
manifest grammar versions need not step when the new tag and unchanged framed
fields are used; all values folding the moved schedule/kernel identity must be
recomputed. The artifact row stays `(InputKey, Axis, AbiType)`. This is a
breaking existing public signature and requires Tom under ADR 0075. Public
diagnostics add `ScheduledRegionDiagnostic::LiveRowMajorSource` and all seven
total-local rules above; compiler decode/root/shape failure remains
`request-binding`.

**Strongest counterargument.** It repeats one region-wide fact on every map,
grows live identity bytes, and creates dangling/inconsistent handles that the
fieldless-marker form cannot represent.

**Reversal evidence.** Prefer the fieldless marker if source repetition measurably grows
the dominant host schedule population or negative-control implementation shows
the repeated equality is error-prone. Prefer this replacement if access
consumers benefit from interpreting a map without a region-wide source scan, or
if a concrete next population needs more than one source.

**Negative controls.** Independently use an out-of-range source, zero and two
self-references, a source on the write, one consumer naming another in-range
access, and one consumer with another axis. Each must reach its exact one of the
seven rules before lowering; changing only the self-reference must not change
which axis becomes the one input-extent operand.

### Explicit source plus referenced consumers — complete replacement survivor

Retire the old live relation and replace it with an explicit source plus a
source-referencing consumer:

```rust
pub enum LogicalAccess {
    LiveRowMajorSource { inner_axis: Axis },
    LiveRowMajor {
        source_access: AccessOrdinal,
    },
    // existing non-live variants unchanged
}
```

Exactly one admitted `Input`/`Read` uses `LiveRowMajorSource`. Every other read
and the final write uses `LiveRowMajor` and names that marker's exact
`AccessOrdinal`. The source has no redundant self-handle; a consumer's source
reference finds the one `inner_axis` which governs the current same-shape
rank-one live loop without a marker search or duplicate axis field. Intrinsic
verification applies the exact six-rule precedence above, and
compiler binding applies the same decoded-identity root and exact normalized
shape proof as the other survivors. The currently admitted slice still has one
live loop and therefore exactly one verified marker; unlike the pure marker
surface, the public fields can later associate different consumers with
different explicit markers without another consumer-signature replacement.
That later multi-source population remains refused by `Multiple` now and needs
its own execution/validation decision.

**Correctness and strictness.** The source is structurally explicit and cannot
carry a dangling or non-self handle. Every consumer states its source locally.
Out-of-range and inconsistent references, zero or two markers, a marker on the
wrong role/mode, and missing live relations are independently refused states.
Consumer-axis disagreement is unrepresentable: the referenced marker owns the
only axis, and compiler binding proves every consumer has the same exact
normalized rank-one shape. The unique-marker proof prevents a consumer from
using its local handle to nominate a second runtime authority.

**Maintenance, compatibility, and host cost.** Two live variants require two
construction and matching arms, but their jobs are disjoint: one declares an
extent source and one consumes it. The source saves one repeated ordinal versus
total-local; every consumer omits total-local's repeated axis; and source
discovery does not depend on equality between a handle and its own position.
Verification remains linear. Exact-base layout probing gives the same
`LogicalAccess` 208/8 and `Access` 232/8 as current and both other survivors,
with no added heap allocation. Canonical map bytes are five for the one source
and five for each consumer: the three-input/read-plus-write fixture uses 20
bytes, above the fieldless marker's eight and below total-local's 36. Kernel
runtime and kernel memory remain unchanged.

**Identity/schema/public consequences.** This is a complete replacement, not
the eliminated additive follower below. Retire old contextual tag `0x09`; use
fresh `0x0A` for `LiveRowMajorSource` and fresh `0x0B` for the referenced
consumer. Every map in a current all-live schedule migrates; no old bytes are
reinterpreted, and static schedule bytes remain exact. Fresh injective tags and
unchanged framed scalar fields need no schedule/kernel/schema domain step; all
identities folding the moved schedule/kernel value must be recomputed. Logical
access tags occur only in the canonical schedule-identity encoder, not in an
artifact/manifest decoder, so an old artifact row is not parsed under the new
payload. A rebuilt live artifact instead receives the recomputed nested identity
value while its artifact row and framing remain unchanged. Public
diagnostics use the six referenced-consumer rules: there is no representable
non-self source or consumer-axis-disagreement state, while
`InconsistentReference` remains necessary for a consumer pointing to itself, a
non-marker, or a different marker. Semantic
decode/root/shape failure remains compiler `request-binding`.

**Strongest counterargument.** It spends a second public variant and two tag
arms to encode a distinction total-local expresses with one self-reference;
every non-source consumer still pays the four-byte handle the fieldless marker
avoids, even though the current grammar proves exactly one marker.
Unlike total-local, it cannot later give a consumer an axis different from its
source without changing that consumer signature.

**Reversal evidence.** Prefer total-local if an implementation prototype shows
the two-arm construction/verification split produces more defects than its
unrepresentable bad-source state prevents, or an admitted population requires
consumer and source axes to differ. Prefer the fieldless marker if no current or next
consumer needs map-local source lookup. Prefer this replacement if a concrete
multi-source design can reuse the marker/reference split without changing
either variant.

**Negative controls.** Independently use an out-of-range consumer reference,
remove and duplicate the marker, and point one consumer at itself and then at a
different in-range non-marker. Require range, missing, multiple, and
inconsistent-reference failures in the exact stated precedence, and prove that
neither a source self-handle nor a consumer axis can be constructed.

### Explicit source plus redundant-axis referenced consumers — eliminated

The earlier packet's hybrid carried both `inner_axis` and `source_access` on
every consumer. It is correct, but the current contract proves one exact
same-shape rank-one live domain, so every consumer axis must equal the marker
axis. Relative to the referenced-consumer survivor above it has the same two
variants, tags, reference failures, 208/8 map layout, 232/8 access layout,
linear verification, kernel runtime, and multi-source association seam, while
adding one public field, `AxisMismatch`, four canonical bytes per consumer, and
one more representable invalid state. Its only advantage is pre-encoding a
future consumer axis distinct from its source. That population is currently
unsupported and needs a separate execution and validation decision; carrying
its state now would weaken current canonicality rather than provide compatible
admission. The narrower referenced-consumer surface therefore dominates this
hybrid for the authorized contract, so it is eliminated before ranking.

### Explicit source plus retained-axis contextual consumers — eliminated

The former marker survivor used `LiveRowMajorSource { inner_axis }` plus
contextual `LiveRowMajor { inner_axis }`. It is correct, but the unique marker
and compiler exact-shape proof already make the source axis the only axis the
current consumer can use. Relative to the fieldless-marker survivor it has the
same two roles, contextual interpretation, one-domain ceiling, 208/8 map
layout, 232/8 access layout, linear region scan, and kernel runtime while adding
one public field, `AxisMismatch`, four canonical bytes per consumer, and one
representable invalid state. Retaining the old `0x09` consumer payload is not a
surviving compatibility advantage: the source map and therefore every valid
live schedule identity already move, and Tiler is pre-production. A future
independent consumer axis is the same unsupported population that cannot save
the redundant-axis referenced hybrid. The fieldless marker dominates this
candidate, so it leaves the frontier.

### Additive/disjoint follower relation — eliminated

The strongest compatibility form would keep existing `LiveRowMajor` for the
one source input and the write, then add an input/read-only
`LiveRowMajorFollower { inner_axis }` for other reads. Exactly-one source and
role restrictions make it correct and disjoint, and current one-source bytes
would remain exact. It is eliminated under Tiler's pre-production replacement
rule: the same variant would mean “source” on one role and “consumer” on
another, leaving permanent role-contextual semantics solely to retain internal
bytes. The fieldless-marker survivor makes both roles explicit with fewer
consumer bytes, and the total replacement has one local relation. Exact
byte continuity must still be captured as migration evidence; it is not a
reason to retain the weaker public model.

An additive source-bearing follower with `source_access` is worse still: it
keeps old `LiveRowMajor`'s mixed role semantics while adding a referenced
consumer. It is not the complete referenced-consumer survivor above, which
retires `0x09`, gives the source and consumer disjoint explicit variants, and
has no duplicate source spelling. Allowing explicit self on the additive
follower would duplicate the old source spelling. Both additive forms are
eliminated.

### Region-level binding — eliminated as dominated

`IndexRegion { live_row_major_source: Option<AccessOrdinal>, .. }` can state the
same one-source relation correctly. Verification would require `Some` exactly
when live maps exist, validate the named input/read/source axis, and require
every pointwise access including the write to carry `LiveRowMajor`; omitting
that coverage proof would reintroduce the mixed-map wrong code the landed P0
now closes and is eliminated as incorrect before ranking. Even in its strongest
fail-closed form it is worse than the fieldless marker on
every relevant dimension: it adds an optional field and invalid
dangling-coordinate states to every `IndexRegion` and changes all 12 actual
construction sites under `crates/` — one IR builder construction, eight
physical-compiler constructions, and three compiler integration fixtures —
including the static ones. The exact source command
`rg -n '\bIndexRegion\s*\{' crates --glob '*.rs'` returns 13 lines: the public
struct definition plus those 12 literals. The option also keeps `LiveRowMajor`
contextual and has the same one-domain ceiling. Its only benefit is direct
lookup; the bounded source scan already exists and both the region-level and
fieldless-marker forms must scan consumers to validate them. It leaves the
frontier.

### Implicit collapse, schedule semantic state, and broad geometry — eliminated

- Deduplicating three current extents by axis, choosing the first input, or
  letting lowering select the first matching operand invents authority. It can
  silently execute a hand-built region against an unrelated extent and is
  incorrect.
- Copying `ShapeSymbol`, `ShapeEnvIdentity`, `InputKey`, or root binding into
  shared schedule IR duplicates compiler/program authority and couples local
  computation identity to caller naming. All three survivors use the existing
  checked projection instead.
- Replacing `IndexRegion::iteration_shape`, launch, cost, and program geometry
  with sourced forms is a coherent broader redesign but buys nothing for the
  rank-one empty-outer-domain slice. It is worse in surface, proof, identity,
  and host risk without improving this outcome.
- Making only one current read live and leaving the other `[n]` reads
  `LinearIdentity` misstates their bounds/addressing; compiler-only dedup would
  also make public schedule/kernel verification disagree. These are not narrow
  correct slices.

### Further research — not on the frontier

All construction, verification, lowering, identity, compiler-binding,
program/artifact association, refusal, and fixture paths needed to decide the
topology are present and bounded. Exact current Rust layout is measured above,
and the exact all-live schedule and kernel bytes are now pinned at this base. A
production prototype would merely implement an unaccepted public
surface, so no research ticket is required before decision. A measured layout
or implementation-defect difference can reverse the recommendation among the
three correct survivors; both redundant-axis forms are dominated for the
current contract rather than further frontier choices.

## Pareto frontier

| Survivor | Correctness / strictness | Maintenance / compatibility | Tiler host runtime / memory | Identity / public / unsupported consequence |
| --- | --- | --- | --- | --- |
| Fieldless contextual marker | Exact one structural marker, complete live relation on every pointwise access, plus decoded-identity compiler proof; consumer axis and bad handles unrepresentable | Two disjoint variants and four rules; contextual consumers; one detached lowering helper must accept checked context; public one-domain ceiling | Linear scan; 208/8 map and 232/8 access; eight fixture bytes; failure diagnostic 12/4; unchanged kernel runtime | Source `0x0A`, fieldless consumer `0x0B`, old `0x09` retired; all valid live values migrate; static bytes/schema versions stay; artifact row unchanged |
| Total local replacement | Exact checked source-bearing relation on every pointwise access plus decoded-identity compiler proof; self-contained | One variant and one match arm; repeated dangling/inconsistent/self handles; explicit future association seam | Linear validation; 208/8 map and 232/8 access; nine canonical bytes per live map; failure diagnostic 16/4; unchanged kernel runtime | Replacement `0x0A`, old `0x09` retired; seven rules; all valid live values migrate; static bytes/schema versions stay; artifact row unchanged |
| Explicit source + referenced consumers | Exact marker plus range-checked local consumer references and decoded-identity compiler proof; bad source self-handle and consumer-axis disagreement unrepresentable | Two disjoint variants; map-local source lookup; public multi-source association seam while current multiple markers refuse; a future independent consumer axis requires a reviewed signature change | Linear validation; 208/8 map and 232/8 access; five bytes per map (20 fixture bytes); failure diagnostic 16/4; unchanged kernel runtime | Source `0x0A`, consumer `0x0B`, old `0x09` retired; six rules; all valid live values migrate; static bytes/schema versions stay; artifact row unchanged |

None dominates. The fieldless marker minimizes current canonical state,
diagnostic surface, and representable invalid states, but its consumers are
contextual and its public shape cannot identify different sources later.
Total-local minimizes variant count, makes every map self-contained, and
already carries independent per-consumer axes, but represents redundant and
potentially wrong source self-reference and axis state. The
referenced-consumer form spends a second variant and repeated handles to make
the source association map-local and leave a multi-source seam, while avoiding
total-local's self-handle and duplicate axis.

The recommendation is the fieldless contextual marker replacement. ADR 0046
makes the `IndexRegion` the owner of its checked access maps; it does not require
one enum value detached from that verified region to carry a duplicate of a
region-wide unique fact. The explicit source variant and the fieldless consumer
variant jointly state the current one-source relation. The governing builder,
verifier, extent derivation, and identity consumers own or can accept the
containing region; one internal lowering helper must be given that checked
context. On this authorized population a consumer handle or axis cannot
distinguish two legal meanings, so retaining either weakens canonicality and
adds failure surface. The fieldless form is therefore the smallest complete
current contract: four rules, eight fixture bytes, and no dangling handle or
axis mismatch. Its strongest counterarguments are total-local's one-variant
self-contained grammar and the referenced form's map-local
association/multi-source seam. Neither dominates: both avoid the contextual
helper refactor but pay repeated state for a property the fieldless form
deliberately defers, while the fieldless form would require a reviewed new
consumer relation if a second source becomes admissible.

## One decision question for Tom

Should Tiler accept the **fieldless contextual marker complete replacement**
(recommended), with `LiveRowMajorSource { inner_axis: Axis }`, fieldless
consumer `LiveRowMajor`, fresh tags `0x0A`/`0x0B`, and the four exact source
rules; accept the **explicit-source/referenced-consumer complete replacement**,
with `LiveRowMajorSource { inner_axis: Axis }`, referenced
`LiveRowMajor { source_access: AccessOrdinal }`, fresh tags `0x0A`/`0x0B`, and
the six exact source rules; accept the
**complete total local replacement**
`LiveRowMajor { inner_axis: Axis, source_access: AccessOrdinal }`,
with the source access self-referencing, every pointwise access carrying the
relation and naming it, and the seven
exact `LiveRowMajorSourceRule`s above; or reject all three and
retain the typed schedule-stage deferral after the independent mixed-map
correctness repair which is already landed?

## Independent evidence required after acceptance

- Preserve the three-input census and prove the source `AccessOrdinal` projects
  to root binding `a[0]` decoded from
  `semantic_identity.shape_environment().as_bytes()`, exactly one kernel extent operand, and eventually
  artifact row `(InputKey("a"), Axis(0), Unsigned)`. Move the root binding to
  `c[0]` with access order unchanged and prove source projection moves to access
  2 rather than staying first.
- Change one input to a same-spelled symbol in another scope, then to a distinct
  symbol the environment proves equal. The admitted exact-shape population must
  refuse; neither spelling equality nor `proves_equal` may widen it silently.
- Truncate the shape-environment subject and independently corrupt its domain in
  a compiler-private helper driven by the same bytes the production decoder
  receives. Both must exercise `decode_shape_env_subject` and surface existing
  `PhysicalError::Intrinsic { rule: "request-binding", .. }`; they may not
  panic, default to an empty environment, or select another binding.
- Intrinsic negatives: missing source, two sources, source on intermediate or
  write, wrong mode, and an access without the selected live
  relation. Independently change one read and then the final write to
  `LinearIdentity`; each must produce
  `LiveRowMajorSourceRule::ConsumerMissingRelation { access }` and stable
  `live-row-major-source-consumer-missing-relation`, never a broad access or
  refinement failure. Quote the exact dedicated stable rules and payload
  coordinates. For the recommended fieldless marker, pin the exact public
  source fields and unit consumer with a positive external construction, then
  compile-fail attempts to add `inner_axis` and `source_access` to the consumer;
  prove the four-rule census is total and neither `AxisMismatch` nor a reference
  failure can be constructed. For the total replacement separately perturb an out-of-range
  source, zero and two self-references, and one live consumer's source handle; require
  `live-row-major-source-reference-out-of-range`,
  `live-row-major-source-missing`, `live-row-major-source-multiple`, and
  `live-row-major-source-inconsistent-reference` independently rather than one
  broad access error. For the referenced-consumer replacement, independently
  perturb an out-of-range consumer reference, remove and duplicate the marker, point one consumer at
  itself and then at another in-range non-marker. Require the
  range/missing/multiple/inconsistent rules in the stated precedence, while
  proving no source self-reference or consumer-axis-disagreement state can be
  constructed. Pin its exact source/reference fields with a positive external
  construction and compile-fail attempts to add `inner_axis` to the consumer.
  For total-local only, independently alter one
  consumer axis and require `live-row-major-source-axis-mismatch`.
- Keep the schedule fixed and perturb only the checked request/root binding;
  `verify_region_subject_binding` must reject invalid reuse. Separately reorder
  stage accesses and prove program/artifact construction cannot map the extent
  to another `InputKey`.
- Exercise `n == 0`, `1`, `14`, and `15`: one static outer invocation,
  zero/one/many inner iterations, one schedule/kernel identity across live
  values, and no element access outside the loop.
- Before migration capture exact canonical schedule and kernel bytes plus
  SHA-256 for `kernel/tests.rs::live_row_major_region`,
  `metal/tests.rs::live_row_major_kernel`, and
  `build/metal_assembly.rs::live_row_major_unit`. Perturb the source read and
  write independently so both are load-bearing. Recompute every nested identity
  value after implementation and prove static schedules byte-identical. Reuse
  `0x0A` for the consumer and then restore it; the injectivity check must fail on
  that production tag collision. Independently restore consumer tag `0x09` and
  prove the old live pin collides with or is reinterpreted by the changed
  payload, demonstrating why `0x0B` is load-bearing.
- Run the literal `[4]` neighbour independently and prove its schedule/kernel
  bytes do not move. Remove the new source path and show the original
  `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }`.
- Count the admitted and refused populations explicitly. Tests must perturb the
  subject, never their assertions, and show the failure text.

## Exact follow-up graph and closing conditions

The hard path is:

```text
decide-the-schedule-local-input-ordinal-model (done)
  -> decide-the-full-list-access-coordinate-for-out-of-list-references (done)
  -> reconcile-input-ordinal-region-local-and-declared-input-semantics (done)
  -> this decision (in-progress packet repair; awaiting Tom only after review)
  -> admit-symbolic-extents-through-schedule-formation (blocked implementation)
  -> associate-live-extent-operands-with-symbolic-semantic-interface-axes (todo)
  -> deliver-an-artifact-family-from-a-symbolic-region (todo, also has its existing live-payload dependency)

refuse-mixed-pointwise-live-row-major-access-relations-before-lowering (P0 done)
  -> admit-symbolic-extents-through-schedule-formation (blocked only on this decision)
```

The documentation repair and P0 are both `done`. The former supplied no runtime
authority; the latter supplied the topology-neutral fail-closed current surface.
Every implementation dependency except this in-progress decision is now done,
so the graph is acyclic and the public choice is the sole unmet implementation
edge. Returning the ticket to `awaiting-decision` is a post-review coordinator
action, not part of this repair.

Only Tom accepts one exact surface. On acceptance, record who/date/venue/relay
provenance here, leave implementation to
[`admit-symbolic-extents-through-schedule-formation`](admit-symbolic-extents-through-schedule-formation.md),
and preserve artifact association as its downstream hard edge. On rejection,
record the rejected premise and keep the current typed refusal; do not mark the
capability implemented or close blocked dependents by inertia.
