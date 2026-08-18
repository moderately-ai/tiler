---
id: decide-the-compilation-selection-provenance-public-and-wire-surface
title: Decide the compilation-selection provenance public and wire surface
status: done
priority: p1
dependencies: [record-the-compilation-selection-in-target-measurement-provenance, refuse-unknown-fact-source-provenance-schemas-in-artifact-decode]
related: [carry-required-compilation-selection-identity-on-compile-profile-contexts, split-metal-profile-measurement-sources-by-compilation-selection, resolve-the-retained-metal-profile-measurement-invocation-authority]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, provenance, identity, metal]
---
## Decision required

The accepted semantic decision requires an exact nonempty backend-owned compilation-selection identity on every compile-profile measurement context and forbids defaulting or inference. It does not by itself fix the public Rust vocabulary, wire grammar, schema retirement, Metal derivation boundary, or the transitive identity migration. Those choices are consequential under ADR 0075.

This packet proposes one complete surface. It changes no production Rust, schema byte, profile row, decision queue, or accepted contract. Tom may accept the packet independently of the retained-row authority decision; implementation remains blocked on `resolve-the-retained-metal-profile-measurement-invocation-authority` so acceptance cannot launder the grid or cost evidence.

## User-visible outcome

Compile-profile measurement evidence states exactly which backend compilation selection produced it. Runtime/device evidence remains a different typed route and never invents compiler flags. Generic layers preserve bounded opaque bytes; Metal derives and validates their meaning. Unsupported historical evidence fails closed.

## Exact-base Fact audit — 2026-08-17

Base and branch were verified before edits: `a01e78b7c99ea8ee00a7e2e58894094587da9def` on `tkt/decide-the-compilation-selection-provenance-public-and-wire-surface`, with live claim `worker-compilation-selection-surface` and a clean initial tree. Main later advanced only through disjoint work; this packet remains intentionally pinned and was not rebased.

Read in full before the audit: repository `AGENTS.md`; the ticketsplease `SKILL.md`; this ticket; both completed dependencies; the implementation carrier and source-split tickets; `docs/README.md`; `docs/decisions/README.md`; accepted ADRs 0074, 0075, 0076, 0077, 0085, 0086, and 0090; `docs/numerical-semantics.md`; `docs/artifact-abi.md`; and `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md`. The source audit followed complete owning definitions and their correctness tests in `crates/tiler-ir/src/numerics.rs`; `crates/tiler-compiler/src/target.rs`, `target/honourability.rs`, `target/feasibility.rs`, `selection.rs`, `request.rs`, `explain.rs`, and domain ledger; `crates/tiler-artifact/src/program/realization.rs`, `realization/codec.rs`, program model/codec/domain paths, and realization tests; `crates/tiler-metal-aot/src/input.rs`, `identity.rs`, `driver.rs`, and `record.rs`; `crates/tiler-build/src/metal_declaration.rs`, `metal_profile.rs`, `metal_assembly.rs`, `metal_plan.rs`, `payload_cache.rs`, and public re-exports; proof-sidecar construction/identity; and the retained grid, crossover, partition-calibration, and Apple numerical harness/record paths.

### Verdicts

1. **Verified — semantic policy.** `record-the-compilation-selection-in-target-measurement-provenance` records a required exact backend-opaque selection per compile context. Metal includes SDK selector, requested platform/target, exact ordered compile flags, and exact ordered linker flags; source and resolved toolchain facts remain separate. There is no absent, default, inferred, or governed selection.

2. **False — the policy uniquely fixed one Rust representation.** `MeasurementContext` and `FactEvidenceBasis::Measurement` currently serve compile and later phases. Separate compile vocabulary and a discriminated shared context were both initially correct candidates. Source comparison below establishes that the separate vocabulary dominates after phase validation, public read views, and migration topology are included.

3. **Imprecise — only the named target constructors were consequential.** `FactSourceProvenance::new` and `FactSourceProvenance::measured` are also public raw assembly routes. More seriously, `FactSourceProvenance::is_valid` checks the exact phase/authority/validity triple only for `Measurement`; governed and external arms check only their authority. A caller can currently assemble governed `(ArtifactEvidence, GovernedProfile, LaunchInstance)` provenance and pass it through public artifact construction. Schema 4 must close governed/external triples and remove the raw constructors, or it preserves a laundering route unrelated to selection bytes. Anchors: `pub fn new`, `pub fn measured`, `pub fn is_valid`, and the `GovernedGuarantee` / `ExternalGuarantee` arms in `crates/tiler-ir/src/numerics.rs`.

4. **Verified with a correction — production perturbability.** `CompileRequest::link_flags()` always returns an empty run and has no production input that can vary it. The driver first selects the linker with `xcrun --sdk <sdk> --find metallib`, then executes that resolved binary with the AIR input and `-o` output; there are no additional linker flags after tool/SDK selection. `ApplePlatform::sdk()` derives the SDK selector, so no valid request can vary SDK independently of platform. A helper-only flag or SDK override would perturb synthetic input. Real request perturbations are platform/target, standard, optimization, and each numerical dimension; the empty additional-linker-flag run is count-pinned until a real input exists. Anchors: `ApplePlatform::sdk`, `CompileRequest::compile_flags`, `CompileRequest::link_flags`, `Toolchain::find_tool`, and `PreparedCompilation::compile`.

5. **False — source partition alone makes the production migration truthful.** One `measured_source(rows)` is cloned across sixteen measured declaration operations, which produce twenty-four canonical profile rows because each of the four dtype/subnormal dimension declarations installs one exact and two explicitly unsupported behaviours. The operations are grid, cost, workgroup-tree-width, two dispatchability, four dtype/subnormal dimensions, and seven other numerical declarations. The grid harness used raw `xcrun metal` without optimization/numerical flags; the cost record pins unavailable harness SHA-256 `d76fcd2fb74ecfe00b492c3042c0d1be58d88a6420f91b5cdb1940555bf9e27b` at a revision without the spike. Tree-width source content is recoverable and dispatch/numerical inputs are retained; both select O2/safe/precise/contract-off with an empty additional-linker-flag run. Grid and cost still lack current production-request authority. Exact evidence and dispositions are moved to `resolve-the-retained-metal-profile-measurement-invocation-authority`; the carrier depends on it.

6. **False — retiring the public adapter eliminates caller self-authorship.** The public `declare_metal_f32_subnormal_behaviour(builder, facts, source)` adapter in `crates/tiler-build/src/metal_profile.rs` accepts independent `MetalTargetFacts` and generic measurement provenance. Its module docs, ADR 0076's 2026-07-30 resolution paragraph (anchor: `the projection is sited in`; the source backticks the crate name, so the longer rendered phrase does not grep), and `docs/numerical-semantics.md` explicitly define that accepted boundary as caller-vouched, non-authenticating, and not production-bound. Public measured `TargetProfileBuilder` operations also intentionally let custom callers author the same rows directly, so deleting the adapter does not eliminate caller-authored profiles. The narrower choice is real: retention preserves the accepted public compatibility and owner-side transactional conversion while keeping explicitly caller-vouched Metal branding; retirement removes that Metal-branded pairing and makes the branding stricter. Both are correct survivors, compared separately below; generic caller-authored profiles remain supported either way and carry no Metal-production authentication claim.

7. **Verified — generic validation is deliberately limited.** IR/compiler can require nonempty bounded bytes, canonicalize/frame them, and compare exact equality. They cannot interpret Metal flags or decide whether Metal facts came from them. Metal AOT owns grammar derivation; Metal/build owns facts-versus-selection comparison before a complete profile or artifact exists.

8. **False — the previously stated identity minimum was complete.** `TargetEvidence::encode` writes provenance unframed under delivered-realization v2, but two other owners do the same. `encode_declaration_table` writes each `source.encode(bytes)` raw inside the checked target descriptor, and `encode_honoured` raw-appends a `HonouredDimension::canonical_key` containing that table inside selected-plan v2. The required direct domain steps are therefore delivered realization v2 to v3, checked target descriptor v10 to v11, and selected physical plan v2 to v3. The complete profile declaration frames each source and stays v11.

9. **False — the selection's 4-MiB source bound is also the construction peak.** `encode_declaration_table` currently calls `source.canonical_bytes()` once while collecting each declaration and again for each row lookup before deduplication. The bound Metal profile has nineteen scalar honourability rows, so one maximal shared source can create roughly 76 MiB of simultaneous canonical-source keys in that encoder alone, before the existing 64-KiB descriptor refusal. `complete_descriptor` repeats the same per-row pattern across all fact families. The carrier must deduplicate source references structurally before encoding and encode each unique source exactly once; the exact allocation contract is below.

These repairs do not change the ticket's purpose. They widen the exact public and delivery boundary needed to achieve it.

### Reproducible source anchors

```sh
rg -n 'pub struct MeasurementContext|pub enum FactEvidenceBasis|pub fn new\(|pub fn measured\(|pub fn is_valid|source-schema=' crates/tiler-ir/src/numerics.rs
rg -n 'TargetCompileProfileMeasurementSource|TargetMeasurementContext|TargetNumericalEvidenceBasis|enum TargetFactSourceError' crates/tiler-compiler/src/target.rs
rg -n 'source\.encode\(bytes\)|encode_declaration_table' crates/tiler-compiler/src/target/honourability.rs
rg -n 'PROFILE_DESCRIPTOR_DOMAIN|encode_honourability_facts' crates/tiler-compiler/src/target/feasibility.rs
rg -n 'SELECTED_PLAN_IDENTITY_TAG|encode_honoured|canonical_key\(\)' crates/tiler-compiler/src/selection.rs
rg -n 'DELIVERED_REALIZATION_DOMAIN|source\.encode\(bytes\)' crates/tiler-artifact/src/program/realization.rs
rg -n 'RETIRED_FACT_SOURCE|decode_provenance|decode_provenance_v3' crates/tiler-artifact/src/program/realization/codec.rs
rg -n 'pub struct CompileRequest|compile_flags|link_flags|pub const fn sdk' crates/tiler-metal-aot/src/input.rs
rg -n 'COMPILATION_DOMAIN|fn encode\(|push_strs' crates/tiler-metal-aot/src/identity.rs
rg -n 'let measured = measured_source|declare_measured_|declare_metal_f32_subnormal_behaviour' crates/tiler-build/src/metal_declaration.rs crates/tiler-build/src/metal_profile.rs
```

## Pareto analysis

| Candidate | Correct and fail-closed | Maintenance / compatibility | Host cost | Verdict |
|---|---|---|---|---|
| Status quo | No: different compile selections collapse and raw governed/external triples launder phase | Smallest diff, wrong authority | None | Eliminated |
| Optional/default/empty/inferred selection | No: absence becomes an unstated policy | Appears compatible but makes old calls silently mean something new | Small | Eliminated |
| One selection on the whole source or compiler build | No: wrong multiplicity; one build/source can cover several selections | Association remains implicit | Small | Eliminated |
| Generic flag map or Metal enum in IR/compiler | Can be complete only by duplicating every backend grammar | Cross-layer semantic leakage and drift | Small | Eliminated |
| Digest-only identity | Collision-bearing second hash authority; evidence is not readable/injective as exact bytes | Fixed size but new digest governance | Lowest | Eliminated by accepted dependency |
| Discriminated shared context | Can be correct with exhaustive phase validation | Every consumer must interpret a phase-sensitive sum; mixed collections and the old context name remain hazards; runtime callers also take the breaking constructor change | Same 64-context × 64-KiB retained selection-payload bound and required encode-once repair | Dominated |
| Separate compile context and evidence arm | Compile selection is required by type; later measurement cannot carry or be asked for it | Adds one explicit exhaustive arm but leaves later context construction intact and makes invalid cross-phase association unrepresentable | At most 4 MiB retained selection payload per source; encode-once construction adds one canonical copy per structurally unique source, independent of row reuse | **Only context-surface survivor** |
| Compiler-only or wire-only slice | No: another boundary can still erase, collide, or self-certify the selection | Partial migration cannot compile truthfully | Smaller partial diff | Eliminated |
| Remove every measured compile row and add no carrier | Fail-closed but discards the supported measured profile population and does not deliver the accepted decision | Simpler only by deleting the subject | Lowest | Not the authorized outcome; row-local withdrawal remains in the prerequisite frontier |
| Further research | The public/wire ambiguity is source-answerable and resolved | Delays no remaining public question | None | Retained only for grid/cost authority, not this surface |
| Deferral | Safe scheduling choice, not a surface | Leaves the accepted semantic decision unimplemented | None | Implementation remains blocked if Tom defers |

### Survivor counterargument and reversal evidence

The strongest counterargument to separate vocabulary is public growth: one owned context, borrowed views, one exhaustive evidence arm, and a schema/domain migration. It would be reversed by a concrete discriminated design that (a) makes mixed compile/post-compile collections impossible without phase inspection, (b) retains separate compile/runtime constructors and read views, (c) changes fewer existing callers, and (d) measurably reduces bounded host allocation. No current-source design satisfies all four; the discriminant recreates the same semantic split inside a less informative type.

The survivor's unsupported cases are explicit: old schema-3 compile evidence, empty or over-ceiling selection bytes, opaque backend bytes without an accepted facts-to-invocation binding, the unresolved grid/cost rows, and future nonempty linker selection before a real request field exists.

### Metal F32 adapter subdecision

ADR 0076 separately ratified a low-level public convenience, so its fate is not implied by choosing the context representation.

| Candidate | Correctness / strictness | Maintenance / compatibility | Verdict |
|---|---|---|---|
| Retain it with the now-required generic compile context | Correct under its existing explicit caller-vouched, non-authenticating contract; it does not claim to validate Metal selection grammar or production association | Preserves accepted public compatibility and one owner-side transactional conversion from `MetalTargetFacts` into the complete F32 tables; keeps a Metal-branded pairing of independent facts and opaque bytes | **Nondominated compatibility survivor** |
| Bind it to a new sealed Metal result | Can make the selection grammar backend-owned, but still does not prove the independently supplied fact came from that execution | Adds another public result/source vocabulary while generic custom declarations and the private production projection already cover both actual uses | Dominated |
| Retire it, keep generic measured declarations, move production projection private | Makes the Metal-branded surface no stronger than production-owned authority and preserves explicit caller-authored custom profiles in backend-neutral vocabulary | Breaks accepted public compatibility and loses the owner-side transactional conversion, but removes a second Metal-branded path with no current independent consumer | **Nondominated strict-branding survivor; recommended** |
| Defer the subdecision | The old adapter can mechanically accept the new generic source, so it can remain honest if its caller-vouched contract is preserved | Leaves an accepted boundary unresolved inside an otherwise exact public migration | Safe scheduling choice, not a complete packet |

Retention's strongest counterargument is that a Metal-branded function accepting independently supplied facts and opaque context bytes is easy to overread as backend-authenticated even when its contract says otherwise. Evidence reversing retention is a demonstrated consumer mistake or documentation burden caused by that branding; its negative control supplies arbitrary non-Metal opaque selection bytes and proves the adapter still describes the result only as caller-vouched, while a failed profile mutation leaves the caller's builder unchanged so the conversion remains transactional.

Retirement's strongest counterargument is that Tom deliberately accepted this small composable seam and a future custom profile may prefer its owner-side total F32 conversion over restating two complete tables. Evidence reversing retirement is one approved consumer outside `BoundMetalCompileDeclaration` that needs this exact caller-vouched composition and cannot use the generic measured declarations without duplicating Metal's conversion. The implementation census is `rg -n 'declare_metal_f32_subnormal_behaviour' crates docs prototypes`; today it reaches only the adapter, its tests, the bound declaration/tests, re-export, and documentation. Its negative control keeps a generic direct measured declaration compiling with a required selection while an external compile-fail fixture that imports the removed Metal convenience quotes the missing-item diagnostic.

The current census justifies recommending retirement for the stricter Metal-branded surface, but compatibility is not dominated. **Exact Tom subquestion:** retain `declare_metal_f32_subnormal_behaviour` as the accepted caller-vouched, owner-side transactional conversion, or retire it so every Metal-branded projection is production-owned while generic caller-authored declarations remain? This packet remains `in-progress` and unqueued until Tom answers; the implementation branch follows exactly one surface below.

## Proposed exact public surface

Everything in this section is **Proposal** until Tom accepts it. An implementation changing a name, field, ownership type, constructor, tag, or error has left the decision and must stop.

### 1. Shared IR vocabulary

Add these public values in `tiler_ir::numerics`:

```rust
pub const MAX_COMPILATION_SELECTION_IDENTITY_BYTES: usize = 64 * 1_024;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CompilationSelectionIdentity(Box<[u8]>);

impl CompilationSelectionIdentity {
    pub fn from_bytes(
        value: impl AsRef<[u8]>,
    ) -> Result<Self, CompilationSelectionIdentityError>;
    pub fn as_bytes(&self) -> &[u8];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CompilationSelectionIdentityError {
    Empty,
    TooLong { actual: usize, max: usize },
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CompileProfileMeasurementContext {
    compiler_builds: Vec<CompilerBuildIdentity>,
    environment: ExecutionEnvironmentIdentity,
    compilation_selection: CompilationSelectionIdentity,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum PostCompileMeasurementAuthority {
    ArtifactEvidence,
    DeviceRuntime,
    PreparedKernel,
    LaunchInstance,
}

impl CompileProfileMeasurementContext {
    pub fn new(
        compiler_builds: Vec<CompilerBuildIdentity>,
        environment: ExecutionEnvironmentIdentity,
        compilation_selection: CompilationSelectionIdentity,
    ) -> Self;
    pub fn is_valid(&self) -> bool;
    pub fn compiler_builds(&self) -> &[CompilerBuildIdentity];
    pub fn environment(&self) -> &ExecutionEnvironmentIdentity;
    pub fn compilation_selection(&self) -> &CompilationSelectionIdentity;
    pub fn encode(&self, bytes: &mut Vec<u8>);
    pub fn render(&self, output: &mut String);
    pub fn canonical_bytes(&self) -> Vec<u8>;
}
```

`CompilationSelectionIdentity::from_bytes` checks empty and `64 KiB + 1` before `Box::from`, so proportional allocation occurs only after admission. The 64 KiB value is exactly the existing complete-target-descriptor ceiling, not a new narrower policy. The profile builder's cumulative descriptor limit remains the final authority when several contexts/sources are combined.

`CompilationSelectionIdentityError` implements `Display` by writing its Debug form and implements `std::error::Error`; its exact displays are `Empty` and `TooLong { actual: 65537, max: 65536 }` for the two boundary controls.

`CompileProfileMeasurementContext::new(compiler_builds, environment, compilation_selection)` takes the identity by value, canonicalizes compiler-build order exactly as `MeasurementContext::new` does, and has accessors `compiler_builds`, `environment`, and `compilation_selection`. Existing `MeasurementContext` remains unchanged in layout and becomes explicitly post-compile-only in provenance.

Append this exact exhaustive arm to `FactEvidenceBasis`, retaining tags `0x01` through `0x03`:

```rust
CompileProfileMeasurement {
    contexts: Vec<CompileProfileMeasurementContext>,
} // tag 0x04
```

Replace the public raw `FactSourceProvenance::new` and phase/triple-taking `measured` constructors. Their internal unchecked assembler becomes private. The only public assembly routes are:

```rust
pub fn governed(authority_identity: ProvenanceIdentity, guarantee: ProvenanceIdentity) -> Self;
pub fn externally_guaranteed(
    authority_identity: ProvenanceIdentity,
    reference: ProvenanceIdentity,
) -> Self;
pub fn compile_profile_measured(
    authority_identity: ProvenanceIdentity,
    contexts: Vec<CompileProfileMeasurementContext>,
) -> Self;
pub fn post_compile_measured(
    authority: PostCompileMeasurementAuthority,
    authority_identity: ProvenanceIdentity,
    contexts: Vec<MeasurementContext>,
) -> Self;
```

`post_compile_measured` derives the phase/authority/validity triple from its enum; callers cannot supply three independent coordinates. `is_valid` closes all four basis classes:

- governed is exactly `(CompileProfile, GovernedProfile, PortableProfile)`;
- external is exactly `(CompileProfile, ExternalProfile, PortableProfile)`;
- compile measurement is exactly `(CompileProfile, MeasuredProfile, MeasuredEnvironment)` and has a nonempty, bounded, strictly increasing complete-context run; and
- ordinary measurement admits only the four exact post-compile triples already encoded by `PostCompileMeasurementAuthority`.

This removes the current public governed/external laundering route. No compile context can exist without one admitted identity; there is no `Option`, `Default`, empty sentinel, profile inference, or conversion from an ordinary context.

`tiler_artifact::program` re-exports `CompilationSelectionIdentity`, `CompilationSelectionIdentityError`, `CompileProfileMeasurementContext`, and `PostCompileMeasurementAuthority` beside its existing provenance re-exports. It does not mint duplicate artifact-owned spellings.

Canonical rendering is exact and readable. The new basis spells `:basis=compile-profile-measurement:contexts=N`; each context retains the existing `env=...;builds=...` spelling and appends `;compilation-selection=` followed by every identity byte as two lowercase hexadecimal digits. `TargetCompileProfileMeasurementSource::new` bounds the context count, not the aggregate encoded source length: a standalone admitted source can retain at most 4 MiB of selection payload (64 × 64 KiB), and explicit hexadecimal rendering can emit at most 8 MiB for that payload plus the already bounded context text. The packet adds no unaccepted smaller aggregate cap; profile construction gets the encode-once repair below so repeated rows cannot multiply one source into the current roughly 76-MiB key population.

### 2. Compiler target construction and read views

Add these public owned wrappers with private fields:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompilationSelectionIdentity(CompilationSelectionIdentity);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompileProfileMeasurementContext(CompileProfileMeasurementContext);

impl TargetCompilationSelectionIdentity {
    pub fn from_bytes(
        value: impl AsRef<[u8]>,
    ) -> Result<Self, TargetFactSourceError>;
    pub fn as_bytes(&self) -> &[u8];
}

impl TargetCompileProfileMeasurementContext {
    pub fn new(
        compiler_builds: impl IntoIterator<Item = TargetCompilerBuild>,
        environment: TargetExecutionEnvironment,
        compilation_selection: TargetCompilationSelectionIdentity,
    ) -> Result<Self, TargetFactSourceError>;
}
```

`TargetCompilationSelectionIdentity::from_bytes(impl AsRef<[u8]>)` returns `TargetFactSourceError`; `as_bytes` borrows the exact admitted bytes. `TargetCompileProfileMeasurementContext::new(compiler_builds, environment, compilation_selection)` owns all three values and performs the existing empty/duplicate/16-build validation before constructing the IR context. `TargetCompileProfileMeasurementSource::new` changes its iterator item from `TargetMeasurementContext` to `TargetCompileProfileMeasurementContext`. `TargetFactSource::measured` and `MeasuredFactAuthority` remain the later-phase public route and cannot name `CompileProfile`.

Add the borrowed read vocabulary:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetCompileProfileMeasurementContextReference<'a>(
    &'a CompileProfileMeasurementContext,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetCompileProfileMeasurementContexts<'a>(
    &'a [CompileProfileMeasurementContext],
);
```

The context reference has `compiler_builds`, `environment`, and `compilation_selection`; the last returns `&'a [u8]` directly. No zero-information borrowed identity wrapper is added. The contexts view has `len`, `is_empty`, `get`, and `iter` with the same semantics as `TargetMeasurementContexts`. Append this exact arm to the existing non-exhaustive `TargetNumericalEvidenceBasis`:

```rust
CompileProfileMeasurement {
    contexts: TargetCompileProfileMeasurementContexts<'a>,
}
```

Keep `Measurement { contexts: TargetMeasurementContexts<'a> }` for post-compile evidence only. Exhaustive internal matches in construction, read projection, canonical encoding, rendering, artifact budgeting, and tests must name both arms; no wildcard may collapse them.

Append these exact variants to the public non-exhaustive `TargetFactSourceError`:

```rust
EmptyCompilationSelectionIdentity,
CompilationSelectionIdentityTooLong { actual: usize, max: usize },
```

Its existing Debug-backed display therefore produces `EmptyCompilationSelectionIdentity` and `CompilationSelectionIdentityTooLong { actual: 65537, max: 65536 }`. Existing compiler-build/context errors remain shared and unchanged.

### 3. Schema-4 wire grammar and decode policy

Set `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` to 4. The exact canonical body is:

```text
u32be 4
u8 phase
u8 authority
u8 validity
ProvenanceIdentity authority_identity
u8 basis
  0x01: ProvenanceIdentity governed guarantee
  0x02: u64be ordinary-context count, then existing ordinary contexts
  0x03: ProvenanceIdentity external reference
  0x04: u64be compile-context count, then for each context:
          u64be compiler-build count
          compiler builds in canonical order, each as:
            role tag (and existing identity for ProviderDefined)
            framed implementation string
            framed version string
            u8 build-present, then framed build string when present
          framed platform string
          framed platform-version string
          framed platform-build string
          framed architecture string
          framed hardware string
          u64be selection byte count
          exact selection bytes
```

The selection is last so the existing build/environment prefix remains one grammar and the new association is context-local. Context sorting/deduplication uses the complete canonical bytes including selection. The existing limits remain sixteen compiler builds per context and sixty-four contexts per source; each selection has the 64-KiB ceiling above. Every length is `u64be`, checked against its limit and remaining input before allocation. No selection table/index, source-level parallel vector, or unframed opaque tail is admitted.

Artifact decode must dispatch literally `4 => decode_provenance_v4`; matching the current constant to a function named `v3` would reinterpret the new body through the old grammar. Append exactly these fieldless variants to the existing public non-exhaustive `TagSubject` enum:

```rust
MeasurementContexts,
CompilerBuilds,
CompilationSelectionIdentity,
```

`RealizationCodecError::MalformedIdentity { subject: TagSubject }` and its public field type stay unchanged; use that existing rule for all three bounded structural identities rather than inventing another error family. Generic decode does not inspect backend bytes.

The schema-4 decoder checks each just-read count before allocating or consuming that count's children. Its exact boundary matrix is:

| Subject perturbation | Exact error text |
|---|---|
| ordinary or compile context count `0` | `malformed-realization-identity: MalformedIdentity { subject: MeasurementContexts }` |
| ordinary or compile context count `65` | `malformed-realization-identity: MalformedIdentity { subject: MeasurementContexts }` |
| compiler-build count `0` in either context kind | `malformed-realization-identity: MalformedIdentity { subject: CompilerBuilds }` |
| compiler-build count `17` in either context kind | `malformed-realization-identity: MalformedIdentity { subject: CompilerBuilds }` |
| compilation-selection length `0` | `malformed-realization-identity: MalformedIdentity { subject: CompilationSelectionIdentity }` |
| compilation-selection length `65_537` | `malformed-realization-identity: MalformedIdentity { subject: CompilationSelectionIdentity }` |
| an incomplete fixed-width count, admitted child, or admitted selection payload | `truncated-realization-record: Truncated { needed: N }`, where `N` is exactly the additional byte count `Cursor::take` requires |

Precedence is fixed. Domain and provenance-schema dispatch occur first. Once a complete count word is present, zero/over-limit refusal occurs before any unconsumed child or payload can produce truncation: for example, a declared `65_537`-byte selection with no payload is `MalformedIdentity`, not `Truncated`. For an admitted count/length, children decode in wire order and the first incomplete field returns `Truncated`; for example, a declared one-byte selection with zero payload is exactly `truncated-realization-record: Truncated { needed: 1 }`. Only after all bounded syntax, ordering, and references decode does a phase/basis-incoherent evidence row return `incomplete-provenance: IncompleteProvenance { index: N }`. Trailing bytes are checked before provenance coherence as in the current record decoder. These rules apply to both basis `0x02` and `0x04` where their context/build grammar is shared.

Retire schema 3: set `RETIRED_FACT_SOURCE_PROVENANCE_SCHEMAS` to `[3]`, delete it from the ordinary decoder, and do not normalize it into 4. Exact controls after migration are:

- current delivered-realization v3 bytes with source schema 3: `unsupported-provenance-schema: RetiredProvenanceSchema { version: 3 }`;
- schema 1: `unsupported-provenance-schema: UnknownProvenanceSchema { version: 1 }`;
- schema 5: `unsupported-provenance-schema: NewerProvenanceSchema { version: 5 }`; and
- an authentic old delivered-realization v2 record: `bad-realization-domain: BadDomain` before its inner schema is inspected.

Do not recognize v2 solely to tunnel to the nested retirement. If migration UX later needs it, that is a separate `RetiredRealizationDomain` decision. There is no evidence of an external persisted artifact population worth a quarantined v3 model, and old compile-profile measurement lacks the now-required selection.

### 4. Metal-owned selection identity

Add this public opaque output in `tiler_metal_aot::identity`:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CompilationSelectionIdentity {
    bytes: Vec<u8>,
}

impl CompilationSelectionIdentity {
    pub fn as_bytes(&self) -> &[u8];
}
```

It has no public constructor, `from_bytes`, `from_flags`, mutable access, or `Default`. Add the only producer admitted by this packet:

```rust
impl CompileRequest {
    pub fn compilation_selection_identity(&self) -> CompilationSelectionIdentity;
}
```

The retained-row prerequisite's recorded-invocation candidate is not silently included here. Choosing it requires an exact sealed Metal-owned evidence producer that binds a complete invocation to the retained run and returns this same opaque output without exposing a raw wrapper. Because the historical grid selection differs from production, it also requires an accepted population-specific transfer/applicability rule and an enforced refusal when that rule does not hold. If either addition needs a new public type, function, constructor, or error, Tom must amend this packet before implementation; acceptance of the generic IR/wire surface does not pre-accept that boundary.

Its exact grammar is:

```text
raw b"tiler.metal-aot.compilation-selection.v1\0"
framed target.sdk().selector()
framed target.platform().as_str()
framed target.triple()
u64be compile-flag count
  each exact flag framed, in CompileRequest::compile_flags order
u64be linker-flag count
  each exact flag framed, in CompileRequest::link_flags order
```

The request is irrefutably destructured over `source`, `target`, `optimization`, and `numerical`; source is explicitly excluded, while optimization/numerical appear exactly once through emitted flags. SDK selector is included even though derived because it selects the `xcrun` tool and is absent from compile flags. Platform and triple remain distinct requested-family/target subjects. The current linker count is exactly zero.

Factor one private `encode_request_selection` that appends only the fields above. The new identity prefixes it with the new domain. Existing `CompilationIdentity::encode` calls the same helper at its current exact location after its evidence byte, without embedding or framing the new identity domain. `tiler.metal-aot.compilation-identity.v1` and every existing full compilation-identity byte stay byte-for-byte unchanged.

The new domain has an exact owner control in `crates/tiler-metal-aot/src/identity.rs::tests::the_compilation_selection_domain_has_one_exact_owner`. Its population is every textual occurrence of `tiler.metal-aot.compilation-selection.v1\0` in every Rust file recursively under `crates/tiler-metal-aot/src/`; the expected census is exactly one occurrence, and its sole path must be `identity.rs`. Construct the search needle in the test from two fragments so the assertion does not create a second occurrence. The same test asserts a derived selection starts with `COMPILATION_SELECTION_DOMAIN` and contains bytes after it. Changing only the owned literal from v1 to v2 must fail with exactly `tiler-metal-aot compilation-selection domain census changed: expected 1 occurrence in src/ owned by identity.rs, found 0: []`. The recursive scan must visit at least the current seven Rust files; fewer fails separately with `tiler-metal-aot compilation-selection domain source census did not reach its population: expected at least 7 Rust files, found N`. This is the new domain's owning census; it does not alter an unrelated existing domain count.

### 5. Build ownership, adapter branch, and atomic partition

Tom's adapter answer selects exactly one coherent public branch; an implementation must not blend them:

- **Retention branch:** keep the public export and exact signature `declare_metal_f32_subnormal_behaviour(builder: &mut TargetProfileBuilder, facts: &MetalTargetFacts, source: TargetCompileProfileMeasurementSource) -> Result<(), MetalF32TargetProfileError>`. Keep `MetalF32TargetProfileError::{UnstatedF32SubnormalBehaviour, Profile(TargetProfileBuildError)}`, its current displays/source chain, clone-stage-commit transactional implementation, and the caller-vouched/non-authenticating ADR and contract language. The required selection is carried transitively inside `TargetCompileProfileMeasurementSource`; the adapter neither parses it nor claims Metal production binding. `BoundMetalDeclarationError::SubnormalProjection(MetalF32TargetProfileError)` stays and the bound production declaration may keep using this conversion after its separate population selection-equality check.
- **Retirement branch — recommended:** delete that public export and `MetalF32TargetProfileError`; update ADR 0076, the numerical contract, Metal backend contract, and authority ledger in the atomic carrier; move the bound profile's F32 projection into its private production implementation. `TargetProfileBuilder::declare_measured_*` remains the explicit generic caller-authored seam and makes no Metal-validation or production-binding claim. Removal establishes only the narrower property that no public Metal-branded build adapter accepts independently supplied `MetalTargetFacts` plus generic opaque selection bytes.

Both branches preserve caller-authored profiles. Retention optimizes accepted compatibility and transactional owner-side conversion; retirement optimizes strict Metal branding and public-surface size.

Add this exact public population vocabulary in `tiler_build`:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MetalProfileMeasurementPopulation {
    GridAxis,
    SaturatedParallelFoldSteps,
    WorkgroupTreeWidthPolicy,
    DispatchabilityAndNumerics,
}

impl MetalProfileMeasurementPopulation {
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::GridAxis,
        Self::SaturatedParallelFoldSteps,
        Self::WorkgroupTreeWidthPolicy,
        Self::DispatchabilityAndNumerics,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GridAxis => "grid-axis",
            Self::SaturatedParallelFoldSteps => "saturated-parallel-fold-steps",
            Self::WorkgroupTreeWidthPolicy => "workgroup-tree-width-policy",
            Self::DispatchabilityAndNumerics => "dispatchability-and-numerics",
        }
    }
}
```

`ALL` lists the four variants in declaration order and is type-sized so population growth is compile-stopping at the census. The carrier adds unconditional `#![feature(variant_count)]` to `crates/tiler-build/src/lib.rs`, matching the repository's nightly toolchain and the existing `tiler-metal-aot` census mechanism; without that exact gate the proposed declaration does not compile. `as_str` is total inside `tiler-build` and returns, respectively, `grid-axis`, `saturated-parallel-fold-steps`, `workgroup-tree-width-policy`, and `dispatchability-and-numerics`. Both adapter branches append this common exact public variant to `BoundMetalDeclarationError`:

```rust
CompilationSelectionMismatch {
    population: MetalProfileMeasurementPopulation,
},
```

Only the retirement branch also replaces `BoundMetalDeclarationError::SubnormalProjection(MetalF32TargetProfileError)` with:

```rust
UnstatedF32SubnormalBehaviour,
F32SubnormalProjection(TargetProfileBuildError),
```

The crate root re-exports `MetalProfileMeasurementPopulation` beside `BoundMetalCompileDeclaration` and `BoundMetalDeclarationError`.

The retirement-only variants mirror the private BF16 ownership and keep the refusing authority visible. Under this packet's admitted request-derived Metal authority, the mismatch display is exactly `compilation selection: retained <population> selection differs from the production CompileRequest selection`, with `<population>` replaced by the exact `as_str` result. The four complete failure texts are therefore:

```text
compilation selection: retained grid-axis selection differs from the production CompileRequest selection
compilation selection: retained saturated-parallel-fold-steps selection differs from the production CompileRequest selection
compilation selection: retained workgroup-tree-width-policy selection differs from the production CompileRequest selection
compilation selection: retained dispatchability-and-numerics selection differs from the production CompileRequest selection
```

In the retirement branch, `UnstatedF32SubnormalBehaviour` keeps the retired adapter error's exact display `Metal target facts do not state f32 subnormal behavior`. `F32SubnormalProjection(error)` displays `compiler target profile refused Metal f32 facts: {error}`. Its `std::error::Error::source` is `Some(error)`; the unstated and mismatch variants return `None`. In the retention branch, the existing `SubnormalProjection(error)` display/source behaviour and `MetalF32TargetProfileError` remain unchanged; only the common mismatch variant is new.

The implementation carrier must land the source partition in the same compiling commit as the required context:

- private ledger data names grid, cost, tree-width, and dispatchability/numerical source populations separately;
- every retained population carries independently derived expected canonical selection bytes;
- Metal/build constructs every context assigned to a population from that expected identity and compares it against the production `CompileRequest::compilation_selection_identity()` before `TargetProfileBuilder::build` creates a complete descriptor;
- rows share a `TargetCompileProfileMeasurementSource` only after exact selection equality, never because environment text matches; and
- a `CompileRequest`-produced measurement derives those expected bytes through `CompileRequest::compilation_selection_identity()`. Generic code never spells Metal flags.

At this base, tree-width and the thirteen dispatchability/numerical declaration operations (twenty-one canonical rows) reconstruct the same O2/safe/precise/contract-off selection and may share after the equality control. Grid and cost cannot yet be populated truthfully; their accepted disposition is the blocking prerequisite. No noncompiling intermediate source partition and no invented production input is allowed.

Exact selection bytes are canonical producer-vouched metadata, not proof that an invocation ran or that its outputs produced the retained facts. Equality with the production request prevents metadata drift and cross-population substitution; it does not by itself authenticate execution. The blocking retained-row ticket must bind each retained harness, source, environment, raw result, and validation to the invocation that executed, or remove the row. A recorded historical selection that differs from production additionally needs an accepted population-specific transfer/applicability rule and enforcement; without one, provenance recovery does not authorize using the fact. The carrier must not call a byte-equality-only path checked or authenticated.

The carrier also repairs the existing source-table allocation multiplier without adding a cap. In both `target/honourability.rs::encode_declaration_table` and `target.rs::complete_descriptor`, collect `&FactSourceProvenance` values first, sort and deduplicate them by the type's structural `Ord`/`Eq`, then call `canonical_bytes()` exactly once per structurally unique source. Sort those retained `(canonical_bytes, source)` pairs by canonical bytes and collapse equal canonical runs exactly as today, assigning every structural source in a run the same compact index. Row loops look up that precomputed structural-source index and never recalculate canonical source bytes. The canonical table order and collision behaviour are therefore byte-identical. The existing checked/complete 64-KiB descriptor checks and `DescriptorTooLong { actual, max }` results stay unchanged. Temporary canonical-source storage is linear in the already retained unique source population rather than multiplied by declaration count; one source shared by the current nineteen scalar rows contributes one canonical copy, not nineteen.

## Identity, schema, pin, and publication cascade

### Direct version owners

| Owner | Required result | Why |
|---|---|---|
| Fact-source provenance | schema 3 → 4 | New context grammar and basis tag |
| Metal compilation selection | new `tiler.metal-aot.compilation-selection.v1` | New independently named subject |
| Checked target descriptor | `tiler.target-profile.descriptor.v10` → `v11` | `encode_declaration_table` writes source bytes unframed |
| Selected physical plan | `tiler.compiler.selected-physical-plan.v2` → `v3` | `encode_honoured` raw-appends a key containing the source table |
| Delivered realization | `tiler.artifact-program.delivered-realization.v2` → `v3` | `TargetEvidence::encode` writes provenance unframed |
| Explain renderer | 9 → 10 | Existing refusal spelling gains schema 4 / compile-selection text |

The checked-descriptor and selected-plan domain steps move **every** value in those domains, even a value with no numerical source/honoured row. The unframed source change is why each owner must step; once its domain steps, the movement is global. This resolves the earlier contrary premise from source rather than copying it.

### Domains and schemas that stay

- complete target declaration `tiler.target-profile.declaration.v11`: each source is length-framed;
- target profile-source domain v4;
- compiler request subject v6: it raw-appends the independently versioned, self-delimiting complete descriptor in an unchanged field position, so the nested value moves without changing the request grammar;
- selected physical portfolio v1 and program alternative v2: they frame nested selected-plan identities;
- explain trace schema 11 and compilation-explain schema/renderer 1: canonical evidence/request subjects are framed; only the human renderer changes globally;
- physical-implementation proposal v3;
- existing Metal full compilation identity v1;
- artifact program v18, payload v1, manifest schema 18.0/domain v1, envelope format/canonical versions, and envelope digest domains;
- proof-sidecar identity and wire domains; and
- cache composed-subject v1 and expansion-key v1.

“Domain stays” does not mean “value stays.” Recompute rather than hand-copy every transitive value.

### Complete transitive population

1. Every source's canonical bytes and rendered `source-schema` move, including governed, external, and post-compile sources whose body fields otherwise do not. Complete descriptors move for profiles carrying any source; the governed and bound Metal profiles do.
2. Checked target descriptors all move because their owner domain steps. Complete-descriptor source-table sorting/deduplication and compact indices may also move.
3. Every selected-plan identity/digest moves because its owner domain steps. Selected portfolios, program-alternative values, deterministic ordering/tie-breaking, and selection/explain values move transitively. A provenance-only migration therefore cannot promise identical equal-cost plan choice or object bytes universally.
4. Request subjects and stable request qualifiers move for requests whose complete profile descriptor moves. Explain trace identities then move through the framed request subject; unhonourable record identities also move through framed refusal evidence. Renderer headers all spell v10.
5. `TargetProfileRef`, backend payload compatibility keys, variant compatibility, delivered-realization bytes, artifact-program identity/digest, manifest bytes/length/digest, envelope bytes/fixed content/digest, and cache composed subjects/keys/shards/paths/locks/bundle identities move for affected artifacts.
6. Proof sidecars copy the exact artifact canonical identity and envelope digest; `ProofSidecarBuilder::build` assembles those fields and calls `proof::codec::derive_identity`, which folds both. Regenerate proof-sidecar identity/wire bytes, association values, and pins while leaving proof domains unchanged.
7. For a fixed request and executable, existing Metal `CompilationIdentity`, payload metadata/identity, emitted object sections/digests, and compilation-subject facets do not move solely because provenance gained the selection. The artifact-program facet and therefore a composed cache value still move.

Direct pin/census owners include IR provenance render tests; artifact realization schema controls and domain ledger; compiler domain ledger, governed descriptor golden, selected-plan/portfolio pins, explain version/header/request pins, pipeline/session renderer prefixes; bound Metal descriptor length; the exact one-occurrence Metal selection-domain owner census above; standard Metal artifact/cache/fixed-content pins and authority-ledger mirrors; proof-sidecar association/identity pins; `docs/numerical-semantics.md`, `docs/artifact-abi.md`, `docs/compiler/optimizer.md`, and the first Metal authority ledger. Existing domain counts stay unchanged; the new Metal selection subject is governed by its explicit literal-owner census rather than a vague increment to an unnamed ledger.

## Required independent controls

The implementation must perturb production subjects, not assertions, and retain the quoted failure.

1. **Generic construction:** empty and 65,537-byte selections fail before copy with the two exact target errors above. Two contexts differing only in selection have different canonical bytes, remain distinct after sorting, and move complete/checked descriptors.
2. **Phase laundering:** mutate only phase/authority/validity bytes of otherwise valid governed and external v4 artifact evidence; each fails `incomplete-provenance: IncompleteProvenance { index: 0 }`. The removed public raw constructor is unreachable to an external compile test.
3. **Schema dispatch:** rewrite only the schema word to 1, 3, and 5; obtain the exact unknown/retired/newer texts above even when the first body byte is also corrupt. An old v2 outer domain fails `bad-realization-domain: BadDomain` first.
4. **Metal identity:** source-only and resolved-toolchain-only changes leave selection equal while moving full compilation identity. Reachable platform/target, language, optimization, math mode, fp32-functions, and contraction changes each move selection. `ApplePlatform::ALL` is sized with `variant_count` and pins the derived SDK mapping. The additional-linker-flag count after tool/SDK selection is asserted zero; no value perturbation exists until production has an input.
5. **Exhaustiveness:** adding a temporary basis variant must fail every encoder/renderer/read projection; adding a real `CompileRequest` field must fail its irrefutable selection destructure. Quote the compiler diagnostics.
6. **Build authority:** alter only one retained population's independently derived expected bytes while leaving the production request fixed; obtain that population's exact `compilation selection: retained ... selection differs from the production CompileRequest selection` text above before the profile descriptor exists. Independently test all four typed populations and prove identical sources deduplicate only after equality. If Tom later accepts a recorded-invocation transfer, independently perturb every premise of that transfer and quote its typed refusal rather than bypassing this control.
7. **Identity owners:** independently perturb all three unframed owners. Checked-descriptor and selected-plan subject changes each fail the compiler census with `PINNED_IDENTITY_DOMAINS occurrence census changed`; the message must name the independently perturbed old literal. The delivered-realization change fails the artifact census with `ProgramDeliveredRealization's exact domain bytes moved: ...`.
8. **Source-table allocation:** instrument canonical-source encoding only under `cfg(test)`. Nineteen declarations sharing one structurally equal source invoke canonical encoding once inside each table encoder; changing one declaration to a valid source differing only in selection makes that encoder's census exactly two, not nineteen or twenty. Run the checked and complete encoders independently and quote each census failure when the encode-once subject is temporarily replaced by the old per-row call.
9. **Metal selection domain owner:** change only `COMPILATION_SELECTION_DOMAIN` from v1 to v2. The recursive `src/**/*.rs` owner census must fail with `tiler-metal-aot compilation-selection domain census changed: expected 1 occurrence in src/ owned by identity.rs, found 0: []`; restore it, then prove the derived subject starts with the exact v1 owner and has trailing content.

Current reachable schema evidence at this base already reports:

```text
unsupported-provenance-schema: UnknownProvenanceSchema { version: 1 }
unsupported-provenance-schema: NewerProvenanceSchema { version: 4 }
```

The current artifact test obtains the first two by changing schema/body bytes. Its `RetiredProvenanceSchema { version: 2 }` text is only a directly constructed Display control because `RETIRED_FACT_SOURCE_PROVENANCE_SCHEMAS` is empty; it is not current subject-perturbation evidence. Migration moves the newer subject to 5 and makes schema 3 the first decoder-reachable retired subject, which must then be exercised by rewriting only the schema word of otherwise current v3-domain bytes.

Two independent live-domain subjects were also changed temporarily and restored before this packet was committed:

- changing only `DELIVERED_REALIZATION_DOMAIN` from v2 to v3 and running `cargo test -p tiler-artifact --lib every_governed_domain_has_its_exact_pinned_bytes -- --nocapture` failed with `ProgramDeliveredRealization's exact domain bytes moved`, printing expected v2 and observed v3;
- changing only `SELECTED_PLAN_IDENTITY_TAG` from v2 to v3 and running `cargo test -p tiler-compiler --lib every_pinned_identity_domain_has_its_exact_source_population -- --nocapture` failed with the exact fragments `PINNED_IDENTITY_DOMAINS occurrence census changed` and `tiler.compiler.selected-physical-plan.v2\0 in src/: expected 1 occurrence(s), found 0` (the terminal output surrounds the identifiers with Markdown backticks).

After restoration, one nextest expression reran those two controls plus the schema perturbation: three tests passed and 1,252 were skipped. The final diff contains no production file.

## Performance and unsupported boundary

The surface adds bounded linear validation, one owned byte copy per context, canonical encode/compare at profile construction, and hex rendering only when explaining. It does not run in kernel execution, device runtime, or physical-plan search. `Box<[u8]>` retains no spare capacity; whole sources remain shared by the compiler's existing `Arc<FactSourceProvenance>`. The per-source worst case is nevertheless material enough to state: 4 MiB of retained selection payload, an approximately 8-MiB selection rendering, and one additional canonical copy for each structurally unique source while a checked or complete descriptor is encoded. The required encode-once repair removes the current declaration-count multiplier; final descriptor output is still refused above 64 KiB. The current Metal source uses a small handful of short contexts. Measure before adding a smaller aggregate cap; that would be a new policy decision.

Unsupported after acceptance, until separately closed:

- schema-3 artifacts and old delivered-realization v2 records;
- grid and cost facts without the retained-row prerequisite's authority;
- arbitrary Metal identity bytes not equal to a backend-derived request;
- under the retirement branch, a Metal-branded custom projection; under retention, the accepted convenience remains explicitly caller-vouched and is still unsupported as production-authenticated evidence;
- future nonempty linker flags without a real `CompileRequest` input and measurement; and
- identities above the existing 64-KiB complete-descriptor ceiling.

## Recommendation and stop boundary

**Proposal:** accept the separate compile-profile context/evidence vocabulary and every exact common surface, tag, domain, ownership rule, and control above; for the independent adapter subdecision, choose between the two nondominated branches by the exact binary question above, with retirement recommended from the current consumer census. Keep this packet unqueued and the implementation carrier blocked until Tom answers that question and also resolves the retained grid and cost authority ticket.

Decision research only. Do not edit production constructors, schema bytes, profiles, public APIs, accepted ADRs, or the decision queue from this ticket. Only Tom changes this ticket to `done` and opens implementation.

## Closes when

Tom accepts this exact packet, or records a different exact surface after the same Pareto and identity gate. Acceptance alone does not authorize implementation or assert that the grid/cost prerequisite is complete.

## Independent review — 2026-08-18

Adversarial review at base `b5189f1009cf42bed6901e852ad538720a798690`, independent of the packet author. Every source named below was read at this base and every command was rerun here; nothing in this section restates packet evidence without reproducing it.

### Per-Fact verdicts

Where the audit labels a verdict **False**, the verdict below applies to the audit's replacement claim.

1. **Verified.** `record-the-compilation-selection-in-target-measurement-provenance` (read in full, status `done`) records exactly the required exact backend-opaque selection per compile context, the Metal field list including SDK selector and ordered compile/link flags with source and resolved toolchain excluded, and no absent/default/inferred selection. Its ranked alternative 2 is the digest elimination the Pareto table cites as "Eliminated by accepted dependency".
2. **Verified.** In `crates/tiler-ir/src/numerics.rs`, the `Measurement` arm of `pub fn is_valid` admits the compile triple and all four post-compile triples over one shared `MeasurementContext`, so the accepted semantics did not uniquely fix one Rust representation.
3. **Verified end to end.** `pub fn new` and `pub fn measured` are public raw assembly routes; the `GovernedGuarantee` and `ExternalGuarantee` arms of `is_valid` check only `self.authority` plus identity validity, never phase or validity. The laundering route is reachable: `TargetEvidenceDeclaration` in `crates/tiler-artifact/src/program/realization.rs` exposes a public `source: FactSourceProvenance` field, and `DeliveredRealizationBuilder::require` admits it through `evidence.source.is_valid()` plus a phase ceiling that the packet's `(ArtifactEvidence, GovernedProfile, LaunchInstance)` example passes.
4. **Verified.** In `crates/tiler-metal-aot/src/input.rs`, `pub fn link_flags` returns `Vec::new()` unconditionally, `pub const fn sdk` derives the selector from the platform on both `ApplePlatform` and `MetalTarget`, `compile_flags` orders target/standard/optimization/three numerical flags, and `ApplePlatform::ALL` is `variant_count`-sized. In `crates/tiler-metal-aot/src/driver.rs`, `fn find_tool` runs `xcrun --sdk <sdk> --find <tool>` and `PreparedCompilation::compile` passes only the empty `link_flags` run, the AIR input, and `-o` to the resolved linker binary.
5. **Verified by independent recount.** `BoundMetalCompileDeclaration::declare` in `crates/tiler-build/src/metal_declaration.rs` makes exactly sixteen measured declaration operations from one `let measured = measured_source(rows)?` — grid, cost, tree-width, two dispatchability, four subnormal-dimension declarations inside the two projection helpers, and seven remaining numerical rows — producing twenty-four canonical rows of which nineteen are scalar honourability rows. `compile_probe` in the grid spike (read at this base) invokes `xcrun --sdk macosx metal` with only `-std`, `-target`, `-c`, `-o` and links with no additional flags. `git show aa17fe8afba0468418c1bc51bf7bc052f1b96742:spikes/target-profiles/metal-grid-axis-extent/src/main.rs | shasum -a 256` reproduces `360eea9e7644e5ef9fcf08c8f410ca13d46665cb9b631b6be0d48374d1655266`, and `git cat-file -e 01f140237f3617a5d415dbc0a67182a83ac8d139:spikes/program-planning/reduction-dispatch-crossover/src/main.rs` fails at this base, so grid and cost still lack production-request authority.
6. **Verified; one anchor repaired in place.** `rg -n 'declare_metal_f32_subnormal_behaviour' crates docs prototypes` reproduces exactly the stated population: the adapter and its tests in `crates/tiler-build/src/metal_profile.rs`, the bound declaration and its tests in `metal_declaration.rs`, the `lib.rs` re-export, and four documents (ADR 0076, `docs/backends/metal.md`, `docs/numerical-semantics.md`, the first Metal authority ledger). The adapter's module docs state the caller-vouched, non-authenticating contract verbatim. The ADR 0076 anchor as previously quoted grepped 0 because the source backticks the crate name; repaired above to a fragment that greps 1. No claim changed.
7. **Verified** against the same reads: the compiler holds provenance as `Arc<FactSourceProvenance>` and nothing in `crates/tiler-compiler/src/target.rs` interprets Metal flags; `MeasuredFactAuthority` omits every compile-profile authority.
8. **Verified — all three unframed owners confirmed.** `TargetEvidence`'s encoder ends with `source.encode(bytes)` inside `tiler.artifact-program.delivered-realization.v2`; `encode_declaration_table` writes `source.encode(bytes)` raw and is reached by `encode_honourability_facts` inside `tiler.target-profile.descriptor.v10`; `fn encode_honoured` raw-appends `HonouredDimension::canonical_key`, which `HonouredDimension::new` builds through `encode_honourability_facts`, inside `tiler.compiler.selected-physical-plan.v2`. The complete declaration `tiler.target-profile.declaration.v11` frames each source with `push_slice` and stays.
9. **Verified with arithmetic.** `encode_declaration_table` maps every declaration to `(source.canonical_bytes(), source)` before deduplication, so nineteen scalar declarations sharing one maximal source — 64 contexts × 64 KiB, against `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` = 64 × 1024 — hold roughly 76 MiB of simultaneous canonical keys, and each row loop re-derives `canonical_bytes` for its lookup. `complete_descriptor` in `target.rs` repeats the same per-row shape across all fact families, and its descriptor bound is checked only after encoding.

### Recommendation and stop boundary re-derived

- The context-surface frontier is complete at this base; no materially distinct option is missing. Attaching the selection to the producer identity or reusing the full AOT `CompilationIdentity` are identity conflations already eliminated through the accepted dependency's ranked alternatives, and every elimination in the table holds on re-reading.
- The adapter subdecision is genuinely nondominated and the census supports the retirement recommendation: the only production caller is the bound declaration's own projection, which retirement makes private, while retention keeps the accepted 2026-07-30 public compatibility and the transactional owner-side conversion. The retirement-only error variants mirror the existing private BF16 ownership (`UnstatedBf16SubnormalBehaviour`, `Bf16SubnormalProjection(TargetProfileBuildError)`), verified in `BoundMetalDeclarationError`.
- The double block is real in the ticket graph: `resolve-the-retained-metal-profile-measurement-invocation-authority` is `blocked`, tagged `needs-tom`, and depends on this decision; the carrier `carry-required-compilation-selection-identity-on-compile-profile-contexts` depends on both plus the two done prerequisites.
- Every "stays" and "steps" constant in the identity cascade reproduces at this base: descriptor `v10`, declaration `v11`, fact-sources `v4`, selected-plan `v2`, portfolio `v1`, delivered-realization `v2`, request-subject `v6` (whose encoder raw-appends the self-delimiting `request_subject_bytes` in a fixed field position), explain schema 11 and renderer 9, `tiler.metal-aot.compilation-identity.v1`, artifact-program `v18`, proposal `v3`, program-alternative `v2`, cache composed-subject `v1` and expansion-key `v1`, and the proof-sidecar `v1` domains with `derive_identity` folding artifact identity and envelope digest.
- The §4 factoring is byte-preserving as claimed: in `CompilationIdentity`'s encoder the selector/platform/triple/`compile_flags`/`link_flags` run sits contiguously after the evidence byte, so `encode_request_selection` extracts without moving existing identity bytes. The `crates/tiler-metal-aot/src/` census is exactly seven Rust files, `#![feature(variant_count)]` is unconditional there, and `tiler-build` lacks it, so the carrier's gate addition is genuinely required for a non-test `variant_count`-sized `ALL`.

### Commands rerun at this base

- All ten reproducible source anchor commands: every pattern hits in the file it names.
- `cargo nextest run -p tiler-artifact -p tiler-compiler -E 'test(every_governed_domain_has_its_exact_pinned_bytes) + test(every_pinned_identity_domain_has_its_exact_source_population) + test(an_unsupported_provenance_schema_is_refused_before_the_body_is_read)'`: `3 tests run: 3 passed, 1252 skipped`, matching the packet's recorded rerun exactly.
- Both temporary live-domain perturbations were independently re-executed and restored, and the tree verified byte-identical afterward (`git status --porcelain` empty). Delivered-realization `v2`→`v3` failed the artifact census with `ProgramDeliveredRealization's exact domain bytes moved`, printing expected `…v2\0` and observed `…v3\0`; selected-plan `v2`→`v3` failed the compiler census with the fragments `PINNED_IDENTITY_DOMAINS occurrence census changed` and `tiler.compiler.selected-physical-plan.v2\0 in src/: expected 1 occurrence(s), found 0` (identifiers backticked in the terminal output), with the scan reporting 149 `tiler.` literals read. After restoration the three-control expression ran green again.
- The current schema-evidence texts in `an_unsupported_provenance_schema_is_refused_before_the_body_is_read` match the packet verbatim, including the display-only `RetiredProvenanceSchema { version: 2 }` control and the damaged-body-after-newer-schema case.

### Discrepancies

1. **Minor — repaired.** Fact 6's ADR 0076 anchor was copied from the rendered view and failed as absence, the dangerous direction; replaced in place with a greppable fragment. No factual claim moved.
2. **Note, no repair.** The current decoder dispatches `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION => decode_provenance_v3` by constant, which is exactly the hazard the packet's "dispatch literally `4 => decode_provenance_v4`" instruction exists to close; verified real at this base and correctly pre-empted by the packet.
3. **Note, no repair.** The schema-4 boundary matrix's zero/over-limit count refusals are new behaviour for basis `0x02` as well: today an out-of-bounds ordinary context count decodes structurally and fails later as `incomplete-provenance`. The packet states the shared rule explicitly and the schema step makes it non-breaking; recorded here so the carrier does not read the matrix as describing current `0x02` behaviour.

### Verdict

**Ready for Tom as-is**, with the single anchor repair made here. Every audit verdict re-derives from source at this base; the frontier is complete and its eliminations hold; the adapter census supports the retirement recommendation while retention remains a real nondominated alternative; the carrier's double block is present in the graph; and every quoted ceiling, census, domain constant, and refusal text reproduces, including both live-domain perturbations re-executed independently.


## Accepted decision — 2026-08-18

Tom accepted, in the live coordination session with the orchestrator (relayed first-hand by the coordinator, reply `agreed, next decision`), **the complete packet body and the adapter retirement branch**: the separate compile-profile context/evidence vocabulary with every exact common surface, tag, domain, ownership rule, and control as packeted and reviewed (`3a265735`), and — for the one binary subdecision — **retire** the public `declare_metal_f32_subnormal_behaviour` adapter, whose only production caller becomes private with it, per the review-rerun consumer census. Generic caller-authored profiles remain supported and carry no Metal-production authentication claim.

The implementation carrier remains blocked on `resolve-the-retained-metal-profile-measurement-invocation-authority` exactly as the packet's stop boundary states; this acceptance does not touch the grid/cost authority question.
