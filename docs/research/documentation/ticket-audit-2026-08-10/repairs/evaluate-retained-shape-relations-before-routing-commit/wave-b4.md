Ticket: evaluate-retained-shape-relations-before-routing-commit
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/evaluate-retained-shape-relations-before-routing-commit/eadf49558869_c99ac54950f2.md
Pre-edit content hash (from ledger): eadf4955886961eb2bdf5098ae1a5368d1a168f7c16cfdeaec9fe29a8cf619c1
Post-edit content hash: c398a2bd42e9e3cdf5864a5bec10a57d57e8bab52f8dd0f0ae4bca1ea6fcb2d2

Changes applied:
  - Moved `bind-repeated-invocations-over-caller-retained-tensors` from `dependencies` to `related` (complementary multi-extent packaging; not a hard prerequisite for evaluating already-bound input extents against retained constraints).
  - Removed `contracts/integrations` from `scopes` (ticket names no `docs/integration/**` edit).
  - Added `related` edges to symbolic packaging / C1 path predecessors: `construct-a-symbolic-region-as-a-semantic-program`, `admit-symbolic-extents-at-the-compiler-request-boundary`, `deliver-an-artifact-family-from-a-symbolic-region`.
  - Rewrote User-visible outcome to match L5 Case 2: refuse mutually inconsistent retained extent bindings; do not claim content-stale detection or capacity-vs-extent confusion as the oracle.
  - Narrowed Inference from "only layer that closes any part of L5's stale-binding case" to "only Tiler layer that closes mutual extent inconsistency"; content staleness remains a consumer obligation.
  - Added Fact paragraphs for present packaging omission (`SymbolicSemanticInterface` / empty-env envelope) and present term binding (`BindingSource::InputDimension` / `AbiRoot::InputExtent` / `bind_input_extent`).
  - Required work: packaging/identity step for retained relations called out as whole-ledger work or stop for Tom; term binding sources named.
  - Required evidence: synthetic retained-relation fixture may discharge preflight unit negatives without full symbolic compile-through.
  - Dated **Correction — 2026-08-10** block recording L5 Case 2 and graph repairs.

Optional items skipped (with reason):
  - none; optional dated correction applied as part of the required prose honesty repair.

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation of envelope carry + preflight evaluation (crates/tiler-artifact, crates/tiler-runtime, docs/artifact-abi.md, pins) — out of wave B ticket-only scope.
  - Exact retained-relation encoding design (full ShapeEnv fifth subject vs narrower projection) remains Tom-facing; ticket already stops for Tom on consequential public/schema choice.
  - No remainder ticket filed: packaging + preflight stay one ticket per report ("none mandatory if packaging + preflight stay one ticket").

Verification:
  - files read:
    - tickets/evaluate-retained-shape-relations-before-routing-commit.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/evaluate-retained-shape-relations-before-routing-commit/eadf49558869_c99ac54950f2.md
    - docs/research/runtime/autoregressive-state-and-kv-cache.md (Case 2 mutual-inconsistency vs content staleness)
    - tickets/bind-repeated-invocations-over-caller-retained-tensors.md (frontmatter related already lists this ticket)
    - tickets/admit-symbolic-extents-at-the-compiler-request-boundary.md, deliver-an-artifact-family-from-a-symbolic-region.md, construct-a-symbolic-region-as-a-semantic-program.md, fold-the-shape-environment-into-semantic-identity.md (ids/status)
    - grep: SymbolicSemanticInterface / shape environment omitted / BindingSource::InputDimension / AbiRoot::InputExtent
  - checks:
    - frontmatter: dependencies omit bind-repeated; related includes bind-repeated + three symbolic packaging ids; scopes omit contracts/integrations
    - body anchors: "mutually inconsistent", "present packaging", "Correction — 2026-08-10", synthetic fixture evidence clause
    - shasum -a 256 tickets/evaluate-retained-shape-relations-before-routing-commit.md → c398a2bd42e9e3cdf5864a5bec10a57d57e8bab52f8dd0f0ae4bca1ea6fcb2d2

Recommended next ledger state:
  integrated
