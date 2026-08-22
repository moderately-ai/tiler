---
id: write-the-frontier-calibration-s-unwritten-quiet-host-gate
title: Write the frontier calibration's unwritten quiet-host gate
status: done
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

## Fact audit at base `3ba89314`, 2026-08-22

Every clause of the reported Fact re-read from its source at this base rather than inherited. The Fact is **verified in substance**; two clauses are imprecise, and one decisive observation was missing.

**"Refused a run at loadavg `{3.08 2.45 2.26}`" — verified, but not from the spike.** The source is [`measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro`](measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro.md), anchor `Measurement hold, 2026-08-14 00:50 EDT`, which records the formal pre-run load as `{ 3.08 2.45 2.26 }` and states that `record` was never started. The spike README carries the event only as `A 00:50 idle precheck was held before launching because Chrome used 82 percent CPU`, with no load figure at all. That relocation supports the ticket's thesis rather than weakening it: the numbers that decided the refusal were never written into the record they gate.

**"Accepted one at `{2.18 2.23 2.24}`" — verified from the retained artifacts, not only the prose.** `spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-14-request-wide-macos-27.0-m3-pro.stdout.txt` opens with `loadavg={ 2.18 2.23 2.24 }`, and `spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-14-request-wide-macos-27.0-m3-pro.environment-before.txt` records `load averages: 2.18 2.23 2.24` against the post-run `load averages: 1.61 2.07 2.17`.

**"One refusal decision turned on a Chrome renderer observed at 82.0% CPU" — verified, but the phrasing implies two refusals where there was one.** The load reading and the Chrome reading are the *same* event, not separate refusals: at 2026-08-14 00:50 EDT the load was `{ 3.08 2.45 2.26 }`, and an immediate process control observed one Chrome renderer at 82.0 percent CPU and its parent at 24.3 percent with load `{ 2.84 2.42 2.25 }`. Exactly one refusal is recorded anywhere in this lane. Minor: the spike README rounds the figure to `82 percent`; `82.0` is the ticket's.

**"There is no threshold, no probe, and nothing a rerun could apply" — verified, and the source is more emphatic than the claim.** `spikes/program-planning/physical-frontier-budget-calibration/src/main.rs "fn host_record"` shells out to `sysctl -n vm.loadavg` and formats the result into a host string; there is no comparison, no branch, and no refusal anywhere in the file. `main()` dispatches `None | Some("record") => record(...)` with no precondition of any kind. The spike README's sole instruction was the sentence *Run the release `record` line only after the idle/noise precheck.*, since replaced, which enumerated twelve *recording* commands and zero thresholds. The spike is a record-don't-gate design, which is the choice the tile-width protocol names as wrong for a wall-clock measurement.

**Missing from the ticket, and it is the decisive evidence for the ticket's own thesis.** The same spike **accepted** a run at load `{ 6.82 3.66 2.70 }`. It is retained at `spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-13-macos-27.0-m3-pro.json` as `loadavg={ 6.82 3.66 2.70 }` and stated under the spike README's heading `Historical single-target host-runtime record` with the rationalization `so the floor is readable even under that load`. That acceptance is roughly 2.2 times the load of the refusal recorded about six hours later on the same host. No threshold on any single quantity yields both decisions, which is the demonstration — rather than the argument — that the gate lived in the operator and not in the record. The ticket reasoned from two observations where three were available and the third settles it.

## Outcome

**Decision: adopt the sibling gate, unchanged. A second rule was considered and rejected on a specific asymmetry rather than on cost.** The gate is the contraction tile-width lane's, in every component and every threshold, invoked as `python3 ../../scheduling/metal_contraction_tile_width/tile_width_sweep.py --mode gate` from the spike directory and chained with `&&` ahead of `record` in both documented rerun blocks. The reasoning is not that adopting is cheaper. Both a copy and a reference can break, but they break in opposite directions: a copied threshold that falls out of date keeps reading **green** while admitting a contaminated host, whereas a moved file or renamed mode is a **nonzero exit**, which the chain reads as a refusal. Silent divergence against loud absence is the whole argument, and it matters here because `spikes/` sits outside every repository gate, so nothing would notice a copy drifting.

**No component was added, and that was decided on this lane's own evidence.** The subject is Tiler host runtime and process RSS rather than kernel time, which invites dropping the GPU component and promoting the dimensions this spike already snapshots — AC power, thermal state, memory pressure, and a concurrent-build count. All refused. The one refusal this spike ever made turned on load and on a renderer's CPU, both of which the CPU-idle component discriminates directly and earlier; power, thermal, and swap were all *clean* at that refusal, so promoting them would write down a rule stricter than the one actually applied. A concurrent-build count is redundant behind the idle floor and is the name-matching shape the sibling lane rejected by name; `spikes/program-planning/reduction-partition-calibration/src/main.rs "fn concurrent_build_processes"` stays correct for its own study and is not generalized.

**The gate is an external probe rather than an in-binary check, because this record has two rerun paths.** One runs the current executable; the other runs a detached worktree at `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6`. A gate compiled into either binary cannot reach the other path, so it would police half its subject while appearing complete. A further constraint confirms the choice: the spike's `Cargo.toml` sets `unsafe_code = "deny"`, so re-implementing the advisory lock in Rust would have required a new dependency, and dropping the lock would have been the second rule starting.

**Both directions of the chained command watched on the bench host, quoted in the spike record.** Admitting: `chain-exit=0` at 98.64% idle, GPU 0%, load 1.88, with the `record` stand-in reached. Refusing under four bounded busy loops: `chain-exit=1` at 53.03% idle and load 3.71, with the stand-in **not** reached — which is the property specific to this ticket, since the four components themselves were already perturbed by the lane that derived them. Those perturbations transfer as evidence rather than by analogy: the probe was verified byte-identical across both hosts at SHA-256 `1815079e7495df2f16d9ecb6d8af4fbc9bc66be5ec5f1f6f081e8f8514db9ff0` before any read. No wall clock was read and no calibration timing was re-run; `echo` stood in for `record` throughout.

**A new observation that independently strengthens the load average's demotion.** The sibling demoted it on evidence that it lags on the way *up* — 61.17% idle while load still read 2.89. The recovery read here shows the same lag on the way *down*: immediately after the busy loops ended, CPU idle had returned to `98.29%` while the one-minute load average still read `4.06` and refused. The instrument lags in both directions, which is a better reason to keep it as a cross-check than either observation alone.

**Reading was not selected for a green verdict.** Every read count was fixed in advance and every verdict reported: a six-read unperturbed census returned four admits and two refusals; a pre-declared single read after the perturbation refused at 88.54% idle and was reported as such rather than retried; a pre-declared three-read chain census returned two admits and one refusal. Across the ten arbitrary reads — excluding the perturbed read and the burner-recovery read, whose load was induced — six admitted and four refused, matching the live console session the amended protocol describes.

**A consequence recorded rather than hidden.** Writing the precondition down makes the superseded 2026-08-13 single-target row fall outside it: taken at `{ 6.82 3.66 2.70 }`, this gate refuses it at start and at end. That row is already labelled historical and superseded by the corrected request-wide result, and nothing here re-runs it, withdraws a number, or changes a value — but a reader is entitled to know it is not reproducible under the rule now stated. The 2026-08-14 record the current result rests on is inside every component.

**Bench host left as found.** The probe directory `~/tiler-frontier-gate-probe` was removed and confirmed gone, the four busy loops self-terminated after 150 s and were confirmed absent by `pgrep`, no stub was placed on any `PATH`, nothing was quiesced or reconfigured, and no ticket status was changed. `/tmp/tiler-contraction-tile-width-sweep.lock` predates this lane — it was present before the first read — and is the sibling gate's designated lock path, so it is deliberately left in place.

**Unverified, stated as such.** Whether the 2026-08-14 run would have passed the CPU-idle and GPU components is not established: neither quantity was recorded at the time, and only the load average and a process snapshot survive. The claim made is the weaker one the evidence supports — the run is inside every component that *was* recorded, and its process snapshot (highest non-observing process at 5.0 percent CPU) is consistent with a high idle mean. This lane re-ran no calibration timing, so nothing here is evidence about any measured value.
