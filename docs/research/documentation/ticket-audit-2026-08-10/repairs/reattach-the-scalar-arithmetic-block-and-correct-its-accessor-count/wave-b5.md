Ticket: reattach-the-scalar-arithmetic-block-and-correct-its-accessor-count
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/reattach-the-scalar-arithmetic-block-and-correct-its-accessor-count/12145604c2c5_c99ac54950f2.md
Pre-edit content hash (from ledger): 12145604c2c568bc41165a602b674094bc040177daae9b2cb76ad57b93b8d239
Post-edit content hash: 3c2f3ea9a91cdec4e01091e769381054eda94b31313daf650d187f8d8da5a1e6

Changes applied:
  - Soft-fixed present-tense "159 hits" Fact to 186 (re-measured 2026-08-10 with `rg -n "2026-0" crates/ --glob '*.rs' | wc -l`); added **Correction — 2026-08-10** recording frozen close measurement 159 vs current 186.
  - Rephrased bf16 claim: three Corrected-2026-08-07 blocks, two opening `The reason given here was retired and is struck. Corrected 2026-08-07.` and one opening `A boundary claimed here was retired and is struck. Corrected 2026-08-07.`
  - Filed remainder ticket `correct-the-five-key-domains-noun-on-the-model-re-export-block` (todo, implementation/artifact) for live domain-block noun imprecision (`// five key domains re-exported from \`model\`` vs ARTIFACT_DOMAIN as identity separator).
  - Wired parent `related: [correct-the-five-key-domains-noun-on-the-model-re-export-block]` and remainder `related` back to parent; Neighbouring census now links the filed remainder and states source remains live until it lands.

Optional items skipped (with reason):
  - none (dated correction on 159 count applied as part of the required soft-fix).

Residuals not applied (docs/crates/new tickets/authority):
  - crates/tiler-artifact/src/program/mod.rs domain-block wording remains live product debt; owned by the new remainder ticket (Class D filed, not executed in this wave).

Verification:
  - files read:
    - audit report 12145604c2c5_c99ac54950f2.md
    - tickets/reattach-the-scalar-arithmetic-block-and-correct-its-accessor-count.md (pre/post)
    - crates/tiler-artifact/src/program/mod.rs domain block (five key domains + model use list)
    - crates/tiler-artifact/src/domains.rs Program* separator docs
    - crates/tiler-conformance/src/bf16_vertical.rs Corrected-2026-08-07 openers
    - sample todo tickets for remainder frontmatter style
  - checks:
    - `rg -n "2026-0" crates/ --glob '*.rs' | wc -l` → 186
    - bf16: two "The reason given here…" + one "A boundary claimed here…" Corrected 2026-08-07
    - domain block still `// five key domains re-exported from \`model\`` at live source
    - no pre-existing five-key-domains remainder under tickets/
    - sha256 post-edit ticket: 3c2f3ea9a91cdec4e01091e769381054eda94b31313daf650d187f8d8da5a1e6

Recommended next ledger state:
  integrated
