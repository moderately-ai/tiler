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

A backend-author conformance suite can assert that each provider disposition it must distinguish — a suite-facing superset of the ADR 0078 / operation-extensions five (admission, rejection, ambiguity, absence, exhausted proof budget), not a contract quote of that list alone; declined strategy and cost disadvantage are example optimizer outcomes the suite may also enumerate — reached the explain trace as its own outcome, without depending on rendered text the contract says is not a parse target.

## Why this is deferred rather than dispatchable

**Fact — the obligation exists and the surface to assert it against does not.** [The operation-extension contract](../docs/operation-extensions.md#public-extension-seams) makes it one of the four properties jointly admitting a seam that "every disposition — admission, rejection, ambiguity, absence, and an exhausted proof budget — is a distinct typed outcome that reaches the explain trace". [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md) asks a suite to cover empty offers, malformed proposals, verifier bypass attempts, and forged provenance/resources, and to document exact maturity.

**Fact — public explain products expose no disposition iteration, and documentation forbids treating rendered text as an interface.** `ExplainReport` (`pub struct ExplainReport` in `crates/tiler-compiler/src/session.rs`) exposes only `ExplainReport::render` (`pub fn render(&self) -> String` under the "not a parse target" doc). Success-path `VerifiedCompilationExplain` (re-exported via `pub use crate::explain::VerifiedCompilationExplain;` in `session.rs`) publicly exposes `render` and `semantic_candidate_count` only — neither yields dispositions or record iteration. `mod explain;` is private in `crates/tiler-compiler/src/lib.rs`, so an out-of-crate suite cannot name `ExplainDisposition`. The `ExplainReport` doc states that the rendered form "is a diagnostic for a human reader and **not a parse target**", that the leading `tiler-explain-v<N>` changes when the rendering does, and that committing to the text "would create a second description of the trace that has to be kept in agreement with its canonical bytes, which is the duplicate-derivation hazard this whole boundary is shaped to avoid".

**Correction — 2026-08-10.** Prior line citations for `ExplainReport` (`session.rs:1137`), `render` (`:1176`), `mod explain` (`lib.rs:22`), and the re-export (`session.rs:65`) have drifted; use the symbol and doc anchors above. The prior claim that the only public accessor is a rendered string understated that `VerifiedCompilationExplain` also exposes `semantic_candidate_count`; the obligation gap is unchanged because neither product exposes dispositions. The User-visible outcome seven-item disposition list is a suite-facing superset, not a quote of ADR 0078's five-item obligation.

**Inference — so a conformance suite today has three options and each is a decision nobody has made.** Parse the rendered text, which the contract above forbids and which would create the second description it exists to prevent; add a structured accessor over the trace, which is a public boundary and Tom's under ADR 0075; or assert dispositions only indirectly through the typed errors and selected-plan provenance already public, and state in the suite's documented maturity that explain coverage is out of scope. The third is cheapest and may well be right, but choosing it silently would let a suite report full coverage of an obligation it does not check.

**This is deferred because the suite that needs it does not exist yet.** Designing an assertion surface before the suite has named which dispositions it must distinguish would fix a shape against guessed requirements — the failure the conformance ticket's own key warns against when it says to extract tests only after the vertical identifies the real public contracts, and not to design a mock-only alternate API.

## Ripens when

The backend-provider conformance suite reaches design and enumerates which dispositions it must distinguish. At that point this becomes one atomic question for Tom — structured accessor, or documented scope limit — with the cost of each stated.

## Closes when

The suite either asserts dispositions through a surface that is not the rendered text, or documents explain coverage as explicitly out of its scope with the reason; and no suite reports coverage of the disposition obligation it does not check.

## Trigger check log

- 2026-08-05 — not fired. `publish-the-backend-provider-conformance-suite` is `todo` and itself blocked on `exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`, so no suite has enumerated its dispositions. Reproduce with `grep -m1 '^status:' tickets/publish-the-backend-provider-conformance-suite.md`.
- 2026-08-09 — **not fired.** [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md) remains `todo`, and its end-to-end portfolio consumer [`exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`](exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio.md) remains `todo`. No published suite yet consumes explain dispositions as an assertable provider contract, so this stays deferred behind that subject rather than being implemented against local trace fixtures alone.
