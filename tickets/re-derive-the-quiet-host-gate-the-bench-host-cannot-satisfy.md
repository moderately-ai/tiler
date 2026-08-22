---
id: re-derive-the-quiet-host-gate-the-bench-host-cannot-satisfy
title: Re-derive the quiet-host gate the bench host cannot satisfy
status: done
priority: p2
dependencies: []
related: [calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol, refresh-the-deferred-triggers-whose-stated-reason-is-now-false, measure-thread-execution-width-on-the-standard-metal-profiles-own-host]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, protocol, unfireable-checks, scheduling]
---
## User-visible outcome

The quiet-host precondition on measurement protocols is one the bench host can actually satisfy when it is idle, so a timing run is gated on the absence of competing work rather than on a number that machine never reaches.

## Why this exists

Filed 2026-08-22 by the coordinator while checking a recorded hold's release trigger, which is what that check is for.

**Fact — the frozen tile-width protocol gates timing on a load average below 0.5, and the bench host's idle baseline is roughly 2.3.** Verified by the coordinator over two independent observations days apart: the delivering lane recorded `2.13 2.17 2.16`, and a later check recorded `2.32 2.38 2.32`. The machine has been up 21 days with one user.

**Fact — that load is not contention, and will not decay.** Diagnosed read-only at `835fdd3f`. Nothing is CPU-bound: after the measuring `ssh` itself (14.1%), the top consumers are the Tailscale network **system extension** at 2.8% and the `AppleBCMWLAN` **DriverKit extension** at 2.5%, both running since boot. There are **no** processes in uninterruptible or disk-wait state, and the runnable count is **2**. On macOS these persistent system extensions carry load average without consuming meaningful CPU, so the figure is a floor imposed by the OS configuration, not a queue that drains.

**Inference — the gate is unsatisfiable as written, and the protocol refuses on it.** `--mode timing` declines outright above the threshold. So the gate does not merely delay the run; **it forecloses it permanently**, which is the same defect class the deferred-pool audit just found across seven trigger checks: a condition whose satisfying case is unreachable, indistinguishable from "not yet" until someone asks what it would take to say yes.

**Fact — the bench host has *not* drifted, and that is worth recording separately.** `sw_vers -buildVersion` on it returns **`26A5388g`**, exactly the build the ledger pins — unlike this coordination host, which is on `26A5416b`. Whatever replaces the load gate, the host itself is the right one to measure on.

## Required work

- Re-audit every Fact at your base and report a per-Fact verdict. **Re-observe the idle baseline yourself**, more than once, and record the observations rather than inheriting mine.
- Derive a precondition that discriminates *competing work* from *baseline*. Candidates worth costing rather than assuming: idle CPU percentage; load measured against a recorded per-host baseline; a runnable-process count; or an explicit check that no other measurement session holds the host. **Say what each would let through and what it would refuse.**
- **Amend the protocol before any timing run, and commit the amendment before the harness executes**, exactly as the original pre-registration did. A gate corrected *before* any measurement preserves pre-registration; one corrected after a run does not, and that distinction is the whole reason the protocol was committed ahead of its harness. Record the amendment with its date and reason.
- **Perturb the new gate in both directions**: show it refusing while real competing work runs, and admitting on the idle host. Quote both. A gate that only ever admits is the defect this ticket exists to remove, reintroduced.
- Check whether any other retained protocol carries the same threshold. Report findings **and** clean results.

## Non-goals

Running the timing sweeps themselves — they belong to their own tickets and unblock when this lands. Changing what is measured, the beneficiary profile key, or any conclusion of the frozen protocol beyond its host precondition. Reconfiguring or quieting the bench host: the extensions carrying the baseline are the machine's networking, and disabling them to satisfy a gate would change the evidence environment, which AGENTS.md reserves to Tom.

## Closes when

The quiet-host precondition admits the idle bench host and refuses a genuinely busy one, both directions have been watched with their output quoted, the amendment is committed before any harness run with its reason recorded, and the sibling-protocol scan is reported with its clean results.

## Fact audit at base `c3cf6f9f`, 2026-08-22

Every Fact re-read at this base from its source, and the bench host re-observed rather than inherited. Two claims needed repair.

**Fact 1 — the 0.5 gate and the ~2.3 baseline. Verified in substance; its supporting evidence was overstated and is repaired.** The gate is real: `spikes/scheduling/metal_contraction_tile_width/tile_width_sweep.py "LOAD_GATE = 0.5"` with the refusal in `def mode_timing`, and the protocol's `The run aborts unless the bench host's one-minute load average is`. The baseline is real but the stated figure is the top of the band, not its centre: twenty-plus one-minute observations on 2026-08-22 give **1.86–2.47**, not "roughly 2.3". **The claim that the two observations were "days apart" is false.** Both are dated 2026-08-22 — the protocol recording `2.13 2.17 2.16` was committed at `a25f0a2f` on 2026-08-22 15:33, and the coordinator's `2.32 2.38 2.32` check was the same afternoon, about two hours later. Repaired with genuinely older evidence rather than by softening the claim: this repository already retains the same floor on the same host nine days earlier, at `spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-13-request-wide-macos-27.0-m3-pro.stdout.txt` reporting `loadavg={ 2.22 2.39 2.26 }` and its 2026-08-14 sibling reporting `loadavg={ 2.18 2.23 2.24 }`, both identifying `Darwin Thomass-MacBook-Pro.local`, `Apple M3 Pro`, `ncpu=11`. Host identity confirmed live as `Mac15,6`, `T6030`, 11 cores. So the conclusion is better supported than the ticket argued, on durable in-tree artefacts rather than transient observations.

**Fact 2 — "that load is not contention, and will not decay". The conclusion is verified; the attribution is false, and the difference changes the design.** The conclusion holds and was re-derived independently: the host sits at **98.8–99.4% CPU idle and 0–1% GPU device utilization while its one-minute load average is 2.2**, with `ps -Axo pid,state,wchan,etime,comm | awk '$2 ~ /^U|^D/'` empty. A quantity reading 2.2 on a machine doing nothing cannot discriminate competing work at any threshold. **But "Nothing is CPU-bound" is not true of this host.** It carries a live console login (`who` reports `tsanterre console` since Aug 1) running **Google Chrome across seven processes**; at one probe a Chrome renderer was the top consumer at **88.8% CPU** with the main process at 21.7%, and `airportd` at 2.6% outranked the `AppleBCMWLAN` extension the ticket names. Sixty consecutive one-second idle samples are **bimodal**: a quiet mode at 97.2–99.7% and an episodic burst at 84.2–89.8%, about 1.1 of 11 cores, lasting ~18 s in that window. The competing work is intermittent, not absent. This is load-bearing rather than pedantic: it is why the replacement gate refuses a meaningful fraction of arbitrary reads on this host today, and why the protocol now instructs the operator to quiesce the interactive session instead of re-reading until green.

**Inference 3 — the gate is unsatisfiable and forecloses rather than delays. Verified** by reading `mode_timing`: the refusal is unconditional above the threshold, at start and again at end.

**Fact 4 — the bench host has not drifted. Verified.** `sw_vers -buildVersion` returns `26A5388g` and `-productVersion` returns `27.0`, the pinned build.

**Imprecise, minor.** The briefed runnable count of `2` is an instantaneous sample of a fluctuating quantity; I observed 2, 3, and 4 on the quiet host. The counting pipeline is itself runnable, so the metric can never read 0 — which is part of why it was rejected as a gate component below.

## Outcome

**The gate is replaced, and the amendment is committed before any wall clock has been read.** Pre-registration is intact and provable from history rather than asserted: the protocol was frozen at `a25f0a2f`, corrected at `431722dd`, and its harness built at `7ed93ab9`, all on 2026-08-22; no `results/` directory exists in the spike, no timing artefact is tracked by `git ls-files`, and the `Result` section still reads *Not yet run*. No timing sweep was run by this lane — that work belongs to `calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol`.

**Candidates costed, and two rejected for reproducing the defect in a new metric.** A *runnable-process count* cannot discriminate: its quiet baseline is 2–4 and a single competing job adds 1, inside the noise. An *explicit no-other-session check* in its natural form — no other interactive login — is **unsatisfiable on this host for the same reason the load gate was**, because the console session is permanent; and in its `pgrep`-pattern form it is the deferred-audit failure class exactly, admitting anything the pattern misses and reading green forever if the pattern rots. What survives from that candidate is a non-pattern form: an exclusive advisory lock, which cannot silently match nothing.

**Chosen: four fail-closed components, all four perturbed.** Mean CPU idle over ten one-second samples `>= 95%` as the primary discriminator; GPU device utilization `<= 5%`, since `GPUEndTime - GPUStartTime` makes the GPU the contended device; one-minute load average `<= 3.5` as a baseline-relative lagging cross-check, never an absolute quiet figure; and an exclusive measurement lock. The 95% floor sits in a **measured empty band** — nothing was observed between 90% and 95% across sixty samples — rather than being chosen round.

**The load average is demoted on measured evidence, not on principle.** Twenty-five seconds into four cores of deliberate competing work, CPU idle had already fallen to `61.17%` while the one-minute load average was still `2.89`, inside any ceiling that also admits this host's 2.2 baseline. A gate resting on load alone would have admitted that run.

**A sampling defect found and fixed in passing.** `top`'s first sample is a since-boot average, not an instantaneous reading. Written naively against `top -l 1`, the probe read 80.40% and 69.18% idle within minutes of sustained sampling reading 98.8–99.4% — so it would have refused a quiet host while appearing to work. The harness requests one extra sample and discards it.

**Both directions watched, with output quoted in [the spike README](../spikes/scheduling/metal_contraction_tile_width/README.md).** Admits the quiet host (`98.65%` idle, GPU `1%`, load `2.33`, lock held, verdict `ADMIT`, quoted in full and unedited). Refuses competing CPU work (`61.17%`, then `51.92%` with load `4.24` firing as a second refusal). Refuses an unreadable probe rather than passing it — the `Device Utilization %` key was renamed on its way out of `ioreg`, reproducing the `system_profiler` rename that left a deferred trigger reading *not fired* forever, with a pass-through stub on the same `PATH` as the negative control so the refusal is attributable to the rename and not to the stub. Refuses a second measurement session holding the lock, and admits again once it exits. The subject was perturbed in every case; no assertion was edited. Unperturbed, the gate was also seen to admit four times and refuse twice within the same few minutes, tracking the host's real desktop bursts.

**What it would take for this gate to say no, and confirmation each case is reachable** — the check the deferred-pool audit exists to force. All four refusal paths were observed firing on the bench host, and the admitting path was observed on the same host in the same minutes. No component's satisfying case, and no component's refusing case, is unreachable.

**Scope respected.** The bench host was not reconfigured or quieted; the system extensions carrying the load floor were not touched. The CPU load used for perturbation was four bounded busy loops that self-terminated after 150 s, confirmed gone afterwards, and the `ioreg` stub was removed. No `crates/` file was edited.

**Unverified, stated as such.** The *attribution* of the residual 2.2 load floor to the Tailscale and `AppleBCMWLAN` extensions specifically is not something I confirmed; establishing it would need `powermetrics` under `sudo`, which I did not run. What is verified is the correlation that matters for the design — load ~2.2 while CPU idle is 98.8–99.4% and GPU is 0–1% — which is sufficient to reject the load average as a discriminator regardless of which extension carries it.
