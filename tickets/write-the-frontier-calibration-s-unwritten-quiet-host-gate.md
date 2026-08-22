---
id: write-the-frontier-calibration-s-unwritten-quiet-host-gate
title: Write the frontier calibration's unwritten quiet-host gate
status: todo
priority: p2
dependencies: []
related: [re-derive-the-quiet-host-gate-the-bench-host-cannot-satisfy, calibrate-the-physical-frontier-provider-and-outcome-budgets]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, protocol, unfireable-checks]
---
## User-visible outcome

The physical-frontier calibration spike states the host precondition it already enforces by judgement, so a rerun admits and refuses the same hosts its retained results were taken under.

## Why this exists

Found 2026-08-22 by `worker-quietgate` while scanning for siblings of the unsatisfiable load gate it had just replaced. It reports this as a **latent instance of the same class**, and flagged it rather than folding it in.

**Fact (reported, unverified by the coordinator) — the spike gates on host quiet by judgement, with no committed rule.** `spikes/program-planning/physical-frontier-budget-calibration` **refused** a run at loadavg `{3.08 2.45 2.26}` and **accepted** one at `{2.18 2.23 2.24}`, and one refusal decision turned on a Chrome renderer observed at 82.0% CPU. None of that is a written precondition: there is no threshold, no probe, and nothing a rerun could apply.

**Why this is the same class rather than a tidy-up.** An unwritten gate is not a lax gate — it is one whose behaviour lives in whoever ran it. A rerun cannot reproduce the admission decision, so the retained numbers are not reproducible in the sense AGENTS.md requires of a measurement, even though the numbers themselves are sound.

**The sibling that was just repaired is the template, and its lesson is measured rather than argued.** The tile-width protocol's `load < 0.5` gate could never be satisfied on the bench host, whose idle band is 1.86–2.47. Its replacement is four fail-closed components — mean CPU idle ≥ 95% over ten one-second samples, GPU utilization ≤ 5%, a **baseline-relative** load ceiling demoted to a lagging cross-check, and an exclusive lock. Load was demoted on evidence: 25 seconds into four cores of competing work, CPU idle read **61.17%** while load still read **2.89**, inside any ceiling that admits the idle baseline.

**One sampling defect worth inheriting rather than rediscovering.** `top`'s first sample is a since-boot average. The lane's own first probes read 80.40% and 69.18% idle on a host that sustained sampling put at 98.8–99.4%. Its probe discards the first sample — without that, the gate refuses quiet hosts.

## Required work

- Re-audit the Fact at your base and report a verdict; **re-read the retained results and the harness yourself** rather than inheriting the figures above, none of which the coordinator has verified.
- Decide **by reading** whether this spike needs a gate of its own or should adopt the tile-width one. Adopting is likely right and cheaper — say so explicitly if you conclude it, rather than writing a second rule that can drift from the first.
- Whatever you land, **perturb it in both directions and quote both**: refusing under real competing work, and admitting on a genuinely quiet host. A gate observed only admitting is the defect this ticket exists to remove, reintroduced.
- State what it would take for the gate to say *no*, and confirm that case is reachable.
- Do **not** re-run the calibration timings. The retained numbers stand; this ticket is about making their precondition reproducible.

## Non-goals

Re-measuring the frontier budgets; changing any accepted budget value; quiescing or reconfiguring the bench host, whose console session is an operator matter and whose networking extensions AGENTS.md reserves to Tom.

## Closes when

The spike states a host precondition a rerun can apply, both directions have been watched with their output quoted, and the choice between adopting the sibling gate and writing a second one is made by reading and recorded.
