---
id: recheck-target-dtype-dispatch-after-semantic-rewrites
title: Recheck target dtype dispatch after semantic rewrites
status: deferred
priority: p2
dependencies: [admit-a-caller-declared-target-profile]
related: [spike-bf16-through-the-second-dtype-seams]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Activation trigger

Activate when an admitted semantic rewrite can introduce, remove, or replace a resolved value type. Admitted rewrites today preserve each value's existing exact resolved type, so initial request admission and candidate readmission observe the same unique dtype set (not an f32-only universe: BF16 is admitted, and CSE clones each input's `resolved_type` rather than rewriting everything to `tiler::f32@1`).

## Required outcome

Recompute the canonical unique exact resolved value types for every rewritten candidate and reassess them against each target at `CompileProfile`. An unsupported, unknown, or deferred rewrite-introduced type rejects that candidate for that target without erasing another candidate or another target's outcome. Candidate readmission must retain typed target-local dtype detail, and a mutation fixture must replace a rewrite result type and observe the check fail.

## Trigger check log

- 2026-08-04 — **not fired.** No admitted semantic rewrite introduces, removes, or replaces a resolved value type: the registered builtin rules are region formation, region candidacy, shared-value normalization, and stage normalization (`RuleRef::builtin(REGION_FORMATION_RULE)` / `RuleRef::builtin(REGION_CANDIDATE_RULE)` in `crates/tiler-compiler/src/region.rs`; `RuleRef::builtin(NORMALIZE_SHARED_VALUE_RULE)` / `RuleRef::builtin(NORMALIZE_STAGE_RULE)` in `crates/tiler-compiler/src/normalize.rs`, still at the 179/247/290 sites), all of which restructure a program without retyping a value. [`spike-bf16-through-the-second-dtype-seams`](spike-bf16-through-the-second-dtype-seams.md) is `done` but is a spike; [`admit-the-bf16-type-and-carrier-into-every-total-map`](admit-the-bf16-type-and-carrier-into-every-total-map.md) is `todo`. Recheck: `grep -rn 'RuleRef::builtin' crates/tiler-compiler/src --include='*.rs' | grep -v tests`.
- 2026-08-09 — **not fired; the old BF16 status sentence is retired.** `admit-the-bf16-type-and-carrier-into-every-total-map` is now `done`, but that widens admitted types rather than making a rewrite change one. The live normalization paths clone or compare each value's existing `resolved_type`, and the four production rewrite identities remain the canonical baseline, common-subexpression normalization, and the two ordered F32 reassociations. No rule introduces, removes, or replaces a resolved type, so candidate and initial admission still see the same exact type set.
- 2026-08-10 — **not fired.** Phase B audit repair: activation body no longer asserts an f32-only ambient universe (type-set preservation under rewrites is the load-bearing claim; BF16 remains an admitted dtype that CSE preserves). The 2026-08-04 region line citations `region.rs:433,497,539` are retired in favor of `RuleRef::builtin(REGION_FORMATION_RULE)` / `RuleRef::builtin(REGION_CANDIDATE_RULE)` anchors (production sites currently near 626/690/732). Normalize anchors remain accurate. Trigger still unfired: no admitted rule introduces, removes, or replaces a resolved value type.
- **Recheck repaired — 2026-08-22; no verdict re-decided here.** The 2026-08-04 entry's `grep -rn 'RuleRef::builtin' crates/tiler-compiler/src --include='*.rs' | grep -v tests` is unusable for this trigger, for two independent reasons, and the second is the worse one.

  **It filters lines, not modules.** The `#[cfg(test)]` modules here are inline, so `grep -v tests` removes exactly one line of 57 — the one path that literally contains `tests.rs`. `RuleRef::builtin("test.root")` at `crates/tiler-compiler/src/region.rs` and `crates/tiler-compiler/src/normalize.rs` **survives the filter**, because the line says `test` and not `tests`; so do roughly twenty `test.rule`, `budget.test`, and other in-module test spellings. `… | grep -c 'test.root'` returns `2`. This defect was already written down inside [`probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary`](probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary.md)'s own 2026-08-06 log entry, and was never propagated here.

  **And it cannot see the subject at all.** This ticket's trigger is an admitted *semantic rewrite* that retypes a value. A rewrite is registered as a `RewriteRuleIdentity`, not as a `RuleRef::builtin`, so the retired command does not observe the population it claims to bound. The replacement is the one the sibling ticket already uses:

  ```sh
  grep -rn 'RewriteRuleIdentity::new("tiler' crates/ --include='*.rs'
  ```

  It returns **exactly four production identities** at this base — `tiler.pipeline/canonical-semantic-baseline.v1`, `tiler.normalize/common-subexpression.v1`, and the two `tiler.algebraic/ordered-reassociate-{add,multiply}-f32.v1` — and needs no `grep -v`, because the `"tiler` prefix is itself the production anchor and the test identities are spelled `("p", "r", 1)`. **Watched producing the firing answer:** on a scratch copy of this tree a fifth production identity `RewriteRuleIdentity::new("tiler.algebraic", "widen-bf16-to-f32.v1", 1)` — a rewrite that retypes a value, which is precisely what fires this ticket — was added to `crates/tiler-compiler/src/rewrite.rs`. The replacement reported five lines, naming `widen-bf16-to-f32.v1` among them. **The retired command reported `0` hits for that same new rule**, confirming it was blind rather than noisy.
