Ticket: pin-the-differing-identity-positions-beside-the-carrier-positions-constant
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/pin-the-differing-identity-positions-beside-the-carrier-positions-constant/810f756c124a_c99ac54950f2.md
Pre-edit content hash (from ledger): 810f756c124ab6784a200b397403fdf8edc37cb4143171287a1431385a97c239
Post-edit content hash: ba3692751ce605a84fa1a44ad076f44e0a7a315fdba35228398a5b50762bddb8

Changes applied:
  - Added `## Outcome` recording landing `b03f2b81`, both constants, length-equality-then-count assertions, separate-subject rule, offsets unpinned, and the two watched-failing perturbation texts (`left: 5, right: 4` count; `left: 40133, right: 40132` lengths).
  - Rephrased open work-order body to past/historical tense ("The open proposal was" / "What closed this") while preserving design Facts.
  - Noted contracts were out of scope for this ticket; status stays `done`.
  - Filed remainder `tickets/retire-the-false-unasserted-identity-difference-prose-after-the-pin.md` (`todo`, scopes `contracts/artifacts` + `contracts/navigation`) covering the three live doc sites; wired it into `related`.

Optional items skipped (with reason):
  - none (optional contracts-out-of-scope note was applied inside Outcome; no dependency/related optional-only hygiene beyond wiring the new remainder).

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/artifact-abi.md`: still carries "What is left unpinned…" (`and no test asserts either.`) and Measurement "differ at exactly four byte positions" — owned by the new remainder, not edited in wave B.
  - `docs/dtype-support.md`: still carries "nothing asserts it either" for the four-byte identity difference — same remainder.
  - No crate edits (pin already present; Class E/code residual none for this wave).

Verification:
  - files read: audit report; full ticket (pre-edit); `crates/tiler-artifact/src/program/codec/tests.rs` pin constants and `a_bf16_artifact_round_trips_and_its_carrier_enters_identity`; greps on `docs/artifact-abi.md` / `docs/dtype-support.md` unasserted anchors; `git show b03f2b81` commit message for perturbation texts; `ticketsplease.toml` ownership of both docs; sibling related tickets' post-pin corrections.
  - checks: `rg DIFFERING_IDENTITY_POSITIONS crates/tiler-artifact/src/program/codec/tests.rs` — declaration + assert; `rg 'no test asserts either|nothing asserts it either|What is left unpinned' docs/artifact-abi.md docs/dtype-support.md` — residual live; `shasum -a 256` on edited ticket.

Recommended next ledger state:
  integrated
