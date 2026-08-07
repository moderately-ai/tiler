---
id: refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew
title: Refresh ADR 0106 and the architecture rows the conformance crate outgrew
status: todo
priority: p2
dependencies: []
related: [correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence, carry-the-device-executed-value-proof-into-the-conformance-crate]
scopes: [contracts/decisions, contracts/foundation]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [docs, doc-drift]
---
## What is stale, verified at source

ADR 0106 was written on 2026-08-07 against a crate admitted as a **smallest useful slice holding no content**. Within hours the same day it gained the BF16 vertical and then the migrated device-executed value proof. The record still describes the empty crate, in **unpinned present tense**, so it reads as current truth rather than as what was true at acceptance.

Confirmed by the coordinator against the tree: `crates/tiler-conformance/src` holds **13 source files** with device dispatch and two named `unsafe` sites, and its manifest carries `unsafe_code = "deny"`.

The false statements:

- **Line 23** — "The crate holds no items at all — only a module header."
- **Line 81** — "it creates no device object, no `MTLDevice`, and no pipeline state, because it contains no code at all." Note the record *anticipated* this going false — "the test will stop being passed once the device half exists" — so the repair is to record that it has, not to rewrite the reasoning.
- **Line 96** — "**Decided.** The crate inherits the workspace lint table unchanged", with `forbid` standing and the device half "unwritable as the crate stands". Tom decided otherwise on 2026-08-07: `deny` with named per-site allows, never crate-level, FFI-with-Metal only. The crate now restates the workspace table rather than inheriting it.
- **Line 104** — "it holds no items, and item 6 is why the half that would need them cannot be written yet."

`docs/architecture.md` carries the same staleness at lines **415, 443, 451**, including a claim that "the live-execution grep … still returns no file under `crates/`" — which is now false.

## How to repair it, and this is the part that matters

**Date rather than overwrite.** These statements were true at acceptance, which makes them the ADR 0077/0088 shape — a record states the profile as of its own acceptance, and the live document is what carries current truth. So add a dated note recording what changed and when, rather than editing the body to describe today and losing what the acceptance covered.

**That is the opposite of the repair its sibling ticket made.** [`correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence`](correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence.md) *substituted* a clause, correctly, because that clause was **never true at any commit** — it followed ADR 0079's precedent for a wrong stated reason with a surviving conclusion. Here the statements were true when written. Getting this distinction right is the ticket: a wrong statement is replaced; a superseded one is dated.

`docs/architecture.md` is the live document, so it is **edited** rather than dated — the two files take different repairs and the ticket must not apply one rule to both.

## Also owed: three tickets carry a claim now known false

Each states that five open conformance tickets share no scope set. Reported by the sibling ticket, which edited no ticket file:

- `decide-where-a-device-reaching-conformance-test-may-live.md:56` — and its attribution is *differently* wrong, grouping `implementation/runtime` with the reference ticket and splitting `research/scheduling` onto its own.
- `record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr.md:26`.
- `survey-what-belongs-in-the-conformance-crate.md:21` — self-correcting, since the same file refutes it at lines 69 and 198.

Three of the five carry **identical** scope sets, all concerning one compiler-resident file. Both counts drift — the population read 283/76 when the survey ran and 289/80 a few hours later — so **do not install another bare number**; state it with its command and its commit, as ADR 0106 now does.

## Explicit non-goals

Do not change what ADR 0106 **decides** — its five items and its eliminations stand, and the crate outgrowing the slice is the admission working rather than failing. Do not re-open where the crate lives, what it is for, or the unsafe rule.

## Closes when

ADR 0106 carries a dated note recording what the crate now holds and that the lint decision superseded item 6; `docs/architecture.md`'s three stale lines state current truth; the three ticket sites are corrected or annotated; and no repaired site carries a bare count without its command and commit.
