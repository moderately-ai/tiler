Ticket: retain-the-c1-attention-block-conformance-evidence
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/retain-the-c1-attention-block-conformance-evidence/7c993657cc77_c99ac54950f2.md
Pre-edit content hash (from ledger): 7c993657cc771477e8a60d8b66367839e877ff0e1fe0dfa6919d9c78bc0ec33f
Post-edit content hash: b07dc37c101286b8dfa1dd5e4eedeb58b8f2dda4a09120ba6cff33a5f1eb9bfb

Changes applied:
  - Why three cases / item 3: dated Correction — 2026-08-10 replaces bare "four-wide worked example sums to 0x3f7ffffe" with post-2026-08-01 L3′ split (reference model 0x3f7ffffe vs pinned formula 0x3f800000 on that row); grounds "assert neither" on C1 111/49 census and L3′ widths [0.0, 2.0] → 0x3f7fffff / [0.0, 1.0, 0.0] → 0x3f800001.
  - Required delivery: dated Correction — 2026-08-10 (partial delivery census) records already-landed host tests for the three discriminating cases and rotary/GQA structural perturbations (assemble / admit-softmax / compose-rotary / admit-gqa); names residual as (a) SHA-256 digests with complete realization boundary after integrate, (b) B1 direct↔tiled cross-check, (c) durable design→retained-evidence link — not a second copy of green host cases unless deliberate consolidation.
  - Closes when: dated Correction — 2026-08-10 notes discriminating-case and structural-perturbation conjuncts already satisfied; status remains todo for residual digests / cross-check / design link and open integrate dependency.
  - Metadata unchanged (status todo, dependencies, related, scopes, tags, priority all hold per report).

Optional items skipped (with reason):
  - none (report optional was packaging choice for host-case consolidation — left as residual uncertainty, not a prose repair; no new remainder ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - product residual: SHA-256 digests of three C1 block outputs with host/toolchain/numerical-realization/schedule boundary; B1 direct↔tiled bit cross-check; durable design→retained-evidence link (out of ticket-only wave; blocked on integrate-the-attention-block-into-the-runtime).
  - no new remainder ticket filed (upstream integrate already owns device path).
  - optional consolidation of scattered host cases into one attention_block_conformance module is an in-scope future delivery choice, not graph work.

Verification:
  - files read:
    - full audit report 7c993657cc77_c99ac54950f2.md
    - full ticket pre- and post-edit
    - docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md (reference vs pinned formula sum split; 0x3f7fffff / 0x3f800001 widths)
    - crates/tiler-reference test anchors for signed zero, fully-masked, row-sum, rotary 20_480, GQA 17_920
  - checks:
    - git show HEAD:ticket sha256 matched ledger pin 7c993657cc771477e8a60d8b66367839e877ff0e1fe0dfa6919d9c78bc0ec33f
    - post-edit shasum -a 256: b07dc37c101286b8dfa1dd5e4eedeb58b8f2dda4a09120ba6cff33a5f1eb9bfb
    - rg confirmed L3′ correction sentence and named test symbols still present

Recommended next ledger state:
  integrated
