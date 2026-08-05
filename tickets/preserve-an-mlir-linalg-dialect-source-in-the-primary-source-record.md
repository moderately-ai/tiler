---
id: preserve-an-mlir-linalg-dialect-source-in-the-primary-source-record
title: Preserve an MLIR Linalg dialect source in the primary-source record
status: done
priority: p3
dependencies: []
related: [enumerate-the-mature-tensor-operation-and-signature-taxonomy, preserve-the-array-api-standard-2025-12-as-a-numerics-source-record]
scopes: [research/numerics, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [sources, provenance, numerics]
---
## User-visible outcome

The MLIR Linalg claims in the operation and signature taxonomy rest on a pinned revision rather than an unpinned rendered page, so the last citation in that record resting on a moving URL closes. (Filed as "the last metadata-only citation"; the Outcome below records why that label was imprecise.)

## Why this exists

**Fact, 2026-08-05.** With the array API standard preserved (`preserve-the-array-api-standard-2025-12-as-a-numerics-source-record`), `mlir-linalg-dialect` is the only remaining metadata-only citation in `docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md`. That record names it as its own weakest citation: the preservation record pins `BuiltinTypes.td`, `QuantBase.td`, `LangRef.rst`, and `mlir-LangRef.md` from `llvm-project-llvmorg-22.1.8`, and **no Linalg source is among them**, so `linalg.generic`'s `indexing_maps`, `iterator_types`, and region body, the named structured operations, and destination-passing style with `outs` operands are cited from `https://mlir.llvm.org/docs/Dialects/Linalg/`, a page that tracks the project's main branch.

The claims are load-bearing in exactly one place — F-32's and F-33's observation that a structured operation carries its index structure as an attribute rather than in its name — which StableHLO's `dot_general` corroborates from a pinned source. So this is a citation-quality gap, not a correctness hole.

## What this ticket owes

Follow the source-record discipline in `docs/research/numerics/sources/README.md` exactly. The natural pin is `llvm/llvm-project` tag `llvmorg-22.1.8`, commit `ca7933e47d3a3451d81e72ac174dcb5aa28b59d1`, which the record already uses for four other files, so new files join the existing `llvm-project-llvmorg-22.1.8/` directory and inherit its preserved Apache-2.0-with-LLVM-Exceptions `LICENSE.TXT` rather than adding a second licence row. Candidate sources are `mlir/include/mlir/Dialect/Linalg/IR/LinalgStructuredOps.td` and `LinalgInterfaces.td` (the definition files behind the rendered page); verify the paths by reading the tree at the pinned commit rather than assuming them.

Update the manifest row, the declared population counts in `verify-sources.sh`, and the README record in the same change; run the verifier, then perturb one recorded digest once and watch it fail before trusting the pass. Then update the taxonomy's "Primary sources and the preservation boundary" section, which currently says one source is metadata-only, and its Traceability bullet, which says the same. That file is scope `research/semantic-graph`, declared here for that reason.

## Closes when

The verifier passes over the incremented declared population, the Linalg record states its verdict with its licence ground, and the taxonomy no longer describes any of its sources as metadata-only.

## Scope

Both scopes this work needed — `research/numerics` for `docs/research/numerics/sources/**` and `research/semantic-graph` for `docs/research/semantic-graph/**` — were already declared in this ticket's frontmatter when it was filed, and both mappings were read from `ticketsplease.toml` rather than asserted. Nothing was added during execution.

## Outcome, 2026-08-05

**Fact.** Seven files preserved under `docs/research/numerics/sources/llvm-project-llvmorg-22.1.8/`, acquired from `https://raw.githubusercontent.com/llvm/llvm-project/llvmorg-22.1.8/<path>`. The tag was dereferenced rather than assumed: it is the annotated tag object `e013073558445169e8732e25fa86e9913bfdd24e`, which resolves to commit `ca7933e47d3a3451d81e72ac174dcb5aa28b59d1` — the revision the five existing LLVM rows already use, so the corpus still carries one LLVM version. Declared population 61 → 68 records, 55 → 62 vendored; metadata-only stays at 6, because this citation was never a manifest row.

**Verdict: vendored,** on two quotations read in the acquired bytes rather than on Apache-2.0's reputation. Six of the seven files open with "Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions… SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception"; the preserved `LICENSE.TXT` grants in section 4 the right to "reproduce and distribute copies of the Work or Derivative Works thereof in any medium… provided that… (a) You must give any other recipients of the Work or Derivative Works a copy of this License", which the already-preserved `LICENSE.TXT` beside the files satisfies. No second licence row was added.

**The seventh file carries no licence header, and the record says so rather than inheriting one silently.** `LinalgNamedStructuredOps.yaml` is generated output beginning `### AUTOGENERATED from core_named_ops.py`; `grep -inE 'licen|copyright|spdx|apache' LinalgNamedStructuredOps.yaml` over the acquired copy returns nothing, so its permission rests on the repository-level `LICENSE.TXT` alone.

**The file set was derived from the claims, not from the ticket's two candidates.** `LinalgStructuredOps.td` and `LinalgInterfaces.td` alone do not carry every cited claim: `pack`/`unpack` moved to `LinalgRelayoutOps.td`, and `conv_*`/`pooling_*` are OpDSL entries in `LinalgNamedStructuredOps.yaml` while `matmul`/`batch_matmul` have moved to TableGen at this revision. `DestinationStyleOpInterface.td` and `IndexingMapOpInterface.td` carry the two definitions the "index structure as an attribute" and "destination-passing style" claims rest on, and `StructuredOpsUtils.td` was added for the reason below.

**An upstream inconsistency found by reading the acquired bytes, now pinned rather than repeated.** `GenericOp`'s description in `LinalgStructuredOps.td` says `iterator_types` elements are "one of the following types: parallel, reduction, window", while `StructuredOpsUtils.td` at the same commit defines `IteratorType` with exactly two cases, `parallel` and `reduction`. The enum is the authority and the prose is stale upstream. Preserving the enum file is what keeps the correction re-derivable instead of resting on a live URL. No taxonomy claim depended on the three-name list.

**Every taxonomy Linalg claim survived the re-check** — `generic`'s `indexing_maps`, `iterator_types`, and region; `transpose`, `broadcast`, `reduce`, `matmul`, `batch_matmul`, `pack`, `unpack`; and the `conv_*` (19) and `pooling_*` (17) families. Two precision points were recorded without changing a row: the OpDSL/TableGen split above, and that `outs` is Linalg's assembly spelling of the operands the interface calls "init" operands. Unlike the Array API wave, nothing failed.

**A wording precision carried into both records.** This ticket and the taxonomy both called `mlir-linalg-dialect` a "metadata-only" citation, but the preservation manifest never held such a row — its `metadata-only` class is six dtype specifications and was unchanged by this work. The taxonomy now describes the boundary as read-during-the-pass versus re-checked-afterwards, which is the distinction that was actually at stake.

**Reproducibility, 2026-08-05.** Each of the seven files was retrieved twice by independent routes — at the tag and at the commit SHA — and compared byte-for-byte: seven compared, seven identical. Each copy's git blob SHA-1 and length were also checked against the GitHub contents API at that commit: seven checked, seven matching. The seventh took eleven retries — seven unauthenticated calls in quick succession trip a secondary rate limit that returns HTTP 429 bodies for about fifteen minutes. That is a refused call, not a failed check, and the distinction is recorded in the source README so the next reader does not mistake one for the other.

**Verifier, three runs.** Clean over the incremented population (`OK: 68 records verified (62 vendored, 6 metadata-only, 0 pending-acquisition)`, exit 0); one digest perturbed on `mlir-linalg-structured-ops-llvmorg-22.1.8` (`FAIL: … digest mismatch for llvm-project-llvmorg-22.1.8/LinalgStructuredOps.td`, `1 check(s) failed over 68 declared records.`, exit 1); restored and clean again, with the manifest byte-compared against its pre-perturbation copy.
