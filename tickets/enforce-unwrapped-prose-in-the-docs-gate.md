---
id: enforce-unwrapped-prose-in-the-docs-gate
title: Enforce unwrapped prose in the documentation gate
status: closed
priority: p3
dependencies: [convert-docs-prose-to-unwrapped-source-form]
related: [record-prose-wrapping-convention-for-docs]
scopes: [contracts/navigation]
shared_scopes: []
paths: []
tags: [documentation, conventions, tooling]
closed_reason: wontdo
---
`docs/document-metadata.md` records that no mechanical check enforces the prose-wrapping convention, and states why: the check that expresses the rule would fail on most of the corpus today.

**Fact:** the repository has no width or wrapping check over Markdown. `grep -rn "line_length\|line-length\|columns\|wrap" scripts/docs.py scripts/check_repository.py` returns one hit, Ruff's `line-length = 100` for Python.

**Proposal:** once `convert-docs-prose-to-unwrapped-source-form` lands, add to `scripts/docs.py` a check that no `paragraph` token contains a `softbreak`, with a matching case in `scripts/tests/test_docs.py`. Reuse the `MarkdownIt("commonmark")` instance the validator already constructs; the predicate is exact rather than heuristic, because a table row, a fenced block, a heading, and a front-matter key are distinct token types that cannot produce a paragraph `softbreak`.

Two things this ticket must not do. A maximum-line-length check asserts the opposite convention and must never be added. A check narrowed to "new or edited documents" must not be added either: it needs either a stored per-file exemption list that every conversion and every parallel branch has to edit, or a diff base that a whole-tree gate does not have.

This is blocked on the conversion rather than merely sequenced after it, because the check cannot pass while any governed document is still wrapped.

## Outcome

Not started.

## Closed

Blocked on a corpus-wide conversion that is itself closed, and the check it
proposed is prose-shape policing of the kind the documentation gate has just
been trimmed of. Wrapping is a convention; a reviewer sees it, and a gate that
fires on honest prose gets argued with rather than obeyed.
