---
id: restore-the-two-path-dependent-spikes-to-a-running-state
title: Restore the two path-dependent spikes to a running state
status: in-progress
priority: p2
dependencies: []
related: [reconcile-the-two-target-profile-key-grammars]
scopes: [research/target-profiles, research/cache]
shared_scopes: []
paths: []
tags: [spikes, evidence, artifacts]
claimed_from: todo
assignee: worker-restore-the-
lease_expires_at: 1785562405
---
## User-visible outcome

The two retained spikes that path-depend on `crates/` compile and run again, so the evidence they are cited for is reproducible rather than only recorded.

## Why this slice exists

**Measurement, at `c142991`.** Neither spike compiles against the tree it depends on. Both were run from their own directory with the invocation their README records, with `CARGO_TARGET_DIR` set locally.

`spikes/target-profiles/scalar-cpu-vertical` — `CARGO_TARGET_DIR=./target cargo run`:

```
error[E0533]: expected value, found struct variant `TensorRole::Input`
    --> src/vertical.rs:1462:17
```

`spikes/cache/build-tool-exercise` — `CARGO_TARGET_DIR=./target cargo check --workspace`, five errors in `envelope/src/lib.rs`:

- `no method named `first` found for struct `Compilation`` (line 63)
- `struct `BindingSpec` has no field named `accessible_bytes`` (line 162)
- `struct `LaunchSpec` has no field named `grid_threads`` (line 166) and `threads_per_workgroup` (line 167)
- `struct `VariantSpec` has no field named `applicability_guard`` (line 186)

**Fact.** The CPU vertical's break is attributable: `TensorRole::Input` gained an `ordinal` field in `2b745f3` ("Name which input tensor a scheduled region reads"), and `git merge-base --is-ancestor 2b745f3 093c23c` fails — that commit postdates `093c23c`, the last change to `vertical.rs`. So the spike was left behind by a `tiler-ir` change that no gate reached.

**Inference.** This is the failure mode `AGENTS.md` names when it says no `make` target reaches `spikes/` and only re-running one detects drift from the source beside it. It is not hypothetical: `state-an-expected-artifact-identity-from-recorded-bytes` recorded the same class of gap for the same CPU vertical and fixed it inline by adding `research/target-profiles` to its own scopes. That remedy was correct and does not scale — it depends on the worker noticing.

**Why this matters beyond tidiness.** The CPU vertical is the **Measurement** ADR 0090 cites in its context paragraph and under item 10, and its retained fixture `results/2026-07-31-macos-arm64.json` is a positive claim that outlives its producer. A spike that cannot be re-run cannot refute the fixture beside it, so the fixture silently degrades from reproducible evidence to a recorded assertion — while `experiment_status: "reproducible"` in the README frontmatter still claims otherwise.

Discovered by `reconcile-the-two-target-profile-key-grammars`, which ran both spikes to confirm that tightening the governed-key alphabet refused nothing they build. It refused nothing — both failed earlier, on unrelated API drift, and neither failure touches key validation.

## Implementation keys

- Fix the call sites against the current public API rather than pinning the spike to an older revision; the point of a path dependency is that the spike tracks the tree.
- Re-run each spike to its own stop condition after fixing, not just `cargo check`. The CPU vertical's product is a verdict and it exits non-zero with the stage named, so a compile is not a pass.
- Compare the re-run against the retained fixtures. A recorded quantity that moved is a finding, not a number to overwrite: `results/2026-07-31-macos-arm64.json` records a 797-byte profile descriptor, a 265-byte payload, a 20,327-byte envelope, and a 9,464-byte artifact identity, and any of those changing means an identity's content changed since `488efac`.
- `experiment_status` and `last_verified` in each README's frontmatter are claims about reproducibility; update them to the commit and date of the re-run.
- Keep `CARGO_TARGET_DIR` spike-local, and do not add a `rust-toolchain.toml` to either spike — the root pin is resolved by directory ancestry and a local file would silently select another compiler for the evidence.

## Closes when

Both spikes compile and complete their own run, each fixture is either reproduced or its divergence recorded as a finding, and each README's reproducibility frontmatter names the commit that verified it.
