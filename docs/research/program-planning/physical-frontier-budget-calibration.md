---
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.physical-frontier-budget-calibration"
kind: "research"
title: "First physical-frontier provider-count and raw-outcome limits"
topics: ["program-planning", "budgets", "measurement", "host-performance"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "exhaustive-finite"]
informs: ["tiler.contract.optimizer"]
ticket: "calibrate-the-physical-frontier-provider-and-outcome-budgets"
---

# First physical-frontier provider-count and raw-outcome limits

- **Status:** measurement complete; the numbers below are calibration inputs for [`replace-provider-offer-with-a-host-bounded-frontier-sink`](../../../tickets/replace-provider-offer-with-a-host-bounded-frontier-sink.md). They are not yet `DeterministicBudgets` fields.
- **Ticket:** [`calibrate-the-physical-frontier-provider-and-outcome-budgets`](../../../tickets/calibrate-the-physical-frontier-provider-and-outcome-budgets.md).
- **Experiment:** [Physical-frontier provider-count and raw-outcome budget calibration](../../../spikes/program-planning/physical-frontier-budget-calibration/README.md).
- **Measurement date:** 2026-08-13, Apple M3 Pro, macOS 27.0 `26A5388g`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`.

## Recommended first limits

**Measurement.** Against the governed five-operation program (`program(4, 3)`), `STRICT_F32`, prototype target-neutral profile:

| Resource | First limit | Headroom | Binding evidence |
| --- | --- | --- | --- |
| Provider count, governed included | 32 | 32 (working empty/decline extras through 64; untyped `InvalidCompilerOutput` at 128 all-decline extras) | Empty extras add 0.3 µs per invocation. 64 extras are 1.11× the 3205 µs governed floor. |
| Raw proposal+decline outcomes, request-scoped | 256 | 256 | Sized from admitted-proposal cost (39 µs incremental) not decline cost (1.1 µs). Sixteen equal-cost specialists emit 272 extra outcomes at 4.62× the floor and 82 MB peak RSS. |

**Fact.** The historical value eight is not a calibration of either axis. A request-scoped eight would refuse the ordinary governed compile: one extra observer is asked 17 times, and the governed provider is asked about the same 17 subjects. Empty extras through 64 do not approach a host-time cliff, so eight is also not a provider-count measurement.

**Fact.** Proposal and decline incremental costs differ by about 34× on this host. One raw-outcome count is still the accepted cardinality bound from [`decide-whether-the-implementation-frontier-owes-a-retention-budget`](../../../tickets/decide-whether-the-implementation-frontier-owes-a-retention-budget.md). It is not a uniform work unit. The first limit is therefore taken from the expensive outcome so a decline-heavy population cannot license a proposal-heavy compile the sweep did not time.

## Census

**Fact.** The compiler-owned production `PhysicalImplementationProvider` population is one: `GovernedPhysicalProvider`. The check reads `crates/tiler-compiler/src` after cutting `#[cfg(test)]` modules and `tests.rs` files.

**Fact.** One compile of the five-operation program enumerates 17 distinct region subjects. An installed observer is invoked 17 times; two observers, 34 times. Three of those subjects have a single-dispatch baseline the public seam can specialize; fourteen do not, and a specialist declines them.

**Measurement.** Complete-plan alternatives on this program grow as `extra + 1` under equal-cost specialists, not as a Cartesian product of every region. Infeasible specialists do not grow the selected set. Empty and decline extras do not grow it.

## Implicit wall

**Measurement.** 1088 extra declines (64 all-decline providers) compile. 1920 extra declines (128 all-decline providers) refuse as `InvalidCompilerOutput`. That is an untyped capacity wall, not a budget. The first limits must fire before it.

## Unsupported guarantee

The limits bound compiler-owned accepted outcomes and the verification that follows an emission. They do not bound arbitrary native provider computation or allocation before an emission, and they do not replace `physical_plan_combinations`.
