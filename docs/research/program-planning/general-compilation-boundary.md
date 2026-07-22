---
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.general-compilation-boundary"
kind: "research"
title: "General compilation boundary with bounded capability support"
topics: ["program-planning", "compiler-api", "capabilities", "extensions"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "adopted"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.architecture", "tiler.contract.optimizer"]
adopted_by: ["ADR-0069"]
ticket: "prototype-target-neutral-baseline-slice"
---

# General compilation boundary with bounded capability support

**Status:** research complete; accepted by ADR 0069

**Evidence boundary:** the precedents and dependency argument below are
primary-source synthesis. A private bounded compiler slice exercises part of
the accepted boundary, but no retained experiment supports this report as an
`executable-model` of the general mature contract.

## Question

Should Tiler expose its first compiler slice through a graph-specific entry
point such as `profiles::serial_sum_baseline`, or should the compiler accept a
general semantic program and reject unsupported capability combinations
explicitly?

The serial `Sum` materialized baseline is deliberately narrow. The question is
whether that coverage boundary should become public compiler vocabulary.

## Existing Tiler constraints

**Fact:** the architecture contract already defines a consumer-independent
`CompilationRequest` over an immutable `SemanticProgram`, numerical contract,
shape environment, target profiles, frozen operation capabilities, budgets,
and options.

**Fact:** ADR 0026 separates representability from operation and product
support. ADR 0044 makes semantic and optional compilation capabilities
explicit in a frozen registry. A valid operation may therefore lack a selected
access, scheduling, target, or kernel-lowering provider.

**Fact:** the current executable model recognizes one exact graph and uses
fixed two-stage and three-buffer Rust arrays. Those cardinalities are evidence
about the first strategy, not invariants of `CompilationRequest` or a mature
compiler product.

**Inference:** publishing the fixed normalized graph or its cardinalities would
make current coverage look like the compiler's abstraction. Renaming the same
types behind a general `compile` function would only hide that coupling.

## Primary precedents

### Apache DataFusion

**Fact:** DataFusion's `PhysicalPlanner::create_physical_plan` accepts a
general `LogicalPlan`. `ExtensionPlanner::plan_extension` returns `None` when
one provider does not know a node, allowing another provider to try; the
default planner reports an error when no installed planner can produce an
execution plan.

Source inspected at DataFusion commit
`c3a288b97a1127c11b8c967f64c530d1cb8671b5`:
`datafusion/core/src/physical_planner.rs`.

**Inference:** general planner input does not imply universal physical support.
Capability resolution and explicit failure preserve extensibility without
creating an entry point for every supported logical pattern.

### Apache TVM

**Fact:** TVM exposes `tvm.compile(mod, target, ...)` as a unified entry point
for a `PrimFunc` or `IRModule`. Pipelines and targets determine which contents
can be lowered; the public entry point is not named after the currently
selected operation pattern.

Source: [TVM driver API](https://tvm.apache.org/docs/reference/api/python/driver.html).

### MLIR dialect conversion

**Fact:** MLIR conversion applies a target legality contract to general input
IR. Full conversion succeeds only if every required operation is legalized;
partial and analysis modes expose different incomplete-lowering contracts.
Legality may be dynamic for a particular operation instance.

Source: [MLIR dialect conversion](https://mlir.llvm.org/docs/DialectConversion/).

**Inference:** support is best represented as an instance-sensitive compiler
outcome, not as a claim that every representable operation has a realization.

## Options assessed

### Graph-specific public entry points

This accurately advertises the first executable coverage and can make a small
demo difficult to misuse. It also makes an exact graph pattern part of public
module and type identity, fragments extension dispatch, and creates migrations
whenever coverage grows from one pattern to arbitrary graphs.

### General entry point with an implicit support envelope

This keeps the public boundary aligned with semantic IR. The frozen registry,
compiler/provider revisions, target, numerical contract, and selected options
already determine realizability and output identity. Typed outcomes must
distinguish invalid input, missing capability, target infeasibility, and
internal verifier failure so that “general” does not imply silent fallback.

### General entry point plus a named support policy

A versioned support policy could preserve an intentionally maintained
acceptance envelope across compiler upgrades. Without two real maintained
envelopes, it duplicates the frozen provider set and compiler version and
creates a compatibility promise whose only member is today's test graph.
Search policy or product certification may justify this later; the serial-Sum
pattern does not justify it now.

## Accepted disposition

Use one general consumer-independent compilation boundary. Keep serial `Sum`
as a private strategy, conformance fixture, and explain-rule identity. Do not
publish an `experimental` namespace, a serial-Sum compiler profile, or the
current fixed-cardinality normalized/product types.

Before public exposure, generalize the request and result seams, use
variable-length verified collections for program cardinalities, and classify
unsupported capability and no-feasible-plan outcomes explicitly. The compiler
may initially realize only the accepted serial-Sum slice and must reject every
other valid program without approximation.

Defer a selectable support-policy type until at least two deliberately
maintained policies, a product-certification requirement, or a compatibility
need independent of the pinned compiler/provider identity exists.
