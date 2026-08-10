---
id: record-the-landed-physical-provider-seam-in-adrs-0078-and-0090
title: Record the landed physical-provider seam in ADRs 0078 and 0090
status: done
priority: p2
dependencies: [drive-an-external-physical-implementation-provider-through-compilation]
related: [accept-the-public-backend-provider-composition-boundary, disclose-offered-and-selected-physical-provider-sets-separately, retire-adr-0078s-stale-physical-provider-standing-clauses]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, decision, documentation, public-boundary]
---
## User-visible outcome

A reader of ADR 0090 learns that item 2's physical-provider registry is implemented rather than pending, and a reader of ADR 0078 finds the physical-implementation seam in its governed inventory at the rung the evidence supports — so the two records stop describing a tree that has moved.

## Why this exists

[`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md) landed the seam on 2026-08-08 but does not hold `contracts/decisions`, so it could not touch `docs/decisions/`. Its graph-maintenance section names this obligation and hands it here rather than leaving a partial sweep. This is a carrier ticket: the landing already applied every consequence inside the scopes it held (`docs/compiler/optimizer.md`, `docs/operation-extensions.md`, `docs/glossary.md`).

**Fact — the two statements that are now false, both read at `c81f9257` before the landing.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)'s accepted status paragraph ends "the item-2 physical-provider registry and the item-5 disclosure accessors remain unimplemented"; item 5's Fact paragraph states "today neither is answerable at all", citing `session.rs` as the population site. The first clause is now false for item 2. The second is *half* false: the **selected** set is answerable through `PlanAlternative::selected_physical_providers`, and the **offered** set is still lowering-only, which is exactly the split item 5 exists to preserve — so correcting it must not collapse the two.

**Fact — ADR 0090 item 5's cited line has been stale twice already.** The record cites `session.rs:1513`; a 2026-08-05 audit found it at `:2092`; at `c81f9257` it is `:2208`. Cite by searchable anchor rather than adding a third line number.

## Fact audit at base `750b29e0`, 2026-08-08

Per-Fact verdicts, each read in full at this base before any edit.

| Claim as written | Verdict | Evidence at `750b29e0` |
| --- | --- | --- |
| ADR 0090's status paragraph ends "the item-2 physical-provider registry and the item-5 disclosure accessors remain unimplemented" | **verified** | the sentence is the paragraph's last clause, unchanged |
| item 5's Fact states "today neither is answerable at all" | **verified** | the bolded lead of item 5's *Grounds* paragraph |
| the first clause is now false for item 2; the second is *half* false, selected answerable and offered still lowering-only | **verified** | `PlanAlternative::selected_physical_providers` exists in `session.rs`; `grep -n "capabilities.0.lowering().providers()" crates/tiler-compiler/src/session.rs` returns one line, still the sole population of `offered_providers` |
| "ADR 0090 item 5's cited line has been stale twice already. The record cites `session.rs:1513`" | **false** | the record cites `session.rs:2158-2159`. `git log -S "session.rs:1513"` shows `:1513` was the drafting citation and `0f2bb9c5` already refreshed it to `:2158`. The drift chain is worse than stated, not better: `:1513` → `:2158` in the record, against a true site of `:2208` at `c81f9257` and `:2307-2308` here. The ticket's instruction — cite by anchor, add no third number — is strengthened, and the correction adds no number at all |
| the `:2092` citation was "a 2026-08-05 audit" | **imprecise** | `:2092-2093` is the sibling ticket `drive-an-external-physical-implementation-provider-through-compilation`'s own Fact-audit row, not this record's citation |
| ADR 0078's open questions on item 5 were closed with dated resolutions in the 2026-07-31 sweep, so what is owed is the inventory row and rung | **verified** | the first two Open questions each carry a dated 2026-07-31 closure; the item-2 table row was the unrepaired part |
| `docs/operation-extensions.md` states the tested-guarantee rung requires a provider "written outside the defining crate" | **verified, and the ticket's gloss of it is imprecise** | the contract says *crate*, and a Rust integration test **is** a separate crate — the fixture's own header says so. The contract's hedge is about the defining *package*, and it resolves the difference by naming out-of-tree viability as inference. See the Outcome below for why the row is recorded at tested guarantee |
| every public surface named remains a labelled draft under ADR 0075 | **verified** | `accept-the-installed-physical-provider-public-surface` is `awaiting-decision`; the landed module's own documentation carries a "Draft boundary" section |

## Implementation keys

- Move ADR 0090's `implementation_status` prose forward for item 2 only, naming the landing ticket and its date, and leave item 5's status accurate to its half-landed state.
- ADR 0078's open questions on item 5 were already closed with dated resolutions during the 2026-07-31 acceptance sweep; what this ticket owes ADR 0078 is its **governed seam inventory** row and the maturity rung, not a reopened question. Read the whole record before deciding which.
- **Do not record a tested-guarantee rung without reading the evidence boundary.** `docs/operation-extensions.md` states that rung requires a provider written outside the defining crate to drive the seam through the ordinary compile path. The landed fixture is an integration test: a separate compilation unit reaching only `pub` items, but inside the defining *package*. The operation-extension contract already records that distinction as a Measurement; copy its boundary rather than rounding it up.
- Every public surface named remains a labelled draft under ADR 0075. Do not write acceptance language.

## Closes when

ADR 0090's status paragraph and ADR 0078's inventory agree with the tree, `make citations` passes, and no statement in either record claims an acceptance Tom has not given.

## Graph maintenance

- Editing during transfer forks a record. If the correct edit changes what either ADR *decides* rather than what it reports, stop and say so.
- If re-reading finds a third stale statement, repair it in the same landing and report the repair.

## Outcome (2026-08-08)

**Both records now agree with the tree, and nothing either one decides was changed.** Every repair is a dated correction placed *beside* the original, quoting the retired string verbatim, because each was true at the commit it was written against; substitution is reserved for a claim never true at any commit, and none here qualified. Each correction states inline that a grep hit for the quoted string lands inside the note, so a later reader cannot mistake presence of the string for standing of the claim.

**Census.** ADR 0090: 17 checkable tree-claim clusters, **9 false or partly false**, 8 verified. ADR 0078: 18 clusters, **11 false or partly false**, 7 verified. Only 7 of the 20 falses were caused by this landing; the rest predate it, chiefly the pipeline split `242fc51c` (2026-07-27) and an AGENTS.md rewrite. Two ADR 0090 line citations were repinned to searchable anchors with the substance unchanged (`verify_schedule_with_feasibility`, `validate_key`); a rotted pointer is an address, not a claim, so it is repinned rather than dated.

**Why the ADR 0078 rung is a tested guarantee rather than implemented support.** The evidence is a separate compilation unit reaching only `pub` items, which is strictly stronger than this table's first row rests on — that row's external semantic provider lives in a `#[cfg(test)]` module *inside* the defining crate. Recording the physical row lower would have made the table inconsistent with itself. The boundary is stated inseparably from the rung: inside the defining package, so an out-of-tree crate is inference, and the forkless spike is the unre-run artifact that would measure it.

**Two follow-ups filed rather than guessed at.** [`repair-adr-0078s-budget-stop-and-unknown-gap-evidence`](repair-adr-0078s-budget-stop-and-unknown-gap-evidence.md) owns item 3's exhausted-budget paragraph, whose three named symbols are all absent from the tree; a true replacement needs the current index-domain discharge model derived, and this ticket left a dated **Correction pending** note rather than restate a false Fact in new words. [`refresh-the-forkless-physical-provider-spike-against-the-landed-seam`](refresh-the-forkless-physical-provider-spike-against-the-landed-seam.md) owns the spike, whose retained `.stderr` golden pins an `E0599` for a method that now exists; the spike is outside this ticket's scopes and gates nothing.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Carrier Outcome remains true for what closed at `774ef881`: both ADRs recorded item-2 landing, citations work, and no statement claimed an acceptance Tom has not given. Two present-tense clauses this ticket wrote into ADR 0078 are now false as standing claims and were not retired when later sibling landings updated ADR 0090 only: `item 5's *offered* disclosure half is still lowering-only` (false after [`disclose-offered-and-selected-physical-provider-sets-separately`](disclose-offered-and-selected-physical-provider-sets-separately.md) landed `Compilation::offered_physical_providers`; `Compilation::offered_providers` remaining lowering-only is deliberate) and `has not been re-run against the landed seam` (false after [`refresh-the-forkless-physical-provider-spike-against-the-landed-seam`](refresh-the-forkless-physical-provider-spike-against-the-landed-seam.md) produced `spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json`). The Outcome and ADR census "seven tests" was accurate at carrier close and under-counts at this audit (`grep -c '#\[test\]' crates/tiler-compiler/tests/external_physical_provider.rs` → 9). Residual ADR 0078 prose repair is owned by [`retire-adr-0078s-stale-physical-provider-standing-clauses`](retire-adr-0078s-stale-physical-provider-standing-clauses.md); this done ticket is not reopened for product work.
