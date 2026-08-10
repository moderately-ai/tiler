---
id: derive-the-conformance-evidence-ledger-cells-from-executed-runs
title: Derive the conformance-evidence ledger cells from executed runs
status: in-progress
priority: p2
dependencies: []
related: [survey-what-belongs-in-the-conformance-crate, decide-whether-the-bf16-conformance-evidence-cell-overstates, own-the-dtype-support-maturity-matrix, conform-the-bf16-vertical-end-to-end]
scopes: [research/verification, implementation/conformance, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, documentation, ledger, trigger-fired]
claimed_from: todo
assignee: sol-conformance-ledger
lease_expires_at: 1786405848
---
## Question

Which support-matrix and ledger cells can a conformance run **derive** from a run that happened, and which are claims no run can make? And can the maturity ladder and the evidence ladder be **stamped** by a harness, or must they stay a writing convention?

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md), which answered as far as reading settles it and parked the build. The findings below are the starting state; do not re-derive them.

## What the survey established

**Fact — exactly one column is derivable, and it is already the one that drifted.** `docs/dtype-support.md`'s physical/execution matrix has nine columns. Eight of them — physical carrier, ABI, optimizer legality, kernel vocabulary, backend lowering, backend execution, runtime semantic validation, target-family dispatchability — report what *authority exists at a layer*, which is a statement about source and about accepted decisions. A run cannot observe them; it can only fail if one is missing. The ninth, **`Conformance evidence`**, reports whether a checked run composed the layers, which is exactly what an executed cross-layer comparison observes about itself.

**Fact — that column is the one with a measured drift record; the ledger restatement is dated 2026-08-07, and this ticket's trigger re-check that moved status to `todo` is dated 2026-08-09.** [`decide-whether-the-bf16-conformance-evidence-cell-overstates`](decide-whether-the-bf16-conformance-evidence-cell-overstates.md) found the BF16 cell reading a bare `tested guarantee` while nothing had dispatched a BF16 kernel and qualified the cell to `per-layer corpora only; no end-to-end run`. The later vertical restated the cell under its own Closes when and the 2026-08-07 ledger correction under `the run exists and the cell is restated` (the decide ticket has no reconsideration trigger). The current ledger's source-safe cell text is `one device run crossing neither the optimizer, the artifact envelope, nor the routing commit`; that dated correction preserves the prior no-run state as history. This two-step drift and repair is precisely the kind of boundary a run-derived comparison should make loud.

**Fact — the ledger's own repair mechanism is manual and visible.** `docs/dtype-support.md` repairs by striking prose (`~~...~~`) and appending dated "Corrected YYYY-MM-DD" paragraphs. `AGENTS.md` records that documentation is manually maintained; the only mechanical check is `make citations` (link resolution), and maturity-cell content is not validated automatically.

**Fact — a run carries the qualifiers a cell needs.** At this base, `crates/tiler-conformance/src/retained_record.rs` compares six environment fields (device, gpu-family, architecture, os, offline-compiler, sdk) against the retained record before comparing digests; `xcode` is present in the record and deliberately not compared. Historically, `publish-an-l3-contraction-cell-through-the-accepted-route`'s 2026-08-05 Outcome measured six fields that included Xcode rather than the current vocabulary. Separately, `conform-the-bf16-vertical-end-to-end` requires host, OS build, Metal version, GPU, and family recorded with no generalization beyond the row that ran. Those are the *bounds* every tested cell in the ledger is supposed to carry and several do not.

**Fact — the maturity ladder has no Rust representation and three spellings.** `AGENTS.md`, anchor `reserved type, architectural seam, implemented support, tested guarantee`, states the four claims. `docs/dtype-support.md`, anchor `absent/unsupported`, uses a five-value variant with `implemented mechanism` in place of `implemented support`. `docs/roadmap.md` maps seven R1–R7 rungs onto the same claims. No enum in `crates/` names that documentation ladder.

**Fact — the evidence ladder has four Rust representations and they disagree.** `ConformanceEvidenceClass` (source anchor `pub enum ConformanceEvidenceClass`, five variants, top rung `FormalProof`), `FusionEvidenceClass` (source anchor `pub(crate) enum FusionEvidenceClass`, `SoundProof` spelling and two reserved classes), `IndexDomainEvidence` (source anchor `pub enum IndexDomainEvidence`, no `NormativeGuarantee` rung and `Empirical` documented as never emitted), and `EvidenceBasis` (source anchor `pub(crate) enum EvidenceBasis`, seven variants including `CheckedInvariant` and `Assumption`).

## The answers, as far as reading settles them

**A run can derive one cell and three qualifiers.** For the `(family, layer) = (dtype family, Conformance evidence)` cell it can emit: whether an end-to-end composition executed; the exact operation set the run covered; the environment row it is bounded to; and whether the measured half was available. Everything else in both matrices is a claim about source, and a harness reading it would be a second authority over what the compiler declares.

**The maturity ladder cannot be stamped, and it should not be.** A rung is a judgement about *what authority exists*, and three of its four rungs describe states with no run at all — a reserved type and an architectural seam are exactly the cases where nothing executes. Only the top rung has an executable witness, and even there the ladder's own rule is that a tested guarantee "must not cover an untested composition", which is a scoping judgement about what the test covered rather than a fact about it passing. What a harness *can* do is refuse to let a `tested guarantee` cell exist without a run naming it — a consistency check between a ledger cell and a run identifier, not a stamp.

**The evidence ladder can be stamped, but not by this crate as it stands.** The class is already a typed value the producing authority mints: `ConformanceEvidenceClass` is `pub` and carries public `ALL`, `spelling()`, and `discharges_hard_requirement()` (private `const fn tag` is an internal encoding helper, not part of the public report surface). A run can *report* the class its inputs carried; it must not *assign* one, because assigning would make the conformance crate a second authority over a claim the semantic and compiler layers own — the first anti-goal. Before any stamping is possible the four types have to be reconciled or their differences declared deliberate, and that reconciliation is a public-boundary question under ADR 0075.

## What a build would have to decide

1. The identity a derived cell is keyed on — a run identifier, or the `(family, operation set, environment row)` triple, and what happens when two runs disagree.
2. Whether the derived text is generated into `docs/dtype-support.md` (which makes a hand edit a merge conflict) or held beside it and *compared* by a check (which leaves the prose authoritative and makes drift a failing test). The survey's reading favours the second: the ledger's cells carry prose justification a generator cannot write, and the failure mode being closed is a cell that overstates, which a comparison catches without owning the file.
3. Whether the four evidence enums are reconciled first.

## Trigger

**Fires when at least two executed cross-layer runs live in `crates/tiler-conformance` and name different ledger cells.** One run cannot demonstrate a derivation; it can only be transcribed, which is what the tree already does. Two runs naming two cells is the smallest population where a generator or a comparison beats a hand edit.

Reproduce the check:

```sh
grep -rln "result_sha256\|ReferenceEvaluator" crates/tiler-conformance/src crates/tiler-conformance/tests 2>/dev/null | wc -l
```

## Trigger check log

- 2026-08-07 — **not fired.** `crates/tiler-conformance` contains exactly two files, `Cargo.toml` and `src/lib.rs`, and `src/lib.rs` holds a module header and no items. The command above returns `0`.
- 2026-08-09 — **fired.** The crate is no longer an empty shell: it carries the dispatched `f32` serial-sum/contraction evidence and the separate pure-BF16 vertical, with retained result hashes, reference evaluation, exact environment qualification, and distinct ledger claims. The ticket's own reproduction now returns eight files rather than zero, including `serial_sum.rs`, `bf16_vertical.rs`, `envelope.rs`, and `publication/proof.rs`. Two distinct conformance cells therefore exist and the comparison/derivation question is no longer speculative. The ticket moves to `todo`; `implementation/conformance` and `contracts/navigation` are declared because the executable evidence and `docs/dtype-support.md` are the actual work surface. Any reconciliation of the public evidence enums remains a Tom-reviewed boundary and must stop before an unaccepted API edit.

## Source audit — 2026-08-10 at `c34f110a11fb922bdbcb9a54455fbf457f3e1523`

Completed before editing. Every Fact was re-read at the exact base; none was false, imprecise, or purpose-changing, so no Fact repair was required.

1. **Verified — one of nine physical/execution columns is run-derived.** [`docs/dtype-support.md`](../docs/dtype-support.md), source anchor `| Family | Physical carrier and encoding |`, has nine layer columns after `Family`; the first eight name source/declaration authorities and the last is `Conformance evidence`. The survey's anchor `Which ledger cells a run can derive` reaches the same distinction.
2. **Verified — the BF16 drift history and both dates.** [`decide-whether-the-bf16-conformance-evidence-cell-overstates`](decide-whether-the-bf16-conformance-evidence-cell-overstates.md), anchor `the run exists and the cell is restated`, preserves the 2026-08-07 correction; this ticket's trigger log preserves the 2026-08-09 re-check. The live ledger still contains the source-safe fragment `one device run crossing neither the optimizer, the artifact envelope, nor the routing commit`.
3. **Verified — the repair mechanism is manual.** [`docs/dtype-support.md`](../docs/dtype-support.md) uses struck clauses and dated `Corrected` paragraphs; `AGENTS.md`, anchor `Documentation is manually maintained`, limits `make citations` to local-link resolution.
4. **Verified — run qualifiers and the six-field comparison.** [`retained_record`](../crates/tiler-conformance/src/retained_record.rs), anchor `Six fields, and the mapping`, compares device, GPU family, architecture, OS, offline compiler, and SDK and states that Xcode is deliberately unobserved. [`publish-an-l3-contraction-cell-through-the-accepted-route`](publish-an-l3-contraction-cell-through-the-accepted-route.md), anchor `all six agree`, records the historical six-field vocabulary including Xcode. [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md), anchor `Measurement boundary, and nothing generalizes past it`, bounds the run to three operations, one target family, one contract, and its exact row.
5. **Verified — no Rust maturity representation and three written schemes.** `AGENTS.md`, anchor `reserved type, architectural seam, implemented support, tested guarantee`; [`docs/dtype-support.md`](../docs/dtype-support.md), anchor `Cell vocabulary`; and [`docs/roadmap.md`](../docs/roadmap.md), anchor `| R1 | Type-system reservation |`, spell the four/five-value and R1–R7 schemes. A source census found no Rust type representing this documentation ladder.
6. **Verified — four distinct evidence vocabularies.** The source anchors `pub enum ConformanceEvidenceClass`, `pub(crate) enum FusionEvidenceClass`, `pub enum IndexDomainEvidence`, and `pub(crate) enum EvidenceBasis` have the variants and visibilities this ticket states. Their definitions and consumers were read; no reconciliation or export is required for this comparison, so the ADR 0075 public-boundary stop did not fire.

## Outcome — two executed cells compared without stamping authority, 2026-08-10

The private, test-only [`ledger`](../crates/tiler-conformance/src/ledger.rs) now derives the qualifier for exactly two `Conformance evidence` cells from typed declarations beside the routed F32 evidence and pure-BF16 vertical. It reads only the physical/execution matrix's last column and deliberately discards the maturity phrase before the first comma. It cannot assign maturity, inspect the other eight columns, or report/assign any of the four evidence vocabularies.

**Identity and disagreement.** The derived-cell identity is the dtype-family row, because the ledger has one conformance answer per family; stable run identifiers are non-empty, globally unique provenance within that aggregate declaration. `core::mem::variant_count` sizes the two-cell population. Two declarations naming one family, one run identifier naming two families, an empty run population, or a prose mismatch is a hard failure. No ordering or winner is invented.

**Exact bounds retained.** F32 names serial sum and contraction, the 30-case routed matrix plus five retained L3 cells, the exact Apple M4 Max / Apple9 / macOS 27.0 `26A5388g` / metal `32023.921` / SDK 27.0 `26A5388f` / arm64 row, an executed measured half, and the `compile()` / artifact-envelope / routing-commit extent. BF16 names constant/multiply/add over fifteen cases on that same exact row, an executed measured half, and retains the historical `one device run crossing neither the optimizer, the artifact envelope, nor the routing commit` wording. Neither claim generalizes past its named operation, corpus, contract, or environment bounds.

**Subject perturbations.** Changing BF16's manual corpus from fifteen to fourteen made `the_manual_conformance_evidence_qualifiers_match_the_executed_runs` fail with `BF16 Conformance evidence disagrees with executed run records ["pure-bf16-vertical@b7c01815"]`, printing the fourteen-case manual value against the fifteen-case derived value. Changing BF16's typed cell identity to F32 made `the_retained_declaration_population_is_total_unique_and_executed` fail with `two retained declarations disagree about the F32 ledger cell`. Both subjects were restored before the gates.

**Unsupported cases stay unsupported.** No generator rewrites the document; no device run derives source-owned cells; no maturity or evidence class is minted; and the typed population contains only the two families for which this crate retains executed cross-layer records. A future third family must add its variant and declaration together, and a second run for one family must deliberately update that family's aggregate declaration rather than silently competing with it.

**Checks.** `cargo check -p tiler-conformance --all-targets`; package Clippy with warnings denied; 77 package tests passed with one deliberate ignored run; package doc-tests and rustdoc; Linux `x86_64-unknown-linux-gnu` package check and Clippy; `make full` (3,257 workspace tests and 1,121 release tests passed, with only deliberate skips); `make citations` (1,189 pinned citations and 6,454 local links resolved); `tkt lint --format json`; `git diff --check`; and exact-base `tkt guard`. The trigger command still finds exactly eight source files. No source, API, evidence enum, schema, identity, or non-conformance matrix cell changed.
