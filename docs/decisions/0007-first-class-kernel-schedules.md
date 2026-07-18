# 0007: Make normalized hardware schedules first-class IR

**Status:** proposed

## Context

Tensor dependency graphs do not encode how multidimensional work maps onto GPU
grids, threadgroups, SIMD groups, lanes, vectors, memory hierarchies, or CPU
loops and SIMD execution. These choices affect correctness, target
compatibility, numerical behavior, and cost. They cannot be represented as a
bag of annotations on logical nodes.

Schedule APIs in Halide, TVM, MLIR, XLA, and Triton distinguish, to varying
degrees, transformed executable structure from the history or parameters used
to obtain it.

## Proposed decision

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

A separate schedule trace records transformations, decisions, preconditions,
and rejection reasons for explanation and replay. Equivalent normalized
schedules may share identity even when produced by different transform
histories.

Target profiles, selected target requirements, applicability predicates,
resource requirements and estimates, boundary contracts, and cost estimates
remain separate typed concepts rather than undifferentiated schedule
properties.

## Consequences

- Schedule verification can prove domain coverage, access safety, race freedom,
  barrier convergence, numerical preservation, resource feasibility, and
  target compatibility before target source emission.
- Backends consume a verified schedule instead of owning undocumented mapping
  heuristics.
- Explain/replay provenance remains available without making transform history
  the cache or artifact identity.
- Canonicalization must normalize symbolic expressions, coordinate maps,
  defaults, and transient IDs.
- Structured kernel IR remains a later imperative lowering; the schedule does
  not collapse into target source code.

## Alternatives considered

Opaque backend configuration prevents cross-backend verification and complete
artifact identity. Storing only a transform log makes identity depend on
history and replay semantics. Expanding hardware threads into physical-program
graph nodes confuses dependency structure with iteration mapping and creates an
impractically large representation.
