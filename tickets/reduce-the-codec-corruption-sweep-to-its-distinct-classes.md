---
id: reduce-the-codec-corruption-sweep-to-its-distinct-classes
title: Reduce the codec corruption sweep to the distinctions it establishes
status: todo
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
