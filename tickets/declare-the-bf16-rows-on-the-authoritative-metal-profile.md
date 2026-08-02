---
id: declare-the-bf16-rows-on-the-authoritative-metal-profile
title: Declare the measured BF16 dispatchability and subnormal rows on the Metal profile
status: in-progress
priority: p1
dependencies: [admit-a-bf16-scalar-arithmetic-subject, measure-macos-apple9-bf16-under-unified-msl4-profile]
related: [spike-bf16-through-the-second-dtype-seams, construct-and-bind-the-first-authoritative-metal-compile-profile, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, decide-per-dtype-dispatchability-as-a-target-capability, measure-apple-numerics-on-physical-ios-device, record-the-compilation-selection-in-target-measurement-provenance]
scopes: [implementation/build, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, target-profiles, metal, apple-targets]
claimed_from: todo
assignee: agent-bf16-profile-r2
lease_expires_at: 1785705719
---
## User-visible outcome

The authoritative Metal compile profile carries the measured BF16 facts: dispatchable on the macOS row, explicitly unsupported on the iOS Simulator, and `Unknown` on the unmeasured iOS device. A BF16 program routed at a family that cannot run it is refused **before** the routing commit rather than failing at pipeline creation after it.

## Why both rows land together

**Fact.** `BoundMetalCompileDeclaration` (`crates/tiler-build/src/metal_declaration.rs`) declares `f32` alone: one measured dispatchability row and six honourability rows, all over `ScalarArithmetic::f32()`. Its own ticket states "Do not infer F16 or BF16 from F32, do not claim BF16 on either iOS family" and names this spike's successor as the first non-F32 use of the mechanism.

**Fact.** The profile descriptor is one identity. A dispatchability row and a numerical row both change its bytes, and the golden that pins them is one fixture. Landing them separately would rebaseline the same golden twice and leave an intermediate commit whose profile claims BF16 is dispatchable while saying nothing about its arithmetic — a profile that is worse than either endpoint. They are merged for that reason and no other.

**Measurement, and its exact boundary — corrected 2026-08-02, and the correction is what blocks this ticket.** From the retained record `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv` on an Apple M4 Max, macOS 27.0 build 26A5388g, Metal 32023.883, Xcode 26.6, **at `probe.fixed_flags -std=metal3.1` against `environment.family.macos.requested_target air64-apple-macos13.0`** — findings 24 and 26 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md):

- macOS: `device_bfloat_support supported`; BF16 arithmetic **flushes** subnormals, sign-preserving (`8040 → 8000`), across all three math modes, at `-O0` and `-O2`, on both compilation paths, with an execution witness on every verdict. `materialize_bf16` returns all eight operands unchanged, so the flush is a property of arithmetic and not of the buffer round trip.
- iOS Simulator: compiles and links every `bfloat` module, then fails pipeline creation with `XPC_ERROR_CONNECTION_INTERRUPTED`. The arithmetic-free `materialize_bf16` is refused too, so the refusal is about the **format**, not one operation.
- iOS device: never asked. `Unknown`, and it stays `Unknown`.

## Blocked (2026-08-02) — the measurement is on a compilation the profile refuses by name

A dispatch attempted this ticket at base `4d08a3f` and stopped without editing `crates/`. The derivation, so a reader can refute the elimination rather than only the conclusion:

**Fact — the two omitted boundary components decide admissibility.** The paragraph above originally named five components of the retained record's boundary — host, OS version, OS build, Metal build, Xcode — and omitted the language standard and the requested target. Those two are the ones this ticket turns on. The record carrying BF16 is `-std=metal3.1` / `air64-apple-macos13.0`; the authoritative profile is MSL 4.0 / macOS 26.0, sourced from `2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883`, whose `probe.dtypes` is `f32` alone.

```sh
for f in spikes/apple-targets/results/*/record.tsv; do
  printf '%s\t%s\t%s\n' "$(grep -m1 '^probe.fixed_flags' "$f" | cut -f2)" \
    "$(grep -m1 '^probe.dtypes' "$f" | cut -f2)" "$f"
done
```

**Fact — three independent authorities refuse the transcription, each stronger than this ticket.** The authority ledger `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` line 227 ("Reusing the older MSL 3.1 / macOS 14.0 record for this profile would attribute measurements to a compilation that did not produce them") and line 247, which names BF16's MSL 3.1 macOS measurement and the iOS-Simulator refusal together and says "neither reaches this profile". `measure-macos-apple9-f32-under-unified-msl4-profile` line 19 ("not evidence for a new MSL 4.0/macOS 26 profile merely because the same host and compiler accept both") and line 23, whose measurement boundary names BF16 among the things not to generalize to. And `crates/tiler-build/src/metal_declaration.rs`, whose live test `the_declaration_does_not_carry_the_superseded_msl_3_1_record` asserts the superseded record absent. An adopted ledger and a `done` p0 measurement ticket both outrank a ticket claim.

**Fact — carrying a second, honestly-labelled source is not available either.** `TargetCompileProfileMeasurementSource` holds compiler builds (role, implementation, version, build) and an execution environment (platform, version, build, architecture, hardware) and nothing else. Every one of those fields is byte-identical between the two records; only `-std` and the target triple differ, and the vocabulary holds neither. An MSL 3.1-sourced BF16 row would encode identically to an MSL 4.0-sourced one, so the profile could not state the distinction even if a worker wanted to. `record-the-compilation-selection-in-target-measurement-provenance` owns that gap.

**The iOS halves are blocked separately and harder.** Two of the three required answers need an authoritative iOS-Simulator profile and an authoritative iOS-device profile. Neither exists. `first-authoritative-ios-metal-compile-declaration` is `deferred` — Tom deprioritized iOS on 2026-08-01 — and its body records why the retained corpus cannot supply one: the MSL 4.0 record has no iOS row at all, the iOS rows that exist are the superseded MSL 3.1 ones, and `IOsDevice` has no execution-side row. A simulator profile carrying only a BF16 refusal would be a dead profile, which this ticket's own required evidence rules out by demanding `f32` resolve `Dispatchable` on all three.

**Eliminated, with reasons.** Declaring the BF16 rows under the existing MSL 4.0 source — asserts a compilation that did not produce them, refused by all three authorities above. A second MSL 3.1-labelled source — not representable, per the paragraph above. A separate MSL 3.1-scoped production profile — reintroduces the record both prototypes migrated off, cannot serve delivery (`crates/tiler-macros/src/delivery.rs:119` pins `PROFILE_MSL_VERSION` to `Metal4_0`), and contradicts this ticket's own "do not add a parallel constructor". Re-measuring inside this dispatch — `spikes/apple-targets/**` maps to `research/apple-targets`, a scope this ticket does not hold, so it would be a guard escape; it is filed instead.

**What lands nothing.** No `crates/` change is reachable, and `docs/dtype-support.md`'s BF16 `Target-family dispatchability` cell must stay `architectural seam`, because moving it to a stated claim without the rows behind it is the promotion that document exists to prevent.

## Implementation keys

- Extend the existing `BoundMetalCompileDeclaration` and its authority ledger. **Do not** add a parallel constructor or a second backend dtype list; the profile-construction ticket explicitly forbids it.
- The macOS BF16 dispatchability row is `Dispatchable` from a measured source. The iOS-Simulator row is `Unsupported` from a measured source carrying the exact diagnostic. The iOS-device row is **absent**, which is `Unknown` — not `Unsupported`, because nobody asked.
- The BF16 subnormal rows project the measured flush through the same `MetalSubnormalArithmeticFacts` path `f32` uses, which already carries the BF16 slot and already refuses to answer from a neighbouring dtype.
- The profile key must change, because the profile's content changed and the key names its content. Decide whether that is a new key or a version bump and state the reasoning; a descriptor change under an unchanged key is exactly the drift ADR 0043 draws its `ProfileKeyMismatch` against.
- No F16 or F64 row. No iOS-device row. No inference from `f32`.

## Required evidence

- ~~BF16 resolves `Dispatchable` on the macOS profile, `Unsupported` on the simulator profile, and `Unknown` on the device profile, all at `AvailabilityPhase::CompileProfile` — three distinct answers, asserted as a matrix whose shape is checked rather than three independent facts.~~ ~~`f32` resolves `Dispatchable` on all three, so no refusal is a dead profile.~~

**Narrowed to the macOS family at integration, 2026-08-02, and the iOS half is split out.** BF16 resolves `Dispatchable` on the **macOS** profile at `AvailabilityPhase::CompileProfile`, and `f32` still resolves `Dispatchable` there, so the row is not a dead profile. The two iOS answers move to [`declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles`](declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles.md).

**The derivation, so it can be refuted rather than only the conclusion.** The three-family matrix needs an authoritative iOS-Simulator profile and an iOS-device profile. Neither exists, and `first-authoritative-ios-metal-compile-declaration` is `deferred` — a parked state that satisfies no dependent — so a ticket requiring that matrix could never reach `ready` no matter what evidence arrived. Both candidates were tested: keeping the ticket whole makes it permanently unreachable, which is the same deadlock `re-point-the-boundary-property-enforcer-edges-after-the-provider-seam-landed` was filed to repair elsewhere in this graph; narrowing makes the macOS half reachable the moment [`measure-macos-apple9-bf16-under-unified-msl4-profile`](measure-macos-apple9-bf16-under-unified-msl4-profile.md) lands, and parks the iOS half behind its own real prerequisite. Only the second survives, so this was derived rather than escalated.

**What the narrowing does not weaken.** The *Why both rows land together* argument above is about the macOS dispatchability row and the macOS subnormal row sharing one descriptor identity and one golden — it is untouched, and those two still land together. The `Unknown`-is-not-`Unsupported` discipline is likewise untouched: with no iOS profile at all, both iOS families remain `Unknown` by absence, which is the correct answer for a question nobody asked.
- `f16` still resolves `Unknown` on every profile, so a measured BF16 row did not fill a neighbour's omission. The existing test asserting this for `f16` against `f32` is the pattern.
- ~~A strict-subnormal-preserving contract is refused for BF16 on macOS with a named numerical gap, since the measured behaviour flushes.~~

**Split on 2026-08-02 after reading the consumer boundary.** The profile can
state the complete BF16 subnormal tables now, but no public request can ask them
the strict-BF16 question: `NumericalContract` documents that every resolution is
for `f32`, its only builder entry point is `strict_f32`, and a pure-BF16 program
is refused at the request boundary with `dtype-f32` before target numerical
feasibility. Substituting `STRICT_F32` would inherit a neighbouring dtype's
contract, while a test-only resolver in `tiler-build` would prove no caller path.
[`state-and-check-a-bf16-numerical-contract`](state-and-check-a-bf16-numerical-contract.md)
owns the consequential compiler/public-boundary work and depends on this
profile ticket; this ticket owes the complete exclusive tables and their
identity, not an unreachable consumer proof.
- The profile descriptor's byte length and identity are recorded before and after.

## Closes when

The macOS profile carries measured BF16 `Dispatchable` while retaining F32
`Dispatchable` and F16 `Unknown`; the measured BF16 flush is declared as complete
exclusive input/result tables and its exact host/toolchain/family boundary is
stated in the ledger rather than generalized; the new rows' own negative
perturbations are observed failing; the profile key and descriptor movement are
recorded; and `docs/dtype-support.md` moves only BF16's numerical-honourability
and target-family-dispatchability cells to the exact stated claims. The iOS
matrix and the end-to-end BF16 contract refusal remain with their split tickets.

## Graph maintenance

- Depends on `admit-a-bf16-scalar-arithmetic-subject`: without a BF16 subject the honourability half is unstatable, and this ticket lands both halves at once. That dependency is satisfied — the subject is statable and no BF16 fact was declared.
- Depends on `measure-macos-apple9-bf16-under-unified-msl4-profile`, added 2026-08-02. The macOS half is unstatable until BF16 is measured on the profile's own compilation row; see the blocked section above.
- `measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes` owns the MSL 3.1 BF16 measurement and is `done`. **Do not re-measure it here**; the new dependency re-measures the same dimensions at MSL 4.0 rather than superseding that record, which remains correct evidence for its own row.
- The two iOS answers are gated on `first-authoritative-ios-metal-compile-declaration`, which is `deferred`. It is deliberately **not** a dependency — a parked state never satisfies a dependent — so this ticket's reachable outcome once the MSL 4.0 BF16 measurement lands is the macOS half alone. Splitting the iOS half into its own ticket, or narrowing this one's stated outcome to macOS, is the coordinator's call and is not assumed here.
- `measure-apple-numerics-on-physical-ios-device` is `deferred` and must not be a dependency — `deferred` never satisfies a dependent. It is the only route to closing the iOS-device `Unknown`, and it stays `related`.
- A differing physical-iOS result would reopen `declare-metal-numerical-honourability`; say so rather than assuming the family agrees.
- `state-and-check-a-bf16-numerical-contract` depends on this ticket and owns the
  first caller-visible consumption of the BF16 subnormal tables. Do not widen
  this ticket into the compiler's F32-only numerical-contract boundary.
