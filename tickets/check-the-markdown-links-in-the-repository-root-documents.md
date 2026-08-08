---
id: check-the-markdown-links-in-the-repository-root-documents
title: Check the markdown links in the repository root documents
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786169827
---
## The gap, as it stood before this ticket landed

`check-citations.sh` reads three populations: the built-in fixture, `tickets/**`, and `docs/**`. The tracked markdown at the repository **root** is in none of them, so its links are unresolved:

| file | local links |
|---|---|
| `AGENTS.md` | 1 |
| `README.md` | 6 |
| `CLAUDE.md` | 0 |

Measured 2026-08-08: all seven resolve today, so this is an unfloored gap rather than a live defect. It was found while demonstrating the new link failures under `resolve-the-markdown-links-the-citation-check-cannot-see` — a dangling link planted in `AGENTS.md` produced **no failure at all**, because the file is never read.

## Why it is worth closing anyway

These are the entry points. `README.md` links to `docs/README.md`, `docs/status.md`, `docs/research/README.md`, `spikes/README.md`, `AGENTS.md`, and `docs/work-tracking.md` — the whole first hop of the corpus. `AGENTS.md` links to `docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md`, the ADR that governs every unsafe site in the workspace. A reader who arrives at this repository reads exactly these files first, and they are the only markdown the gate cannot see.

## Why it was not done in the same change

The parent ticket bounded the work to "the populations the script already reads" and said explicitly to follow the existing tickets/docs shape "rather than inventing a third". Root documents genuinely are a third population and need their own branch in `role_of()`, because the fall-through there calls them tickets and the ticket rule in `decide()` reports "no status in frontmatter; cannot decide whether to check it" and fails them. The header of `check-citations.sh` documents what adding a population costs — an append, a `role_of()` branch, an answer in `decide()`, a census line, and a floor.

**Correction, 2026-08-08 at `0f319ec8` — the stated reason was partly false, and the `decide()` half of it was wrong.** Two claims above were repaired in place; the retired wording was "they carry no frontmatter at all" and "their own answer in `decide()`".

- **False: "they carry no frontmatter at all."** `README.md` opens with `schema: "tiler-doc/v1"` / `kind: "portal"` frontmatter and is a first-class Tiler document; it simply carries no *status facet*, which the kind table requires of no portal. Only `AGENTS.md` and `CLAUDE.md` have no frontmatter. The conclusion — that all three fail the fall-through — survives, but by two different roads: `README.md` has frontmatter with no `status:` key, so `status` stays empty and the ticket branch fails it; the other two never enter the frontmatter reader at all.
- **False: they need "their own answer in `decide()`."** The `doc` branch already answers exactly this question — `if (!doc_status_seen) ...; files_live[role]++`, reached by 24 files under `docs/` on every run — and root documents now share that branch rather than duplicating it. What was genuinely missing was only the `role_of()` classification.

## Closes when

The three root documents are in the checked population with their own census line and floor, a planted dangling link in `AGENTS.md` has been watched failing, and the script header states how their status is decided given that they have none.

## Coordinator verification, 2026-08-08 at `2aa69bfe`

**Fact — the gap is real and demonstrated, not inferred.** Appending `[planted](docs/decisions/9999-no-such-adr.md)` to `AGENTS.md` and running `./check-citations.sh` leaves it at **exit 0**. The root documents are in no population, so no planted defect there can turn the gate red. Restored immediately; `git diff --stat AGENTS.md` empty after.

**Fact — the gap is latent, not active.** The three root documents carry **7** local links between them — `AGENTS.md` 1, `README.md` 6, `CLAUDE.md` 0 — and every one resolves against the index today. So this ticket adds coverage; it does not repair a break. Size the work accordingly and do not expect to find defects.

**Inference — why it still matters at p1 despite finding nothing.** `AGENTS.md` is the document every agent and reviewer is told to read first, and `docs/README.md` and `docs/decisions/README.md` are the entry points it routes readers through. A dangling link there misroutes the reader who is being onboarded, which is the worst-placed reader to misroute. The population is tiny, so the coverage is cheap.

**The known obstacle, from the worker that filed this.** This paragraph opened by asserting that root files carry no frontmatter, so the existing `decide()` path would fail them rather than classify them. Both halves are false and the dated correction above gives the evidence: `README.md` carries `tiler-doc/v1` frontmatter, and it is the `role_of()` fall-through rather than `decide()` that fails them. A third population needs a stated rule for documents that have no status facet at all. Note `docs` already reports `24 carrying no status facet`, so a precedent for status-free live documents exists in the current run; check whether that path can be reused before inventing a third one. **It can, and it was.**

**Make the new check fail deliberately before trusting it**, using the plant above, and quote the failure text. A population that silently reads zero root files would look identical to a green one — assert the file count as a floor, and print the census.

## Worker outcome, 2026-08-08 at `0f319ec8`

**Fact — the coordinator's Facts re-verified at this base.** The three-population reading, the 1/6/0 link table, the seven-of-seven resolution, the exit-0 plant, and `24 carrying no status facet` all hold as stated. The only defect found was the frontmatter premise repaired above.

**Fact — what landed in `check-citations.sh`.** A `root` role covering every tracked markdown file with no directory component, classified by shape (`if (path !~ /\//)`) rather than by the three names, so a document added beside them is covered the day it lands. It shares `decide()`'s `doc` branch whole — including the `superseded` skip — and diverges only in its counters, which are keyed by role. `docs_no_status` became `no_status[role]` so the `docs` line keeps reporting 24 and cannot absorb the root count.

**Measurement — the census, at this base.** `root  0 citation(s) from 3 live file(s) of 3 read against a floor of 3, 0 skipped as superseded, 3 carrying no status facet` and `root  7 link(s)`. Tree-wide totals moved from 5949 to 5956 links and from 3311 to 3321 bare path mentions; citations stayed at 948 and `docs` stayed at 24.

**Fact — why the root floor is a file count and not a citation count.** This population carries **zero** pinned citations: every path in the three files is a bare mention with no line and no anchor, which the bare-path rule deliberately declines to check. A citation floor here would fail on a correct tree, so the floor is `files_read["root"] >= 3` plus a link floor, and the header says so.

**Measurement — three perturbations of the subject, each fired.**

1. `[planted](docs/decisions/9999-no-such-adr.md)` appended to each root file in turn: all three fail, exit 1, `FAIL  CLAUDE.md / link: [...](docs/decisions/9999-no-such-adr.md) / no tracked file or directory at docs/decisions/9999-no-such-adr.md`. Planting in `CLAUDE.md` is the load-bearing one — it contributes no link and no citation today, so only a plant proves it is read at all.
2. `mv CLAUDE.md CLAUDE.markdown` shrinks the glob: `SHORT  the repository-root population reached 2 file(s), below its floor of 3.`, exit 1, while the link floor stayed green at 7. The two floors are independent.
3. Breaking the seven `](` into `] (` leaves all 3 files read and 0 links resolved: `EMPTY  the repository-root markdown link population contributed 0 checked link(s), so nothing in it was verified.`, exit 1, while the file floor stayed green at 3.

**What it would take for this check to say no**, and each case is reachable: a dangling link in a root document (1); the glob or `role_of()` ceasing to reach them (2); `scan_links` ceasing to walk their prose (3). A broken pinned citation would also fail, by the same path every other population uses.
