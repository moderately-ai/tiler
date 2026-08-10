---
id: retire-the-operation-extension-contract-s-stale-forkless-provider-rerun-future-tense
title: Retire the operation-extension contract’s stale forkless-provider rerun future tense
status: in-progress
priority: p3
dependencies: []
related: [refresh-the-forkless-physical-provider-spike-against-the-landed-seam, retire-adr-0078s-stale-physical-provider-standing-clauses, accept-the-installed-physical-provider-public-surface]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, backend-providers]
claimed_from: todo
assignee: sol-provider-rerun-contract
lease_expires_at: 1786407548
---

## User-visible outcome

The operation-extension contract records the completed out-of-tree physical-provider run as bounded evidence rather than still saying that re-running the spike *would* establish it, so its maturity account agrees with the retained result and the corrected ADRs.

## Why this exists

Independent review of [`retire-adr-0078s-stale-physical-provider-standing-clauses`](retire-adr-0078s-stale-physical-provider-standing-clauses.md) found one same-subject residual outside that ticket's `contracts/decisions` scope.

**Fact audit at integration base `d5c7a2cbe0608b6864ee8fa0ccec6d1be7f7c17b` — verified after reading the complete contract and retained evidence.** [`docs/operation-extensions.md`](../docs/operation-extensions.md), anchor `The spike is the artifact that would upgrade it`, continues in present/future tense: `re-running it is what would show the two blockers it recorded are gone`. The sentence was true before the refresh and is false as a live account now.

**Fact — the named measurement exists and is bounded.** [`refresh-the-forkless-physical-provider-spike-against-the-landed-seam`](refresh-the-forkless-physical-provider-spike-against-the-landed-seam.md) is `done`. Its complete retained result at [`spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json`](../spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json) records **8 tests run, 8 passed, 0 skipped** in a separate workspace against crates subject `cb62784c7d8b63aa2e73c9ac490101b748abc0ec`, with its exact toolchain and host. That is the out-of-tree Measurement the contract says would upgrade the in-package evidence; it is not an unbounded portability guarantee or acceptance of the public surface.

**Fact — the surrounding evidence distinction survives.** The in-package integration fixture remains a separate compilation unit inside the defining package. The refreshed spike is a separately authored workspace run. The physical-provider public surface remains a labelled draft awaiting Tom at [`accept-the-installed-physical-provider-public-surface`](accept-the-installed-physical-provider-public-surface.md). The correction must preserve all three distinctions.

## Fact audit at exact base `d7d142a87561cbd3797b8bda14875f31373ce8f4` — 2026-08-10

Every Fact above was re-read before any edit. The complete sources read were `AGENTS.md`, this ticket, [the operation-extension contract](../docs/operation-extensions.md), [ADRs 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md) and [0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), the related refresh, ADR-correction, and acceptance tickets, the complete retained result JSON, the complete in-package integration fixture and physical-provider module, and the relevant `CompileRequest`, `Compilation`, and `PlanAlternative` construction and accessor sites in `session.rs`.

| Fact | Verdict | Evidence at this base |
| --- | --- | --- |
| the contract still presents the provider-spike re-run as future work | **verified** | The source-safe anchors `The spike is the artifact that would upgrade it` and `re-running it is what would show the two blockers it recorded are gone` still resolve together in the live Measurement paragraph, with no dated correction following them before this ticket's edit. |
| the named out-of-tree Measurement exists and is bounded | **verified** | The refresh ticket is `done`. Its complete retained result records `cargo nextest run --workspace`, **8 tests run, 8 passed, 0 skipped**, against crates subject `cb62784c7d8b63aa2e73c9ac490101b748abc0ec`, on `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`), `aarch64-apple-darwin`, macOS `27.0 (26A5388g)`. |
| the in-package, separate-workspace, and acceptance distinctions survive | **verified** | The complete in-package fixture identifies itself as a separate crate and contains nine `#[test]` items; the retained result identifies a separate workspace with its own lockfile; the physical-provider module and ADR 0090 still label the exact surface a draft, and the acceptance ticket remains `awaiting-decision`. |

No verdict changes the ticket's purpose, either accepted decision, the public-boundary authority, or the evidence class. The work remains a quotation-preserving current-state correction only.

## Work

- Retire the live future-tense clause with a dated, quotation-preserving correction beside the existing Measurement paragraph.
- Cite the retained result and refresh ticket, including the recorded subject/toolchain/host boundary without re-running or widening it.
- Preserve the in-package-versus-out-of-tree distinction and the separation between tested evidence and accepted public surface.
- Contract and owning-ticket prose only; no crate, spike, result, ADR, maturity rung, or acceptance change.

## Closes when

The contract no longer presents the already-completed rerun as future work, the retained Measurement remains bounded to its recorded subject and environment, `make citations` passes, and no sentence implies Tom accepted the physical-provider surface.

## Outcome — 2026-08-10

The operation-extension contract now follows its stale future-tense sentence with a dated correction that quotes the retired wording, records the completed separate-workspace 8/8 Measurement at its exact subject, toolchain, and host, and preserves both the distinct in-package evidence and Tom's still-open acceptance authority. No crate, spike, result, ADR, maturity rung, or public-surface state changed.

**Verification and gate carry.** `make citations`, `tkt lint --format json`, and `git diff --check` pass. The exact base `d7d142a8` is a claim-only child of the green full-gate commit `0b0e6952`, and this delta touches only this ticket and `docs/operation-extensions.md` — none of the paths that invalidate a carried full gate — so it carries that gate. `tkt guard tkt/retire-the-operation-extension-contract-s-stale-forkless-provider-rerun-future-tense --base d7d142a8 --config-ref d7d142a8 --format json` passes on the committed diff with exactly those two files, both declared scopes affected directly, and no under-declaration; its `warn` severity reports declared sibling collisions rather than a scope escape.
