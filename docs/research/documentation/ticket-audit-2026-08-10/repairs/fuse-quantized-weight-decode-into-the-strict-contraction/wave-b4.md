Ticket: fuse-quantized-weight-decode-into-the-strict-contraction
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/fuse-quantized-weight-decode-into-the-strict-contraction/8a850663ed45_c99ac54950f2.md
Pre-edit content hash (from ledger): 8a850663ed45ce6284f3ae64a4f2e1d213a02ef807628b504b309ad113ac13b7
Post-edit content hash: 88a0ccef928bc26bbce1727b1cf3f746d132d10dd880586c5166bce3da192819

Changes applied:
  - dependencies: replaced hard dep realize-the-tiled-contraction-schedule-and-its-metal-emission with realize-the-contraction-through-the-appendable-direct-path (done); kept maps, widen, reclassify
  - related: added realize-the-tiled-contraction-schedule-and-its-metal-emission and carry-semantic-enforcement-plans-through-program-and-artifact; kept implement-first-quantized-backend-profile, admit-reassociated-contraction-schedule-alternatives, calibrate-device-cost-models, scope-first-quantized-lm-profile
  - Fact weight size 622,207,744 → 622,329,856 with L3 2026-07-31 correction anchor; kept 4,247 µs / 146 GB/s / bandwidth-bound
  - materializing slowdown "at least 2.2×" → "2.25×" to match profile arithmetic
  - emission bullet: full registered decode steps (widen code and zero point to i32; subtract; convert to f32; multiply by scale)
  - user-visible outcome: fusion into strict contraction weight operand access on landed direct schedule; tiled retained as additional host schedule when it lands; dual costed alternatives and prefill/decode selection freedom preserved
  - dated Fact audit — 2026-08-10 correction block documenting byte-count retirement, tiled→direct dependency inheritance, emission, and related carry-semantic-enforcement edge

Optional items skipped (with reason):
  - none (optional 2.25× consistency applied as cheap graph/prose hygiene on the same ticket; optional implementation/reference scope not required by report)

Residuals not applied (docs/crates/new tickets/authority):
  - product implementation remains open (no crates/docs edited): fused dequantize-into-contraction schedule/lowering, dual costed plans, bit-identity checks, Metal no-FMA assert extension
  - ParameterIndexMap public per-axis constructor remains Tom's decision on implement-workload-selected-quantized-parameter-maps
  - E-2 / device-optimal claims remain under calibrate-device-cost-models
  - no new remainder tickets (implement-first-quantized-backend-profile and carry-semantic-enforcement-plans already hang off this ticket)

Verification:
  - files read:
    - tickets/fuse-quantized-weight-decode-into-the-strict-contraction.md (pre/post)
    - audit report 8a850663ed45_c99ac54950f2.md
    - docs/research/scheduling/first-metal-contraction-realizations.md (622,329,856 / L3 correction)
    - docs/research/numerics/first-quantized-lm-profile.md (2.25×; decode evaluation)
    - crates/tiler-ir/src/semantic/quantization.rs (ENCODED_NUMERIC_DECODE_EVALUATION string via rg)
    - tickets/realize-the-contraction-through-the-appendable-direct-path.md status: done
    - tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md status: deferred
    - tickets/carry-semantic-enforcement-plans-through-program-and-artifact.md (depends on this ticket)
  - checks:
    - shasum -a 256 ticket → 88a0ccef928bc26bbce1727b1cf3f746d132d10dd880586c5166bce3da192819
    - re-verified L3 corrected byte count, profile 2.25×, registered decode evaluation, dep statuses, carry-semantic reverse edge

Recommended next ledger state:
  integrated
