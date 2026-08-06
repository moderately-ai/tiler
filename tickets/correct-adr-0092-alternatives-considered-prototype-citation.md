---
id: correct-adr-0092-alternatives-considered-prototype-citation
title: Correct ADR 0092's alternatives-considered prototype citation
status: done
priority: p3
dependencies: []
related: [correct-the-sdk-apple-family-range-in-the-runtime-answer-record, close-the-serial-sum-run-gpu-family-probe-table]
scopes: [contracts/decisions]
shared_scopes: [research/runtime, project/tickets]
paths: []
tags: [documentation, decisions, status-drift]
---
## User-visible outcome

An accepted ADR stops citing a prototype pair table that no longer exists as evidence for one of its eliminations, so a reader checking the elimination against the tree finds what the record describes.

## Why this exists

**Fact — the citation is present tense and false.** [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md):62, under "Alternatives considered", eliminates "Publish the family vocabulary and let each consumer observe the device itself" on ADR 0074 conventions 5b and 5c, and states: "written as a table rather than a match — **which is what the existing prototype does** — a variant added to the vocabulary compiles cleanly, is never probed, and silently under-reports the device."

**Fact — no prototype pair table survives.** `prototypes/serial-sum-run/src/proof.rs:700-706` now documents the opposite construction: "This is the *binding's* vocabulary and not Tiler's, and the two are joined by Apple's own enumerator value rather than by a pair table. `MTLGPUFamily` is `#[repr(i64)]` with each variant declared at the number `MTLDevice.h` gives it, and `AppleGpuFamilyConstant` carries that same number transcribed from the same header, so the correspondence is arithmetic that already exists rather than a second table someone has to keep in step." [`close-the-serial-sum-run-gpu-family-probe-table`](close-the-serial-sum-run-gpu-family-probe-table.md) is `done`, and the candle adapter's table was removed at `662d9be` ("Make a new Apple GPU family stop the build rather than a device").

**Fact — the flag was raised in the research record and never discharged.** `docs/research/runtime/backend-scoped-route-requirement-answers.md:74` reads "**Fact — the identical table still stands in the second prototype, which this record did not cite**", filed by [`correct-the-sdk-apple-family-range-in-the-runtime-answer-record`](correct-the-sdk-apple-family-range-in-the-runtime-answer-record.md). That sentence is now stale in the same direction, so both sites move together.

**Inference — the acceptance sweep was scoped past it.** The sweep under [`land-the-cpu-vector-lane-tier-adr`](land-the-cpu-vector-lane-tier-adr.md) covered status, disposition, span links, and catalogs — not the body prose of an alternatives-considered section. Nothing validates the corpus, so a citation inside an elimination is checked only by a reader following it.

## Boundaries

- **The elimination is unaffected and must survive the correction.** Convention 5b's argument — a total map with no derivable wildcard value — does not depend on any prototype having written a table; the prototype was an illustration. Do not weaken or restate the elimination while repairing its example, and if a current illustration is wanted, the enumerator-value join is one: it is what the convention *asks for*, so cite it as the positive case rather than the negative.
- Correct `docs/research/runtime/backend-scoped-route-requirement-answers.md:74` in the same change; its `research/runtime` claim is shared here for that reason.
- Do not touch ADR 0092's `decision_status`, `implementation_status`, or the seven unaccepted public-boundary items at `:20` — those belong to [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md).

## Closes when

ADR 0092:62 describes a construction that exists, or drops the prototype clause without weakening the elimination; the research record's parallel flag at `:74` is discharged in the same change; and both sentences were checked against `prototypes/serial-sum-run/src/proof.rs` as it stands rather than against the ticket that changed it.

## Outcome — 2026-08-06, `67abe1da`

Delivered with [`re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent`](re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent.md) as one act on `tkt/re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent` from base `01ad1c99`, because the ADR correction and the span re-transfer are the same edit read from two sides. Docs-only: `docs/decisions/0092-…md` and `docs/research/runtime/backend-scoped-route-requirement-answers.md`.

**The wording chosen, and why it is not the referent swap the sibling ticket prescribed.** The clause "which is what the existing prototype does" became "which is what both prototypes independently wrote before the walk moved into `tiler-metal`", and a positive case was added: "Neither writes such a table now: both drive the walk from `tiler-metal` over `MetalGpuFamily::ALL` and reach Apple's constant by the enumerator value both sides transcribe from `MTLDevice.h`, which is the construction decision item 3 requires."

The two repairs this ticket's Boundaries left open were *drop the clause* and *cite the enumerator-value join as the positive case*. Both were taken, because they answer different halves. Dropping the present-tense clause is what makes the entry true; but the clause was carrying more than illustration — the research record's own **Inference** rests on the table having been written **twice, independently, against two bindings**, which is "measured rather than predicted" and is the strongest evidence that a published vocabulary would be mapped by hand again. Deleting it outright would have spent that evidence to fix a tense. Moving it to the past keeps it and costs nothing, since it is history and history does not drift. The positive case is then added rather than substituted, because a reader checking the elimination against the tree needs to find *something*, and what they find is decision item 3's construction — the shape the convention asks for, reached twice for the defect's own sake rather than this design's.

**Fact — read from source at this commit, not from either ticket.** `prototypes/serial-sum-run/src/proof.rs:1161-1319`: `BINDING_APPLE_FAMILIES` lists the nine enumerators `metal` 0.33.0 names, `binding_apple_enumerator` finds one by comparing `enumerator as isize` against `AppleGpuFamilyConstant::value()`, `probe_apple_families` drives the walk from `observe_highest_gpu_family`, and a `const _: ()` block asserts `MetalGpuFamily::COUNT == 5` and sweeps `ALL` for nameability. `prototypes/candle-metal-adapter/src/adapter.rs:658-661`: `observed_apple_family` is `observe_highest_gpu_family(|family| raw.supportsFamily(MTLGPUFamily(family.value())))`, with the doc comment stating the pair table was the defect. **No pair table exists in either prototype**, so the sibling ticket's prescribed wording "a prototype still does" would itself have been false — the one thing that ticket said must not be applied mechanically.

**The elimination was not weakened.** Convention 5b's argument is that mapping a Tiler family onto its Apple constant is a total map with no derivable wildcard value; that is a property of the map, not of anyone having written one. The 5c half — that the *table* form removes even the compile error the attribute would force — is stated exactly as before. Both sentences of the argument are untouched; only the illustration moved.

**Not touched, as required:** `decision_status`, `implementation_status`, the seven unaccepted public-boundary items at `:20`, and every decision item including item 8's already-applied `RouteResourceRequirement` spelling. `git diff 01ad1c99..67abe1da -- docs/decisions/` is two hunks, both inside *Alternatives considered*.

**The `:74` flag is discharged, and so are its three siblings.** The flagged sentence now records the second site as closed at `8a5e20c5` and states what replaced it. The same stale claim stood in three further places in that record and was rewritten with it, because a scan that fixes one instance of a pattern and leaves its siblings is not a discharge: the section heading (`— since closed in one of its two sites`), the b2 **Inference** (`the other is filed and open`), the question-1 elimination bullet (`what \`prototypes/serial-sum-run\` still does`), and the deferral bullet (`The identical table survives at …`). All four now say the defect is closed in both sites.

A correction note recording this repair was added to the ADR's *Alternatives considered* section, outside the six transferred entries, so the span re-transfer stays byte-clean.
