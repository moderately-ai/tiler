---
id: bind-recorded-metal-toolchain-to-the-tools-that-execute
title: Bind Metal provenance to the tools that produce the artifact
status: done
priority: p1
dependencies: []
related: [promote-the-metal-aot-compilation-identity, prototype-metal-aot-slice]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [metal, aot, provenance, correctness]
---
The compiler must not record toolchain A as artifact provenance while toolchain
B actually produces the AIR or metallib bytes.

## Fact

Preflight resolves and records absolute `metal` and `metallib` paths and their
versions. Compilation later asks `xcrun` to select bare tool names again. Tool
selection can therefore change between the recorded observation and execution.

## Outcome

The canonical compilation identity describes the exact SDK and executable
tools used for compilation and linking. Execute the resolved tools directly or
make one recorded launcher resolution authoritative for both observation and
execution. A changing-selection test must fail closed rather than misattribute
the artifact.

## Closes when

Every tool identity construction site is paired with the command that uses that
tool, and tests prove a selector change cannot produce bytes under stale
provenance.

## Outcome

Done. One resolution is now authoritative for both observation and execution, and a selection change fails closed instead of misattributing the artifact.

**The defect, measured.** One compilation performed *four* independent `xcrun` selections: `--find metal`, `--find metallib`, `metal --version`, `metallib --version` — then two more for the stages, `xcrun --sdk … metal …` and `… metallib …` by bare name. `resolve` recorded the paths and versions from the first four; `run_stage` produced the bytes with whatever the last two selected. Nothing compared them, and nothing pinned `DEVELOPER_DIR`, `SDKROOT`, `TOOLCHAINS`, or `PATH` between them.

The consequence reached identity: `CompilationIdentity::encode` folds the reported tool *versions* and deliberately excludes paths, so an artifact's cache key was derived from a version string obtained by a different invocation than the one that compiled.

**The fix — execute what was resolved.** `run_stage` takes the `ResolvedTool` rather than an SDK and a bare name, and runs `Command::new(&tool.path)`. `tool_version` runs the binary just located instead of selecting again. Six selections become two `--find` calls, and the recorded tool is the object that runs.

Direct invocation was verified before being relied on, not assumed: the resolved `metal` compiles a kernel with no `xcrun` wrapper and no environment beyond `ZERO_AR_DATE`, because it resolves its own SDK. The real-toolchain tests in `tiler-metal-aot` and `tiler-metal` — including the fifteen-combination realization matrix and the four golden recompilations — all still pass, which is the evidence that the stage flags still reach the compiler through the direct path.

**The test the ticket asked for, and it reproduces the defect.** `a_changed_tool_selection_fails_closed_rather_than_misattributing` installs a launcher whose `--find` answer differs from what its tool branch runs: `--find` reports `/bin/echo` while the tool branch forwards to the real `xcrun`. That is the selection change made deterministic.

Reverting `run_stage` to bare-name selection makes it fail with *"the driver produced an artifact while the tool it recorded was `/bin/echo`"* — the old driver compiles successfully and stamps genuine `metallib` bytes with a toolchain that never ran them. That is the misattribution demonstrated rather than argued.

**One record improved and says so.** `ToolchainEvidence::ReportedVersions` now gains a paragraph stating what a folded version describes: the binary that was located and that the compilation executed. The evidence class is unchanged and still does not license cross-host reuse — paths stay excluded from the subject for the portability reason already recorded — but on the host that compiled, the version is now a fact about the compiler that ran.

**Not addressed, and out of scope here.** The remaining ambient inputs are unpinned: `Command` inherits `DEVELOPER_DIR`, `SDKROOT`, `TOOLCHAINS`, and `PATH`, and the two `--find` calls are still two selections that could in principle disagree with each other. Both are narrower than the defect this ticket names — the recorded tool is now the executed one — and pinning the environment is a separable change with its own reproducibility argument.

Gate: `make full` green (974 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck).
