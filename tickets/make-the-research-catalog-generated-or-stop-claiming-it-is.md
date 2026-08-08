---
id: make-the-research-catalog-generated-or-stop-claiming-it-is
title: Make the research catalog generated or stop claiming it is
status: todo
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
