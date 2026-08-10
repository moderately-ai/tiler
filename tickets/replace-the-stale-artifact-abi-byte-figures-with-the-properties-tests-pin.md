---
id: replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin
title: Replace the stale artifact ABI byte figures with the properties tests pin
status: done
priority: p1
dependencies: []
related: [recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix, pin-the-differing-identity-positions-beside-the-carrier-positions-constant, date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, abi, identity, documentation]
---

`docs/artifact-abi.md` carries byte figures that no test asserts and that the `v15 -> v16` identity step moved. Two of them point **past the end of the structure they index**, so a reader following them lands nowhere. This is the sibling of [the dtype-support repair](recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix.md), which resolved the same defect class in `contracts/navigation`; that ticket's route and its argument should be read before starting here.

## Facts

**Reported by the dtype-support worker, coordinator-verified in part.** Verified at `775e410f`: `36,832` appears in `docs/artifact-abi.md` and in no other document. Verified: `DIFFERING_CARRIER_POSITIONS = 68` is asserted at `crates/tiler-artifact/src/program/codec/tests.rs`, anchored by the constant name.

**Reported and NOT independently verified by the coordinator — re-measure before relying on any of it.** ~~Six figures~~ **Corrected 2026-08-08 by the worker: there are nine, not six.** Counting the list this sentence itself gives — identity length `48,584`, offsets `3,104`, `3,106`, `47,898`, `47,899`, and the four lengths `90,806` / `45,457` / `73,556` / `36,832` — yields `1 + 4 + 4 = 9`. The miscount is inherited verbatim from the sibling ticket's `## Out of scope` block, which introduces the same list with "six stale figures", so a worker checking one against the other would have found them consistent and still been wrong about the population. All nine are stale; every one was re-measured at this ticket's base and none matched. The claim that **`47,898` and `47,899` fall past the end of the identity** is **verified**: the identity measures `40,132` bytes, so both offsets exceed it, and that is the reason this is p1 rather than p2 — a reader is directed to an offset that does not exist. The two in-range offsets are wrong in the quieter way and the ticket did not say so: `3,104` and `3,106` resolve to real bytes that are not the ones they name.

**Reported: the document's differing-position count and its pinning account are correct.** Do not "fix" those. A repair that overshoots into correct text is the failure this ticket most invites, because the surrounding figures are wrong.

## What closes this

The stale figures replaced the way the sibling ticket replaced its own: **name the property and the constant that pins it, rather than copying a value into prose.** The document already contains the precedent in its own voice — the neighbouring paragraph reading *"Measurement, and it is now pinned by a test rather than carried as prose here."* Follow that sentence's lead; it is the house style for exactly this situation, written before the drift happened.

Do not restate the measured numbers as fresh prose figures. They will decay the same way on the next identity step, which is the whole lesson of the sibling ticket: an unasserted number that looks measured is worse than no number, and the `v15 -> v16` step moved these by tens of thousands of bytes **downward** while every reader's instinct was that an envelope only grows.

**Where a figure genuinely has no pin behind it**, say so in the text and either propose the assertion — naming the construction and the value, for a `crates/**` ticket to carry, since that is out of scope here — or state the property qualitatively. Do not leave a bare number with no owner.

Before closing, enumerate every numeral in the document and classify each as pinned, spec-constant, dated measurement, or unowned. **Report the census with its counts**, so "no others" is distinguishable from "did not look". The sibling ticket did this over its whole file and found zero survivors, which is what made its result trustworthy.

## Outcome

**Per-Fact audit, at base `97282deffd91924ab35e447626a7e0022176d673`, each Fact re-read at that base rather than inherited.**

| Ticket claim | Verdict | Evidence |
| --- | --- | --- |
| `36,832` appears in `docs/artifact-abi.md` and in no other document | **verified** | `grep -rn "36,832\|36832" docs/` returns the one hit at `docs/artifact-abi.md`, the paragraph anchored *"the carrier reaches artifact identity"* |
| `DIFFERING_CARRIER_POSITIONS = 68` is asserted in `crates/tiler-artifact/src/program/codec/tests.rs` | **verified** | the constant is declared there and compared in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity`; the test passes at this base |
| "six figures" are stale | **imprecise — there are nine**, repaired above | the ticket's own list is `1 + 4 + 4` |
| identity length `48,584`, measured `40,132` | **verified false in the document; the replacement value confirmed** | re-measured `40,132` for both members of the pair |
| offsets `3,104` / `3,106` | **verified false**, and stale in a way the ticket did not state | measured `3,111` / `3,113` — in range, wrong bytes |
| offsets `47,898` / `47,899` fall past the end of the identity | **verified** | measured `39,446` / `39,447`, against a `40,132`-byte identity; the stated pair exceeds it |
| lengths `90,806` / `45,457` / `73,556` / `36,832` | **verified false** | measured `39,781` / `39,865` / `31,212` / `31,296` |
| the document's differing-position count and its pinning account are correct | **verified — not touched** | `68` is pinned and its test is green at this base; the forged pair's identity difference measures exactly four positions, as the document says |

**Measurement, at `97282def`, `cargo nextest run -p tiler-artifact -E 'test(temporary_probe_for_artifact_abi_figures)' --no-capture`, with a temporary probe test appended to `crates/tiler-artifact/src/program/codec/tests.rs` and reverted before any commit (`git status --porcelain` empty, `git diff --stat` empty).** Nothing was derived arithmetically; each figure was regenerated from the construction that produced it — the forged pair from `bf16_input_envelope()` against `envelope_of(&default_artifact())`, the producer pair from `bf16_pointwise_artifact()` and `f32_pointwise_artifact()`.

| Figure | As stated | Measured at `97282def` | Delta |
| --- | --- | --- | --- |
| forged pair, canonical identity length | 48,584 (each) | **40,132** (each) | −8,452 |
| forged pair, identity differing positions | 4 | **4** | unchanged |
| forged pair, component carrier tag offset | 3,104 | **3,111** | +7 |
| forged pair, component access tag offset | 3,106 | **3,113** | +7 |
| forged pair, binding row offset | 47,898 | **39,446** | −8,452 |
| forged pair, binding row offset | 47,899 | **39,447** | −8,452 |
| producer, BF16 envelope | 90,806 | **39,781** | −51,025 |
| producer, BF16 identity | 45,457 | **39,865** | −5,592 |
| producer, F32 envelope | 73,556 | **31,212** | −42,344 |
| producer, F32 identity | 36,832 | **31,296** | −5,536 |

The six lengths reproduce the sibling ticket's numbers exactly, which is corroboration rather than inheritance: they were measured here independently and only compared afterwards. The four offsets are new — the sibling measured lengths and the count, never the positions.

**The two forged-pair offsets moved in opposite directions, which no reader would have guessed.** The component tags moved *forward* by 7 while the binding row moved *backward* by 8,452. Adding a constant to all four, the obvious "recompute", would have been wrong at every one of them.

**Route — retire, do not refresh, and it is the sibling's route for the sibling's reason.** The nine figures are removed from `docs/artifact-abi.md` and replaced with the properties they existed to evidence. The producer pair's two inequalities are already pinned and are now named in the document instead of restated as digits: `a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity` for the length inequality, `the_bf16_artifact_and_its_f32_twin_are_two_artifacts` for the identity inequality. The forged pair's structural description of *which* bytes differ is retained, because it is what the offsets were a lossy spelling of and it does not decay. A dated correction block records the retirement, the direction of the drift, and why an offset in particular is the worst of these to carry in prose.

**Proposed pin, for a `crates/**` ticket — the one figure with no owner.** The forged pair's identity *length equality* and its *count of four differing positions* survive as properties and nothing asserts either. Both belong in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` (`crates/tiler-artifact/src/program/codec/tests.rs`), which already derives `identity_at_bf16` and `at_f32.canonical_identity()` and already pins the envelope's count: add `DIFFERING_IDENTITY_POSITIONS: usize = 4` beside `DIFFERING_CARRIER_POSITIONS: usize = 68`, assert the two identity byte runs are equal in length, and assert the differing count against the new constant. The two constants must stay separate subjects — the envelope's count covers two digests and can move by coincidence, which is why its own comment forbids simplifying it to the arithmetic, while the identity's count is the two tag pairs and nothing else. The offsets are deliberately **not** proposed for pinning: they moved in both directions at one step and carry nothing the structural description does not.

## Numeral census, `docs/artifact-abi.md`, whole file at `97282def`

Enumerated mechanically, then every survivor resolved by reading its site. `grep -o '[0-9][0-9,]*\(\.[0-9][0-9]*\)\?'` yields **708 digit-run occurrences over 163 distinct tokens**; a second pass over spelled-out numerals yields **259 occurrences over 23 distinct words**.

**Set aside as identifiers rather than quantities (not classified further):** 16 dates, 45 ADR and decision-record references, 165 domain-version tags (`tiler.*.vN`), 55 manifest and component schema versions (`N.0`), 17 hex tag literals, and the dependency and platform version strings `sha2` 0.11.0, macOS SDK 26.5, macOS 27.0, MSRV 1.89, CUDA compute capability 7.0, FIPS 180-4. Of the spelled-out population, 249 of the 259 occurrences are `one`/`two`/`three`/`four`/`five`/`six`/`seven`/`eight`/`nine` used as ordinary determiners in prose ("one authority", "two checks"), not as measurements.

**The remainder falls on 24 sites, every one read.** Three carry only the dependency and platform version strings already set aside, leaving **21 sites carrying a quantity**. Four of the 21 are *mixed* — they hold figures of more than one class — so they are named under each class they belong to and the class counts below sum to 27 rather than 21. The four are the `v15 -> v16` step block, the differing-position paragraph, the identity-growth ladder, and the received-opaque-identity bounds.

- **Pinned in a workspace test or crate constant — 9 sites.** `HEADER_BYTES` is `69` in both `crates/tiler-artifact/src/program/codec/encode.rs` and `.../proof/codec.rs`, once per framing-header table. `tiler_digest::DIGEST_BYTES` is the `32`-byte digest width. `FIXED_CONTENT_BYTES` is `65_313` in `crates/tiler-build/src/metal_plan.rs`, read at this base, and is the terminal total of the `v15 -> v16` block. `DIFFERING_CARRIER_POSITIONS` is `68`. The `seven`, `four`, and `eighteen` governed-domain counts are asserted against `core::mem::variant_count` in a `const` block. The two "Governed budgets" paragraphs (256 MiB, 64 MiB, 16 MiB, 4 KiB, 1,024, 4,096, 256, 64, 8 MiB, 128, 64 KiB, 256 proof cases) each name a crate constant enforced on both sides. The received-identity bounds paragraph names `MAX_KERNEL_IDENTITY_BYTES` (16 MiB) and `MAX_ARTIFACT_IDENTITY_BYTES`, both read directly, and carries `1,121` as the fixture length in `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it`, which asserts the property that figure exists for — that a real reduction's kernel identity exceeds the shared bound. The identity-growth ladder's region-shape budgets `62` / `80` / `3` are pinned in `DeterministicBudgets::governed` and exercised by `crates/tiler-compiler/tests/region_search_budget_coverage.rs`.
- **Spec constant, or this document's own wire definition rather than a measurement of it — 8 sites.** The two framing-header tables (offsets 0/8/12/16/17/25/33/37 and widths 8/2/2/1/8/8/4/32). The `0x01` digest-algorithm tag. The canonical NaN payloads `0x0000_7fc0` and `0x7fc0_0000` with their thirty-two and sixteen bits. The header flip sweep, whose `69` is the header width above. Subgroup widths 32 and 64 with their five and six combine steps. The `28` pinned provenance bytes. The embedding gate's 1 MiB per invocation, 32 invocations, and 3.2 MiB per package, which is a declared gate rather than an observation.
- **Dated measurement, anchored to a named commit, host, or retained artifact — 6 sites.** The `v15 -> v16` step block (65,308 / 41,113 / 41,116 / 24,134 / 24,136 / 62,183 / 62,187 beside the pinned total). The ADR 0103 manifest-digest block at commit `eee734cf` (114,059 / 57,978 / 56,081 / 49.17% / 88,069 / 31,988 / 22,911 / 2,974 / 56,105). The identity-growth ladder re-run 2026-08-07 and retained at a named `spikes/program-planning/identity-growth/results/…/growth.tsv` (3530n + 723, 134n + 149, 19,011, 148/149, 1,046,326, 1,053,386, 219,583, 41.9%, 283%). The decoder-allocation block linked to its research note (226,214 / 4,000 / 1,569,620,906 / 670,658 / 2,340 / 2.48× / 3.23× / 31-fold / 15.0× / 6,938.7× / 1,569,451,274 / 768,193). The 30 retained proof cases on one Apple M4 Max corpus. And the kernel-identity paragraph, which is the one flagged below.
- **Explicitly retired in place by a dated correction, kept only as the narrative of what moved — 2 sites.** The `3525n + 727` ladder and its derived figures (19,038 / 695 / 50 / 51 / 219,277 / 41.8%), struck by the 2026-08-07 correction directly beneath them. The `40` and `67` earlier readings of the differing-position count, kept inside the paragraph that explains why that count is measured and never derived.
- **Unowned — 2 sites.** This ticket's nine figures, repaired above. And the kernel-identity paragraph flagged below, whose five unpinned lengths I could not regenerate within this ticket's scope.

**One sibling flagged and deliberately not repaired, because I could not verify it and will not guess.** The "Governed budgets" paragraph anchored at *"the three identities never shared a subject"* states six kernel-identity byte lengths — 736, 1,483, 1,845, 1,700, 2,279 beside the pinned 1,121 — in the present tense ("measures 736 bytes"), bounded to "this checkout (Apple M4 Max, macOS, the pinned toolchain)" but to no commit and no date. Five of the six have no pin. That is the same shape as this ticket's defect, and the kernel identity has stepped since (`tiler.kernel.v7`, `tiler.kernel-program.v11`), so they are plausibly stale; the pinned `1,121` would not catch it, because its assertion is only that the value exceeds `MAX_OPAQUE_IDENTITY_BYTES`. Regenerating them needs the serial-`f32`-sum kernel built at one contributor and at ranks 3 to 8 through `crates/tiler-conformance/src/serial_sum.rs`'s `serial_sum_program` and `compile_under`, which is a different subject from this ticket and a real measurement job. **Not repaired, not asserted either way** — it wants its own narrow `contracts/artifacts` ticket, and an arithmetic refresh of it would be exactly the failure this ticket exists to record.

## Later follow-through — 2026-08-09

Both *implementation* remainders above are complete. The six kernel-identity figures were re-measured, dated, and qualified by [`date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi`](date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi.md), which also corrected this record's false proposed construction: the named conformance helper was rank-two-only and could not regenerate ranks three through eight. The surviving unpinned BF16-versus-F32 identity property was then made executable by [`pin-the-differing-identity-positions-beside-the-carrier-positions-constant`](pin-the-differing-identity-positions-beside-the-carrier-positions-constant.md): `DIFFERING_IDENTITY_POSITIONS` is now distinct from the carrier-envelope count, and the test first proves equal identity lengths before counting the four structural tag positions. The offsets remain deliberately unpinned. The Outcome's proposed pin work and the unowned-quantity account of the nine absolute figures are therefore historical for *implementation*.

**Correction — 2026-08-10.** The 2026-08-09 close sentence claiming "no live remainder remains under this ticket" is false while this ticket's own authored contracts prose still denies the landed pin. Live `docs/artifact-abi.md` still carries the paragraph beginning `What is left unpinned, stated rather than left for a reader to assume.`, which claims that equal identity length and the four differing identity positions have "no test asserts either" and that "Until that lands… not a guarantee." Those properties are now asserted in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` via length equality and `DIFFERING_IDENTITY_POSITIONS = 4`. The implementation pin landed under `implementation/artifact` only and did not rewrite contracts prose. Status stays `done` for the primary retirement deliverable (absolute figures out; properties named; census and route recorded). Residual product debt under `contracts/artifacts`: retire or rewrite that "What is left unpinned" paragraph into a house-style pin citation (name `DIFFERING_IDENTITY_POSITIONS` and the length-equality assert), optionally framing the forged-pair Measurement's "exactly four byte positions" the way **68** is framed, without reintroducing absolute offsets. Do not reopen absolute length pinning.
