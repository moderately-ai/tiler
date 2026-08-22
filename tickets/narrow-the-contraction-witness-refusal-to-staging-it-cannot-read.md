---
id: narrow-the-contraction-witness-refusal-to-staging-it-cannot-read
title: Narrow the contraction witness refusal to staging it cannot read
status: todo
priority: p2
dependencies: []
related: [join-the-scheduled-region-into-the-contraction-witness, derive-staged-combine-structure-from-program-scope]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [ir, witness, fail-closed, correctness]
---
## User-visible outcome

The contraction plan witness refuses the kernels whose combine tree it genuinely cannot establish, rather than every kernel that declares any workgroup staging at all.

## Why this exists

Found 2026-08-22 by the staged-combine derivability spike while auditing a Fact the coordinator wrote, and recorded separately because it is a different defect from the join that spike motivated.

**Fact — the refusal is broader than its own documentation says.** `crates/tiler-ir/src/program/contraction_witness.rs` refuses at the predicate `staging().len() != 0` — two sites, one for the covering realization and one for a split's combiner. The **enum-level** doc of `ContractionF32PlanWitnessError` says the catch-all covers a realization *"whose exact binary combine tree cannot be derived from program scope — including any kernel that declares workgroup staging"*. **The code does not implement that.** It refuses on the presence of staging alone, including staging that carries no combine structure at all, so the documented "including" is in fact the whole predicate.

**Correction — 2026-08-22, the coordinator's original wording of this Fact was wrong and is repaired above.** I wrote that the prose "sits in the doc comment of the `TopologyUnsupported` variant, several lines above". Verified at `e7a2d0d4`: **three distinct pieces of prose were conflated.** The `// A kernel declaring workgroup staging combines inside the workgroup;` line is a plain `//` comment **two lines directly above** the refusal it introduces, not a doc comment and not distant from it. The `TopologyUnsupported` variant's own doc reads only *"The covering realization's exact binary combine tree cannot be derived from program scope"* and does not mention staging. The "including any kernel that declares workgroup staging" clause lives in the **enum-level** doc block, above `pub enum ContractionF32PlanWitnessError {`. Found by `worker-site411`, which was auditing a sibling ticket carrying the same conflation from the same source. **The defect this ticket names survives all three corrections** — the predicate is still broader than the enum doc's "including" implies — but the citation was wrong and a worker following it would have edited prose that says something else.

**Why this is worth its own ticket rather than folding into the join.** The two are separable and fail differently. The join adds a route for kernels whose region states the tree; this narrows the arm so kernels with *structurally irrelevant* staging stop being refused for a reason that does not apply to them. Landing only the join would leave the over-broad arm in place for every staged kernel the join does not reach, still reporting a cause that misdescribes it.

**This fails closed, so it is not a wrong answer.** It is an over-refusal and a false doc claim. Ranked accordingly.

## Required work

- Re-audit the Fact at your base and report a per-Fact verdict; read both refusal sites, not one.
- Decide **by reading** whether staging that carries no combine structure is distinguishable at this layer at all. **If it is not, the correct outcome is to repair the documentation to state the real predicate and record why the arm must stay broad** — that is a valid close, and better than a narrowing that guesses.
- If it is distinguishable, narrow the arm and keep the refusal fail-closed: an unrecognized staging shape must still refuse, never fall through to a derived tree.
- Perturb both sites separately with quoted failure text. Show a kernel that should now be admitted being admitted, and one that must still refuse still refusing.

## Non-goals

The scheduled-region join — that is [`join-the-scheduled-region-into-the-contraction-witness`](join-the-scheduled-region-into-the-contraction-witness.md); any new encoding or identity change; and recovering a combine tree from a kernel body.

## Closes when

Either the refusal names only what it cannot read, with both admitted and still-refused cases watched, or the arm is recorded as necessarily broad with its documentation repaired to state the real predicate and a reconsideration trigger attached.
