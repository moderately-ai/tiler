---
id: preserve-an-mlir-linalg-dialect-source-in-the-primary-source-record
title: Preserve an MLIR Linalg dialect source in the primary-source record
status: in-progress
priority: p3
dependencies: []
related: [enumerate-the-mature-tensor-operation-and-signature-taxonomy, preserve-the-array-api-standard-2025-12-as-a-numerics-source-record]
scopes: [research/numerics, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [sources, provenance, numerics]
claimed_from: todo
assignee: agent-mlir-linalg
lease_expires_at: 1785940859
---
## User-visible outcome

The MLIR Linalg claims in the operation and signature taxonomy rest on a pinned revision rather than an unpinned rendered page, so the last metadata-only citation in that record closes.

## Why this exists

**Fact, 2026-08-05.** With the array API standard preserved (`preserve-the-array-api-standard-2025-12-as-a-numerics-source-record`), `mlir-linalg-dialect` is the only remaining metadata-only citation in `docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md`. That record names it as its own weakest citation: the preservation record pins `BuiltinTypes.td`, `QuantBase.td`, `LangRef.rst`, and `mlir-LangRef.md` from `llvm-project-llvmorg-22.1.8`, and **no Linalg source is among them**, so `linalg.generic`'s `indexing_maps`, `iterator_types`, and region body, the named structured operations, and destination-passing style with `outs` operands are cited from `https://mlir.llvm.org/docs/Dialects/Linalg/`, a page that tracks the project's main branch.

The claims are load-bearing in exactly one place — F-32's and F-33's observation that a structured operation carries its index structure as an attribute rather than in its name — which StableHLO's `dot_general` corroborates from a pinned source. So this is a citation-quality gap, not a correctness hole.

## What this ticket owes

Follow the source-record discipline in `docs/research/numerics/sources/README.md` exactly. The natural pin is `llvm/llvm-project` tag `llvmorg-22.1.8`, commit `ca7933e47d3a3451d81e72ac174dcb5aa28b59d1`, which the record already uses for four other files, so new files join the existing `llvm-project-llvmorg-22.1.8/` directory and inherit its preserved Apache-2.0-with-LLVM-Exceptions `LICENSE.TXT` rather than adding a second licence row. Candidate sources are `mlir/include/mlir/Dialect/Linalg/IR/LinalgStructuredOps.td` and `LinalgInterfaces.td` (the definition files behind the rendered page); verify the paths by reading the tree at the pinned commit rather than assuming them.

Update the manifest row, the declared population counts in `verify-sources.sh`, and the README record in the same change; run the verifier, then perturb one recorded digest once and watch it fail before trusting the pass. Then update the taxonomy's "Primary sources and the preservation boundary" section, which currently says one source is metadata-only, and its Traceability bullet, which says the same. That file is scope `research/semantic-graph`, declared here for that reason.

## Closes when

The verifier passes over the incremented declared population, the Linalg record states its verdict with its licence ground, and the taxonomy no longer describes any of its sources as metadata-only.
