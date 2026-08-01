---
id: generate-cfg-gated-artifact-family-delivery
title: Generate the cfg-gated delivery half of the artifact-family selection
status: todo
priority: p1
dependencies: [prototype-inline-proc-macro-frontend, prototype-artifact-family-delivery]
related: [prototype-macro-embedding-and-cargo-behavior, record-that-the-frontend-axis-is-review-gated]
scopes: [implementation/frontend]
shared_scopes: [contracts/navigation, project/tickets]
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

## Outcome

**What landed.** The delivery half is complete and inert: it emits what a compilation produced, and nothing compiles yet, so `FallbackOnly` still performs no backend compiler work and still expands token-for-token to the block it expanded to before.

- `crates/tiler-macros/src/family_cfg.rs` — the versioned family-to-consumer-`cfg` map. Version `tiler.frontend.family-consumer-cfg.v1`. Every one of the ten `ApplePlatform` families gets a predicate naming **both** governed keys, `all(target_os = "…", target_abi = "…")`, and the match is exhaustive with no wildcard arm, so widening the driver's vocabulary is a compile error here rather than a family with no predicate.
- `crates/tiler-macros/src/delivery.rs` — `NamedProfile` (four profiles expanding to a canonical `ArtifactFamilySelection`), `FamilyDelivery` (`Payload` or `Retained(String)`), `DeliveryPlan` (a verified product over one selection and its per-family outcomes), and `DeliveryPlan::items_source`, the pure emitter.
- `crates/tiler-macros/src/lib.rs` — `emit` places the plan's items in the region's own block. `expand` builds the plan through `delivery::stated_plan`, so the seam is exercised on every expansion.
- Fixtures: `crates/tiler/tests/facade/pass/family_cfg_matching_family_embeds_its_payload.rs`, `.../pass/family_cfg_nonmatching_targets_fall_back.rs`, and `.../fail/family_cfg_matching_family_retains_its_diagnostic.rs` with its `.stderr`. Each holds the emitter's byte-identical output, cross-checked by tests in the macro crate — the idiom `binding` already uses for `RegionFacts`.

**Fact — no new public item, and no dependency edge.** Everything added is `pub(crate)` inside `tiler-macros`, which exports only `tensor!`. `Cargo.lock` is unchanged: the delivery half is a pure function from an outcome to tokens, so it needs neither `tiler-build` nor `tiler-cache` nor `tiler-compiler`. The `implementation/cargo-lock` scope the "Unparked 2026-07-31" note anticipated was therefore not needed; the crate that first *invokes* the compiler is what will take those edges.

**The emitted shape, and why it is total.** One envelope, N payloads, per Tom's 2026-07-25 decision. The bytes are embedded once and unconditionally; only the payload *position* is gated:

```text
#[cfg(<retained family's predicate>)]
const _: () = { ::core::compile_error!("<the driver's own diagnostic>"); };
const __TILER_ARTIFACT: &[u8] = b"…";
#[cfg(<built family's predicate>)]
const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = ::core::option::Option::Some(0usize);
#[cfg(not(any(<every built family's predicate>)))]
const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = ::core::option::Option::None;
```

Every arm defines one name and the arms partition every target, which turns the two ways the map could be wrong into build errors in the consumer's own compilation: overlapping predicates define the name twice (E0428), and a gap leaves it undefined (E0425, watched failing). Neither can produce a silently wrong payload — the outcome `docs/research/apple-targets/artifact-compatibility.md` forbids and which nothing downstream would catch. A retained family's predicate is deliberately *absent* from `any(…)`, so its target gets one actionable `compile_error!` and a well-formed selector rather than that error plus an undefined name.

**Measurement — `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, macOS 27.0, 2026-07-31, `rustc --print cfg --target <triple>`.** `target_abi` is emitted for every target, including as the empty string, so `target_abi = ""` is a writable predicate. That is what makes the map fail-closed: iOS device, the iOS simulator, and Mac Catalyst all report `target_os = "ios"`, so an iOS-device predicate written on `target_os` alone would deliver a device payload to two families it was not built for. The four Apple rows agree exactly with `spikes/macro-environment/results/family-cfg-2026-07-24.json`'s own `target_predicates`.

**The five-target matrix.** `docs/correctness-and-testing.md`'s normative list, checked by evaluating the emitter's actual output against `rustc`'s own `cfg` answer (`the_emitted_arms_select_exactly_one_payload_per_consumer_target`, `a_retained_diagnostic_fires_only_on_the_family_it_names`). For a plan selecting macOS and iOS device, both built:

| consumer target | selector arm | retained diagnostic |
| --- | --- | --- |
| `aarch64-apple-darwin` | `Some(1)` — the macOS payload | fires when macOS is the retained family, and only then |
| `aarch64-apple-ios` | `Some(0)` — the iOS device payload | silent |
| `aarch64-apple-ios-sim` | `None` — semantic fallback | silent |
| `aarch64-apple-ios-macabi` | `None` — semantic fallback | silent |
| `x86_64-unknown-linux-gnu` | `None` — semantic fallback | silent |

Canonical family order puts `ios-device` before `macos`, so macOS selecting position 1 rather than 0 is the assertion that a consumer receives *its own* family's payload rather than the first one in the envelope. Ten further targets are covered in `each_family_predicate_matches_exactly_its_own_rust_target`, one per governed family plus a second non-Apple target.

**Fact — a stale premise in this ticket's own "Tests this ticket owes", corrected.** It says Catalyst remains "a deferred family that `ApplePlatform` cannot represent". `ApplePlatform::MacCatalyst` exists (`crates/tiler-metal-aot/src/input.rs:171`) and has its own predicate. The accurate reason no profile names it is the governed table: `target_language` admits Catalyst only at MSL 4.0, so `MetalTarget::new(MacCatalyst, _, Metal3_1)` returns `LanguageUnavailable` and a profile at MSL 3.1 cannot name it. The ticket's *conclusion* is unaffected and is what the tests assert — a Catalyst consumer matches no selected family and takes the fallback, never relabelled bytes.

**Cross-target compile evidence is out of reach on this host, and the boundary is exact.** `rustup target list --installed` reports `aarch64-apple-darwin` alone, so `trybuild` compiles only for the host; installing a rustup target is a host-toolchain change AGENTS.md reserves to Tom. Host-target compilation therefore proves the *matching* (macOS) and *nonmatching* (iOS device, simulator, Catalyst, non-Apple) shapes, and `rustc --print cfg` — which needs no installed standard library — decides the predicates for the other four targets. Perturbation B below is what makes the Catalyst claim concrete rather than inferred.

**Q-ART-008 — the stated close condition is met; the question stays open on one residue.** `docs/open-questions.md` is updated and retargeted to the new ticket [`accept-the-inline-artifact-family-profile-syntax`](accept-the-inline-artifact-family-profile-syntax.md). Four profiles (`fallback-only`, `macos`, `ios`, `macos-and-ios`) expand to canonical selections with the generated `cfg` compile-pass/fail tests the condition names. What is not met is the *ergonomic* half: the approved region grammar has no production for a profile name, so nothing constructs one during an expansion. Inventing that syntax is a consumer-visible boundary ADR 0075 reserves to Tom, which is the boundary packet that new ticket carries.

**Every new check was watched failing** (perturbation, revert, re-verify):

| perturbation | what failed |
| --- | --- |
| Mac Catalyst remapped to the iOS-device predicate | `no_two_families_share_a_consumer_predicate`, `the_three_ios_families_are_kept_apart`, `the_versioned_map_is_pinned_row_by_row`, `each_family_predicate_matches_exactly_its_own_rust_target` |
| `MAP_VERSION` bumped with no row change | `the_versioned_map_is_pinned_row_by_row` |
| selector catch-all arm removed from the emitter | `one_built_family_emits_its_gated_selector_and_a_total_catch_all`, `a_mixed_plan_gates_…`, `the_emitted_arms_select_exactly_one_payload_per_consumer_target`, `a_retained_diagnostic_fires_only_on_the_family_it_names` |
| `"` no longer escaped in the byte-string literal | `every_byte_renders_as_a_literal_rust_accepts`, `the_matching_fixture_compiles_what_this_emitter_produces` |
| `stated_policy` returns a profile instead of `FallbackOnly` | `the_current_expansion_states_a_deliverable_fallback_only_policy`, `the_production_expansion_plans_no_delivery_items` |
| A: catch-all arm deleted from the pass fixture | `trybuild` — `error[E0425]: cannot find value __TILER_SELECTED_PAYLOAD in this scope` |
| B: fail fixture regated from macOS onto Mac Catalyst | `trybuild` — the compile-fail case compiled, which is the direct evidence that a Catalyst-gated diagnostic does not fire on a macOS consumer |

The two `should_panic` tests on the predicate evaluator are the same discipline applied to the test harness: a parser nobody had watched refuse would let a widened predicate be evaluated by a model that no longer describes it.

**The spike was re-run first, per its own caveat.** `spikes/macro-environment/run-family-cfg.sh` at revision `2a1f57b`, macOS 27.0, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`: `success: true`, `matching_family_diagnostic: true`, `required_compile_status: 1`. Its recorded host cfg (`target_os="macos"`, `target_abi=""`) and its four `target_predicates` rows agree with the map. Nothing under `spikes/` was modified.

**Fact — one contract sentence is now imprecise, and correcting it needs a scope this ticket does not hold.** `docs/integration/frontends.md` says "For each selected family, successful expansion embeds its payload under the family's governed consumer-target `#[cfg]`." Tom's 2026-07-25 decision on this ticket supersedes that shape: one envelope carries N payloads, embedded once and unconditionally, and the `#[cfg]` selects a payload *within* an artifact the consumer already holds — the accepted cost being that "a consumer needing one family carries bytes for all of them". The implemented behaviour follows the decision. Correcting the sentence needs `contracts/integrations`, which is not in this ticket's scopes, so it is reported rather than absorbed.

**Deliberately not done.**

- No consumer-visible syntax was invented; see the boundary ticket.
- No backend compilation. `stated_delivery` still refuses `SelectedFamilies` with `BackendCompilationUnavailable`, which is the fail-closed guard while nothing can build a family; `prototype-inline-aot-integration-proof` is what removes it and supplies real outcomes.
- No runtime dispatch. `__TILER_SELECTED_PAYLOAD` is what a runtime entry will consume; until then no expansion emits it, because the production policy is `FallbackOnly`, so no consumer sees an unused constant.
- `MAP_VERSION` does not yet reach an identity subject, because the frontend computes no artifact identity. Recorded in the constant's own `#[allow]` reason rather than left implicit.
- `crates/tiler-macros/src/cache_root.rs`'s `#[allow]` reason named this ticket as the slice that consumes the cache-root resolver. It is corrected to name `prototype-inline-aot-integration-proof`: this ticket emits what a compilation produced and does not itself compile, so it opens no cache.
