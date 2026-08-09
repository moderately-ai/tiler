---
id: audit-dead-code-admissions-after-public-boundary-promotions
title: Audit dead-code admissions after public-boundary promotions
status: todo
priority: p3
dependencies: [promote-the-symbolic-index-profile-to-a-public-boundary, promote-the-metal-aot-compilation-identity, expose-the-governed-fact-field-vocabulary, expose-the-numerical-contract-preference-list, wire-the-delivered-realization-record-into-the-artifact]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [maintenance, lints, public-boundary, deferred]
---
Re-audit production `dead_code` admissions after the current public-boundary
promotion wave makes the intended producers and consumers reachable.

## The inventory this audit starts from (2026-07-28)

**Twelve production files carried a file-scope admission on 2026-07-28**, so the claim then current — that "the whole-file admissions observed by the production audit are no longer in the current tree" — was false, and this table corrected it. One-line check as written that day: `grep -rn -B2 dead_code crates/*/src --include='*.rs' | grep '#!\[allow('` printed twelve lines; adding `prototypes/*/src` printed thirteen, the extra being a file-scope admission in the serial-sum-compile prototype, which is not production.

**Correction, 2026-08-07, found by `check-citations.sh` and verified by reading.** Two Facts in the paragraph above had rotted. (1) The prototype citation named `prototypes/serial-sum-compile/src/target.rs` at line 29; that file was added at `8dbffb93` and **deleted at `2d2a7bd7`**, so the citation resolved to nothing and a reader following it would have found no such path. (The retired line number is written in prose rather than pinned to the path, which is the convention that keeps a dated correction quotable without asserting a citation the tree cannot satisfy.) The line number is dropped here rather than re-pointed, because there is nothing left to point at: a multi-line-aware scan of `prototypes/*/src` for an inner `#![allow(…)]` mentioning `dead_code` now returns **zero** files, so the "thirteenth" is gone entirely and the prototype clause is now history, not a live exclusion. (2) The count is **seven**, not twelve and not the eight recorded on 2026-08-04 below: `crates/tiler-artifact/src/program/realization.rs` lost its file-scope admission at `8bfcd432` and now carries only an item-level one, leaving `codec/mod.rs`, `policy.rs`, `boundary.rs`, `explain.rs`, and `target/{accuracy,honourability,feasibility}.rs`. The 2026-07-28 table below is left standing as the dated inventory it declares itself to be; it is the starting point this audit compares against, not a claim about today.

**The reproduce command in this ticket is itself line-oriented, which this audit should not inherit.** `grep -B2 dead_code … | grep '#!\[allow('` only sees a `dead_code` that falls within two lines of its opening `#![allow(`. Every admission in the tree today happens to sit inside that window, so the count is right by luck rather than by construction — a widened attribute would silently drop out of the population. A scan that reads each `#![allow(…)]` as one whole construct returns the same seven files and does not depend on that spacing.

| File | Allow at | Reopening owner named by the reason |
| --- | --- | --- |
| `crates/tiler-artifact/src/program/realization.rs` | 1 | `redesign-the-delivered-realization-record-from-typed-evidence`, `accept-the-delivered-realization-artifact-surface`, then `wire-the-delivered-realization-record-into-the-artifact`; the redesign may replace the file and its allow |
| `crates/tiler-artifact/src/program/codec/mod.rs` | 1 | Tom's ADR 0075 call on promoting the codec surface (also covers `unused_imports`) |
| `crates/tiler-cache/src/expansion/collect.rs` | 101 | `accept-the-tiler-cache-public-boundary`, plus a proc-macro frontend or maintenance command as the calling slice |
| `crates/tiler-cache/src/expansion/preflight.rs` | 1 | `accept-the-tiler-cache-public-boundary` |
| `crates/tiler-compiler/src/boundary.rs` | 1 | a top-down property search or a second execution profile |
| `crates/tiler-compiler/src/explain.rs` | 1 | a trace consumer, for the presentation renderer and the reserved vocabulary |
| `crates/tiler-compiler/src/target/feasibility.rs` | 1 | a later-phase assessment: artifact-evidence, device-runtime, prepared-kernel, launch |
| `crates/tiler-compiler/src/target/honourability.rs` | 1 | `declare-metal-numerical-honourability`, first to reach a non-exact honouring means |
| `crates/tiler-ir/src/index/sourced.rs` | 1 | `promote-the-symbolic-index-profile-to-a-public-boundary` |
| `crates/tiler-ir/src/shape/env.rs` | 1 | `promote-the-symbolic-index-profile-to-a-public-boundary` |
| `crates/tiler-metal-aot/src/family.rs` | 44 | `prototype-inline-proc-macro-frontend`, via `record-that-the-frontend-axis-is-review-gated` |
| `crates/tiler-metal-aot/src/identity.rs` | 60 | the caller holding both `tiler-metal-aot` and `tiler-cache`, reached from the frontend layer |

Three of the twelve sit below a long module doc rather than at line 1; the attribute is still an inner `#![allow(…)]` over the whole file in every case, so "file-scope" is about reach, not about line number.

**A reason can go stale while the admission stays correct, and that is the harder half of this audit.** `crates/tiler-compiler/src/boundary.rs:1` listed alias materialization among the property values nothing constructs. `call_declaration::guaranteed_properties_for` produces `MaterializationForm::AliasView` for an opaque call declaring `MayAliasInputs` (`crates/tiler-compiler/src/call_declaration.rs:373`), so the list was wrong while the allow itself was still needed for the other reserved values. Commit `588be6e` corrected that reason; the audit's job is to catch the next one, and the check is per *clause* of a reason, not per allow.

## Outcome

Remove an admission when its item is now used or public. Keep a private
reservation only at the narrowest item or submodule whose missing producer or
consumer is real, with a reason naming that boundary and the trigger that will
reopen it. Do not add artificial call sites merely to satisfy the lint.

**Worked example, already discharged — three item admissions whose stated trigger had fired.** At `befbba7`, `crates/tiler-compiler/src/frontier.rs` carried three item-level allows each reading "lands with its tests ahead of the admission that calls it" or equivalent: on `derive_call_boundary_contract`, on `encode_call_subject`, and on `resolve_work_items`. The admission that calls them had landed — all three are called from `enumerate_frontier` (`frontier.rs:1346`), at `:1410`, `:1436`, and `:1456` — so each reason was refuted by the file it sat in. Commit `0792510` removed all three (`git show 0792510 -- crates/tiler-compiler/src/frontier.rs | grep -c '^-    dead_code,'` prints `3`), and `grep -n dead_code crates/tiler-compiler/src/frontier.rs` now prints ten lines, none of them those. This is the exact shape the audit is looking for and the exact remedy: delete the allow, do not rewrite the reason.

## Trigger

**Dependency status, checked 2026-07-29:** `expose-the-governed-fact-field-vocabulary` **done**; `expose-the-numerical-contract-preference-list` **done**; `promote-the-metal-aot-compilation-identity` has landed its prepared-compilation boundary; `promote-the-symbolic-index-profile-to-a-public-boundary` remains split across owner-reviewed public shapes; `wire-the-delivered-realization-record-into-the-artifact` is `todo` behind engineering through the shared Metal honourability form, structured provenance, and a compile-checked redesign before Tom reviews the exact replacement boundary. The artifact path is not currently waiting on Tom alone.

**Do not run this audit per promotion.** It is a sweep, and its value comes from every promotion in the wave having landed, so that an item still unreached is genuinely unreached rather than merely waiting for its own ticket. **Run it when the last of the five lands**, or earlier if a *subsystem* completes — all of `tiler-ir`'s promotions, or all of `tiler-artifact`'s — in which case audit only that crate's admissions and say in the record that the sweep was partial.

**Who acts.** Whoever lands the last promotion of the wave owns opening this ticket; whoever lands the last promotion within one crate owns the partial sweep for that crate. Neither is a coordinator decision — it is the closing step of the promotion that made the items reachable.

## Closes when

Every production `dead_code` admission is either gone or justified against a
current construction/consumer search, no whole-file admission has returned
without a reason naming a real unavailable producer or consumer, every retained
reason has had each of its clauses checked rather than the allow as a whole,
and the full gate passes.

## Trigger check log

- 2026-08-04 — **FIRED, partially; reactivated to `todo`.** The ticket's own trigger admits a partial sweep "if a *subsystem* completes — all of `tiler-ir`'s promotions, or all of `tiler-artifact`'s". [`promote-the-symbolic-index-profile-to-a-public-boundary`](promote-the-symbolic-index-profile-to-a-public-boundary.md) is `done`, which is the whole of `tiler-ir`'s promotion set, and [`promote-the-metal-aot-compilation-identity`](promote-the-metal-aot-compilation-identity.md) is `done`, which is `tiler-metal-aot`'s. Both partial sweeps were owed by the promotion that closed them and neither was run. **The inventory has also moved and the 2026-07-28 table is stale**: the file-scope admissions are now eight rather than twelve, `crates/tiler-ir/src/index/sourced.rs`, `crates/tiler-ir/src/shape/env.rs`, `crates/tiler-cache/src/expansion/{collect,preflight}.rs`, and `crates/tiler-metal-aot/src/{family,identity}.rs` are gone, and `crates/tiler-compiler/src/policy.rs` and `crates/tiler-compiler/src/target/accuracy.rs` are new. Reproduce: `grep -rn -B2 dead_code crates/*/src --include='*.rs' | grep '#!\[allow('` prints eight lines. The *item*-level admissions in the two completed subsystems are what the partial sweep now owes; the frontmatter dependencies still gate the full five-promotion sweep on [`wire-the-delivered-realization-record-into-the-artifact`](wire-the-delivered-realization-record-into-the-artifact.md), which is `todo`, so this sits at `todo` waiting on that edge rather than at `deferred` waiting on nothing.
- 2026-08-09 — **FIRED in full.** All five frontmatter dependencies are now `done`, including `wire-the-delivered-realization-record-into-the-artifact`; the dated 2026-08-04 statement that the full sweep still waits on that edge is historical. A multiline-aware current scan finds seven file-scope admissions: artifact `program/codec/mod.rs`; compiler `policy.rs`, `boundary.rs`, `explain.rs`, and `target/{accuracy,honourability,feasibility}.rs`. The complete sweep is current `todo` work and must re-read each retained reason clause rather than inheriting either historical table.
