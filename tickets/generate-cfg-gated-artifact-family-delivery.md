---
id: generate-cfg-gated-artifact-family-delivery
title: Generate the cfg-gated delivery half of the artifact-family selection
status: blocked
priority: p1
dependencies: [prototype-inline-proc-macro-frontend, prototype-artifact-family-delivery]
related: [prototype-macro-embedding-and-cargo-behavior, record-that-the-frontend-axis-is-review-gated]
scopes: [implementation/frontend]
shared_scopes: []
paths: []
tags: [implementation, apple-targets, inline-dx, frontend]
---
`prototype-artifact-family-delivery` landed the driver-side half of ADR 0049 and ADR 0053 — the canonical typed `ArtifactFamilySelection`, its `SelectedFamilies`/`FallbackOnly` delivery policy, and the fan-out to one `MetalTarget` per selected family — as a crate-private draft in `crates/tiler-metal-aot/src/family.rs`. This ticket owns the half that could not land with it, and the reason is a boundary rather than an omission.

**Fact — the remaining half is generated Rust, and an accepted packaging profile puts it elsewhere.** ADR 0053 states: "Generated Rust gates the payload or diagnostic by the family's versioned consumer-target `#[cfg]` predicate. A matching target requires the selected artifact and sees `compile_error!` on build failure; a nonmatching target uses the semantic fallback." ADR 0077 item 1 states that `tiler-metal-aot` "does not emit MSL, does not assemble the target-neutral artifact bundle, and does not implement the expansion cache or the proc-macro layer", and `docs/architecture.md`'s crate table assigns "emit artifact plus runtime/fallback tokens" to the frontend proc-macro crate. A family's consumer-target `#[cfg]` predicate is a fact about a *Rust* target; the driver knows only about `xcrun`. Landing versioned generated-code data in the driver would have given it a second responsibility an accepted profile places on a crate that does not exist.

**Fact — the owning crate cannot be created by the parent ticket.** `ticketsplease.toml` maps `implementation/frontend` to `crates/tiler-macros/**` and `crates/tiler-frontend-*/**`; neither exists. Admitting a workspace member requires the root `Cargo.toml` (`implementation/workspace`), `Cargo.lock` (`implementation/cargo-lock`), and `scripts/check_workspace.py`'s pinned member, description, and dependency tables. `prototype-artifact-family-delivery` declares none of those scopes. `prototype-apple-aot-driver`, which did admit a crate, declared `implementation/workspace` and carried an explicit clause authorizing it; the parent has neither.

**Fact — the axis is gated on a review, not on engineering.** `record-that-the-frontend-axis-is-review-gated` records that `prototype-inline-proc-macro-frontend` depends on `prototype-public-compiler-api`, whose closing condition is Tom's acceptance of a public boundary, and closes with: "Do not close this by starting frontend work that routes around the unreviewed boundary — that would answer the review question by omission." This ticket is `blocked` for that reason rather than `todo`.

## The work

With a frontend proc-macro crate admitted, implement the delivery half over the parent's `ArtifactFamilySelection`:

- The versioned family-to-consumer-`cfg` predicate map, as versioned Tiler data. The measured distinctions are recorded in `docs/research/macro-environment/proc-macro-build-environment.md`: macOS is `target_os = "macos"`, iOS device is `target_os = "ios"` with an empty `target_abi`, the iOS simulator is `target_abi = "sim"`, and Mac Catalyst is `target_abi = "macabi"`. `docs/integration/frontends.md` calls the map "versioned Tiler data and covered by generated-code tests", so a widened map that does not bump its version is a defect.
- Emission of the gated tokens: for each selected family, either its embedded payload or its retained toolchain/compiler diagnostic as a `#[cfg]`-gated `compile_error!`, plus the semantic fallback for every nonmatching target. Target-neutral semantic, optimizer, verifier, and envelope failures stay unconditional compile errors.
- The named ergonomic profiles that `docs/open-questions.md` Q-ART-008 tracks, expanding to a canonical `ArtifactFamilySelection`. Q-ART-008 names `prototype-artifact-family-delivery` as its owner and its close condition is "named profiles expand to canonical `ArtifactFamilySelection` with generated `cfg` compile-pass/fail tests". Retarget the open question to this ticket, or record why it stays with the parent.

## Tests this ticket owes

`docs/correctness-and-testing.md` states them normatively: "Generated consumer-`cfg` tests cover macOS, iOS device, iOS simulator, Catalyst, and an unrelated non-Apple target. A selected matching family embeds its payload or emits its retained actionable compile error; a nonmatching target compiles the semantic fallback; `FallbackOnly` performs no backend compiler work."

Catalyst is in that list while remaining a deferred family that `ApplePlatform` cannot represent. Its case is therefore that a Catalyst consumer target matches *no* selected family and takes the fallback — never that an iOS-device or macOS payload is relabelled as Catalyst-compatible, which `docs/backends/metal.md` forbids explicitly.

The checked-in probe `spikes/macro-environment/run-family-cfg.sh` already demonstrates the behaviour on the measured macOS host: a nonmatching iOS family removes its `compile_error!` and executes fallback, while the matching macOS family produces the retained diagnostic. Reuse it as the evidence rather than re-deriving it. These are compile-pass/fail cases, and `AGENTS.md` compiles a spike workspace in the gate exactly when it retains a `trybuild` golden, so decide that posture deliberately rather than inheriting it.

## Do not

Do not infer the consumer family from the proc-macro host. ADR 0049 rejects it, and the measurement behind that rejection is that `TARGET` and `CARGO_CFG_TARGET_*` were absent in the measured macro process.

Do not let a nonmatching target receive another family's bytes, and do not rely on a wrong-family payload failing loudly. `docs/research/apple-targets/numerical-behaviour.md` records that an `air64-apple-ios16.0` metallib loads and dispatches on the macOS host GPU without error, returning results; the load does not fail. That is why `docs/research/apple-targets/artifact-compatibility.md` requires runtime selection "by declared family and compatibility, never by trial-loading every metallib".
