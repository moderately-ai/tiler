---
schema: "tiler-doc/v1"
id: "ADR-0073"
kind: "decision"
title: "Own typed explain infrastructure in tiler-compiler"
topics: ["explain", "architecture", "dependencies", "rust"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.optimizer"]
evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv", "tiler.research.program-planning.general-compilation-boundary"]
refines: ["ADR-0070"]
ticket: "record-explain-ownership-decision"
---

# 0073: Own typed explain infrastructure in tiler-compiler

**Status:** accepted

## Context

The typed explain authority merged as `tiler_compiler::explain`: stable stage,
disposition, reason, rule, and provider keys, typed subject references, evidence
classes, bounded retention, causal integrity, and a presentation-only renderer.
Its review handoff deferred one packaging question — whether that vocabulary
belongs to `tiler-compiler` or to a separate `tiler-explain` crate.

Three properties of the merged implementation answer it.

Explain's subjects are compiler-internal. The module imports
`crate::fusion::FusionNumericalProof` and
`crate::request::{LoweringProviderIdentity, VerifiedTargetRequest}`, each defined
in `tiler-compiler`, while the pipeline emits through `crate::explain`. A
`tiler-explain` crate would therefore need `tiler-explain -> tiler-compiler` for
its subject types and `tiler-compiler -> tiler-explain` for emission: a Cargo
cycle. Extraction is not a file move; it first requires relocating those subject
types.

There is no second consumer. `tiler-artifact` depends on `tiler-ir` alone, no
crate outside `tiler-compiler` names an explain type, and the artifact envelope
contract does not contemplate embedding canonical traces. Explain also shares
almost nothing with the IR: two of the module's 3,047 lines mention `tiler_ir`,
and both are test imports.

ADR 0070 had just consolidated shared target-neutral IR into `tiler-ir` and
removed the unused compiler-to-artifact edge. Adding a package boundary with no
semantic boundary immediately afterward would cut against that consolidation and
repeat the pattern ADR 0068 rejected for a generic expression crate.

## Decision

Typed explain infrastructure is a module of `tiler-compiler`. Do not add a
`tiler-explain` crate.

`tiler-compiler` owns explain record construction, canonical identity, retention
bounds, causal integrity, and the versioned renderer. Emission stays
compiler-owned: sibling compiler modules obtain record handles from a writer, and
this decision publishes no provider-facing emission trait.

Module visibility is not a crate decision. `tiler_compiler::explain` is private
today, and the reviewed public compiler facade may promote it to `pub` without
revisiting this ADR.

**Reconsideration trigger.** If a second crate must ever read canonical explain
traces, the record, subject, and disposition vocabulary moves into `tiler-ir`
following the `AbiExpr` co-location precedent of ADRs 0068 and 0070, with
emission staying compiler-owned. A new crate is not the expansion path.

## Consequences

- The accepted packaging profile is unchanged: explain adds no package, no
  dependency edge, and no second authority over compiler decisions.
- Explain records reference compiler-internal subjects directly instead of
  generic or opaque placeholder types invented to cross a package boundary.
- Promoting the module to `pub` is a facade review under the public compiler
  boundary, not a packaging change.
- Because the expansion path targets `tiler-ir`, a future reader inherits the
  shared-IR checked-construction and identity rules of ADRs 0070 and 0071 rather
  than a parallel explain authority with its own verifier.
- A component that must emit explain records without depending on
  `tiler-compiler` is outside this contract until the trigger fires. It is an
  explicit unsupported case, not grounds for a private copy of the vocabulary.

## Implementation boundary

`tiler_compiler::explain` is implemented and integrated for the bounded compiler
slice: each successful target compilation product carries a sealed,
request-qualified `VerifiedExplainTrace`, and the normalization, fusion,
feasibility, costing, selection, kernel, program, and artifact-plan flow emit
typed records. Every item in the module is `pub(crate)`, `lib.rs` declares only
`mod explain;`, and the crate exports nothing new.

The placement this decision fixes is therefore realized in merged code, while the
public explain surface, whether canonical traces are ever serialized, renderer
and redaction guarantees, and the stage coverage of a general optimizer remain
unimplemented. Those belong to the reviewed public compiler boundary, not to this
ownership decision.

## Alternatives considered

A `tiler-explain` crate packages the vocabulary before an independent consumer
exists and today requires either a Cargo cycle or a preparatory relocation of
fusion and request types. Moving the vocabulary into `tiler-ir` now would add a
shared-IR authority with no second reader and would place optimizer-stage
vocabulary — stage names, dispositions, budget stops — in the foundational
representation crate before a non-compiler consumer constrains its shape; that
move is the accepted expansion path, not the current state. Keeping explanations
as presentation strings inside each pass was already rejected: rejection reasons
must be typed data with stable keys so they can be compared, tested, and rendered
without becoming the contract.
