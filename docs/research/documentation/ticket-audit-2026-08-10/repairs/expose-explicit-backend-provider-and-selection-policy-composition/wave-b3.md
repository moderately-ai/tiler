Ticket: expose-explicit-backend-provider-and-selection-policy-composition
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/expose-explicit-backend-provider-and-selection-policy-composition/0ffad0ffa464_c99ac54950f2.md
Pre-edit content hash (from ledger): 0ffad0ffa464f0f19087e5bf3e545064b6a37157737b7577a87868c1ad8cc5a7
Post-edit content hash: 9ecb52be10cdde3370b9709f0c9c7a685f31184f2d09fd94e6fb2c4bc7869ae8

Changes applied:
  - Rewrote **Closes when** to match the 2026-08-09 Outcome: close on delivered per-responsibility composition seams, governed cost identity, installation refusals, absence of global/bundle registries, and successful split of family-policy + multi-backend examples onto named successors — not on this ticket producing “all example policies.”
  - In the freeze/offered_providers Implementation key, replaced “documented-versus-actual subject gap” with a **Correction — 2026-08-10.**: documentation half is discharged (`offered_providers` is lowering-only and points at `offered_physical_providers`); remaining remainder is durable artifact `CompilationEnvironment` still minted from lowering providers alone (owned by disclose-the-physical-provider-environment-a-compilation-was-offered).
  - Added a one-line **Correction — 2026-08-10.** under Outcome audit noting Closes when was rewritten for terminal honesty.

Optional items skipped (with reason):
  - none beyond the optional dated note, which was applied as cheap hygiene with the Closes when rewrite.

Residuals not applied (docs/crates/new tickets/authority):
  - none required; report listed no docs/crates edits and no new remainder tickets. Policy, examples, and public-surface acceptance remain on their existing successor tickets.

Verification:
  - files read:
    - tickets/expose-explicit-backend-provider-and-selection-policy-composition.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/expose-explicit-backend-provider-and-selection-policy-composition/0ffad0ffa464_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/session.rs (`offered_providers` / `offered_physical_providers` docs; artifact provenance still from offered_providers alone)
    - crates/tiler-build/src/plan_artifact.rs (`CompilationEnvironment::new(compilation.offered_providers().iter().cloned())`)
  - checks:
    - Closes when no longer requires “all example policies” on this parent
    - freeze key no longer labels the remainder as documented-versus-actual
    - shasum -a 256 of ticket after edit → post-edit hash above

Recommended next ledger state:
  integrated
