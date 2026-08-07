---
id: declare-evaluation-order-preservation-in-the-target-profile
title: Declare evaluation-order preservation in the target profile
status: done
priority: p2
dependencies: []
related: [measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order, admit-a-refutation-only-derived-bound-conformance-oracle]
scopes: [implementation/metal, implementation/compiler, implementation/build, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, target-profiles, numerics, public-boundary]
---
## User-visible outcome

A target profile declares, per math mode, whether the backend compiler preserves an emitted floating-point evaluation order, so the permitted-divergence oracle's pinned-order premise is consulted from a declared fact instead of being asserted by the flags Tiler happens to pass. Today `MetalTargetFacts` (five fields) and `CapabilityAxis` (seven) declare nothing about it, which [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md)'s item 5 records as the gap.

## The measured basis

[Finding 34](../docs/research/apple-targets/numerical-behaviour.md): on the named macOS row, an emitted two-by-two split is re-serialized under `relaxed` and `fast` on both compilation paths and preserved under `safe` in every measured cell. The declaration this ticket adds is honest exactly per row and mode: `Preserved` only where measured, `Unknown` elsewhere (including the qualified numerical row, which finding 34 was not taken on — re-measuring there is in scope or explicitly deferred with the toolchain-authorization constraint named).

## Boundary

A new target-profile field is a public surface under ADR 0075: implement as a labelled draft with an acceptance node parked for Tom, stepping the complete-declaration domain version only if previously-encodable bytes move (an appended row family under per-tag framing does not). Silence must stay fail-closed: a profile that declares nothing about the property answers `Unknown`, which never reaches an executable frontier.

## Closes when

The field exists as a draft with its node, the macOS row declares the measured values with finding 34 as provenance, every other profile answers `Unknown`, and the oracle derivation's item 5 cites the declaration rather than the absence.

## Outcome — 2026-08-06

**The field exists as a labelled draft, no profile declares a row, and the second clause of "closes when" is delivered as an explicit deferral rather than as a declared macOS row. The reason is the toolchain row, and it is the finding this work turned up.**

### The declaration's shape

A new declared-fact family on the compiler's `TargetProfile`, a peer of the synchronization family rather than a twelfth numerical dimension. Public draft surface in `crates/tiler-compiler/src/target.rs`:

- `BackendArithmeticLicence` — `Withheld` / `Granted`, with `key()`.
- `EvaluationOrderPreservation` — `Preserved` / `NotPreserved`, with `key()`. No `Unknown` variant: absence is the `Unknown`, as it is for `DTypeDispatchability`.
- `EvaluationOrderResolution` — `Preserved` / `NotPreserved` / `Deferred { available_at }` / `Unknown`.
- `TargetProfileBuilder::declare_evaluation_order_preservation` and `::declare_measured_evaluation_order_preservation`, keyed by `(exact scalar subject, licence, phase)`; the verdict is excluded from the key for the reason `DuplicateSynchronizationRealization` excludes it.
- `TargetProfile::evaluation_order_preservation(&subject, licence, available_phase)`.
- `TargetProfileBuildError::DuplicateEvaluationOrderPreservation { licence, phase }`.

**Why the key is a licence and not a math mode.** The ticket asks for a per-math-mode declaration and the measurement is indexed that way, but `safe`/`relaxed`/`fast` are `-fmetal-math-mode` tokens — one backend driver's option values — and `MathMode` lives in `tiler-metal-aot`, which the compiler core may not learn. What finding 34 actually attributes the behaviour to is the emitted licence set: the reordering fires exactly where the operations carry LLVM's `reassoc`, which the finding names as "the licence that authorizes regrouping", and `relaxed` and `fast` differ only in `nnan`/`ninf`, which no measured cell attributes an order change to. So `Withheld` is `safe` and `Granted` is `relaxed` and `fast` together, all three modes covered with nothing inherited between them. A future measurement separating `relaxed` from `fast` needs a third value here — a build error at every match, never a silent read of a neighbour's row. The two-value collapse is the one place this delivery is narrower than the ticket's literal wording, and it is deliberate.

**Why it is not a twelfth `NumericalDimension`.** `CANONICAL_DIMENSIONS` states what a caller's contract may grant *Tiler*; this states what Tiler's emission grants the *backend translator*. Adding it to the dimension list would have made it grantable by a contract, which it is not, and would have reopened a vocabulary the oracle derivation explicitly leaves at eleven.

### The domain step-or-not derivation

**`COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` stays at `v11`.** The rule is a byte rule: the domain steps when previously-encodable bytes *move*. The family is written last, behind `EVALUATION_ORDER_DOMAIN` (`tiler.target-profile.evaluation-order-preservation.v1`), and **only when it holds a row**, so every profile that existed before it — the governed baseline, `BoundMetalCompileDeclaration::first_macos_apple9`, every test profile — encodes byte for byte what it encoded at `v11`. Its sources join the shared source table through an iterator that is empty for those profiles, so no source index shifts either. Injectivity survives the conditional section because every earlier section is self-delimiting: two descriptors agreeing on the `v11` prefix agree on every earlier row, and the remainder is then either empty or this family's separator-led bytes. An empty family and an absent family denote the same thing here — `Unknown` for every subject and licence, which no admission path acts on differently — so nothing is lost by not writing a zero count. The contrast with `v11` itself is stated at the encoding site: the synchronization family frames itself *unconditionally* and therefore had to step `v10`, which was a choice about what silence should record and not a rule this family breaks.

**Checked, not asserted.** `the_declared_profile_states_one_barrier_realization`'s 1,999-byte pin on the bound macOS descriptor is unchanged and still green, and a perturbation that emits the family unconditionally moves it to 2,139 and fails.

### Rows declared

**None, on any profile, and that is the honest answer.** Finding 34 was measured on a **neighbouring toolchain row** — Xcode 27.0 build `27A5228h`, SDK 27.0 build `26A5388f`, offline `Apple metal version 32023.921 (metalfe-32023.921)`. Every row `FIRST_MACOS_APPLE9` carries was measured under Xcode 26.6 build `17F113`, SDK 26.5, offline `metalfe-32023.883`. The property is a property *of the backend compiler build*, and finding 8 records that build moving independently of the OS and of the runtime compiler. Declaring build `.921`'s behaviour on a profile whose plans build `.883` compiles is exactly the inheritance the authority ledger refuses by name everywhere else, and it would refuse *less* visibly than an absent row because it would arrive carrying exact provenance. So `first_macos_apple9` resolves `Unknown` at every phase, for both licences, for `f32` and `bf16` alike, and `the_declared_profile_answers_unknown_on_evaluation_order_preservation` pins that negative at `LaunchPreflight` so a row declared at *any* phase breaks it.

The ledger gains a full row — "Evaluation-order preservation — **absent, and therefore `Unknown`**" — in the ledger discipline: owner, why absent, what the absence costs and does not cost, what must not be substituted (finding 17's reassociation rows are not this row), and the reconsideration trigger. It also gains the fourth `Unknown` entry and Outcome 6.

**The deferral's toolchain-authorization constraint, named.** Two measurements close it and each moves a host toolchain component: re-run the evaluation-order probe against Xcode 26.6 / `metalfe-32023.883`, which needs `xcode-select` moved back off Xcode 27.0; or re-take the ledger's whole numerical row against Xcode 27.0 / `metalfe-32023.921`, which needs every numerical row re-measured and re-transcribed. AGENTS.md reserves changing Rust, Xcode, SDK, simulator, or GPU components for a measurement to Tom, so neither is a step a worker takes. A third route closes nothing: a `relaxed` or `fast` row cannot be declared on this profile at all, because it compiles under `safe` and a row read from another selection would be a different fact about a different compilation.

### The owed oracle sentence

`docs/research/reference` is not in this ticket's scopes. The coordinator owes item 5 of `docs/research/reference/permitted-divergence-oracle.md`'s Part 7 this replacement for its closing sentence — currently "The declaration gap this item opens with is unchanged: no target-profile fact carries the property, and adding one is a public boundary this record still does not presume." — verbatim:

> **The declaration gap this item opens with is closed as a vocabulary and open as a row.** `tiler_compiler::target` now carries the fact as a labelled draft — `TargetProfileBuilder::declare_evaluation_order_preservation` keyed by the exact scalar subject and by `BackendArithmeticLicence`, resolved through `TargetProfile::evaluation_order_preservation`, with silence answering `EvaluationOrderResolution::Unknown` for every subject and licence — so refusal class 3 consults a declared fact rather than the absence of one. No profile declares a row: finding 34 was measured on offline `metalfe-32023.921` and the macOS compile profile's every other row on `metalfe-32023.883`, so the [authority ledger](../target-profiles/first-macos-metal-compile-profile-authority-ledger.md)'s evaluation-order row records the deferral and its two closing measurements rather than attributing one compiler build's behaviour to another's compilations. The exact public surface remains a boundary this record does not presume; `accept-the-evaluation-order-preservation-target-fact` parks it for Tom.

### Filed acceptance node

`accept-the-evaluation-order-preservation-target-fact`, `awaiting-decision`, tagged `public-boundary`/`needs-tom`. It enumerates the exact included surface, the four deliberate exclusions with their consequences, and the two things a reader should check before deciding.

### Tests, each watched failing

Five in `tiler-compiler`, one in `tiler-build`. Perturbations run and observed refusing, then reverted:

1. Silence resolving `Preserved` instead of `Unknown` → `a_profile_declaring_no_evaluation_order_row_resolves_unknown` and `declared_evaluation_order_rows_resolve_per_licence_and_are_not_inherited` both fail (the second on the `bf16` non-inheritance assertion).
2. The family emitted unconditionally → the same fail-closed test's descriptor assertion fails, and `the_declared_profile_states_one_barrier_realization` reports 2,139 against its 1,999 pin.
3. The licence dropped from the duplicate key → the second `declare_…` call is refused where it must succeed.
4. The verdict removed from the encoding → `evaluation_order_subject_licence_and_verdict_participate_in_complete_identity` fails.
5. The duplicate refusal removed → `a_second_evaluation_order_verdict_at_one_phase_is_refused_atomically` fails.
6. A row declared on `FIRST_MACOS_APPLE9` → the `tiler-build` `Unknown` test and the 1,999-byte pin both fail.

`a_later_phase_evaluation_order_row_defers_rather_than_resolving` drives the `Deferred` arm through a `MeasuredFactAuthority::DeviceRuntime` source, so no variant of the resolution is unreachable.

### Checks

`cargo fmt --all --check`; `cargo clippy -p tiler-compiler -p tiler-build --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler -p tiler-build`; `cargo nextest run --workspace` (2,891 passed, 7 skipped); `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard`. **This delta touches `crates/`, so the coordinator's gate cannot carry a previous green result.**

### Scopes added

`implementation/build` (the bound macOS declaration's module doc and its fail-closed test) and `research/target-profiles` (the authority ledger row). `implementation/metal` was declared and **not used**: no `MetalTargetFacts` field was added, because a sixth field with no measured value for this profile's toolchain row would be ceremony, and the per-`MathMode` keying it would have carried is exactly the Metal vocabulary the compiler-side key deliberately avoids.

### Commit

``a8d169c0` carries the whole delivery — the compiler vocabulary, the bound macOS declaration's absent-row account and its fail-closed test, the authority ledger row, and the acceptance node — on `tkt/declare-evaluation-order-preservation-in-the-target-profile` from base `b84b8f81`. This Outcome follows in its own commit so it can name that hash.`
