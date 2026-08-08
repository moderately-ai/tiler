---
id: pin-the-sdk-provenance-on-the-compile-profile-ledger-reproduce-block
title: Pin the SDK provenance on the compile-profile ledger xcrun reproduce block
status: in-progress
priority: p2
dependencies: []
related: [pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase]
scopes: [research/target-profiles]
shared_scopes: []
paths: []
tags: []
claimed_from: todo
assignee: w-pin-the-s
lease_expires_at: 1786165341
---
## Three host-resolved SDK header paths in the compile-profile authority ledger

[`pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase`](pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase.md) repaired the `$(xcrun --sdk macosx --show-sdk-path)/…` form in `docs/research/runtime/backend-scoped-route-requirement-answers.md` and could not reach the other half of the population: `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` is `research/target-profiles`, and that ticket held `contracts/navigation` and `research/runtime`. It stopped rather than reaching. This ticket owns the remainder.

**Fact — three sites, all inside one fenced `sh` block, verified at `7c371155`.** `first-macos-metal-compile-profile-authority-ledger.md:394` and `:396` spell `"$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLComputeCommandEncoder.h"`, and `:398` spells the same prefix with `MTLComputePipeline.h`. Reproduce: `grep -n 'xcrun --sdk macosx --show-sdk-path' docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md`.

**Fact — these are the *least* defective of the six, and the difference decides what is owed.** None carries a line pin: all three are `rg` reproduce commands whose whole value is that they run on the reader's host. They are inside a fence, so `check-citations.sh` never parses them (`check-citations.sh "if (in_fence) next"`), and they were never citations in that script's sense. The ledger also already states its SDK provenance in its own environment table — `first-macos-metal-compile-profile-authority-ledger.md:39 "macOS SDK"` is the row recording `macosx` 26.5, build `25F70` — and `:400-405` already byte-compares `MTLComputeCommandEncoder.h` across both installed SDKs and asserts one identical digest twice.

**Fact — the divergence is nonetheless real and unstated at the commands themselves.** Measured 2026-08-07 on this host: `xcrun --sdk macosx --show-sdk-version` prints `27.0` and `--show-sdk-build-version` prints `26A5388f`, resolving to `/Applications/Xcode-beta.app/…/MacOSX27.0.sdk`, while the table at `:39` records the read at 26.5 / `25F70`. A reader running the block gets the 27.0 header and no line in the block says so.

## What to do

The sibling ticket took a split treatment and the argument transfers: a *pinned content claim* gets rooted at a version-naming path so the line numbers stay attached to the read, and a *reproduce command* keeps `$(xcrun …)` so it stays runnable, with the resolved identity stated beside it rather than baked in. All three sites here are reproduce commands, so all three take the second arm — a sentence in the fence's leading comment (or immediately above it) naming the SDK the block was recorded against and the SDK `xcrun` selects on this host today.

Do **not** re-base the ledger's environment table onto 27.0. That table is a dated, host-bound measurement record and rewriting it asserts one worker's read as the record's provenance. The `:400-405` two-SDK digest check is the evidence that the header agreement is observed rather than assumed, and it should be cited by whatever sentence is added.

## Closes when

Each of the three commands is covered by an explicit statement of the SDK the block was recorded against and what `$(xcrun …)` resolves to on this host, with the 26.5/27.0 divergence stated rather than latent; the environment table is unchanged; and `make citations` passes.
