---
id: list-the-corpus-reachability-spike-in-the-spike-index
title: List the corpus reachability spike in the spike index
status: todo
priority: p3
dependencies: []
related: [define-the-model-level-conformance-corpus, design-model-level-qualification-and-optimization]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, navigation, spikes, language-model]
---
## Why this is a ticket rather than part of the work that created the spike

[`define-the-model-level-conformance-corpus`](define-the-model-level-conformance-corpus.md) added [`spikes/program-planning/qwen3-corpus-reachability/`](../spikes/program-planning/qwen3-corpus-reachability/README.md) under `research/program-planning`, which covers `spikes/program-planning/**`. `spikes/README.md` is mapped to `contracts/navigation`, which that ticket does not hold, so the index entry could not land in the same change. Nothing validates the corpus — the index is hand-maintained prose — so an unlisted spike costs a reader rather than a gate, which is exactly why it is filed instead of absorbed.

## Required content

- An entry in the spike index's `program-planning` list, in the existing form: title, reproducibility, evidence classes, and the records it supports.
  - Title: **Qwen3-0.6B-Base conformance-corpus reachability probe**
  - Reproducible; evidence classes **exhaustive-finite** (the 65,536-pattern BF16→F32 class map) and **bounded-measurement** (the checkpoint's stored-value counts and the 330-position tie search). The two are separate classes on purpose and the entry should not collapse them into one.
  - Supports [Model-level correctness and performance qualification](../docs/research/program-planning/model-level-qualification.md), [First Metal language-model workload profile](../docs/research/program-planning/first-metal-lm-workload.md), and [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md).
- The dependency note near the top of that file already names `qwen3-conformance-fixture` as one of the four harnesses that are not standard library plus `pytest`. This spike is a fifth: it pins its own locked `torch` and `transformers` environment, copied from the fixture's resolution rather than re-resolved, for the same reason the fixture states — the probe's C1 control compares against figures the fixture measured under exactly that resolution, so a second resolution would make "the same reference" an assumption instead of a checked fact. Update the count and the list rather than leaving "four" wrong.

## Closes when

The spike appears in the index with its evidence classes and supported records, and the harness-dependency note counts and names it.
