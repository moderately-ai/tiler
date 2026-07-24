---
id: record-prose-wrapping-convention-for-docs
title: Record the prose-wrapping convention for authored documentation
status: in-progress
priority: p2
dependencies: []
related: [draft-public-api-conventions-adr]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, conventions]
claimed_from: todo
assignee: agent-record-prose-wrapping-convention-for-docs
lease_expires_at: 1784932575
---
Tom's standing authoring instruction is to **never hard-wrap prose** — no
newlines inserted mid-paragraph to hit a column width, in file writes as much as
in terminal output; a paragraph is one long line and the renderer soft-wraps.
Accepted ADR 0074 follows it and is currently the only unwrapped document among
74 decision records; every other ADR is wrapped at roughly 78 columns.

No mechanical gate enforces either form, so today each author guesses from the
nearest sibling and the corpus drifts by imitation — the same failure mode ADR
0074 itself was written to stop for API shape.

**Decided 2026-07-24: unwrapped is the convention going forward.** Record it in
the appropriate docs-authoring contract (most likely `docs/document-metadata.md`,
which already contains unwrapped body paragraphs — confirm before assuming) so a
future author can cite a rule instead of copying whichever neighbour they opened
first.

Explicitly **do not mass-rewrap the existing corpus.** Rewrapping 73 ADRs would
produce an enormous diff touching documents whose content nobody reviewed, for no
reader benefit. The corpus migrates naturally as documents are edited for other
reasons; mixed wrapping in the meantime is expected and is not drift to be
"fixed".

If a mechanical check is ever added, it must only assert the convention for newly
authored or substantively edited documents, never reformat wholesale.

## Outcome

`docs/document-metadata.md` gained a **Prose source form** section and extended its Ownership statement to cover the source form of governed Markdown; `docs/README.md`'s reading-order note now routes an author there before authoring. The section states the rule, its scope (prose inside list items, table cells, block quotations, and footnotes, never the newlines that carry structure), the transitional policy (the paragraph converts when edited; the file is not reflowed; a half-wrapped paragraph is a defect rather than a transitional state), and why no gate enforces it yet.

Two facts in that section were verified rather than assumed. `scripts/docs.py` constructs `MarkdownIt("commonmark")` at line 454 against a locked `markdown-it-py==4.2.0`, so the `softbreak`-inside-`paragraph` predicate the section names is exact rather than heuristic. `catalog()` emits each generated entry as a single line, so the renderer already writes the form the rule asks authors for.

Two consequences the ticket declined to take are recorded as tickets rather than left implicit: `convert-docs-prose-to-unwrapped-source-form` (deferred; carries the 133/15/6 corpus measurement, the mechanical-safety argument, and the quiescent-tree trigger) and `enforce-unwrapped-prose-in-the-docs-gate` (deferred behind it, with the two check shapes that must never be added).

`uv run --locked python scripts/docs.py render` and `tkt lint` pass.
