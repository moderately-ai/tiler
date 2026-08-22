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
