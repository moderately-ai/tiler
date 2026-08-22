---
id: scope-the-ieee-decimal-vertical
title: Scope the IEEE decimal vertical
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, own-the-dtype-support-maturity-matrix, state-the-non-enumerable-float-conformance-profile]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, decimal]
---
## User-visible outcome

`tiler::decimal32@1`, `tiler::decimal64@1`, and `tiler::decimal128@1` have an owner for the step past identity, and their two storage encodings are treated as the distinct things they are.

## Why this exists

**Fact.** [ADR 0035](../docs/decisions/0035-recognize-ieee-decimal-floating-formats.md) recognizes the three formats and keeps densely packed decimal and binary-integer-decimal as physical encodings of the same logical dtype. [The dtype support ledger](../docs/dtype-support.md) records them registered with interchange width and coefficient precision, an architectural seam at the physical carrier, and absent everywhere else.

**Fact — the obligation that separates this from every binary float is the storage one.** [The mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md) states that "DPD and BID are separate storage encodings, and a bit-preserving operation or ABI must distinguish them even though they encode the same logical decimal format". A binary float has one encoding per format; decimal has two, so the ABI and any bit-preserving operation owe a distinction no binary track owes.

**Fact.** decimal32 is exhaustively enumerable and decimal128 is not. D-7 reuses the bounded-measurement evidence-class methodology that [`state-the-non-enumerable-float-conformance-profile`](state-the-non-enumerable-float-conformance-profile.md) (D-3, for binary `f16`/`f64`/`f128`) is charged to put in Correctness and testing once that ticket closes; it does not own a decimal-applicable answer today, and D-7 must either consume a landed D-3 profile framework or derive decimal128's bounded universe explicitly.

## Activation trigger

A named frontend or accelerator consumer requires a decimal tensor element. The taxonomy is explicit that current GPU tensor arithmetic does not imply execution support, so backend availability does not fire this.

## Closes when

The trigger has fired and the vertical is stated including which storage encoding a program's bytes carry — or decimal is explicitly excluded from the intended product surface by a recorded decision.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-7 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).

## Trigger check log

- 2026-08-04 — **not fired.** Track D-7's trigger is checked in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md) under `#### D-7 — IEEE decimal32`: no named frontend or accelerator consumer requires a decimal tensor element, and backend availability is explicitly not a trigger.
- 2026-08-09 — **not fired.** Decimal32/64/128 remain recognized catalog identities only. No named frontend, model, or accelerator consumer requires a decimal tensor element, and no DPD/BID storage-carrier choice has been requested.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. The trigger has two halves and only one is a repository state. **Checkable half — catalog recognition is not operation admission.** `rg -n 'pub fn \w+_op\(\) -> OpKey' crates/tiler-ir/src/semantic --glob '*.rs'` reports the **19** registered operation-key constructors, and `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` reports **50 unique governed keys** — unique keys through `sort -u`, not lines of output; the census contains `tiler::decimal32@1`, `tiler::decimal64@1`, and `tiler::decimal128@1` as *catalog identities*, and none of the 19 operation constructors is a decimal operation. A decimal operation key joining that list is the changed answer. This check is **one-directional**: it can establish *not fired*, because an operation the trigger needs would have to appear there, and it cannot establish *fired*. **This condition is not mechanically checkable, and saying so is the repair.** The other half — a named frontend or accelerator consumer requiring a decimal tensor element — is a naming act outside the repository. A human must read `docs/dtype-support.md`'s decimal trigger row and `docs/research/numerics/dtype-family-research-tracks.md` under `#### D-7 — IEEE decimal32` for a consumer that has actually asked; backend availability explicitly does not fire it. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
