# 0048: Verify structured kernels as schedule refinements

**Status:** proposed

## Context

A normalized `ScheduledRegion` selects execution mapping, memory placement,
write ownership, reduction topology/order, synchronization, tails, and launch
intent. Backends still need a typed executable form with control flow and
memory operations. Merely checking that this form is well typed would allow a
lowering to omit a predicate, move a barrier, change reduction order, or store
from the wrong execution instance while remaining syntactically valid.

Conversely, making the kernel verifier redo scheduling, target feasibility, or
semantic equivalence would create competing authorities and make diagnostics
and artifact identity ambiguous.

## Decision

Lower a verified `ScheduledRegion` to a typed structured kernel IR with lexical
`If`/bounded `For` regions, immutable typed values and loop-carried values,
explicit buffer resources and governed memory spaces, explicit loads/stores/
atomics, typed conversions, governed invocation builtins, and barriers and
collectives with execution, memory, fence, ordering, and convergence fields.

The kernel verifier checks both local well-formedness and refinement of the
referenced schedule. Memory accesses retain schedule-derived bounds evidence;
ordinary stores retain ownership evidence; barriers, collectives, reductions,
tails, conversions, and launch references retain the schedule or semantic
contract that authorized them. The verifier rejects missing or inconsistent
evidence rather than trusting backend-authored assertions.

The first representation uses typed buffer references plus checked element or
storage offsets, not unrestricted pointers. It admits no general CFG,
recursion, unbounded loop, or call with unknown effects.

Target feasibility remains a separate assessment over the scheduled region and
target profile. A backend receives a verified kernel plus the selected target
requirements, resource requirements, providers, and ABI. It may select syntax
and equivalent target instructions, but may not change ownership, addressing,
synchronization, reduction order, numerical behavior, or launch intent.

## Consequences

- Backend source generation is a bounded translation boundary rather than an
  implicit continuation of scheduling.
- A well-typed but physically incorrect lowering fails before source emission.
- Execution scope, memory scope, fenced spaces, and ordering remain distinct
  even when one target builtin combines them.
- Conservative convergence or bounds analysis can reject an otherwise legal
  kernel; improving the proof system expands acceptance without weakening the
  IR contract.
- Target-specific operations may be introduced in a later lowering IR without
  contaminating common kernel identity.

## Alternatives considered

Trusting the schedule verifier alone cannot detect bugs introduced while
materializing structured code. Running only a conventional type verifier misses
ownership, bounds, convergence, numerical, and launch correspondence. Re-running
general schedule legality from emitted code duplicates authority and loses
high-level proof structure. Embedding Metal/CUDA syntax or runtime objects in
the common IR prevents backend independence.
