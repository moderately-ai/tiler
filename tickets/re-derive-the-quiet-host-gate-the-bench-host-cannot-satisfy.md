---
id: re-derive-the-quiet-host-gate-the-bench-host-cannot-satisfy
title: Re-derive the quiet-host gate the bench host cannot satisfy
status: in-progress
priority: p2
dependencies: []
related: [calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol, refresh-the-deferred-triggers-whose-stated-reason-is-now-false, measure-thread-execution-width-on-the-standard-metal-profiles-own-host]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, protocol, unfireable-checks, scheduling]
claimed_from: todo
assignee: worker-quietgate
lease_expires_at: 1787437174
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
