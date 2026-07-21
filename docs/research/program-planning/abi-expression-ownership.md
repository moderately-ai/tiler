---
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.abi-expression-ownership"
kind: "research"
title: "Ownership of target-neutral ABI expressions"
topics: ["program-planning", "abi", "expressions", "rust"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.artifact-abi"]
ticket: "prototype-target-neutral-baseline-slice"
---

# Ownership of target-neutral ABI expressions

**Status:** research complete; crate-placement recommendation awaits acceptance

## Question

Where should the closed, typed `AbiExpr` language live when a target-neutral
`KernelProgram` uses it for guards, output and temporary byte sizes, launch
geometry, and scalar ABI values, while a later artifact envelope serializes and
evaluates the same expressions at runtime?

The issue is ownership rather than arithmetic syntax. The authoritative AST,
source vocabulary, validation, canonical identity, and checked evaluation
semantics must agree. Serialization codecs, runtime fact binding, and
backend-transport mappings are separate consumers.

## Existing Tiler constraints

**Fact:** accepted Tiler design keeps `ShapeExpr` and `AbiExpr` as distinct
newtyped domain IRs. They may share implementation components, but have
different source vocabularies, validation, identity, and versioning. Lowering
from `ShapeExpr` to `AbiExpr` is explicit. See the
[shape-environment research](../shapes/shape-environment-contract.md#accepted-decision-domain-specific-expression-irs-over-shared-components).

**Fact:** the IR contract currently assigns field-level `KernelProgram` and
`BufferPlan` models and their canonical identity and verifiers to the IR
contract. The artifact contract owns their serialized envelope and execution
routing. The artifact research says the neutral layer owns the expression DAG
and the values it computes, while backend payloads only map stable neutral IDs
to native transport.

**Fact:** `AbiExpr` sources are not foundation-free arithmetic variables. They
refer to typed program/interface facts such as input dimensions, materialized
values, view ranges, target properties, and prepared-entry facts with declared
availability phases. Those IDs and phase contracts are part of the executable
program model.

**Inference:** putting a public `AbiExpr` type in a lower generic expression
crate would either move program-specific IDs into that crate or parameterize
source identity behind a generic/opaque mechanism. Both weaken the domain
boundary accepted for `AbiExpr`. A private shared arithmetic implementation can
still be extracted later without moving the public domain type.

## Primary precedents

### IREE HAL

**Fact:** IREE's `hal.executable.export` owns a `workgroup_count` region that
computes the three dispatch grid dimensions from the captured workload. The
same executable export may own a condition region that chooses whether an
entry point applies and names a compatible fallback. IREE documents host and
indirect execution consequences based on where dynamic workload information is
available. These computations are executable-plan content, not fields invented
by a binary-container codec.

Source: [IREE HAL executable export](https://iree.dev/reference/mlir-dialects/HAL/#halexecutableexport-hal-executableexportop).

**Inference:** launch and applicability computation belongs with the executable
entry contract whose correctness and fallback behavior it controls. Artifact
serialization may carry that contract, but does not become its semantic
authority merely because the runtime consumes it.

### MLIR GPU and MemRef dialects

**Fact:** MLIR's `gpu.launch_func` takes grid sizes, block sizes, dynamic shared
memory size, and kernel operands as typed SSA operands. `memref.alloc` similarly
takes dynamic sizes and layout symbols as operands of the allocation operation.
The operations and their dynamic computations remain in executable IR; later
lowering translates them to target/runtime mechanisms.

Sources: [MLIR GPU dialect](https://mlir.llvm.org/docs/Dialects/GPU/#gpulaunch_func-gpulaunchfuncop),
[MLIR MemRef dialect](https://mlir.llvm.org/docs/Dialects/MemRef/#memrefalloc-memrefallocop).

**Inference:** dynamic launch and allocation values are part of the verified
program relation, even when their producer expression language is shared with
other IR operations. Encoding them separately from the program must preserve,
not recreate, that relation.

### Apache TVM TensorIR

**Fact:** TVM places `PrimExpr`, buffers whose shapes/strides/offsets are
`PrimExpr`s, loop extents, and thread-launch extents in its low-level TensorIR
definitions. Scheduling and arithmetic analysis operate on those IR
expressions; target code generation consumes the resulting IR later.

Sources: [TVM architecture](https://tvm.apache.org/docs/arch/index.html#tvm-tirx),
[TVM TIR builder reference](https://tvm.apache.org/docs/reference/api/doxygen/namespacetvm_1_1script_1_1ir__builder_1_1tirx.html).

**Inference:** TVM favors co-owning the expression representation with the
low-level executable IR it parameterizes, while keeping scheduling and target
code generation as consumers.

### Apache DataFusion

**Fact:** at local DataFusion commit
`c3a288b97a1127c11b8c967f64c530d1cb8671b5`, `PhysicalExpr` and direct
evaluation live in `datafusion-physical-expr-common`; the physical-plan crate
depends on the physical-expression crates. The published common expression
crate is an independently reused subsystem with Arrow evaluation, bounds,
statistics, placement, tree traversal, and a source size reported by docs.rs at
roughly 353 KiB for version 54.0.0.

Sources: inspected local
`datafusion/physical-expr-common/src/physical_expr.rs` and both crates'
`Cargo.toml`; [DataFusion `PhysicalExpr`](https://docs.rs/datafusion/latest/datafusion/physical_expr_common/physical_expr/trait.PhysicalExpr.html),
[`datafusion-physical-expr-common`](https://docs.rs/crate/datafusion-physical-expr-common/latest).

**Inference:** DataFusion supports extracting an expression crate once the
expression system is a large, independently consumed boundary. It does not
support extracting a tiny domain IR before its source identities and reuse
boundary are stable. Its physical plan still depends on the expression
authority rather than carrying opaque expression IDs whose meaning is owned by
a serialization crate.

## Dependency analysis

### `AbiExpr` in `tiler-artifact`

This creates an invalid direction if `KernelProgram` remains in `tiler-ir`:

```text
tiler-ir -> tiler-artifact -> tiler-ir
```

Keeping only opaque expression IDs in `KernelProgram` avoids the Cargo cycle
but makes the program non-self-contained: its verifier, canonical identity,
launch coverage, buffer sizes, and routing meaning require an external artifact
side table. It also makes artifact construction, rather than physical planning,
the first point at which a complete executable program exists.

### `AbiExpr` in `tiler-compiler`

This avoids a direct cycle but forces runtime and artifact validation to depend
on optimizer implementation or duplicate the expression schema and evaluator.
It also prevents backends and third-party plan producers from constructing a
complete target-neutral program without compiler internals.

### a new public expression crate

A new crate gives clean reuse only if its public concepts are genuinely below
both IR domains. `AbiExpr` is not: its sources and phases are program-specific.
The crate could own generic checked arithmetic internals, but the public
`AbiExpr` newtype, roots, validation, and identity would still belong with the
program IR. Extracting private mechanics before there is an independent
consumer adds a package boundary without resolving a semantic boundary.

### `AbiExpr` in `tiler-ir`

This yields the acyclic graph already selected for the prototype:

```text
tiler-ir
  ^       ^
  |       |
compiler  artifact -> runtime integrations
  ^
  |
backends also consume verified IR directly
```

`tiler-ir` owns the domain type, source vocabulary, validation, canonical
identity, and one authoritative checked evaluator. `tiler-artifact` owns the
versioned wire encoding, envelope validation, compatibility policy, runtime
fact binding, and failure classification. Backends own native transport
mappings. `tiler-compiler` owns lowering from `ShapeExpr` and construction of
program expressions.

## Recommendation

**Proposal:** place public `AbiExpr` and its authoritative pure checked
evaluation semantics in the same experimental `tiler-ir` physical-program
surface as `KernelProgram`. Keep `ShapeExpr` a distinct newtyped IR. Share
private arithmetic components only where semantics coincide. Keep artifact
serialization and runtime binding in `tiler-artifact`.

Do not add a public expression crate for the prototype. Reconsider extraction
only when at least one of these triggers occurs:

1. a second crate needs the arithmetic engine without depending on Tiler IR;
2. expression compilation/evaluation becomes a material incremental-build or
   code-size boundary;
3. multiple domain IRs need a stable public algebra below their distinct root
   vocabularies; or
4. the artifact/runtime must be distributed independently of the IR crate.

Even then, extract shared mechanics; do not erase the nominal distinction or
source contracts of `ShapeExpr`, index expressions, and `AbiExpr`.
