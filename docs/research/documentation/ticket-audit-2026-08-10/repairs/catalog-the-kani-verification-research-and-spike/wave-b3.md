Ticket: catalog-the-kani-verification-research-and-spike
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/catalog-the-kani-verification-research-and-spike/090833eebbfb_c99ac54950f2.md
Pre-edit content hash (from ledger): 090833eebbfbd4309fd531fbf49b8d63aaadb8bf5589d0daa21a001f02d6682e
Post-edit content hash: f219468fc3bf732c85b1c184ce75983432ad71514c3a9b5e694525850794205a

Changes applied:
  - Optional per-Fact hygiene only (report required none on this ticket): added **Correction — 2026-08-10.** after the 2026-08-08 per-Fact table, striking the live reading of `make-the-research-catalog-generated-or-stop-claiming-it-is` "is open on exactly this wording". Sibling is `status: done`; hand-maintained catalog claim and the **false** "generated research catalog" verdict stand. Historical table cell left intact.

Optional items skipped (with reason):
  - none (optional "is open" hygiene applied)

Residuals not applied (docs/crates/new tickets/authority):
  - docs/research/verification/kani-bounded-encoder-verification.md — Deferred **Catalog rows** bullet still claims non-reachability / carrier-owed; false after rows landed. Scope research/verification; docs/ out of wave B ticket-only edit.
  - tickets/spike-kani-bounded-verification-on-one-inexhaustible-encoder.md — Outcome present tense "Catalog rows are owed and filed as …" after carrier done. Other ticket; wave forbids touching it.

Verification:
  - files read:
    - full audit report 090833eebbfb_c99ac54950f2.md
    - full tickets/catalog-the-kani-verification-research-and-spike.md
    - tickets/make-the-research-catalog-generated-or-stop-claiming-it-is.md frontmatter (status: done)
  - checks:
    - report Repair required: metadata none; prose none required; dated correction none required; remainder is docs + other ticket only
    - shasum -a 256 tickets/catalog-the-kani-verification-research-and-spike.md → f219468fc3bf732c85b1c184ce75983432ad71514c3a9b5e694525850794205a
    - correction block present; historical "is open" cell retained with dated strike

Recommended next ledger state:
  integrated
