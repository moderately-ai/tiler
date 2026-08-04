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
