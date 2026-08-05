---
id: preserve-the-array-api-standard-2025-12-as-a-numerics-source-record
title: Preserve the array API standard 2025.12 as a numerics source record
status: done
priority: p3
dependencies: []
related: [enumerate-the-mature-tensor-operation-and-signature-taxonomy]
scopes: [research/numerics, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [sources, provenance, numerics]
---
## User-visible outcome

The Python array API standard **2025.12** has a proper row in the numerics source record, so the operation taxonomy's citation of it is re-derivable without a version-qualified fetch of a `latest` path that moves.

## Why this exists

**Fact, 2026-08-04.** `docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md` cites the array API standard 2025.12 as one of two metadata-only citations, and its promotion-policy finding is load-bearing (the record's no-promotion posture rests partly on the standard leaving cross-kind promotion deliberately unspecified). No local copy exists and no digest was recorded — the taxonomy worker fetched it and said so rather than implying a pin.

## What this ticket owes

Follow the source-record discipline in `docs/research/numerics/sources/README.md` exactly: acquire the 2025.12 document, review its own licence text for a redistribution grant (it is expected to be permissive, in which case the verdict is **vendored** rather than metadata-only — but the verdict comes from the licence text read in the acquired copy, never from expectation), record the digest over the exact retrieved bytes, and update the manifest row, the declared population counts in `verify-sources.sh`, and the README record in the same change. Run the verifier and watch it pass; perturb one recorded digest once and watch it fail before trusting the pass.

## Closes when

The verifier passes over the incremented declared population, the record entry states the verdict with its licence ground, and the taxonomy record's "Primary sources and preservation boundary" section is updated to drop the metadata-only caveat for this source.

## Scope added during execution

`research/semantic-graph` was added on 2026-08-05. The close condition names an edit to `docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md`, which `ticketsplease.toml` maps to `research/semantic-graph` ("research/semantic-graph" = ["docs/research/semantic-graph/**"], read from the config rather than asserted); the ticket declared only `research/numerics`. This is declaration bookkeeping for already-authorized work, not new product scope. No other live ticket held `research/semantic-graph` during this wave.

## Outcome, 2026-08-05

**Fact.** Acquired at `data-apis/array-api` tag `2025.12`, commit `d016d578040d151707a5b7dd2ba1e55f48a8d511`, via `https://raw.githubusercontent.com/data-apis/array-api/2025.12/<path>`. Fifteen files preserved under `docs/research/numerics/sources/array-api-2025.12/`. **Verdict: vendored**, on the MIT grant read in the acquired `LICENSE` — "Permission is hereby granted... to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute" — which `spec/2025.12/license.rst` extends to the specification text, not only to code. Declared population 46 → 61 records, 40 → 55 vendored.

**Two defects in the cited record were found by reading the preserved bytes, and both are corrected in the same commit** — which is the argument for preservation stated as evidence rather than as principle.

1. Section E's F-24 paragraph asserted that "Array API 2025.12's `take` and Python-style slicing follow the same permissive convention" as ONNX `Slice`'s clamping. The standard says the opposite: `indexing.rst` states it "does not require \"clipping\" out-of-bounds slice indices. This is in contrast to Python slice semantics where `0:100` and `0:10` are equivalent on a list of length `10`", and `take` has left out-of-bounds behaviour "unspecified and thus implementation-defined" explicitly since 2023.12. The corrected fact strengthens the paragraph's conclusion rather than weakening it — three authorities, three postures, none inheritable — so the inference is preserved and only its arithmetic changed.
2. Section I's source line listed `eig` in the Array API `linalg` extension. The extension defines `eigh` and `eigvalsh` and no general `eig`.

Every other Array API claim in the taxonomy was checked against the preserved bytes and holds: ten chapter inventories, three verbatim type-promotion quotations, and the verbatim sort-order note.

**Stale assertions corrected along the way.** The sources README declared "46 records, of which 40 vendored, 4 metadata-only, 2 pending-acquisition" while `verify-sources.sh` enforced 40/6/0; the counts sentence now matches the verifier, and the "Pending-acquisition records" heading now says in one sentence that it is a narrative grouping and that the verifier's classes are the authority.

**Preservation boundary recorded.** Only `indexing_functions.py` is preserved from `src/array_api_stubs/_2025_12/`, because exactly one taxonomy claim rests on per-function docstring text. Any future citation of per-function prose from another chapter, or of the design-topic chapters, must extend the record; that is stated in the record itself.

**Filed:** `preserve-an-mlir-linalg-dialect-source-in-the-primary-source-record` — `mlir-linalg-dialect` is now the only metadata-only citation left in the taxonomy, and no ticket covered it.
