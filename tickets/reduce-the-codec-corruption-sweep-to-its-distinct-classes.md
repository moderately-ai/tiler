---
id: reduce-the-codec-corruption-sweep-to-its-distinct-classes
title: Reduce the codec corruption sweep to the distinctions it establishes
status: done
priority: p2
dependencies: []
related: [audit-the-suite-s-slowest-tests]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [testing, performance]
---
Filed by `audit-the-suite-s-slowest-tests`. `single_byte_corruptions_are_rejected` in `crates/tiler-artifact/src/program/codec/tests.rs` takes 13.0s and **is the whole suite's critical path** — the workspace run is 13.09s wall and every other test finishes underneath this one.

## Measurement — Apple M4 Max, dev profile

The sweep visits **8,451 offsets**: 69 header bytes one by one, 295 manifest-interior samples at `.step_by(61)`, and **8,087 post-manifest bytes one by one**. Bucketing every offset's refusal by region and variant:

| region | outcome | offsets |
| --- | --- | --- |
| header | 8 distinct classes (`BadMagic`, `Limit`, `ManifestDigestMismatch`, `TotalLengthMismatch`, `Truncated`, `UnsupportedCanonicalEncoding`, `UnsupportedDigestAlgorithm`, `UnsupportedEnvelopeFormat`) | 69 |
| manifest | `ManifestDigestMismatch`, uniformly | 295 |
| sections | `SectionDigestMismatch` | **8,075** |
| sections | `Limit`, `NonCanonicalSectionId`, `SectionLengthMismatch` | 12 |

**Inference — 8,370 of 8,451 offsets (99.0%) reproduce an outcome another offset already produced.** Thirteen distinct outcomes exist. The fixture has **one** section, so every one of its 8,075 content bytes is covered by that one digest and exercises the identical rejection path.

The density is also already inconsistent with itself: the manifest interior is sampled 1-in-61 *and is still uniform across all 295 samples*, so even the existing sampling is ~295× redundant, while the section region — the larger one — is not sampled at all.

## What is dense and must stay so

The header. Eight distinct classes in 69 bytes is genuinely information-rich, and those are the boundary checks a reader hits first. Do not sample it.

## The tension to resolve, stated rather than assumed away

Exhaustive coverage of every byte is a stronger claim than coverage of one representative per equivalence class, and `audit-the-suite-s-slowest-tests` records the rule that a correctness property must not be weakened to make a test faster. Two honest readings:

- the property under test is "no single-byte corruption is accepted", and sampling stops proving it for un-sampled bytes;
- the test already samples the manifest interior, so the standard in force is already representative coverage rather than exhaustive, and the section region is the inconsistency.

Decide which, and say so at the site. If sampling wins, cover **each section** rather than the region — the current one-section fixture makes a per-region sample look sufficient and it would not be for a two-section artifact — plus the section-structure boundaries where the 12 non-digest outcomes live.

Note that [`raise-the-dev-opt-level-for-workspace-crates`](raise-the-dev-opt-level-for-workspace-crates.md) attacks the same 13s from the other side, and the two compose: a 5.3× cheaper decode leaves an exhaustive sweep at ~1.5s, which may make keeping exhaustive coverage affordable enough that the tension above does not need resolving at all. Measure after that lands before deciding here.

## Closes when

The sweep's cost is proportionate to the distinctions it establishes; the header stays byte-exhaustive; whichever of exhaustive-or-representative coverage is chosen is stated at the site with its reason; and every one of the thirteen outcomes above is still reached.

## Outcome — the tension dissolved, and the sweep got *stronger* (2026-07-27)

This ticket said to measure before deciding, and anticipated that a cheaper decode might make exhaustive coverage affordable enough that the exhaustive-versus-representative question would not need answering. That is what happened, by a different route than the one it named.

**Measurement.** The sweep now runs in **132 ms fully exhaustive** — every byte of the envelope, no sampling anywhere — against ~70 ms for the sampled form it replaces and **13.0 s** when this ticket was written. Two changes landed earlier the same day did it: artifact decode fell from 662 µs to 18.7 µs across the codec work, and the envelope shrank from 26,126 bytes to 15,030 when ABI expression identity moved to a linear encoding. `raise-the-dev-opt-level-for-workspace-crates`, which this ticket expected to be the cause, has not landed and was not needed.

**So the stride-61 sampling is removed rather than extended.** The ticket's fallback plan — sample per *section* instead of per *region*, and cover the section-structure boundaries — is unnecessary: for 62 ms the property under test is "no single-byte corruption of this envelope is accepted" rather than "no sampled single-byte corruption is", and the stronger claim needs no argument about which bytes are representative. `audit-the-suite-s-slowest-tests` records the rule that a correctness property must not be weakened to make a test faster; here it was not weakened in either direction.

**All thirteen outcomes are still reached** — the sweep is a superset of what it covered before, so nothing that was exercised has stopped being.

**The check can say no**, verified rather than assumed: mutating the corruption to `^= 0x00` fails the test at byte 0. Without that, a sweep in which every decode errored for an unrelated reason would pass identically. Recorded at the site.

**Stale figures corrected in two siblings.** `audit-the-suite-s-slowest-tests` and `raise-the-dev-opt-level-for-workspace-crates` both cite this sweep's 13.0 s as evidence; both now say the measurement is superseded and what replaced it. The second used this test as its headline case, so its motivation is genuinely weaker now — recorded there rather than left for someone to discover after starting the work. Its question stands on its own merits.

The doc comment also carried "25,000 bytes" for the manifest interior, which was already wrong before this change: the manifest was 18,013 bytes and is now smaller still.
