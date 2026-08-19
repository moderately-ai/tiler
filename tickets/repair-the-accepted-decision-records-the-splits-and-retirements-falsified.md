---
id: repair-the-accepted-decision-records-the-splits-and-retirements-falsified
title: Repair the accepted decision records the splits and retirements falsified
status: todo
priority: p2
dependencies: []
related: [repair-the-navigation-and-contract-docs-the-audit-falsified, repair-the-research-records-the-key-replacement-and-splits-falsified, repair-the-ticket-population-facts-the-splits-and-retirements-falsified]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, audit]
---
## User-visible outcome

No accepted ADR states a variant count, a public-surface claim, or a source path that the tree contradicts. Each stale claim is retired in place with a dated correction in the file's own convention, so a reader arriving at an accepted record is not told something false about current code.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit, then re-verified site by site by the coordinator at `de18ebdb` before filing. Every Fact below was read at that base; two were found **worse** than the audit reported and are stated here in their corrected form.

**Fact — ADR 0071 states a variant count that is three times off.** Anchor `remains closed and three-variant` (`docs/decisions/0071-*.md`). `ScalarProgram` (`crates/tiler-ir/src/schedule/model.rs`, anchor `pub enum ScalarProgram`) has **nine** variants: `PointwiseF32`, `PointwiseBf16`, `StrictAffineU4Dequantize`, `StrictSerialSum`, `FusedMultiplyAddSerialSum`, `SquaredSerialSum`, `SquaredSerialSumThenEpilogue`, `StrictTensorContraction`, `StrictSerialMaximum`. The sentence's *surrounding* argument — that the schedule layer's vocabulary is closed while the index layer's is an open registry-governed SSA graph — is unaffected and must be preserved; only the count is stale.

**Fact — ADR 0094 carries two false counts in one sentence, not one.** Anchor `still has exactly one variant with no subgroup-lane source` (`docs/decisions/0094-*.md`). The audit reported only the `LocalCoordinateSource` half. Read in full, the same sentence also asserts "`ReductionTopology` still has exactly five variants". Both are false at this base: `ReductionTopology` (`crates/tiler-ir/src/schedule/model.rs`, anchor `pub enum ReductionTopology`) has **seven** — `None`, `Serial`, `MultiPass`, `Contraction`, `LiveContraction`, `CooperativeWorkgroup`, `CooperativeContraction` — and `LocalCoordinateSource` (`crates/tiler-ir/src/schedule/cooperative.rs`, anchor `pub enum LocalCoordinateSource`) has **two**, `LocalLinearInvocation` and `LocalWorkgroupPosition`. The sentence's *claim* is that acceptance implemented nothing, and the counts are its evidence; repair each count without weakening that claim, and check the sentence's three remaining conjuncts (`MemoryScope` has no `Subgroup` variant, `CapabilityAxis` declares no subgroup width, no target profile declares a subgroup realization subject) rather than assuming they still hold. **A repair that fixes one count and republishes the other as current is the exact failure this ticket exists to prevent.**

**Fact — ADR 0079 names a path the inventory no longer pins.** Anchor `The inventory pins that exact path and name`. The record names `exhaustive_enum_population` in `crates/tiler-artifact/src/program/codec/tests.rs`; that file does not exist, and `WORKSPACE_LOCAL_MACRO_RULES` in `crates/tiler/tests/workspace_unsafe_sites.rs` pins `crates/tiler-artifact/src/program/codec/tests/vocabularies.rs` instead. The companion claim "exactly two same-file invocations" was verified still true by the audit; re-verify rather than carry it.

**Fact — two accepted public-boundary ADRs still name a retired public adapter.** `declare_metal_f32_subnormal_behaviour` is declared `fn`, not `pub fn`, at `crates/tiler-build/src/metal_declaration.rs` (anchor `fn declare_metal_f32_subnormal_behaviour`), and is absent from `crates/tiler-build/src/lib.rs`. Its only non-test caller is inside `BoundMetalCompileDeclaration::declare`. Two records still describe it as public surface: `docs/decisions/0076-declare-target-honourable-numerical-realizations.md`, anchor `No production caller uses it yet` (the production caller is now the only caller), and `docs/decisions/0078-name-the-intended-public-extension-seams.md`, anchor `now owns the bounded caller-vouched F32 projection`, where it is cited as settled public-seam evidence. ADR 0076 already carries a `**Superseded in part, 2026-08-18:**` clause, so the retirement reached one paragraph of that record and not the rest — read the whole file, not the anchor's paragraph.

**Fact — two ADRs cite a module path deleted by the index split.** `crates/tiler-ir/src/index/refinement.rs` does not exist; the directory `crates/tiler-ir/src/index/refinement/` replaced it. Cited by `docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md` (twice: anchors `encode_executable_coverage_identity` and `IndexRefinementExecutableCoverageIdentity`) and `docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md` (twice, in the evidence-boundary and reconsideration-trigger paragraphs). **The audit named this ADR as `0104-derive-executable-coverage-from-the-refined-index-domain`, which is not a file in this tree** — trust the anchors here, not that filename. Locate each named symbol in the split modules and repoint; ADR 0104's note at its own anchor `The stale text is the source comment, not this record` describes a *different* staleness that is out of this ticket's scopes and must be left alone.

**Fact — one further ADR cites a deleted test path.** `docs/decisions/0079-*.md` also cites `crates/tiler-artifact/src/program/tests.rs`, which no longer exists; that module is now the directory `crates/tiler-artifact/src/program/tests/`.

Additional ADRs in this scope cite `crates/tiler-ir/src/schedule/builder.rs`, which is likewise deleted (now `crates/tiler-ir/src/schedule/builder/`): `0012`, `0014`, `0022`, `0074`, `0097`, `0100`. `crates/tiler-compiler/src/request.rs` **still exists** as the spine beside its new `request/` submodules, so a citation to that path is not automatically stale — but a symbol it names may have moved into a submodule. Treat path and symbol separately.

## Required work

- Re-audit every Fact above at your actual base before editing, per the stale-Facts rule, and report a per-Fact verdict first. Counts here were derived by reading each enum body; re-derive them rather than trusting this ticket.
- Repair each site with a dated correction in the file's existing convention. Preserve the surviving argument and retire only the false claim. Where retired wording is quoted inside a correction note, keep it on one source line so sibling anchors still resolve into the note.
- For every path repair, confirm the replacement by locating the named **symbol**, not by assuming a directory rename. Do not repoint a citation at a file that does not define what the sentence claims.
- Enumerate the whole scoped population rather than only the sites listed here: `grep -rlF 'schedule/builder.rs' docs/decisions/`, the same for `index/refinement.rs`, `program/tests.rs`, and `codec/tests.rs`. Report the census with its counts so a later reader can tell coverage from sampling.
- Where a claim is a **count of a live vocabulary**, prefer wording that fails loudly over a bare number where the record's argument permits it, and say in the correction which enum the count is of, so the next reader can re-derive it.

## Non-goals

Navigation and contract documents (`docs/status.md`, `docs/roadmap.md`, `docs/glossary.md`, `docs/artifact-abi.md`, `docs/dtype-support.md`, `docs/backends/metal.md`, `docs/correctness-and-testing.md`), research records under `docs/research/`, ticket bodies, and any source change — each is a sibling ticket's scope. Do not widen an ADR's decision, only its accuracy about the tree.

## Closes when

Every site above is repaired or verified already-correct with the evidence stated, the four path censuses are quoted with counts, `make citations` is green, and no accepted ADR in `docs/decisions/` states a variant count or source path this tree contradicts.
