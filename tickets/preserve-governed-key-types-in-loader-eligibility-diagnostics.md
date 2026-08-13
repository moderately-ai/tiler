---
id: preserve-governed-key-types-in-loader-eligibility-diagnostics
title: Preserve governed key types in loader eligibility diagnostics
status: review
priority: p1
dependencies: [select-executable-variants-across-registered-backend-families]
related: [accept-the-loader-variant-eligibility-vocabulary]
scopes: [implementation/runtime, implementation/candle, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, public-boundary, correctness]
claimed_from: todo
assignee: worker-loader-eligibility-keys
lease_expires_at: 1786649004
---
## User-visible outcome

Loader eligibility refusals retain governed backend, representation, and target-profile key types rather than erasing them to strings, so callers can compare and route diagnostics without reparsing governed text or mixing key domains.

## Facts to re-verify

**Fact — routing decisions are already typed.** `ExecutionEnvironment` carries `BackendKey`, `RepresentationKey`, and `TargetProfileRef`; `variant_eligibility` compares those typed values before constructing a diagnostic.

**Fact — the diagnostic boundary erases them afterwards.** `UnsupportedRepresentation` clones four `.as_str()` values into `String`; `UndispatchableDType::host_profile` does the same; `TargetCompatibility` stores profile-key mismatches as `String`. The conversions cannot admit a route, but they make a public value documented as a governed key accept arbitrary text and lose compile-time distinction between key domains.

**Fact — no dependency or asymptotic-cost reason requires erasure.** The runtime already imports the owned artifact key newtypes. Cloning a typed key clones the same owned string allocation the current `.to_owned()` path makes.

## Required outcome

- Use `BackendKey` and `RepresentationKey` in `UnsupportedRepresentation`.
- Use `TargetProfileKey` throughout the directly coupled `TargetCompatibility` and `UndispatchableDType::host_profile` payloads.
- Update every construction, public pattern match, display implementation, test, conformance probe, and prototype consumer as one source-breaking pre-production sweep.
- Preserve exact displayed key text and every routing/refusal decision.
- Keep `FilteredVariant`'s public leaf-data fields as accepted; do not turn this repair into an accessor redesign.

## Stop conditions

Stop if a typed replacement introduces a dependency cycle, requires changing artifact identity/bytes, or reveals a genuine external compatibility commitment. None is known at filing.

## Required evidence

Perturb one backend, representation, and target-profile typed subject independently with assertions unchanged; show each diagnostic still reaches and names the intended field. Run runtime/package consumers, doctests, Clippy and rustdoc with warnings denied, citations, lint, exact-base guard, and the exact-tip publication gate required by the touched crate paths.

## Fact audit — 2026-08-13 at `761f6802`

All three ticket Facts and all five coordinator-verified Facts were **verified** at this base before the repair. None was false. After the repair they describe the pre-change tree.

**Ticket Fact — routing decisions are already typed.** Verified. `ExecutionEnvironment` fields are `TargetProfileRef`, `BackendKey`, and `RepresentationKey`. `variant_eligibility` compares `payload.backend != environment.backend` and `payload.representation != environment.representation` before constructing a diagnostic. `ExecutionEnvironment::classify` compares `declared.key != self.target_profile.key` as `TargetProfileKey`.

**Ticket Fact — the diagnostic boundary erases them afterwards.** Verified at this base. `UnsupportedRepresentation` stored four `String` fields constructed with `payload.backend.as_str().to_owned()` and the three siblings; `UndispatchableDType` stored `host_profile: environment.target_profile.key.as_str().to_owned()`; `TargetCompatibility::{ProfileKeyMismatch,DescriptorMismatch}` stored `declared`/`host`/`key` as `String`.

**Ticket Fact — no dependency or asymptotic-cost reason requires erasure.** Verified. `tiler-runtime` already depends on `tiler-artifact` alone. `load/host.rs` tests already imported `BackendKey`, `RepresentationKey`, and `TargetProfileKey`. Cloning a typed key clones the same owned string the `.to_owned()` path made.

**Coordinator Fact 1 — routing already compares typed keys.** Verified. Same `payload.backend != environment.backend` / `payload.representation != environment.representation` comparisons.

**Coordinator Fact 2 — the diagnostic then erases them.** Verified at this base via `declared_backend: String` and the four `.as_str().to_owned()` constructions.

**Coordinator Fact 3 — `UndispatchableDType::host_profile` is a `String`.** Verified at this base. Construction was `host_profile: environment.target_profile.key.as_str().to_owned()`.

**Coordinator Fact 4 — `TargetCompatibility` stores profile keys as `String`.** Verified at this base. `ProfileKeyMismatch { declared, host }` and `DescriptorMismatch { key }` were `String`.

**Coordinator Fact 5 — the runtime already imports the owned newtypes.** Verified. The `use tiler_artifact::program::{` block in `load/host.rs` tests imports `BackendKey`, `RepresentationKey`, `TargetProfileKey`.

## Scope additions

`implementation/candle` and `research/target-profiles` were added with `tkt set --add-scope` before those trees were edited. The Candle prototype constructs `TargetCompatibility::DescriptorMismatch` in a distinctness test; the scalar CPU vertical binds `host_backend`/`host_representation` and compares them to string literals. `implementation/conformance`, `implementation/frontend`, and `research/runtime` were not added: those consumers only match `{ .. }` and compiled without edits. `ForeignRouteRequirementOwner::{owner,host_backend}` remain `String`; they are not part of this eligibility-payload sweep.

## Worker report

**HEAD.** Landed on `tkt/preserve-governed-key-types-in-loader-eligibility-diagnostics` from base `761f6802414cb98b68999ef85c87610460ac844a`.

**What changed.** `UnsupportedRepresentation` now carries `BackendKey`/`RepresentationKey`. `TargetCompatibility::{ProfileKeyMismatch,DescriptorMismatch}` and `UndispatchableDType::host_profile` now carry `TargetProfileKey`. Construction clones the typed keys the loader already compared. There is no remaining `String` constructor on these payloads. `FilteredVariant` still has public `variant` and `reason` fields.

**Displayed key text.** `{declared_backend}`, `{declared_representation}`, `{host_backend}`, `{host_representation}`, and `{host_profile}` still render through `Display`, which writes `as_str()`, so the key text is unchanged. Routing and refusal decisions are unchanged. `AssessedProfile`/`PayloadProfile` still format `{classification:?}`; Debug of a `TargetProfileKey` now prints `TargetProfileKey("…")` rather than a bare string, which is the type's own Debug, not a change to the key text.

**Public-boundary status.** The changed variants are labelled drafts under ADR 0075. No new public type was introduced. No acceptance language was written. Tom's existing packet `accept-the-loader-variant-eligibility-vocabulary` already names this typed-key repair; no additional packet is required.

**Perturbations** (subject changed, assertions unchanged, then reverted):

1. Backend. `declared_backend` cloned `environment.backend` instead of `payload.backend`. `either_half_of_the_backend_representation_pair_filters_the_variant` failed at `assert_eq!(declared_backend.as_str(), fixture::BACKEND_KEY)`:

```
assertion `left == right` failed
  left: "tiler.test.other-backend"
 right: "tiler.test.scalar-host"
```

2. Representation. `declared_representation` cloned `environment.representation` instead of `payload.representation`. The same test failed at `assert_eq!(declared_representation.as_str(), fixture::REPRESENTATION_KEY)`:

```
assertion `left == right` failed
  left: "tiler.test.scalar-host-image-v2"
 right: "tiler.test.scalar-host-image-v1"
```

3. Target profile. `ProfileKeyMismatch.declared` cloned the host key instead of `declared.key`. `load::host::tests::a_different_family_is_a_key_mismatch` failed:

```
assertion `left == right` failed
  left: ProfileKeyMismatch { declared: TargetProfileKey("tiler.target.apple-m4"), host: TargetProfileKey("tiler.target.apple-m4") }
 right: ProfileKeyMismatch { declared: TargetProfileKey("tiler.target.apple-m1"), host: TargetProfileKey("tiler.target.apple-m4") }
```

**Commands.**

```
cargo nextest run -p tiler-runtime --offline
cargo test -p tiler-runtime --doc --offline
cargo nextest run -p tiler-prototype-run -p tiler-prototype-candle -p tiler-conformance --offline
cargo clippy -p tiler-runtime -p tiler-prototype-candle --all-targets --offline -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-runtime -p tiler-prototype-run -p tiler-prototype-candle -p tiler-conformance --no-deps --offline
cargo check --manifest-path spikes/target-profiles/scalar-cpu-vertical/Cargo.toml --offline
cargo check --manifest-path spikes/runtime/backend-provider-portfolio/Cargo.toml --offline
cargo check -p tiler --tests --offline
```

`tiler-runtime` nextest: 78 passed. Doctests: 9 passed. Prototype/conformance nextest: 144 passed, 1 skipped. Frontend check is clean. Clippy `-D warnings` on the edited packages is clean. rustdoc `-D warnings` on the touched workspace packages is clean. `cargo clippy -p tiler-prototype-run --all-targets -- -D warnings` reports three pre-existing lints in `prototypes/serial-sum-run/src/proof.rs` (`redundant_closure_for_method_calls`, two `err_expect`); that file is not in this diff.

**Measurement boundary.** Device-free host-side type and formatting change. No kernel work, no artifact bytes, no identity domain.

**Unsupported cases.** `ForeignRouteRequirementOwner` still stores backend keys as `String`. No external consumer compatibility commitment was found.

**Stop conditions.** No dependency cycle, no artifact identity/bytes change, no external compatibility commitment.
