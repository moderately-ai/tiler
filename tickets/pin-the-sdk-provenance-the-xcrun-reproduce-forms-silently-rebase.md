---
id: pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase
title: Pin the SDK provenance the xcrun reproduce forms silently rebase
status: todo
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## A reproduce form that records "whatever is installed" rather than a version

Six citations across `docs/` spell an Apple SDK header as `$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h:<line>`. That form is **host-resolved**, so it names no version at all — it re-runs against whatever SDK the machine happens to carry.

**Coordinator-verified today:** the host resolves `--show-sdk-version` **27.0**, build **26A5388f**. Five records name macOS SDK **26.5**, build **`25F70`** — two mentions in `docs/status.md`, three in `docs/research/runtime/backend-scoped-route-requirement-answers.md`.

**Nothing asserted is currently wrong**, which is why this is a p2 and not a correctness defect: the header's line numbers happen to be unchanged across the two SDKs (the constants `MTLGPUFamilyApple1 = 1001` through `Apple10 = 1010` sit at the same lines), so the dated corrections remain reproducible. That is luck, not a mechanism — `AGENTS.md` requires dependency versions or commits to be recorded, and a form that resolves differently per host records neither.

Found by the worker repairing those citations' provenance, which deliberately did **not** rebase the recorded build to 27.0: doing so would assert one worker's read as the record's provenance and rewrite a dated, host-bound measurement. That restraint was correct and is why this is a separate ticket.

## What to decide, and it is a genuine choice

- **Root the citations at a version-naming path**, matching the `metal-0.33.0/src/device.rs` convention the same checker already skips as an external crate source. Names the version, but hardcodes a path no `xcrun` will produce.
- **Keep `$(xcrun …)` and state the SDK build beside it** in the prose, so the command stays runnable while the provenance is explicit. Cheaper, and preserves reproducibility on a host with a different SDK.
- **Re-base the records onto 27.0** with a fresh reading. Most work, and it discards a dated measurement rather than dating it.

Whichever is taken, apply it to **all six** rather than the one that prompted this — the recurring defect here is repairing the named instance and leaving its siblings.

## Verify before writing

Re-read each of the six at your base and confirm the line numbers still hold on **this** host's SDK; report per-citation. `AGENTS.md` requires host-specific observations be separated from portable guarantees, so say which of the two each repaired citation is.

## Closes when

Every `$(xcrun …)` citation carries an explicit SDK provenance or is rooted at a version-naming path; the 26.5/27.0 divergence is stated rather than latent; and `make citations` passes.
