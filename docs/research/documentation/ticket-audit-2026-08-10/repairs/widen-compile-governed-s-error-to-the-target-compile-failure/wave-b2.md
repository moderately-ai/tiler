Ticket: widen-compile-governed-s-error-to-the-target-compile-failure
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/widen-compile-governed-s-error-to-the-target-compile-failure/42fda9908d49_c99ac54950f2.md
Pre-edit content hash (from ledger): 42fda9908d49b85472be46fe5db3d55a2145bd9b50411b7eee75a784640dd571
Post-edit content hash: 47563c98fb1bfea09f8b7efb827b54c2e9066e65a251e45890a77cefd690f40e

Changes applied:
  - Rewrote the three present-tense **Fact** sentences under "## Why this exists" so they cannot be read as current defects: struck as historical at `de377fb1` / pre-landing, past tense, and headed with **Correction — 2026-08-10** stating live return type, refusal retention docs, watched equality test, and 2026-08-06 acceptance.
  - No metadata changes (status/deps/related/scopes already coherent per report).

Optional items skipped (with reason):
  - Optional dated note on Outcome 45-line / 26-call census: skipped; Outcome already dates the measurement at base `de377fb1` / landing branch; report said only needed if a reader re-runs the command at HEAD without reading the dating clause.

Residuals not applied (docs/crates/new tickets/authority):
  - none (repair was ticket prose only; report required no docs/crates edits or remainder tickets).

Verification:
  - files read:
    - tickets/widen-compile-governed-s-error-to-the-target-compile-failure.md (full)
    - reports/.../42fda9908d49_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/session.rs (compile_governed rustdoc + signature around "does not extend to the refusal")
    - grep: the_governed_convenience_entry_carries_the_same_typed_refusal_as_the_general_path in bf16_numerical_contract.rs
    - docs/correctness-and-testing.md facade acceptance sentence (via grep)
  - checks:
    - current `pub fn compile_governed` returns `Result<Compilation, TargetCompileFailure>` (no lossy `map_err(|failure| failure.failure)` tail)
    - live anchors match audit claims 4–5, 13
    - shasum -a 256 of ticket after edit → 47563c98fb1bfea09f8b7efb827b54c2e9066e65a251e45890a77cefd690f40e

Recommended next ledger state:
  integrated
