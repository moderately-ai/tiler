---
id: record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr
title: Record the conformance crate in the architecture table and an admission ADR
status: done
priority: p2
dependencies: []
related: [admit-the-conformance-crate-to-the-workspace]
scopes: [contracts/foundation, contracts/decisions]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [docs, architecture]
---
## What this owes

Two records the crate admission could not write from its own scopes, kept together because they state the same fact to two audiences.

**1. `docs/architecture.md`'s component-ownership table is stale.** It enumerates every crate row by row and `tiler-conformance` is missing. The row should say what the crate owns — cross-layer executed conformance evidence — and what it deliberately is not: not a second semantic authority (`tiler-reference` remains the oracle), not a benchmark harness, and not a home for layer-local tests. It is also the one member **nothing depends on and nothing may**, which is the inverse of the facade's position and worth stating in the table rather than only in the crate header.

**2. There is no admission ADR, and every prior crate admission has one.** ADR 0077 admitted `tiler-metal-aot`, 0081 `tiler-runtime`, 0082 `tiler-cache`, 0085 `tiler-build`, and 0088 the frontend pair; `docs/architecture.md:314` names those records as the reason those rows postdate ADR 0070. ADR 0075 classifies a new crate as a publicly reachable namespace, so this admission is the same category as those. **Tom's acceptance is currently recorded in a ticket rather than in `docs/decisions/`**, which is exactly the asymmetry [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md) exists because of — an accepted decision with no record in the decisions corpus.

## What the ADR must carry, and it is already derived

Do not re-derive any of this; it is on [`decide-where-a-device-reaching-conformance-test-may-live`](decide-where-a-device-reaching-conformance-test-may-live.md) and should be transferred with its provenance:

- The eliminations, each from the code rather than from preference: `prototypes/serial-sum-run` rejected because prototypes are throwaway and long-term holding evidence must not live there; `crates/tiler` ruled out by `dependency_direction.rs`, which forbids the facade a `tiler-metal-aot` edge and reads `Cargo.lock`, so even a dev-dependency trips it; `crates/tiler-runtime` ruled out by its own stated boundary that its tests must not reach `tiler-compiler`; `crates/tiler-build` rejected because it would put the consume half of an end-to-end test inside the produce crate.
- The missing-component evidence: five conformance tickets spanning six scopes with none common to all, three of them carrying identical sets about one compiler-resident file, with oracle plumbing living inside `tiler-compiler` because there was nowhere else.
- The three anti-goals, which are the crate's actual boundary.
- The acceptance provenance: Tom, 2026-08-07, coordination session, witnessed first-hand by the coordinator.

**Correction — 2026-08-07, on the second bullet.** It read *"five open conformance tickets, no two sharing a scope set"*, which the tickets refute: three of the five carry identical scope sets, all about one compiler-resident file. It is **substituted rather than dated beside**, because the clause was never true at any commit — the shape [`correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence`](correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence.md) used on the ADR this ticket asked for. Reproduced at `3e0074d5` with `tkt show <id>` on each of the five. The full per-ticket attribution and the pinned population counts are on [`decide-where-a-device-reaching-conformance-test-may-live`](decide-where-a-device-reaching-conformance-test-may-live.md), which carried the claim first; "five" is the slice the decision was taken under and not the population, and "open" is stale since `conform-the-bf16-vertical-end-to-end` is `done`.

**The ADR must not claim more than the crate does.** The crate was admitted as a smallest useful slice and holds no items at all; a record describing an evidence authority that derives support-matrix rows would be describing [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)'s subject as though it were decided. State the admitted scope and cite the survey for the rest.

**Superseded — 2026-08-07, later the same day, on "holds no items at all" and not on the constraint.** The crate gained the BF16 vertical at `b7c01815` and the migrated device-executed value proof at `0f948637`; at `3e0074d5` it holds thirteen source files and dispatches on a device. The sentence is dated rather than substituted because it was true when this brief was written — the opposite treatment from the scope-set clause above, which never was. The constraint it states is unaffected and still binds the record: deriving support-matrix rows remains the survey's subject, and ADR 0106 still decides only that the crate exists and what it owns.

## Closes when

The architecture table carries the row with its anti-goals and its nothing-depends-on-it position, the ADR exists with its number allocated and its status and acceptance provenance recorded, `docs/decisions/README.md`'s index carries it, and `docs/architecture.md`'s reference to the admission ADRs includes it.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration. Not blocking the crate's first content: the member exists and is gated, and holding evidence work behind a documentation sweep would invert the order the admission was scoped for.

## Outcome — delivered 2026-08-07 at `c466cc11`

**ADR 0106** — `docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md`. The number was verified free across every local branch, not just against `docs/decisions/`, so no in-flight allocation collides.

**The ADR claims no more than the crate holds**, which was the constraint that mattered. Item 5 lists what the admission does *not* decide, each pointing at its owner: no public surface (ADR 0075), no migration and no support-matrix authority (both the survey, named as open with both answers available), no lint relaxation. A Consequences bullet says outright that reading the record as evidence a cross-layer run exists would be reading a boundary as a result.

**One place the worker had to reason rather than transfer, and got it right.** ADR 0056's withheld-crate clause names five crates and `tiler-conformance` is none of them, so unlike ADR 0082/0088 (amend) and ADR 0081 (apply ADR 0077's stated test) it needs **neither move** — the clause is untouched. It explicitly refuses the tempting reading that the crate passes ADR 0077's no-device test: it creates no device object today only because it contains no code, and its whole purpose is a run that executes on one. What keeps it outside the clause afterwards is that it is not *reusable* — nothing may depend on it — so it publishes no runtime boundary. The non-precedent warning is stated twice.

### Five pre-existing stale claims corrected, found by reading in full

None was in the brief; all were found because `docs/architecture.md` was read whole rather than patched at the insertion point. This is the full-read discipline earning its cost.

- `tiler-digest` was missing from the admission-record enumeration whose closing sentence claims to name every unadmitted row, making the enumeration false.
- The dependency block read `tiler-ir -> []` and `tiler-artifact -> [tiler-ir]`; both were wrong since ADR 0104 — the real edges are `[tiler-digest]` and `[tiler-digest, tiler-ir]`, and `tiler-digest -> []` was absent entirely.
- The section **contradicted itself**: "Nothing pins the member set or any package's dependency list" sat two paragraphs from its own citation of `workspace_population.rs`, which pins exactly the member set. Corrected to distinguish the two properties.
- "two rows carry third-party edges it does not show" → several, naming `tiler-digest`'s hash dependency, since a new `-> []` row was added whose closure is not empty.
- "None of those **five** records" listed six.

Also corrected: `prototypes/serial-sum-run` as "the one member that talks to [a device]" → the one member whose *code* reaches one, since `tiler-conformance` now declares the same macOS-gated binding with nothing behind it — which is why the live-execution grep recorded earlier in the document still returns no file under `crates/`.

**Superseded — 2026-08-07, later the same day, on the sentence above.** It was true at `c466cc11`, which this Outcome is pinned to, and stopped being true at `0f948637`: the grep now returns `crates/tiler-conformance/src/dispatch.rs` and `src/device_buffer.rs`, and the prototype is no longer the one member whose code reaches a device. Repaired in the live document by [`refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew`](refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew.md), which edited `docs/architecture.md` and dated ADR 0106 rather than editing it.

**Catalogs verified complete rather than assumed.** `docs/decisions/README.md`'s two generated blocks both updated, with a grep confirming no generator exists outside the README. Every file mentioning ADRs 0105, 0104, or 0088 was checked; outside that README every hit is substantive prose, not a catalog row. `docs/design-map.md`, `docs/README.md`, `docs/status.md` and the root README carry no roster this falsifies.

**Delta rule applied and stated.** The complete file list is `docs/architecture.md`, the new ADR, and `docs/decisions/README.md` — touching none of `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh` — so it carries the latest green gate, with `tkt lint` rerun as the rule requires. Confirmed by the coordinator against the merge's own file list.
