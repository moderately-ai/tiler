---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.delivered-realization-record"
kind: "experiment"
title: "The delivered-realization record, redesigned from typed evidence"
topics: ["numerics", "artifacts", "provenance", "api", "feasibility"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "exhaustive-finite"]
supports: ["tiler.contract.artifact-abi", "tiler.contract.numerical-semantics"]
entrypoints: ["spikes/numerics/delivered-realization-record/src/main.rs"]
last_verified: "2026-08-05"
ticket: "redesign-the-delivered-realization-record-from-typed-evidence"
---

# The delivered-realization record, redesigned from typed evidence

A **compile-checked private design packet** for the record ADR 0076 item 4 requires. It replaces the staged four-dimension draft in `crates/tiler-artifact/src/program/realization.rs`, which a full-tree audit disproved.

**This is a proposal.** Nothing here is accepted. `accept-the-delivered-realization-artifact-surface` owns Tom's ratification of the public boundary, and `wire-the-delivered-realization-record-into-the-artifact` owns every production change. No file under `crates/` is modified by this packet — the exact check is `git diff --name-only <base>..HEAD | grep '^crates/'`, which returns nothing.

## Running it

From this directory. `rust-toolchain.toml` resolves by directory ancestry from the repository root, so no selector is passed and this spike deliberately carries no toolchain file of its own.

```sh
cd spikes/numerics/delivered-realization-record
CARGO_TARGET_DIR=./target cargo run
```

`CARGO_TARGET_DIR` is set explicitly because this is a nested workspace and sharing one target directory across unrelated workspaces is forbidden. The binary's only product is a verdict: every stage that fails exits non-zero with the stage named, and there is no partial success. It needs no GPU, no Xcode, and no simulator.

Ten stages run, ending in 38 perturbations covering all 25 distinct rule identifiers the two proposed error vocabularies define.

## Why the staged draft had to be replaced — each cited defect verified at source

Every claim below was checked by reading the file, not by search. Line numbers are at base `6544d4f`.

**1. The artifact declares a second, drifting `NumericalDimension` with four cases.** `crates/tiler-artifact/src/program/realization.rs:163-172` declares `InputSubnormals`, `ResultSubnormals`, `Contraction`, `Reassociation`, and `CANONICAL_DIMENSIONS` at `:175-180` is length 4. The compiler's authority at `crates/tiler-compiler/src/target/honourability.rs:799-828` has **eleven** cases, with `CANONICAL_DIMENSIONS: [_; 11]` at `:832-844`. **Confirmed, with an additional finding:** the draft's own doc comment at `:144-155` says "These are the four behaviour dimensions `tiler_ir::schedule::NumericalRealization` carries" — and that realization carries **eight** (`crates/tiler-ir/src/schedule/numerics.rs:207-229`: the two subnormal dimensions, contraction, reassociation, permutation, signed zero, and the two exceptional-value assumptions). So does the artifact codec's `NumericalFacts` (`crates/tiler-artifact/src/program/codec/model.rs:231-243`). The doc comment is a load-bearing claim that is false, in the direction that makes the draft look complete.

**2. Compiler honourability is keyed by more than the dimension, and one profile genuinely needs two answers.** `NumericalRequirement::subject` (`honourability.rs:1492-1498`) returns `(NumericalDimension, ArithmeticType, Vec<u8>)` — the ticket's `(NumericalDimension, ArithmeticType)` is a simplification; the **complete resolved type** is the third coordinate. `feasibility.rs:1146-1157` filters candidate facts on all of dimension, arithmetic, resolved type, and behaviour. The public subject type is `tiler_compiler::target::ScalarArithmetic` (`target.rs:1290-1381`), validated against the governed scalar catalog. **Confirmed:** a dtype-free `honoured(dimension)` cannot return one correct answer.

**Correction to the ticket's supporting sentence, and it matters for the fixture.** The ticket says "the measured Apple profile preserves `f16` input subnormals and flushes `f32`". That divergence is real but it is **not declared by any target profile in this tree**. It lives in `tiler-metal`'s `MetalSubnormalArithmeticFacts` (`crates/tiler-metal/src/golden_compilation.rs:184-207`: F32 flush, F16 preserve, BF16 flush). Every `ScalarHonourabilityDeclaration` in the workspace is over `ScalarArithmetic::governed_f32()` or, in `crates/tiler-build/src/metal_declaration.rs`, `f32` and `bf16`; that file states at `:616-620` that "F16 is deliberately absent", and `metal_profile.rs:21-23` states `f16` behaviour "remains unknown". Exact check: every `DeclaredBehaviour::` construction site, of which there is exactly one non-test producer at `target.rs:1473-1482`. The packet's two-dtype fixture therefore uses **checked synthetic** evidence — which `express-metal-honourability-in-the-shared-form` explicitly admits for this ticket — and proves a property of the *record*, not of any measured target.

**3. `HonouringMeans::key` collapses every declared relaxation to one string.** `honourability.rs:1094-1103`: the `SupportedOnlyUnderDeclaredRelaxation { .. }` arm returns the constant `"supported-only-under-declared-relaxation"`, discarding the `RelaxationRequirement` payload. `HonouringMeans::encode` at `:1120-1131` *does* write the relaxation. The staged artifact draft carries `HonouringMeansKey` bytes minted from `key()` (`realization.rs:232-268`), so it carries the non-injective projection and not the identity. **Confirmed, and it is the decisive defect:** two artifacts honouring one contract under different relaxations would share the record, and a reader cannot say which relaxation made a requirement honourable.

**4. Required provenance is absent.** ADR 0076 item 3 requires "an availability phase, a validity scope, an authority, and the declaring profile's identity", and adds that the validity scope "must identify which compiler build and which execution environment the declared behaviour was measured on"; item 4 states the record "inherits that requirement rather than adding one". The draft's `HonouredDimensionFact` (`realization.rs:288-292`) carries a means key and a phase; `DeliveredNumericalRealization` (`:320-326`) carries one record-level `TargetProfileRef`. **Confirmed:** authority, validity scope, compiler build, and execution environment are all absent. The draft's own comment at `:283-287` says so and names `carry-the-honourability-fact-provenance-into-the-artifact-record` as the owner — that ticket is now `done` and the vocabulary exists (`FactSourceProvenance`, `MeasurementContext`, `CompilerBuildIdentity`, `ExecutionEnvironmentIdentity` in `honourability.rs:69-654`), all `pub(crate)`.

**5. `declare` accepts arbitrary bytes and a caller-selected phase.** `realization.rs:423-446` takes `means: impl AsRef<[u8]>` and `available_at: AvailabilityPhase`, checks emptiness, a 256-byte bound, redeclaration, and a phase ceiling. **Confirmed:** it validates framing. Nothing ties the claim to a checked compiler plan.

**A sixth finding, not cited by the ticket, found while verifying the second.** `crates/tiler-compiler/src/policy.rs:636-654` builds every `NumericalRequirement` with a hard-coded `F32::resolved_type()` while reading `contract.arithmetic` from the contract. An `f16` contract therefore produces requirements whose resolved type is `tiler::f32@1`. The pair never matches a declaration, so the outcome is `Unknown` and **fails closed** — this is correctness-preserving, not a live wrong answer — but it means no non-`f32` contract can be honoured whatever a profile declares. Filed as `key-numerical-requirements-by-the-contract-s-own-resolved-type`.

## The proposed boundary

Four layers. The packet compiles each against the real production vocabulary; module docs carry the derivation, and this section is the map.

### 1. Shared vocabulary — proposed `tiler_ir::numerics` (`src/shared.rs`)

The single dimension authority, deleting both local lists. `NumericalDimension` (11 cases, `tag`/`from_tag`/`key`/`space`/`index`/`admits`), `CANONICAL_DIMENSIONS`, `DIMENSION_COUNT`, `BehaviourSpace`, `DimensionBehaviour` (+`encode`/`decode`), `ScalarArithmeticSubject`, `ScalarArithmeticSubjectIdentity`, `RelaxationRequirement`, `HonouringMeans`, `PolicyLocus`, `NumericalObligationKey`, and the relocated provenance vocabulary (`FactAuthority`, `FactValidityScope`, `ProvenanceIdentity`, `CompilerBuildRole`, `CompilerBuildIdentity`, `ExecutionEnvironmentIdentity`, `MeasurementContext`, `FactEvidenceBasis`, `FactSourceProvenance`).

**Why `tiler-ir`, when a previous decision eliminated it.** `record-delivered-numerical-realization`'s "Decision — auto-resolved" eliminated relocation and chose opaque compiler-minted key bytes. It also recorded its own reopening trigger: *"A consumer of the artifact that must reason over the means rather than compare them."* That trigger has fired twice over. ADR 0076 item 4 names exactly that consumer — one comparing generated output against a CPU reference, which must distinguish an emulated dimension from a natively honoured one. And defect 3 shows the opaque-key mechanism cannot carry the record even for *comparison*, because `key()` is not injective. `accept-the-delivered-realization-artifact-surface` independently withdrew the opaque-bytes premise. This is an explicit supersession of that elimination, on evidence the elimination itself named as sufficient to reopen it.

**The siting follows precedent rather than inventing one.** `AvailabilityPhase` is ADR 0043 target-fact provenance vocabulary, it is defined in `tiler_ir::program::abi:114-163`, and both siblings name it by re-export (`tiler-compiler`: `target/feasibility.rs:196`; `tiler-artifact`: `program/expr.rs:20`). Every behaviour vocabulary is already a `tiler_ir::schedule` type, and `DimensionBehaviour` is a sum over exactly those five. So the relocation moves no meaning into the semantic graph: `tiler_ir::numerics` sits beside `tiler_ir::schedule`, not inside `tiler_ir::semantic`, and the target-aware *assessment* stays entirely in `tiler_compiler::target`.

**Wire tags are preserved byte for byte.** `FactAuthority::ExternalProfile` is `0x06` and `MeasuredProfile` is `0x07`; `FactValidityScope::MeasuredEnvironment` is `0x05`. These are deliberately out of declaration order in production because the variants were inserted after the neighbouring tags were committed. Renumbering them into tidy order during a move would silently change `tiler.target-profile.descriptor.v10` for every profile declaring a measured fact — an identity-domain step this ticket is forbidden to take and one a diff would not make obvious.

### 2. Compiler evidence view — proposed `tiler_compiler::session` (`src/compiler_view.rs`)

```rust
impl PlanAlternative<'_> {
    pub fn delivered_realization(&self) -> DeliveredRealizationView<'_>;
}
```

`DeliveredRealizationView` exposes `profile_key`, `profile_descriptor`, `scalar_arithmetic() -> impl ExactSizeIterator<Item = SelectedScalarArithmetic<'_>>`, and `obligations() -> impl ExactSizeIterator<Item = SelectedObligation<'_>>`. Every view is `Copy`, borrowed, and constructor-free: no `Arc`, no vector, no canonical encoder crosses the boundary, so a consumer can read typed selected evidence without forging a compiler-verified fact.

It is deliberately **one** view rather than three iterators a caller zips itself, because the total boundary must cross-check subjects, coverage, obligation associations, and the evidence pool together — and three iterators can be zipped wrongly.

**Nothing like this exists today.** Exact check: `grep -rln "HonouredNumericalFact" --include="*.rs" .` returns no files. `SelectedPlan::honoured` is `pub(crate)` inside the private `mod selection`; `ProvenEvidence`, `NumericalHonourabilityFact`, `HonouringMeans`, and `DimensionBehaviour` are `pub(crate)` inside `pub(crate) mod target::{feasibility, honourability}`.

### 3. Artifact record — proposed `tiler_artifact::program::realization` (`src/record.rs`, `src/codec.rs`)

```rust
pub struct DeliveredRealizationRecord { /* private */ }

impl DeliveredRealizationRecord {
    pub fn profile(&self) -> &TargetProfileRef;
    pub fn subjects(&self) -> &[NumericalPolicySubject];
    pub fn obligations(&self) -> &[NumericalObligation];
    pub fn evidence(&self) -> &[TargetEvidence];
    pub fn bindings(&self) -> &[EntryPolicyBinding];
    pub fn scalar_arithmetic(&self, subject: &ScalarArithmeticSubjectIdentity)
        -> Option<ScalarArithmeticView<'_>>;
    pub fn canonical_bytes(&self) -> Vec<u8>;
}

pub enum DispositionView<'a> { NotRequired, Required(&'a [NumericalObligation]) }
```

**Required, not optional.** There is no `Option`, no `UnrecordedRealization`, and no reader that can return an absent record. The staged draft's `require_recorded` was migration state that the required terminal record contradicts.

**The dense-array representation.** `ScalarArithmeticRecord` stores the subject identity once plus `[DimensionBehaviour; 11]` and `[AssessmentDisposition; 11]`, indexed by one exhaustive `NumericalDimension::index`. Eleven named fields are eliminated: they would duplicate the dimension set in the type system, force a public signature change per dimension, and require an eleven-arm match in the record, the builder, and the codec. Completeness survives — an array of length `DIMENSION_COUNT` cannot be missing a dimension — with one place to change.

**Why the artifact carries the resolved type as bytes.** `ResolvedValueType::canonical_encoding` is one-way; `tiler-ir` publishes the collision-free encoder and no decoder, under the accepted policy that decoding yields a dispatch record rather than reconstructed compiler IR. That is the right shape rather than a limitation: the exact canonical bytes **are** the full identity, collision-free by construction, and their leading family discriminant distinguishes nominal from parameterized from encoded-numeric. The arithmetic type rides beside them as a decodable tag, because a consumer must be able to read which dtype a record speaks for.

**Canonical ordering.** Evidence table first (sorted and deduplicated by canonical row bytes), then subjects (sorted by family tag + subject identity), then obligations (sorted by `(subject index, dimension tag, locus)`), then entry bindings (sorted by `(entry, subject)`). Rows referencing a table are written after it. Dispositions are **derived** at `build()` from the canonical obligation slice, never declared, so a `Required` range is contiguous by construction and cannot name a row that is not there.

**Failure vocabulary.** `DeliveredRealizationError` (12 rules) on the producer side; `RealizationCodecError` (19 rules) on decode and cross-check. Both carry a `rule()` identifier and an `ALL_RULES` inventory the perturbation harness counts its coverage against.

### 4. Build translation — proposed `tiler_build::realization` (`src/translate.rs`)

```rust
pub fn translate(
    view: DeliveredRealizationView<'_>,
    profile: &TargetProfileRef,
    entry_subjects: &[(u32, ScalarArithmeticSubject)],
) -> Result<DeliveredRealizationRecord, RealizationTranslationError>;
```

`tiler-build` is the only crate that can see both authorities, which is the same derivation `express-metal-honourability-in-the-shared-form` recorded for the Metal projection. The translation matches every subject, dimension, structured means, and provenance variant, and never reconstructs evidence from flags, target names, neighbouring dtypes, profile digests, or outer value shape — because ADR 0076's forcing measurement is that a readable proxy states the opposite of the truth. Dispositions are not translated at all; carrying them beside the obligations would be the same claim twice, and the two copies could disagree.

## Where each cross-check is proved

| Claim | Proving layer |
| --- | --- |
| The policy subject, the obligation loci, the required behaviours, and every `NotRequired` | **Compiler**, from the checked plan |
| The translated subject and obligation references agree with the compiler view | **`tiler-build`** (`ObligationSubjectNotOffered`, profile equality) |
| The record's profile equals the artifact's single `TargetProfileRef` | **Artifact** (`ProfileMismatch`) |
| Every packaged entry references an existing policy subject | **Artifact** (`UnboundEntry`, `DanglingReference`) |
| The eight overlapping resolutions equal every bound entry's `NumericalFacts` | **Artifact** (`OverlappingRealizationMismatch`) |
| Canonical order, references, coverage ranges, tags, provenance completeness | **Artifact decode** |
| The semantic meaning of the entry↔subject association | **Compiler/build producer** — the artifact validates the encoding and documents that it cannot derive the arithmetic type from a dispatch record |

**The honest boundary.** The artifact builder validates internal consistency and provenance; it cannot provide authenticity and cannot re-run the compiler's consumption analysis. **An untrusted producer can write a wholly self-consistent record, including a false `NotRequired`, and every check here passes.** Decode verifies integrity, canonical coverage, references, and associations; it does not upgrade producer assertions into independently proved semantics. Ordinary checked production goes through `tiler-build`; any retained low-level seam accepts typed producer assertions and must be named so.

`NotRequired` is nonetheless a *written byte* rather than recoverable silence, on the precedent `docs/artifact-abi.md` already sets for the synchronization realization: "an entry requiring no realization writes `0x00` rather than nothing."

## Evidence

All ten stages pass. Counts are the harness's own output.

| Stage | What it establishes |
| --- | --- |
| `subject-validation` | 28 recognized `(type, arithmetic)` pairs refused by the **production** validator — bool, i32, decimal64, complex-over-f32, strict-affine u4, and two MX element formats × 4 arithmetic types — plus an owner-namespaced `acme::posit16@1` refused for all four, and the two near-miss cases (`u32` = f32's width, wrong class; `f16` = f32's class, wrong width). 2 governed pairs admitted. |
| `complete-eleven-dimension-subject` | 11 dimensions covered; 3 required, 8 `NotRequired`; **3 distinct loci on one `(type, dimension)`**, two carrying different legal requirements, each with its own evidence. |
| `two-dtype-evidence` | One record, `f32` flushing and `f16` preserving on `InputSubnormals`, 2 subjects, 2 evidence rows, no collision, exact round trip. |
| `relaxation-payload-distinct` | Two conditional means differing only in relaxation payload share a `label()` (documented non-injective) and differ in canonical identity, in record bytes, and after decode — a reader can still say *which* relaxation applied. |
| `builder-canonicalizes` | Two declaration orders produce byte-identical records; 4 obligations cite 2 deduplicated evidence rows; a duplicate rejects and leaves the draft usable. |
| `zero-obligation-subject` | A selected contract with no obligations yields a complete 11-dimension subject, all `NotRequired`, written explicitly across 258 canonical bytes. |
| `resolved-type-identity-families` | Nominal, parameterized, and encoded-numeric identities are 3 distinct subjects; a namespace change is a distinct identity; none calibrates an arithmetic subject. |
| `canonical-round-trip` | 1498 bytes round-trip exactly and re-encode identically; the 8 overlapping resolutions agree with the entry's realization. |
| `build-translation` | 1 subject and 1 obligation translated exhaustively; 10 dispositions derived as `NotRequired`; a profile disagreement is refused rather than re-attributed. |
| `perturbations` | 38 perturbations, 25 distinct rules — **every rule in both `ALL_RULES` inventories watched refusing.** |

Each perturbation asserts the **exact** rule identifier, not merely that something failed: a perturbation tripping a neighbouring check would otherwise report a pass for a check nothing exercised. The harness also counts its coverage against the named rule inventories, so a rule added without a perturbation fails the run rather than quietly shrinking what has been watched.

Two shapes are used. **Structural** perturbations rebuild a deliberately non-canonical record and re-encode it with the one production encoder, so a perturbation cannot pass by disagreeing with the encoder it tests. **Tag** perturbations poke one byte at an offset computed from the record's own field widths, because an unknown tag is by construction a value no Rust value can hold. Six shapes are wire-only — a behaviour from another space, an evidence behaviour mismatch, incomplete provenance, a phase escape, an empty `Required` range, and a non-component locus carrying an ordinal — because the typed producer path cannot express them, which is precisely why decode must check them independently.

Every precondition is an assertion rather than an `if`. An earlier revision guarded the malformed-locus perturbation with `if let`, the fixture had no component-locus row, and the perturbation **silently did not run while the stage reported success** — the exact failure `AGENTS.md` names. The fixture now carries a component obligation and the guard is `expect`.

## Proposed contract text

Drafted here because this ticket does not edit the contracts operatively. `wire-the-delivered-realization-record-into-the-artifact` applies them after ratification.

### ADR 0076 item 4 — replace the "each dimension's means" sentence

Current first Proposal paragraph of item 4 reads: *"A produced artifact carries a first-class record of the numerical realization actually delivered: the resolved contract, complete over every dimension, with each dimension's means of honouring it, together with the target facts relied on and the identity of the profile that declared them."*

Proposed replacement:

> A produced artifact carries a first-class record of the numerical realization actually delivered, keyed by the compiler-produced scalar-arithmetic policy subject rather than by dimension alone. For each such subject the record carries the resolved contract complete over every governed dimension, and for each dimension an explicit **assessment disposition**: either compiler-produced `NotRequired` for every packaged route, or `Required` naming a non-empty canonical range of locus-specific obligations. Each obligation names the program occurrence and policy locus that produced it, the behaviour that locus requires, and the target evidence relied on; each evidence row carries the declared behaviour, the structured honouring means including any declared-relaxation payload, and the complete provenance item 3 governs. A dimension no packaged route consumes therefore carries no target fact at all, rather than a fabricated one. The record states the union of obligations every packaged variant and stage that routing may select relies on; it never states what was "actually exercised", because the artifact exists before any route executes.

### ADR 0076 item 4 — add, after the "why flags cannot substitute" measurement

> **Consequence — the means cannot be carried as its presentation key.** `HonouringMeans::key` returns one constant string for every `SupportedOnlyUnderDeclaredRelaxation` value whatever relaxation it names, so two artifacts honouring one contract under different relaxations would mint identical bytes. A record that carried that key would be unable to answer the question this item exists to make answerable. The means is therefore carried structurally, with its relaxation payload, and the non-injective spelling is a presentation label under ADR 0074 convention 2 rather than an identity.

### `docs/numerical-semantics.md` — append to "Per-dimension honourability, and how it composes with feasibility"

> **The delivered record refines the key by a policy locus.** A profile's declaration is keyed by `(subject, dimension)`, and that is the right key for a *target fact*: a target honours a behaviour for a dtype, not for a position in a program. A caller's requirement is not the same shape. ADR 0011's per-operation restrictions attach to a position, so one `f32` operation's accumulator and its observable materialization boundary can carry different legal requirements, and a record keyed by type alone would keep whichever was written last. The delivered-realization record therefore carries `(subject, dimension, locus)` obligation rows above the dtype-wide ceiling, where a locus names the program occurrence and one of input, computation, accumulator, result, component, or materialization. The ceiling and the obligations are separate statements and neither is derived from the other.

### `docs/artifact-abi.md` — new subsection after the numerical-facts fact at line 168

> **Fact — the delivered-realization record is separate from the entry's numerical facts, and the two are cross-checked.** An executable entry's resource requirements and numerical realization state the eight dimensions the bounded operation vocabulary can consume, dtype-free. The artifact-wide delivered-realization record states, per compiler-produced scalar-arithmetic subject, all eleven governed dimensions plus the means and provenance by which each required one is honoured. The two overlap on eight dimensions and must agree: construction and decode reject a record whose resolution differs from any bound entry's own statement. Because an entry's realization carries no arithmetic type, the record additionally carries one explicit entry-to-subject association per packaged entry; the neutral artifact validates that the association is encoded and references an existing subject, and the compiler and `tiler-build` producer are what prove its semantic meaning. An entry with no association, a record naming a profile other than the artifact's single `TargetProfileRef`, a dangling obligation or evidence reference, and an unknown record-family tag each reject.

## What this packet deliberately does not do

- **No production edit.** `crates/tiler-artifact/src/program/realization.rs` and every other production module are byte-identical.
- **No identity or schema advance.** `DELIVERED_REALIZATION_DOMAIN` is spelled `…delivered-realization.v2` inside the spike and is **not** applied anywhere; the artifact identity domain stays `tiler.artifact-program.v14` and the manifest schema stays 12.0. Advancing them, folding `canonical_bytes` into `encode_identity`, and recomputing every pinned identity on the merged tree are `wire-…`'s, deliberately, because that step is executed completely or not at all.
- **No contract edit.** The text above is drafted, not applied.
- **No `spikes/README.md` catalog entry.** That file maps to `contracts/navigation`, which the live ticket `cite-adr-0095-in-the-milestone-6-distributivity-framing` holds. `spikes/numerics/README.md` — inside this ticket's `research/numerics` scope — lists this packet; the top-level entry belongs to whichever ticket next holds navigation, and is recorded on this ticket as a remainder.

## Known gaps in the producer, filed rather than absorbed

**The compiler cannot produce locus-keyed obligations today.** Exact check: `grep -rni "locus" --include="*.rs" crates/` returns nothing. `StrictF32NumericalContract` is one flat record for one arithmetic type, and `policy::dimension_requirements` projects it into exactly eight **whole-program** requirements. The record's shape is right — a dtype-wide ceiling genuinely cannot express two `f32` loci with different legal requirements, which is what stage `complete-eleven-dimension-subject` exhibits — and the producer for it does not exist. Until it does, a conforming producer emits one obligation per consumable dimension at the computation locus of the occurrence that consumes it, which is exactly as much as the compiler can honestly say. `derive-per-locus-numerical-obligations` owns the remainder.

**Requirements are built with a hard-coded `f32` resolved type.** See the sixth finding above. Filed as `key-numerical-requirements-by-the-contract-s-own-resolved-type`.

**Lookup cost is unmeasured and deliberately so.** `scalar_arithmetic` binary-searches the subject slice. At today's one-subject scale a linear scan would win; the ticket asks for a targeted benchmark rather than an asymptotic assumption, and that benchmark belongs with a real multi-subject portfolio. Recorded as an open question rather than answered by assertion.
