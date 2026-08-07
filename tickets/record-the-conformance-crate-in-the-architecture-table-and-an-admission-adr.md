---
id: record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr
title: Record the conformance crate in the architecture table and an admission ADR
status: in-progress
priority: p2
dependencies: []
related: [admit-the-conformance-crate-to-the-workspace]
scopes: [contracts/foundation, contracts/decisions]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [docs, architecture]
claimed_from: todo
assignee: agent-crate-record
lease_expires_at: 1786121747
---
## What this owes

Two records the crate admission could not write from its own scopes, kept together because they state the same fact to two audiences.

**1. `docs/architecture.md`'s component-ownership table is stale.** It enumerates every crate row by row and `tiler-conformance` is missing. The row should say what the crate owns — cross-layer executed conformance evidence — and what it deliberately is not: not a second semantic authority (`tiler-reference` remains the oracle), not a benchmark harness, and not a home for layer-local tests. It is also the one member **nothing depends on and nothing may**, which is the inverse of the facade's position and worth stating in the table rather than only in the crate header.

**2. There is no admission ADR, and every prior crate admission has one.** ADR 0077 admitted `tiler-metal-aot`, 0081 `tiler-runtime`, 0082 `tiler-cache`, 0085 `tiler-build`, and 0088 the frontend pair; `docs/architecture.md:314` names those records as the reason those rows postdate ADR 0070. ADR 0075 classifies a new crate as a publicly reachable namespace, so this admission is the same category as those. **Tom's acceptance is currently recorded in a ticket rather than in `docs/decisions/`**, which is exactly the asymmetry [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md) exists because of — an accepted decision with no record in the decisions corpus.

## What the ADR must carry, and it is already derived

Do not re-derive any of this; it is on [`decide-where-a-device-reaching-conformance-test-may-live`](decide-where-a-device-reaching-conformance-test-may-live.md) and should be transferred with its provenance:

- The eliminations, each from the code rather than from preference: `prototypes/serial-sum-run` rejected because prototypes are throwaway and long-term holding evidence must not live there; `crates/tiler` ruled out by `dependency_direction.rs`, which forbids the facade a `tiler-metal-aot` edge and reads `Cargo.lock`, so even a dev-dependency trips it; `crates/tiler-runtime` ruled out by its own stated boundary that its tests must not reach `tiler-compiler`; `crates/tiler-build` rejected because it would put the consume half of an end-to-end test inside the produce crate.
- The missing-component evidence: five open conformance tickets, no two sharing a scope set, with oracle plumbing living inside `tiler-compiler` because there was nowhere else.
- The three anti-goals, which are the crate's actual boundary.
- The acceptance provenance: Tom, 2026-08-07, coordination session, witnessed first-hand by the coordinator.

**The ADR must not claim more than the crate does.** The crate was admitted as a smallest useful slice and holds no items at all; a record describing an evidence authority that derives support-matrix rows would be describing [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)'s subject as though it were decided. State the admitted scope and cite the survey for the rest.

## Closes when

The architecture table carries the row with its anti-goals and its nothing-depends-on-it position, the ADR exists with its number allocated and its status and acceptance provenance recorded, `docs/decisions/README.md`'s index carries it, and `docs/architecture.md`'s reference to the admission ADRs includes it.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration. Not blocking the crate's first content: the member exists and is gated, and holding evidence work behind a documentation sweep would invert the order the admission was scoped for.
