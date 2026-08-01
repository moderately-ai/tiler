---
id: raise-the-adopted-research-records-to-their-landed-implementation-status
title: Raise the adopted research records to their landed implementation status
status: todo
priority: p3
dependencies: []
related: [close-remaining-adr-status-drift, re-audit-adr-implementation-status-after-the-runtime-and-metal-landings]
scopes: [research/kernel-ir, research/scheduling, research/indexing, research/cache, research/extensions, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, status-drift, graph-repair]
---
## User-visible outcome

An adopted research record stops reading `spike-only` when its adopting ADR reads `partial` and its decided behaviour is production code in `crates/`, so a reader can tell a record whose design shipped from one whose design is still a spike.

## Why this exists

**Fact — six adopted records read `spike-only` against `partial` ADRs with production code behind them.** Each reads `disposition: "adopted"` and `implementation_status: "spike-only"`; verify with `grep -n "disposition\|implementation_status" docs/research/*/*.md`.

| record | adopting ADR | production code |
| --- | --- | --- |
| `docs/research/kernel-ir/structured-kernel-ir-verifier.md` | 0048 | `crates/tiler-ir/src/kernel/verify.rs` |
| `docs/research/scheduling/scheduled-region-model.md` | 0007 | `crates/tiler-ir/src/schedule/` |
| `docs/research/indexing/index-access-model.md` | 0046 | `crates/tiler-ir/src/index/` |
| `docs/research/cache/crash-and-race-protocol.md` | 0050 | `crates/tiler-cache/src/expansion/` |
| `docs/research/extensions/operation-extension-surface.md` | 0005 / 0044 / 0052 | the registered operation-extension surface |
| `docs/research/runtime/semantic-validation-enforcement.md` | 0033 / 0051 | `crates/tiler-runtime/` |

**Fact — the audit that fixed the ADR side could not reach these.** [`close-remaining-adr-status-drift`](close-remaining-adr-status-drift.md) held `contracts/decisions` only — the `docs/decisions/[0-9]*.md` glob — so it never reached `docs/research/`. It bumped ADR 0007, 0043, 0046, 0048, and 0069 from `spike-only` to `partial` at `:24` "because each now has real production code in `crates/`, not only a spike", and left their research records where they were. This is that sweep's other half.

**Fact — the field's meaning makes the drift one-directional and safe to correct.** `docs/document-metadata.md:63`: "`implementation_status` names the highest implementation maturity the record's own decided behaviour has reached. It is a retained high-water mark, not a live mirror of the working tree."

## Boundaries

- **Read each record in full against the named code before bumping it.** A record's `implementation_status` describes *its own decided behaviour*, not its adopting ADR's — the two can legitimately differ, and inferring one from the other is the shortcut that would make this sweep wrong. Where they differ, say so rather than aligning them.
- Never lower a status; it is a high-water mark.
- Distinguish the four maturity claims AGENTS.md keeps apart — a type-system reservation, an architectural seam, implemented support, and a tested guarantee. `partial` is not a synonym for "some code exists".
- Scope is `docs/research/` and `spikes/` under the six named areas. `docs/research/runtime/runtime-execution-contract.md` belongs to [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md), which holds `research/runtime` for exactly that record; coordinate on the shared scope and do not both edit it.

## Closes when

Each of the six records carries a status its own decided behaviour supports, with the supporting code named in the record; any record deliberately left at `spike-only` records why in a sentence a reader can check; and no bump was made from the adopting ADR's status alone.
