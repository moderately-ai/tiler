---
id: correct-residual-qwen-weight-permutation-products
title: Correct residual Qwen weight permutation products
status: todo
priority: p2
dependencies: []
related: [define-the-model-weight-binding-manifest, design-model-ingestion-and-complete-execution, design-model-level-qualification-and-optimization]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, language-model, correctness]
---
## User-visible outcome

The L6 and L8 planning records state the exact shape-and-dtype-preserving permutation population for the pinned Qwen checkpoint, so the quantitative explanation agrees with the checkpoint header it cites.

## Facts

**Fact — exact local header census at `07aca5cd8f67824019d8c183fd3a9584ce84b670`.** The safetensors header at pinned revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`, checkpoint SHA-256 `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba`, contains one size-1 class, one size-57 class, three size-56 classes, and three size-28 classes. The product is therefore `57! · 56!³ · 28!³`.

**Fact — four residual live sites.** The false `57! · 56!² · 28!⁴` product remains under these source anchors:

- `The number of shape-and-dtype-preserving bindings is` in `docs/research/program-planning/complete-model-ingestion-and-execution.md`;
- `the weight binding manifest against the pinned safetensors header` in `docs/research/program-planning/model-level-qualification.md`;
- `### The permutation nothing refuses` in `tickets/design-model-ingestion-and-complete-execution.md`; and
- `the weight binding manifest against the pinned safetensors header` in `tickets/design-model-level-qualification-and-optimization.md`.

The defining manifest ticket already carries the corrected product, so no fifth repair is needed there.

## Required work

- Re-read the exact checkpoint population evidence and all four complete owning files at the worker's base before editing.
- Replace the false product and any directly dependent ratio wording at exactly those four sites. Preserve the qualitative argument and do not widen the work into the binding-manifest implementation or model ingestion design.
- Verify all source anchors from source text, then run `tkt lint`, `make citations`, and the repository-required diff/scope checks.

## Closes when

All four residual sites state `57! · 56!³ · 28!³`, no live L6/L8 site retains the false product, and the correction is checked against the exact 310-name safetensors header census rather than copied from another prose record.
