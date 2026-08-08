---
id: pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase
title: Pin the SDK provenance the xcrun reproduce forms silently rebase
status: done
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

## Worker record, 2026-08-07, at base `7c371155`

**The six are not one population, and the split decided the treatment.** Re-read at base: only **two** of the six carry the `MTLDevice.h:<line>` form this ticket describes (`backend-scoped-route-requirement-answers.md:161` and `:333`). One more is a `grep` reproduce command with no line pin (`:163`). The remaining three are `rg` reproduce commands inside a fenced `sh` block in `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md:394,396,398`, they carry no line pin, and two of them name `MTLComputeCommandEncoder.h` and one `MTLComputePipeline.h` — not `MTLDevice.h` at all.

**Option 1 for the pinned content claims, option 2 for the reproduce commands**, because those are two different kinds of assertion. A line pin is a claim about what a file contains at a version, so it must carry the version: `:161` and `:333` are now rooted at `MacOSX26.5.sdk/…`, matching the `metal-0.33.0/src/device.rs:74-82` spelling one clause away in the same sentence. A reproduction is worth having precisely because it runs on the *reader's* host, so `:163` keeps `$(xcrun …)` and states what it resolves to here. Option 3 was declined for the reason the finding worker gave: the 26.5 read is a dated host-bound measurement, and re-basing it would assert a fresh read as the record's provenance.

**The `$(xcrun …)` form was never checked by anything.** `check-citations.sh`'s `PATHRE_LOOSE` requires a leading `[A-Za-z0-9_]`, so a span opening `$(` is not parsed as a citation and is silently dropped. Demonstrated rather than asserted: with the old form restored and its line number corrupted to `:99999` against a 1530-line header, `./check-citations.sh` exits **0**. The version-rooted form is parsed — line-only citations 1058 → 1060, `rooted outside this tree` 15 → 17, and `--verbose` names both — and re-rooting one at a tracked component (`docs/System/…`) fails with `no file in the tree is or ends with`, exit 1. Both probes were reverted.

**Scope.** `contracts/navigation` needed no edit: `docs/status.md:24` and `:27` name SDK 26.5 build `25F70` inside dated measurement records that also state the OS build separately, carry no `$(xcrun …)` form, and are correct as written. The three ledger sites are `research/target-profiles`, which this ticket does not hold; work stopped rather than reaching and the remainder is [`pin-the-sdk-provenance-on-the-compile-profile-ledger-reproduce-block`](pin-the-sdk-provenance-on-the-compile-profile-ledger-reproduce-block.md).

## Outcome — done, 2026-08-07

Landed at merge `58cfe3d9` (worker commit `a050b0b5`). Three files, `docs/` + `tickets/` only, carries the green gate.

### This ticket's premise was false, and the falsity changed the answer

I said six citations share one form. **Only two do.** Three name a **different header entirely** (`MTLComputeCommandEncoder.h`, `MTLComputePipeline.h`) inside a fenced reproduce block on a different record, and one is a `grep` command carrying **no line pin at all**. One uniform treatment would have broken something either way.

### The decisive fact neither I nor the finding worker had

**The `$(xcrun …)` form was never checked by anything.** `check-citations.sh`'s loose path pattern requires a leading alphanumeric, so a span opening `$(` is dropped before classification — all six contributed **zero to every counter**.

Demonstrated rather than reasoned: restoring the old form with a deliberately absurd `:99999` against a 1,530-line header leaves `make citations` at **exit 0, "every pinned citation resolves"**, with the counters dropping by exactly one. So the drift I filed as latent was not merely unchecked — it was invisible.

### Two options taken, because the population is two populations

**A line pin is a claim about what a file contains at a version**, so the two pinned citations are now rooted at `MacOSX26.5.sdk/…`, matching the `metal-0.33.0/src/device.rs:74-82` spelling that already sits **one clause away in the same sentence**. **A reproduction is worth having because it runs on the reader's host**, so the `grep` command keeps `$(xcrun …)`; rooting it at a fixed SDK would destroy its only value. What it resolves to here is stated in the measurement boundary instead.

The tiebreak for rooting over prose-only: prose leaves the citation permanently invisible to the checker, while rooting makes it **parse and then skip as external, with the skip counted and named** — the checker's own counted-rather-than-silent principle.

Option 3 declined, agreeing with the finding worker: the 26.5 read is a dated host-bound measurement, and re-basing would assert a fresh read as the record's provenance.

### Verified per citation, and labelled per AGENTS.md

`MTLGPUFamilyApple1 = 1001` through `Apple10 = 1010` occupy lines **233-242 in both** SDK 26.5 and 27.0 — so both pinned citations still hold here. The headers are **not the same file** (1,530 lines against 1,551), so the agreement is luck. The 26.5 read is labelled a **host-specific observation** and the 27.0 agreement a **second host-specific observation, explicitly not a portable guarantee**.

`docs/status.md` needed **no edit** — its two mentions are dated measurement records stating OS and SDK builds separately, carrying no `$(xcrun …)` form. So `contracts/navigation` was declared and correctly unused.

### Stopped rather than reached

The three ledger citations are `research/target-profiles`, not held. Filed as `pin-the-sdk-provenance-on-the-compile-profile-ledger-reproduce-block` with the transferred argument and an explicit *"do not re-base the environment table"* — and the note that those three are the **least** defective of the six, since that ledger already declares the SDK build in its environment table and already byte-compares the header across both installed SDKs.

Four anchor-reach demonstrations, each broken, watched failing, and reverted.
