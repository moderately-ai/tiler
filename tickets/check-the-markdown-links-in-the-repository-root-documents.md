---
id: check-the-markdown-links-in-the-repository-root-documents
title: Check the markdown links in the repository root documents
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: []
paths: []
tags: []
---
## The gap

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

The parent ticket bounded the work to "the populations the script already reads" and said explicitly to follow the existing tickets/docs shape "rather than inventing a third". Root documents genuinely are a third population and need their own branch in `role_of()` and their own answer in `decide()`: they carry no frontmatter at all, so the current fall-through would report `no status in frontmatter; cannot decide whether to check it` and fail. The header of `check-citations.sh` documents what adding a population costs — an append, a `role_of()` branch, a `decide()` branch, a census line, and a floor.

## Closes when

The three root documents are in the checked population with their own census line and floor, a planted dangling link in `AGENTS.md` has been watched failing, and the script header states how their status is decided given that they have none.

## Coordinator verification, 2026-08-08 at `2aa69bfe`

**Fact — the gap is real and demonstrated, not inferred.** Appending `[planted](docs/decisions/9999-no-such-adr.md)` to `AGENTS.md` and running `./check-citations.sh` leaves it at **exit 0**. The root documents are in no population, so no planted defect there can turn the gate red. Restored immediately; `git diff --stat AGENTS.md` empty after.

**Fact — the gap is latent, not active.** The three root documents carry **7** local links between them — `AGENTS.md` 1, `README.md` 6, `CLAUDE.md` 0 — and every one resolves against the index today. So this ticket adds coverage; it does not repair a break. Size the work accordingly and do not expect to find defects.

**Inference — why it still matters at p1 despite finding nothing.** `AGENTS.md` is the document every agent and reviewer is told to read first, and `docs/README.md` and `docs/decisions/README.md` are the entry points it routes readers through. A dangling link there misroutes the reader who is being onboarded, which is the worst-placed reader to misroute. The population is tiny, so the coverage is cheap.

**The known obstacle, from the worker that filed this.** Root files carry **no frontmatter**, so the existing `decide()` path would fail them rather than classify them — that is why they were excluded rather than overlooked. A third population needs a stated rule for documents that have no status facet at all. Note `docs` already reports `24 carrying no status facet`, so a precedent for status-free live documents exists in the current run; check whether that path can be reused before inventing a third one.

**Make the new check fail deliberately before trusting it**, using the plant above, and quote the failure text. A population that silently reads zero root files would look identical to a green one — assert the file count as a floor, and print the census.
