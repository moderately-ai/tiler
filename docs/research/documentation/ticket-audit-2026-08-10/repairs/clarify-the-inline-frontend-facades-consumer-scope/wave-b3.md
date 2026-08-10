Ticket: clarify-the-inline-frontend-facades-consumer-scope
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/clarify-the-inline-frontend-facades-consumer-scope/2ae8b7bbf01c_c99ac54950f2.md
Pre-edit content hash (from ledger): 2ae8b7bbf01cd9196221a1fad780e783376be34be1efce412e5a75c7b9060218
Post-edit content hash: 6d4906306e759c564dc90017fbc2d91acd624a98eecb373793f9308c3f9c5676

Changes applied:
  - Outcome Fact "the pending boundary is described, not decided": replaced bare `docs/correctness-and-testing.md:117` with a searchable multi-output / facade-acceptance paragraph description; clarified that this ticket accepted no interface.
  - Added **Correction — 2026-08-10** stating that after 2026-08-06 `tiler_compiler::session` is accepted (acceptance ≠ stabilization) per `accept-the-public-compiler-facade-boundary` and `session.rs` / correctness accepted-in-full language; the ticket's delivered "reviewed experimental draft / has not been decided" wording is historical and later went stale in the frontend crate headers.
  - Outcome Fact "Markdown contract already agreed": replaced bare `docs/architecture.md:331` / `:424` with searchable anchors (`The inline Rust frontend's import path` and `does not make `tiler` the accepted facade…`); scoped the agreement claim to the consumer-scope half.
  - Added **Correction — 2026-08-10** on the architecture packaging paragraph's present-tense "eventual coherent facade" tail as residual disclosure drift of the same class.
  - Metadata unchanged: `status: done`; dependency and related edges left as-is.

Optional items skipped (with reason):
  - none (no optional-only repair bullets beyond the residual remainder connection, which is residual product work without a concrete new ticket id in the report).

Residuals not applied (docs/crates/new tickets/authority):
  - Present-tense compiler-facade disclosure refresh in `crates/tiler/src/lib.rs` ("has not been decided" / "reviewed experimental draft rather than an accepted…").
  - Same class in `crates/tiler-macros/src/lib.rs` ("undecided boundary owned by `accept-the-public-compiler-facade-boundary`").
  - Optional same class in `docs/architecture.md` packaging paragraph tail ("their eventual coherent facade is a separate public-boundary decision").
  - No new remainder ticket filed: report asked to connect a narrow remainder or attach to the acceptance ticket's incomplete disclosure sweep but listed no concrete new ticket id; wave B3 is ticket-prose only for this path set.
  - Evidence filing Fact that "acceptance of a coherent public compiler facade remains a separate boundary" left as historical filing evidence (report required Outcome correction only).

Verification:
  - files read:
    - full audit report `…/reports/clarify-the-inline-frontend-facades-consumer-scope/2ae8b7bbf01c_c99ac54950f2.md`
    - full ticket before/after edit
    - greps on `crates/tiler`, `crates/tiler-macros`, `crates/tiler-compiler/src/session.rs`, `docs/architecture.md`, `docs/correctness-and-testing.md`, `tickets/accept-the-public-compiler-facade-boundary.md`
  - checks:
    - `rg` confirms inline-frontend opener still present; draft/undecided sentences still in both frontend crates
    - `session.rs` opens accepted; acceptance ticket `status: done`; correctness carries `the facade is accepted in full`
    - architecture anchors resolve (import-path table cell; does-not-make-facade packaging sentence)
    - post-edit `shasum -a 256` of ticket file

Recommended next ledger state:
  integrated
