---
id: decide-the-artifact-physical-selection-provenance-surface
title: Decide the artifact physical-selection provenance surface
status: awaiting-decision
priority: p1
dependencies: [disclose-the-physical-provider-environment-a-compilation-was-offered, publish-occurrence-bound-selected-physical-implementation-evidence, replace-flat-selected-lowering-capability-keys-with-structured-subjects]
related: [package-selected-physical-implementation-provenance-in-artifact-identity]
scopes: []
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, identity, schema, backend-providers, provenance, artifact]
---
## Decision required

The three accepted dependencies uniquely fix the **subject** that must cross the
compiler/artifact boundary: a separate per-variant run of whole region-occurrence
identity bytes, whole implementation-proposal identity bytes,
`ProviderIdentity`, and the closed proposal-kind code. They do not fix the
artifact crate's public Rust spelling, insertion topology, error vocabulary, or
wire tags. The live artifact API cannot carry the run without adding one of
those consequential public surfaces. ADR 0075 therefore stops
[`package-selected-physical-implementation-provenance-in-artifact-identity`](package-selected-physical-implementation-provenance-in-artifact-identity.md)
until Tom accepts the exact dominant surface below.

This ticket owns only that decision. It changes no production code, contract,
identity, schema, or pin. Only Tom moves it from `awaiting-decision` to `done`.
Both this prerequisite and its claimed implementation ticket declare the exact
`.ticketsplease/decision-queue.md` path; `project/tickets` owns their ticket
files. No configured scope maps `.ticketsplease/**`, so the explicit path is
the repository's available guard metadata for the queue edit rather than a
fabricated production or contract scope.

## Exact-base Fact audit — 2026-08-16

Base: `4e02be6f4aed72209bb15019c43c247abf530e17`.

Read in full before this packet: repository `AGENTS.md`; this ticket's
implementation ticket and all three dependencies; accepted ADRs 0072, 0075,
and 0090; `docs/artifact-abi.md`; `docs/work-tracking.md`. The source audit
followed the complete construction, validation, identity, envelope projection,
encoding, decoding, read-view, equality, limit, refusal, production assembly,
compiler projection, and canonical-selection paths in:

- `crates/tiler-artifact/src/program/{builder,error,keys,model,mod}.rs`;
- `crates/tiler-artifact/src/program/codec/{budget,decode,encode,error,model,validate,view}.rs`;
- the artifact program and codec correctness-bearing tests;
- `crates/tiler-build/src/plan_artifact.rs`;
- `crates/tiler-compiler/src/{cover,frontier,program,region,request,selection,session}.rs`;
- `crates/tiler-compiler/src/pipeline/planning.rs`.

### Verdicts

1. **Verified — the artifact construction authority is one untyped offered
   set.** `CompilationEnvironment` stores one `available` vector;
   `CompilationEnvironment::new(providers)` sorts and deduplicates it;
   `available()` and private `offers()` expose only that one role. Searchable
   anchors: `pub struct CompilationEnvironment`, `pub fn available`, and
   `fn offers` in `builder.rs`.

2. **Verified — production fills that set with lowering providers only.**
   `assemble_plan_artifact` passes only
   `compilation.offered_lowering_providers()` to the one-argument constructor
   and forwards only `plan.selected_capabilities()` through
   `builder.select_provider(SelectedProvider { ... })`. Searchable anchors:
   `CompilationEnvironment::new(` and `for selected in
   plan.selected_capabilities()` in `plan_artifact.rs`.

3. **Verified — the missing physical source already exists separately.**
   `Compilation::offered_physical_providers()` returns the complete frozen
   offered physical set. `PlanAlternative::selected_physical_providers()`
   returns the selected run. Searchable anchors are those two symbols in
   `session.rs`.

4. **Verified — each selected compiler row already exposes every accepted
   subject whole.** `SelectedImplementation` exposes
   `region_occurrence_identity()`, `implementation_proposal_identity()`,
   `provider()`, and `proposal_kind()`. Its constructor remains private.
   `assemble_plan` proves a one-to-one occurrence binding and sorts selections
   by the whole occurrence bytes before a `SelectedPlan` exists. Searchable
   anchors: `pub struct SelectedImplementation`, `fn assemble_plan`,
   `duplicate-selection`, and `ordered.sort_by`.

5. **Verified — the compiler's closed source vocabulary has three selectable
   kinds and one rejected reservation.** `PhysicalProposalKind` assigns
   `ScheduledKernel = 1`, `KernelSubprogram = 2`, `OpaqueCall = 3`, and
   `View = 4`; `ProposalBody::View` is rejected and cannot reach a selected
   implementation. The public projection intentionally exposes stable text
   (`scheduled-kernel`, `kernel-subprogram`, `opaque-call`) rather than that
   crate-private enum. Searchable anchors: `enum PhysicalProposalKind`,
   `const fn tag`, `const fn name`, and `ProposalBody::View` in `frontier.rs`.

6. **Verified — artifact storage and wire have no physical-selection row.**
   `VariantSpec`, `VariantData`, `VariantRow`, `VariantRef`, and
   `DecodedVariant` carry profile, rules, deferred predicates, route
   requirements, entries, order, and dependencies, but no occurrence/proposal
   provenance. Artifact-global `SelectedProvider` is a lowering capability row
   and its own `PROVIDER_KEY_DOMAIN` is `v3`. Searchable anchors:
   `pub struct VariantSpec`, `pub(crate) struct VariantRow`,
   `pub struct SelectedProvider`, and `pub struct DecodedVariant`.

7. **Verified — the live version owners are artifact `v18`, selected lowering
   provider key `v3`, and manifest 18.0.** Searchable anchors:
   `ARTIFACT_DOMAIN`, `PROVIDER_KEY_DOMAIN`, and `MANIFEST_SCHEMA`. The
   component schemas are program 1.0, ABI expression 1.0, guard/routing 1.0,
   and target requirement 3.0 at `ArtifactSchema::GOVERNED`.

8. **Verified — existing artifact vocabulary cannot carry the accepted subject
   without a new public choice.** `tiler-build` is a separate crate, so it can
   only pass public artifact types and methods. No current public record has
   fields for either opaque identity, no public proposal-kind type exists, and
   neither `push_variant` nor a `VariantId` mutator accepts the run.

9. **Imprecise ticket premise repaired — “add a run” did not determine the
   Rust or wire surface.** Several correct surfaces preserve exactly the
   accepted subject. Choosing one would decide a consequential public type,
   constructor, accessor, and error vocabulary under ADR 0075. This is not an
   implementation detail and is the reason the implementation ticket is now
   dependency-blocked rather than silently completed with one worker's taste.

10. **Verified — identity and manifest variant grammars have one exact common
    insertion seam.** `program::model::push_variant` and
    `codec::encode::encode_variants` both write target-profile key/descriptor
    and feasibility-rule key/fixed `u32` revision immediately before the
    deferred-predicate run; `codec::decode::parse_variants` reads the same
    order. Searchable anchors: `fn push_variant`, `fn encode_variants`, and
    `fn parse_variants`.

11. **Verified Fact plus bounded Inference — envelope projection owns cloned
    variant rows.** `ArtifactEnvelope::project(&ArtifactProgramData)` walks
    borrowed `VariantData`, clones every owned field needed by `VariantRow`, and
    retains the verified source while the envelope is encoded. Searchable
    anchors: `pub(crate) fn project` and `variants.push(VariantRow` in codec
    `model.rs`. **Inference:** adding the same owned physical row to both data
    structures makes `Box<[u8]>` clone its bytes and `Arc<[u8]>` share them;
    section projection's existing “borrowed content” rationale confirms peak
    publication memory is an explicit design concern rather than an invented
    optimization target.

12. **Verified — twelve is the exact current compiler-production selection
    ceiling, not an artifact-model ceiling.**
    The governed request's `regions` budget is 12;
    `CoverAssembly::from_plan` requires one nonempty scheduled-stage run per
    selection; and the program's `region-budget` check refuses more than twelve
    flattened stages before `build_alternative_for_origin` can retain a public
    alternative. Thus selected rows are no more numerous than the bounded
    program stages on the current ordinary compiler path. This does not bound
    larger retained covers that fail assembly, and it does not narrow the
    artifact crate's independently supported direct-construction population.
    Searchable anchors: `regions: 12`, `unlowerable-opaque-body`,
    `cover-region-undispatched`, `region-budget`, and
    `build_alternative_for_origin`.

13. **Verified — the two topology migration populations differ.** The exact
    broader anchored census has 13 `VariantSpec` literals. The positional
    alternative reaches 65 `.push_variant(` call expressions across 13 files.
    The root `crates` text search has 11 `VariantSpec {` hits, but includes the
    struct declaration, function signatures, and rustdoc and is not a literal
    population. The exact commands and outputs are recorded below.

14. **Imprecise packet derivation repaired — selected-row capacity must follow
    the artifact's executable-entry population and association rule.** Current
    `ArtifactProgramBuilder::push_variant` requires `spec.entries.len()` to
    equal `program.stages().len()` and then bounds that shared quantity by
    `MAX_VARIANT_ENTRIES = 4_096`; the shared IR independently admits
    `MAX_PROGRAM_STAGES = 4_096`. Direct artifact construction is supported; it
    is not restricted to the compiler request whose current region budget is
    twelve. The accepted compiler relation is selected cover regions no more
    numerous than their nonempty flattened scheduled stages, so the artifact
    rule is `selected physical rows <= executable entries`. The earlier packet
    hard-coded twelve but omitted that relation, allowing a one-entry direct
    artifact to state twelve unrelated selections while refusing a structurally
    consistent thirteen-entry/thirteen-selection artifact. Searchable anchors:
    `if stages != spec.entries.len()`, `limit(stages,
    MAX_VARIANT_ENTRIES`, and `pub const MAX_VARIANT_ENTRIES` in artifact
    `builder.rs` and `mod.rs`; `pub const MAX_PROGRAM_STAGES` in shared-IR
    `program/mod.rs`; and `cursor.vec(MAX_VARIANT_ENTRIES` in codec `decode.rs`
    proves the wire admits the same entry population.

15. **False packet failure repaired — a decoded 64 MiB physical-provenance
    overflow is unreachable.** `read_header` checks `manifest_bytes` against
    `MAX_MANIFEST_BYTES = 64 MiB` before `parse_manifest` receives the borrowed
    manifest. The physical run is a strict, positively framed subset of that
    manifest, so neither its complete contribution nor one identity inside it
    can exceed 64 MiB in a stream that reaches variant parsing. A decoder-local
    `SelectedPhysicalProvenanceBytes` or
    `PhysicalSelectionIdentityBytes` limit, error, and boundary test would
    therefore claim a refusal the decoder cannot emit. Construction is
    different: a direct caller can hand the builder owned rows before any
    manifest exists. The same shared run is a strict subset of canonical
    artifact identity, whose reachable whole-build
    `ArtifactDiagnostic::IdentityLimit` uses
    `MAX_ARTIFACT_IDENTITY_BYTES = 64 MiB`; an early construction-time lower
    bound can truthfully refuse only against that existing global limit.
    Searchable anchors: `let manifest_bytes = cursor.count(MAX_MANIFEST_BYTES`,
    `pub(super) const MAX_MANIFEST_BYTES`, `fn encode_identity`, and
    `if bytes.len() > MAX_ARTIFACT_IDENTITY_BYTES`.

### Reproducing commands, run at the base above

```sh
git merge-base HEAD 4e02be6f4aed72209bb15019c43c247abf530e17
git diff --quiet 4e02be6f4aed72209bb15019c43c247abf530e17..HEAD -- crates prototypes spikes
rg -n 'pub struct SelectedImplementation|region_occurrence_identity|implementation_proposal_identity|proposal_kind\(' crates/tiler-compiler/src/session.rs
rg -n 'CompilationEnvironment::new\(|select_provider\(|selected_physical_providers\(' crates/tiler-build/src/plan_artifact.rs crates/tiler-artifact/src/program/builder.rs crates/tiler-compiler/src/session.rs
rg -n '^\s*VariantSpec \{' crates prototypes spikes --glob '*.rs'
rg -n 'VariantSpec \{' crates --glob '*.rs'
rg -n '\.push_variant\(' crates prototypes spikes --glob '*.rs'
rg -n 'CompilationEnvironment::new\(' crates prototypes spikes --glob '*.rs' | wc -l
rg -l 'CompilationEnvironment::new\(' crates prototypes spikes --glob '*.rs' | wc -l
rg -n 'pub fn available\(|\.available\(\)' crates/tiler-artifact crates/tiler-build prototypes spikes --glob '*.rs'
rg -n '\bSelectedProvider\b' crates prototypes spikes --glob '*.rs' | wc -l
rg -l '\bSelectedProvider\b' crates prototypes spikes --glob '*.rs' | wc -l
rg -n '\.select_provider\(' crates prototypes spikes --glob '*.rs' | wc -l
rg -l '\.select_provider\(' crates prototypes spikes --glob '*.rs' | wc -l
rg -n '\.selected_providers\(' crates prototypes spikes --glob '*.rs' | wc -l
rg -l '\.selected_providers\(' crates prototypes spikes --glob '*.rs' | wc -l
rg -n '^pub\(crate\) const (ARTIFACT_DOMAIN|PROVIDER_KEY_DOMAIN|STAGE_KEY_DOMAIN)|^pub\(super\) const MANIFEST_SCHEMA' crates/tiler-artifact/src/program --glob '*.rs'
rg -n 'fn push_variant|fn encode_variants|fn parse_variants|pub\(crate\) fn project|variants\.push\(VariantRow' crates/tiler-artifact/src/program --glob '*.rs'
rg -n 'regions: 12|unlowerable-opaque-body|cover-region-undispatched|region-budget|build_alternative_for_origin' crates/tiler-compiler/src --glob '*.rs'
rg -n 'if stages != spec\.entries\.len\(\)|limit\(stages, MAX_VARIANT_ENTRIES|pub const MAX_VARIANT_ENTRIES|cursor\.vec\(MAX_VARIANT_ENTRIES' crates/tiler-artifact/src/program/{builder.rs,mod.rs,codec/decode.rs}
rg -n 'pub const MAX_PROGRAM_STAGES' crates/tiler-ir/src/program/mod.rs
rg -n 'manifest_bytes = cursor\.count\(MAX_MANIFEST_BYTES|const MAX_MANIFEST_BYTES|fn encode_identity|bytes\.len\(\) > MAX_ARTIFACT_IDENTITY_BYTES' crates/tiler-artifact/src/program/{model.rs,codec/decode.rs,codec/encode.rs}
```

Observed census: the broad anchored command finds 13 exact `VariantSpec`
literals across `crates prototypes spikes`; the deliberately different root
`crates` textual command finds 11 hits, but that population includes the struct
declaration, function signatures, and rustdoc examples and is not a literal
count. The positional alternative reaches 65 `push_variant` call expressions
across 13 source files; 69
`CompilationEnvironment::new` calls across 15 source files; no artifact
environment `available()` consumer beyond its definition; 50
`SelectedProvider` references across 17 files; 61 `select_provider` calls
across 13 files; and eight `selected_providers()` calls across three files.
The artifact-limit command finds the 4,096 constant, builder entry/stage
equality followed by that limit, and both entry-table decoder consumers; the
shared-IR command finds its equal 4,096 verified-program stage ceiling. The
manifest/identity command finds the pre-parse 64 MiB manifest admission, the
equal 64 MiB identity authority, and the reachable final identity refusal. The
migration is real but bounded and entirely in-tree at this base.

## Exact proposed surface

This packet resolves every public and wire choice. An implementation that
varies one of them has left the proposed surface and must stop again.

### 1. Role-separated construction authority and lowering names

Replace the one-role constructor exactly with:

```rust
pub fn new(
    offered_lowering_providers: impl IntoIterator<Item = ProviderIdentity>,
    offered_physical_providers: impl IntoIterator<Item = ProviderIdentity>,
) -> Result<Self, ArtifactBuildError>;

pub fn offered_lowering_providers(&self) -> &[ProviderIdentity];
pub fn offered_physical_providers(&self) -> &[ProviderIdentity];
```

The implementation stores private `offered_lowering_providers` and
`offered_physical_providers` vectors, canonicalizes each independently by
`(namespace, name, revision)`, deduplicates only within one role, and permits
the same `ProviderIdentity` in both roles. Each iterator's collected input
length is checked against its own 4,096 limit **before** sorting/deduplication,
preserving the current anti-amplification rule; repeated input identities
therefore spend input capacity but collapse in the returned canonical slice.
There is no `Default`, one-argument
overload, union, `available()` alias, payload/backend/profile inference, or
ambient registry lookup. Both sets remain construction-only and are discarded
from `VerifiedArtifactProgram` and `ArtifactEnvelope`.

Make the existing lowering surface say its role:

- `SelectedProvider` becomes `SelectedLoweringProvider`;
- `select_provider` becomes `select_lowering_provider`;
- built and decoded `selected_providers()` accessors become
  `selected_lowering_providers()`;
- `ArtifactEntityKind::Provider`, `OrderedSubject::Provider`, and
  `CodecLimitKind::SelectedProviders` become the role-exact
  `LoweringProvider`, `SelectedLoweringProvider`, and
  `SelectedLoweringProviders` variants respectively;
- `ArtifactDiagnostic::MissingSelectedProvider` becomes
  `MissingSelectedLoweringProvider`, with rule
  `missing-selected-lowering-provider`;
- `MAX_SELECTED_PROVIDERS` becomes
  `MAX_SELECTED_LOWERING_PROVIDERS` (value unchanged at 256);
- `MAX_ENVIRONMENT_PROVIDERS` becomes two independent constants,
  `MAX_OFFERED_LOWERING_PROVIDERS` and
  `MAX_OFFERED_PHYSICAL_PROVIDERS`, each 4,096.

Do not retain deprecated aliases: they would leave the exact cross-role API
ambiguous after the two-argument constructor already makes this a breaking
change. The selected-lowering row grammar and `PROVIDER_KEY_DOMAIN` remain
unchanged at v3.

### 2. Exact owned physical row and opaque identities

Add these public artifact-owned values:

```rust
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PhysicalRegionOccurrenceIdentity(Arc<[u8]>);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PhysicalImplementationProposalIdentity(Arc<[u8]>);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum PhysicalProposalKind {
    ScheduledKernel,
    KernelSubprogram,
    OpaqueCall,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SelectedPhysicalImplementation {
    pub region_occurrence: PhysicalRegionOccurrenceIdentity,
    pub implementation_proposal: PhysicalImplementationProposalIdentity,
    pub provider: ProviderIdentity,
    pub proposal_kind: PhysicalProposalKind,
}
```

Each identity gets the existing opaque-identity shape:

```rust
pub fn from_bytes(value: impl AsRef<[u8]>) -> Result<Self, ArtifactBuildError>;
pub fn as_bytes(&self) -> &[u8];
```

The private `Arc<[u8]>` is required and `from_bytes` performs
`Arc::from(value.as_ref())`. Raw public `Vec<u8>` fields are rejected: they
permit occurrence/proposal swaps, bypass validation, and make immutable
identity storage retain spare capacity. `Box<[u8]>` is also eliminated after
following the actual publication path: `ArtifactEnvelope::project` borrows the
verified `ArtifactProgramData` and builds an owned `VariantRow`, so it must
clone every row while the verified value remains live. A boxed identity would
deep-copy both byte runs before manifest encoding and transiently double the
largest new payload; `Arc` shares those immutable bytes with one atomic
increment/decrement per wrapper clone. The bytes can be tens of MiB and are
encoded linearly, so avoiding that copy dominates the refcount cost on the
actual publication path without changing the
public API, equality, ordering, or schema. A borrowing envelope is not a
survivor: decode must own the same envelope shape and adding lifetimes would
widen every codec consumer merely to avoid a clone `Arc` already removes.

The artifact does not parse or re-derive either identity. `ArtifactKeyKind` gains
`PhysicalRegionOccurrence` and `PhysicalImplementationProposal`, so empty and
overlong values reuse exact `EmptyKey` / `KeyTooLong` causes.

Both wrappers use the public per-value admission ceiling
`MAX_PHYSICAL_SELECTION_IDENTITY_BYTES = MAX_ARTIFACT_IDENTITY_BYTES` (64 MiB).
This is not a claim about the compiler's minting bound: neither compiler type
currently publishes one. It prevents an individual received value from
escaping the artifact's existing complete-identity resource class at the direct
construction boundary. Empty and oversize direct construction reach the exact
`EmptyKey` and `KeyTooLong` errors above. There is no corresponding decoder
limit kind or oversize-decode test: `read_header` has already bounded the whole
manifest to 64 MiB before variant parsing, and either framed identity is a
strict subset of it. Decode still constructs the same wrapper so one model
constructor owns the invariant; empty bytes are reachable and surface as
`ArtifactCodecFailure::Invalid`, while the oversize arm is defensive but
unreachable from an admitted manifest.

`PhysicalProposalKind` is `#[non_exhaustive]` because an additive future
artifact proposal kind must not source-break external readers that classify
provenance; that source-compatibility seam does not widen the present closed
three-variant value population. Internally it declares exactly
`const ALL: [Self; variant_count::<Self>()] = [ScheduledKernel,
KernelSubprogram, OpaqueCall]`. `tag(self)` uses an exhaustive enum match;
`from_tag(u8)` maps `0x01..=0x03` to those variants and has one explicit
unknown-`u8` catch-all returning `None`. The tag test derives its population
from `ALL`, asserts `ALL.len() == variant_count::<PhysicalProposalKind>()`, tag
injectivity, and `from_tag(kind.tag()) == Some(kind)` for every member.

The artifact-owned manifest and canonical-row tags are
`ScheduledKernel = 0x01`, `KernelSubprogram = 0x02`, and
`OpaqueCall = 0x03`. `0x04` is reserved specifically for future `View`: the
compiler currently has four `PhysicalProposalKind` values, but rejects
`ProposalBody::View` before selection, so no currently constructible selected
row may carry it. It is never reusable for another meaning. Admitting `View`
later requires a reviewed compiler/artifact vocabulary ticket, adding the
artifact variant at `0x04`, and another owning artifact/manifest version step;
until then the decoder's unknown-tag path rejects it. Tests exercise `0x00`,
reserved `0x04`, and `0xff` through the explicit catch-all. Unknown tags fail
as `UnknownTag { subject:
TagSubject::PhysicalProposalKind, ... }`, publicly `Malformed`, after schema
admission. There is no public string constructor and no serialized presentation
text.

`tiler-build` translates the compiler's three exact stable strings to the
three enum variants. Any other string returns the new
`PlanArtifactError::UnsupportedPhysicalProposalKind { kind: &'static str }`;
it is never defaulted to a known kind. Because the compiler deliberately keeps
its enum private, this cross-crate translation cannot be compile-time
exhaustive; a new compiler code is deliberately a typed fail-closed packaging
error until this mapping and schema are reviewed.

### 3. Association, multiplicity, order, and limits

Each variant owns exactly one non-empty `Vec<SelectedPhysicalImplementation>`.
`MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS` is defined exactly as
`MAX_VARIANT_ENTRIES` and therefore has value 4,096. This is one artifact-owned
capacity authority, not two equal literals. The separate public name and limit
kind tell a caller which received collection was refused; the value follows the
entry table because the selected run is structurally bounded by that table.

Construction requires
`selected_physical_implementations.len() <= spec.entries.len()`. The existing
`push_variant` rule has already proved `spec.entries.len() ==
program.stages().len()` and bounded that shared count by
`MAX_VARIANT_ENTRIES` before this check runs. Equality is deliberately not
required: one selected cover region may flatten to several executable stages,
so fewer selections than entries preserves the accepted association. More
selections than entries cannot be the occurrence-to-nonempty-stage population
the accepted compiler boundary describes and is refused as the exact new
`ArtifactBuildError::PhysicalSelectionCardinality { selected, entries }`.

The current ordinary compiler path remains tighter:
`DeterministicBudgets::governed().regions` is 12;
`CoverAssembly::from_plan` requires every selected cover region to contribute a
non-empty scheduled-stage run; it flattens those runs into the assembled
scheduled regions; and `build_cover_kernel_program_with_lowering` refuses a
program whose stage count exceeds that same `regions` budget before a public
`PlanAlternative` exists. Production therefore proves
`selected physical rows <= scheduled stages == artifact entries <= 12` before
calling the artifact builder. Twelve is producer evidence, not authority to
reject a direct artifact whose verified program and entry table legally contain
thirteen or more stages. A future compiler budget can grow without an artifact
public-limit change until it reaches the existing entry ceiling.

This does not claim every retained pre-assembly cover has at most twelve
regions: duplication and the bounded candidate search can retain larger covers,
and opaque-call selections carry no scheduled stage. Those plans are refused
before the current production artifact boundary and remain unsupported. The
model and codec expose separate
`ArtifactLimitKind::SelectedPhysicalImplementations` and
`CodecLimitKind::SelectedPhysicalImplementations` rows. Offered lowering and
physical counts are bounded independently; no sum-of-both bound exists.

Row count alone does not bound owned identity bytes, but this surface adds no
second byte budget. A private checked sizing helper computes the **exact byte
contribution the complete physical-selection population makes to canonical
artifact identity**: every variant's one-byte run tag, `u64` count framing,
plus every `u64`-length-framed `PHYSICAL_SELECTION_KEY_DOMAIN` row key. The
shared run encoder uses that same helper, so sizing and writing have one byte
definition.

The builder retains that exact physical contribution as a private running
total. In `push_variant`, preserve the
existing semantic-subject, variant-count, entry-cardinality, and stage-count
refusal precedence. Immediately after the existing
`limit(stages, MAX_VARIANT_ENTRIES, ...)` and before
`program_abi_use_sites`/`adopt_abi`, validate the physical run in this order:
nonempty; absolute row count against
`MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS`; row count no greater than
`spec.entries.len()`; prospective canonical-artifact identity lower bound;
duplicate versus descending occurrence order; then offered-physical membership
in row order. No builder mutation has occurred at that point.

The lower bound is the prospective exact physical contribution plus one byte,
because the run is a strict subset of canonical artifact identity and the
complete identity necessarily contains nonphysical bytes. Checked arithmetic
that cannot represent the exact physical sum records
`MAX_ARTIFACT_IDENTITY_BYTES + 1`, which is still a truthful proved minimum. A
minimum above the existing 64 MiB identity limit returns the exact new
`ArtifactBuildError::IdentityLowerBound { minimum_bytes, limit }`. This is not a
new limit: it is an insertion-time proof that the already-governed final
identity cannot pass its reachable `ArtifactDiagnostic::IdentityLimit`. The
consumed `VariantSpec` is dropped and the builder, including its running total
and prior variants, remains unchanged. The vector and new total are committed
only with the successfully accepted `VariantData`.

This early proof dominates waiting for `build`: it admits every artifact the
final identity limit could admit and prevents the builder from retaining more
canonical physical-run bytes than the existing 64 MiB whole-identity limit can
possibly publish. Removing it would shrink the public error vocabulary by one
variant but allow up to the product of the variant, entry, and per-value limits
to remain owned until whole-build identity derivation, with no additional
supported artifact. Evidence that would reverse the early check is a builder
storage design that never retains caller-owned physical rows before deriving
the complete identity, or an accepted fallible streaming identity encoder that
enforces the same global bound without accumulating those rows.

The manifest keeps the physical run at its post-feasibility/pre-deferred
position. `read_header` refuses `manifest_bytes > MAX_MANIFEST_BYTES` before
`parse_manifest`; the physical run and both identities are strict subsets of
those at-most-64-MiB borrowed bytes. Decode therefore needs only the reachable
4,096 count check before reserving the row vector, the row's framing/model/order
checks, and the wrapper copies. It then parses deferred predicates, route
requirements, and entries in wire order. Immediately after the entry vector is
complete — the first point the relation is decidable — it rejects `rows > entries` as
`ArtifactCodecError::ModelRule { cause:
PhysicalSelectionCardinality { selected, entries } }` before deferred-entry
references, execution order, or dependencies are checked. Malformed physical,
deferred, route, or entry bytes encountered earlier in the stream retain their
existing earlier precedence. The borrowed manifest is at most 64 MiB, and all
copied physical raw identity bytes together are a strict subset of it and thus
also less than 64 MiB; decode peak for those two byte populations is therefore
less than 128 MiB, plus the already-bounded row/provider metadata. There is no
separate decoded physical-byte admission rule.
Any later cardinality refusal drops the bounded partial decoded model and
returns no product. Encoder budget validation repeats the reachable count and
`rows <= entries` checks before writing. The existing whole-manifest encoder
limit and whole-identity diagnostic remain the byte authorities.

The run is already compiler-canonical. Artifact construction preserves the
input order and requires occurrence bytes to be strictly ascending; it does
not sort a caller's statement. Equal occurrence bytes are
`DuplicatePhysicalRegionOccurrence`; descending bytes are
`NoncanonicalPhysicalRegionOccurrenceOrder { previous, current }`. Empty is
`EmptySelectedPhysicalImplementations`. A provider absent from the physical
offered set is `PhysicalProviderNotOffered { provider }`. Existing lowering
membership and duplication errors become
`LoweringProviderNotOffered { provider }` and
`DuplicateSelectedLoweringProvider { provider }` and consult only the lowering
set. The duplicate predicate remains equality of the complete
`SelectedLoweringProvider` row, not provider identity alone: one provider may
legally appear in distinct selected lowering-capability rows.

Provider, proposal, and proposal-kind repetition across **different**
occurrences is legal and preserved. Nothing is deduplicated by provider,
proposal, kind, complete row, iterator position, backend entry, or payload.
One occurrence may appear exactly once. The artifact cannot authenticate that a
direct low-level caller copied every compiler row; the required non-empty
atomic run validates shape, order, identity bounds, and offered
authority, while production `assemble_plan_artifact` is the producer authority
that forwards the compiler iterator without omission or reconstruction.

The exact insertion vocabulary is:

```rust
LoweringProviderNotOffered {
    provider: Box<ProviderIdentity>,
}
PhysicalProviderNotOffered {
    provider: Box<ProviderIdentity>,
}
DuplicateSelectedLoweringProvider {
    provider: Box<ProviderIdentity>,
}
EmptySelectedPhysicalImplementations
PhysicalSelectionCardinality {
    selected: usize,
    entries: usize,
}
IdentityLowerBound {
    minimum_bytes: usize,
    limit: usize,
}
DuplicatePhysicalRegionOccurrence {
    occurrence: Box<PhysicalRegionOccurrenceIdentity>,
}
NoncanonicalPhysicalRegionOccurrenceOrder {
    previous: Box<PhysicalRegionOccurrenceIdentity>,
    current: Box<PhysicalRegionOccurrenceIdentity>,
}
```

The structural counts use `ArtifactBuildError::StructuralLimit` with exact
resources `SelectedLoweringProviders`, `OfferedLoweringProviders`,
`OfferedPhysicalProviders`, or `SelectedPhysicalImplementations` as applicable.
The byte lower-bound proof uses `IdentityLowerBound`, not `StructuralLimit` and
not a new `ArtifactLimitKind`. Codec has no offered-set limit because offered
sets are never serialized.

Within that exact insertion point, deterministic physical refusal precedence
is run emptiness, absolute row count, row count versus entries, prospective
canonical-identity lower bound, duplicate versus descending occurrence order,
then offered-physical membership in row order, all before any ABI adoption or
builder mutation. The consumed spec and its owned
vector are dropped on refusal; no row, byte total, ABI node, delivery count, or
subject field is committed to the builder. The codec checks the fixed run tag,
nonzero bounded count, and, for each outer-framed row, domain, field framing,
identity constructors, provider grammar, kind tag, zero trailing bytes, and
duplicate-versus-descending occurrence order. There is no aggregate or
per-identity codec limit between framing and wrapper construction: both would
be unreachable after header admission. It pushes the completed row only after
all reachable row checks pass. After parsing the later entry vector it applies
the relational cardinality rule before any later cross-reference or ordering
check. Thus a refused row never partially mutates the decoded vector, and a
refused variant never reaches a decoded product.

### 4. Exact read surface and equality

Both `VariantRef` and `DecodedVariant` add exactly:

```rust
pub fn selected_physical_implementations(
    self,
) -> &'a [SelectedPhysicalImplementation];
```

The returned slice is in canonical occurrence order and preserves all rows.
There is no artifact-global physical-provider accessor and no association to an
entry or payload. `SelectedPhysicalImplementation`, both wrappers,
`PhysicalProposalKind`, `VariantData`, `VariantRow`, `ArtifactEnvelope`, and the
verified/decoded product paths participate in `Eq`/`PartialEq`; decoding
reconstructs the same owned row type rather than a second public view record.

### 5. Exact canonical identity and wire step

Add `PHYSICAL_SELECTION_KEY_DOMAIN =
b"tiler.artifact-program.physical-selection.v1\0"` and
`PHYSICAL_SELECTION_RUN_TAG: u8 = 0x01`. There is one row encoder and one run
encoder, both in artifact `program::model`; the identity and manifest call the
same run encoder rather than restating its fields.

`SelectedPhysicalImplementation::canonical_key()` destructures the public row
irrefutably and writes exactly this byte grammar, in order:

1. raw fixed `PHYSICAL_SELECTION_KEY_DOMAIN` bytes, including its NUL;
2. occurrence length as big-endian `u64`, then exactly those occurrence bytes;
3. proposal length as big-endian `u64`, then exactly those proposal bytes;
4. provider-namespace UTF-8 byte length as big-endian `u64`, then those bytes;
5. provider-name UTF-8 byte length as big-endian `u64`, then those bytes;
6. provider revision as a fixed-width big-endian `u32`, with **no** length
   prefix; and
7. the proposal-kind tag as one `u8`.

The revision is not length-framed. The four variable byte fields use the
existing `push_slice` grammar. There is no row terminator, string presentation,
iterator ordinal, entry, payload, backend, or padding byte.

`push_selected_physical_implementation_run(bytes, rows)` writes exactly:

1. `PHYSICAL_SELECTION_RUN_TAG` as one `u8`;
2. row count as big-endian `u64`; and
3. for each row in preserved strict occurrence order, its canonical-key length
   as big-endian `u64`, then the complete canonical-key bytes above.

The enclosing row-key frame is therefore present in **both** canonical artifact
identity and the manifest. The manifest embeds that key; it does not write the
six row fields a second way. Decode reads one outer length-framed key into a
bounded nested cursor, requires the raw row domain, then parses the two opaque
identity slices, two provider text slices, fixed `u32` revision, and one kind
tag and requires zero trailing key bytes. This is the only admitted byte stream.

In canonical identity, `program::model::push_variant` calls the shared run
encoder at the exact insertion point after
`variant.feasibility_rules.revision.to_be_bytes()` and before deriving/writing
the deferred-predicate run. In `codec::encode::encode_variants`, call the same
encoder after the manifest feasibility-rule revision `u32` and before its
deferred-predicate count. `codec::decode::parse_variants` parses it at that same
position, before `deferred`. `VariantData`, projected `VariantRow`, and decoded
`VariantRow` all store the same owned row type; projection clones row records
and only increments the two identity `Arc`s.

Step `ARTIFACT_DOMAIN` from `tiler.artifact-program.v18` to
`tiler.artifact-program.v19`. Step `MANIFEST_SCHEMA` from 18.0 to 19.0. Add
`TagSubject::PhysicalSelectionRun`; any tag but 0x01 is `UnknownTag`. The run is
unconditional and non-empty, so no required-feature key is added.

The major schema step is required because an 18.0 reader would otherwise read
the new tag/count as the deferred-predicate count and lose framing. Schema is
checked before component schemas or variants: a v18 reader rejects a v19
manifest as `UnsupportedManifestSchema { major: 19, minor: 0 }`, and the new
v19 reader rejects a v18 manifest as
`UnsupportedManifestSchema { major: 18, minor: 0 }`. No legacy branch or
defaulted empty run is added. The
artifact component schemas remain program 1.0, ABI expression 1.0,
guard/routing 1.0, and target requirement 3.0: this adds provenance to the
manifest but no executable program, expression, routing, or target-requirement
vocabulary.

Pin derivation is unique: `encode_identity` folds the shared physical run at the
position above under artifact domain v19; the encoded manifest carries that
resulting identity and the byte-identical shared run under schema 19.0; the
manifest digest is then derived from those exact manifest bytes and the
envelope digest/bytes from the existing envelope grammar. Expansion-cache
subjects that fold canonical artifact identity move from that one v19 identity.
Every checked-in artifact identity, manifest/envelope byte, digest, and cache
pin is recomputed from this chain on the implementation tree; no hand-written
alternate row encoder or pin formula is permitted.

Do **not** step `PROVIDER_KEY_DOMAIN` (the lowering row is unchanged),
`STAGE_KEY_DOMAIN`, payload key/content domains, manifest/envelope digest
domains, envelope format 1.0, canonical encoding profile 1.0, semantic,
schedule, structured-kernel, or kernel-program domains. Enumerate the new
physical-row domain in `crate::domains` and increase its typed domain census;
update every exhaustive `TagSubject`, `OrderedSubject`, `CodecLimitKind`,
`ArtifactKeyKind`, `ArtifactLimitKind`, `ArtifactBuildError`,
`ArtifactCodecError` classification/source/display, and public-export map
touched by the new rows. `ArtifactDiagnostic` gains no variant: its existing
`IdentityLimit` remains the whole-artifact check after every transactional
model rule. The existing lowering-only
`MissingSelectedProvider` variant is renamed as specified in section 1, so its
exhaustive consumers migrate spelling but the diagnostic population does not
grow.

### 6. Exact whole-build and codec failures

Add no `ArtifactDiagnostic`: the physical run is accepted or refused
transactionally with the variant, so whole-artifact verification never sees an
absent run. Builder wrapper construction rejects empty identity bytes with
`ArtifactBuildError::EmptyKey` and a row identity over 64 MiB with
`ArtifactBuildError::KeyTooLong`, naming the exact new `ArtifactKeyKind`.
`push_variant` count failures use `StructuralLimit` with the exact resource
above. A physical subset proving the complete identity must exceed its existing
global limit uses `IdentityLowerBound { minimum_bytes, limit }`. More physical
rows than entries uses
`PhysicalSelectionCardinality { selected, entries }`; other association and
offered-authority failures use the exact variants in section 3.

Decoder canonical order and duplicates use
`OrderedSubject::SelectedPhysicalImplementation` with the
existing `NonCanonicalOrder` and `DuplicateItem` codec errors; empty decoded
runs use `ModelRule { cause: EmptySelectedPhysicalImplementations }`; unknown
kind/run tags use `UnknownTag`. A wrong embedded row-key domain is the new
`BadPhysicalSelectionDomain`; bytes left in its bounded nested cursor are the
new `TrailingPhysicalSelectionKeyBytes { remaining }`; both are `Malformed`.
Oversize row counts use
`CodecLimitKind::SelectedPhysicalImplementations` and are `Limit`, checked
before reserving the refused vector. Do not add
`PhysicalSelectionIdentityBytes` or `SelectedPhysicalProvenanceBytes` codec
limit kinds: header admission makes both overflow populations unreachable. A
bounded run that outnumbers the later entry table is `ModelRule { cause:
PhysicalSelectionCardinality { selected, entries } }`, checked immediately
after entry parsing. Model-rule and order failures are publicly `Invalid`,
tag/domain/framing/provider-grammar failures `Malformed`, and reachable budget
failures `Limit`. The decoder validates all of them before returning a
`DecodedArtifact` and re-encoding must reproduce the exact manifest.

## Options considered and eliminated

### Status quo — eliminated

It silently omits selected physical authority from artifact identity. Two
plans with different admitted providers/proposals can share artifact bytes and
cache subjects. That is a wrong identity, not a compatibility option.

### Serialize offered sets or form a lowering/physical union — eliminated

Serialization makes unused providers move artifact/cache identity contrary to
ADR 0072. A union accepts cross-role authority and cannot prove which
responsibility was offered. Both are wrong.

### Infer/default from payload, backend, profile, entry, or iterator position — eliminated

None is the authority that admitted the implementation. Each can silently name
a different subject. A missing row/member must refuse.

### Artifact-global physical-provider set — eliminated

It loses occurrence-to-proposal association and multiplicity. Two occurrences
using one provider and two proposals become indistinguishable from one row.

### Raw byte vectors or parse/rederive compiler identities — eliminated

Raw vectors erase the two nominal roles and bypass boundary checks. Parsing
makes the artifact a second authority for compiler-private identity grammars.
Opaque owned wrappers carry the exact accepted bytes without either defect.

### Boxed identity storage — eliminated

`Box<[u8]>` preserves immutable nominal wrappers and avoids atomic reference
counts, but the live `ArtifactEnvelope::project(&ArtifactProgramData)` path must
own a projected row while the verified data remains live. Its row clone would
deep-copy both boxes immediately before manifest encoding, so peak physical
identity storage and copy time grow by the complete provenance payload. The
chosen `Arc<[u8]>` has identical public/schema/equality semantics, retains no
spare capacity, and replaces each deep copy with two atomic refcount bumps per
row. It is no worse on correctness, strictness, compatibility, or public
surface, and is strictly better on end-to-end publication time/RSS at the 64
MiB admitted edge. The two reference counters add a small pre-projection header
per allocation, but the actual build/encode lifecycle immediately avoids a
second allocation header plus a second copy of the entire identity, so that is
not a host-memory advantage for `Box` on the operation this surface exists to
perform. Therefore `Box` is not on the frontier.

Strongest counterargument to `Arc`: tiny synthetic rows pay atomic increment
and decrement cost that a deep `Box` clone may hide in noise, and an artifact
held indefinitely without projection carries the `Arc` control blocks without
receiving their clone benefit. Evidence that would reverse the storage choice
is an authorized build-only lifecycle that dominates publication, or an
accepted projection/codec redesign that can move each identity exactly once
through both build and decode without lifetimes or a second public row shape,
plus measurement showing lower peak RSS and time over the 4,096-row/64 MiB
edge.
The implementation evidence must measure/project a multi-MiB row and perturb
the envelope clone back to deep copies so the RSS/copy assertion is
load-bearing.

### Add physical-specific decoder byte limits — eliminated

The prior packet proposed per-identity and aggregate 64 MiB codec limits. No
admitted envelope can reach either refusal: `read_header` rejects a manifest
over the same 64 MiB ceiling before physical parsing, and each identity and the
complete run are strict framed subsets of that manifest. Keeping either limit
would enlarge the public codec vocabulary and require a boundary test whose
claimed error cannot be emitted. Decode instead relies on the existing
`ManifestBytes` header refusal, then the reachable row-count and relational
checks. Evidence that would reverse this elimination is an accepted format in
which physical bytes are external to the bounded manifest, or an independently
authorized physical budget strictly below the manifest budget. Neither exists.

### Wait until `build` to reject an identity that its physical subset already proves oversize — eliminated

This has the smallest builder error enum, but it allows a direct builder to
retain caller-owned physical rows whose exact canonical subset already exceeds
the existing whole-identity limit. It admits no artifact the early
`IdentityLowerBound` check refuses: the eventual `encode_identity` must reject
every such candidate. The early check therefore preserves correctness,
strictness, schema, and the supported population while bounding retained
unpublishable state; its cost is one public error and checked linear sizing that
the shared encoder also needs. Reversal evidence is a builder that derives the
complete bounded identity before retaining a candidate, or an accepted
streaming encoder with the same property.

### Hard-code twelve selected rows, even with `rows <= entries` — eliminated

This is candidate A from the count re-audit. It is correct for every currently
packageable compiler plan and its strongest counterargument to the chosen
surface is a smaller absolute row-allocation bound: a hostile or synthetic
manifest declaring thirteen rows is refused before row reservation rather than
being permitted to declare as many as 4,096.

It is not on the frontier. On the population both candidates admit, it performs
the same count, order, authority, and existing-global-limit checks and retains
the same rows. Its cheaper handling of row 13 is obtained only by refusing an
otherwise supported direct artifact whose verified program and entry table may
legally contain thirteen or more stages. Refusing useful supported work is no
more a host-time advantage here than deferring the entire feature. The fixed twelve
also couples the artifact crate's public contract to a compiler request policy
the artifact crate neither imports nor owns, requiring a public-limit review
when that producer grows even though the artifact's entry and 64 MiB budgets
remain satisfied.

Candidate B — `MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS = MAX_VARIANT_ENTRIES`
plus mandatory `rows <= entries` — is equally fail-closed: it admits no row
population outside the executable association and bounds the count before
decoder allocation; complete bytes remain governed by the existing global
manifest/identity limits. Its extra work exists only for the additional
direct-artifact population it correctly admits, is linear in an entry table the
artifact already admits, and remains inside 4,096 rows plus those global
bounds. It is strictly better on supported population, cross-layer ownership,
and maintainability, with no correctness, strictness,
identity, schema, public-surface, or common-population host-cost loss.

Evidence that would reverse the elimination is an accepted contract making the
artifact builder compiler-exclusive, or an independently governed
artifact-owned selection bound below the entry ceiling with measurements that
justify rejecting the larger direct-artifact population. Negative controls are
a thirteen-entry/thirteen-row direct artifact, which B must admit and A would
refuse; a twelve-entry/thirteen-row artifact, which the relational rule must
refuse unchanged; 4,096 entries/rows, which must reach the identity-lower-bound
and model checks; and a forged 4,097-row count, which must fail the codec limit
before allocation.

### Include `View` as an artifact kind — eliminated

`View` is compiler-reserved and rejected before selection. Publishing it would
let a direct artifact caller assert a selected state no compiler can produce.
Its tag remains reserved rather than admitted.

### Add an optional field, default empty, legacy overload, or compatibility alias — eliminated

Each permits an omitted authority statement to pass a construction path or
keeps a role-ambiguous API alive. Version skew is handled by schema rejection,
not by manufacturing provenance.

### Defer the decision — correct park, not delivery

Deferral preserves correctness only by keeping the implementation ticket
blocked and shipping no physical provenance. It costs no immediate migration
but leaves the known cache-identity omission. Choose it only by leaving this
ticket `awaiting-decision`; it cannot satisfy the implementation outcome.

## Pareto frontier: one dominant complete surface

Require the atomic `VariantSpec` field:

Add exactly:

```rust
pub struct VariantSpec {
    // existing target_profile and feasibility_rules fields unchanged
    pub selected_physical_implementations: Vec<SelectedPhysicalImplementation>,
    // existing deferred_predicates and entries fields unchanged
}
```

This is the exact public field position: after `feasibility_rules` and before
`deferred_predicates`. `VariantData` and `VariantRow` store the run at the same
conceptual position, and both public read views place its accessor immediately
after `feasibility_rules()`. `push_variant` validates the run before mutating
builder state and commits it with the rest of `VariantData`. Physical selection
is already known at plan
assembly time, unlike route requirements that genuinely arise after backend
emission. The exact broader `crates prototypes spikes` census above finds
thirteen source literals to migrate. That is a bounded in-tree cost in this
pre-production repository; a narrower `crates` textual search is a different
population and also includes struct, signature, and documentation hits, so it
must not be quoted as the literal census.

Across the required dimensions, this is top-tier on correctness (the accepted
four-subject row is lossless), fail-closed strictness (no variant insertion can
omit it), maintainability (one non-optional state and no second association
API), host runtime/RSS (one linear validation and one retained vector, with no
sort), identity/schema completeness (v19/19.0 folds every row), and public
surface size (one required field rather than a mutator/error/diagnostic family).
Its only cost is source compatibility at the thirteen exact broader-population
literals, which has no independent product authority in this pre-production
tree.

The same dominant surface uses the artifact-owned 4,096 entry ceiling and the
mandatory `selected rows <= entries` relation. This preserves every current
compiler plan, every structurally consistent direct artifact already admitted
by the entry table, and the exact occurrence multiplicity while refusing a run
that cannot fit the stated executable population. The builder's exact
physical-subset lower bound prevents retention once publication is already
impossible under the existing 64 MiB whole-identity limit; decoder memory stays
bounded by the existing pre-parse 64 MiB whole-manifest limit. Candidate A's
fixed twelve and candidate B's derived ceiling are not a remaining Tom
trade-off: the count analysis above eliminates A.

Strongest counterargument: adding a required public field is an immediate
source break for every literal, including artifacts assembled directly in
spikes/tests where synthetic physical provenance must now be stated. Evidence
that would reverse the recommendation: a real authorized producer that cannot
know compiler-selected physical rows until after `push_variant`, or external
compatibility obligations not visible in this repository.

Subject perturbations: remove the field from the `push_variant` transfer and
the selected-provider/proposal/occurrence/kind identity tests must retain the
old bytes and fail; pass an empty run, cross-role provider, duplicate
occurrence, descending occurrence, unknown build kind, or oversize identity and
assert the exact typed refusal with builder state unchanged.

The one-shot `VariantId` setter is eliminated rather than presented as a
manufactured choice. Its only advantage is preserving thirteen in-tree literal
shapes. It has the same verified-artifact memory and no host-time, identity, or
correctness advantage. It creates a partial draft state, an extra public
mutator, another association through a handle, an already-declared insertion
error, a missing-run whole-build diagnostic, and draft-to-verified
normalization. Under the repository's pre-production rule and stated
priorities, the atomic field dominates that
compatibility-only alternative. Passing the run as another `push_variant`
positional argument is strictly broader, not the same population: the rerun
census finds 65 call expressions across 13 source files versus 13 exact
`VariantSpec` literals. It also scatters one variant's declaration. A per-row
mutator adds still more partial states; an optional field is fail-open.

## Tom question

Accept or reject the exact atomic surface in this packet: require
`VariantSpec::selected_physical_implementations`, bound it by the artifact's
4,096-entry ceiling and `rows <= entries`, and commit the run transactionally
in `push_variant`.

Leaving this ticket unanswered or rejecting without a replacement means
deliberate deferral and keeps the implementation blocked. Acceptance covers the
entire surface in this packet; changing a type, field, accessor, error, limit,
tag, or domain requires an amended packet before code.

## Follow-up graph and implementation evidence

On acceptance, mark this ticket `done`; its only dependent implementation
ticket becomes schedulable and owns all code/contract/pin changes. No separate
hidden follow-up is required for the chosen slice. Any future fourth admitted
artifact proposal kind — including compiler-reserved `View` at `0x04` —
requires its own compiler/artifact vocabulary and manifest step;
any future legacy decoder requires a separate compatibility ticket.

The implementation must demonstrate, at production `assemble_plan_artifact`
and direct artifact boundaries:

- lowering-selected/physical-only offered and physical-selected/lowering-only
  offered fail with the exact role-specific errors, as do wholly absent roles;
- independently adding an unused lowering provider and an unused physical
  provider leaves artifact identity bytes, encoded envelope bytes, envelope
  digest, and expansion-cache subject unchanged;
- independently changing selected provider, proposal identity, occurrence
  association, and proposal kind moves identity/bytes/digest/cache subject;
- multiplicity across distinct occurrences and canonical occurrence order
  survive build, equality, encode, decode, and both read views;
- empty/duplicate/descending/oversize/unknown-tag populations fail before a
  product is returned;
- thirteen entries with thirteen rows are admitted, twelve entries with
  thirteen rows fail as `PhysicalSelectionCardinality`, 4,096 entries/rows
  reach the later identity-lower-bound/model checks, and a 4,097-row count
  fails before allocation; production separately asserts its selected-row count
  cannot exceed the nonempty scheduled-stage count or the current governed
  compiler limit of twelve;
- a builder candidate whose exact physical canonical contribution plus the
  mandatory nonphysical byte is at the existing 64 MiB identity boundary
  passes the early check, while one byte more fails transactionally as
  `IdentityLowerBound`; perturb the private running-total update and quote the
  unchanged boundary assertion's failure;
- a manifest header declaring one byte more than `MAX_MANIFEST_BYTES` fails as
  the existing `CodecLimitKind::ManifestBytes` before `parse_manifest`; assert
  that no physical-specific decoder byte-limit kind or boundary test exists;
- all enum/domain/limit/error maps are exhaustive and their typed populations
  are size-derived where Rust exposes the type; the compiler string bridge is
  the explicitly documented fail-closed exception;
- `PhysicalProposalKind::ALL` is `variant_count`-sized, enum-to-tag mapping is
  exhaustive and injective, every tag round-trips, and `0x00`, reserved `View`
  tag `0x04`, and `0xff` reach the unchanged unknown-tag assertion;
- identity and manifest tests assert the byte-identical shared physical run at
  the exact post-feasibility/pre-deferred insertion point, including every
  `u64` frame and the fixed `u32` provider revision; v18/v19 cross-read attempts
  quote the exact unsupported-schema failures;
- projection of a multi-MiB row proves the identity allocation is shared into
  `ArtifactEnvelope` rather than deep-copied, with the clone subject perturbed;
- each new check is subject-perturbed with assertions unchanged and its failure
  text quoted; and
- v19/19.0 and all identity/envelope/cache pins are recomputed from the exact
  implementation tree, with unrelated domains proven unchanged.

## Resource and unsupported boundary

Construction holds two independently sorted offered vectors transiently:
`O(L + P)` provider storage and `O(L log L + P log P)` comparison time. Neither
is serialized. Each selected physical row retains two `Arc<[u8]>` identity byte
runs, one provider identity, and one byte-sized kind per variant. The complete
model has at most 4,096 rows per variant, never more rows than that variant has
entries, and the builder refuses a candidate before retention when its exact
canonical physical-run contribution alone proves the complete identity must
exceed the existing 64 MiB limit.
`ArtifactEnvelope::project` clones the row/provider records
but shares both identity allocations, preventing the otherwise complete deep
copy before manifest encoding. Encoding and decode are linear in row plus
identity bytes. Decode admits at most a 64 MiB borrowed manifest before parsing;
the copied raw identity bytes are a strict subset under 64 MiB, so those two
byte populations peak below 128 MiB plus bounded row/provider metadata. No
additional sort is permitted. A production bridge copies each
compiler-borrowed identity once into the artifact's owned shared allocation. A
direct caller may allocate inputs before calling the builder, as it can for any
owned public value; refusal retains none of that candidate state in the
builder. No temporary optional physical-selection state exists.

Unsupported and refused: empty selection; more than 4,096 selected rows in one
variant; a selected row count greater than that variant's entry count;
duplicate or noncanonical occurrence order; empty/oversize opaque identity;
construction whose physical subset already proves the complete identity must
exceed its existing 64 MiB limit; an envelope whose complete manifest exceeds
its existing 64 MiB limit; missing or cross-role
offered provider; a retained pre-assembly compiler plan that cannot become a
bounded nonempty scheduled program; current compiler `View` (reserved at
`0x04`) or any unknown proposal-kind code; unknown run/kind tag, wrong row-key
domain, or trailing row-key bytes; v18 legacy decode; provenance inferred from
payload/backend/profile/entry/position; and a
claim that the artifact independently proves a direct low-level caller's rows
are the compiler's exhaustive plan. The production bridge is the authority for
that last association, and the artifact preserves exactly what it receives.
