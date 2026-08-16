---
id: decide-the-source-bound-live-row-major-access-surface
title: Decide the source-bound LiveRowMajor access surface
status: awaiting-decision
priority: p1
dependencies: [accept-the-live-extent-operand-public-surface, reconcile-input-ordinal-region-local-and-declared-input-semantics]
related: [admit-symbolic-extents-through-schedule-formation, associate-live-extent-operands-with-symbolic-semantic-interface-axes, deliver-an-artifact-family-from-a-symbolic-region, repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation, refuse-mixed-pointwise-live-row-major-access-relations-before-lowering]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, schedule, shapes, identity]
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

## Exact-base Fact audit — 2026-08-16, `98669e8ea9cafc91b3a9139ff821781560c526bd`

This supersedes the 2026-08-14 packet completely. No production or decision
document was edited before this audit.

1. **False — the source evidence did move.** The old packet said the relevant
   paths were byte-identical to `67fc9cac`; the accepted AccessOrdinal
   reconciliation changed nine of the named files by 499 insertions and 411
   deletions between old packet base `a660ed61` and this base. Current source,
   not that historical comparison, governs this decision.
2. **Imprecise — one authored shape and root survive as checked identity bytes,
   not as the authored environment object.** `request.rs`, anchors
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
3. **Verified with repaired coordinate names — no shared live source is
   represented today.** `live_input_extents` emits one
   `(AccessOrdinal, Axis)` for every input read whose map is `LiveRowMajor`.
   Three live reads therefore produce accesses 0, 1, and 2. Kernel lowering's
   `emit_live_row_major` chooses one matching operand as `columns`, while
   `verify_input_extents` requires the complete derived list and rejects the two
   unused operands as `UnusedInputExtent`. No compiler live-row-major
   construction exists at this base.
4. **Verified, with one wording repair — rank one needs no sourced schedule
   geometry.** The accepted relation is a static outer product plus a live inner
   loop. For rank one the outer `Shape` is empty, its product is one, and
   `KernelSchedule::{work_items, launch.grid_threads}` can both be one. At
   `n == 0` that one invocation enters a zero-trip loop; all element loads and
   stores remain inside the loop, so it executes none. “One launch” means one
   static outer invocation, not a nonempty live domain.
5. **False — the ordinal-authority prerequisite is resolved.** Current
   `TensorRole::Input` is fieldless. `AccessOrdinal`, anchor `The exact position
   in a scheduled region's complete ordered access list`, is the sole shared
   local coordinate, including intermediate reads and the final write. Public
   `InputOrdinal` no longer exists; retained declared association uses
   compiler-private `DeclaredInputOrdinal`.
6. **Imprecise — artifact mapping now resolves the exact coordinate.**
   `InputExtentParameter { access: AccessOrdinal, axis: Axis }` directly indexes
   the complete schedule access and corresponding kernel-buffer list.
   `derive_extent_operands`, anchor `maps that position through the matching
   stage access`, follows that exact position to
   `MaterializedOrigin::ProgramInput { key }` and emits the existing artifact
   row `(InputKey, Axis, AbiType)`. It neither filters inputs nor searches for a
   first matching role.
7. **Imprecise — compiler request binding owns the proof, but through two
   retained subjects.** `VerifiedScheduledRegion::declared_input_at`, anchor
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
9. **Verified construction census, but false as an identity pin.** Every current
   live-row-major constructor uses exactly one input/read plus one
   intermediate-or-output/write with `LiveRowMajor`: kernel tests, artifact
   tests, Metal tests, build assembly, and the runtime fixture. The tests compare
   identity equality/difference, but no exact schedule or kernel canonical byte
   string or digest is pinned. Capturing those bytes before migration is
   required implementation evidence, not current Fact.
10. **Verified — symbolic artifact association remains downstream.** Current
    fixed-shape artifact construction can project an already-selected access to
    `InputKey`; it does not yet prove that the key/axis is the root binding of
    the semantic symbol. [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md)
    remains downstream of schedule formation and owns that artifact/semantic
    validation. Making it a prerequisite here would create a cycle.
11. **Verified — the landed reconciliation left a separate documentation
    defect.** `docs/architecture.md`, anchor `distinguishes inputs by ordinal and carries none`,
    and compiler comments still attribute declared ordinals to the fieldless
    role or to intrinsic schedule validation. The nonblocking
    [`repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation`](repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation.md)
    ticket owns those production/doc edits; they are not authority for this
    surface.
12. **False former omission — current mixed pointwise access maps are silently
    unsound.** `verify_pointwise_region`, anchor
    `LogicalAccess::LinearIdentity | LogicalAccess::LiveRowMajor { .. }`, admits
    either map independently on each read and the write. Kernel lowering,
    anchor `.any(|addressing| matches!(addressing,
    ReadAddressing::LiveRowMajor { .. }))`, selects `emit_live_row_major` when
    any read is live; that emitter loads every read and stores the result at its
    one `row * columns + col` offset without consulting the remaining per-read
    maps. Canonical kernel verification re-derives the same body and does not
    recover the promised per-map offsets. The new independent P0
    [`refuse-mixed-pointwise-live-row-major-access-relations-before-lowering`](refuse-mixed-pointwise-live-row-major-access-relations-before-lowering.md)
    owns the immediate fail-closed repair and is a hard prerequisite of source
    implementation, not of Tom choosing the exact surface.
13. **False former Pareto-completeness claim — a clean hybrid replacement was
    omitted.** Current source has one `LiveRowMajor { inner_axis }` arm and no
    source carrier; nothing forces a replacement to choose between a pure
    marker and a self-referencing relation. Retiring that arm and introducing
    `LiveRowMajorSource { inner_axis }` plus referenced consumers
    `LiveRowMajor { inner_axis, source_access }` is materially distinct. It
    removes role-contextual source meaning, makes consumers self-contained,
    cannot encode a bad source self-reference, and saves one ordinal relative
    to total-local, at the cost of a second variant and handles the pure marker
    avoids. Exact reading finds fresh logical-access tags `0x0A` and `0x0B`;
    the measured host layout is unchanged. It survives below as a third
    nondominated candidate.

Reproduce:

```sh
git rev-parse HEAD
git merge-base HEAD main
git diff --stat a660ed618446ade55234993b835e75e26d44921c 98669e8ea9cafc91b3a9139ff821781560c526bd -- crates/tiler-ir/src/schedule/model.rs crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/kernel/model.rs crates/tiler-ir/src/kernel/lower.rs crates/tiler-ir/src/kernel/verify.rs crates/tiler-ir/src/program/builder.rs crates/tiler-compiler/src/physical.rs crates/tiler-compiler/src/pipeline.rs crates/tiler-compiler/src/request.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'fn request_environment|draft.bind\(&declared|fn symbolic_three_input_elementwise|fn plan_elementwise|sourced_shape_ref\(program, value\) != Some\(shape\)' crates/tiler-compiler/src/request.rs
rg -n 'pub struct SemanticIdentity|fn shape_environment|pub fn decode_shape_env_subject|enum ShapeEnvSubjectError' crates/tiler-ir/src/semantic/identity.rs crates/tiler-ir/src/shape/env/subject.rs
rg -n 'struct VerifiedTargetRequest|struct VerifiedRequestSubject|fn semantic_identity|request-binding' crates/tiler-compiler/src/request.rs crates/tiler-compiler/src/physical.rs
rg -n 'pub struct AccessOrdinal|pub enum TensorRole|fn live_input_extents|pub struct InputExtentParameter|fn declare_plan_live_extents|fn emit_live_row_major|fn verify_input_extents' crates/tiler-ir/src/schedule crates/tiler-ir/src/kernel
rg -n 'fn declared_input_at|fn elementwise_reads_match|fn derive_extent_operands|MaterializedOrigin::ProgramInput' crates/tiler-compiler/src/physical.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'LiveRowMajor \{' crates --glob '*.rs'
rg -n 'TAG_LIVE_ROW_MAJOR|0x0A|0x0B|fn push_logical_access' crates/tiler-ir/src/schedule/model.rs
rg -n -C 12 'LogicalAccess::LinearIdentity \| LogicalAccess::LiveRowMajor|any\(\|addressing\| matches!\(addressing, ReadAddressing::LiveRowMajor|fn emit_live_row_major|if data != &canonical' crates/tiler-ir/src/schedule/builder.rs crates/tiler-ir/src/kernel/lower.rs crates/tiler-ir/src/kernel/verify.rs
rustc --version --verbose
cargo test -p tiler-compiler request::tests::a_symbolic_elementwise_neighbour_reaches_region_formation -- --exact --nocapture
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

If Tom chooses the marker survivor, the exact public
`#[non_exhaustive] LiveRowMajorSourceRule` population is:

```rust
Missing,
Multiple { first: AccessOrdinal, second: AccessOrdinal },
SourceNotInputRead { source: AccessOrdinal },
ConsumerMissingRelation { access: AccessOrdinal },
AxisMismatch {
    access: AccessOrdinal,
    source_axis: Axis,
    access_axis: Axis,
},
```

If Tom chooses either source-reference survivor — the total-local relation or
the explicit-source/referenced-consumer hybrid — the exact population adds the
two invalid states a consumer reference can construct:

```rust
ReferenceOutOfRange { access: AccessOrdinal, source: AccessOrdinal },
InconsistentReference {
    access: AccessOrdinal,
    expected: AccessOrdinal,
    actual: AccessOrdinal,
},
```

The stable rules are respectively
`live-row-major-source-missing`, `live-row-major-source-multiple`,
`live-row-major-source-not-input-read`,
`live-row-major-source-consumer-missing-relation`,
`live-row-major-source-axis-mismatch`,
`live-row-major-source-reference-out-of-range`, and
`live-row-major-source-inconsistent-reference`. Marker verification recognizes
whether any marker/consumer live relation is present, counts markers, validates
the unique marker's role/mode, requires every pointwise access including the
final write to carry either the unique source marker or the consumer relation,
then checks axes, reporting the first failing access in list order. No marker is
`Missing`; the second marker is `Multiple`; the first static or other relation
inside the live-driven access list is `ConsumerMissingRelation { access }`.
Hybrid verification has this exact precedence: reject the first consumer whose
reference is out of range; count source markers and report `Missing` or the
first/second `Multiple`; validate the unique marker's input/read role and mode;
report the first pointwise access, including the final write, which carries
neither selected live relation as `ConsumerMissingRelation`; report the first
consumer whose in-range reference does not name the unique marker as
`InconsistentReference { access, expected: marker, actual }`; then report the
first axis disagreement. A consumer which points to itself, a non-marker, or a
different marker is therefore inconsistent rather than a second source. The
source has no reference field, so a non-self source state is unrepresentable.

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

This additive diagnostic vocabulary changes no identity preimage, but marker
validation does narrow current values: a formerly verified all-`0x09`
`LiveRowMajor` region has no new marker and becomes `Missing`. Its existing
schedule bytes are not re-encoded or reinterpreted; they leave the verified
population. The new refusal is that subject's first trace value rather than a
different encoding of an older refusal. Marker uses fresh logical-access tag
`0x0A` for `LiveRowMajorSource` and retains `0x09` only for its narrowed
consumer. Total-local uses `0x0A` for its replacement and retires `0x09`.
Hybrid retires `0x09` completely: `LiveRowMajorSource` uses fresh `0x0A`, and
the referenced consumer uses fresh `0x0B`. Exact source reading confirms
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
marker, total-local, and hybrid `LogicalAccess` are all exactly 208 bytes with
alignment 8; `Access` is 232/8 in all four cases. The five-rule marker enum is
16/4; either seven-rule reference enum is 16/4. Adding the nested diagnostic
grows `ScheduledRegionDiagnostic` from 2/1 to 16/4 for every survivor. That
last growth enlarges entries in the failure-only diagnostics vector; verified
schedules retain no diagnostic, and kernel runtime memory is unchanged. Rust
layout is not a stable ABI promise, so implementation must pin these sizes on
the same toolchain rather than extrapolate them to another compiler.

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

candidate!(Marker,
    LiveRowMajorSource { inner_axis: Axis },
    LiveRowMajor { inner_axis: Axis },
);
candidate!(Total,
    LiveRowMajor { inner_axis: Axis, source_access: AccessOrdinal },
);
candidate!(Hybrid,
    LiveRowMajorSource { inner_axis: Axis },
    LiveRowMajor { inner_axis: Axis, source_access: AccessOrdinal },
);

struct AccessProbe<M> {
    tensor: TensorRole,
    component_role: Option<EncodedComponentRole>,
    mode: AccessMode,
    map: M,
    bounds: BoundsWitnessId,
    ownership: Option<OwnershipWitnessId>,
}

enum MarkerRule {
    Missing,
    Multiple { first: AccessOrdinal, second: AccessOrdinal },
    SourceNotInputRead { source: AccessOrdinal },
    ConsumerMissingRelation { access: AccessOrdinal },
    AxisMismatch { access: AccessOrdinal, source_axis: Axis, access_axis: Axis },
}

enum ReferenceRule {
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
    assert_eq!(layout::<Marker>(), layout::<LogicalAccess>());
    assert_eq!(layout::<Total>(), layout::<LogicalAccess>());
    assert_eq!(layout::<Hybrid>(), layout::<LogicalAccess>());
    assert_eq!(layout::<AccessProbe<Marker>>(), layout::<Access>());
    assert_eq!(layout::<AccessProbe<Total>>(), layout::<Access>());
    assert_eq!(layout::<AccessProbe<Hybrid>>(), layout::<Access>());
    assert_eq!(layout::<ScheduledRegionDiagnostic>(), (2, 1));
    assert_eq!(layout::<MarkerRule>(), (16, 4));
    assert_eq!(layout::<ReferenceRule>(), (16, 4));
    assert_eq!(layout::<Diagnostic<MarkerRule>>(), (16, 4));
    assert_eq!(layout::<Diagnostic<ReferenceRule>>(), (16, 4));
    println!(
        "real map/access = {:?}/{:?}; candidates equal; real diagnostic = {:?}; marker/reference/projected diagnostics = {:?}/{:?}/{:?}/{:?}",
        layout::<LogicalAccess>(),
        layout::<Access>(),
        layout::<ScheduledRegionDiagnostic>(),
        layout::<MarkerRule>(),
        layout::<ReferenceRule>(),
        layout::<Diagnostic<MarkerRule>>(),
        layout::<Diagnostic<ReferenceRule>>(),
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

The current unpatched status quo is eliminated: mixed `LinearIdentity` and
`LiveRowMajor` pointwise accesses can verify and lower with one live offset
applied to every buffer, so it is not correct or fail-closed. The valid deferral
is the status quo only *after* the independent P0 repair rejects every mixed
addressing regime. That repaired state keeps `LiveRowMajor { inner_axis }` and
the compiler schedule-stage `UnsupportedSymbolicExtent`, changes no valid
identity bytes or host cost, and supplies none of this ticket's capability. It
is the result if Tom rejects all three survivors, not a completed source-bound
implementation outcome. The P0 is topology-neutral: it narrows the existing
surface before any source model is accepted and neither selects nor eliminates
one of the three source-bound relations below.

### Narrow fail-closed marker slice — survivor

Add one source-marker variant and narrow the accepted variant to consumers:

```rust
pub enum LogicalAccess {
    LiveRowMajorSource { inner_axis: Axis },
    LiveRowMajor { inner_axis: Axis },
    // existing non-live variants unchanged
}
```

Exactly one `TensorRole::Input`/`AccessMode::Read` in an admitted live
pointwise region uses `LiveRowMajorSource`; it both reads that tensor and marks
its axis as the extent operand. Every other pointwise read and the final write
uses `LiveRowMajor` as a consumer; no access executed by the live loop may keep
`LinearIdentity` or another map. A source marker on an intermediate/write, no
marker, two markers, any access missing the selected live relation, or an axis
disagreement fails intrinsic verification. `live_input_extents` derives the
marker's exact `AccessOrdinal` and `inner_axis`; it never searches for the first
input.

Compiler binding then decodes the shape-environment subject from retained
`SemanticIdentity` bytes, proves that the marker projects to the symbol's exact
`InputDimension` root, and requires all consumers to carry the exact normalized
shape. Thus the marker cannot silently nominate `b[0]` for the fixture whose
authority is `a[0]`.

**Correctness and strictness.** The source position is explicit by variant and
cannot be out of range or name a detached access. Exactly-one validation makes
missing/multiple authority fail closed, and
`ConsumerMissingRelation { access }` closes the current mixed-map wrong-code
population before lowering. The cost is contextual meaning:
interpreting a consumer requires the containing verified region's unique
marker, and the representation deliberately has a one-live-domain ceiling.

**Maintenance, compatibility, and host cost.** It adds one map arm but no
retained ordinal per consumer. Verification/lowering scan the bounded access
list once, as `live_input_extents` already does. The measured Rust layout stays
`LogicalAccess` 208/8 and `Access` 232/8. Canonical schedule bytes remain one
tag plus one axis — five bytes — per live map. The current one-source
source-read changes from tag `0x09` to `0x0A`; consumer reads and the write
retain narrowed `0x09`. Static subjects are untouched.

**Identity/schema/public consequences.** Use fresh unused logical-access tag
`0x0A` for `LiveRowMajorSource`; do not reinterpret `0x09` as a marker. Existing live
schedule values migrate; static values remain byte-identical. Schedule v6 and
kernel v8 domains need not step because no existing tag payload or kernel
grammar changes; the kernel identity value moves because it frames the moved
schedule identity. `InputExtentParameter`, kernel-program grammar, artifact
extent row, and artifact schemas remain unchanged. Public diagnostics add
`ScheduledRegionDiagnostic::LiveRowMajorSource` and the five marker rules above;
malformed semantic/root binding stays compiler `request-binding`. This is public
semantic and diagnostic growth plus a narrowing of where the accepted
`LiveRowMajor` input form verifies, so Tom must accept it despite the enums being
`#[non_exhaustive]`.

**Strongest counterargument.** The consumer map is not self-contained and the
one-marker grammar must be replaced if one region later consumes two independent
live domains.

**Reversal evidence.** Eliminate this survivor if a current consumer observes
one `LogicalAccess` without its verified region, or if the next admitted
population requires two live sources in one region. Prefer it if a bounded
prototype shows the total reference only creates repeated invalid state and no
current or next consumer benefits from map-local source identity.

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
proof as the marker survivor.

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
marker form cannot represent.

**Reversal evidence.** Prefer the marker if source repetition measurably grows
the dominant host schedule population or negative-control implementation shows
the repeated equality is error-prone. Prefer this replacement if access
consumers benefit from interpreting a map without a region-wide source scan, or
if a concrete next population needs more than one source.

### Explicit source plus referenced consumers — complete replacement survivor and recommendation

Retire the old live relation and replace it with an explicit source plus a
self-contained consumer:

```rust
pub enum LogicalAccess {
    LiveRowMajorSource { inner_axis: Axis },
    LiveRowMajor {
        inner_axis: Axis,
        source_access: AccessOrdinal,
    },
    // existing non-live variants unchanged
}
```

Exactly one admitted `Input`/`Read` uses `LiveRowMajorSource`. Every other read
and the final write uses `LiveRowMajor` and names that marker's exact
`AccessOrdinal`. The source has no redundant self-handle; a consumer has all
the information needed to find and check its source without a marker search.
Intrinsic verification applies the exact seven-rule precedence above, and
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
wrong role/mode, missing live relations, and axis disagreement are all
representable, independently refused states. The unique-marker proof prevents
a consumer from using its local handle to nominate a second runtime authority.

**Maintenance, compatibility, and host cost.** Two live variants require two
construction and matching arms, but their jobs are disjoint: one declares an
extent source and one consumes it. The source saves one repeated ordinal versus
total-local, and source discovery does not depend on equality between a handle
and its own position. Verification remains linear. Exact-base layout probing
gives the same `LogicalAccess` 208/8 and `Access` 232/8 as current and both
other survivors, with no added heap allocation. Canonical map bytes are five
for the one source and nine for each consumer: the three-input/read-plus-write
fixture uses 32 bytes rather than marker's 20 or total-local's 36. Kernel
runtime and kernel memory remain unchanged.

**Identity/schema/public consequences.** This is a complete replacement, not
the eliminated additive follower below. Retire old contextual tag `0x09`; use
fresh `0x0A` for `LiveRowMajorSource` and fresh `0x0B` for the referenced
consumer. Both current source and consumer values migrate, no old bytes are
reinterpreted, and static schedule bytes remain exact. Fresh injective tags and
unchanged framed scalar fields need no schedule/kernel/schema domain step; all
identities folding the moved schedule/kernel value must be recomputed. Public
diagnostics use the same seven rules as total-local: there is no representable
non-self source state, while `InconsistentReference` remains necessary for a
consumer pointing to itself, a non-marker, or a different marker. Semantic
decode/root/shape failure remains compiler `request-binding`.

**Strongest counterargument.** It spends a second public variant and two tag
arms to encode a distinction total-local expresses with one self-reference;
every non-source consumer still pays the four-byte handle the marker avoids.

**Reversal evidence.** Prefer total-local if an implementation prototype shows
the two-arm construction/verification split produces more defects than its
unrepresentable bad-source state prevents. Prefer the marker if no current or
next consumer needs map-local source lookup and identity volume becomes a
material host cost. Prefer the hybrid if source self-reference is otherwise
only validation state and a concrete next multi-source design can reuse the
marker/reference split without changing either variant.

### Additive/disjoint follower relation — eliminated

The strongest compatibility form would keep existing `LiveRowMajor` for the
one source input and the write, then add an input/read-only
`LiveRowMajorFollower { inner_axis }` for other reads. Exactly-one source and
role restrictions make it correct and disjoint, and current one-source bytes
would remain exact. It is eliminated under Tiler's pre-production replacement
rule: the same variant would mean “source” on one role and “consumer” on
another, leaving permanent role-contextual semantics solely to retain internal
bytes. The marker survivor has the same two-variant/runtime cost while making
the source explicit, and the total replacement has one local relation. Exact
byte continuity must still be captured as migration evidence; it is not a
reason to retain the weaker public model.

An additive source-bearing follower with `source_access` is worse still: it
keeps old `LiveRowMajor`'s mixed role semantics while adding a referenced
consumer. It is not the hybrid survivor above, which retires `0x09`, gives the
source and consumer disjoint explicit variants, and has no duplicate source
spelling. Allowing explicit self on the additive follower would duplicate the
old source spelling. Both additive forms are eliminated.

### Region-level binding — eliminated as dominated

`IndexRegion { live_row_major_source: Option<AccessOrdinal>, .. }` can state the
same one-source relation correctly. Verification would require `Some` exactly
when live maps exist, validate the named input/read/source axis, and require
every pointwise access including the write to carry `LiveRowMajor`; otherwise it
inherits the current mixed-map wrong code and is eliminated as incorrect before
ranking. Even in that strongest fail-closed form it is worse than the marker on
every relevant dimension: it adds an optional field and invalid
dangling-coordinate states to every `IndexRegion` and changes all 12 actual
construction sites under `crates/` — one IR builder construction, eight
physical-compiler constructions, and three compiler integration fixtures —
including the static ones. The exact source command
`rg -n '\bIndexRegion\s*\{' crates --glob '*.rs'` returns 13 lines: the public
struct definition plus those 12 literals. The option also keeps `LiveRowMajor`
contextual and has the same one-domain ceiling. Its only benefit is direct
lookup; the bounded source scan already exists and both the region-level and
marker forms must scan consumers to validate them. It leaves the frontier.

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
topology are present and bounded. Exact current Rust layout is measured above;
canonical byte pins remain implementation evidence because no test currently
pins them. A production prototype would merely implement an unaccepted public
surface, so no research ticket is required before decision. A measured layout
or implementation-defect difference can reverse the recommendation among the
three correct survivors without hiding a fourth authority or blocking Tom's
choice.

## Pareto frontier

| Survivor | Correctness / strictness | Maintenance / compatibility | Tiler host runtime / memory | Identity / public / unsupported consequence |
| --- | --- | --- | --- | --- |
| Unique source marker | Exact one structural marker, complete live relation on every pointwise access, plus decoded-identity compiler proof; contextual consumers | No repeated handles; two variants; public one-domain ceiling | Linear scan; 208/8 map and 232/8 access; no per-consumer ordinal; failure diagnostic 16/4; unchanged kernel runtime | Source `0x0A`, narrowed consumer `0x09`; five rules; current all-`0x09` values become `Missing`; valid live values migrate; static bytes/schema versions stay |
| Total local replacement | Exact checked source-bearing relation on every pointwise access plus decoded-identity compiler proof; self-contained | One variant and one match arm; repeated dangling/inconsistent/self handles; explicit future association seam | Linear validation; 208/8 map and 232/8 access; nine canonical bytes per live map; failure diagnostic 16/4; unchanged kernel runtime | Replacement `0x0A`, old `0x09` retired; seven rules; all valid live values migrate; static bytes/schema versions stay; artifact row unchanged |
| Explicit source + referenced consumers | Exact marker plus range-checked local consumer references and decoded-identity compiler proof; bad source self-handle unrepresentable | Two disjoint variants; local consumer meaning; public multi-source association seam while current multiple markers refuse | Linear validation; 208/8 map and 232/8 access; five source bytes plus nine per consumer; failure diagnostic 16/4; unchanged kernel runtime | Source `0x0A`, consumer `0x0B`, old `0x09` retired; seven rules; all valid live values migrate; static bytes/schema versions stay; artifact row unchanged |

None dominates. The marker minimizes canonical state and representable bad
handles, but its consumers are contextual and its public shape cannot identify
different sources later. Total-local minimizes variant count and makes every
map self-contained, but represents a redundant and potentially wrong source
self-reference. The hybrid spends a second variant and consumer handles to make
source and consumer jobs explicit, saves the source's redundant ordinal, and
leaves a multi-source association seam without authorizing multiple live loops
today.

The recommendation is the hybrid. ADR 0046 makes each logical access relation,
not a region side field or role convention, the owner of tensor-coordinate
meaning. The hybrid follows that boundary for consumers while making source
declaration structurally explicit and canonical rather than a
handle-equals-own-position special case. Its strongest counterargument is
total-local's one-variant grammar; the marker remains the lower-state answer
for the current exactly-one-loop population.

## One decision question for Tom

Should Tiler accept the **explicit-source/referenced-consumer complete
replacement** (recommended), with `LiveRowMajorSource { inner_axis: Axis }`,
referenced `LiveRowMajor { inner_axis: Axis, source_access: AccessOrdinal }`,
fresh tags `0x0A`/`0x0B`, and the seven exact source rules; accept the
**complete total local replacement**
`LiveRowMajor { inner_axis: Axis, source_access: AccessOrdinal }`,
with the source access self-referencing, every pointwise access carrying the
relation and naming it, and the seven
exact `LiveRowMajorSourceRule`s above; accept the
**unique marker** alternative
`LiveRowMajorSource { inner_axis: Axis }` plus consumer
`LiveRowMajor { inner_axis: Axis }`, every pointwise access carrying one of
those relations, plus its five exact source rules; or reject all three and
retain the typed schedule-stage deferral after the independent mixed-map
correctness repair?

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
  write, wrong mode, inconsistent axes, and an access without the selected live
  relation. Independently change one read and then the final write to
  `LinearIdentity`; each must produce
  `LiveRowMajorSourceRule::ConsumerMissingRelation { access }` and stable
  `live-row-major-source-consumer-missing-relation`, never a broad access or
  refinement failure. Quote the exact dedicated stable rules and payload
  coordinates. For the total replacement separately perturb an out-of-range
  source, zero and two self-references, and one live consumer's source handle; require
  `live-row-major-source-reference-out-of-range`,
  `live-row-major-source-missing`, `live-row-major-source-multiple`, and
  `live-row-major-source-inconsistent-reference` independently rather than one
  broad access error. For the hybrid, independently perturb an out-of-range
  consumer reference, remove and duplicate the marker, point one consumer at
  itself and then at another in-range non-marker, and alter only one consumer's
  axis. Require the same range/missing/multiple/inconsistent/axis rules in the
  stated precedence, while proving no source self-reference state can be
  constructed.
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
  value after implementation and prove static schedules byte-identical.
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
  -> this decision (awaiting Tom)
  -> admit-symbolic-extents-through-schedule-formation (blocked implementation; also waits on P0 below)
  -> associate-live-extent-operands-with-symbolic-semantic-interface-axes (todo)
  -> deliver-an-artifact-family-from-a-symbolic-region (todo, also has its existing live-payload dependency)

refuse-mixed-pointwise-live-row-major-access-relations-before-lowering (P0 todo)
  -> admit-symbolic-extents-through-schedule-formation
```

The documentation repair is related and nonblocking; it fixes false records but
supplies no runtime authority. The P0 is an unresolved correctness prerequisite,
not an unresolved source authority: it can land its current-surface fail-closed
repair before Tom decides, and the two incoming implementation edges are
acyclic.

Only Tom accepts one exact surface. On acceptance, record who/date/venue/relay
provenance here, leave implementation to
[`admit-symbolic-extents-through-schedule-formation`](admit-symbolic-extents-through-schedule-formation.md),
and preserve artifact association as its downstream hard edge. On rejection,
record the rejected premise and keep the current typed refusal; do not mark the
capability implemented or close blocked dependents by inertia.
