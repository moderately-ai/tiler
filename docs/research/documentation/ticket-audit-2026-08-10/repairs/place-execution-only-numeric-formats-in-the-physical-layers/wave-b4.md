Ticket: place-execution-only-numeric-formats-in-the-physical-layers
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/place-execution-only-numeric-formats-in-the-physical-layers/d5cccf8f0b3e_c99ac54950f2.md
Pre-edit content hash (from ledger): d5cccf8f0b3e341dea35fbf3fa441e40a8c2bb838bb070b343e9223020903ef7
Post-edit content hash: 3067a2f95d78945ccd44511e483cb2b00691bf0e25dcd9c9be2ca727ec5bb645

Changes applied:
  - Revised Trigger check log 2026-08-09: removed the false "names only in taxonomy + ledger + negative census" claim; kept verified no-physical-consumer clause (`KernelType` / `StorageScalar` / carrier / target-profile); noted docs/ADRs also name the formats as exclusions; noted catalog negative census covers only tf32, x86_fp80, ppc_fp128 (not ue4m3/ue8m0); noted crates/ still only hits that three-name census.
  - Optional 2026-08-04 locator note: marked original `:217` as historical; D-11 found by heading `#### D-11 — Execution-only and target ABI formats` (near line 224); left **not fired** verdict standing.
  - Metadata unchanged (status deferred, dependencies [], related triple, scopes, tags).

Optional items skipped (with reason):
  - none (optional `:217` note applied as cheap graph hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report. Product placement remains deferred until a selected backend consumer fires the trigger; no remainder ticket and no docs/crates edits in wave B.

Verification:
  - files read:
    - tickets/place-execution-only-numeric-formats-in-the-physical-layers.md (full, pre and post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/place-execution-only-numeric-formats-in-the-physical-layers/d5cccf8f0b3e_c99ac54950f2.md (full)
    - docs/research/numerics/dtype-family-research-tracks.md (D-11 heading at line 224)
    - crates/tiler-ir/src/semantic/catalog/tests.rs (alias_spellings_and_lookalikes_have_no_authority: tf32, x86_fp80, ppc_fp128 only)
  - checks:
    - rg tf32|ue4m3|ue8m0|x86_fp80|ppc_fp128 under docs/ (hits taxonomy, ledger, ADR 0036, tracks, sources, etc. beyond the three named locations)
    - rg same under crates/ → only catalog tests for the three census names
    - rg `#### D-11 — Execution-only` → line 224
    - shasum -a 256 of ticket after edit

Recommended next ledger state:
  integrated
