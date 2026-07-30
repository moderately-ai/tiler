---
id: accept-the-delivered-realization-artifact-surface
title: Accept the delivered-realization artifact surface
status: todo
priority: p1
dependencies: [redesign-the-delivered-realization-record-from-typed-evidence]
related: [record-delivered-numerical-realization, accept-adr-0076-numerical-realizations, carry-the-honourability-fact-provenance-into-the-artifact-record, wire-the-delivered-realization-record-into-the-artifact]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [api, decision, numerics, artifact, needs-tom]
---
## User-visible outcome

Tom receives one exact, compile-checked public-boundary proposal whose scalar-arithmetic record is complete over the governing eleven-dimension numerical contract, identifies the dtype-wide contract by canonical resolved-type identity, distinguishes operation and policy loci with stable obligation keys, preserves structured honouring means and provenance, and reserves future operation- or scheme-owned contract families without applying floating semantics universally.

## Withdrawal of the former decision

The 2026-07-28 request to accept the staged `DeliveredRealizationBuilder` and readers is withdrawn. Source review disproved its premises:

- the staged record has four fixed dimension fields while the governing compiler vocabulary has eleven and the widened scheduled/artifact realization has eight;
- the accessor omits arithmetic dtype despite a measured profile giving different `f16` and `f32` answers for the same dimension;
- the artifact dimension enum is a drifting duplicate;
- opaque means bytes erase the payload of a declared relaxation and cannot support the reference-comparison consumer the ADR names;
- required authority, validity, compiler-build, and execution-environment provenance is absent;
- a raw byte declaration accepts caller assertions rather than compiler-selected typed evidence; and
- `UnrecordedRealization` is temporary migration state immediately contradicted by the required terminal record.

Publishing that draft would be incorrect and less maintainable, so Tom is not being asked to choose between it and delay. Delay is correctness-derived until a complete concrete replacement exists.

## Boundary that will be reviewed

`redesign-the-delivered-realization-record-from-typed-evidence` owns the exact compile-checked review packet, including proposed signatures, call sites, contract text, and a bounded spike or equivalent private draft. This ticket becomes `awaiting-decision` only after that packet has:

- one shared exhaustive eleven-dimension scalar-arithmetic authority;
- checked compiler-produced policy subjects carrying canonical resolved-type identity;
- complete scalar-arithmetic contract rows, canonical policy-locus obligations, and complete required/not-required assessment coverage;
- structured means and complete provenance;
- compact dense scalar-arithmetic storage, sorted sparse evidence, and borrowed allocation-free lookup;
- exact proposed top-level readers with no optional-record state;
- an exhaustive proposed `tiler-build` translation from compiler-produced typed evidence;
- typed proposed construction/decode refusals and explicit identity/schema migration work owned downstream; and
- adversarial fixtures whose failure paths were each observed.

Tom then reviews the exact public crate/module/type/constructor/reader/error boundary, not an abstract promise that implementation will later fill.

## Design constraints already derived

- One artifact-wide record is correct while the artifact builder enforces one numerical contract and target profile across the portfolio. A future multi-subject portfolio must move the record to its subject.
- Exhaustive dimension handling is required; `#[non_exhaustive]` is wrong for a vocabulary total encoders and renderers must cover.
- Eleven named fields are eliminated in favour of one dense scalar-arithmetic schema because named fields duplicate the dimension set and force public churn.
- Raw means strings are eliminated because they erase structured relaxation meaning and invite a second authority in consumers.
- Every scalar-arithmetic dimension records whether any packaged route requires it. A required disposition names a non-empty canonical range of locus-specific obligations and evidence; `NotRequired` is an explicit compiler-produced assertion rather than a fact the artifact decoder can independently derive.
- Unknown numerical record families or tags reject fail-closed; an older reader never skips them and still calls an executable artifact validated.
- Recognition of a dtype identity never creates a policy subject or implies operation, reference, storage, dispatch, lowering, runtime, or backend support.

## Closes when

The redesign dependency is done; its exact tested public diff and call site are presented here; every candidate alternative that fails correctness, performance, or long-term maintainability is explicitly eliminated; Tom ratifies the surviving boundary; and `wire-the-delivered-realization-record-into-the-artifact` is unblocked against that accepted shape.

## Graph maintenance

- The target/profile provenance producer now precedes the redesign instead of depending on later artifact wiring.
- Keep `wire-the-delivered-realization-record-into-the-artifact` blocked on this acceptance ticket.
- Qualify the completed four-dimension draft as historical evidence and remove its stale current recommendation from downstream briefs.
- Do not return this ticket to `awaiting-decision` until the compile-checked review packet exists and has passed its targeted checks. Production wiring, codec, readers, identity/schema movement, and rebaselines remain downstream.
