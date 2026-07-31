---
id: correct-apple-numerical-registry-id-authority
title: Correct Apple numerical registry-ID authority
status: todo
priority: p1
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, validate-macos-metal-profile-host-applicability]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [apple, numerics, evidence, provenance]
---
## User-visible outcome

The retained Apple numerical prose agrees with its authoritative records and states that Metal registry ID is an IORegistry identifier useful for correlating a GPU across tasks in an active environment, not a durable cross-record hardware identity or a host-applicability predicate.

## Facts and measurement boundary

**Measurement:** the 2026-07-25 retained records and prose report registry ID `4294968621`; the later 2026-07-27 covering and exhaustive records report `4294968452` for the same named Apple M4 Max and still show equality between macOS and the iOS Simulator within each run.

**Fact:** the current research memo still embeds the earlier number while naming the later record as authoritative. The invariant supported by both records is same-run equality between host and simulator, not persistence of the numeric value across boots or runs.

**Fact:** the locally vendored SDK's `MTLDevice.h` documents `registryID` as globally unique across all tasks and usable to correlate a GPU across task boundaries. The retained measurements do not establish persistence across boots or historical records.

**Inference:** registry ID must not be used as durable profile identity, cross-record hardware identity, or runtime eligibility. Device name plus supported GPU family and the exact measured environment are separate predicates; this correction does not itself establish their sufficiency.

## Implementation keys

Reconcile the prose with both paired retained measurements, preserve the historical raw values, state the exact purpose and measured lifetime of registry ID, and update every research sentence that currently implies cross-record stability. Add a portable check over an explicitly enumerated population: the 2026-07-25 macOS/simulator pair equals `4294968621`, the 2026-07-27 covering/exhaustive macOS/simulator rows equal `4294968452`, and differing values between those measurements are positively accepted. Keep the 2026-07-30 unified macOS-only v7 record out of the pair check because it has no simulator row.

## Required evidence

A reproducible search must find no prose claiming one registry ID across retained records. Tests must name and count the exact expected paired population, pass for each historical measurement, fail when macOS and simulator IDs differ within one measurement, and continue to accept different IDs between measurements. Perturb one within-measurement value and observe failure before restoration. No raw retained measurement may be rewritten merely to make the values agree.

## Closes when

The numerical memo and spike README distinguish correlation from identity, every cited value matches the record it describes, the negative mutation demonstrates the same-run check can fail, and the research-only tests pass.

## Graph maintenance

Keep this ticket related to, but not a dependency of, `construct-and-bind-the-first-authoritative-metal-compile-profile` and `validate-macos-metal-profile-host-applicability`. The production profile is already required not to use registry ID; correcting the prose is important evidence hygiene but must not deadlock the production path.
