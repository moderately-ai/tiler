Ticket: apply-the-declared-numerical-conformance-on-every-reference-evaluation-path
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/apply-the-declared-numerical-conformance-on-every-reference-evaluation-path/4a49d3c97515_c99ac54950f2.md
Pre-edit content hash (from ledger): 4a49d3c97515944219ef02cc1e77d70830f1d77ab694033c6904421d8583dbc4
Post-edit content hash: 6da6720fef0c138542b6b0b42f53470c4a8db9d6d536ab09b584e1b98a8ff125

Changes applied:
  - Outcome: added **Correction — 2026-08-10** after the SiLU 2026-08-06 update: BF16 multiply/add "out of reach / deferred" marked landing-day history; carry-a-bf16-subnormal-realization-the-reference-can-be-told delivered Bf16SubnormalRealization (status done); constant-bf16 remains documented immune; gather-f32 noted as documented-immune under structural transport (omitted from exhaustive table at landing).
  - No metadata changes (status done, related list already includes carry-a-bf16-… and remains sensible).

Optional items skipped (with reason):
  - None; optional gather-f32 exhaustiveness note was cheap table hygiene and was included in the same dated correction.

Residuals not applied (docs/crates/new tickets/authority):
  - crates/tiler-reference/src/silu.rs rustdoc still narrates the pre-repair SILU_F32_FACT_SUBNORMALS claim ("declared operation fact does not cover the case that reaches them") — product path, out of wave B ticket-only scope.
  - Public _under surface remains labelled draft awaiting Tom acceptance (already parked in Outcome; not a new remainder).

Verification:
  - files read: audit report; full ticket; carry-a-bf16-… frontmatter status: done; bf16.rs Bf16SubnormalRealization; standard.rs header multiply-bf16/add-bf16 applies + constant-bf16 immune; structural.rs gather-f32 docs (conformance not read with the five transports); sample repair note format.
  - checks: required dated Outcome correction applied; optional gather note applied; no docs/crates edits.

Recommended next ledger state:
  integrated
