Ticket: compile-an-elementary-function-golden-through-the-metal-toolchain
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/compile-an-elementary-function-golden-through-the-metal-toolchain/a5c93b0db98e_c99ac54950f2.md
Pre-edit content hash (from ledger): a5c93b0db98e711b78232bf690d9125cee154ea24017bf31a844d69327001838
Post-edit content hash: ec24036ef5ea759c8acadfa69369f3c33439530718369e2602d62fd1dde166d8

Changes applied:
  - Required: dated Correction — 2026-08-10 on Outcome § Measurement linked-library symbol — landing pin `tiler_kernel_b1e08c4feb69be47` kept as historical; live golden entry point is `tiler_kernel_474c1b875639dceb` (`kernel identity digest: 474c1b875639dceb`).
  - Optional (cheap, same ticket): tightened "every other golden's arithmetic is `*`/`+`/comparison" to float/elementary scope (integer `/` and `-` remain elsewhere); landing wording marked overstated.
  - Optional (cheap, same ticket): dated Correction that `FpContract::FastHonorPragmas` residual is closed — enum is only `Off | On | Fast`; driver watches string rejection of `fast-honor-pragmas`.

Optional items skipped (with reason):
  - none (both optional Outcome tightenings applied)

Residuals not applied (docs/crates/new tickets/authority):
  - `crates/tiler-metal/src/golden_compilation.rs` elementary measurement paragraph still names `tiler_kernel_b1e08c4feb69be47` (product path; out of ticket-only wave)
  - `docs/roadmap.md` SiLU cell still cites `b1e08c4feb69be47` via navigation move evidence (`contracts/navigation`; not this ticket's scope)
  - Linked library byte sizes / AIR intrinsic names not re-measured
  - Why-this-exists pre-delivery Facts left as historical problem statement (status done; Outcome is the delivery record; report did not require striking them)

Verification:
  - files read: full audit report; full ticket; elementary_silu_activation.metal entry point; FpContract in tiler-metal-aot input.rs; golden_compilation.rs b1e08c pin / GOLDENS comment; house-style Correction samples in other tickets
  - checks: golden has `tiler_kernel_474c1b875639dceb`; FpContract variants Off/On/Fast only (no FastHonorPragmas); shasum -a 256 post-edit

Recommended next ledger state:
  integrated
