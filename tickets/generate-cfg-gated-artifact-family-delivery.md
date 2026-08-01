---
id: generate-cfg-gated-artifact-family-delivery
title: Generate the cfg-gated delivery half of the artifact-family selection
status: todo
priority: p1
dependencies: [prototype-inline-proc-macro-frontend, prototype-artifact-family-delivery]
related: [prototype-macro-embedding-and-cargo-behavior, record-that-the-frontend-axis-is-review-gated]
scopes: [implementation/frontend]
shared_scopes: [contracts/navigation]
paths: []
tags: [implementation, apple-targets, inline-dx, frontend]
---
`prototype-artifact-family-delivery` landed the driver-side half of ADR 0049 and ADR 0053 — the canonical typed `ArtifactFamilySelection`, its `SelectedFamilies`/`FallbackOnly` delivery policy, and the fan-out to one `MetalTarget` per selected family — as a crate-private draft in `crates/tiler-metal-aot/src/family.rs`. This ticket owns the half that could not land with it, and the reason is a boundary rather than an omission.

**Fact — the remaining half is generated Rust, and an accepted packaging profile puts it elsewhere.** ADR 0053 states: "Generated Rust gates the payload or diagnostic by the family's versioned consumer-target `#[cfg]` predicate. A matching target requires the selected artifact and sees `compile_error!` on build failure; a nonmatching target uses the semantic fallback." ADR 0077 item 1 states that `tiler-metal-aot` "does not emit MSL, does not assemble the target-neutral artifact bundle, and does not implement the expansion cache or the proc-macro layer", and `docs/architecture.md`'s crate table assigns "emit artifact plus runtime/fallback tokens" to the frontend proc-macro crate. A family's consumer-target `#[cfg]` predicate is a fact about a *Rust* target; the driver knows only about `xcrun`. Landing versioned generated-code data in the driver would have given it a second responsibility an accepted profile places on a crate that does not exist.

**Fact — the owning crate cannot be created by the parent ticket.**
`ticketsplease.toml` maps `implementation/frontend` to frontend crate paths that
do not yet exist. Admitting a workspace member also requires
`implementation/workspace` and `implementation/cargo-lock`, which the parent
does not hold. The current Cargo/Makefile gate has no separate Python member
table.

**Fact — the axis was gated on review and is now released to engineering.** `record-that-the-frontend-axis-is-review-gated` recorded that `prototype-inline-proc-macro-frontend` depended on `prototype-public-compiler-api`, whose closing condition was Tom's acceptance of a public boundary. Tom approved the extensible region syntax on 2026-07-30, and `prototype-inline-proc-macro-frontend` is now `todo`. This ticket remains `blocked` until its two declared dependencies deliver; the review gate itself is no longer the reason.

## The work

With a frontend proc-macro crate admitted, implement the delivery half over the parent's `ArtifactFamilySelection`:

- The versioned family-to-consumer-`cfg` predicate map, as versioned Tiler data. The measured distinctions are recorded in `docs/research/macro-environment/proc-macro-build-environment.md`: macOS is `target_os = "macos"`, iOS device is `target_os = "ios"` with an empty `target_abi`, the iOS simulator is `target_abi = "sim"`, and Mac Catalyst is `target_abi = "macabi"`. `docs/integration/frontends.md` calls the map "versioned Tiler data and covered by generated-code tests", so a widened map that does not bump its version is a defect.
- Emission of the gated tokens: for each selected family, either its embedded payload or its retained toolchain/compiler diagnostic as a `#[cfg]`-gated `compile_error!`, plus the semantic fallback for every nonmatching target. Target-neutral semantic, optimizer, verifier, and envelope failures stay unconditional compile errors.
- The named ergonomic profiles that `docs/open-questions.md` Q-ART-008 tracks, expanding to a canonical `ArtifactFamilySelection`. Q-ART-008 names `prototype-artifact-family-delivery` as its owner and its close condition is "named profiles expand to canonical `ArtifactFamilySelection` with generated `cfg` compile-pass/fail tests". Retarget the open question to this ticket, or record why it stays with the parent.

## Tests this ticket owes

`docs/correctness-and-testing.md` states them normatively: "Generated consumer-`cfg` tests cover macOS, iOS device, iOS simulator, Catalyst, and an unrelated non-Apple target. A selected matching family embeds its payload or emits its retained actionable compile error; a nonmatching target compiles the semantic fallback; `FallbackOnly` performs no backend compiler work."

Catalyst is in that list while remaining a deferred family that `ApplePlatform` cannot represent. Its case is therefore that a Catalyst consumer target matches *no* selected family and takes the fallback — never that an iOS-device or macOS payload is relabelled as Catalyst-compatible, which `docs/backends/metal.md` forbids explicitly.

The checked-in probe `spikes/macro-environment/run-family-cfg.sh` already
demonstrates the behaviour on the measured macOS host: a nonmatching iOS family
removes its `compile_error!` and executes fallback, while the matching macOS
family produces the retained diagnostic. Reuse it as evidence or fixture input.
Production generated-code compile-pass/fail tests belong in the admitted
frontend crate and run through the production gate; no root `make` target
executes spikes.

## Do not

Do not infer the consumer family from the proc-macro host. ADR 0049 rejects it, and the measurement behind that rejection is that `TARGET` and `CARGO_CFG_TARGET_*` were absent in the measured macro process.

Do not let a nonmatching target receive another family's bytes, and do not rely on a wrong-family payload failing loudly. `docs/research/apple-targets/numerical-behaviour.md` records that an `air64-apple-ios16.0` metallib loads and dispatches on the macOS host GPU without error, returning results; the load does not fail. That is why `docs/research/apple-targets/artifact-compatibility.md` requires runtime selection "by declared family and compatibility, never by trial-loading every metallib".

## Decision — Tom, 2026-07-25

**Decided: one envelope carrying N payloads, not N envelopes.** A selection naming several Apple families produces ONE artifact with one payload descriptor per family, each with its own compatibility contract. A consumer's `#[cfg]` selects a payload within an artifact it already holds.

**Why, in the terms that decided it:** one artifact identity per compilation means the cache key covers the whole selection and a partial delivery is impossible by construction. N envelopes would leave the selection itself with no identity — N artifacts and nothing binding them as one compilation, so 'these came from one program' becomes an external convention rather than a checked fact. That is the same class of gap that produced three separate identity defects this week, and it would additionally make a partial cache hit representable, which would then have to be made impossible or explicitly refused.

**It matches what already exists:** `push_carried_payload` takes a per-payload `compatibility: TargetProfileRef`, which exists precisely so payloads within one artifact can target different profiles.

**Accepted cost:** a consumer needing one family carries bytes for all of them. That is a delivery-time filtering concern, not an identity one, and may be addressed later without moving any artifact identity.

## Status re-verified against HEAD (2026-07-28)

**The Q-ART-008 retarget asked for at the end of "The work" has been performed, and this ticket is now its owner.** `docs/open-questions.md:198-207` names `generate-cfg-gated-artifact-family-delivery` on the Owner/track line and records why: the previous owner `prototype-artifact-family-delivery` closed `done` with the close condition unmet, which left the question owned by a terminal ticket and therefore unowned in fact. Reproduce with `sed -n '198,208p' docs/open-questions.md`. The close condition is unchanged — "named profiles expand to canonical `ArtifactFamilySelection` with generated `cfg` compile-pass/fail tests" — so the third bullet of "The work" is now this ticket's obligation outright rather than a retarget request, and it is not met.

**The implementation blockers were rechecked on 2026-07-30.** Neither `crates/tiler-macros/**` nor `crates/tiler-frontend-*/**`, the paths `ticketsplease.toml` maps for `implementation/frontend`, exists yet. `prototype-inline-proc-macro-frontend` is now `todo` after Tom approved the extensible region syntax, while `prototype-artifact-family-delivery` remains the other declared dependency. `status: blocked` is therefore still correct until those dependencies deliver, but no unresolved frontend decision remains.

**The 2026-07-25 decision's supporting fact still holds.** `push_carried_payload` (`crates/tiler-artifact/src/program/builder.rs:368-374`) still takes a per-payload `compatibility: TargetProfileRef`, so the one-envelope-N-payloads shape remains expressible in the artifact builder exactly as the decision claimed.

## Unparked 2026-07-31 — both declared dependencies delivered

**Fact.** `grep -m1 '^status:' tickets/prototype-inline-proc-macro-frontend.md tickets/prototype-artifact-family-delivery.md` reports `done` for both. The "Status re-verified against HEAD (2026-07-28)" section above conditioned `status: blocked` on exactly those two dependencies delivering, and they have; `status` is therefore corrected to `todo` from `prototype-inline-aot-integration-proof` at base `e6a47d9`.

**Fact — the implementation blocker recorded on 2026-07-30 is also stale.** That section states "Neither `crates/tiler-macros/**` nor `crates/tiler-frontend-*/**`, the paths `ticketsplease.toml` maps for `implementation/frontend`, exists yet." `crates/tiler-macros/` now exists with seven modules, and `crates/tiler/` is an accepted public boundary (Tom, 2026-07-31, `admit-the-tiler-facade-and-proc-macro-crate-boundary`). The owning crate this ticket needed no longer has to be created.

**Fact — one blocker recorded above does still stand, and it is a scope rather than a decision.** Reaching a selected family means `tiler-macros` gaining `tiler-build`/`tiler-cache`/`tiler-compiler` edges, and `Cargo.lock:419-424` currently records only `tiler-ir` and `tiler-metal-aot` for that package. Whoever dispatches this ticket must add `implementation/cargo-lock` to its scopes, and `implementation/workspace` if a new member is admitted.

**Fact — a second, independent gap now sits in front of this ticket's user-visible value.** `prototype-inline-aot-integration-proof`'s Outcome records the measurement: no region the approved grammar can express is admitted by `tiler_compiler`'s strategy selection, because both recognized normalizations require exactly one tensor input plus constant operations and the grammar has neither a scalar-literal nor a reduction production. Compiling a selected family is therefore implementable here, but nothing a consumer can currently write will exercise it until `admit-multi-input-elementwise-programs-at-the-compiler-boundary` lands or the grammar gains constant syntax (a `tensor!` public-boundary change, Tom's under ADR 0075).
