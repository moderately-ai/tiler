---
id: derive-the-conformance-evidence-ledger-cells-from-executed-runs
title: Derive the conformance-evidence ledger cells from executed runs
status: todo
priority: p2
dependencies: []
related: [survey-what-belongs-in-the-conformance-crate, decide-whether-the-bf16-conformance-evidence-cell-overstates, own-the-dtype-support-maturity-matrix, conform-the-bf16-vertical-end-to-end]
scopes: [research/verification, implementation/conformance, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, documentation, ledger, trigger-fired]
---
## Question

Which support-matrix and ledger cells can a conformance run **derive** from a run that happened, and which are claims no run can make? And can the maturity ladder and the evidence ladder be **stamped** by a harness, or must they stay a writing convention?

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md), which answered as far as reading settles it and parked the build. The findings below are the starting state; do not re-derive them.

## What the survey established

**Fact — exactly one column is derivable, and it is already the one that drifted.** `docs/dtype-support.md`'s physical/execution matrix has nine columns. Eight of them — physical carrier, ABI, optimizer legality, kernel vocabulary, backend lowering, backend execution, runtime semantic validation, target-family dispatchability — report what *authority exists at a layer*, which is a statement about source and about accepted decisions. A run cannot observe them; it can only fail if one is missing. The ninth, **`Conformance evidence`**, reports whether a checked run composed the layers, which is exactly what an executed cross-layer comparison observes about itself.

**Fact — that column is the one with a measured drift record.** [`decide-whether-the-bf16-conformance-evidence-cell-overstates`](decide-whether-the-bf16-conformance-evidence-cell-overstates.md) found the BF16 cell reading a bare `tested guarantee` — the same two words as the `f32` cell, which rests on a device-executed thirty-case comparison — while nothing had dispatched a BF16 kernel. It now reads `tested guarantee, per-layer corpora only; no end-to-end run`. That correction is precisely what a run-derived cell produces for free.

**Fact — the ledger's own repair mechanism is manual and visible.** `docs/dtype-support.md` repairs by striking prose (`~~...~~`) and appending dated "Corrected YYYY-MM-DD" paragraphs, because `AGENTS.md` records that documentation has no automated validator.

**Fact — a run carries the qualifiers a cell needs.** `publish-an-l3-contraction-cell-through-the-accepted-route` compares six environment fields (host, OS build, Xcode, SDK, offline compiler, architecture) against the retained record before comparing anything, and `conform-the-bf16-vertical-end-to-end` requires host, OS build, Metal version, GPU, and family recorded with no generalization beyond the row that ran. Those are the *bounds* every tested cell in the ledger is supposed to carry and several do not.

**Fact — the maturity ladder has no Rust representation and three spellings.** `AGENTS.md:56` states it as reserved type / architectural seam / implemented support / tested guarantee. `docs/dtype-support.md:21` uses a five-value variant with `implemented mechanism` in place of `implemented support` and an `absent/unsupported` rung below. `docs/roadmap.md:441` maps seven R1–R7 rungs onto the same four claims. No enum anywhere in `crates/` names them.

**Fact — the evidence ladder has four Rust representations and they disagree.** `ConformanceEvidenceClass` (`crates/tiler-ir/src/semantic/accuracy/evidence.rs:50`, `pub`, five variants, spells the top rung `FormalProof`), `FusionEvidenceClass` (`crates/tiler-compiler/src/fusion_legality.rs:90`, `pub(crate)`, `SoundProof` spelling, two variants reserved and never constructed), `IndexDomainEvidence` (`crates/tiler-ir/src/index/predicate.rs:89`, no `NormativeGuarantee` rung, `Empirical` documented as never emitted), and `EvidenceBasis` (`crates/tiler-compiler/src/explain.rs:379`, `pub(crate)`, seven variants including `CheckedInvariant` and `Assumption`).

## The answers, as far as reading settles them

**A run can derive one cell and three qualifiers.** For the `(family, layer) = (dtype family, Conformance evidence)` cell it can emit: whether an end-to-end composition executed; the exact operation set the run covered; the environment row it is bounded to; and whether the measured half was available. Everything else in both matrices is a claim about source, and a harness reading it would be a second authority over what the compiler declares.

**The maturity ladder cannot be stamped, and it should not be.** A rung is a judgement about *what authority exists*, and three of its four rungs describe states with no run at all — a reserved type and an architectural seam are exactly the cases where nothing executes. Only the top rung has an executable witness, and even there the ladder's own rule is that a tested guarantee "must not cover an untested composition", which is a scoping judgement about what the test covered rather than a fact about it passing. What a harness *can* do is refuse to let a `tested guarantee` cell exist without a run naming it — a consistency check between a ledger cell and a run identifier, not a stamp.

**The evidence ladder can be stamped, but not by this crate as it stands.** The class is already a typed value the producing authority mints: `ConformanceEvidenceClass` is `pub` and carries `ALL`, `spelling()`, `tag()`, and `discharges_hard_requirement()`. A run can *report* the class its inputs carried; it must not *assign* one, because assigning would make the conformance crate a second authority over a claim the semantic and compiler layers own — the first anti-goal. Before any stamping is possible the four types have to be reconciled or their differences declared deliberate, and that reconciliation is a public-boundary question under ADR 0075.

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
