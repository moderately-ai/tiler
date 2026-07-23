---
schema: "tiler-doc/v1"
id: "ADR-0007"
kind: "decision"
title: "Make normalized hardware schedules first-class IR"
topics: ["scheduling", "ir", "gpu"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.optimizer", "tiler.contract.fusion-and-scheduling"]
evidence: ["tiler.research.scheduling.scheduled-region-model"]
ticket: "scheduled-region-model"
---

# 0007: Make normalized hardware schedules first-class IR

**Status:** accepted

## Context

Tensor dependency graphs do not encode how multidimensional work maps onto GPU
grids, threadgroups, SIMD groups, lanes, vectors, memory hierarchies, or CPU
loops and SIMD execution. These choices affect correctness, target
compatibility, numerical behavior, and cost. They cannot be represented as a
bag of annotations on logical nodes.

Schedule APIs in Halide, TVM, MLIR, XLA, and Triton distinguish, to varying
degrees, transformed executable structure from the history or parameters used
to obtain it.

## Decision

Each scheduled-kernel implementation contains a first-class `ScheduledRegion`
pairing its canonical `IndexRegion` with a normalized `KernelSchedule`. The
schedule explicitly records
loop/tile/vector hierarchy, mappings from hardware coordinates to logical
coordinates, memory placement and staging, reduction topology,
synchronization, tail policy, unrolling/pipelining, launch expressions, and
specialization choices.

The normalized scheduled region is authoritative, serializable,
identity-bearing, and independently verified. All automatic/default decisions
are resolved before identity is formed. The same mapping structure paired with
a different scalar/access program is a different scheduled region.

The normalized schedule contains canonical execution axes, the mapping from
execution coordinates to logical spatial and reduction coordinates, output
ownership, loop/vector organization, tail policy, local staging and lifetimes,
reduction topology and result visibility, synchronization and abstract phase
order, launch formulas, and specialization bindings. The scalar program and
buffer access maps remain owned by `IndexRegion`; a schedule references rather
than restates them.

A separate schedule trace records transformations, decisions, preconditions,
and rejection reasons for explanation and replay. Equivalent normalized
schedules may share identity even when produced by different transform
histories.

Target profiles, selected target requirements, applicability predicates,
resource requirements and estimates, boundary contracts, and cost estimates
remain separate typed concepts rather than undifferentiated schedule
properties.

Verification is layered. Intrinsic verification proves domain coverage,
ownership, bounds under predicates, race freedom, staging lifetime,
synchronization convergence, tail/vector behavior, reduction contributor and
numerical-order legality, and zero-domain behavior. It then derives exact or
proven resource requirements and typed predicates for the separate phased
target-feasibility assessment. Cost evidence never proves legality.

One scheduled region describes one kernel. Cross-kernel temporaries,
dependencies, and multi-pass reductions belong to `KernelSubprogram` or
`KernelProgram`.

## Consequences

- Intrinsic schedule verification proves domain coverage, access safety, race
  freedom, barrier convergence, and numerical preservation before target
  source emission. Phased feasibility separately proves or safely defers
  target/resource predicates.
- Backends consume a verified schedule instead of owning undocumented mapping
  heuristics.
- Explain/replay provenance remains available without making transform history
  the cache or artifact identity.
- Canonicalization must normalize symbolic expressions, coordinate maps,
  defaults, and transient IDs.
- Structured kernel IR remains a later imperative lowering; the schedule does
  not collapse into target source code.
- Rejections are attributed to intrinsic schedule, numerical, applicability,
  target-feasibility, cost, or search-budget rules rather than flattened into
  one unsupported result.

## Alternatives considered

Opaque backend configuration prevents cross-backend verification and complete
artifact identity. Storing only a transform log makes identity depend on
history and replay semantics. Expanding hardware threads into physical-program
graph nodes confuses dependency structure with iteration mapping and creates an
impractically large representation.
