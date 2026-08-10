Ticket: correct-the-residual-floor-relation-prose-outside-the-artifact-scopes
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-residual-floor-relation-prose-outside-the-artifact-scopes/4c7d43f6d686_c99ac54950f2.md
Pre-edit content hash (from ledger): 4c7d43f6d686d73e4fe2aef74b0201f3099e0187e3c90361f67176f723f092b0
Post-edit content hash: d8d594212f509b57d8ff5a7b7ab41e57c764226319b4752cd1eb5751f14f49f7

Changes applied:
  - What closes this: **Correction — 2026-08-10.** absolute "any research record" close wording narrowed to held trees (decisions + research/scheduling + research/extensions) plus intentional drafted-span exception; names done runtime residual and new research/runtime remainder
  - Outcome residual Decision: **Correction — 2026-08-10.** records that runtime residual is done; research/runtime backend-scoped record floor/capacity/`floor.is_satisfied_by` live assertions filed as remainder; status done retained for held work
  - ## Fact audit — 2026-08-10: close-condition imprecision, equality source, runtime done, research/runtime remainder still live
  - related: linked correct-the-runtime-route-requirement-relation-prose and correct-the-floor-relation-prose-in-the-backend-scoped-route-answer-record (status/scopes/dependencies unchanged)
  - Class D remainder filed: tickets/correct-the-floor-relation-prose-in-the-backend-scoped-route-answer-record.md (todo, research/runtime only; anchors verdict for a floor, capacity comparison, floor.is_satisfied_by / stale requirement.rs pins; align to equality/required/loader-owned without forking ADR 0090 item 4)

Optional items skipped (with reason):
  - none (optional related link applied with the new remainder)

Residuals not applied (docs/crates/new tickets/authority):
  - docs/research/runtime/backend-scoped-route-requirement-answers.md live floor/capacity/`floor.is_satisfied_by` sentences remain product/docs debt for the remainder ticket (not edited in this wave)
  - Outcome line numbers (ADR 0090 :69 etc.) left as delivery-base anchors; content anchors still resolve; report did not require rewriting historical line pins on this done ticket

Verification:
  - files read:
    - full audit report 4c7d43f6d686_c99ac54950f2.md
    - full parent ticket pre/post
    - docs/research/runtime/backend-scoped-route-requirement-answers.md shape bullet + ADR 0090 conflict paragraph (anchors)
    - crates/tiler-artifact/src/program/requirement.rs equality arm / floor-reject heading (grep)
    - correct-the-runtime-route-requirement-relation-prose.md (frontmatter + Outcome head; status done)
  - checks:
    - rg floor.is_satisfied_by|verdict for a floor on backend-scoped record → still live (remainder subject)
    - requirement.rs SubgroupThreads => self.required == observed present
    - parent status remains done; remainder status todo scopes research/runtime
    - shasum -a 256 parent → d8d594212f509b57d8ff5a7b7ab41e57c764226319b4752cd1eb5751f14f49f7

Recommended next ledger state:
  integrated
