---
id: preserve-typed-metal-aot-failure-causes
title: Preserve typed Metal AOT failure causes
status: done
priority: p2
dependencies: [bind-recorded-metal-toolchain-to-the-tools-that-execute]
related: [promote-the-metal-aot-compilation-identity, prototype-metal-aot-slice]
scopes: [implementation/metal-aot]
shared_scopes: []
paths: []
tags: [metal, aot, diagnostics, api]
---
Let callers retain the tool, phase, executable, exit status, and bounded output
that explain an offline Metal compilation failure.

The driver distinguishes discovery, version probing, source compilation,
linking, and output validation internally, but some public-facing failures
flatten their causal detail into strings. The exact executed-tool authority
must land first so diagnostics name the tool that actually ran.

## Outcome

Carry typed phase/tool/status/output causes through `DriverError` and retained
artifact-family diagnostics. Rendering remains convenient and bounded, but it
is a view of structured evidence rather than the only copy of it. Preserve
non-UTF-8 and truncated output honestly.

Any changed public error shape requires Tom's review before acceptance.

## Closes when

Callers can branch on the failure phase without parsing text, rendered errors
remain actionable, tests cover discovery/compile/link/output-validation
neighbors, and the full gate passes.

## Outcome (2026-07-27)

Three typed causes replace formatted strings on `DriverError`, and one field was added that the driver already knew and did not report.

**`ToolStatus`** — `Code(i32)`, `Signal(i32)`, `Unreported` — replaces `status: String`, which was `ExitStatus::to_string()`. That text is the host's wording, so a caller branching on a specific exit code (the Metal front end distinguishes them) was parsing a string this crate does not own. `Unreported` is kept as its own case rather than folded into `Code(-1)`, which would present a code no tool returned as one it did.

**`ToolOutput`** replaces `stderr: String` and retains **bytes**. `String::from_utf8_lossy` at the capture site replaces invalid sequences with `U+FFFD` and leaves nothing able to distinguish that from a tool that really emitted a replacement character — the ticket's "preserve non-UTF-8 honestly" is not satisfied by rendering lossily and saying nothing. `is_valid_utf8` reports which case a reader is in, and `Display` appends `[output was not valid UTF-8]` when it matters. Output is bounded at 16 KiB with `total_bytes` and `is_truncated` retained, so a prefix is never shown as the whole; the `Display` says how much of how much was kept.

**`ToolchainPhase`** — `Discovery` or `VersionProbe` — is added to `ToolchainUnavailable`, which previously covered both and distinguished them only inside `detail`. The remedies differ: discovery failing means no such tool is installed for the selected SDK, while a version probe failing means the tool exists and did not identify itself. One is an installation problem and the other a qualification problem.

**`ToolFailure` now carries `executable: PathBuf`**, the binary that actually ran. The stage's bare tool name and the resolved path are two different observations, and executing the resolved tool — which `promote-the-metal-aot-compilation-identity` established — is only half the property if the diagnostic still names the other one.

### Neighbours

Discovery versus version probe with an identical `detail` compare unequal; a signal is not an exit code and neither is the absence of both; non-UTF-8 bytes survive `capture` exactly and the rendering declares itself; a truncated output reports its true total, with the **exact-fit** case asserted not to claim truncation.

### The enum is still not `#[non_exhaustive]`, deliberately

Adding `ToolchainPhase` to an existing variant and changing two field types are breaking changes for `tiler-metal`, which matches `DriverError` out of crate to decide whether a failure means an absent Apple toolchain — in which case its compiling tests self-skip — or a defect it must report. That crate compiled unchanged here because its arms bind `{ .. }` and its one field read is `stderr.is_empty()`, which `ToolOutput` answers. The compile-error property the enum's documentation relies on is intact.
