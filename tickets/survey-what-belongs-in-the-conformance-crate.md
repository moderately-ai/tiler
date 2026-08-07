---
id: survey-what-belongs-in-the-conformance-crate
title: Survey what belongs in the conformance crate
status: in-progress
priority: p2
dependencies: []
related: [admit-the-conformance-crate-to-the-workspace, decide-where-a-device-reaching-conformance-test-may-live]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, architecture]
claimed_from: todo
assignee: agent-survey
lease_expires_at: 1786123387
---
## User-visible outcome

A read-only survey that names, ticket by ticket, what should move into `crates/tiler-conformance` and what should stay where it is — so migration happens as reviewed decisions rather than as drift, and the crate's boundary is derived from the work rather than asserted.

## Why this exists

**Asked for by Tom on 2026-08-07**, in the same answer that admitted the crate: assess what else should move and file it as future tickets. The crate was accepted on the argument that conformance is a *missing component* rather than a homeless file, and that argument makes a claim this ticket has to test — that the work currently scattered across four crates belongs together.

**Fact — the scattering is measurable.** Five open conformance tickets and no two share a scope set: `route-the-contraction-conformance-through-the-staged-oracle`, `route-the-index-region-conformance-through-the-staged-oracle` and `retain-the-selected-semantic-candidate-for-the-conformance-oracle` are `implementation/compiler`; `retain-contraction-conformance-evidence` adds `implementation/reference`, `contracts/numerics` and `research/scheduling`; `conform-the-bf16-vertical-end-to-end` adds `implementation/runtime`. Conformance tests exist under `crates/tiler-compiler/tests/`, `crates/tiler-reference/tests/`, and `crates/tiler-build/tests/`.

## What this must produce

**A classification of every candidate, with the reason, and it must be willing to conclude "stays".** A survey that recommends moving everything it looked at has not discriminated. For each candidate name which of these it is:

- **Cross-layer executed evidence** — a run spanning produce and consume, compared against the reference oracle. Moves.
- **Layer-local evidence** — a test of one crate's own behaviour that happens to use the word conformance. **Stays**, and the survey says so explicitly, because the crate's stated anti-goal is becoming the place tests go when nobody wants to decide.
- **Oracle *plumbing* rather than evidence** — machinery currently inside `tiler-compiler` because there was nowhere else. This is the interesting class and the one the missing-component argument rests on. Decide whether it is genuinely conformance machinery or genuinely compiler machinery; both answers are available and the survey must argue rather than assume.

**Then the harder question, which is the crate's actual long-term shape.** The accepted decision describes conformance as the refuting half of a declaring/refuting pair, whose eventual job includes *producing* support-matrix rows from runs that happened rather than having them hand-asserted in markdown. `AGENTS.md` records that documentation has no automated validator and that a ticket advancing a support-matrix row must remember to file the ledger update — a "must remember" that has already produced measurable drift. Assess, without building anything:

- Which support-matrix and ledger cells could be **derived** from an executed run carrying its host, OS build, toolchain, GPU family, and extent, and which are claims no run can make.
- Whether the maturity ladder (reserved type / architectural seam / implemented support / tested guarantee) and the evidence ladder (`SoundProof` / exhaustive finite / empirical / normative / `Unknown`) can be **stamped** by a harness or must stay a writing convention. Name what each would require.
- What the crate would need to parameterize over as targets multiply — the matrix is `operation family x dtype x contract x target profile x shape class`, and deferred work already names iOS profiles, a CPU vector tier, subgroup tiers, and CUDA. Hand-written per-combination tests do not survive that multiplication; say what shape does.

## Explicit non-goals

**Move nothing.** This is a read-only survey; every migration it recommends is a separate ticket that a later change executes. Do not design the harness API — the crate's public surface is reserved under ADR 0075 and admitting the member accepted no API. Do not re-open where the crate lives; that is decided. Do not build the support-matrix derivation; assess its feasibility and cost.

## Required evidence

Read the candidates in full rather than classifying from titles or grep — the argument for the crate came from reading, and a survey that classifies from names would be weaker evidence than the thing it is checking. Cite each candidate by path and say what reading it established. Where a classification is genuinely uncertain, say so and name what would settle it, rather than picking to keep the table tidy.

## Closes when

Every candidate is classified with its reason, the "stays" population is non-trivial or its emptiness is argued, each recommended migration is a filed ticket, the long-term questions above are answered or recorded as deferred with triggers, and the crate's stated boundary is either confirmed by the survey or revised with evidence.

## Graph maintenance

Filed 2026-08-07 by the coordinator on Tom's instruction. Deliberately not blocking [`admit-the-conformance-crate-to-the-workspace`](admit-the-conformance-crate-to-the-workspace.md) or the BF16 vertical: the crate's first content is already decided, and holding it behind a survey would invert the smallest-useful-slice order the admission ticket was scoped for.
