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

Activate when an admitted semantic rewrite can introduce, remove, or replace a resolved value type. The current algebraic rewrite vocabulary preserves every value as exact `tiler::f32@1`, so initial request admission and candidate readmission observe the same unique dtype set today.

## Required outcome

Recompute the canonical unique exact resolved value types for every rewritten candidate and reassess them against each target at `CompileProfile`. An unsupported, unknown, or deferred rewrite-introduced type rejects that candidate for that target without erasing another candidate or another target's outcome. Candidate readmission must retain typed target-local dtype detail, and a mutation fixture must replace a rewrite result type and observe the check fail.

## Trigger check log

- 2026-08-04 — **not fired.** No admitted semantic rewrite introduces, removes, or replaces a resolved value type: the registered builtin rules are region formation, region candidacy, shared-value normalization, and stage normalization (`crates/tiler-compiler/src/region.rs:433,497,539`, `crates/tiler-compiler/src/normalize.rs:179,247,290`), all of which restructure a program without retyping a value. [`spike-bf16-through-the-second-dtype-seams`](spike-bf16-through-the-second-dtype-seams.md) is `done` but is a spike; [`admit-the-bf16-type-and-carrier-into-every-total-map`](admit-the-bf16-type-and-carrier-into-every-total-map.md) is `todo`. Recheck: `grep -rn 'RuleRef::builtin' crates/tiler-compiler/src --include='*.rs' | grep -v tests`.
- 2026-08-09 — **not fired; the old BF16 status sentence is retired.** `admit-the-bf16-type-and-carrier-into-every-total-map` is now `done`, but that widens admitted types rather than making a rewrite change one. The live normalization paths clone or compare each value's existing `resolved_type`, and the four production rewrite identities remain the canonical baseline, common-subexpression normalization, and the two ordered F32 reassociations. No rule introduces, removes, or replaces a resolved type, so candidate and initial admission still see the same exact type set.
