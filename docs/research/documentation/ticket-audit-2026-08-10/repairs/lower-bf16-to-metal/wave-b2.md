Ticket: lower-bf16-to-metal
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/lower-bf16-to-metal/dd53292677bd_c99ac54950f2.md
Pre-edit content hash (from ledger): dd53292677bdd3fc0b34bfe6910631347a0450d01899c54308b6a69c2cce7d3a
Post-edit content hash: 7d055eae01e191c90d6f1d6fb0abda71ed1fbb8437df79d64f79953621b55f9e

Changes applied:
  - Outcome offline-measurement pin: replaced bare `metal_declaration.rs:281` with searchable anchors `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` / `requested_target air64-apple-macos26.0` in `crates/tiler-build/src/metal_declaration.rs`.
  - Outcome derivation: restated `metal.workspace` consumers as `serial-sum-run` and `tiler-conformance` (not sole `serial-sum-run`); scope argument unchanged.
  - Outcome / helper section: replaced bare harness and type line pins with searchable fragments (`canonicalizer`, `BF16 = BrainFloat(`, `_ZL32tiler_canonicalize_nan_bf16_7fc0DF16b`, `pub enum DTypeDispatchability`, `pub struct ReferenceNumericalConformance`).
  - Optional clarity: dated **Correction — 2026-08-10** under User-visible outcome and Closes when superseding dispatch clauses to the 2026-08-06 offline boundary and done dependents.
  - `## Fact audit — 2026-08-10` recording the two live-false Outcome citations and the pin refresh; status/graph left as `done` with no new remainder.
  - Metadata unchanged (status, deps, related, scopes OK per report).

Optional items skipped (with reason):
  - none; optional Closes when / User-visible supersession applied as cheap same-ticket clarity.

Residuals not applied (docs/crates/new tickets/authority):
  - none. Report listed ticket-only prose; re-measurement of offline golden library symbol not required; no new remainder tickets.

Verification:
  - files read:
    - tickets/lower-bf16-to-metal.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/lower-bf16-to-metal/dd53292677bd_c99ac54950f2.md (full)
    - crates/tiler-build/src/metal_declaration.rs (profile key / air64 anchors via rg)
    - crates/tiler-compiler/src/target.rs, crates/tiler-reference/src/conformance.rs (symbol sites via rg)
    - spikes/apple-targets/numerical_probe.py, test_numerical_probe.py (canonicalizer / mangled pin via rg)
    - Cargo.toml consumers of `^metal\.workspace` (serial-sum-run, tiler-conformance)
  - checks:
    - `rg -n 'air64-apple-macos26.0|msl4-0.f32-bf16' crates/tiler-build/src/metal_declaration.rs` — profile key and requested_target present
    - `rg -n 'pub enum DTypeDispatchability|pub struct ReferenceNumericalConformance'` — symbols present
    - `rg -n '^metal\.workspace' --glob 'Cargo.toml'` — two consumers
    - bare Outcome line pins `:281` / `:1415` / `:123` / `:587` / `:1031` gone from live claims (retired wording only inside Fact audit)
    - post-edit `shasum -a 256` of ticket file

Recommended next ledger state:
  integrated
