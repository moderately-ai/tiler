---
id: recheck-target-dtype-dispatch-after-semantic-rewrites
title: Recheck target dtype dispatch after semantic rewrites
status: todo
priority: p2
dependencies: [admit-a-caller-declared-target-profile]
related: [spike-bf16-through-the-second-dtype-seams]
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: []
---
## Activation trigger

Activate when an admitted semantic rewrite can introduce, remove, or replace a resolved value type. The current algebraic rewrite vocabulary preserves every value as exact `tiler::f32@1`, so initial request admission and candidate readmission observe the same unique dtype set today.

## Required outcome

Recompute the canonical unique exact resolved value types for every rewritten candidate and reassess them against each target at `CompileProfile`. An unsupported, unknown, or deferred rewrite-introduced type rejects that candidate for that target without erasing another candidate or another target's outcome. Candidate readmission must retain typed target-local dtype detail, and a mutation fixture must replace a rewrite result type and observe the check fail.
