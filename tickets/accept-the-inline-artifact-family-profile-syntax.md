---
id: accept-the-inline-artifact-family-profile-syntax
title: Decide how a consumer states an artifact-family delivery profile
status: done
priority: p1
dependencies: [generate-cfg-gated-artifact-family-delivery]
related: [prototype-inline-aot-integration-proof, prototype-inline-proc-macro-frontend]
scopes: [implementation/frontend]
shared_scopes: [contracts/navigation, contracts/integrations, project/tickets]
paths: []
tags: [decision, public-boundary, inline-dx, frontend, apple-targets]
---
## Why this is a ticket and not a line of code

`generate-cfg-gated-artifact-family-delivery` landed everything an artifact-family delivery profile needs *except* the way a consumer asks for one. `crates/tiler-macros/src/delivery.rs` holds four named profiles — `fallback-only`, `macos`, `ios`, `macos-and-ios` — each expanding to a canonical `ArtifactFamilySelection` through `ArtifactFamilySelection::new`, with the governed floors the driver's own table fixes. `crates/tiler-macros/src/family_cfg.rs` holds the versioned consumer-`cfg` map, and `DeliveryPlan::items_source` emits the gated payload selector, the retained diagnostic, and the total fallback arm. All of it is tested and none of it is reachable, because `delivery::stated_policy` returns `FallbackOnly` unconditionally and the approved region grammar admits no way to say otherwise.

**Fact.** The approved grammar (Tom, 2026-07-30, `prototype-inline-proc-macro-frontend`) is a declaration block of `sym` and `in` statements followed by one `out` expression. It has no production for a profile name, an artifact family, a deployment minimum, or a language standard.

**Why a worker must not simply add one.** ADR 0075 reserves consumer-visible surface to Tom, and this surface is consequential in both directions: the spelling is what every consumer writes and cannot be changed silently afterwards, and the vocabulary it publishes — Apple family names, or an opaque profile name that hides them — decides how much Apple backend policy appears on a consumer-neutral frontend boundary. `docs/integration/frontends.md` already reserves a second axis on the same surface ("a frontend may expose a separate explicit 'acceleration required' policy"), so a syntax chosen for profiles alone has to leave room for it.

## The decision

Which of these a consumer writes, and where in the region it goes:

1. **A named profile in the declaration block**, e.g. `deliver macos-and-ios;` beside `sym` and `in`. Cheapest to read and to parse, and it publishes no Apple version vocabulary. It also fixes each family's deployment minimum and language standard at whatever the profile says, so a consumer needing a different floor has no way to state one and must wait for option 2 or 3.
2. **An explicit family list**, e.g. `deliver macos 14.0, ios 17.0;`. Maximally explicit and matches `ArtifactFamilySelection` one-to-one, at the cost of publishing Apple family, deployment-minimum, and MSL vocabulary on the consumer-facing surface — and of making every consumer restate floors the driver already governs.
3. **Both**, with a profile name as the ergonomic default and a family list as the escape hatch. `docs/integration/frontends.md` permits exactly this shape — "A frontend may offer an ergonomic literal default profile, but the resolved selection is still explicit compiler input" — at the cost of two productions to keep consistent.
4. **An attribute rather than a statement**, e.g. `#[tiler::deliver(macos)]` above the invocation. Rejected here rather than offered: a `#[proc_macro]` cannot see attributes outside its own token stream, so this would require a second macro form and would break the accepted "each invocation is a self-contained AOT and embedding unit".

The profile *names* are equally Tom's, and are today a crate-internal draft rather than an accepted vocabulary.

## Decision — Tom, 2026-07-31

**Accepted: option 3, with the draft profile names.** A consumer states a delivery profile as a named profile in the declaration block (`deliver macos-and-ios;`) with an explicit family list (`deliver macos 14.0, ios 17.0;`) as the escape hatch; the profile-name vocabulary is `fallback-only`, `macos`, `ios`, `macos-and-ios`. The grounds: it is the shape `docs/integration/frontends.md` already reserves — an ergonomic literal default whose resolved selection is still explicit compiler input — and it keeps the floor-override case expressible without publishing Apple vocabulary on the mandatory path. Option 4 stays eliminated on the proc-macro token-visibility constraint.

## Closes when

Tom accepts a syntax and a profile-name vocabulary; the grammar admits it; `delivery::stated_policy` becomes a function of the parsed region rather than a constant; the `#[allow(dead_code)]` reasons on `NamedProfile` and `FamilyDelivery` come off; and `docs/integration/frontends.md` states the accepted spelling. Q-ART-008 closes with it.

## Graph maintenance

Q-ART-008 in `docs/open-questions.md` names this ticket as the owner of its remaining half. Record the accepted spelling in the frontend contract and, if the choice is consequential enough to outlive a ticket, in an ADR.

## Outcome

**The accepted syntax is implemented, and one item of the closes-when could not be done for a stated reason.**

### The grammar as implemented

`crates/tiler-macros/src/grammar.rs` admits `deliver` as a fourth declaration-block statement beside `sym` and `in`, at most once, in two productions:

```text
deliver <name>;                                        // a named profile
deliver <name> <major>.<minor>[, <name> <major>.<minor>]*;  // a family list
```

The productions are told apart by the token *after* the first name and by nothing else: `;` ends a profile and a literal opens a family list. That is decidable with one token of lookahead and without knowing which names exist, which is what keeps the vocabulary in `delivery.rs` — widening the profile list changes no parsing rule. A name may carry hyphens, because Rust's lexer admits none inside an identifier and `macos-and-ios` therefore arrives as five tokens; the joined name carries the span of its *first* identifier, since joining spans needs the unstable `Span::join` and this crate holds only accepted stable proc-macro contracts. A deployment minimum is read from the literal's **source text**, not from a parsed float: `14.10` and `14.1` are different minimums and the same `f64`. A trailing comma closes no `deliver` list, matching `sym` and `in` rather than an axis list.

Absence is behaviour-identical. `stated_policy(None)` is `FallbackOnly`, and `the_production_expansion_plans_no_delivery_items` covers both spellings of it — writing nothing and writing `deliver fallback-only;` — against the same emptiness assertion. The compile-pass fixture asserts the same thing end to end, by evaluating a stated and an unstated region in one file and comparing the values.

### What `deliver macos;` produces today

A spanned `compile_error!` on the `deliver` keyword, verbatim from `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.stderr`:

> this `deliver` statement selects the macos artifact family, but no expansion runs the offline Metal driver yet, so there is no payload to deliver; a selected family must not silently become fallback on a matching target, so this is a refusal rather than a quiet downgrade. Remove the statement, or state `fallback-only`, to expand with the semantic fallback on every target

It is the `DeliveryRefusal::BackendCompilationUnavailable` path that already existed, now reachable and now spanned on the statement rather than on the invocation. Nothing about it is defensive: `stated_policy` resolves `macos` to a real `SelectedFamilies` policy, `ArtifactFamilySelection::new` validates it, and `invokes_backend_compiler()` is what refuses. `deliver fallback-only;` and the statement's absence are consequently the only spellings an expansion completes.

### The refusal table

Every refusal names one token. Spans below are from the byte-compared goldens.

| Written | Refused at | By |
| --- | --- | --- |
| `deliver macos-and-tvos;` | the first identifier of the name | `StatementRefusal::UnknownProfile` |
| `deliver fallback_only;` | the name | `UnknownProfile` (the underscored near miss) |
| `deliver ios-device 17.0;` | the name | `UnknownFamily` — a driver identifier is not a consumer spelling |
| `deliver macos 13.0;` | the version | `UngovernedTarget`, carrying `MetalTarget::new`'s own `DeploymentMinimumTooLow` |
| `deliver macos 14;` | the literal | `SyntaxError::MalformedDeploymentMinimum` |
| `deliver macos 14.0, ios;` | the `;` | `SyntaxError::ExpectedDeploymentMinimum` |
| `deliver macos 14.0, macos 15.0;` | the second `macos` | `StatementRefusal::RepeatedFamily` |
| two `deliver` statements | the second keyword | `SyntaxError::RepeatedDeliveryStatement` |
| `deliver macos, ios;` | the `,` | `SyntaxError::ExpectedDeliverySpecifier` — neither production |
| `deliver macos-;` | the `;` | `SyntaxError::ExpectedName` |
| `deliver macos 14.0 ios 17.0;` | the `ios` | `SyntaxError::ExpectedPunct` |
| a stated selected family | the `deliver` keyword | `DeliveryRefusal::BackendCompilationUnavailable` |

### Evidence

Five new fixtures and one changed golden under `crates/tiler/tests/facade/`: `pass/deliver_states_fallback_only.rs` (the profile production, executing, and equal to the unstated region), `fail/deliver_selects_an_artifact_family.rs` (both productions reaching the honest refusal — a syntax error there would be a different diagnostic, which is what makes it evidence the family list is admitted), and `fail/deliver_statement_diagnostics.rs` with its byte-compared stderr. `fail/region_syntax_diagnostics.stderr` changed in exactly one line, because `ExpectedStatement` now offers `deliver`.

Unit coverage is in `grammar/tests.rs` (both productions parsed to their exact spans, twelve counted near-miss versions, every statement-level refusal, each with an accepting neighbour differing in one token) and `delivery/tests.rs` (every accepted profile name resolving to its families, the list resolving at stated floors, the list on the governed floors proved *equal* to the profile naming the same families, and each `StatementRefusal`).

Nine deliberate defects were applied and watched failing: an unknown profile resolving to `FallbackOnly`; the governed floor never checked; a repeated family accepted; a second `deliver` overwriting the first; a malformed minimum becoming `0.0`; a selected family downgraded to fallback instead of refused; the statement ignored entirely; an unknown family resolving to macOS; and `ios` dropping the simulator. Every one was caught by the test named for it, and the tree was restored and re-verified green afterwards.

### One closes-when item is not done, and why

**`FamilyDelivery`'s `#[allow(dead_code)]` could not come off.** It was removed and the compiler was asked: `variants Payload and Retained are never constructed`. Nothing constructs one because nothing *runs the driver* — `stated_delivery` refuses every selected family, so every plan an expansion builds is `FallbackOnly`'s, which names no family and carries no outcome. Removing the attribute fails `-D warnings`, and the only ways to satisfy it today are to fake a construction or to wire an emission with no compiled payload behind it, which is the lie this ticket's constraints forbid. The attribute is retained with a corrected reason naming the true cause and `admit-multi-input-elementwise-programs-at-the-compiler-boundary` as what makes it reachable. `NamedProfile`'s allow *did* come off, as did the one on its `impl` block; `MAP_VERSION`'s reason in `family_cfg.rs` was corrected for the same staleness.

`NamedProfile::selection` was removed rather than allowed: with `stated_policy` on the production path, the profile now reaches `ArtifactFamilySelection::new` through `stated_delivery`, so the helper was test-only. Its tests call the canonical constructor directly.

### Consolidation

`NamedProfile`'s family lists and the `deliver` list's family names were one vocabulary in two places waiting to disagree about what `ios` means. `DeliveredFamily` is now the single owner: `NamedProfile::families` names its members, and `deliver ios 17.0;` resolves through the same `platforms()` and the same `governed_minimum()`. `a_family_list_on_the_governed_floors_equals_the_profile_that_names_them` asserts the two are one selection.

### Q-ART-008

Closed and removed from `docs/open-questions.md`, per the corpus's own convention for a closed question (the Q-PKG-005 precedent at `94ae3b9`): the section is deleted and a short prose paragraph at the head of its section records the closure and names the durable authority. Both halves of the close condition now hold — Tom accepted a syntax and a profile-name vocabulary, the grammar admits it, and `delivery::stated_policy` is a function of the parsed region.

### `docs/integration/frontends.md`

Three corrections plus the new material. **The accepted spelling** is stated as a new subsection under Target policy, with both productions, both vocabularies, why `ios` covers device and simulator, why the absence is `fallback-only`, why the attribute form stays eliminated, and what a stated selected family produces today. **The one-envelope staleness the delivery worker reported** is fixed: "expansion embeds its payload under the family's `#[cfg]`" described one artifact per family and is superseded by Tom's 2026-07-25 decision — one envelope embedded once and unconditionally, with the `#[cfg]` gating the payload's *position* — and the correction says so with the decision cited. Two further stale claims were corrected in the same file because they contradicted its own line 166 and the change being made: the status paragraph said "nothing constructs a region's declarations from real tokens, because there is still no grammar to parse them from", and the fusion-boundary section said "`tensor!` has no grammar". Both were false as of `prototype-inline-proc-macro-frontend`; the replacements state what the grammar does and does not admit rather than deleting the caveat.

### Filed

[`draft-an-adr-for-the-inline-delivery-statement`](draft-an-adr-for-the-inline-delivery-statement.md), `todo`, scope `contracts/decisions`. The graph-maintenance section asked for an ADR "if the choice is consequential enough to outlive a ticket", and the neighbouring accepted consumer-visible spelling — the expansion cache root — got ADR 0089 on the same day. `contracts/decisions` is outside this ticket's scopes, so the record is a ticket rather than absorbed.

### Not done, deliberately

No ADR (out of scope, filed above). No `acceleration required` policy — the surface leaves room for it as its own statement and nothing here claims it. No change to `DeliveryPlan`, `items_source`, `family_cfg`, or any emission path: the delivery machinery was wired, not rebuilt, and the three existing fixture-comparison tests still pass unchanged.

## Current-delivery correction — 2026-08-09

The Outcome above accurately records the tree on which the syntax first
landed, but its backend-refusal and dead-code conclusions are no longer current.
The delivery work tracked by
[`prototype-inline-aot-integration-proof`](prototype-inline-aot-integration-proof.md)
and its completed follow-ons now compiles and delivers the selected macOS
family end to end. `FamilyDelivery` is constructed and consumed by the live
delivery path and carries no dead-code allowance, and the old
`BackendCompilationUnavailable` source path is gone. The accepted grammar and
profile-name vocabulary did not change. iOS and a general multi-family delivery
remain deferred to their own authority; this correction does not imply them.
