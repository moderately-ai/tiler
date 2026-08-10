Ticket: declare-evaluation-order-preservation-in-the-target-profile
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/declare-evaluation-order-preservation-in-the-target-profile/c706e203c863_c99ac54950f2.md
Pre-edit content hash (from ledger): c706e203c86395a1f09e9ed5951ad45e206479b33d15971d95ba8fb0bb12bb9c
Post-edit content hash: 8002d85d7af0ca185bfa99f4d7037973d3d17e636be14c9be8d0d0eba7bde864

Changes applied:
  - related: added `accept-the-evaluation-order-preservation-target-fact` (graph symmetry with accept ticket)
  - appended `## Fact audit — 2026-08-10` with three dated corrections: (1) labelled draft / awaiting-decision is historical; surface accepted 2026-08-06, accept ticket done, source labels Accepted public surface per 2026-08-09 handoff; (2) Closes-when second clause discharged as ledger deferral — that is what done means; (3) descriptor pin 1,999 → live 2_099 from cost-row family, evaluation-order still zero when empty

Optional items skipped (with reason):
  - none (optional related edge and optional pin note both applied as cheap same-ticket hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - docs/research/reference/permitted-divergence-oracle.md Part 7 item 5 still ends with labelled-draft / "parks it for Tom" (outside this ticket's scopes; separate prose sweep)
  - oracle Part 7 roll-up table still listing the measure ticket as todo while measure is done (external graph/prose rot noted in audit residual uncertainty)
  - no new remainder ticket; row declaration stays on ledger toolchain trigger; feasibility consumer stays with deferred admit/oracle work

Verification:
  - files read:
    - tickets/declare-evaluation-order-preservation-in-the-target-profile.md (full, pre and post)
    - audit report c706e203c863_c99ac54950f2.md (full)
    - tickets/accept-the-evaluation-order-preservation-target-fact.md (frontmatter + Outcome + 2026-08-09 correction)
    - crates/tiler-build/src/metal_declaration.rs (descriptor pin 2_099 via grep)
    - crates/tiler-compiler/src/target.rs (Accepted public surface anchors via grep)
  - checks:
    - accept ticket status: done; Accepted by Tom on 2026-08-06
    - live pin: assert_eq!( descriptor.len(), 2_099 in metal_declaration.rs
    - source labels: Accepted public surface / acceptance-of-its-own anchors present
    - shasum -a 256 post-edit ticket = 8002d85d7af0ca185bfa99f4d7037973d3d17e636be14c9be8d0d0eba7bde864

Recommended next ledger state:
  integrated
