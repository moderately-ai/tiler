Ticket: enforce-resolved-encoded-value-binding-conformance
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/enforce-resolved-encoded-value-binding-conformance/8aea8d101015_c99ac54950f2.md
Pre-edit content hash (from ledger): 8aea8d1010156caafa88687ffb91c20c29175ffeca5621b298d3e93e40fecb4f
Post-edit content hash: 827ee5a5b94298567b33dcd20c18cc704349824491b0715f1e1c7e8995951ec9

Changes applied:
  - Implementation keys: "positive finite f32" → "positive normal f32 (`positive-normal-f32` / `ENCODED_NUMERIC_SCALE_DOMAIN`)".
  - Adversarial evidence: "Smallest positive f32 subnormal scale passes; …" → smallest positive normal (`f32::MIN_POSITIVE`) passes; every positive subnormal and the other non-domain classes fail.
  - Outcome unsupported-representation count: "five classes" + separate Nested constructor sentence → six `UnsupportedValueRepresentation` variants, Nested constructor-primary with defensive derive arm.
  - Added **Correction — 2026-08-10.** under Outcome recording that the adversarial/implementation-keys scale domain was wrong relative to the normal-domain contract accepted 2026-08-04.
  - related hygiene: dropped `produce-typed-strict-affine-assemble-semantic-precondition` (already in `dependencies`).

Optional items skipped (with reason):
  - none (optional related hygiene and five→six precision were both applied).

Residuals not applied (docs/crates/new tickets/authority):
  - none required. Report: no new remainder tickets; runtime binding and physical tail already owned elsewhere; Exact files listed ticket only. Historical "seventeen fault-proofs" process claim and artifact-domain counterfactual narrative left as residual uncertainty per audit (not repair-required prose).

Verification:
  - files read:
    - tickets/enforce-resolved-encoded-value-binding-conformance.md (full, pre and post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/enforce-resolved-encoded-value-binding-conformance/8aea8d101015_c99ac54950f2.md (full)
    - crates/tiler-ir/src/semantic/conformance.rs (`PositiveNormalF32` / `positive-normal-f32` check_scalar arm; `UnsupportedValueRepresentation` six variants including `NestedComponent`)
    - crates/tiler-ir/src/semantic/conformance/tests.rs (scale-class admitted/refused table anchors)
  - checks:
    - `rg` post-edit: no remaining "positive finite"; adversarial line uses MIN_POSITIVE / subnormal fails; Correction dated 2026-08-10 present; related no longer duplicates dependency
    - post-edit `shasum -a 256` of ticket file

Recommended next ledger state:
  integrated
