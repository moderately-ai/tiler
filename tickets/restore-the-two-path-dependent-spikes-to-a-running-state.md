---
id: restore-the-two-path-dependent-spikes-to-a-running-state
title: Restore the two path-dependent spikes to a running state
status: in-progress
priority: p2
dependencies: []
related: [reconcile-the-two-target-profile-key-grammars]
scopes: [research/target-profiles, research/cache, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [spikes, evidence, artifacts]
claimed_from: todo
assignee: worker-restore-the-
lease_expires_at: 1785563637
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

## Outcome

**Scope added 2026-08-01: `research/extensions`.** The ticket was filed naming two broken spikes. A sweep of every spike workspace holding a path dependency on `crates/` found **three**, not two — `spikes/extensions/forkless-physical-provider` was broken by the same `TensorRole::Input` change and the filing worker did not see it from its own vantage. `spikes/extensions/**` maps to `research/extensions` in `ticketsplease.toml`, so the scope was added here rather than the fix deferred to a second ticket: it is the same commit, the same API change, and the same one-line repair.

The fourth path-dependent workspace, `spikes/embedding/self-contained`, is **not** broken. `cargo check --workspace` fails there by design — `embed!` refuses to expand without `TILER_EMBED_MEMBER_A`/`_B` rather than substituting a default — and its harness `self_contained.py` sets them. That is the macro's fail-closed behaviour, not drift, and it needed no change.

### Per-spike drift and repair

**`spikes/target-profiles/scalar-cpu-vertical`** — one site. `TensorRole::Input` → `TensorRole::Input { ordinal: InputOrdinal::FIRST }` in `src/vertical.rs:1462`, a synthetic `BufferParameter` used only to probe the translator's buffer refusals; `translate_buffer` never reads `parameter.tensor`, so the ordinal is inert there. Re-run with `CARGO_TARGET_DIR=./target cargo run -- results/2026-07-31-macos-arm64.json`: green, every stage and every refusal reproduced.

**`spikes/extensions/forkless-physical-provider`** — three sites in `acme-provider/src/lib.rs`: the region's one `Access`, its one `BoundsProof`, and `PointwiseF32ExpressionBuilder::input`, which gained an ordinal argument. All three take `InputOrdinal::FIRST`, which is what `crates/tiler-compiler/src/physical.rs:46` spells as `FIRST_INPUT` and what its `scale_bias_expression` passes — the spelling the request-subject binding compares against, so the mirror the spike depends on is preserved. Re-run with `cargo nextest run --workspace`: 7/7 pass.

**`spikes/cache/build-tool-exercise`** — five errors in `envelope/src/lib.rs`, a deeper change than the ordinal. `compile_governed` now returns one `Compilation` rather than a collection, and `bind-the-artifact-variant-abi-to-the-program-abi` moved the variant ABI replay *inside* `push_variant`, which now calls `adopt_abi` on the program's own arena and derives the guard, launch, and accessible extents. The producer-declared `BindingSpec::accessible_bytes`, `LaunchSpec::{grid_threads, threads_per_workgroup}`, and `VariantSpec::applicability_guard` fields are gone. The spike's `replay`, `variant_roots`, and `reachable_from` helpers were the caller-side version of exactly that replay and were deleted rather than adapted; entries now come from `program.stages()` and bindings from `stage.accesses()`, the way `crates/tiler-build/src/metal_plan.rs` does it. Re-run with `python3 spikes/cache/build_tool_exercise.py --concurrency 3 --analyzer "$(rustup which --toolchain nightly rust-analyzer)" --record macos-27.0-2026-08-01`: green.

### Retained values: one moved, two held

**Reproduced — the forkless compile-fail goldens.** All three `.stderr` files matched byte for byte with no regeneration. The API change moved no line the diagnostics point at, so nothing was blessed with `TRYBUILD=overwrite` and the falsification stands unchanged.

**Reproduced — the build-tool exercise.** Every counted quantity in all eight scenarios — events, builds, published, hit, uncached, processes, drivers — is identical to the 2026-07-25 row, including `negative-control-x3`, which is what makes the others evidence. `overlaps` and `seconds` vary run to run and are not claims; three runs at this commit gave 21, 18, and 5 overlaps for `cargo-concurrent-x3` while no counted column moved. Recorded as `results/build-tool-exercise-macos-27.0-2026-08-01.tsv` beside the 2026-07-25 fixture, which is retained.

**MOVED — four numbers in the CPU vertical's fixture.** This is the finding the ticket predicted, and it is recorded prominently in that spike's README rather than overwritten silently:

| Quantity | `488efac` | `63f9259` |
| --- | --- | --- |
| selected plan | `program-alternative:506a3f9171c1b383` | `program-alternative:5ef3467e50acb6f7` |
| envelope bytes | 20,327 | 20,953 |
| artifact identity bytes | 9,464 | 9,753 |
| reference registry identity bytes | 80,104 | 438,805 |

**No accepted record needs correcting, and that was checked rather than assumed.** ADR 0090 cites this spike for the bit-for-bit agreement of twelve `f32` elements including a negative zero, both least-magnitude subnormals, a canonicalized NaN payload, and both infinities; for the `CanonicalizeF32Nan` perturbation failing at the comparison; and for installing no physical provider. **All of those reproduced exactly** — the twelve output bit patterns are byte-identical, as are the 797-byte profile descriptor, the 265-byte payload, and the zero deferred predicates. The four moved numbers are identity and encoding *sizes*, they are cited nowhere outside this spike's own README and fixture, and they moved because the content folded into those identities grew as `tiler-ir`, `tiler-artifact`, and `tiler-reference` gained operations and fields between the two commits. The reference registry identity grew most because it enumerates reference implementations, and that set grew.

The fixture path was deliberately **not** renamed to today's date. Its stable path is the drift signal the retention convention depends on — "re-running overwrites it; a diff is drift" only works if the path holds still, and a rename would turn a reviewable diff into a delete-plus-add while breaking the link from `prototype-a-bounded-scalar-cpu-backend-vertical`.

### Proving the checks can say no

The ADR-cited perturbation was re-run, not carried forward on the strength of the earlier record: replacing the `CanonicalizeF32Nan` arm in `src/interpret.rs` with an identity made the run exit non-zero at the comparison naming exactly one differing element — backend `0x7fc01234`, reference `0x7fc00000`, every other element still agreeing. Reverted, and `git status` confirmed the file byte-identical to HEAD before committing.

### Notes for the coordinator

- The brief stated a claim was held as `worker-restore-the-`; `tkt show --format json` reported `assignee: null` and `status: todo`. The ticket was claimed at the start of this work instead.
- The forkless suite failed once on `No space left on device` during trybuild, not on a golden mismatch — the host was at 2.5 GiB free after four spike target directories. All four were swept; the re-run from a clean target passed 7/7. A relative `CARGO_TARGET_DIR=./target` also causes trybuild to write `probe/target/`, which the spike's anchored `/target/` gitignore does not cover; the final run used the README's plain `cargo nextest run --workspace` and left the tree clean.
