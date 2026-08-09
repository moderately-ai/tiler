---
id: decide-whether-heading-anchors-in-markdown-links-can-be-resolved
title: Resolve heading anchors when a Markdown renderer is pinned
status: deferred
priority: p3
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: []
paths: []
tags: [deferred, documentation, tooling]
---
## The deferred question

`check-citations.sh` resolves the **path** part of a markdown link and deliberately does not resolve the `#fragment`. Measured 2026-08-08 over the live population: **312** links carry a `#heading` into another document and **486** are same-document `#anchor` links with no path at all. Both counts are printed on the `links:` census block so the size of the unresolved property stays visible.

## Why it was not resolved

The anchor slug is produced by whatever renders the markdown, and this repository pins no renderer. GitHub, common editors, and rustdoc disagree on punctuation stripping, on unicode handling, and on how a duplicate heading is suffixed. A checker that picked one algorithm would report failures that are artifacts of the choice against links that work where the documents are actually read — and a check that invents failures gets weakened rather than repaired, which is the failure mode the whole `check-citations.sh` design is built against.

Every alternative was worse for the same reason, not cheaper: matching heading **text** rather than a slug fails on any link whose fragment was written for a renderer; accepting a fragment that matches under any known slugifier is a check that cannot say no.

## What would change the answer

A pinned renderer. If this repository ever commits to one way the documents are read and rendered — a documentation build, a published site, a linter with a stated slug algorithm — the slug becomes a defined function and the 312 path-carrying fragments become checkable against it. Until then the honest position is that the fragment is not a property of this tree.

## Trigger check log

- 2026-08-08 — **not fired**. No renderer is pinned. Reproduce: `grep -rn "slug\|anchor" AGENTS.md docs/document-metadata.md` returns no rendering commitment, and no documentation build target exists in `Makefile` (`grep -n "^doc" Makefile` shows only `cargo doc`, which renders rustdoc for crates and not `docs/**`).
- 2026-08-09 — **not fired.** The repository still pins no Markdown renderer or heading-slug algorithm. The current repository guide now states the narrower implemented fact explicitly: `make citations` resolves the file but not the heading anchor after `#`; that is a declared limitation, not a renderer contract that can decide this ticket. `Makefile` still has only the Rust `doc` target. Recheck the anchor `not the heading anchor after a \`#\`` in `AGENTS.md` and `doc:` in `Makefile`.
