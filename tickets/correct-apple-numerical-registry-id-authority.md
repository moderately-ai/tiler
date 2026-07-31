---
id: correct-apple-numerical-registry-id-authority
title: Correct Apple numerical registry-ID authority
status: done
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

**Fact — the exact header wording, read on this host.** `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h` declares `@property (readonly) uint64_t registryID API_AVAILABLE(macos(10.13), ios(11.0));` on `@protocol MTLDevice`, abstract "Returns the IORegistry ID for the Metal device", discussion "The registryID value for a Metal device is global to all tasks, and may be used to identify the GPU across task boundaries." The header states a scope and a use and claims no persistence across boots; "same-boot correlation" is a reading of it, not its wording, so the memo quotes the header and states the measured lifetime separately.

**Measurement — the enumerated population is seven records, not four.** This ticket named the 2026-07-25 pair, the 2026-07-27 pair, and the 2026-07-30 macOS-only record. Two of those are actually two records each (covering and exhaustive), and a fifth paired record exists that the ticket did not name: `results/2026-07-24-numerics-families-xcode26.6-metal32023.883/record.tsv` carries `environment.family.macos.device_registry_id` and `environment.family.ios-simulator.device_registry_id`, both `4294968621`. It is retained and cited by the memo as the previous row, so excluding it would have left a paired record outside a check whose whole point is a named, counted population. The check therefore enumerates five paired records and the two 2026-07-30 macOS-only ones, and requires the enumeration to equal every retained record carrying a registry-ID row.

**Measurement — the lifetime the retained `environment.date_utc` rows bound.** `4294968621` at `2026-07-24T20:36:24Z`, `2026-07-25T23:01:16Z`, and `2026-07-25T23:02:18Z`; `4294968452` at `2026-07-28T00:12:48Z`, `2026-07-28T00:13:40Z`, `2026-07-30T21:15:27Z`, and `2026-07-30T21:15:42Z` — same named `Apple M4 Max`, same macOS build 26A5388g, same offline toolchain. The value survived each of those spans and changed at least once between them; **what changed it is unmeasured**, because no reboot or IORegistry event was recorded.

**Fact — one out-of-scope mention remains, and it is accurate for its own row.** `tickets/widen-the-apple-numerical-probe-to-a-second-dtype.md` records "Apple M4 Max (`registryID` 4294968621)" as that ticket's own 2026-07-25 environment row. The value is correct for the record that work produced and the line does not name its date, so a reader comparing it against the current authoritative record would see a spurious difference. It is a historical work record under `project/tickets` rather than research prose and was left unedited; qualifying it needs its own change.

## Implementation keys

Reconcile the prose with both paired retained measurements, preserve the historical raw values, state the exact purpose and measured lifetime of registry ID, and update every research sentence that currently implies cross-record stability. Add a portable check over an explicitly enumerated population: the 2026-07-25 macOS/simulator pair equals `4294968621`, the 2026-07-27 covering/exhaustive macOS/simulator rows equal `4294968452`, and differing values between those measurements are positively accepted. Keep the 2026-07-30 unified macOS-only v7 record out of the pair check because it has no simulator row.

## Required evidence

A reproducible search must find no prose claiming one registry ID across retained records. Tests must name and count the exact expected paired population, pass for each historical measurement, fail when macOS and simulator IDs differ within one measurement, and continue to accept different IDs between measurements. Perturb one within-measurement value and observe failure before restoration. No raw retained measurement may be rewritten merely to make the values agree.

**The reproducible search, run from the repository root.** Every registry-ID mention in the Apple research prose and its harness must name, on its own line, the record it was measured in; the retained records themselves are excluded because they *are* the raw values. A printed line is a registry ID stated without the record it belongs to, which is the exact shape of a cross-record-stability claim:

```sh
grep -rnE '4294968621|4294968452' docs/research/apple-targets spikes/apple-targets \
  | grep -v '^spikes/apple-targets/results/' \
  | grep -vE '2026-07-(24|25|27|28|30)'
```

**Measurement — it prints nothing** on the correcting commit (exit 1 from the final `grep`), and it prints a line when one is planted: a scratch file containing `The Apple M4 Max reports registryID 4294968621 on this host.` under `docs/research/apple-targets/` was reported and then removed, so the empty result is a check that ran rather than a pattern that matches nothing.

**Measurement — the enumerated population and its perturbations.** `test_the_registry_id_agrees_within_a_measurement_and_is_free_between_them` in `spikes/apple-targets/test_numerical_probe.py` enumerates seven retained records — five paired (2026-07-24 families and the 2026-07-25 and 2026-07-27 covering/exhaustive pairs) and two macOS-only (the 2026-07-30 covering/exhaustive named-profile records) — asserts the enumeration equals the set of retained records carrying a `device_registry_id` row with both counts in the failure message, holds each paired record to macOS/simulator equality at its own value, holds each macOS-only record to carrying no simulator row, and asserts positively that the values disagree between measurements. Three deliberate failures were observed: the on-disk 2026-07-25 covering simulator row changed from `4294968621` to `4294968452` failed with `ios-simulator='4294968452', expected 4294968621` and was restored from git; the 2026-07-27 exhaustive `record.tsv` moved aside failed with `the enumerated registry-ID population (7 records) is not the retained one (6 records)` and was restored; and the test carries two in-memory perturbations of its own so both refusals run on every invocation. `uv run --with pytest pytest spikes/apple-targets` passed 89 tests with a resolved toolchain and GPU.

## Closes when

The numerical memo and spike README distinguish correlation from identity, every cited value matches the record it describes, the negative mutation demonstrates the same-run check can fail, and the research-only tests pass.

## Graph maintenance

Keep this ticket related to, but not a dependency of, `construct-and-bind-the-first-authoritative-metal-compile-profile` and `validate-macos-metal-profile-host-applicability`. The production profile is already required not to use registry ID; correcting the prose is important evidence hygiene but must not deadlock the production path.
