---
id: make-the-research-catalog-generated-or-stop-claiming-it-is
title: Make the research catalog generated or stop claiming it is
status: done
priority: p2
dependencies: []
related: [catalog-the-four-cost-model-reading-notes-and-correct-the-stale-token-out-key-count]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The marker is false, and it is the root cause rather than a cosmetic defect

`docs/research/README.md` delimits its catalog with `<!-- BEGIN GENERATED RESEARCH CATALOG -->`. **No generator exists anywhere in the tree** — coordinator-verified: the marker string appears in that one file and in no `.py`, `.sh`, `.rs`, or `Makefile`. The only other occurrence in the repository is a one-off Python snippet quoted inside an unrelated ticket, which produces nothing.

**Why this is causal and will recur.** A reader who sees `GENERATED` reasonably concludes a tool keeps the block in sync and that hand-editing it is pointless or will be overwritten. That belief is exactly why four `kind: research` records landed uncatalogued and stayed invisible until a worker went looking — and `AGENTS.md` states plainly that this documentation is **manually maintained**, with links, frontmatter, supersession, entry points and catalogs having no automated validator. The marker contradicts the working contract in the file the contract is about.

Found while catalogueing those four notes; reported rather than absorbed, because fixing the four rows leaves the cause in place.

## The decision, which is genuinely two-sided

**Write the generator.** *Enables:* the marker becomes true, and a record landing without a row becomes a mechanical failure instead of an invisible one — closing the same class the citation checker closes for tickets. *Costs:* a second authority over a file several lanes edit, plus every hazard this repository keeps hitting — it must name and count its population so a run that finds zero records **fails**, it must be multi-line aware, and it must be watched failing before it is trusted. It also has to decide what happens to rows a human wrote that the generator would not produce.

**Remove the marker.** *Enables:* the file stops lying, immediately and at near-zero cost, and readers are told the truth that keeping it current is a human obligation — which `AGENTS.md` already asserts. *Costs:* nothing detects the next uncatalogued record. The failure that produced this ticket stays possible.

**Recommendation: remove the marker now, and treat the generator as a separate, larger decision.** The marker is actively harmful today because it misdirects readers; the generator is a real piece of tooling with its own failure modes and should not be smuggled in under a comment fix. Removing it is reversible in one line and makes the current obligation honest. **The counterpoint is real:** removing it closes nothing, and the class of defect that produced this ticket recurs the next time a record lands — so if the generator is wanted, filing it immediately rather than "later" is what stops this being a documentation-only fix.

## Requirements if the generator is written

- **Name and count the population; fail an empty run.** A generator that emits zero rows and reports success is the failure this repository keeps finding.
- **Watch it fail** by removing a record's row and confirming the check reddens — perturbing the subject, not an assertion.
- Decide and state what governs rows a human authored, including the `primary documents:` clause that no frontmatter field produces and that a naive generator would silently delete. That clause is load-bearing: a ticket once prescribed a row form omitting it, and following that prescription would have left a source record unreachable.

## Closes when

Either the marker is gone and the file states the obligation as manual, or a generator exists that produces the block, fails on an empty population, has been watched failing, and preserves the non-frontmatter clauses the row form carries.

## Outcome — done, 2026-08-07

Landed at merge `e416dc82` (worker commit `fcec068b`). `docs/` + `spikes/README.md` + `tickets/` only, carries the green gate.

### A generator did exist, and the markers are fossils of a deliberate deletion

`scripts/docs.py` — *"Validate Tiler's documentation graph and render checked-in catalogs"* — carried a `MARKERS` table for all three catalogs and was deleted at **`e197176f`** ("Replace the Python gate with a Makefile of cargo commands"), coordinator-verified. So the markers are not an aspiration never built; they are residue of a decision already taken.

### This ticket's framing was wrong, and the worker was right to refuse it

I called it "a genuinely two-sided decision". It is not a worker-available trade-off, for three verified reasons:

1. **A merged contract already forbids the claim.** `docs/document-metadata.md`: *"There is no validator and no renderer… maintained by hand and checked by reading. **No sentence anywhere in this document promises otherwise; if one appears to, it is the defect.**"* Coordinator-verified. Three portals were carrying exactly that defect.
2. **A `done` ticket records the absence as deliberate** — `reconcile-the-research-and-experiment-catalogs-with-their-frontmatter` names "no generator, no gate, and no schema change" as an explicit non-goal.
3. Writing one would therefore **reverse a recorded deletion and contradict a merged contract**, which is Tom's under `AGENTS.md`, not a cost/benefit call.

**And it is not cheap.** The block is not currently a function of frontmatter: eight rows carry `primary documents:` with **no frontmatter source**, two carry free-prose supersession qualifiers, and `adopted_by` renders two different ways. A generator's first run would *rewrite* the file rather than confirm it, and would need frontmatter backfill across four-plus research scopes. That is a docs-wide metadata redesign.

### Scope the worker added, flagged rather than absorbed

The ticket named **one** marker. There were **four pairs across three files** — research catalog, experiment catalog, ADR topics, ADR chronology — all from the same deleted script, all in `contracts/navigation`. Coordinator-confirmed at the base: 2 markers in `docs/research/README.md`, 4 in `docs/decisions/README.md`, 2 in `spikes/README.md`. Fixing only the named one would have left three identical lies, which is the error this ticket exists to prevent.

It also removed an empty `### Documentation governance` heading from the experiment catalog, having verified in the **deleted script's own source** that it emitted every heading from a fixed `GROUPS` table unconditionally, and that no experiment record carries `catalog_group` at all.

Delimiters were **renamed, not deleted** — they bound the block for the by-hand reconciliation check `docs/document-metadata.md` advertises, and deleting them outright would have broken a check the contract points readers at. My third stated Fact was wrong on all three clauses: there were three other occurrences rather than one, the snippet is not in an "unrelated" ticket but is one of the contract's advertised checks, and it does not "produce nothing".

### Two things left standing, deliberately

**A 107th record is uncatalogued right now** — `docs/research/verification/kani-bounded-encoder-verification.md` and its spike, owned by `catalog-the-kani-verification-research-and-spike` (`todo`, unclaimed). Live proof the defect class is still open: nothing detects an uncatalogued record, and closing that gap means reversing `e197176f` and amending a merged contract. The worker deliberately did not file a ticket for it, because filing one would presume the answer. That judgement is correct and the question is recorded here instead.

The reconcile check was **watched failing**: deleting a row gives `105 rows, 107 records, REDDENED`, and the stale split literal gives a loud `IndexError` rather than a green run over zero rows.
