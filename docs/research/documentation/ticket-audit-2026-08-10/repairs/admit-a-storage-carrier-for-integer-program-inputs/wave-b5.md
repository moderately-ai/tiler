Ticket: admit-a-storage-carrier-for-integer-program-inputs
Wave: B5
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-storage-carrier-for-integer-program-inputs/99afe4fe32f2_c99ac54950f2.md
Pre-edit content hash (from ledger): 99afe4fe32f2f9c84ab6d769c0f47e0ddeb4b35f9a952642ea3af06e443615a2
Post-edit content hash: 9c91f7ec896bd8999de02b8a876d28a0dd43a6a8fdf29c0eece4924311d1369d

Changes applied:
  - Site-enumeration table: Declared? set to yes for `every_storage_carrier_has_a_representable_alignment` and `msl_type`; replaced false `index_arithmetic_requirement` / `physical.rs` / `implementation/compiler` row with `IndexArithmetic::of` / `crates/tiler-ir/src/kernel/model.rs` / `implementation/ir`; noted that physical.rs matches only IndexArithmetic and is not broken by KernelType::U32.
  - Softened present-tense "cannot be landed under its declared scopes" to historical pre–scope-repair Fact; cleared obsolete "ticket remains blocked" for dependency/scope holds to point at awaiting-decision public surface.
  - Redispatch notes: no live BF16 msl_type refusal (Bf16 spells bfloat); U32 refuse-by-name via UnsupportedValueType on maturity ground; msl_type decision section updated the same way.
  - Coordinator site list: dropped physical.rs as a KernelType widening site; marked 2213-test count as historical at 68f1ced6 requiring re-measure on land.
  - Added ## Fact audit — 2026-08-10 / **Correction — 2026-08-10.** with the three required audit points (IndexArithmetic::of, Declared? staleness, BF16 live spelling vs U32 refuse).
  - Frontmatter status/deps/scopes left unchanged (report: none required).

Optional items skipped (with reason):
  - none (optional BF16 msl_type note included in dated correction as report allowed).

Residuals not applied (docs/crates/new tickets/authority):
  - Product outcome still undelivered: no StorageScalar::U32 / KernelType::U32; implementation remains parked on Tom's public-surface acceptance under ADR 0075 (awaiting-decision). Class E: crates not edited this wave.
  - No remainder ticket: report says none; atomic landing stays this ticket after acceptance.
  - docs/ crates/ not touched (wave ticket-only).

Verification:
  - files read:
    - tickets/admit-a-storage-carrier-for-integer-program-inputs.md (full, pre and post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-storage-carrier-for-integer-program-inputs/99afe4fe32f2_c99ac54950f2.md (full)
    - crates/tiler-ir/src/kernel/model.rs (KernelType, IndexArithmetic::of)
    - crates/tiler-ir/src/program/model.rs (StorageScalar)
    - crates/tiler-compiler/src/physical.rs (index_arithmetic_requirement → IndexArithmetic only)
    - crates/tiler-metal/src/emit.rs (msl_type Bf16 => bfloat)
    - crates/tiler-artifact/src/program/codec/tests.rs (UNASSIGNED_CARRIER 0x04 / UNASSIGNED_ACCESS 0x07)
  - checks:
    - rg IndexArithmetic::of / index_arithmetic_requirement / msl_type / StorageScalar against current main tree; matches audit verdicts 2, 8, 11, 13.
    - shasum -a 256 on edited ticket → 9c91f7ec896bd8999de02b8a876d28a0dd43a6a8fdf29c0eece4924311d1369d

Recommended next ledger state:
  integrated
