Ticket: spike-kani-bounded-verification-on-one-inexhaustible-encoder
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/spike-kani-bounded-verification-on-one-inexhaustible-encoder/664eff44c617_c99ac54950f2.md
Pre-edit content hash (from ledger): 664eff44c617067d778f8a6551714564b7466c0efe0225e22729f041de0c6096
Post-edit content hash: 022cb626012fe8c3e1ad361cbc75c3751593c21d063dfa50b0772a154199f95c

Changes applied:
  - Required prose: Outcome — done replaced false live "Filed as the next bounded experiment." with "Not attempted here; the obvious next bounded experiment." (matches research record / worker Outcome).
  - Required remainder: filed `spike-kani-push-slice-framing-over-a-symbolic-byte-run` (`status: todo`, scopes `research/verification`) for `push_slice` framing over a symbolic byte run; wired in parent `related`.
  - **Correction — 2026-08-10.** documents the false close-time "Filed" claim, distinguishes catalog (actually filed, now done) from the string-decomposition experiment (not filed until this repair).
  - Optional maintenance remainder: filed `resync-the-kani-encoder-injectivity-shim-after-index-arithmetic` (`status: todo`) for post-`IndexArithmetic` shim re-sync + guard/proof re-run; wired in parent `related`.
  - Optional dated guard-drift correction on parent: `./guard.sh` fails on `ResourceRequirements` / `push_resources` at audit base and current tree; complete resources proofs no longer attach to live sources.
  - Optional graph hygiene: `related` also lists `catalog-the-kani-verification-research-and-spike` (reverse edge; catalog already related this ticket one-way).
  - Status/deps/scopes unchanged (`done`; dependency on native exhaustibility sweep retained).

Optional items skipped (with reason):
  - none (optional guard-drift dated correction and re-sync remainder both applied as cheap same-ticket honesty + Class D remainder filing).

Residuals not applied (docs/crates/new tickets/authority):
  - No crates/ or spike source edits (product re-sync is owned by the new todo ticket, not this wave).
  - No docs/research write-up edits (research record already says "not attempted here"; disposition still pending).
  - Classification of complete-copy proofs vs `SoundProof`-with-bound still open (`disposition: "pending"`).
  - Primary in-crate Kani path still blocked on toolchain convergence (re-probe command unchanged).

Verification:
  - files read:
    - full audit report `664eff44c617_c99ac54950f2.md`
    - full tickets/spike-kani-bounded-verification-on-one-inexhaustible-encoder.md (pre/post)
    - tickets/catalog-the-kani-verification-research-and-spike.md (frontmatter + related)
    - docs/research/verification/kani-bounded-encoder-verification.md (string-encoder / push_slice sections)
    - rust-toolchain.toml channel pin
    - crates/tiler-ir/src/schedule/model.rs (IndexArithmetic on ResourceRequirements)
  - checks:
    - `./spikes/verification/kani-encoder-injectivity/guard.sh` → exit 1, 2/28 drifted (ResourceRequirements, push_resources)
    - `rg 'Filed as the next bounded experiment\.' tickets/spike-kani-bounded-verification-on-one-inexhaustible-encoder.md` → only inside the Correction quote
    - new remainder tickets exist with `status: todo` and reverse related to parent
    - `shasum -a 256 tickets/spike-kani-bounded-verification-on-one-inexhaustible-encoder.md` → 022cb626012fe8c3e1ad361cbc75c3751593c21d063dfa50b0772a154199f95c

Recommended next ledger state:
  integrated
