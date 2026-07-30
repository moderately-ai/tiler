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

The retained Apple numerical prose agrees with its authoritative records and states that Metal registry ID is an observation useful for correlating devices within one run, not a stable hardware identity or a host-applicability predicate.

## Facts and measurement boundary

**Measurement:** the 2026-07-25 retained records and prose report registry ID `4294968621`; the later 2026-07-27 covering and exhaustive records report `4294968452` for the same named Apple M4 Max and still show equality between macOS and the iOS Simulator within each run.

**Fact:** the current research memo still embeds the earlier number while naming the later record as authoritative. The invariant supported by both records is same-run equality between host and simulator, not persistence of the numeric value across boots or runs.

**Inference:** registry ID must not be used as stable profile identity, hardware identity, or runtime eligibility. Device name plus supported GPU family and the exact measured environment are separate predicates; this correction does not itself establish their sufficiency.

## Implementation keys

Reconcile the prose with both retained records, preserve the historical raw values, state the exact purpose and lifetime of registry ID, and update every research sentence that currently implies cross-run stability. Add a portable check that derives the intended same-run equality assertion from each retained record without pinning one global number.

## Required evidence

A reproducible search must find no prose claiming one registry ID across retained runs. Tests must pass for each historical record, fail when macOS and simulator IDs differ within one record, and continue to accept different IDs between records. No raw retained measurement may be rewritten merely to make the values agree.

## Closes when

The numerical memo and spike README distinguish correlation from identity, every cited value matches the record it describes, the negative mutation demonstrates the same-run check can fail, and the research-only tests pass.

## Graph maintenance

Keep this ticket related to, but not a dependency of, `construct-and-bind-the-first-authoritative-metal-compile-profile` and `validate-macos-metal-profile-host-applicability`. The production profile is already required not to use registry ID; correcting the prose is important evidence hygiene but must not deadlock the production path.
