---
id: correct-the-roadmap-s-zero-extent-prefill-shape-after-the-concatenate-change
title: Correct the roadmap s zero-extent prefill shape after the concatenate change
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---

`docs/roadmap.md` says the concatenate family "states the zero-extent rule at L5" and names a `[8, 0, 128]` prefill shape. **That became false at merge `ab64f334`**, which removed the concrete shapes from the normative definition.

**Commit repaired 2026-08-08 by the worker: the merge is `3948ca3c`, not `ab64f334`.** `git show --stat ab64f334` touches two files, both under `tickets/`, and its subject is `Close the concatenate shape removal` — it closed the removal ticket and opened this one, and reached no source. The merge that emptied `CONCATENATE_F32_NORMATIVE_DEFINITION` is `3948ca3c` (`Remove the workload shapes from the concatenate normative definition`) over worker commit `abe42412`, and `git show 580e8c1f:crates/tiler-ir/src/semantic/concatenate.rs` against `git show 3948ca3c:crates/tiler-ir/src/semantic/concatenate.rs` shows the bracketed run dropping from two sites to the one doc-comment site. The false hash is left standing above rather than substituted, so a reader who acted on it can find the correction.

## Facts

**Coordinator-verified.** The concatenate normative definition no longer names concrete shapes; the rule is now stated over the operands and the illustration moved to a doc comment, which is **not encoded**. The change moved exactly one pin — the explain request qualifier, `940c09e0821665a6` → `4e10437fec85d7b1` — and no identity domain stepped.

**Reported by the worker that landed it, not coordinator-verified:** this roadmap site is the one document left naming the retired shape.

**Fact — why the shape was removed at all.** It was pinned-workload text (KV heads by head dimension) reaching canonical identity through the registered definition's bytes. Concatenate was reportedly the only family whose normative definition named concrete shapes; the new guard `no_registered_normative_definition_names_a_concrete_shape` walks **every** registered operation and value-type definition, so a future family inherits it.

## What closes this

The row stating the zero-extent rule without the retired shape. **Do not simply delete the shape** — check what the sentence was using it for. If it illustrates the rule, the illustration now lives in the doc comment and can be referenced; if it was standing in for the rule itself, the row needs the rule stated.

**Treatment:** true when written → dated beside. Verify with `git show <commit>:<file>`. Repository **practice**, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**; say inline that a later hit lands inside your note. Note the fifth variant of that hazard, found this session: a repair quoting retired wording makes the retired anchor resolve to **the repair**, so a later reader searching for the origin lands on the correction — disclose any occurrence count that moves.

**Preserve `git log -S` anchors.** This file is heavily anchored and a sibling's rung-cell anchors already occur twice by construction. Two workers this session achieved append-only edits — one at **14 insertions, 0 deletions**, another at 2 — and both ran a **ten-word overlap scan** of their inserted lines against the pre-edit file; one found **eleven** accidental near-quotations and rewrote until zero at ten, eight, and seven words. Meet that standard.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously.

**Check the neighbouring rows and name the count.** Sweeps of this file this session found a rung cell undercounting execution rows and a reduction row whose figures the tree-width rule change did not in fact move — so both directions of error are live here.

## Worker per-Fact audit, 2026-08-08, at base `d9492f840de9820144721ffc563a463ada6d448b`

Each source below was read in full at this base before any edit. Anchors were run with `grep -F` against the file each citation names.

| # | Fact as written | Verdict |
| --- | --- | --- |
| 1 | The change landed at merge `ab64f334` | **False.** That commit is `tickets/`-only; the source merge is `3948ca3c` over `abe42412`. Repaired in the body above |
| 2 | The normative definition no longer names concrete shapes and the rule is stated over the operands | **Verified.** `CONCATENATE_F32_NORMATIVE_DEFINITION` in `crates/tiler-ir/src/semantic/concatenate.rs` was read end to end; no bracketed extent survives in it, and the empty case is decided from the operands — anchor `with one other therefore yields that other operand's extent` resolves once |
| 3 | The illustration moved to a doc comment, which is not encoded | **Verified.** `concatenate_result_shape`'s doc comment holds it; anchor `prefill binds an empty cache` resolves once. Encoding runs through `fn encode_registered_operation` into `fn encode_operation_definition`, which is what `compute_identity` folds under `tiler.semantic-registry.v7`; a doc comment is on no path into it |
| 4 | Exactly one pin moved, the explain request qualifier `940c09e0821665a6` → `4e10437fec85d7b1` | **Verified as to the landed value.** `crates/tiler-compiler/src/explain.rs` reads `"tiler-explain-v7 request=4e10437fec85d7b1\n",` at this base. The *exactly one* half is inherited from the sibling ticket's measured suite run and is not re-measured here — this branch runs no Rust gate, so it is carried as that ticket's measurement rather than as mine |
| 5 | This roadmap site is the one document left naming the retired shape | **False as worded, true on the claim that matters.** Three live documents name the bracketed run: `docs/roadmap.md`, `docs/research/runtime/autoregressive-state-and-kv-cache.md`, and `docs/research/program-planning/model-level-qualification.md`. The other two are unharmed and must not be swept — the first names it as an expressible extent the shape layer does not refuse, the second as an operand whose behaviour agrees with what the definition states, and neither asserts that the definition *names* it. What is unique to the roadmap is the assertion that the registered text states the rule **at** that shape, and only that assertion was false |
| 6 | The guard `no_registered_normative_definition_names_a_concrete_shape` walks every registered operation and value-type definition | **Verified by reading the test and its matcher.** It iterates `registry.operation_definitions()` and `registry.value_type_definitions()` and applies `shape_spelled_run`, which admits a bracketed run whose comma-separated tokens are each a digit run or one uppercase letter, with at least one numeric. Its own doc comment states the blind spot: `(8, 0, 128)` and `8x128` pass |
| 7 | Concatenate was the only family whose normative definition named concrete shapes | **Not re-verified; recorded as inherited.** The sibling ticket's audit already downgraded this from an exhaustive scan to a conclusion holding on a population the scan did not cover. Nothing here depends on it, because the guard now decides the property for the whole registry rather than a scan deciding it once |

### The neighbouring census, counted rather than assumed

The family-state table is **23 rows**, counted at lines 479–501 and agreeing with the section's own `twenty-three is the number of families` sentence. That sentence's neighbour is a live example of the emphasis-marker failure: `of the taxonomy's forty-seven families have no row here` resolves once, while the same phrase extended leftward to include the bolded numeral returns **0**, because the source writes `**Twenty-five**`. Of the 23 rows, **exactly one** — this row — makes any claim about a normative definition or the zero-extent rule; `grep -n -E 'zero-extent|normative definition' docs/roadmap.md` returned one line before the edit, and `concrete shape`, `prefill shape`, and `NORMATIVE_DEFINITION` are each confined to it. The structural row above and the sub-tensor row below assert nothing this landing touches, so the neighbours are clean in the direction the sweep was worried about.

**Two rows carry the explain qualifier reading this landing superseded**, and neither is repaired, which is a decision rather than an omission. This row and the sub-tensor row below both read `940c09e0821665a6` **at base `cc667626`** — base-qualified, therefore true as stated — and both already tell a reader that the ledger comment beside the literal is the authority and that a sentence in this file never is. Writing `4e10437fec85d7b1` into either would recommit the precise anti-pattern they name and go stale on the next landing that folds the registry snapshot. The correct reading remains `grep -n 'tiler-explain-v7 request=' crates/tiler-compiler/src/explain.rs`.

## Worker outcome, 2026-08-08 — complete

**What the shape was doing there, read before deciding.** The clause sat at the end of an admission list describing what the registered definition does, and it named the shape as the *locus* of the statement — the rule was stated at that instance. So it was neither pure illustration nor a stand-in for the rule: half of it survives the landing (the definition does state the rule) and half of it does not (it no longer states it there, or at any instance). The repair keeps the surviving half, restates the rule over the operands as the registered text now does, and sends the reader to the doc comment for the instance.

**Dated beside, not substituted, and additive at the byte level.** The superseded clause stands where it was; the correction follows the evidence sentence in the same Fact. Measured on the edited tree: `docs/roadmap.md` changed on exactly one line, and removing a single contiguous 2,570-character block from that line reproduces the pre-edit line byte for byte — zero deletions, so no existing string's occurrence count fell and every `git log -S` anchor is preserved. Line-level append-only was unreachable here for a structural reason worth recording: a markdown table row is one line, so any correction inside a cell is a one-line replacement however additive its bytes are.

**Greppability, disclosed rather than claimed clean.** The correction reproduces none of the superseded phrasing, so `grep -F 'states the zero-extent rule at L5' docs/roadmap.md` still returns **1** and resolves to the origin, not to the repair — the fifth variant is avoided rather than mitigated. One count did move and was caught by measuring instead of asserting: a first draft closed by saying the bracketed run "stays at two occurrences" and spelled it out, which took the file from two to three. That sentence was rewritten to describe the runs without spelling them, and the file is back at **2**.

**Ten-word overlap scan, run against the pre-edit file.** Zero overlapping windows at ten and at eight words over 410 inserted windows. At seven words there is exactly **one**, `crates/tiler-ir/src/semantic/registry.rs` — a citation path, seven tokens long, which has to be identical to resolve and cannot be paraphrased away. An earlier draft also collided at eight on `in crates/tiler-ir/src/semantic/registry.rs`, matching the pointwise row's `StandardSemantics in …` phrasing; that one was real prose overlap and was rewritten out. The scanner was proved able to fail first, by feeding it a sentence lifted verbatim from the file: 11 windows at ten, 13 at eight, 14 at seven.

**Anchors, each run before commit.** `with one other therefore yields that other operand's extent` (1) and `prefill binds an empty cache` (1) in `crates/tiler-ir/src/semantic/concatenate.rs`; `no_registered_normative_definition_names_a_concrete_shape` (1), `fn encode_registered_operation` (1), `fn encode_operation_definition` (1), and `tiler.semantic-registry.v7` (1) in `crates/tiler-ir/src/semantic/registry.rs`. **One anchor the sibling ticket published does not resolve, and the failure mode is the documented one:** `push_slice(output, definition.normative_definition().as_str().as_bytes())` returns 0, because `rustfmt` wraps that call across four lines in `encode_operation_definition`. The single-line spelling is real only in `encode_type_definition`. This note therefore cites the two functions by name rather than by call text.
