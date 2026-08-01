---
id: deliver-several-artifact-families-from-one-expansion
title: Deliver several artifact families from one expansion
status: in-progress
priority: p2
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/frontend, implementation/build, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, inline-dx, artifacts]
claimed_from: todo
assignee: worker-deliver-seve
lease_expires_at: 1785564763
---
## Why this exists

Tom decided on 2026-07-25 that one selection produces **one envelope carrying one payload per built family**, and `tiler_macros::delivery::DeliveryPlan` implements the emission half completely: positional outcomes, a total `#[cfg]` selector, and one byte-string literal. Nothing produces a multi-payload envelope for it.

**Fact.** `tiler_build::accept_or_publish_single_payload_metal_artifact` refuses anything but exactly one payload (`MetalArtifactProtocolError::PayloadPortfolio`), and `accept_or_publish_metal_plan` reads position 0 alone.

**Fact.** `tiler_build::BoundMetalCompileDeclaration` publishes one constructor, `first_macos_apple9`, and its documentation states that widening to another Apple family is "a new measurement rather than a new argument". So a second family has no compile-time declaration to be compiled against even if the envelope could carry it.

**Consequence, today.** `deliver ios;` and `deliver macos-and-ios;` are refused by `tiler_macros::aot::require_buildable`, naming the one target the frontend builds. `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.rs` and its golden pin both refusals.

## Closes when

A selection naming several families compiles each against its own bound declaration, produces one envelope carrying one payload per built family in canonical order, and the emitted selector routes each consumer target to its own payload — with a test that a wrong-family payload position is a build error rather than a wrong artifact. The measured second declaration is a prerequisite and may be its own ticket.
