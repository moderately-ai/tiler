---
id: state-a-subject-on-the-contraction-publication-path-s-reference-oracle
title: State a subject on the contraction publication path's reference oracle
status: todo
priority: p2
dependencies: []
related: [route-the-realization-conformance-half-into-the-conformance-crate, give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The asymmetry

`crates/tiler-conformance/src/publication/proof.rs` computes every published expectation through `ReferenceEvaluator::standard()` → `under(registry, strict())`. `strict()` produces **`ConformanceSubject::Unstated`**, which reaches every capability **unchecked** — while the artifacts those expectations are compared against are compiled under `FLUSH_SUBNORMALS_TO_ZERO_F32`.

So the oracle is not told the contract the device half runs under. That is the same window `give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject` closed for the BF16 vertical, still open on the contraction path.

**Unobservable on the current operands, and that is not a reason to leave it.** The probe stream is `m·2⁻²⁴`, so no subnormal arises and the two readings agree on every value the corpus contains. The agreement is a property of the operands, not of the contract — a corpus that grew a subnormal would silently compare a flushing device against a preserving oracle, which is exactly the failure the subject exists to refuse.

## Why this is a design step and not a rename

Routing it through `from_realization` needs a **`VerifiedScheduledRegion`**, because that is what `RealizationWitness::of` requires — and the contraction publication path does not hold one. The BF16 vertical could do this because it assembles its region directly; the contraction path receives a plan.

So the work is to establish **where the subject comes from on a plan-derived route**, which is a question about what the publication path is handed, not a substitution at the call site. Answer that before writing anything.

`strict()` and `new()` keeping `Unstated` is deliberate and stated — see the bridge's own record. This ticket does not change them; it stops this route relying on them.

## Read in full first

`crates/tiler-conformance/src/publication/proof.rs`; `crates/tiler-conformance/src/bf16_vertical.rs`'s `conformance_of`, which is the closed case to model against; and `crates/tiler-reference/src/conformance.rs` for what `from_realization` requires and refuses. `crates/tiler-reference/**` is **out of scope** — read it, never edit it; if the route needs a signature change there, stop and report, exactly as the two previous workers on this thread did.

## Closes when

The contraction publication path's oracle carries a subject derived from what the plan declares, or the reason it cannot is recorded with the evidence — and a test watches the bridge's refusal **fire** on this route, perturbing the subject rather than an assertion.
