---
id: give-the-qwen3-reference-citation-its-provenance
title: Give the Qwen3 reference citation its provenance
status: in-progress
priority: p2
dependencies: []
related: [extend-the-citation-check-to-docs-and-repair-adr-0079-s-drifted-test-citation]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-give-the-
lease_expires_at: 1786161552
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

## Outcome

**Fact — the three Facts the ticket asserts all hold at this base, verified against the pinned bytes rather than inferred.** The reference table in the record pins `src/transformers/models/qwen3/modeling_qwen3.py` at `huggingface/transformers` v4.51.0, commit `0720e206c6ba28887e4d60ef60a6a089f6c1cc76`, 51,968 bytes, digest `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2`. Two copies on this host reproduce that digest and that byte size exactly, both `transformers` 4.51.0 wheel extractions in the uv archive cache (`archive-v0/4aLHrLNU6abBTd6O` and `archive-v0/lPePHGFsUPukwsjl`, each carrying a `transformers-4.51.0.dist-info`). Read from one of them: line 73 is `hidden_states = hidden_states.to(torch.float32)`, the first statement of `Qwen3RMSNorm.forward`; line 162 is the `nn.functional.softmax(..., dtype=torch.float32)` inside `eager_attention_forward`; and lines 336–344 are the `.float()` calls and the disabled-autocast block of `Qwen3RotaryEmbedding.forward` that build `cos` and `sin`. All three sites are unconditional, so the record's claim is unchanged and only its spelling moved. No network fetch was needed and none was attempted.

**Fact — the repair, and the convention it follows.** The citation is now rooted at the distribution the record already pins, as `transformers-4.51.0/src/transformers/models/qwen3/modeling_qwen3.py:73`, and the sentence names the version and the commit beside it. That is the first of the checker's two external branches — the version-pinned dependency-source form the header describes at `objc2-metal-0.3.2/src/generated/MTLDevice.rs:238` — reached by the `^[A-Za-z0-9_-]+-[0-9]+\.[0-9]+\.[0-9]+\//` test in `classify()`. The second branch, the component test the Candle citations in `docs/backends/metal.md` and `docs/integration/candle.md` use, is unavailable here: the upstream path begins `src/`, which is a component of hundreds of tracked paths, so an unversioned repo-relative spelling would still be demanded to resolve against this tree. The two neighbouring line references stay unpinned prose, as the ticket states, and now say which file they are in.

**Measurement — `./check-citations.sh` before and after, on this branch.** Before: 3 failures, 926 pinned citations checked, 658 from `docs/**`, 16 skipped as rooted outside this tree. After: 2 failures, 925 checked, 657 from `docs/**`, 17 skipped as rooted outside this tree. The failure count fell by exactly one and it is this record's; nothing new appeared. The five per-form counters and both wrapped/spanned floors are unmoved at 1073 line-only, 11 anchor-only, 1 line+anchor, 1 wrapped, 1 spanned, because the form is counted before the external branch is reached. The two survivors are `docs/research/runtime/backend-scoped-route-requirement-answers.md`, owned by [`give-the-two-runtime-record-external-citations-their-provenance`](give-the-two-runtime-record-external-citations-their-provenance.md), so the run stays red until that sibling lands.
