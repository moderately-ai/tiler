---
id: refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew
title: Refresh ADR 0106 and the architecture rows the conformance crate outgrew
status: done
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

## Outcome — delivered 2026-08-07 at `98d572c7`

**Every stale statement was where it was reported, and three more existed that the brief did not name.** ADR 0106 `:92` (item 5's "inherits the workspace table whole, including `unsafe_code = "forbid"`") and `:107` (the crate "holds nothing behind it"), plus — the important one — **`docs/architecture.md:74`, the *origin* of the claim `:415` refers back to**: "No stage of live execution lives in `crates/`", grounded on a grep that now returns two conformance-crate files. Repairing `:415` and leaving `:74` would have left the document contradicting itself.

**The date-versus-substitute distinction was applied structurally rather than asserted.** ADR 0106's body is unedited; six dated `Superseded` notes sit beside the statements that were true at acceptance, and each note **says which shape it is and why** — so a later reader can check the rule rather than infer it. They are spelled `Superseded` rather than `Correction` deliberately, to stay visually distinct from the substituted never-true clause already in the same file. `record-the-conformance-crate-…` now carries a `Correction` and a `Superseded` twenty lines apart, which makes the distinction legible in one place.

Item 4's note is the one to keep: it records that ADR 0077's test is no longer passed **"exactly as this paragraph said it would stop being"**, and leaves the reasoning unedited because it was correct at both ends. That is the nuance the brief asked to preserve, honoured rather than flattened.

**`docs/architecture.md` was edited, being the live document** — including `:74`, which now names the two files, records that the path exists three times, and draws by hand the distinction the grep no longer draws: both files are `#[cfg(all(test, target_os = "macos"))]` evidence machinery, so the table's reading survives — **but nothing mechanical confirms it any more, which the paragraph now says.** That last clause is the honest part; a repair that restored the conclusion without recording the lost check would have been worse than the drift.

**The three ticket sites got three different, argued treatments.** `decide-where-…:56` substituted, because its attribution was wrong *twice* — it grouped `implementation/runtime` with the reference ticket and split `research/scheduling` the wrong way, omitting `implementation/conformance` entirely. `record-the-conformance-crate-…:26` substituted, with two further stale sites in the same file found and dated. And `survey-…:21` **annotated rather than substituted**, on the argument that the survey's own table and Outcome *quote* that paragraph by name, so replacing it would leave them refuting a sentence that no longer exists — and that it is the premise the ticket was filed to test, which the file would lose if quietly repaired. That is the right call and not the obvious one.

**Counts reproduced with commands and pinned to the commit: 294 / 82** at `3e0074d5`, against 283/76 and 289/80 earlier the same day. Recorded as a moving number rather than a bare one, following the shape the ADR's own correction established.

**Catalogs checked rather than assumed**, with the check itself mechanical: a diff-grep for every frontmatter key and every heading returns nothing, so neither generated block in `docs/decisions/README.md` moves. The coordinator re-ran that check independently. All nine ADR-0106 references in `docs/architecture.md` were examined; five correctly needed nothing.

**One thing found beyond the brief and filed rather than absorbed.** Reading ADR 0079 in full to check for the same defect class showed it has one: three unpinned Consequences bullets say unsafe is "spent only in the one layer" and name "the one crate permitted to have them" — now two, verified by a manifest sweep for members dropping `[lints] workspace = true`. Its *decision* is not violated, since item 4 reserves a second diverging member to Tom and Tom decided it. Filed as [`date-adr-0079-s-one-crate-claims-for-the-second-diverging-member`](date-adr-0079-s-one-crate-claims-for-the-second-diverging-member.md) with the dated-note shape specified, rather than repaired inside a ticket scoped to a different ADR.

**Delta rule confirmed by the coordinator against the merge's own file list**: six files under `docs/` and `tickets/`, none under the build-configuration set, so it carries the latest green gate with `tkt lint` rerun.
