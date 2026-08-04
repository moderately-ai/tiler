---
id: retain-and-attribute-a-real-msl-failure-through-an-expansion
title: Retain a real MSL front-end failure through an expansion and attribute it to region source
status: in-progress
priority: p3
dependencies: []
related: [prototype-inline-aot-integration-proof, generate-cfg-gated-artifact-family-delivery]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, diagnostics, macro-aot]
claimed_from: todo
assignee: agent-msl-retain
lease_expires_at: 1785874660
---
## User-visible outcome

When `xcrun metal` rejects the MSL an expansion emitted, the consumer sees the compiler's own text attributed to the region construct that produced it, rather than to the invocation as a whole.

## Why this exists

**Fact — the retention machinery exists and is exercised, but only by the wrong failure.** `DriverError::ToolFailure` carries the failing tool's stderr as bounded bytes (`MAX_RETAINED_OUTPUT_BYTES`, 16 KiB, truncation recorded), `tiler_macros::aot::retained` renders it into the family-scoped `#[cfg]`-gated `compile_error!`, and `crates/tiler/tests/facade/fail/family_cfg_matching_family_retains_its_diagnostic.rs` pins the result byte for byte. The failure that fixture induces is `ToolchainUnavailable` — `aot.rs` says so in the doc comment on `deliver`'s `toolchain` parameter: "pointing it at a path that is not there reaches the same `DriverError::ToolchainUnavailable` a host with no Apple tools produces, which is how the retained-diagnostic path below is exercised on a machine that does have them." No test drives `CompileStage::Metal` to a nonzero exit through an expansion.

**Fact — the span is the invocation's, not the construct's.** `docs/integration/frontends.md`'s remaining-checks list asked for "source-spanned retained MSL diagnostics". What is delivered is a diagnostic at the invocation span carrying MSL text verbatim; nothing maps an MSL line and column back to the `out` sub-expression, operand, or `deliver` token that caused it.

**Fact — the cache half is permission, not delivery.** The same contract says "debug configuration may retain canonical MSL and tool diagnostics under the cache entry." `crates/tiler-cache/src/expansion.rs` mentions neither MSL nor source; no entry carries either.

**Inference — reachability is the hard part and belongs in the ticket, not in the fix.** A region that reaches the driver has already passed Tiler's own verifier and emitter, so a genuine `metal` rejection means a defect in the emitter. Deciding whether such a failure is reachable at all — and if it is only reachable by injection, saying so — is the first deliverable, because a diagnostic path nothing can reach is a different obligation from one a consumer can hit.

## Implementation keys

- Establish reachability before designing attribution. If the only route is an injected malformed emission, the honest outcome may be a narrow injection-only test plus a contract sentence saying the case is a frontend defect rather than consumer-facing input, and *that* is a legitimate close.
- Attribution needs a correspondence the emitter can carry. Do not infer a span by matching MSL text against region tokens; that is a second authority over what produced which line.
- The cache-retention permission is separable and may split into its own ticket; it changes what an entry stores, which is cache-identity-adjacent and needs its own reasoning about whether retained diagnostics participate in validation.

## Closes when

A real `metal` front-end rejection is reached through an expansion and its retained text is observed in the emitted `compile_error!`; the attribution question is either delivered or explicitly answered as unreachable-by-construction with the reason recorded; the cache-retention permission is implemented or split out; and `docs/integration/frontends.md`'s remaining-checks list is updated with the outcome.

## Outcome — 2026-08-04

**Fact — reachability has two routes, and the ticket's inference was half right.** The derivation is recorded in full in `crates/tiler-macros/src/aot.rs` under "Reaching the `metal` stage's own refusal". A `metal` rejection *of the emitted source* is unreachable from any invocation: `tiler_metal`'s emitter names entry points, NaN helpers, and staging allocations from identity digests, names buffers `b<argument-table ordinal>`, and emits scalar constants as hexadecimal bit patterns, so no `InputKey`, `OutputKey`, or region token reaches the translation unit as a token — that class is a defect in Tiler's emitter, and reaching it needs injection. But a *second* route exists that the ticket did not anticipate and that is not a frontend defect: nothing between `deliver` and `Toolchain::run_stage` compares the language standard the bound declaration requests against the `metal` that was resolved, so a build host whose Apple toolchain predates MSL 4.0 resolves, runs, and is refused its own `-std=metal4.0`. That is consumer-facing, its remedy is the host, and the family-scoped `#[cfg]` gate is the right delivery for it.

**Fact — both routes are exercised against the host's real `metal` binary.** `tiler_macros::aot::tests::a_real_metal_front_end_rejection_is_retained_under_its_family` drives `deliver` end to end for each, through a launcher whose `--find metal` answers with a wrapper around the real compiler; the emitted items are asserted to be a `#[cfg]`-gated `compile_error!` carrying `offline metal failed`, `exit code 1`, and the compiler's own words. It carries an unshimmed control so the shim is demonstrably what caused the rejection. `a_retained_msl_diagnostic_carries_the_emitted_source_position` holds that a real MSL diagnostic reaches the consumer with its own path, line, column, and quoted source.

**Fact — the consumer-visible end is pinned byte for byte.** `crates/tiler/tests/facade/fail/family_cfg_matching_family_retains_a_metal_front_end_diagnostic.rs` and its golden carry a verbatim capture of a real `metal` refusal produced by the test above (macOS 27.0, Apple M4 Max, Metal Toolchain 27A5228f, 2026-08-04); the two absolute paths, the line, and the column are that run's and are recorded as such rather than presented as reproducible. `the_metal_front_end_fixture_compiles_what_this_emitter_produces` ties the fixture to the emitter and additionally counts the emitted lines, because a four-line diagnostic emitting as more than one item would mean a raw newline had closed the `compile_error!` literal — the property the pre-existing one-line retained text could not distinguish.

**Fact — attribution is answered, not delivered, and the reason is that it would name the wrong thing.** No correspondence from an MSL position back to a region construct exists at two independent points: `tiler_ir`'s semantic program carries no frontend spans and must not, and the emitter attaches no per-statement provenance. Building one is a public boundary across three crates, and in both reachable routes it would point at a construct that is not at fault — the source-rejection route is a Tiler defect whose reader is a Tiler developer, and the host route is about the machine. Deferred with triggers as `carry-a-source-correspondence-from-region-text-to-emitted-msl`.

**Fact — the cache-retention permission is split out, not absorbed.** `retain-canonical-msl-under-a-debug-expansion-cache-entry` owns it. Two reasons, both structural: it changes what a bundle stores, which needs its own decision about key participation, digest participation, and absent-section behaviour on a hit; and every file it touches is under `crates/tiler-cache/**` and `crates/tiler-build/**`, which this ticket's `implementation/frontend` scope does not reach.

**Boundary — `docs/integration/frontends.md` was not edited here.** A live docs worker held `contracts/integrations` during this wave, so the remaining-checks replacement text was handed to the integrator rather than written from this branch.
