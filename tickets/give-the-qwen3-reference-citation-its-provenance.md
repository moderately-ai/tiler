---
id: give-the-qwen3-reference-citation-its-provenance
title: Give the Qwen3 reference citation its provenance
status: todo
priority: p2
dependencies: []
related: [extend-the-citation-check-to-docs-and-repair-adr-0079-s-drifted-test-citation]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## What the docs citation check surfaced

`check-citations.sh` gained a `docs/**` population under `extend-the-citation-check-to-docs-and-repair-adr-0079-s-drifted-test-citation`. It reports one failure in `docs/research/program-planning/first-metal-lm-workload.md`:

```
FAIL  docs/research/program-planning/first-metal-lm-workload.md
        citation: `modeling_qwen3.py:73`
        no file in the tree is or ends with modeling_qwen3.py
```

**Fact — this is not Tiler drift.** It cites the pinned HuggingFace `transformers` reference implementation, in the sentence "the three float32 sites this profile records above — `Qwen3RMSNorm.forward` at" that file, line 73. The file is a real line in a real upstream source; it is simply not in this tree.

**How this ticket spells that extent, and why.** As a bare path plus a prose line number, never pinned as `path:LINE` — the same convention a dated correction uses when it retires a citation, and the reason a bare path carrying no pin is deliberately not checked. A ticket that pinned the broken form would fail the very check it is asking someone to satisfy; this one did, on its first run, before this paragraph existed. The verbatim failing spelling is preserved in the fenced block above, which the checker skips.

**Fact — the checker cannot skip it, and the reason is deliberate.** A path is skipped as rooted outside this tree only when it has a `/` and its leading segment is a component of no tracked path. `modeling_qwen3.py` has no `/`. Widening the rule to bare filenames was considered and refused in that ticket: a bare filename is this repository's own shorthand for its own files, so treating an unresolvable one as external would silently stop reporting real drift.

## The repair

Spell it with the provenance the record already establishes — the `transformers` version this profile is pinned to — so the path is rooted in the project it names, in the shape the checker already recognizes for external sources (`objc2-metal-0.3.2/src/generated/MTLDevice.rs:238` is the standing example). The two neighbouring line references in the same sentence ("line 162", "lines 336–344") are prose rather than pinned citations and are not affected.

Nothing the fact asserts changes. Re-read the pinned source at your own base before editing.

## Closes when

`./check-citations.sh` reports no failure in `docs/research/program-planning/`, and the citation names the `transformers` revision it is about.
