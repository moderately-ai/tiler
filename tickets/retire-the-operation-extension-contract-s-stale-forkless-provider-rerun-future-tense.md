---
id: retire-the-operation-extension-contract-s-stale-forkless-provider-rerun-future-tense
title: Retire the operation-extension contract’s stale forkless-provider rerun future tense
status: todo
priority: p3
dependencies: []
related: [refresh-the-forkless-physical-provider-spike-against-the-landed-seam, retire-adr-0078s-stale-physical-provider-standing-clauses, accept-the-installed-physical-provider-public-surface]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, backend-providers]
---

## User-visible outcome

The operation-extension contract records the completed out-of-tree physical-provider run as bounded evidence rather than still saying that re-running the spike *would* establish it, so its maturity account agrees with the retained result and the corrected ADRs.

## Why this exists

Independent review of [`retire-adr-0078s-stale-physical-provider-standing-clauses`](retire-adr-0078s-stale-physical-provider-standing-clauses.md) found one same-subject residual outside that ticket's `contracts/decisions` scope.

**Fact audit at integration base `d5c7a2cbe0608b6864ee8fa0ccec6d1be7f7c17b` — verified after reading the complete contract and retained evidence.** [`docs/operation-extensions.md`](../docs/operation-extensions.md), anchor `The spike is the artifact that would upgrade it`, continues in present/future tense: `re-running it is what would show the two blockers it recorded are gone`. The sentence was true before the refresh and is false as a live account now.

**Fact — the named measurement exists and is bounded.** [`refresh-the-forkless-physical-provider-spike-against-the-landed-seam`](refresh-the-forkless-physical-provider-spike-against-the-landed-seam.md) is `done`. Its complete retained result at [`spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json`](../spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json) records **8 tests run, 8 passed, 0 skipped** in a separate workspace against crates subject `cb62784c7d8b63aa2e73c9ac490101b748abc0ec`, with its exact toolchain and host. That is the out-of-tree Measurement the contract says would upgrade the in-package evidence; it is not an unbounded portability guarantee or acceptance of the public surface.

**Fact — the surrounding evidence distinction survives.** The in-package integration fixture remains a separate compilation unit inside the defining package. The refreshed spike is a separately authored workspace run. The physical-provider public surface remains a labelled draft awaiting Tom at [`accept-the-installed-physical-provider-public-surface`](accept-the-installed-physical-provider-public-surface.md). The correction must preserve all three distinctions.

## Work

- Retire the live future-tense clause with a dated, quotation-preserving correction beside the existing Measurement paragraph.
- Cite the retained result and refresh ticket, including the recorded subject/toolchain/host boundary without re-running or widening it.
- Preserve the in-package-versus-out-of-tree distinction and the separation between tested evidence and accepted public surface.
- Contract and owning-ticket prose only; no crate, spike, result, ADR, maturity rung, or acceptance change.

## Closes when

The contract no longer presents the already-completed rerun as future work, the retained Measurement remains bounded to its recorded subject and environment, `make citations` passes, and no sentence implies Tom accepted the physical-provider surface.
