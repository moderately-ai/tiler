---
id: prototype-candle-metal-adapter
title: Prototype the Candle Metal adapter
status: todo
priority: p1
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/candle, implementation/runtime, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, integration, candle]
---
Implement the first consumer adapter without contaminating compiler semantics: storage/layout validation, output allocation, device-scoped runtime cache identity, ABI binding, asynchronous lifetimes, preflight before custom-op application, and wrapper-level fallback. Start with the explicit contiguous/no-autograd subset and reject unsupported cases.

## Workspace admission — current facts (2026-07-28)

The owning production crate **is** absent, so this ticket owns its atomic workspace admission and lockfile update. `ls crates/` returns eight crates — `tiler-artifact`, `tiler-cache`, `tiler-compiler`, `tiler-ir`, `tiler-metal`, `tiler-metal-aot`, `tiler-reference`, `tiler-runtime` — and `ticketsplease.toml:122` maps `implementation/candle` to `crates/tiler-candle/**` and `prototypes/candle-*/**`, neither of which exists. This ticket already holds the two scopes admission needs: `implementation/workspace` in `scopes` and `implementation/cargo-lock` in `shared_scopes`.

**One thing changed about how that admission is checked.** The Python workspace gate that maintained a member table is gone — `e197176` ("Replace the Python gate with a Makefile of cargo commands") is in `main`, and `make full` is now a list of cargo commands with no separate member inventory. So adding a workspace member is caught by **reading the diff**, not by a gate that knows the expected set. Add the member and the lockfile update in one commit so a reviewer sees both, and note that `make lint` skips `prototypes/` for Clippy while still building and testing it — a prototype adapter is compiled and run by the gate, and only the style pass skips it.

After the real crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

## Closes when (2026-07-28)

The correctness priorities this adapter sits on are the ones `AGENTS.md` singles out for special scrutiny, so they are the closing criteria rather than a checklist appended to them.

1. **Preflight completes before the custom op is applied.** Every check that can decline — storage contiguity, layout, dtype, device, autograd absence, target availability — runs and commits to a route *before* Candle's custom-op path is entered. A decline discovered after application is not a decline.
2. **Fallback exists only at the wrapper level and only before any program work.** Once the adapter has allocated an output, begun encoding, submitted a command buffer, or failed semantic validation, there is **no fallback** — the failure surfaces as a typed error. A fallback after any of those four points cannot know what state the device is in, and a fallback after semantic validation failure would return a result the compiler refused. The wrapper must also be able to report *which* numerical realization it delivered, or a caller cannot tell a fast path from a fallback.
3. **Command-buffer terminal success is confirmed before any host readback.** No validation, comparison, or returned tensor reads device memory before the command buffer reports terminal success.
4. **The runtime cache is device- and context-scoped in its identity**, not global and not keyed by a name that two devices could share. An entry built for one device must be unusable from another by construction rather than by convention.
5. **Asynchronous resources are retained through their final device use.** Buffers, pipeline states, and encoded arguments outlive the submission that reads them; nothing is dropped on the strength of the host having finished with it.
6. **The contiguous / no-autograd subset is enforced with typed refusals, and everything outside it is refused by name.** An affine-strided layout, a non-contiguous view, an autograd-tracked tensor, or an unsupported dtype produces a typed error naming what was unsupported — never a silent copy, a relayout, or an approximation. `docs/open-questions.md` Q-RUNTIME-002 tracks affine-strided support as explicitly beyond this first profile.
7. **No Candle type reaches `tiler-compiler` or `tiler-ir`.** Reproduce with a dependency check, not by inspection: neither crate's manifest may gain a Candle dependency, direct or transitive through the adapter. This is the guardrail the ticket's first sentence names as "without contaminating compiler semantics", and it is the one that a working prototype most easily violates.
8. **`make full` passes**, with the new member and its lockfile change in the same commit.
