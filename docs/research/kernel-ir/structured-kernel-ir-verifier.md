---
schema: "tiler-doc/v1"
id: "tiler.research.kernel-ir.structured-kernel-ir-verifier"
kind: "research"
title: "Structured kernel IR and verifier boundary"
topics: ["kernel-ir", "verification", "scheduling"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.fusion-and-scheduling"]
adopted_by: ["ADR-0048"]
ticket: "structured-kernel-ir-verifier"
---

# Structured kernel IR and verifier boundary

**Status:** research basis for ADR 0048
**Ticket:** `structured-kernel-ir-verifier`

## Conclusion

Tiler should lower each verified `ScheduledRegion` into a typed, structured,
target-neutral kernel body. The body is executable enough for a backend to
translate mechanically, but it is not a second scheduler: execution-coordinate
uses, accesses, stores, barriers, collectives, conversions, and reduction order
all retain stable references to the schedule contracts that authorized them.

The kernel verifier has two jobs. It checks ordinary program well-formedness
(types, lexical dominance, address spaces, access modes, and structured control)
and it checks refinement of the input schedule (ownership, bounds, convergence,
memory visibility, numerical realization, and launch references). Successful
verification means that a backend may choose syntax and target instructions
only within the already selected realization. It does not mean that the target
supports the program; target feasibility remains a separate prior assessment
whose proved or deferred requirements accompany lowering.

## Primary precedents

- MLIR's [SCF dialect](https://mlir.llvm.org/docs/Dialects/SCFDialect/)
  represents `if` and `for` as nested regions with typed yields and loop-carried
  values. This makes lexical structure and value flow explicit before lowering
  to a CFG or final target.
- MLIR's [GPU dialect](https://mlir.llvm.org/docs/Dialects/GPU/) exposes
  invocation coordinates, global/workgroup/private/constant address spaces,
  function-scoped memory attribution, barriers, and collectives. Its stated
  rationale for function-level memory attribution is to keep ownership and
  lifetime visible rather than infer them from target globals.
- MLIR's [side-effect rationale](https://mlir.llvm.org/docs/Rationale/SideEffectsAndSpeculation/)
  distinguishes SSA data flow from effects on named resources. It also warns
  that resource hierarchy is not a substitute for fine-grained alias/range
  analysis. Tiler therefore records operation effects, while bounds and
  ownership remain explicit schedule-derived evidence.
- The [SPIR-V specification](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html)
  uses typed SSA values, typed storage-class pointers, explicit loads/stores,
  and structured control. `OpControlBarrier` separately names execution scope,
  memory scope, and memory semantics, and is only guaranteed in uniform control
  flow. This is direct evidence that one opaque `Barrier` flag is insufficient.
- WGSL specifies a static
  [uniformity analysis](https://www.w3.org/TR/WGSL/#uniformity-analysis)
  because workgroup and subgroup operations must occur in uniform control flow.
  Its scope-sensitive analysis and deliberately conservative composite-value
  behavior are useful precedent for a sound verifier that may reject programs
  it cannot prove.
- LLVM's
  [convergent operation model](https://llvm.org/docs/ConvergentOperations.html)
  shows that convergence constrains legal control-flow transformations, not
  just source placement. Tiler must preserve the convergence obligation through
  later lowering rather than merely check the initial source tree once.

These systems validate the ingredients, not a requirement to reproduce their
IRs. Tiler's distinguishing requirement is a serializable refinement relation
to its normalized schedule and numerical contracts.

## Proposed representation

The notation is structural, not a committed Rust API:

```text
StructuredKernel {
  schema_version,
  scheduled_region_identity,
  signature: KernelSignature,
  local_allocations,
  body: Region,
  derived_requirements,
}

KernelSignature {
  buffers: [BufferParameter],
  scalars: [ScalarParameter],
  specialization_values: [SpecializationParameter],
  admitted_builtins: [BuiltinCoordinate],
}
```

All durable references use canonical newtyped IDs. Types are resolved scalar,
vector, index-role, or governed compound types. Values are immutable SSA-style
results; `For` carries accumulator values through typed region arguments and
`Yield`. The first form need not expose general mutable variables. A backend
may lower those values into mutable source locals without changing IR meaning.

The initial structured operations are deliberately bounded:

```text
Constant        Builtin        Unary          Binary
Convert         Bitcast        CheckedNarrow  Select
For             If             Yield
Load            Store          AtomicUpdate
Barrier         Collective
```

`If` and `For` own single-entry nested regions. Unstructured branches, arbitrary
pointer arithmetic, recursion, calls with unknown effects, and unbounded loops
are outside the first representation. Adding them later is possible, but each
requires corresponding dominance, effect, convergence, and termination rules.

## Values, buffers, and address spaces

A `BufferParameter` or local allocation declares:

```text
BufferRef {
  id,
  element_or_storage_type,
  memory_space,
  access: Read | Write | ReadWrite | Atomic(contract),
  alignment,
  accessible_range,
  source_view_or_staging_allocation,
  alias_class,
}
```

The governed initial memory-space vocabulary is `Device`, `Workgroup`,
`InvocationPrivate`, and `Constant`. These describe visibility and lifetime,
not MSL, CUDA, or SPIR-V spellings. A target profile maps a supported governed
space to its target realization or rejects it. Transparent caches and registers
are not address spaces; they remain lowering/performance facts.

The initial IR uses a buffer reference plus typed allocation-relative element
offset rather than a freely manipulable pointer. This preserves ADR 0046's
logical-access/storage-address split and prevents unchecked pointer arithmetic
from bypassing view ranges or storage encodings. Packed formats may use an
explicit storage-address operation that names its encoding contract.

Each `Load`, `Store`, and `AtomicUpdate` has a statically known effect on a
specific buffer resource. Effects support ordering and dead-code reasoning,
but they do not prove disjointness. The operation also carries a reference to
the derived access and its dominating bounds evidence. Ordinary stores carry a
schedule ownership witness; reductions and atomics carry their selected update
protocol instead.

## Bounds and ownership evidence

The schedule verifier remains authoritative for domain coverage, unique output
ownership, and race freedom. Kernel lowering materializes those decisions and
emits stable witness references:

```text
Load  { buffer, element_offset, bounds_witness, result_type }
Store { buffer, element_offset, value, bounds_witness, ownership_witness }
```

A bounds witness is either a proved range fact or a predicate that dominates
the access and is the schedule's active predicate (or a proven strengthening).
It is not an unchecked assertion authored by a backend. The verifier recomputes
the offset expression's correspondence to the selected `TensorAccessMap` and
`BufferView`, including the chosen coordinate, element-offset, byte/packed, and
dispatch widths.

An ownership witness identifies the schedule output and owning execution
instance. The verifier checks that the store location and control predicate are
the selected realization of that witness. This avoids pretending that type
checking can prove race freedom while also avoiding a second general dependence
analysis after scheduling.

## Control flow, uniformity, and synchronization

Every value has a scope-sensitive uniformity classification derived from its
roots and operations. Examples:

- a scalar parameter and workgroup coordinate are uniform within a workgroup;
- a local invocation coordinate varies within a workgroup;
- a value loaded from device or workgroup memory is conservatively varying
  unless a stronger schedule-derived fact is present;
- a value derived from any varying operand is conservatively varying.

`Barrier` explicitly names:

```text
Barrier {
  schedule_sync,
  execution_scope,
  memory_scope,
  fenced_spaces,
  ordering,
}
```

The verifier checks exact agreement with the scheduled synchronization point,
that all participants reach the same dynamic barrier instance, and that the
named fence covers the producer/consumer effects. A workgroup barrier beneath
an `if` controlled by local invocation ID is rejected. Tail handling instead
predicates the memory effects while keeping a required workgroup barrier
outside the divergent branch. Loops containing barriers require a uniform trip
count at the barrier's execution scope.

Execution scope, memory scope, fenced spaces, and ordering are separate fields.
Some targets combine them in one builtin; that is backend spelling, not a reason
to collapse the portable contract. A backend must also preserve convergence
metadata through any later CFG lowering.

## Reductions and collectives

There is no target-neutral opaque `Reduce` whose order is selected by the
backend. Serial reduction is an explicit `For` with a loop-carried accumulator.
A subgroup or workgroup `Collective` references a scheduled reduction step and
names:

- operation and resolved accumulator/result types;
- participants and execution scope;
- exact combine tree/order and active/identity lanes;
- result owner and visibility;
- numerical realization, including reassociation/permutation permissions; and
- synchronization obligations before and after the operation.

The verifier checks correspondence to `KernelSchedule::reduction_plans`; it
does not re-prove the semantic legality of the selected order. A backend may
use a native collective only when the target profile has proved that the native
operation realizes the exact contract. Otherwise it emits the selected
structured steps or rejects the lowering.

## Conversions

Conversion operations preserve three distinct authorities:

1. A semantic `Convert` references the resolved operation contract, including
   rounding, overflow, NaN, infinity, signed-zero, and subnormal behavior.
2. A representation conversion names a selected storage/provider realization
   and cannot erase a semantic materialization/quantization boundary.
3. `CheckedNarrow` converts index/address roles only with a corresponding proof
   or variant guard for every intermediate under the fixed evaluation order.

`Bitcast` is separate because it preserves bits rather than numerical value.
Backends may not insert value-changing casts for source-language convenience.

## Launch references

Kernel code may read only builtins and parameters admitted by its signature.
Builtins use governed execution-scope and axis keys, such as a workgroup
coordinate or local invocation coordinate, never `thread_position_in_grid` or
`threadIdx.x`. Each builtin reference maps to a schedule execution axis.

The schedule remains the authority for host-evaluable launch formulas. The
kernel does not carry an independently editable grid size. Artifact launch
expressions are checked derivations, and the verifier rejects a free builtin,
specialization value, or scalar binding. Device- or prepared-pipeline facts
remain target requirements/preflight assertions, not readable kernel values
unless a separate semantic target-property binding explicitly authorizes them.

## Verification gates

### Kernel structural/type verifier

- IDs are unique; definitions dominate uses; nested region arguments and
  yields have exact arity and types.
- Every operation has a known governed key and valid typed signature.
- Buffer element/storage types, access modes, address spaces, alignment, and
  local-allocation lifetimes agree with each use.
- Effects are explicit; read-only/write-only restrictions and atomic contracts
  are enforced.
- Loops are bounded and their induction/index arithmetic has the declared
  signedness, width, and overflow contract.

### Schedule-refinement verifier

- The kernel references exactly one verified scheduled-region identity.
- Builtins and lexical loops realize the selected execution-axis mapping.
- Loads/stores derive the selected logical access and physical view address;
  bounds evidence dominates every effect.
- Ordinary stores match output ownership; atomics/reductions match their
  scheduled protocol.
- Barriers and collectives match participant, scope, fence, phase, convergence,
  visibility, and order obligations.
- Tail predicates, inactive lanes, local staging lifetime, and conversions
  preserve the schedule and numerical contracts.
- Derived resource requirements match the schedule rather than forming a
  second editable authority.

### Target/backend preconditions

Before source emission the backend receives:

1. a structurally and refinement-verified kernel;
2. the selected target profile and canonical `TargetRequirement` result;
3. exact/proven `ResourceRequirements` and any named admissible deferred checks;
4. the selected operation/dtype/conversion providers; and
5. the canonical ABI/binding layout needed by the target translation.

The backend verifies that it supports every type, operation, memory space,
builtin, collective, fence, conversion, and source-language feature. It may
choose equivalent syntax, helper functions, instruction selection, and local
variable spelling. It may not change iteration ownership, barriers, reduction
order, address arithmetic, numerical behavior, ABI, or launch formulas. A
lowering gap is a typed backend rejection, not license to approximate.

Backend compiler acceptance and reflection are later validation layers. They
cannot repair or substitute for the kernel verifier.

## Worked lowering sketches

### Predicated fused elementwise

For `z[i] = max(x[i] + y[i], 0)` with 256 local invocations:

```text
group   = builtin WorkgroupCoordinate(X)
lane    = builtin LocalInvocationCoordinate(X)
i       = group * 256 + lane
active  = i < N
if active {
  xv = load x[offset(i)]  bounds tail.x
  yv = load y[offset(i)]  bounds tail.y
  zv = maximum(add(xv, yv), 0)
  store z[offset(i)], zv  bounds tail.z ownership z_by_i
}
```

The body is target-neutral. Metal can spell its builtin parameters and address
spaces only after checking the target profile and ABI.

### Workgroup reduction

Each invocation conditionally loads an input or selects the proved identity,
writes its partial to `Workgroup` storage, then all invocations encounter the
same barrier outside the tail branch. Explicit combine steps and intervening
barriers follow the scheduled tree; only the result owner stores the output.
Moving the first barrier into the bounds `if`, omitting an intermediate fence,
or allowing every lane to store is rejected even though all operations remain
well typed.

## Spike

[`structured_kernel_ir.rs`](../../../spikes/kernel-ir/structured_kernel_ir.rs)
is a dependency-free Rust model. It constructs a valid predicated elementwise
kernel and workgroup reduction, then rejects use-before-definition, wrong
operand and buffer types, illegal access modes, missing bounds/ownership
witnesses, undeclared builtins, a workgroup barrier under lane-varying control,
a nonuniform barrier loop, synchronization mismatch, reduction-order mismatch,
and unsupported backend features.

Run it with:

```sh
rustc --edition 2021 --test \
  spikes/kernel-ir/structured_kernel_ir.rs \
  -o /tmp/tiler-kernel-ir-spike
/tmp/tiler-kernel-ir-spike
```

## Explicit deferrals

- General CFGs, recursion, arbitrary calls, unrestricted pointers, and
  unbounded loops await a demonstrated tensor-kernel use case and verifier.
- Alias-rich in-place writes and indirect gather/scatter need the future effect,
  alias, and collision contracts already deferred by ADR 0046.
- Asynchronous copies and split-phase barriers need dependence tokens or a
  partial-order extension to the scheduled phase model.
- The first implementation may use a conservative uniformity analysis. Better
  proofs may admit more kernels without changing barrier semantics.
- Target-specific operations may live in a later target-lowering IR after this
  boundary; they do not enter the common kernel schema merely because one
  backend exposes them.

These deferrals restrict accepted programs, not the ability to extend the IR
with versioned operations and verifier rules later.
