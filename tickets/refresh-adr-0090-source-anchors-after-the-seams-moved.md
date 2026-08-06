---
id: refresh-adr-0090-source-anchors-after-the-seams-moved
title: Refresh ADR 0090's source anchors after the seams moved
status: review
priority: p3
dependencies: []
related: [audit-backend-authoring-against-all-thirteen-responsibilities, specify-the-consumer-neutral-backend-provider-composition-contract]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, backend-providers]
claimed_from: todo
assignee: agent-adr-fixes
lease_expires_at: 1786050483
---
## User-visible outcome

ADR 0090's reproductions resolve against the tree a reader runs them on, so a reader checking the record's Facts gets the evidence rather than an unrelated line or an empty result.

## Why this exists

Carrier ticket. The [backend-authoring audit](audit-backend-authoring-against-all-thirteen-responsibilities.md) re-ran ADR 0090's own reproductions at base `51e9374a` and found three anchors that no longer resolve. The audit's scopes reach `docs/research/extensions/` but not `docs/decisions/`, so the research record carries the corrections and the ADR does not. Each item below was verified by reading the cited file, not by search alone.

**The row 4 and row 5 reproduction returns nothing, and the sentence it supports is half stale.** ADR 0090:33 says to reproduce with `grep -n "dyn PhysicalImplementationProvider; 1\|OpaqueCallRegistry::new()" crates/tiler-compiler/src/pipeline/planning.rs`. That command now exits 1 with no output. The hardcoded one-element provider array and the inline empty registry were replaced by `PhysicalAuthorities` (`crates/tiler-compiler/src/frontier.rs:2893`), constructed as `PhysicalAuthorities::governed()` at `crates/tiler-compiler/src/pipeline.rs:604` and consumed at `pipeline/planning.rs:292`. **Row 4's claim survives** — nothing still installs a physical implementation from outside the crate, and `PhysicalImplementationProvider` is `pub(crate)` at `frontier.rs:1234` behind a private `mod frontier` at `lib.rs:24`. **Row 5's claim does not**: `PhysicalAuthorities::composed` (`frontier.rs:2920`) is driven from the compile path at `crates/tiler-compiler/src/pipeline/tests.rs:3449`, and [`register-opaque-calls-on-the-compile-path`](register-opaque-calls-on-the-compile-path.md) is `done`. Production still installs an empty registry, which the type's own documentation records as the ordinary case rather than a gap.

**Item 9's loader citation moved.** ADR 0090:97 cites `crates/tiler-runtime/src/load.rs:309-310` for `refuse_route_requirements` followed by `refuse_deferred`. Those calls are at `load.rs:410-411` at this base, in the same order and with the same comment giving the same reason; `load.rs:309` is now `DecodedProgram::payloads`. The finding is unchanged and only the anchor moved.

**Item 5's population site moved.** ADR 0090:77 cites `crates/tiler-compiler/src/session.rs:1513` for `Compilation::offered_providers` being populated from the lowering registry alone. That population is at `session.rs:2092-2093` at this base. The claim is unchanged and still true.

## Implementation keys

- Re-verify each anchor by reading the file at the base the edit lands on rather than trusting this ticket's line numbers, which are facts about `51e9374a`.
- Correct row 5's sentence rather than only its anchor: "nothing registers one on the compile path" is the part that is now false, and an anchor refresh that left it standing would preserve the wrong claim.
- Prefer an anchor that survives movement where one exists — an item name plus a reproducing command beats a bare line number, which is the pattern the record already uses for its greps.
- Do not restate the audit's maturity table here; the research record owns it and duplicating it would create a second authority over one subject.

## Closes when

Every reproduction in ADR 0090 runs and returns what the sentence beside it claims; row 5's stale sentence is corrected; and no anchor this ticket names still points at an unrelated line.

## Outcome — 2026-08-06

Delivered as `0f2bb9c5`, the second of two commits on `tkt/correct-the-ocp-source-status-in-adrs-0036-and-0038`, over base `76fe3a8e`. Fifteen corrections across thirteen lines of one file. **Every line number in this ticket's body is a fact about `51e9374a` and none of them survived**: each anchor was re-derived by reading the cited file at base `76fe3a8e`, and each landed at a different line than the ticket predicted. The three the ticket names are corrected; reading the whole record for siblings of the same defect found **seven more anchors and three more stale claims**, all inside `contracts/decisions`, all corrected in the same commit on the "find one, check all siblings" ground.

**Fact — the three named anchors, re-derived.**

| Ticket's claim at `51e9374a` | Re-derived at `76fe3a8e` |
| --- | --- |
| `PhysicalAuthorities` at `frontier.rs:2893`, `composed` at `2920`, provider trait `pub(crate)` at `1234` | `frontier.rs:2934`, `2961`, `1259` |
| `PhysicalAuthorities::governed()` at `pipeline.rs:604`, consumed at `planning.rs:292` | `pipeline.rs:615`, consumed at `planning.rs:307` |
| `composed` driven from the compile path at `pipeline/tests.rs:3449` | `pipeline/tests.rs:5370`, inside `opaque_call_authorities`, driven by the test at `5454` |
| `refuse_route_requirements` then `refuse_deferred` at `load.rs:410-411` | `load.rs:411-412`, inside `DecodedProgram::preflight` |
| `offered_providers` populated at `session.rs:2092-2093` | `session.rs:2158-2159`, `Arc::from(capabilities.0.lowering().providers())` inside `compile` |

**Row 5's sentence was corrected in both places it appears, not only at the reproduction.** ADR 0090:33 read "nothing registers an opaque call on the compile path (row 5)" and ADR 0090:125 read "this record adds only that no caller of any kind registers one on the compile path, which [`register-opaque-calls-on-the-compile-path`] owns as an internal wiring gap". That ticket is `status: done`, and the wiring exists: `PhysicalAuthorities` (`frontier.rs:2934`) composes the provider list with the opaque-call registry, `compile` installs `PhysicalAuthorities::governed()` (`pipeline.rs:615`), and `planning.rs:307` passes both halves into `enumerate_frontier`. Line 33 now says nothing outside the crate can register one *though the compile path itself carries the registry*, and line 125 says the ticket closed it by composing the two into the one authority `compile` installs, with production registering no call because Tiler declares none of its own. **Row 4's claim survives untouched** — the trait is `pub(crate)` at `frontier.rs:1259` behind a private `mod frontier` at `lib.rs:24` — and only its reproduction was replaced.

**The reproductions written into the record, each run at `76fe3a8e` and each returning what its sentence claims.**

- Row 4: `grep -rn "^mod frontier;\|pub(crate) trait PhysicalImplementationProvider" crates/tiler-compiler/src/` → two lines, `lib.rs:24` and `frontier.rs:1259`. Replaces a command that exited 1 with no output.
- Row 5: `grep -rn "PhysicalAuthorities::governed()\|PhysicalAuthorities::composed(" crates/tiler-compiler/src/` → five lines: `pipeline.rs:615`, `pipeline/tests.rs:4586`, `4611`, `4992`, and `pipeline/tests.rs:5370`. The record states exactly that shape — one production installation, three compile-path test drives of the governed pair, one composition.
- Row 8, unchanged and re-run: `grep -rniE "(backend|emitter)[_ ]?(registry|register|factory|plugin|dispatcher|selector)" --include='*.rs' crates/` → nothing, as the sentence claims. Its positive control with `lowering` returns 77 lines here against the 73 the record records at `2a1f57b`, which is a dated measurement and was left as recorded.
- Item 10, unchanged and re-run: `grep -rn "is_ascii\|InvalidByte" crates/tiler-artifact/src/` → one line (`keys.rs:122`); the identical grep over `crates/tiler-compiler/src/target.rs` → six lines, which is what "still returns six lines" claims.

**The seven sibling anchors, each read before it was moved.** `load.rs:586-595` for item 4's `RouteRequirement::BackendFeature` owner refusal — that span is now `select_route`'s guard-evaluation doc comment, and the refusal is `route_requirements` at `load.rs:888-897`, reached from `prepare` at `load.rs:448`, which the corrected sentence now names. `feasibility.rs:195` for `CapabilityAxis` being `pub(crate)` → `211`; the enum still has exactly seven variants, so item 1's count is unchanged. `physical.rs:1016` for `verify_schedule_with_feasibility` → `2145`; its doc comment still names the same five gates in the same order. `keys.rs:124-143` for `validate_key` → `125-143`. `metal_plan.rs:266` for the private `assemble_artifact` → the promoted `assemble_plan_artifact` at `crates/tiler-build/src/plan_artifact.rs:179`. `metal_assembly.rs:27-28` for the two hardcoded literals → `28-29`. `metal_plan.rs:302-304` and `306-309` for item 11's residue → `metal_plan.rs:403-405`, inside `metal_entry_declaration`.

**Three sibling claims were stale in substance, not only in anchor.**

1. Item 11's residue read "genuinely not yet neutral". The promotion landed on 2026-08-01, and `metal_plan.rs`'s own doc comment at the new site says the three values "are now this backend's answer rather than the orchestrator's assumption". The sentence now says the residue *was* not neutral and names where the backend states it. The proposal's referent `assemble_artifact` was renamed at promotion, so it is named `assemble_plan_artifact` — a reader grepping the old name found nothing.
2. The Context Fact about row 8 read "the useful half of row 8 already exists **in private**". `assemble_plan_artifact` is `pub`, and its `declare_payload` closure now returns `Vec<PayloadId>` rather than one `PayloadId` — the `v13` delivery-position change the record's own status paragraph records. Both corrected; the derivation list beside it was re-read against the function body and is unchanged.
3. Item 4's `ExecutionEnvironment` enumeration listed three fields. `crates/tiler-runtime/src/load/host.rs:77-91` carries four: the fourth is `dtype_dispatch`, a map from arithmetic type to dispatch verdict. Added, because "the only things a loading host states about itself" is a closed claim and a closed claim with a missing member is false. The load-bearing half — that nothing there names the producer — was re-read and holds.

One consequence bullet was also wrong on a count: `crates/tiler-compiler/Cargo.toml` depends on `tiler-ir` and **three** numeric crates (`num-bigint`, `num-integer`, `num-traits`), not two. The claim it supports, that the compiler does not know Metal exists, holds.

**The record's own provenance line was updated to match.** ADR 0090:21 claimed every Fact was "re-checked at `2a1f57b`". Every Fact-bearing anchor and reproduction in the record has now been re-derived at `76fe3a8e` by reading the cited file, and the line says so. Two anchors are deliberately left as recorded because they are dated measurements about a named past commit rather than present-tense claims: `keys.rs:73-85` "when this was written", and `keys.rs:121` "at `7ad2aca`".

The audit's maturity table is not restated here or in the ADR; `docs/research/extensions/backend-provider-composition.md` remains its only authority, and this commit touches no research record.

**Checks, docs-and-tickets only.** No `crates/`, `prototypes/`, or build-configuration path is touched, so no Cargo gate applies and no reproduction in this ticket compiles anything. `tkt lint` → `ok: no problems found`. `git diff --check` → no output.
