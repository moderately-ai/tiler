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
