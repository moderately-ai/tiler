---
id: compile-extension-spike-fixtures-in-the-gate
title: Decide whether the gate should compile the extension spike fixtures
status: in-progress
priority: p2
dependencies: []
related: [preserve-non-exhaustive-visibility-probe]
scopes: [implementation/workspace, contracts/navigation, research/extensions, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [testing, gate-reliability, harness]
claimed_from: todo
assignee: agent-compile-extension-spike-fixtures-in-the-gate
lease_expires_at: 1784925959
---
`preserve-non-exhaustive-visibility-probe` closed a decay gap, and left one open. The retained `#[non_exhaustive]` diagnostics are now verified by the repository gate — `scripts/tests/test_research_harnesses.py` runs `spikes/extensions/run.py --self-test`, which reads the checked-in `.stderr` files and the measurement beside them without invoking Cargo.

**What that check cannot see.** It verifies that a retained diagnostic still says what the record claims. It does not verify that the fixture `.rs` beside it still *produces* that diagnostic. Someone can edit `consuming/tests/ui/fail/cross_crate_total_map.rs` until it no longer fails for the recorded reason — or no longer fails at all — and every gate check stays green, because only a Cargo run compares the two. The same is true of `operation-api`, `proc-macro-visibility`, and `semantic-foundation-api-v2`: no Cargo fixture under `spikes/extensions/` is compiled by the gate or by CI.

**Fact — the gate deliberately builds no spike Cargo fixture except one.** `scripts/check_rust.py` runs the root workspace plus `spikes/shapes/nightly-dependent-static-shapes` through its `check.sh`, and nothing else. Every pytest module the gate collects (`spikes/apple-targets`, `spikes/embedding`, `spikes/macro-environment`, `spikes/numerics/sound_accuracy`) is a verifier over retained results; not one invokes Cargo. So this is a standing policy, not an oversight in one directory.

**The decision.** Whether that policy should hold for `spikes/extensions`. Compiling its suite costs a small Cargo build on both CI hosts and buys the fixture/diagnostic agreement above. Two routes exist and they differ in who owns them:

- add `spikes/extensions` to `testpaths` in `pyproject.toml` and to `EXPECTED_PYTEST_PATHS` in `scripts/check_repository.py` (both `implementation/workspace`, which is why this ticket declares it), then add a pytest module that shells out to `run.py --suite …`; or
- extend `scripts/tests/test_research_harnesses.py` (`contracts/navigation`) with the Cargo suite alongside the `--self-test` entry it already runs.

Either way the ticket must state the added gate wall-clock cost measured on both supported hosts, and must keep `spikes/shapes/nightly-dependent-static-shapes` as the one Cargo spike the *Rust* gate owns rather than moving it. If the answer is that the policy should hold, record that and close: an explicit "the gate verifies retained evidence, not fixture reproduction" is a better resting state than an unexamined gap.

## Outcome

**Answer: the policy should not hold, and the standing "exactly one spike" arrangement was never a policy — it was an unstated special case that a third workspace had already fallen through.** The gate now compiles `spikes/extensions/non-exhaustive-visibility` on every invocation, and the admission rule is stated generally and enforced mechanically rather than applied case by case.

### The rule

**The Rust sub-gate owns every Cargo invocation the repository gate makes.** It is the only phase that selects the pinned toolchain explicitly through `rustup run <exact dated nightly>`, *rejects* rather than merely strips hostile Rust environment controls, validates the Cargo configuration visible from each workspace and its parents, snapshots and verifies the governed lockfiles, and gives each workspace its own `CARGO_TARGET_DIR`. Both routes the ticket enumerated put Cargo in the pytest phase, which runs at `scripts/check_repository.py` line 318 — twenty-nine lines before `scripts/check_rust.py` establishes any of those five protections. A Cargo command there is a weaker check wearing the same name, so this ticket departed from both enumerated routes and added the step to the Rust sub-gate instead. `spikes/shapes/nightly-dependent-static-shapes` was not moved, reorganized, or otherwise disturbed; it gained one guard, described below.

**A spike Cargo workspace is compiled by the gate exactly when it retains a compiler-produced golden artifact — a `trybuild` `.stderr` — captured on the toolchain `rust-toolchain.toml` pins.** Such a file is a positive claim about what a compiler emits, and it outlives whatever produced it: it stays on disk unchanged when the source beside it is edited, and only a compilation compares the two. A spike whose evidence is deliberately tied to a *different* toolchain is not gate-compilable at all, and is recorded as an explicit off-pin exclusion. A spike that retains no golden artifact is not compiled, because its conclusion is about whatever code is present and nothing checked in can go stale against it.

**Fact — the criterion partitions the tree, and it is not a rationalization of the status quo.** Three spike workspaces retain `.stderr` fixtures, not one. `spikes/shapes/nightly-dependent-static-shapes` retains four on the pinned `nightly-2026-07-19` and was already compiled. `spikes/extensions/non-exhaustive-visibility` retains three whose record names `"channel": "nightly-2026-07-19"`, exactly the pin, and was not. `spikes/shapes/shape-evidence` retains six captured on **stable Rust 1.89.0** — `measure.py:30` sets `TOOLCHAIN = "1.89.0"`, `Cargo.toml` declares `rust-version = "1.89"`, and its README documents `cargo +1.89.0 test` — and was not, and cannot be: reproducing it needs a toolchain `AGENTS.md` forbids installing without Tom's authorization, and re-recording it on the pin would destroy the stable-Rust claim the spike exists to make. So the rule admits two, excludes one for a stated reason, and leaves the three golden-artifact-free extension workspaces (`operation-api`, `proc-macro-visibility`, `semantic-foundation-api-v2`) out. The ticket's own framing — that this was a single unexamined directory — was incomplete.

**The rule is enforced, not merely written.** `scripts/check_rust.py` carries `GATED_SPIKE_WORKSPACES`, `OFF_PIN_SPIKE_WORKSPACES`, and `validate_spike_evidence_custody`, which runs before any Cargo command and fails when a retained diagnostic appears under `spikes/` outside both sets, or when an off-pin exclusion no longer retains anything. A future worker who checks in a compile-fail fixture under `spikes/` cannot reach a third, unexamined state by adding a file: the gate makes them admit the workspace or record the toolchain its evidence belongs to. Generated `target/` trees are excluded, because `trybuild` writes a rebuilt copy of every diagnostic into its scratch crate and demanding custody for a regenerable file would be noise.

### Why compiling wins on the evidence

**Measurement — the marginal cost, macOS arm64 (M-series, `nightly-2026-07-19`, this worktree, `/usr/bin/time -p`).** The added step in isolation, `rustup run nightly-2026-07-19 cargo test --locked --manifest-path spikes/extensions/non-exhaustive-visibility/Cargo.toml` with `CARGO_TARGET_DIR` set to the spike's own tree: **15.09 s real / 15.61 s user cold**, **1.23 s real / 0.50 s user warm**. That is the reliable figure and it is ~2.8 % of a warm gate.

**Measurement boundary — the end-to-end gate deltas on this host are dominated by contention, not by this step.** Complete gate before: 115.40 s real / 253.33 s user cold; 49.11 / 43.28 / 44.66 s real warm. Complete gate after: 154.24 s real / 276.03 s user cold; 50.47 / 41.44 / 61.65 / 52.72 s real warm during a busy period, then 45.37 / 34.59 / 35.69 s real warm on the finished tree once the host quietened. **Every one of those last three is faster than every pre-change baseline run**, which is only possible if the spread is host load rather than gate work: six sibling agent worktrees were compiling concurrently throughout the earlier batches. User CPU time is the more contention-resistant comparison and moved +22.70 s cold against an isolated step measuring +15.61 s user, which bounds the contention contribution rather than eliminating it. Read the per-run cost off the isolated step, not off the end-to-end numbers.

**Fact — it introduces no new external dependency and no new download.** The two spike lockfiles resolve the *identical* set of 27 crates.io packages — compared name by name, neither has one the other lacks — because both depend on `trybuild` 1.0.118 and nothing else (`serde`, `serde_derive`, `syn`, `toml`, `glob`, `termcolor`, `winnow`, …). The gate already resolves, downloads, and compiles every one of them for the shapes spike. The marginal cost is a rebuild into a second target directory, not a new supply-chain edge. `~/.cargo` sharing is unchanged and the two target directories stay separate, per `AGENTS.md`'s prohibition on sharing one `CARGO_TARGET_DIR` across workspaces.

**Fact — the cross-host portability concern is already answered by the repository's own posture.** Byte-for-byte `.stderr` comparison is not a new risk class here: the root workspace retains **14** `trybuild` diagnostics that `cargo test --workspace --locked` already compiles on both `macos-15` and `ubuntu-24.04`, and the shapes spike retains 4 more that `check.sh` compiles on both. Eighteen fixtures of the same shape are already green on both CI runners. **Measurement boundary:** the numbers above are macOS arm64 only. No GNU Linux x86-64 host was available to this ticket, so the Linux figure is *not* measured, and the claim rests on the structural argument that the visibility suite's dependency closure and diagnostic shape are the same as the 18 fixtures already gated there. If Ubuntu CI reddens on a normalization difference, that is the falsification, and the fix is a recorded Linux measurement rather than a weakened check.

**Inference — the "unstable lint expected to change on a toolchain bump" objection does not survive.** `non_exhaustive_omitted_patterns` is unstable, but the pin is an *exact dated nightly*, so under a fixed pin its diagnostics are deterministic. A pin move already hard-fails the gate today, before this change: `verify_visibility_evidence` compares `rust-toolchain.toml`'s channel against the record and refuses to reuse a measurement taken on another. Compiling therefore adds no new toolchain-bump failure mode — it adds a *fixture-edit* failure mode, which is precisely the decay this ticket names, and it makes the existing bump failure more informative rather than louder. There is no case for an opt-in tier: the step is cheap, deterministic under the pin, and its only red condition is one somebody must fix.

**Fact — a missing or inconsistent lockfile fails closed and needs no special handling.** The workspace checks in a `Cargo.lock` and the gate runs `--locked`, so a runner whose state disagrees gets an explicit resolution failure rather than a silent re-resolve. Its lock joined `LOCKFILES`, so a Cargo command that mutated it would be caught even if `--locked` were somehow bypassed.

### What was fixed beyond the stated gap

**Fact — a vacuous `trybuild` run could report agreement, in both gated workspaces.** `trybuild` resolves its cases from a glob. A suite whose fixtures moved, or whose glob stopped matching, reports an ordinary passing test having compiled nothing, and nothing else in the run distinguishes that from real agreement. `run.py` already guarded its hand-run path by naming each case in `require_output`; `check.sh` did not, and a bare gate step would not have either. `verify_fixture_coverage` now requires each workspace's run transcript to name every `tests/ui/*/*.rs` case it retains, and it is applied to **both** gated workspaces — exempting the older one would have reproduced the special-casing this ticket exists to end. This is a strengthening of the shapes step; nothing there was weakened or relaxed.

**Fact — a captured compile step's diagnostics were unreachable.** `run(capture=True)` folded a `CalledProcessError` into `GateFailure` without its output, so a failing captured command reported only an exit status. `run(combined=True)` now tees the merged stream — echoing each line as it arrives, so a cold build's progress stays visible — and attaches the full transcript to the failure. Verified by the tamper runs below, whose compiler output appears in the gate's error.

### Tamper evidence — the check has teeth

Three mutations, each reverted, each run against `scripts/check_rust.py`:

1. **The fixture stops failing at all.** Added `_ => 0,` to `cross_crate_total_map.rs`, leaving the `.stderr` and the record untouched. `run.py --self-test` — the pre-existing predicate — **passed**, which is the gap demonstrated directly. The gate **failed**: `Expected test case to fail to compile, but it succeeded`, then `Rust gate failed: command.failed: … cargo test … exit status 101`.
2. **The fixture still fails, for a different reason.** Deleted the `Growing::B => 2,` arm. The gate **failed** with `trybuild`'s exact `EXPECTED` / `ACTUAL OUTPUT` diff carried through to the gate error.
3. **The custody rule.** Planted `spikes/indexing/index-access-model/tests/ui/fail/planted.stderr`. The gate **failed** before any Cargo ran: `spike.ungoverned-evidence: [...] retain a compiler diagnostic outside every workspace this gate compiles or explicitly excludes`.

`scripts/tests/test_rust_gate_integrity.py` pins the rest: the exact command plan and phase order including the new step and both coverage calls, the per-workspace `CARGO_TARGET_DIR`, the absence of a `rust-toolchain.toml` in *every* gated spike, the ungoverned-diagnostic and stale-exclusion rejections, the `target/`-tree exclusion, and the vacuous-transcript rejection.

### What the retained-evidence predicate now claims

Before this ticket, `run.py --self-test` claimed exactly: *a record exists attributing these diagnostics to the currently-pinned channel, the diagnostics on disk still match that record, and the fixture inventory matches it in both directions.* It claimed nothing about the `.rs` files, whose content was entirely unconstrained. It still claims exactly that, and it is still worth running — it is the fast path, it catches a `TRYBUILD=overwrite` refresh and a moved pin without Cargo, and it proves nine tampering paths are rejected. What is new is that a second, independent predicate now claims the other half: **the fixture, compiled by the pinned toolchain on this host, produces that diagnostic byte for byte.**

**Fact — ADR 0074 did not overclaim and needs no qualification.** Its evidence paragraph already said the self-test runs "without invoking Cargo" and that "the fixtures themselves compile only when the suite is run by hand", which was accurate. That clause is now false in the other direction, so it was updated in place — an evidence refresh, not an amendment: no convention changed, and the record cites the same measurement, now checked in both directions. `spikes/extensions/README.md`'s matching sentence was corrected the same way, and `preserve-non-exhaustive-visibility-probe`'s outcome gained a **Closed.** note rather than being rewritten.

### Scope note

`contracts/navigation` was added before editing `scripts/tests/test_rust_gate_integrity.py`, whose command-plan assertion is exact equality and cannot pass otherwise. `research/extensions` was added for `spikes/extensions/README.md` and `contracts/decisions` for the one clause in ADR 0074 that this change falsifies. Both of the latter are single-paragraph corrections to statements this ticket made untrue; leaving them stale would have violated the documentation-coherence contract. `contracts/navigation` and `contracts/decisions` overlap the concurrently open `record-an-adr-for-the-metal-aot-crate-admission`, in different files.
