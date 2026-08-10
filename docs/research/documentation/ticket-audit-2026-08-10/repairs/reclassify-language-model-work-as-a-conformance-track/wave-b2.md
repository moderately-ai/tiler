Ticket: reclassify-language-model-work-as-a-conformance-track
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/reclassify-language-model-work-as-a-conformance-track/b136be2c53c6_c99ac54950f2.md
Pre-edit content hash (from ledger): b136be2c53c6b467023289f5211894d28af333682702ccfc5345091482e69fcf
Post-edit content hash: a640ed7751934adf3e479b35daf8a53d116a4f422f891c3bce3f40eafe9ca399

Changes applied:
  - Outcome class-obsolete parenthetical: replaced the false link to `supersede-the-runtime-owned-kv-state-design` with the two actual tagged nodes `define-the-model-execution-state-boundary` and `define-the-runtime-kv-state-boundary` (already closed/superseded; inventory-only tags); noted supersede as the superseding agent, not a class-obsolete node.
  - Outcome reproduction command: replaced bare `grep -l 'class-' tickets/*.md` with the four full-tag greps (`class-generic-capability` and three siblings) and an explicit note that a bare `class-` prefix is not a safe match (matches roadmap).
  - Dated **Correction — 2026-08-10** on “Findings outside this ticket's scopes”: (a) ladder signposts and empty `language-model inference ladder` grep discharged by `complete-the-kv-ownership-supersession-sweep` (done); (b) ten KV-ownership sentences and L5-wait exclusion corrected there; (c) historical bullets no longer live corpus claims. Struck the two present-tense ladder/KV remainder paragraphs.
  - Optional orphan-sweep tighten: iOS deferred node described as product-trigger parked work with `dependencies: []`; tiled contraction left as activation-trigger work naming cooperative-tile deps.

Optional items skipped (with reason):
  - none (orphan-sweep optional prose was applied as cheap same-ticket hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report; no docs/crates edits; no new remainder tickets. Live follow-ons already owned elsewhere (`decide-the-source-bearing-slice-offset-boundary`, deferred parked work).

Verification:
  - files read:
    - tickets/reclassify-language-model-work-as-a-conformance-track.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/reclassify-language-model-work-as-a-conformance-track/b136be2c53c6_c99ac54950f2.md (full)
    - tickets/define-the-model-execution-state-boundary.md / define-the-runtime-kv-state-boundary.md (class-obsolete tags via grep)
    - tickets/complete-the-kv-ownership-supersession-sweep.md (status: done)
    - tickets/first-authoritative-ios-metal-compile-declaration.md (status: deferred, dependencies: [])
    - docs/roadmap.md (four full-tag reproduction command)
    - docs/ for `language-model inference ladder` (empty outside audit reports)
  - checks:
    - `grep -l class-obsolete tickets/*.md` → define-the-model-execution-state-boundary, define-the-runtime-kv-state-boundary only (plus this ticket’s prose)
    - `grep -rn 'language-model inference ladder' docs/` → no live research/status hits (audit reports only)
    - post-edit sha256: a640ed7751934adf3e479b35daf8a53d116a4f422f891c3bce3f40eafe9ca399

Recommended next ledger state:
  integrated
