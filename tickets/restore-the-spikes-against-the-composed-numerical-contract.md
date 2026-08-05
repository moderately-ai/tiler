---
id: restore-the-spikes-against-the-composed-numerical-contract
title: Restore the contract-naming spikes against the composed numerical contract
status: review
priority: p3
dependencies: [compose-the-numerical-contract-from-its-decided-dimensions]
related: [restore-the-scalar-cpu-vertical-spike-against-the-current-crates]
scopes: [research/cache, research/extensions, research/target-profiles, research/scheduling, research/numerics, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [spikes, maintenance, numerics]
claimed_from: todo
assignee: agent-spike-restore
lease_expires_at: 1785960717
---
## User-visible outcome

Every retained spike that names a numerical contract builds against the composed `NumericalContract`, so re-running one still produces the evidence it claims to.

## Why this exists

**Fact.** `compose-the-numerical-contract-from-its-decided-dimensions` replaced the public `NumericalContract` enum with a composed type whose named points are associated constants, and it deliberately did not edit `spikes/`. No `make` target reaches a spike, so the workspace gate stayed green while three spikes were left naming variants that no longer exist. Each is a one-token change per site.

**Fact — the exact sites, from `grep -rn --include="*.rs" 'NumericalContract::' spikes` on the landing commit.**

- `spikes/cache/build-tool-exercise/envelope/src/lib.rs:59` — `NumericalContract::FlushSubnormalsToZeroF32`.
- `spikes/extensions/forkless-physical-provider/probe/tests/composition.rs:54` and `:89` — the same variant. Note this spike also carries `trybuild` UI fixtures naming it: `tests/ui/pass/lowering_installation_seam_exists.rs:27` and `tests/ui/fail/no_physical_provider_installation_seam.rs:35`, and the `fail` case's `.stderr` golden may move with the type.
- `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs:759` — `NumericalContract::StrictF32`.

The replacement in every case is the corresponding associated constant: `FLUSH_SUBNORMALS_TO_ZERO_F32`, `STRICT_F32`.

**Fact — observed 2026-08-01 at base `29a9680`, and the cache spike needs a second edit this enumeration does not name.** `retire-the-device-translation-policy-from-the-cache-spike-and-its-citing-records` ran `CARGO_TARGET_DIR=./target cargo check` from `spikes/cache/build-tool-exercise` and `CARGO_TARGET_DIR=./target cargo run -- results/2026-07-31-macos-arm64.json` from `spikes/target-profiles/scalar-cpu-vertical`, and the compiler's own lines are what these facts come from rather than a re-reading of the source:

- `spikes/cache/build-tool-exercise` reports **two** errors, not one. The enumerated `E0599` at `envelope/src/lib.rs:59` is the first; the second is `E0560: struct BackendEntryRef has no field named payload` at `envelope/src/lib.rs:160`, which is the `payload` → `payloads` step `restore-the-scalar-cpu-vertical-spike-against-the-current-crates` repaired for the other spike and nothing repaired for this one. The single-object spelling is `payloads: vec![payload]`, and the delivery-position decision that ticket had to take does **not** arise here: `grep -rn "DecodedProgram\|decode_artifact\|payloads" expansion-macro/src/lib.rs envelope/src/lib.rs consumer/src/lib.rs` finds no call site, because this spike assembles and encodes an envelope and never decodes one.
- `spikes/target-profiles/scalar-cpu-vertical`'s site has drifted to `src/vertical.rs:779` from the `:759` recorded above — the 2119b20 restoration moved it — and it is that spike's only remaining error. The run therefore stops at compilation and never reaches the fixture, so `results/2026-07-31-macos-arm64.json` is unchanged (sha256 `7c774b159d06f489c6c8d8ab44d29ae09d277b5fbd5eb0da9e4530da05877196`) and re-running it is still this ticket's work rather than something a later reader can assume happened.

*Inference.* The cache spike's own `Closes when` obligation — build **and run** — was already broader than the site list, and this is the gap that made the difference visible. Whoever claims this ticket should expect the enumeration to be a floor rather than a census, and re-derive it from a clean `cargo check` per spike.

**Fact — one of the three has a live owner.** `restore-the-scalar-cpu-vertical-spike-against-the-current-crates` was in progress against `spikes/target-profiles/scalar-cpu-vertical` when this drift was introduced, which is why that spike was left alone rather than edited underneath it. Whoever claims this ticket checks that ticket's state first and either coordinates or narrows this one to the other two scopes.

## The research records naming the same removed spellings

**Fact — four retained research documents name a Rust identifier that no longer exists**, found by `grep -rn 'StrictF32\b\|FlushSubnormalsToZeroF32\|RelaxedF32\|ReassociateF32' docs/research` on the landing commit: `docs/research/scheduling/first-metal-contraction-realizations.md:85`, `docs/research/numerics/first-quantized-lm-profile.md:134` and `:211`, `docs/research/program-planning/first-metal-lm-workload.md:221`, and `docs/research/extensions/backend-provider-composition.md:201` and `:292` — the last two being compilable example code.

They were left alone deliberately rather than swept. Each sits inside a recorded measurement or derivation, so the edit is a *spelling* correction inside a retained record and has to preserve what the record claims; `docs/research/program-planning/**` was additionally held by a live ticket at the time. The two `backend-provider-composition.md` sites are the sharper ones: they are example code a reader is meant to be able to run.

## Boundaries

- A spike runs by hand from its own directory using the invocation its README records; a build that is only *checked* is not the evidence. Re-run each spike and confirm the fixture or golden it retains still matches, rather than only making it compile.
- Do not repoint a `trybuild` golden by copying the new output without reading it: the fail case's message is the claim, and a diagnostic that moved for a different reason would be laundered by a blind rebaseline.

## Closes when

All three spikes build and run from their own directories, every retained fixture or golden they cite still matches or has been rebaselined with the reason recorded, the six research-record sites name the current spelling without changing what each record claims, and `grep -rn 'NumericalContract::[A-Z][a-z]' spikes docs` reports no match.

## Outcome

**All three spikes build and run from their own directories, at base `d5960e81`, macOS 27.0 (26A5388g) arm64, the pinned `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, `cargo 1.99.0-nightly (3efb1f477 2026-07-17)`), each with `CARGO_TARGET_DIR=./target`.**

**The enumeration was a floor, exactly as this ticket predicted, and the excess was in the spike it did not expect.** Re-derived from a clean `cargo check --workspace --all-targets` per spike:

| Spike | errors predicted | errors observed | the excess |
| --- | --- | --- | --- |
| `spikes/cache/build-tool-exercise` | 2 (already repaired at `9ec8028c`) | **0** | none — no repair needed |
| `spikes/extensions/forkless-physical-provider` | 2 + 2 `trybuild` fixtures | **2**, both `tests/composition.rs` | none; the `ui/` fixtures are compiled by `trybuild`, not `cargo`, so they never reach a compiler line and were repaired from the same reading |
| `spikes/target-profiles/scalar-cpu-vertical` | 1 (`vertical.rs:779`) | **2** | `E0004`: `KernelType::Bf16` not covered in `ImageType::from_kernel` |

**The unpredicted error is the type system doing its job, and the repair is a refusal rather than a spelling.** `KernelType` is deliberately not `#[non_exhaustive]` so that widening it is a build error at every backend that must decide what the new variant means, and `KernelType::Bf16`'s own definition asks a backend that cannot lower it to refuse it *by name*. This profile declares `f32` dispatchable and says nothing about `bf16`, so `Bf16` joined `U8` and `I32` in the refusing arm and the refusal carries the type through `TranslationError::UnsupportedValueType { found: KernelType }`. **No wildcard arm was added** — a catch-all is precisely what would stop the next widening from being caught this way.

**Per spike, the re-run verdict and the comparison.**

*`spikes/target-profiles/scalar-cpu-vertical`* — `cargo run -- results/2026-07-31-macos-arm64.json` exit 0; twelve `f32` elements agree bit for bit with `tiler-reference`. The prior fixture hashed `7c774b159d06f489c6c8d8ab44d29ae09d277b5fbd5eb0da9e4530da05877196`, confirming it was unchanged since this ticket recorded it. Four quantities moved and are tabled in the README: plan `72d49e71d668fff8` → `f6c5c487fbfbd8fa`, envelope 21,296 → 82,918, artifact identity 9,969 → 40,622, reference registry identity 912,256 → 1,420,906. The twelve output bit patterns, the profile descriptor (865), the payload (265), the element count, the zero deferred predicates, the host string, and every governed key did **not** move. 517 commits separate the bases, 125 touching `crates/` across 198 files, so the README states *that* they moved rather than *what* moved them, under the byte-count boundary it already carries. Because the reference oracle moved, the README's own rule required the `CanonicalizeF32Nan` perturbation again: removing it exits 1 naming exactly one differing element, `0x7fc01234` where the reference requires `0x7fc00000`, all eleven others still agreeing; reverted, and `git diff` over `src/interpret.rs` is empty. Four consecutive runs — one after the rebuild the perturbation forced — produced byte-identical fixtures and byte-identical 47-line narratives, `diff` exit 0 on every pair. `cargo fmt --check` and `cargo clippy --workspace --all-targets` clean.

*`spikes/extensions/forkless-physical-provider`* — `cargo nextest run --workspace`: **7 tests run, 7 passed**. Two of the three `.stderr` goldens matched with no regeneration. The third, `fail/no_physical_provider_installation_seam.stderr`, moved in **one line**, and it was not blessed: the new golden was written *by hand* from the reasoning that the file echoes its own fixture's source and line 35 of that echo is the contract spelling, and the suite was then run against the prediction and went green. It was then re-perturbed — golden reverted to the stale spelling — and `trybuild` reported `mismatch`, printing an ACTUAL OUTPUT differing from the stale EXPECTED in that one echoed line and nowhere else. The `E0599` claim itself, its span, and its caret column are byte-identical to the golden recorded at `7b1e3a7`, so what moved is the source being quoted rather than the diagnostic being claimed. The toolchain is byte-identical to the one `results/2026-07-31-macos-arm64.json` records — same channel, `rustc` commit hash, `cargo`, host, and OS build — so `ui.rs`'s "re-record the toolchain when you bless a golden" rule had nothing to re-record. `cargo fmt --check` and `cargo clippy --workspace --all-targets` clean.

*`spikes/cache/build-tool-exercise`* — no repair needed, but re-run anyway because 146 commits had landed since its `9ec8028c` restoration, 37 touching `crates/` across 91 files including `tiler-cache/src/expansion/`. `python3 spikes/cache/build_tool_exercise.py --concurrency 3 --analyzer "$(rustup which --toolchain nightly rust-analyzer)"` exit 0, analyzer scenarios included. All **72** counted cells identical to the 2026-08-05 row. The comparison counts the cells it compared and fails when the population is not 72, so "no differences" is distinguishable from "the comparison did not run", and it was proved able to say no by perturbing `negative-control-x3.builds` 12 → 4 and watching it report exactly that. `overlaps` and `seconds` moved as always and carry no claim. **No fixture was recorded**: every counted cell reproduced and the label would have collided with the existing `macos-27.0-2026-08-05` file, which was taken at a different commit — a second file under one date, or an overwrite of `9ec8028c`'s evidence. The README records the reproduction and the reproducing command instead.

**The six research-record sites name the current spelling.** `docs/research/scheduling/first-metal-contraction-realizations.md` (the `Subnormals` obligation row), `docs/research/numerics/first-quantized-lm-profile.md` (×2), `docs/research/program-planning/first-metal-lm-workload.md`, and `docs/research/extensions/backend-provider-composition.md` (×2, the compilable example and its prose). Each was read in context first; all six are the same named point under its current spelling and no record's claim changed. The bare identifiers were qualified as `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` rather than left bare, because an associated constant without its type is not a resolvable name and findability is the point.

**The closing grep does not return clean, and cannot from this ticket's scopes.** `grep -rn 'NumericalContract::[A-Z][a-z]' spikes docs` at `31114b2b` returns four lines, every one **prose** and none of them code:

- three `spikes/**/README.md` paragraphs — one landed by the `9ec8028c` restoration and two written by this one — that quote the old spelling in order to *explain* the migration. Sweeping these would make the corpus worse: a reader holding the old name could no longer find what replaced it.
- `docs/decisions/0011-per-operation-numerical-permissions.md:78`, an **accepted ADR** whose realization paragraph still calls `NumericalContract` an enum with "two named user-facing modes". This is a genuinely stale claim about current code and it sits in `contracts/decisions`, which this ticket does not hold.

The condition the check was written to enforce — no *code* site names a removed variant — does hold, and its sharp form is one line: `grep -rn 'NumericalContract::[A-Z][a-z]' --include='*.rs' spikes docs crates prototypes` returns exactly one match, `IncoherentNumericalContract::UnfoundedValueDomainProvenance` at `crates/tiler-compiler/src/session.rs:1808`, a different type caught only because the pattern is unanchored.

**Filed rather than absorbed.** [`correct-the-numerical-contract-spelling-outside-the-restored-spike-scopes`](correct-the-numerical-contract-spelling-outside-the-restored-spike-scopes.md) owns ADR 0011's realization paragraph and a stale verbatim quotation in `spikes/apple-targets/code-domain-integer-decode/test_decode_probe.py:122`, both outside these six scopes. It deliberately excludes `spikes/numerics/bf16-second-dtype/README.md:91`, which spells `NumericalContract::{StrictF32, …}` and says "Four presets": that row is a *measured survey of the surface at `59a2fe2`*, scoped by that spike's own `verified_at_commit`, and its prediction — "A BF16 contract is a fifth key, not a widened fourth" — was borne out by the BF16 work that has since landed. Editing a measured snapshot to match a later tree would falsify what it recorded; what that spike wants is a re-run, which is its own ticket.

**One housekeeping defect, found by running the spike rather than by reading it.** `trybuild` resolves its scratch project against the *test crate's* manifest directory and writes `probe/target/`, which the anchored `/target/` in the forkless spike's `.gitignore` did not cover — so the recorded invocation left 763 MB untracked in `git status` whatever `CARGO_TARGET_DIR` was set to, and the ignore file's own comment claimed the opposite. The rule is now unanchored; `git check-ignore -v` confirms both `target/` and `probe/target/` resolve to it.

**Commit `31114b2b`.** `tkt lint` ok, `git diff --check` clean, `tkt guard --format json` → `"conflict": false`, `"under_declared": []`, severity `warn` (declared-area overlap with open siblings, the non-failing class). No `rust-toolchain.toml` was added anywhere: `git ls-files | grep rust-toolchain.toml` returns exactly one path, the repository root's.
