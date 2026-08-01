---
id: correct-the-roadmap-rows-falsified-by-the-contraction-and-accuracy-landings
title: Correct the roadmap rows falsified by the contraction and accuracy landings
status: done
priority: p2
dependencies: []
related: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary, bound-the-reference-contraction-comparison-for-the-profile-cells, re-audit-adr-implementation-status-after-the-runtime-and-metal-landings]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, roadmap, graph-repair]
---
## User-visible outcome

`docs/roadmap.md` stops asserting three facts the 2026-08-01 landings falsified, so a reader planning against the ladder is not routed to a done ticket, an unowned job, or an edit already made.

## Why one ticket

Three corrections, one file, one scope. Filing them separately would produce tickets smaller than their briefs and serialize three workers on `contracts/navigation`. Each site is named below so this ticket is writable from its line.

## The three falsified facts

**(i) The whole-program-recognizer limit is attributed to a `done` ticket, and one of its two statements of the limit is now false on both counts.** `docs/roadmap.md:298` ends "That limit is owned by the [optimizer conformance gate](../tickets/prototype-optimizer-conformance-gate.md)", and `:494` repeats "The first limit is owned by the [optimizer conformance gate]". That ticket is `done`. Worse, `:494`'s statement of the limit — "the compilation request path in `crates/tiler-compiler/src/request.rs` recognizes exactly two one-input/one-output F32 shapes" — is false on both counts: `select_supported_strategy` (`crates/tiler-compiler/src/request.rs:2138`) dispatches **three** strategies, and one of them, `normalize_contraction` (`:2180`), admits exactly **two** inputs.

Repoint at [`admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary`](admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary.md). Do **not** repoint at [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md), which `:409` and `:410` already name as the recognizer's owner and whose own Non-goals (`tickets/reach-a-verified-kernel-through-the-structural-families.md:37`) disclaim "a general program-shape recognizer" verbatim. Those two rows need the same repoint, and each also still says the recognizer "recognizes two whole-program shapes" — three, since the contraction landed.

**(ii) The contraction row asserts four cells uncompared and points R7's residual at a `done` ticket.** `docs/roadmap.md:421` reads "**Fact — no execution row, and four cells uncompared.** … The reference's 16,777,216-step work bound refuses the four prefill cells, which [`bound-the-reference-contraction-comparison-for-the-profile-cells`] owns and this rung deliberately did not settle", and its R7 column repeats the pointer. That ticket is `done`, with all six L3 cells reproducing their `direct` `result_sha256`. Restate what R7 still needs — a dispatched device comparison, which [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) owns — and remove the refused-cells residual rather than leaving it pointed at its own discharge.

**(iii) A row asserts two ADRs read `not-started` and hands off an edit already made.** `docs/roadmap.md:408` reads "Both ADRs' frontmatter still reads `implementation_status: not-started`, which this landing makes stale — the carrier is implemented and the initial supported subset is not, so `partial` is the value they now describe; correcting it is a `contracts/decisions` edit this row does not hold." Both now read `partial` at line 9 — `docs/decisions/0016-transcendental-accuracy-contracts.md:9` and `docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md:9`. Remove the handoff; the edit it asks for is done.

## Boundaries

- Scope is `contracts/navigation`, which is `docs/roadmap.md` and its siblings. Do not edit the ADRs (that is [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md)'s territory) and do not edit `tickets/reach-a-verified-kernel-through-the-structural-families.md`.
- Correct the assertions; preserve the rungs. None of these three corrections moves a rung, and a correction that quietly promotes one would be a second change hidden inside a repair.
- Preserve the original rationale where a row records a superseded claim — the roadmap's idiom is to keep the falsified sentence and name what falsified it, and several rows already do.

## Closes when

No roadmap row attributes a live limit to a terminal ticket; `:494`'s recognizer description matches `select_supported_strategy`'s actual three strategies and their arities; the contraction row's R7 residual names only what is still owed; `:408`'s handoff is removed; and each remaining cross-reference in the rows touched was checked against its target's current status rather than assumed.
