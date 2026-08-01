---
id: accept-the-inline-artifact-family-profile-syntax
title: Decide how a consumer states an artifact-family delivery profile
status: in-progress
priority: p1
dependencies: [generate-cfg-gated-artifact-family-delivery]
related: [prototype-inline-aot-integration-proof, prototype-inline-proc-macro-frontend]
scopes: [implementation/frontend]
shared_scopes: [contracts/navigation, contracts/integrations]
paths: []
tags: [decision, public-boundary, inline-dx, frontend, apple-targets]
claimed_from: todo
assignee: worker-deliver-syntax
lease_expires_at: 1785554584
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
