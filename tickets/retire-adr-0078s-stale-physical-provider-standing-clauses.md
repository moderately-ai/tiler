---
id: retire-adr-0078s-stale-physical-provider-standing-clauses
title: Retire ADR 0078's stale physical-provider standing clauses
status: todo
priority: p3
dependencies: []
related: [record-the-landed-physical-provider-seam-in-adrs-0078-and-0090, disclose-offered-and-selected-physical-provider-sets-separately, refresh-the-forkless-physical-provider-spike-against-the-landed-seam, accept-the-installed-physical-provider-public-surface]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, documentation, backend-providers]
---
## User-visible outcome

ADR 0078 no longer asserts, in present tense, that item 5's offered physical disclosure is still lowering-only or that the forkless physical-provider spike has not been re-run against the landed seam — so a reader of the inventory/rung record matches the tree and ADR 0090's later-same-day corrections.

## Why this exists

[`record-the-landed-physical-provider-seam-in-adrs-0078-and-0090`](record-the-landed-physical-provider-seam-in-adrs-0078-and-0090.md) correctly recorded item-2 landing on 2026-08-08 and closed. Two present-tense residual clauses it wrote into [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md) were true at that close and became false when later landings updated ADR 0090 (and the tree) without retiring the same claims on ADR 0078. The parent stays `done`; this ticket owns only the documentation residual.

**Fact — offered physical disclosure is no longer missing.** `Compilation::offered_physical_providers` exists in `crates/tiler-compiler/src/session.rs` (`grep -n "pub fn offered_physical_providers" crates/tiler-compiler/src/session.rs` → one line). It landed under [`disclose-offered-and-selected-physical-provider-sets-separately`](disclose-offered-and-selected-physical-provider-sets-separately.md). `Compilation::offered_providers` remains populated from `capabilities.0.lowering().providers()` only; that split is deliberate and must not be collapsed.

**Fact — the forkless spike has a post-landing re-run artifact.** [`refresh-the-forkless-physical-provider-spike-against-the-landed-seam`](refresh-the-forkless-physical-provider-spike-against-the-landed-seam.md) is `done` and `spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json` is present. The package-vs-out-of-tree Measurement distinction on the tested-guarantee rung stays; only the "has not been re-run" standing clause is false.

**Fact — the standing false strings are still present on ADR 0078.** Reproduce:

```sh
grep -n "item 5's \*offered\* disclosure half is still lowering-only" docs/decisions/0078-name-the-intended-public-extension-seams.md
grep -n "has not been re-run against the landed seam" docs/decisions/0078-name-the-intended-public-extension-seams.md
```

Both return hits outside a retirement note.

**Fact — the seven-test census under-counts.** At carrier close the fixture had seven `#[test]` items; at this writing `grep -c '#\[test\]' crates/tiler-compiler/tests/external_physical_provider.rs` returns 9 (disclose added cases). Optional to repin as current count or "at least seven at landing".

## Implementation keys

- Beside or inside the 2026-08-08 implementation-boundary correction on ADR 0078, retire the standing clause `item 5's *offered* disclosure half is still lowering-only` with a dated note that quotes it verbatim; state that `Compilation::offered_physical_providers` landed under the disclose ticket, that `offered_providers` remaining lowering-only is deliberate, and that acceptance (not implementation) remains open at [`accept-the-installed-physical-provider-public-surface`](accept-the-installed-physical-provider-public-surface.md). Mirror ADR 0090's later-same-day pattern.
- Beside the tested-guarantee boundary paragraph, retire `has not been re-run against the landed seam` with a dated note citing `spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json` and the refresh ticket, keeping the package-vs-out-of-tree Measurement distinction.
- Optionally repin "seven tests" to the current count or "at least seven at landing" so the population does not silently under-count.
- Do not write acceptance language. Do not reopen the parent carrier. Do not change crate code.

## Closes when

Both standing false clauses are retired by dated corrections that quote the retired strings, `make citations` passes on the touched paths, and a reader of ADR 0078 cannot take either clause as a live claim.

## Non-goals

- Public-surface acceptance (Tom's at the accept ticket).
- Changing `offered_providers` to include physical identities.
- Re-running or expanding the forkless spike beyond citing its existing result artifact.
- Reopening or amending the closed carrier Outcome beyond its existing 2026-08-10 audit note.
