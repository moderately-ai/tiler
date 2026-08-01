---
id: correct-the-navigation-docs-for-the-loop-carried-body
title: Correct the navigation docs falsified by the loop-carried body landing
status: todo
priority: p2
dependencies: [lower-a-loop-carried-cooperative-body]
related: [lower-a-loop-carried-cooperative-body, realize-the-strict-contraction-on-metal]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning, contraction]
---
## User-visible outcome

`docs/status.md` and `docs/roadmap.md` stop saying that a multi-round cooperative tile has no body. Until they do, the two documents a reader is pointed at first say the `tiled` contraction is blocked behind work that has landed, which is the failure mode that costs a coordinator a dispatch rather than a gate.

## Why it is a separate node rather than part of the landing

**Fact — `contracts/navigation` was held by a live claim at the time.** `lower-a-loop-carried-cooperative-body` corrected `docs/ir.md` in its own commit, under the `contracts/foundation` scope it added, because that scope was unheld. `docs/status.md` and `docs/roadmap.md` map to `contracts/navigation` (`ticketsplease.toml`), which `land-the-two-level-reduction-adr` held live. AGENTS admits an edit inside another live ticket's scope only when file-level disjointness is verified against *that worker's actual branch diff* — and no `tkt/land-the-two-level-reduction-adr` branch existed to diff, locally or on the remote, so the verification the rule requires could not be performed. Filing was the only admissible move.

## The exact sentences

**`docs/status.md`, the "one typed synchronization point" bullet.** "sits where its tile's round structure makes it convergent — outside every predicate, and inside the round loop exactly when there is one" is now wrong in its second half. The rule is *at most* one enclosing loop, not exactly one: a loop-carried body peels round zero, so the phase boundary is realized once at the top level and `rounds - 1` times inside the round loop. What refuses a stray top-level barrier is the realization count, not the nesting rule.

**`docs/status.md`, the "loop-carried cooperative staging is representable and not yet lowered" bullet.** The title and the "**No body realizes one.**" sentence are both falsified. The KIR vocabulary question it forwards — whether a predicated region may yield values — turned out to be answerable *no change required*: the accumulator stays outside every predicate because staged accesses are not the boundary effects predicate dominance governs. The remaining blocker for the log-depth tree (a per-access active-participant subset and a round-varying span) is unchanged and should survive the rewrite.

**`docs/roadmap.md`, the contraction row.** "What remains is a *body*: the canonical lowering refuses a multi-round tile by name" and the closing "`tiled` stays open behind it" are falsified. `realize-the-strict-contraction-on-metal` is no longer blocked by the body; what it still owns is the tiled schedule itself, its `K ≡ 0 (mod 16)` precondition, and its Metal emission. The row's `direct`-path facts, its measurements, and its R5/R7 residuals are untouched by this and must not be swept into the rewrite.

## Closes when

All three spans say what the code does, no other assertion in either document moved, and `tkt lint` is clean.
