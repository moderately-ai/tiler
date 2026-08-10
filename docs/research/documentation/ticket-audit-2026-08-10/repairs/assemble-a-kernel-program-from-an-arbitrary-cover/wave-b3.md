Ticket: assemble-a-kernel-program-from-an-arbitrary-cover
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/assemble-a-kernel-program-from-an-arbitrary-cover/96c33a08f4e4_c99ac54950f2.md
Pre-edit content hash (from ledger): 96c33a08f4e46bbad2fca671a7b133e2fa518c33d48ac64d7db9313a471c77fc
Post-edit content hash: bef9dc3dfb7812d2724cc6e0fa60f265b4c75a375dda879c5c729440c57d33dc

Changes applied:
  - **Correction — 2026-08-10** on "What landed" `output_count() != 1` guard: landing-day "this ticket did not relax" kept; present-tense "is untouched" struck as live tree state; multi-output ticket later removed both arity guards; ordered key equality + `cover-named-output-attribution` remain
  - **Correction — 2026-08-10** on reachability section: Inference "no cover of more than two regions is retainable" / only fused·materialized·split live set marked false as present tense; three-region ordinary path (`outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read`); assembler remains general; four-region still not tested guarantee
  - Same reachability correction aligns short `verify_region_subject_binding` partition list with staged fold/pass, epilogue, publishing-copy, and workgroup-tree arms (optional report item, cheap same-section)

Optional items skipped (with reason):
  - Explicit HISTORICAL labels on "Why this exists" filing Facts: already pinned to `57474a09`; convention satisfied without restating

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/compiler/optimizer.md` stage-11 reachability sentence still claims longest assembled program is three-stage split — stale relative to three-region multi-output conformance; report lists as optional docs consistency, not ticket-only wave work
  - Four-region compile-path tested guarantee remains open maturity gap owned by region-vocabulary widenings; no remainder ticket required by report

Verification:
  - files read: audit report; full ticket; `crates/tiler-compiler/src/request.rs` (arity-guard history docs); `crates/tiler-compiler/src/pipeline/conformance.rs` (three-region assert); `physical.rs` binding arms; greps for `output_count() != 1`, `region_count() == 3/4`, cover-named-output-attribution
  - checks: `rg 'output_count\(\) != 1' crates/tiler-compiler` → history only; three-region assert present; no four-region assert under crates/; shasum -a 256 post-edit; status/deps/scopes unchanged

Recommended next ledger state:
  integrated
