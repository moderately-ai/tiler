---
id: repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row
title: Repair the fifth mistyped supports edge and its missing experiment-catalog row
status: done
priority: p3
dependencies: []
related: [repair-the-four-mistyped-typed-frontmatter-edges, reconcile-the-research-and-experiment-catalogs-with-their-frontmatter, redesign-the-delivered-realization-record-from-typed-evidence, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [docs, navigation, catalog, metadata]
---
## User-visible outcome

The experiment catalog renders a row for every governed experiment record, and the one record it renders no row for stops being invisible to a reader browsing `spikes/README.md`.

## Why this exists

**Measurement — found on 2026-08-05 while landing the operation-family delivery graph, by running the check [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md) already owns.** Over a named population of 37 experiment records against 36 rendered rows, it reports exactly one discrepancy: `MISSING experiment rows for ['tiler.spike.numerics.delivered-realization-record']`. The same run reports zero discrepancies on the research side over 84 rows against 84 records, so the check reached both halves.

**Fact — the discrepancy pre-dates that landing and is reproducible at its base.** Re-running the experiment half against the tree at `b63dd5d0` reports the same single missing id, so it is not a consequence of the delivery-graph change; that change added a research record and a research row, and touched no experiment record and no experiment row.

**Fact — the missing row is the second half of a mistyped edge, not an omission on its own.** `spikes/numerics/delivered-realization-record/README.md` declares `supports: ["tiler.contract.artifact-abi", "tiler.contract.numerical-semantics"]`. [`docs/document-metadata.md`](../docs/document-metadata.md) types `supports` as **experiment to research**, so both targets are the wrong kind. This is a fifth instance of exactly the class [`repair-the-four-mistyped-typed-frontmatter-edges`](repair-the-four-mistyped-typed-frontmatter-edges.md) enumerates — its own check encodes the rule as `rules.append(("supports", ("research",)))` — and it appeared after that ticket's table was written, which is why it is filed rather than folded in silently.

**Inference — the two halves must be repaired together and in that order.** Rendering a row now would render a `supports:` clause derived from two contract ids, which the catalog's own derivation cannot express; the edge has to point at the research record that states the bounded universe and procedure behind the packet's claims first, and the row then renders from it. Rendering the row first would bake the mistyped edge into the view that is supposed to check it.

## Required work

- Decide the correct `supports` target by reading what the packet actually establishes, not by picking the nearest research record. If no existing research record states the packet's boundary, that absence is the finding and the repair is a research record rather than a re-pointed edge.
- Repoint the edge, then add the `spikes/README.md` row in the **same** change, since the catalog is a hand-maintained derived view with no generator.
- Re-run the reconciliation check and report the count, which must reach zero for this id, with its failing perturbation watched — drop the row again and see it reported.

## Outcome — the edge is repaired; the catalog half is split out

**Fact — the first Required-work bullet resolved to its second branch: no existing research record states the packet's boundary, so the repair is a research record.** The survey was over every record carrying `kind: "research"` under `docs/research/**` — 91 at this base — filtered to those whose `informs` names either contract the mistyped edge pointed at, then read rather than matched. `tiler.research.artifacts.target-neutral-envelope` states the artifact container's framing, digests, sections, and payload boundaries and says nothing about the numerical realization record; `tiler.research.program-planning.abi-expression-ownership` states where ABI expressions live. ADR 0076 owns item 4 and its own `evidence` names four records — Apple GPU numerical behaviour, the operation conformance matrix, the physical feasibility model, and Apple artifact compatibility — every one of which states a measured-target or conformance boundary, while the packet is explicit that its two-dtype fixture is checked synthetic and "proves a property of the *record*, not of any measured target". No record stated the packet's universe, so none could be pointed at without the edge asserting a boundary its target does not carry.

**Fact — the type rule that grounds the repair, and the tell that identifies it.** [`docs/document-metadata.md`](../docs/document-metadata.md) types `supports` as experiment to research, and its section *A decision does not cite an experiment in metadata* records why the analogous relaxation was refused: a document able to name a harness directly "is never pushed to say what bounded universe, environment, and procedure make the measurement carry the weight put on it — naming that boundary being the research record's job." The two contract ids the broken edge named are exactly the two the new record's `informs` now carries, which is the tell — the author meant "this packet backs these two contracts", and the typed route for that is experiment → research → contract, with the research record supplying the boundary the direct edge skipped.

**Fact — what landed.** [`docs/research/numerics/delivered-numerical-realization-record.md`](../docs/research/numerics/delivered-numerical-realization-record.md) states the boundary: `executable-model` and `exhaustive-finite` over three explicitly named finite universes — the eleven governed dimensions, the twenty-five rule identifiers, the thirty judged `(type, arithmetic)` subject pairs — with the synthetic-evidence limit and the untrusted-producer limit both recorded, because a reader who mistakes decode success for authenticity draws the opposite of the right conclusion. `spikes/numerics/delivered-realization-record/README.md` now reads `supports: ["tiler.research.numerics.delivered-numerical-realization-record"]`. `spikes/numerics/README.md` already listed the packet and needed no edit.

**Measurement — re-run of the spike at `fac629a7`, 2026-08-05, macOS 27.0.0, from `spikes/numerics/delivered-realization-record` with `CARGO_TARGET_DIR=./target cargo run`.** All ten stages pass, exit zero, 38 perturbations tripping 25 distinct rules. The record cites this run rather than transcribing the packet's own claims.

**Measurement — the typed-edge check from [`repair-the-four-mistyped-typed-frontmatter-edges`](repair-the-four-mistyped-typed-frontmatter-edges.md), over 264 governed documents and 455 typed edges, goes from `MISTYPED: 6` to `MISTYPED: 4`.** The two edges that disappear are this record's, and the four that remain are exactly the four that ticket owns. Watched failing: reverting the edge to its two contract targets returns `MISTYPED: 6` with both spike lines named, so the check is measuring this repair and not a coincidence.

**Measurement — the reconciliation check goes from `DISCREPANCIES: 1` to `DISCREPANCIES: 2`, and that is the recorded remainder rather than a regression.** At base it reported one survivor, `MISSING experiment rows for ['tiler.spike.numerics.delivered-realization-record']`, over 91 research rows against 91 records and 40 experiment rows against 41 records — the last one standing, as expected. After the repair it reports that same missing experiment row plus `MISSING rows for ['tiler.research.numerics.delivered-numerical-realization-record']`, because the new research record needs a research-catalog row of its own. Both are rows in `docs/research/README.md` and `spikes/README.md`, and **both files map to `contracts/navigation`** — read from `ticketsplease.toml`, not asserted.

**Fact — the catalog half was not landed because file-level disjointness could not be established, and the honest verdict is vacuous rather than clear.** `contracts/navigation` is held live by `admit-bf16-into-the-schedule-and-kernel-vocabulary` (assignee `agent-bf16-vocab`), whose branch-side ticket copy declares it. `git diff --name-only main...tkt/admit-bf16-into-the-schedule-and-kernel-vocabulary` is empty and `git rev-list --count main..tkt/admit-bf16-into-the-schedule-and-kernel-vocabulary` is `0`, so the branch has no commits and evidences nothing about what it will touch. An empty diff is not disjointness evidence, so the two catalog files stayed untouched.

**Measurement — the owed rows are nonetheless proved, not guessed.** Both rows were inserted as a dry run and the reconciliation check re-run: `population: 92 research rows, 92 research records`, `41 experiment rows against 41 records`, `DISCREPANCIES: 0`. The rows were then reverted (`git checkout -- docs/research/README.md spikes/README.md`, both clean afterward) and the check re-run to watch it report the drop, returning to `DISCREPANCIES: 2` naming both ids. [`render-the-delivered-realization-catalog-rows`](render-the-delivered-realization-catalog-rows.md) carries the exact row text from the run that reached zero and is the bounded remainder; this ticket's revised outcome — the edge repaired and the boundary recorded — is supported.

A third check was watched failing as well: the link check over the new record's 13 local targets reports `DEAD LINKS: 0`, and a single perturbed target reports it dead, so a green result distinguishes "no dead links" from "the check did not run".

## Explicit non-goals

- Widening `supports` to admit a contract target. The metadata contract records the measured reason it declined that relaxation, and reversing it is an ADR-level decision rather than a repair — the four-edge ticket says so in its own non-goals and this ticket inherits it.
- The other four edges, which that ticket owns. This is a fifth, filed separately because it was found separately and because its catalog half is a different symptom.

## Closes when

**Revised on completion, because the repair turned out to require a research record and the catalog scope was unavailable.** The edge points at a `research` target that states the packet's bounded universe, the typed-edge check reports `MISTYPED: 4` over its named population with the two edges this ticket owned gone and its failing perturbation watched, and the two catalog rows are a live ticket carrying the exact text a dry run proved reaches `DISCREPANCIES: 0`. The original wording also required the experiment catalog to render the row; that half is [`render-the-delivered-realization-catalog-rows`](render-the-delivered-realization-catalog-rows.md)'s, and holding this ticket open for it would block nothing and deadlock a dependent.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md), which ran the reconciliation check as one of its own required checks and found this survivor. That ticket held `contracts/navigation` and could have added the row; it did not, because adding a row over a mistyped edge is the repair that hides the defect.
- `research/numerics` is declared because the spike record carrying the edge lives under `spikes/numerics/**`, which that scope maps. It also maps `docs/research/numerics/**`, so the new research record this repair turned out to require needed no scope addition. `contracts/navigation` was required for the catalog rows and is deliberately **not** declared here: this ticket touches neither file, and declaring a scope it does not edit would false-conflict against the live claim holding it. [`render-the-delivered-realization-catalog-rows`](render-the-delivered-realization-catalog-rows.md) declares it instead.
- The remainder is filed rather than absorbed, and the parent's outcome is revised rather than overstated: the edge points at a `research` target and the boundary it points at now exists, which is what this ticket can support. The two catalog rows are that ticket's, with their exact text and the run that proved them.
