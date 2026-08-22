---
id: calibrate-the-physical-provider-count-at-the-installation-seam
title: Calibrate the physical provider count at the installation seam
status: todo
priority: p2
dependencies: []
related: [calibrate-the-physical-frontier-provider-and-outcome-budgets, replace-provider-offer-with-a-host-bounded-frontier-sink]
scopes: [research/program-planning, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, budgets, measurement, public-boundary]
---
## User-visible outcome

The physical provider-count limit has an accepted value enforced at the authority that can actually refuse it, instead of inheriting a number from a superseded single-target table that never passed a decision gate.

## Why this exists

Split out 2026-08-22 by the coordinator when `calibrate-the-physical-frontier-provider-and-outcome-budgets` **fired its own stop condition**: *"provider count belongs to a different preflight authority than the complete budget policy."* The worker stopped rather than pushing through, which was correct. That ticket now owns the raw-outcome axis only.

**Fact — the two axes have different enforcement points, and only one has an accepted value.** The raw-outcome bound is a per-request `DeterministicBudgets` field. A provider-count bound could only refuse at `InstalledPhysicalProviders::installed`, which runs **before** compile and today branches only on governed identity and duplicates — it has no count branch at all. A budget field cannot enforce a limit at a seam it never reaches.

**Fact — the `32` is superseded, not accepted.** It comes from the single-target table that ticket retains as history rather than as a live recommendation. The 2026-08-18 acceptance names only the request-scoped raw-outcome value; its decision gate enumerates raw-outcome powers and no provider count. Verify this by reading that ticket's `## Accepted policy — 2026-08-18` section in full before relying on any number.

**Reported by the calibration lane, unverified by the coordinator:** `installed()` has no provider-count refusal as a type bound, and the durable finite witness installs 129 identities — but that 129 lives in the **spike harness**, not in `crates/`. Re-derive both at your base; the lane flagged its own production-impl census as textual rather than type-enumerated, closing the gap at that base but not by construction.

## Required work

- Re-audit every Fact above at your base and report a per-Fact verdict before proposing a value.
- Decide **by reading** whether a provider-count limit belongs at the installation seam at all, or whether the raw-outcome ceiling already bounds what matters. **A limit with no honest enforcement point should be eliminated, not relocated** — that conclusion is a valid and possibly correct outcome of this ticket.
- If a limit is warranted, derive it from a measured population rather than reusing `32`, and say what it refuses that the raw-outcome ceiling does not.
- Size the production-impl population **from the type where possible** rather than textually; state which unit you report and anchor every pattern.
- Whatever you conclude, perturb the subject and quote the failure text. If you add a refusal, show it firing on a population that should be refused and *not* firing on one that should not.

## Non-goals

Changing the accepted request-scoped raw-outcome value; merging or rebasing the preserved draft `54e272ba`, which its own ticket records as unmergeable; and any wall-clock claim taken on a loaded coordination host.

## Closes when

The provider-count axis has either an accepted limit enforced where it can actually refuse, or a recorded decision that it should not exist, with the reasoning and a reconsideration trigger in both cases.
