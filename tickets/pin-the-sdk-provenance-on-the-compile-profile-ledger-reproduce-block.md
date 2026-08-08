---
id: pin-the-sdk-provenance-on-the-compile-profile-ledger-reproduce-block
title: Pin the SDK provenance on the compile-profile ledger xcrun reproduce block
status: in-progress
priority: p2
dependencies: []
related: [pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase]
scopes: [research/target-profiles]
shared_scopes: [project/tickets]
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

## Worker record, 2026-08-07, at base `209013bd`

### Per-Fact audit before editing

**Fact 1 — verified, unchanged at this base.** `grep -n 'xcrun --sdk macosx --show-sdk-path'` returns exactly three hits at `:394`, `:396`, `:398`, spelling `MTLComputeCommandEncoder.h` twice and `MTLComputePipeline.h` once. All three sit in the one fenced `sh` block under "Reproducible checks".

**Fact 2 — verified in every sub-claim, and its conclusion is the one that did not survive.** None of the three carries a line pin; all three are `rg` reproduce commands. `check-citations.sh:415` is `if (in_fence) next`, so fenced content is never classified. `:39` is the `| macOS SDK | \`macosx\` 26.5, build \`25F70\` |` row. `:400-405` does byte-compare a header across both installed SDKs and print one identical digest twice. **But "least defective" holds for only two of the three**, and that is the finding below.

**Fact 3 — verified.** On this host today, `xcrun --sdk macosx --show-sdk-version` prints `27.0`, `--show-sdk-build-version` prints `26A5388f`, and `--show-sdk-path` resolves to `/Applications/Xcode-beta.app/…/MacOSX27.0.sdk`. `xcode-select -p` is `/Applications/Xcode-beta.app/Contents/Developer`. That SDK build matches the one the ledger already records at its evaluation-order row for finding 34, which is corroboration rather than a second reading.

### The three are two populations, and the digest check covers the wrong one

The ticket's premise — that the block's existing digest check makes these three the least defective of the six — is **half true, and the false half is the whole of the work.**

`:400-405` byte-compares `MTLComputeCommandEncoder.h` only. Measured: it is identical across both SDKs (`610bcf8f…`), so the two commands reading it are genuinely unaffected by which SDK `xcrun` picks and print identical output at both. **`MTLComputePipeline.h` is not covered by that check and is not identical**: 305 lines at 26.5 against 297 at 27.0, digests `8f194e26c3df43a8787edc1aa6898f7156f065fb21bf043629d7e5227865c9aa` and `1b30d5dbf85c6ae007fb5b5c2a5194fce225d0afcf01fd02d5600d8660f9e3b5`. The command's own output moves with it: `maxTotalThreadsPerThreadgroup` reports at lines 52/53/55/227/230 at 26.5 and 52/53/55/217/220 at 27.0.

The block's comment read "The two installed SDKs agree on the dispatch header" — true, and a reader five lines below the third command takes it as covering the block. It did not cover the one header that differs.

**What the difference consists of, measured by `diff` rather than inferred from the digests: reflection-related `///` documentation comments and two blank lines. No declaration is added or withdrawn**, and `maxTotalThreadsPerThreadgroup` is declared identically on `MTLComputePipelineState` in both. Checked by filtering the diff for any changed line that is not `///` or blank — none. So nothing the ledger asserts is wrong, and the workgroup-threads row's evidence survives the SDK selection intact.

### Did each of the three need a change

- **`:394`, `:396` (`MTLComputeCommandEncoder.h`) — no change owed on their own terms.** The header is byte-identical across both SDKs and the block already proves it. The argument for leaving a reproduction on `$(xcrun …)` applies in full, and rooting them at a fixed SDK would destroy their only value.
- **`:398` (`MTLComputePipeline.h`) — a change was owed.** Its header is not covered by the digest check and does differ. Left alone, the block silently hands a 27.0 reader a different file with no line in the block saying so and a neighbouring comment implying otherwise.

Because the defect is the *asymmetry*, the repair is one statement covering all three rather than three sentences: a paragraph above the fence naming the recorded SDK (26.5 / `25F70`, cited from the environment table, not re-read), what `xcrun` resolves to here (27.0 / `26A5388f`), and the per-header consequence; a pointer in the fence's own leading comment for a reader who reads only the block; the digest loop extended to cover **both** headers so the asymmetry is reproducible instead of asserted; and a `diff` command establishing the difference is comments-only.

### Deliberately left alone

The environment table at `:34-42` is untouched — every diff hunk is at `:374` or later, and `:39` still reads `| macOS SDK | \`macosx\` 26.5, build \`25F70\` |`. The `:79` grid-axis elimination needed no repair: it already scopes its byte-comparison claim to `MTLComputeCommandEncoder.h` and `MTLTypes.h` by name and is correct as written. The three `$(xcrun …)` forms are kept, per the sibling's argument that a reproduction is worth having because it runs on the reader's host.

### The three sites are now at `:406`, `:408`, `:410`

Fact 1's `:394`/`:396`/`:398` are stamped "verified at `7c371155`" and were still true at `209013bd`; they are left as the dated claims they are rather than re-based, and the post-change numbers are recorded here instead.
