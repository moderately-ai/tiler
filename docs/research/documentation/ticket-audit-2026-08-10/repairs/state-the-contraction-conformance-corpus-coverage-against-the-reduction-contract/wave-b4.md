Ticket: state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract/3ae4fcfb25ca_c99ac54950f2.md
Pre-edit content hash (from ledger): 3ae4fcfb25caa52cc8a86082efc1b80347a23ca3ca44cf8fc6a54de91f4ebcac
Post-edit content hash: 24447ce9ecac7e513699397f124fd3666f2269767f245a36007f8ade3466b41d

Changes applied:
  - Fact 1 realization sentence: replaced "compares them only on a matching environment row, and declines the retained comparison by name otherwise" with the actual rule (pin six direct digests; decline only on hardware device/gpu-family differences; announce toolchain fields and proceed; xcode not compared; ordinary gate one cell, four prefill cells #[ignore]).
  - Fact 2: stated the research section bullets as the ledger subject set; labelled the ticket inventory as a non-exhaustive compression; expanded inventory to include integer wrapping/saturating/checked/widening, f16/bf16 accumulate-and-finalize, empty partials with masks/has_value, every-cell preamble, and fuller axis/tree wording so workers do not under-count rows.
  - Work: pointed subject sourcing at the research section bullets; documented when to add implementation/conformance or research/numerics without inventing a single home (placement deliberately open; current scopes cover the two primary homes).

Optional items skipped (with reason):
  - Choosing a single ledger home and fixing scopes to only that path: residual uncertainty in the audit leaves placement deliberately open; inventing a home would be a product decision. Work now states the two in-scope homes and the two alternate homes that require scope adds.

Residuals not applied (docs/crates/new tickets/authority):
  - Delivering the coverage ledger and removal-sensitive check (this ticket's product outcome; wave B is ticket prose only).
  - Full census of ordinary tests already covering Required-adversarial subjects outside the eight cases (ticket deliverable, not audit repair).
  - No new remainder ticket required; retain's implementation halves stay closed.

Verification:
  - files read:
    - full audit report 3ae4fcfb25ca_c99ac54950f2.md
    - full ticket (pre-edit)
    - docs/research/numerics/reduction-semantics-and-legality.md Required adversarial tests bullets (412–434)
    - rg anchors in crates/tiler-conformance (hardware_differences, deliberately not compared, L3_CORRECTNESS_CELLS, prefill ignore)
    - rg anchors in crates/tiler-reference/tests/contraction_conformance.rs (eight named exceptional cases)
  - checks:
    - hardware decline rule matches retained_record/envelope/apple source
    - research section still lists integer/f16-bf16/empty partials/has_value subjects omitted from old Fact 2 compression
    - status/deps/related left as todo / retain / reduction-semantics-contract (correct per audit)

Recommended next ledger state:
  integrated
