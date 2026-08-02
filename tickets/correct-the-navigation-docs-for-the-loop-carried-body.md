---
id: correct-the-navigation-docs-for-the-loop-carried-body
title: Correct the navigation docs falsified by the loop-carried body landing
status: in-progress
priority: p2
dependencies: [lower-a-loop-carried-cooperative-body]
related: [lower-a-loop-carried-cooperative-body, realize-the-strict-contraction-on-metal]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning, contraction]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785685340
---
## User-visible outcome

`docs/status.md` and `docs/roadmap.md` stop saying that a multi-round cooperative tile has no body. Until they do, the two documents a reader is pointed at first say the `tiled` contraction is blocked behind work that has landed, which is the failure mode that costs a coordinator a dispatch rather than a gate.

## Why it is a separate node rather than part of the landing

**Fact — `contracts/navigation` was held by a live claim at the time.** `lower-a-loop-carried-cooperative-body` corrected `docs/ir.md` in its own commit, under the `contracts/foundation` scope it added, because that scope was unheld. `docs/status.md` and `docs/roadmap.md` map to `contracts/navigation` (`ticketsplease.toml`), which `land-the-two-level-reduction-adr` held live. AGENTS admits an edit inside another live ticket's scope only when file-level disjointness is verified against *that worker's actual branch diff* — and no `tkt/land-the-two-level-reduction-adr` branch existed to diff, locally or on the remote, so the verification the rule requires could not be performed. Filing was the only admissible move.

## The exact sentences

**`docs/status.md`, the "one typed synchronization point" bullet.** "sits where its tile's round structure makes it convergent — outside every predicate, and inside the round loop exactly when there is one" is now wrong in its second half. The rule is *at most* one enclosing loop, not exactly one: a loop-carried body peels round zero, so the phase boundary is realized once at the top level and `rounds - 1` times inside the round loop. What refuses a stray top-level barrier is the realization count, not the nesting rule.

**`docs/status.md`, the "loop-carried cooperative staging is representable and not yet lowered" bullet.** The title and the "**No body realizes one.**" sentence are both falsified. The KIR vocabulary question it forwards — whether a predicated region may yield values — turned out to be answerable *no change required*: the accumulator stays outside every predicate because staged accesses are not the boundary effects predicate dominance governs. The remaining blocker for the log-depth tree (a per-access active-participant subset and a round-varying span) is unchanged and should survive the rewrite.

**`docs/roadmap.md`, the contraction row.** "What remains is a *body*: the canonical lowering refuses a multi-round tile by name" and the closing "`tiled` stays open behind it" are falsified. `realize-the-strict-contraction-on-metal` is no longer blocked by the body; what it still owns is the tiled schedule itself, its `K ≡ 0 (mod 16)` precondition, and its Metal emission. The row's `direct`-path facts, its measurements, and its R5/R7 residuals are untouched by this and must not be swept into the rewrite.

## Correction — 2026-08-02, one of the four named spans did not exist

**Fact.** The closing "`tiled` stays open behind it" this ticket attributes to the contraction row is not in the corpus: `grep -rn "stays open behind it" docs/` reports no match. That half of the roadmap claim had already been corrected before this ticket was worked — the row's residual column now reads "behind [`admit-a-two-dimensional-cooperative-staging-relation`] and [`admit-a-cooperative-tile-over-shared-operands`] rather than behind the loop-carried body, which has landed" — so there was nothing to rewrite. The other three spans were verified present and falsified, and were corrected.

**Fact — the roadmap's surviving span was tensed rather than deleted.** The row states its facts as a dated chain, and the sentence after the falsified one already carries "the body landed on 2026-08-01". Deleting the superseded assertion would have removed the rationale the later fact corrects, so "What remains is a *body*: the canonical lowering refuses" became "What remained then was a *body*: the canonical lowering refused", which is what the row's own chronological form asks for.

## Closes when

All three spans that exist say what the code does, no other assertion in either document moved, and `tkt lint` is clean.
