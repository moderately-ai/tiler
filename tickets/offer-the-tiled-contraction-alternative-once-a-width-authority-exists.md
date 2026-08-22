---
id: offer-the-tiled-contraction-alternative-once-a-width-authority-exists
title: Offer the tiled contraction alternative once a width authority exists
status: todo
priority: p2
dependencies: [carry-the-contraction-tile-width-policy-as-a-target-profile-row, offer-the-tiled-contraction-alternative-in-physical-planning]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, scheduling, contraction]
---
## User-visible outcome

A caller whose request reaches a strict `f32` tensor contraction is actually offered the tiled cooperative alternative, constructed from the declared width authority rather than a literal.

## Why this exists

Split out 2026-08-22 by the coordinator. The parent ticket [`offer-the-tiled-contraction-alternative-in-physical-planning`](offer-the-tiled-contraction-alternative-in-physical-planning.md) bundled three width-**independent** repairs with the width-**dependent** offer, and the tile-width packet showed the dependency ran backwards: *no tile width can be compared against another until the cost model can score the topology at all.* The parent now owns the three repairs and is dispatchable today; this ticket owns the offer and waits on the authority.

**Fact — nothing in `tiler-compiler` names the tiled surface at all.** The tile-width lane ran `grep -rn --include='*.rs' -E 'CooperativeContraction|BlockedWorkgroup|blocked_operand_tile|admit_exact_cooperative|admit_predicated_cooperative' crates/tiler-compiler` and reports no match. Re-derive at your base and say which unit you report.

**Fact — FALSE at `6f3c2594`, and already false when this ticket was written.** Retired wording, preserved: *"nothing in `tiler-compiler` names the tiled surface at all"*, and of the recorded command, *"reports no match"*. It did report no match at the tile-width lane's own base, but the cooperative-contraction cost arm had merged at `7a3caca7` — in the same batch and ahead of this ticket — so the Fact was false at the commit this ticket landed on. Re-run at `6f3c2594` the recorded command returns **23 matched lines** (unit: lines, not occurrences) across two files, `crates/tiler-compiler/src/physical.rs` and `crates/tiler-compiler/src/measured_cost.rs`; **10** sit above their file's `#[cfg(test)] mod tests` and **13** below it. Corrected 2026-08-22 by [`re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree`](re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree.md).

**Fact — restated over what is actually in the tree, because what this ticket needed the Fact for survives.** No production path in `tiler-compiler` **constructs or offers** a tiled region, which is the claim this ticket rests on. All ten live hits consume or verify an already-built topology: one `use` import, three doc-comment mentions, the `matches!` arm and the `let … else` that open the request-subject binding's verifier, its `ExecutionBinding::BlockedWorkgroup` destructuring, the two `admit_*` calls at `crates/tiler-compiler/src/physical.rs "TailPolicy::Predicated => admit_predicated_cooperative_contraction("` that re-derive a supplied region's tiling facts rather than choosing them, and the cost model's scoring arm. Every construction site is under `mod tests`. The source says so directly at `crates/tiler-compiler/src/physical.rs "constructs a region with this identifier"`, on the doc comment of the region identifier a tiled contraction must carry — a claim that does not rot as the variant gets named more often. **What the falsification actually records is progress on this ticket's own dependency:** the cost model can now score the topology, which this ticket's rationale says had to land before any width could be compared. Re-derive at your base before acting, and say which unit you report.

**Fact — the constructor already takes the width as a parameter.** `blocked_operand_tile(block, rounds)` is a labelled draft public boundary in `tiler-ir`. No production path in `crates/` hard-codes 16. Reported by the tile-width lane; re-verify.

## Required work

- Re-audit both Facts at your base with a per-Fact verdict.
- Offer the alternative using the width the accepted authority supplies. **Never a literal, and never a default on a target whose profile is silent** — the workgroup-tree precedent refuses exactly that, and the same refusal path (`TargetPolicyUndeclared`) already exists.
- One negative control: a target whose profile declares no policy must decline by name, not fall back.
- One negative control: the direct fold is still offered and still chosen where it should be.
- Perturb each behaviour separately with quoted failure text.

## Non-goals

The `work_span` arm, the wildcard removal, and the output-binding widening — all three belong to the parent ticket and need no width. Choosing the width. Dispatching on a device.

## Closes when

A strict contraction request on a policy-declaring target is offered both alternatives, a silent target declines by name rather than defaulting, both negative controls hold, and the workspace gate is green.
