---
id: root-cause-the-intermittent-leaky-test-in-the-workspace-gate
title: Root-cause the intermittent leaky test in the workspace gate
status: review
priority: p2
dependencies: []
related: [package-a-multi-entry-bundle-from-one-expansion, prototype-inline-aot-integration-proof]
scopes: [implementation/workspace, implementation/ir]
shared_scopes: [project/tickets]
tags: [testing, gate, determinism, process-lifetime]
claimed_from: todo
assignee: agent-leak-hunt
lease_expires_at: 1785945916
---
## User-visible outcome

The workspace gate either never reports a leaky test, or names the one it means and explains why the leak is benign. Today it reports a count with no name, on some runs and not others, which is a dice roll a reader cannot act on.

## What was observed, and it is a count rather than a name

**Measurement — two occurrences on 2026-08-02, on the same host, across a day of gate runs.** `cargo nextest run --workspace` printed:

```
Summary [  10.282s] 2335 tests run: 2335 passed (1 leaky), 7 skipped
```

once at `c6bdd55` (integration gate) and once during `package-a-multi-entry-bundle-from-one-expansion`'s own run at `52d3835`. Every other gate run that day printed no `leaky` marker at all. The worker who saw it first could not reproduce it: not in a second full-workspace run with `--final-status-level=all`, nor in five consecutive `-p tiler-macros` runs, with no `LEAK` line in any.

**Fact — the gate cannot tell you which test it was, by configuration.** `.config/nextest.toml` reports failures only, so a leaky-but-passing test is counted and never named. That is the right default for a green run printing one line, and it is exactly what makes this observation hard to act on. Reproducing with `--final-status-level=all` or `--status-level=all` is the documented override and does not require changing the profile.

**Inference — the shape points at a child process outliving its test.** A nextest "leaky" verdict means the test exited but left a handle open — commonly a spawned child still holding the pipe. Every AOT test in `crates/tiler-macros` spawns `xcrun`/`metal`, which is the obvious candidate, and both observations came from runs that included those tests. This is a hypothesis, not a finding; it is written down so it can be refuted rather than assumed.

## Why this is worth a ticket rather than a shrug

AGENTS.md is explicit: "An intermittent failure is a defect in the mechanism to be root-caused and fixed; re-running until green, loosening the assertion, or shrugging it off as flaky each converts the gate into a dice roll, which is Tom's stated line." A leaky test is not a failure today — but it is the same class of nondeterminism, and a leaked child that later *does* hold a shared path is how a gate goes red for reasons unrelated to the change under test. The repository has already been bitten by exactly that shape: a producer's own binary path stopped resolving mid-run because sibling invocations unlinked and relinked the shared Cargo hardlink.

## Required work

- **Name the test.** Re-run the workspace gate with `--final-status-level=all` (repeatedly if needed, since it is intermittent) and capture the `LEAK` line. Until the test is named, nothing below is actionable.
- Determine what handle is left open. If it is a spawned `xcrun`/`metal` child, establish whether the test waits on it at all, and whether the wait can be missed on an early return or an error path.
- Fix the mechanism rather than the symptom: a test that spawns a child owns reaping it on every path, including the failing one. Do not silence the report and do not add a retry.
- If the leak turns out to be benign and unavoidable — a platform behaviour the test cannot control — say so with the evidence and record it where the next reader will look, rather than leaving a count nobody can explain.

## Required evidence

- The leaking test named, with the `LEAK` line quoted.
- The handle identified, not inferred from the test's name.
- After the fix, the gate run repeatedly enough to make the absence meaningful, with the number of runs stated — a single green run proves nothing about an intermittent defect, and an uncounted "it stopped happening" is the shrug this ticket exists to refuse.

## Explicit non-goals

Do not change `.config/nextest.toml`'s failure-only reporting to make the leak visible in every run; that trades one line of green output for noise on every gate, and the per-invocation override already exists. Do not add a retry, a sleep, or a `--no-capture` workaround.

## Graph maintenance

Filed 2026-08-02 at integration, on the second observation. The first was recorded by `package-a-multi-entry-bundle-from-one-expansion`'s worker, which correctly declined to chase it as out of scope and said it would be worth a ticket if it recurred. It recurred.

## Named but not yet root-caused, 2026-08-02 (base `3aa94f9`)

**Measurement — the leaking test is now named, and the Metal hypothesis is
refuted.** A warm `cargo nextest run --workspace --status-level=all
--final-status-level=all` produced this terminal line on workspace run 2:

```text
        LEAK [   2.467s] (2350/2367) tiler-ir::typed_handles typed_authoring_contract
```

The summary was `2367 passed (1 leaky), 7 skipped`. Workspace runs 1 and 3--8
on the same checkout reported no leak. Twelve consecutive
`cargo nextest run -p tiler-ir --status-level=all
--final-status-level=all` runs, one isolated `typed_handles` run, and one
package run with a diagnostic-only 1 ms leak timeout also reported no leak.
That is one occurrence in eight full-workspace runs and none in fourteen
narrower runs in this investigation; it is a measurement boundary, not an
absence claim. The environment was Apple M4 Max, arm64 macOS 27.0 build
26A5388g, `nightly-2026-07-19`, cargo-nextest 0.9.133, and trybuild 1.0.118.

**Fact — the named test never reaches the frontend AOT path.**
`crates/tiler-ir/tests/typed_handles.rs` creates one `trybuild::TestCases`, then
registers one compile-pass fixture and six compile-fail fixtures. The pass
fixture at `tests/typed-handles/pass/checked_authoring.rs` performs only local
IR construction and starts no process. A search across all 111 Rust source
files under `crates/tiler-ir` found no `std::process` or `Command::new`; its one
`.spawn` match is a scoped Rust thread. The process owner is trybuild.

**Fact — one reachable trybuild path inherits nextest's capture handles.**
Inspection of trybuild 1.0.118's `src/cargo.rs` found nine process invocation
sites. The metadata, fixture build/check, and fixture run calls use
`Command::output`, so they replace
stdout/stderr with their own pipes and cannot return until those pipes reach
EOF. The clean and `--keep-going` probes send both streams to `Stdio::null`.
Two sites use `Command::status` without redirecting either stream: the
`generate-lockfile` fallback and the initial dependency build. The fallback was
unreachable in every measured run because this checkout had one `Cargo.lock`;
the initial dependency build was reachable, so Cargo and its compiler/linker
descendants inherited the test's nextest capture handles. Cargo itself is
waited.

**Inference — the observed leak came from a descendant of that reachable
initial Cargo build.** Nextest's verdict means a process retained captured
stdout or stderr for more than its default 100 ms after the test exited, and
the audit above leaves the initial dependency build as the only reachable
trybuild call that inherited those handles. This identifies the handle and the
reachable ownership chain, not the exact retaining process: the leaking run's
descriptor/process chain was not captured. It does establish that the leak was
not an unreaped `xcrun`/`metal` child owned by this repository.

**Measurement — process sampling did not name the transient descendant.** One
full-workspace run was sampled every 20 ms and observed the expected
`typed_handles -> cargo -> rustc -> clang` compilation chain, but that run did
not leak. Five further full-workspace runs sampled every 10 ms for an orphaned
process under this worktree's `target/tests/trybuild` tree; none leaked and no
orphan was observed. The exact process retaining the inherited descriptor is
therefore still unknown. Calling the `rustc` or `clang` observed in a non-leaky
run the culprit would turn timing correlation into a false finding.

**Stop — the remaining investigation and any repair collide with a live IR
claim.** The named test and its fixture live in `implementation/ir`, which
`admit-an-additive-extent-relation` held concurrently during this
investigation. This ticket now claims that scope so the board cannot redispatch
the collision. `implementation/frontend` was removed: the named test never
reaches that code, so no surviving repair path can require it.
`implementation/workspace` remains because an exact finding of unavoidable
trybuild/Cargo/compiler teardown latency could justify a narrowly scoped
nextest override in `.config/nextest.toml`; retaining the scope does not presume
that outcome. No implementation file changed. Resume when the exclusive IR
claim is free; reproduce with status reporting set to `all` and capture the
live process/descriptor chain on the leaking run. If the retained process is
controlled by repository code, add an omission perturbation that leaks before
fixing ownership on every path. If it is trybuild/Cargo/compiler teardown
latency outside repository control, establish the exact process and closure
duration before proposing that override; the current evidence does not
authorize changing the failure-only profile, silencing leak reports, adding
sleeps, or marking this done.

## Root-caused, 2026-08-05 (base `e9ef24dc`)

**Fact — the leak is a descriptor-inheritance race in the standard library on macOS, and no test in this repository owns the handle.** Nextest captures each test's stdout and stderr through a pipe and calls the test leaky when that pipe has not reached EOF within `leak-timeout` (100ms by default) after the test process exits. EOF arrives only once every copy of the pipe's write end is closed. On macOS `std` has no `pipe2`: `library/std/src/sys/pipe/unix.rs` in `nightly-2026-07-19` gates the atomic `pipe2(O_CLOEXEC)` form on a target list that excludes Apple platforms and falls through to `libc::pipe()` followed by two separate `set_cloexec()` calls (lines 32--42). Between the `pipe()` and the `set_cloexec()` the two descriptors are inheritable, and `library/std/src/sys/process/unix/unix.rs` sets only `POSIX_SPAWN_SETPGROUP`, `POSIX_SPAWN_SETSIGDEF` and `POSIX_SPAWN_SETSID` -- never `POSIX_SPAWN_CLOEXEC_DEFAULT`. So when one nextest worker thread is creating a test's pipes while another worker thread calls `posix_spawn`, the second thread's child inherits the first test's pipe. That child holds the write end for its whole life, and the arbitrary test whose pipe was captured is reported leaky through no fault of its own.

**Fact — the previous section's trybuild inference is refuted, and so is the original `xcrun`/`metal` hypothesis.** The leaky test is not a fixed test. Twenty instrumented full-workspace runs on this base produced two leaky verdicts naming two different tests, neither of them the previously named one and neither using trybuild in the leaking path:

```text
        LEAK [   0.218s] (2477/2582) tiler-reference::grouped_query_head_layout a_merge_over_non_adjacent_axes_refuses_by_name
        LEAK [   0.223s] (1962/2582) tiler-macros delivery::tests::the_matching_fixture_compiles_what_this_emitter_produces
```

Together with the earlier `tiler-ir::typed_handles typed_authoring_contract`, three unrelated tests in three packages have now been named. The earlier reasoning was sound about what it inspected -- trybuild 1.0.118's `cargo.rs:103` really does inherit the capture handles -- but it was reasoning from the one test it had a name for, and the victim is arbitrary. That is also why the earlier narrow runs found nothing: fourteen `-p tiler-ir` runs exercised trybuild every time and never leaked, which the trybuild explanation could not account for and this one predicts, because a package run spawns too few processes concurrently to hit the window.

**Measurement — the race is observable directly, in isolation and in the gate.** A standalone reproducer spawning `Command` with piped stdio from 16 threads, with each child reporting its own descriptor table before `exec`, had 16 of 32000 children start holding pipe descriptors belonging to other spawns; the leaked descriptors carry the parent's own descriptor numbers and are marked inheritable, and several consecutive children shared the same pair, which is one thread descheduled inside the window while others spawned through it. In the gate itself, a `CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER` shim recording every test process's descriptor table across 20 full-workspace runs found 139 of 54020 test processes starting with a stray inherited pipe descriptor -- roughly 1 in 389 spawns, present in every single run -- while only 2 of the 20 runs produced a leaky verdict. The gap between those two rates is the explanation for the intermittency: the race is routine, and it only becomes a verdict when the process that captured the descriptor happens to outlive the victim's exit by more than the 100ms leak timeout.

**Measurement — both leaking runs were matched to the process that held the descriptor, by pipe inode.** In the first, the victim was pid 33002 running `a_merge_over_non_adjacent_axes_refuses_by_name` with its stdout pipe at inode 2472918796834978017; pid 33001, spawned 1.994ms earlier and running the unrelated `contraction_profile_cells the_staged_oracle_reaches_the_cheapest_refused_cell`, began life with that exact inode on fd39 and the matching read end on fd38. In the second, the victim was pid 27117 running `delivery::tests::the_matching_fixture_compiles_what_this_emitter_produces` with its stdout pipe at inode 7051982010477279622; pid 27116, spawned 1.695ms apart and running the sibling test `delivery::tests::the_emitted_arms_select_exactly_one_payload_per_consumer_target`, held that inode on fd35. In both cases the holder is another nextest test process, not a child of the victim, which is what distinguishes this from an unreaped child and is why no ownership fix inside a test could remove it.

**Inference — the verdict is benign, and the bound on that claim is stated.** The victim has already exited when the verdict is computed, its own output is complete because its own write ends closed on exit, and the stray copy carries no data -- nothing writes to it. Nextest proceeds after the timeout and the run's exit status is unaffected; `leak-timeout` is left at its default, so no `result = "fail"` setting turns it into a failure. The claim is bounded: this is benign for the current configuration on this host and toolchain, not a universal claim about leaky verdicts, which is exactly why the reporting change below preserves the ability to tell a new cause from this one.

**Decision — the mechanism is not repository-controlled, so the deliverable is a gate that names what it means.** No change to any test can prevent this: the window is inside `std`, the spawner is nextest, and the victim is chosen by scheduling. The three candidate repairs all fail. Patching or pinning around `std` is not available to a workspace and would be a toolchain change that is Tom's decision regardless. Lowering test concurrency to close the window trades the gate's whole runtime against a verdict that costs nothing. Setting `leak-timeout.result = "fail"` would convert a benign platform race into red gates, which is the dice roll AGENTS.md forbids, pointed the wrong way. What remains is to stop the verdict from being an unactionable count, which is what cost three separate investigations.

`.config/nextest.toml` therefore moves `status-level` from `fail` to `leak`, and leaves `final-status-level` at `fail` (`leak` is not one of its levels, and the summary already carries the leaky count). This is narrower than the ticket's non-goal forbids and shares its reasoning rather than overriding it: the non-goal rejects trading a quiet green run for noise on every gate, and `leak` sits between `slow` and `pass` in nextest's hierarchy, so it prints nothing on a clean run. With `slow-timeout` at 60s the `slow` level it subsumes is also silent in practice. Verified in both directions rather than assumed, in a throwaway project carrying a byte copy of this config: with a test that deliberately spawns `/bin/sleep 5` and never reaps it, `status-level = "fail"` printed `2 tests run: 2 passed (1 leaky), 0 skipped` and no name -- reproducing this ticket's original symptom exactly -- while `status-level = "leak"` printed `LEAK [   0.208s] (2/2) leakdemo::leaky deliberately_leaks_a_child` above the same summary. With the leaky test removed the same setting printed only the summary line, and with a failing test present it still printed the `FAIL` line and its captured output, so nothing the gate reported before is lost.

The discriminator this buys is the point: a leaky verdict that names a different innocent test on each occurrence is this platform race and is benign, while a leaky verdict that names the same test repeatedly is a real unreaped child and a defect in that test. A count can never make that distinction, and a name can. That reading is recorded in the config comment and in AGENTS.md so the next reader acts on it instead of reopening the hunt.

**Measurement — the gate after the change.** Twelve consecutive `cargo nextest run --workspace --locked` runs on the edited config, uninstrumented and under normal conditions, at ~11s of test time each: every green run printed exactly three lines after `Starting` -- the start line, the separator, and the summary -- confirming no added noise. Run counts for the whole investigation: 20 instrumented full-workspace runs (54020 test processes, 2 leaky verdicts), 12 uninstrumented full-workspace runs on the new config, 1 targeted `typed_authoring_contract` run to validate the instrument, 40000 spawns across two standalone race reproductions, and 4 throwaway-project runs proving the reporting change reports and stays quiet. Environment: Apple M4 Max, arm64 macOS 27.0 (Darwin 27.0.0), `nightly-2026-07-19`, cargo-nextest 0.9.133, trybuild 1.0.118.

**Reproduction.** The instruments are not preserved under `spikes/`: every `spikes/**` glob in `ticketsplease.toml` is owned by a `research/*` scope, none of which this ticket holds, so checking them in would have been a scope escape. They are three short files reconstructible from this description -- a C shim that walks fds 0..255 with `fcntl(F_GETFD)` and `fstat`, logs each descriptor's type, inode and `FD_CLOEXEC` state, and then `execv`s its arguments; a Rust program that spawns that shim from N threads with `Stdio::piped()`; and the same idea with a quarter of the children sleeping, applying nextest's own rule of waiting 100ms for EOF after `wait()`. To observe the race in the gate itself, build the shim and run `FDCHECK_LOG=<log> CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=<shim> cargo nextest run --workspace --status-level=all`, then look for any logged process whose table contains a descriptor above fd 2; matching that descriptor's inode against another process's fd0, fd1 or fd2 names the pair. Note that pipe inodes are recycled within a run, so a match is only meaningful when the two processes' lifetimes overlap.

**Graph maintenance.** The named-but-not-root-caused section above is left standing as the record of what was inspected; its trybuild inference is superseded by this section, not deleted. `implementation/ir` is no longer needed by any surviving repair path -- the leak is not owned by `tiler-ir`, and no file under `crates/` changed -- but the scope stays declared because the section above claimed it to stop a redispatch onto the same collision, and dropping it now would reopen that. Only `.config/nextest.toml` and `AGENTS.md` changed, both `implementation/workspace`.
