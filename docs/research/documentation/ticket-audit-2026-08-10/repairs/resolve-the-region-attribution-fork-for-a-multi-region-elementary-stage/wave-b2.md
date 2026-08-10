Ticket: resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage/28c77509a0f3_c99ac54950f2.md
Pre-edit content hash (from ledger): 28c77509a0f300d09ea0451d1ad093d398cc914ba2b22a09f9a6e172c666494d
Post-edit content hash: a855606264996c3fc95468efeb3631218e094858f3dc44226f233c2908da452b

Changes applied:
  - frontmatter `related`: added `implement-stage-level-cover-atoms-for-multi-region-occurrences` (symmetric with that ticket's related list and this ticket's Outcome link); kept existing admit/widen entries
  - opening problem sentence struck as historical/obsolete; **Correction — 2026-08-10.** block frames the whole derivation as pre-decision and points at live `SemanticStage`
  - three derivation **Fact.** / **Inference** blocks reframed as historical (`~~Fact~~` / `~~Inference~~`) with per-citation **Correction — 2026-08-10.** notes
  - code citations updated to symbol anchors without rotten line ranges: `owns_region_members` (now `&[SemanticStage]`), `spell_output`, `derive_duplication`, `RegionWrite` — retired ranges `request.rs:1660-1680`, `physical.rs:442-518`, `cover.rs:1999-2018`, `physical.rs:222-252` explicitly called out as do-not-cite
  - Outcome section: dated correction that Option A landed, implement ticket done, bare-occurrence keying is pre-decision only, and this ticket must not absorb Option A′ / identity-fold / enumeration remainder

Optional items skipped (with reason):
  - add `resolve-which-authority-mints-a-multi-stage-region-candidate` to `related` — optional graph cataloguing of Option A′ sequel only; reverse edge already exists on that ticket; not required for close correctness

Residuals not applied (docs/crates/new tickets/authority):
  - none; report required only this ticket file; no new remainder tickets; no docs/crates edits

Verification:
  - files read:
    - tickets/resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage.md (pre- and post-edit)
    - audit report 28c77509a0f3_c99ac54950f2.md in full
    - tickets/implement-stage-level-cover-atoms-for-multi-region-occurrences.md (frontmatter + decision surface)
    - crates/tiler-compiler/src/region.rs (`SemanticStage` vicinity)
    - crates/tiler-compiler/src/cover.rs (`derive_duplication` body)
  - checks:
    - `rg` for `fn owns_region_members|fn spell_output|fn derive_duplication|enum RegionWrite|struct SemanticStage|owns_stage_members` under crates/tiler-compiler/src — confirms stage-atom signatures and locations (request.rs ~2458, physical.rs ~809/547, cover.rs ~2038, region.rs ~176)
    - sha256 of ticket after edit: a855606264996c3fc95468efeb3631218e094858f3dc44226f233c2908da452b

Recommended next ledger state:
  integrated
