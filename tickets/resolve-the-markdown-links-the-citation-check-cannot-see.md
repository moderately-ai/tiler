---
id: resolve-the-markdown-links-the-citation-check-cannot-see
title: Resolve the markdown links the citation check cannot see
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The catalog rows that navigate the whole corpus are unchecked

`check-citations.sh` validates **pinned** citations — a path with a line number or a quoted anchor. It does not validate **markdown links**, which are how every catalog, index and cross-reference in `docs/` is navigated.

**Coordinator-verified by planting the failure.** Replacing a live ADR link in `docs/decisions/README.md` with `](9999-no-such-adr-…)` leaves `make citations` at **exit 0**, reporting `every pinned citation resolves`, with the citation count unchanged. The dangling link is counted among **`3255 bare path mention(s) carrying no line or anchor`** and never resolved.

Found by the worker repairing ADR 0107's catalog rows, which pointed *both* links in its new row at non-existent files and got byte-identical output. Its break test succeeded at showing the gate is blind here rather than at showing it works — and it said so plainly instead of recording a green run as evidence.

## Why this is p1 rather than housekeeping

The population is **3,255 references**, and it includes the entry points `AGENTS.md` directs every reader to: *"For broad design work, start with `docs/README.md`; accepted decisions are indexed in `docs/decisions/README.md`."* A dangling row there is a reader sent nowhere from the document that exists to route them.

It is also the exact defect class this repository keeps finding: the checker reports a clean verdict over a population it never examined, and a reader reasonably takes "citations green" to mean the document's references resolve. `AGENTS.md` is explicit that these catalogs have **no automated validator** — this closes the half of that gap that is mechanical.

The worker resolved all **582 local links** with a one-off check that correctly reported the two dangling ones and exited 1, so the property is cheaply checkable. What is missing is that it runs.

## Requirements

- **Resolve every local markdown link** in the populations the script already reads, relative to its own file's directory, and **report the population separately** from pinned citations so neither can collapse into the other. The script already keeps `tickets` and `docs` apart; follow that shape.
- **Floor it.** A run that resolves zero links must fail — the failure this repository has hit repeatedly, and which the script's five existing per-form floors exist to prevent.
- **Do not check external links** (`http`, `mailto`) or anchors into other documents' headings without deciding that separately and saying why.
- **Watch it fail**, separately, for: a dangling relative link, a link to a file outside the tree, and an empty link population. Perturb the subject, quote each message.
- **Expect the first full run to fail** on real defects and **report them rather than weakening the check**. The docs extension found fourteen when it landed; this population is larger.

## Scope note

`check-citations.sh` and `Makefile` are both in the delta rule's gated set, so a change here **cannot carry the gate** — run `make full`.

## Closes when

`make citations` resolves local markdown links across both populations with counts reported separately, fails on an empty link population, has been watched failing in three ways, and every defect the first run surfaces is repaired or filed.
