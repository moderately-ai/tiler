# Quiet-window run instructions — 2026-08-18 saturated-cost re-measurement

Prepared under the accepted (R, R) disposition of `resolve-the-retained-metal-profile-measurement-invocation-authority` (standing measurement authorization, Tom, 2026-08-18). The pipeline is proven: the retained [`smoke.txt`](smoke.txt) in this directory shows all 92 cells compiling through the production `CompileRequest`, retaining all three alternatives, matching the capped-tree stage model, and verifying against the oracle, untimed, on this host at build `26A5406e`. **Only the timed sweep remains, and it is load-sensitive: run it exactly as below, in a quiet window, and nowhere else.**

## Preconditions — check every one before starting

1. **Host and environment attestation.** `sw_vers -buildVersion` must report `26A5406e` and `sysctl -n machdep.cpu.brand_string` must report `Apple M4 Max`. Under `DEVELOPER_DIR=/Applications/Xcode.app`: `xcrun --sdk macosx metal --version` must report `Apple metal version 32023.883 (metalfe-32023.883)`, `xcrun --sdk macosx metallib --version` must report `AIR-LLD 32023.883`, `xcodebuild -version` must report Xcode 26.6 build `17F113`, and the SDK must resolve to 26.5 build `25F70`. If the OS build has moved past `26A5406e`, STOP: this directory's name and the smoke record no longer describe the execution environment, and the session's environment declaration must be redone under the new build (new dated directory, fresh smoke).
2. **Sole occupancy.** Drain every other agent lane; dispatch nothing for the duration. No builds, gates, or background sweeps anywhere on the host. The retained 2026-08-07 run recorded `host.load_before 2.93 3.09 4.46` as this machine's idle desktop; require `sysctl -n vm.loadavg` to be in that band (1-minute load roughly at or below 3.5, and certainly not the 6–10 seen during live lane activity) before starting. The harness records load before and after; a run whose recorded loads are materially above the retained band is contaminated and must not be retained as the timed record.
3. **Exact source.** The worktree must be at the session commit recorded in the ticket (branch `tkt/metal-remeasure-session`), clean. Verify the harness hashes match this directory's `environment.tsv` pins once assembled — the committed `src/main.rs` must hash `ca32f6b049a15137dd20d18920c931f7561b0f745b99c9520b0cae178e816844` and `src/model.rs` must hash `f7da070d81a31b9fdb69a74f8ae64c1147242175f274c00a5147bbd7088577c0` (`shasum -a 256 src/main.rs src/model.rs`). If they do not match, the run is not the reviewed harness: STOP.

## The run

```sh
cd spikes/program-planning/reduction-dispatch-crossover
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  cargo run --release --bin reduction-dispatch-sweep \
  > results/2026-08-18-apple-m4-max-macos27.0-26A5406e/sweep.tsv
```

Frozen protocol parameters (compiled into the harness; verify they appear in the sweep header): `warmup 8`, `repetitions 30`, `batch 16`, contract `FLUSH_AND_REASSOCIATE_F32`, declaration `BoundMetalCompileDeclaration::first_macos_apple9`, 92 cells / 276 dispatched alternatives, per-plan estimator `(batched - single) / 15` on interleaved rotated rounds, minimum/median/p90/stddev at both encode counts. The header must also carry `selection.compile_flags -target air64-apple-macos26.0 -std=metal4.0 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off` and `selection.link_flag_count 0` — these are printed from the executing request and resolved toolchain, and a header that says anything else means the wrong toolchain answered: discard the run.

## After the sweep

```sh
cargo run --release --bin reduction-cost-fit -- \
  results/2026-08-18-apple-m4-max-macos27.0-26A5406e/sweep.tsv \
  > results/2026-08-18-apple-m4-max-macos27.0-26A5406e/calibration.txt
# Mutation proof, same four perturbations as the retained 2026-08-07 record:
for p in "parallel 0.25" "parallel 4" "encoder 20" "step 0.1"; do
  cargo run --release --bin reduction-cost-fit -- \
    results/2026-08-18-apple-m4-max-macos27.0-26A5406e/sweep.tsv --perturb $p
done > results/2026-08-18-apple-m4-max-macos27.0-26A5406e/perturbations.txt
```

**Stop condition (part of the accepted option):** if the fitted model loses its held-out serial-versus-parallel separation — the separated held-out cells no longer agree beyond what the perturbations distinguish, or the crossover contour vanishes — the cost row is **withdrawn**, not shipped: a selector that cannot select carries no authority. Record the failure and its numbers; do not tune parameters or re-run until a number looks better. A single contaminated run may be re-run once in a genuinely quiet window; a clean run whose fit fails its stop condition is a result, not a retry.

## Custody

Assemble `environment.tsv` in this directory in the retained 2026-08-07 record's key/value format: date_utc, os product/version/build (`26A5406e` expected), architecture, device, logical cores, developer_dir, xcode, sdk version/build, offline compiler/linker, default_developer_dir, rustc, toolchain, `environment.repository_base` (the session commit the run's worktree was at), `sweep.harness_sha256` (of `src/main.rs`, must equal the pin above), `model.harness_sha256` (of `src/model.rs`), `fit.harness_sha256` (of `src/fit.rs`, `45cfa10bd45a69bdb448e1a632bf8e2a8f85f024ab5536f935842b197f27da0b`), `sweep.sha256` (of the produced `sweep.tsv`), `host.occupancy` (state the drain), and `host.load_before` / `host.load_after` copied from the sweep's own header rows. Commit `sweep.tsv`, `calibration.txt`, `perturbations.txt`, and `environment.tsv` together, then verify every pinned hash against the committed blobs (`git show <commit>:<path> | shasum -a 256`). Ledger third-environment tables, profile-row updates, and pin recomputation belong to the implementation carrier, not to this run.
