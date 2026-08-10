Ticket: measure-artifact-decoder-allocation-amplification
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/measure-artifact-decoder-allocation-amplification/015474461cae_c99ac54950f2.md
Pre-edit content hash (from ledger): 015474461cae9948f6f39bd054a409e4bab85f003683a5f756cc637e6212cde8
Post-edit content hash: 0b9b1172c16ab5745f19269004e0e30061bc3bab7de85cfe49e51a24812ea603

Changes applied:
  - Outcome remainder bullets: present-tense "makes" / "peaks" / "allocates" / "is filed" → filing-time past ("At filing … made/peaked/allocated/was filed") so discovery peaks are not unannotated live claims.
  - Dated correction after the two remainder bullets (`**Correction — 2026-08-10.**`): both successors done at base c99ac54950f2; arena 4k peak 670,658 (2.96×) in comparator.tsv not 1,569,620,906; program-codec MANIFEST_SCHEMA (16, 0); encode 64 MiB 4.99× → 2.00× (projection.tsv / spike table); reproduce anchors listed.
  - Optional micro-fix: watched-fail offsets 74343 and 97077 qualified as then-fixture-specific historical watch-fail positions; 0 and HEADER_BYTES 69 remain live.

Optional items skipped (with reason):
  - none (optional offset qualification applied as cheap graph/prose hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none (report: metadata unchanged; no new remainder; research note / crates already advanced by successors; ticket-only repair).

Verification:
  - files read:
    - tickets/measure-artifact-decoder-allocation-amplification.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/measure-artifact-decoder-allocation-amplification/015474461cae_c99ac54950f2.md
    - tickets/replace-the-codec-arena-content-key-with-the-existing-comparator.md (status: done)
    - tickets/stop-copying-the-carried-payload-through-the-envelope-projection.md (status: done)
    - crates/tiler-artifact/src/program/codec/encode.rs (MANIFEST_SCHEMA = (16, 0))
    - spikes/artifacts/decoder-allocation/results/decoder-allocation-macos-27.0-2026-08-06-comparator.tsv (4000-node decode 670658 / 2.96)
    - spikes/artifacts/decoder-allocation/results/decoder-allocation-macos-27.0-2026-08-06-projection.tsv (64 MiB encode 134558207 / 2.00)
    - spikes/artifacts/decoder-allocation/README.md (tables)
    - docs/research/artifacts/decoder-allocation-amplification.md (successor measurement narrative)
  - checks:
    - related tickets frontmatter status done
    - `const MANIFEST_SCHEMA: (u16, u16) = (16, 0)` in program codec encode.rs
    - comparator.tsv arena_chain=4000 decode peak 670658; projection.tsv 64 MiB encode 2.00×
    - ticket contains Correction — 2026-08-10 and "At filing" remainder wording
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
