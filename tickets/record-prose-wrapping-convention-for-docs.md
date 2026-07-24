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
