---
id: audit-dead-code-admissions-after-public-boundary-promotions
title: Audit dead-code admissions after public-boundary promotions
status: deferred
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

**Twelve production files carry a file-scope admission**, so the earlier claim that "the whole-file admissions observed by the production audit are no longer in the current tree" is false and is corrected here. One-line check: `grep -rn -B2 dead_code crates/*/src --include='*.rs' | grep '#!\[allow('` prints twelve lines; add `prototypes/*/src` and it prints thirteen, the extra being `prototypes/serial-sum-compile/src/target.rs:29`, which is not production.

| File | Allow at | Reopening owner named by the reason |
| --- | --- | --- |
| `crates/tiler-artifact/src/program/realization.rs` | 1 | `accept-the-delivered-realization-artifact-surface`, then `wire-the-delivered-realization-record-into-the-artifact` |
| `crates/tiler-artifact/src/program/codec/mod.rs` | 1 | Tom's ADR 0075 call on promoting the codec surface (also covers `unused_imports`) |
| `crates/tiler-cache/src/expansion/collect.rs` | 101 | `accept-the-tiler-cache-public-boundary`, plus a proc-macro frontend or maintenance command as the calling slice |
| `crates/tiler-cache/src/expansion/preflight.rs` | 1 | `accept-the-tiler-cache-public-boundary` |
| `crates/tiler-compiler/src/boundary.rs` | 1 | a top-down property search or a second execution profile |
| `crates/tiler-compiler/src/explain.rs` | 1 | a trace consumer, for the presentation renderer and the reserved vocabulary |
| `crates/tiler-compiler/src/feasibility.rs` | 1 | a later-phase assessment: artifact-evidence, device-runtime, prepared-kernel, launch |
| `crates/tiler-compiler/src/honourability.rs` | 1 | `declare-metal-numerical-honourability`, first to reach a non-exact honouring means |
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

**Dependency status, checked 2026-07-28:** `expose-the-governed-fact-field-vocabulary` **done**; `expose-the-numerical-contract-preference-list` **done**; `promote-the-metal-aot-compilation-identity` **in-progress**; `promote-the-symbolic-index-profile-to-a-public-boundary` **awaiting-decision** (blocked on four owner questions recorded on that ticket); `wire-the-delivered-realization-record-into-the-artifact` **todo** (blocked on `accept-the-delivered-realization-artifact-surface`). So the wave is two-fifths landed and the two furthest out are both waiting on Tom rather than on engineering.

**Do not run this audit per promotion.** It is a sweep, and its value comes from every promotion in the wave having landed, so that an item still unreached is genuinely unreached rather than merely waiting for its own ticket. **Run it when the last of the five lands**, or earlier if a *subsystem* completes — all of `tiler-ir`'s promotions, or all of `tiler-artifact`'s — in which case audit only that crate's admissions and say in the record that the sweep was partial.

**Who acts.** Whoever lands the last promotion of the wave owns opening this ticket; whoever lands the last promotion within one crate owns the partial sweep for that crate. Neither is a coordinator decision — it is the closing step of the promotion that made the items reachable.

## Closes when

Every production `dead_code` admission is either gone or justified against a
current construction/consumer search, no whole-file admission has returned
without a reason naming a real unavailable producer or consumer, every retained
reason has had each of its clauses checked rather than the allow as a whole,
and the full gate passes.
