---
id: make-explain-dispositions-assertable-by-a-conformance-suite
title: Make explain dispositions assertable by a conformance suite
status: deferred
priority: p2
dependencies: [publish-the-backend-provider-conformance-suite]
related: [audit-backend-authoring-against-all-thirteen-responsibilities, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [explainability, conformance, backend-providers, deferred]
---
## User-visible outcome

A backend-author conformance suite can assert that each provider disposition — admission, rejection, ambiguity, absence, exhausted budget, declined strategy, cost disadvantage — reached the explain trace as its own outcome, without depending on rendered text the contract says is not a parse target.

## Why this is deferred rather than dispatchable

**Fact — the obligation exists and the surface to assert it against does not.** [The operation-extension contract](../docs/operation-extensions.md#public-extension-seams) makes it one of the four properties jointly admitting a seam that "every disposition — admission, rejection, ambiguity, absence, and an exhausted proof budget — is a distinct typed outcome that reaches the explain trace". [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md):20 asks a suite to cover "empty offers, malformed proposals, verifier bypass attempts, forged provenance/resources" and :24 to document exact maturity.

**Fact — the only public accessor is a rendered string, and its documentation forbids treating it as an interface.** `ExplainReport` (`crates/tiler-compiler/src/session.rs:1137`) exposes `render(&self) -> String` (`:1176`) and nothing else; `mod explain` is private (`crates/tiler-compiler/src/lib.rs:22`), and only `VerifiedCompilationExplain` is re-exported (`session.rs:65`). The doc comment at `session.rs:1152-1158` states that the rendered form "is a diagnostic for a human reader and **not a parse target**", that the leading `tiler-explain-v<N>` changes when the rendering does, and that committing to the text "would create a second description of the trace that has to be kept in agreement with its canonical bytes, which is the duplicate-derivation hazard this whole boundary is shaped to avoid".

**Inference — so a conformance suite today has three options and each is a decision nobody has made.** Parse the rendered text, which the contract above forbids and which would create the second description it exists to prevent; add a structured accessor over the trace, which is a public boundary and Tom's under ADR 0075; or assert dispositions only indirectly through the typed errors and selected-plan provenance already public, and state in the suite's documented maturity that explain coverage is out of scope. The third is cheapest and may well be right, but choosing it silently would let a suite report full coverage of an obligation it does not check.

**This is deferred because the suite that needs it does not exist yet.** Designing an assertion surface before the suite has named which dispositions it must distinguish would fix a shape against guessed requirements — the failure the conformance ticket's own key 19 warns against when it says to extract tests only after the vertical identifies the real public contracts, and not to design a mock-only alternate API.

## Ripens when

The backend-provider conformance suite reaches design and enumerates which dispositions it must distinguish. At that point this becomes one atomic question for Tom — structured accessor, or documented scope limit — with the cost of each stated.

## Closes when

The suite either asserts dispositions through a surface that is not the rendered text, or documents explain coverage as explicitly out of its scope with the reason; and no suite reports coverage of the disposition obligation it does not check.

## Trigger check log

- 2026-08-05 — not fired. `publish-the-backend-provider-conformance-suite` is `todo` and itself blocked on `exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`, so no suite has enumerated its dispositions. Reproduce with `grep -m1 '^status:' tickets/publish-the-backend-provider-conformance-suite.md`.
