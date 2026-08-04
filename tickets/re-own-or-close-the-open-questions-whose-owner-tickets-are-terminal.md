---
id: re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal
title: Re-own or close the open questions whose owner tickets are terminal
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-oq-sweep
lease_expires_at: 1785878542
---
## User-visible outcome

Every question in `docs/open-questions.md` either has a live owner, a stated demand trigger, or is closed into the durable contract that answers it — so no question sits owned by a terminal ticket, which is ownership in name and orphanhood in fact.

## Why

**Fact — the failure mode has already happened once.** Q-ART-008 was owned by `prototype-artifact-family-delivery`, which closed `done` with the close condition unmet, leaving the question unowned until a later worker noticed. A 2026-07-31 audit (reproduce with the script in this ticket's provenance, or re-derive: extract each `Q-*` section's `tickets/*.md` references and compare against ticket statuses) found eight questions whose every referenced ticket is now `done` or `closed`: Q-SEM-001, Q-SEM-007, Q-PLAN-001, Q-PLAN-007, Q-PLAN-009, Q-ART-002, Q-PKG-002, and Q-PKG-003 — the last owned by `prototype-inline-proc-macro-frontend`, which closed the same day. Thirty-three further questions reference no ticket at all; most are deliberate demand-triggered reservations, but nothing distinguishes a stated trigger from an accidental orphan except reading.

## Work

For each of the eight: read the question against what its terminal owner actually delivered; close it into the contract that now answers it, re-point it at the live successor ticket, or record why it stays open with a stated trigger. For the thirty-three unreferenced: verify each states a closure trigger a reader can evaluate; give any that does not either a trigger or an owner. Do not close a question whose answer does not live in a durable contract or an accepted ADR.

## Closes when

The audit re-run reports zero questions owned solely by terminal tickets, and every unreferenced question carries an explicit trigger.

## Four named questions the stated Work would pass over (2026-08-01)

**Why this widening is needed rather than implied.** The Work above says of the thirty-three unreferenced questions: "verify each states a closure trigger a reader can evaluate." Four questions **satisfy that check and are nonetheless stale** — each states an evaluable trigger, and the trigger has fired, or its owner has gone terminal, or the evidence it waits on has already been supplied. This ticket's outcome could therefore be met with all four defects standing. They are named here so the sweep reaches them; each was verified against the tree at base `0017345`.

- **Q-ART-006 — rust-analyzer cold and warm expansion costs** (`docs/open-questions.md:247`). Its last open column is stated at `:256-258`: "What remains is the *edit* column: that needs a real language-server session rather than `analysis-stats`, which loads a project and expands once." That measurement was supplied at [`avoid-toolchain-resolution-on-a-warm-expansion-cache-hit`](avoid-toolchain-resolution-on-a-warm-expansion-cache-hit.md):79-85 — a real LSP session (initialize, didOpen, then didChange edits each followed by a `textDocument/semanticTokens/full` round trip) under `rust-analyzer 1.97.0-nightly`, expansions counted exactly, in-region edits at 137–217 ms. That ticket is `done` and has **no Graph-maintenance section** (its headings are `Why this exists`, `Closes when`, `Outcome`), so nothing propagated the result. Close the question into the durable record that now carries it, or restate the remainder.
- **Q-SEM-004 — First-profile transcendental tuples** (`:97`). Both reasons the question gives at `:102` for staying open were discharged on 2026-08-01. "Adopting the `exp` bound needs a registered cross-metric implication because Apple's ULP definition is a different key" — that implication is registered, as `RegisteredImplication::ScaledMetric` at `crates/tiler-compiler/src/target/accuracy.rs:139` with its derivation attached. "Adopting any correctly rounded entry needs the rounding mode Metal's §8.2 declines to fix" — `docs/roadmap.md:408` records the observation that Gap 4's rounding-mode question does not bind an entry stated as a ULP bound, and that a faithful contract is metric-free. What is genuinely still open is the **reference half**, which the question itself calls "wholly open". Restate the remainder as that and give it an owner; do not close it on the backend half alone.
- **Q-PLAN-011 — CPU execution and vector profile** (`:331`). Its trigger at `:334` is "the CPU backend enters the active roadmap", and it sits under a deferred-until-an-explicit-trigger heading. The trigger fired: [`prototype-a-bounded-scalar-cpu-backend-vertical`](prototype-a-bounded-scalar-cpu-backend-vertical.md) is `done`, ADR 0093's CPU vector-lane tier is accepted, and three implementation tickets are filed against it. Shared with [`sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired`](sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired.md), which names it among its starters — coordinate, and do not both make the edit.
- **Q-SEM-015 — Tensor contraction** (`:298`). Its Owner/tracking line at `:300` names [`scope-einsum-contraction-support`](scope-einsum-contraction-support.md), which is `done` — the exact terminal-owner pattern this ticket exists for, missed by the original audit because the line also names the Milestone 6 framing and so does not read as unowned. Its trigger bullet at `:301` reserves a contraction choice that had no node until now; repoint that clause at [`decide-whether-a-contraction-may-consume-more-than-two-operands`](decide-whether-a-contraction-may-consume-more-than-two-operands.md). The third reserved choice in the same bullet was decided on 2026-08-01 — declined, recorded on [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md) — so `:301`'s description of it as an open choice needs correcting too.

**The check this adds to the closing condition.** A question passes only if a reader can evaluate its trigger *and* the trigger has not already fired. "States an evaluable trigger" was the original bar and all four of these clear it; evaluating each trigger against the tree is what this widening requires.
