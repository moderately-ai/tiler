---
id: close-or-retype-the-operand-permutation-inference
title: Close or retype the operand-permutation inference in the first Metal profile
status: in-progress
priority: p2
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, measure-macos-apple9-f32-under-unified-msl4-profile, admit-measured-compile-profile-sources-across-fact-families]
scopes: [research/target-profiles, research/apple-targets, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, metal, target-profile, provenance]
claimed_from: todo
assignee: loop-close-or-ret
lease_expires_at: 1785532186
---
## User-visible outcome

The first authoritative macOS Metal profile's operand-permutation row is either an isolated **Measurement** like its four neighbours, or it is typed in a way that stops a reader mistaking it for one.

## Why this is a remainder rather than a defect

**Fact.** The [compile-profile authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md) labels operand permutation an `Inference` and states why: contraction, reassociation, signed zero, and the NaN/infinity assumptions are each isolated by an emitted fast-math attribute or by a result lane that separates the math modes, and operand permutation has neither. It is delivered by the same `safe` compilation, whose attribute strings carry no relaxation at all, so a permutation relaxation would have to be one the front end applied without recording it.

**Fact.** `tiler_build::BoundMetalCompileDeclaration` declares the row through `declare_measured_permutation`, so it carries `FactAuthority::MeasuredProfile` in the descriptor exactly like its four neighbours, and only a code comment and the ledger distinguish it. That was the brief's instruction for the parent ticket — the label lives in the source documentation — and it is a real asymmetry between what the descriptor says and what the evidence supports.

**Inference.** Nothing downstream is wrong today: the row's *value* (permutation forbidden) is what the `safe` realization delivers, and the compiler consults it identically either way. What is unproven is the strength of the evidence, and the profile descriptor cannot currently express the difference.

## Work

1. Attempt the cheaper close first: cite MSL 4.0's normative statement of what `-fmetal-math-mode=safe` guarantees about operand order, if one exists. A citation retypes the row as an external normative guarantee and closes this outright.
2. Otherwise retain one kernel under the exact offline compiler whose result distinguishes an operand order — the ledger's own stated closing condition — and promote the row to an isolated measurement beside its neighbours.
3. If neither is reachable, decide whether the compiler's fact-source vocabulary should carry a distinct authority class for a fact *inferred from* a measurement, and record the elimination. Adding an authority class is a public compiler boundary and is Tom's.

Do not close this by deleting the row: a profile with no permutation row resolves `Unknown`, which refuses the governed numerical contract the bounded serial sum compiles under.

## Closes when

The row is either an isolated measurement, an external normative guarantee, or a typed evidence class that a descriptor reader can tell apart from its neighbours — and the ledger's fourth outcome is updated to match.

## Outcome

Closed by work item 2: the row is an isolated **Measurement** beside its four neighbours. Work item 3 was never reached, so no fact-source vocabulary change is proposed and none is needed.

**Fact — work item 1 was attempted first and eliminated, with the exact check.** Neither vendored Metal Shading Language specification — `apple-metal-shading-language-specification-v4-2025-10-23.pdf` nor `apple-metal-shading-language-specification-v4.1-2026-06-04.pdf` — contains the string `operand order`, any occurrence of `commut`, or any occurrence of `evaluation order` or `order of evaluation`, case-insensitively, in its extracted text. The sentence that comes closest is MSL 4.0 §1.6.3 page 15: "If you set the option to safe, it disables unsafe floating-point optimizations by preventing the compiler from making any transformations that might affect the results." Read as a universal it is refuted on this exact toolchain by evidence the ledger already carried — `safe` emits `air.compile.denorms_disable` and the F32 subnormal rows measure that changing results — and read narrowly it scopes to the six enumerated fast-math relaxations, which is a statement about what a relaxed mode enables rather than what the strict mode guarantees. The elimination and both readings are recorded in the ledger's new "The route this row did not close by" section so the next reader gets the refutation and not only the conclusion.

**Measurement — what closed it.** `permutation_chain` and `permutation_chain_reordered` carry the same three contributors — `2**30`, `2.0`, `-2**30` — in two orders and differ in nothing else. Under `safe`/`-O2`/`-ffp-contract=off` on `metalfe-32023.883`, each emits three bare `fadd`s and the Apple M4 Max returns `00000000` and `40000000` respectively on all eight lanes, both with an executed witness on `80000000`. The twin is the perturbation: the result lane moves when the source order moves.

**Fact — why this isolates permutation rather than re-reading reassociation.** ADR 0014 makes the two independent permissions, and the permuted value is unreachable by any parenthesization of the canonical leaf order. Four leaves admit exactly five full binary trees; `test_the_permutation_probe_is_unreachable_by_reassociating_the_canonical_order` enumerates all five for every operand and holds `40000000` to being absent from each, and the check was run against a deliberately reachable candidate and observed to fail.

**Measurement — the widening changed no measured value.** All four retained records were re-run because the kernel table is shared and its digest moved. Against their predecessors, 0 `case.*`, `comparison.*`, or `hazard.*` rows disappeared and 0 changed, across 3,215 and 4,079 pre-existing legacy rows and 828 and 960 named-profile rows; every added row names a `permutation_chain` kernel. The profile descriptor is unchanged, because the declaration already called `declare_measured_permutation` and only its comment was wrong.

**Fact — one remainder, filed rather than absorbed.** Two citations of the superseded 2026-07-30 named-profile record live outside this ticket's scopes, in `crates/tiler-metal/src/applicability.rs` and `docs/research/program-planning/first-metal-lm-workload.md`. Both still resolve and every fact they cite is unchanged, so neither is wrong today; they name the previous row. `repoint-the-superseded-apple9-record-citations` owns moving them.

**Fact — a boundary this created.** Contributor permutation is measured at `f32` only. Finding 21 established that a licence measurement at one width is not evidence about another, so `f16` and `bf16` permutation behaviour is `Unknown` rather than inherited, and the numerical-behaviour record's boundaries say so.
