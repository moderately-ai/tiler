---
schema: "tiler-doc/v1"
id: "tiler.portal.glossary"
kind: "portal"
title: "Glossary"
topics: ["terminology"]
---

# Glossary

Use these terms consistently in documentation, diagnostics, and code.

| Term | Definition |
| --- | --- |
| Access mode | Whether a binding may be read, written, or both. |
| Accumulation dtype | Type specified independently for reduction accumulation; it may equal the input or output dtype. |
| Accuracy contract | Canonical per-operation allowed result set relative to immutable reference semantics, including domains and versioned metrics where applicable. |
| Accuracy guarantee | Machine-checkable result-set claim made by one candidate implementation; it must refine the requested accuracy contract. |
| Artifact | A versioned bundle or kernel-entry record consumed across the compiler/runtime boundary. |
| Axis symbol | Stable frontend identity for a logical axis such as `b`, `h`, or `w`. |
| Bundle | Self-contained target artifact and manifest containing a complete program portfolio; an integration may scope one bundle to one macro invocation. |
| Capability fact | Typed value for a governed target key with explicit availability phase, validity scope, authority, and provenance. |
| Capability phase | One of `CompileProfile`, `ArtifactEvidence`, `LiveDevicePreflight`, `PreparedKernelPreflight`, or `LaunchPreflight`, stating when a physical fact becomes available. |
| Canonical attribute | Host-owned bounded typed value attached to a semantic operation; its normalized encoding, not provider serialization, participates in identity. |
| Byte offset | Offset used by a buffer-binding API, measured in bytes. |
| Candidate region set | Overlapping region candidates considered by program planning; a hypergraph may be used as its internal index. |
| Boundary enforcer | Explicit materialization, layout conversion, cast, or copy that satisfies a boundary requirement. |
| Boundary guarantee | Layout/materialization/storage contract a region implementation provides for an outgoing value. |
| Boundary requirement | Layout/materialization/storage contract a region implementation requires of an incoming value. |
| Buffer plan | Kernel-program mapping from logical values/views to allocation identities and verified lifetime intervals. |
| Collective | Operation involving several hardware lanes or threads, such as a reduction. |
| Compilation request | One semantic graph plus numerical/shape context, frozen operation registry, targets, budgets, and deterministic compiler options. |
| Compile guarantee | Conservative typed capability fact promised by a declared target profile before artifact generation. |
| Element offset | Typed index into elements of a buffer view, distinct from a byte offset. |
| Evidence class | Scope and strength of support for an implementation guarantee: proof, exhaustive, normative guarantee, empirical qualification, or unknown. |
| Expansion compiler cache | Disposable global content-addressed cache used by proc macros to avoid repeated external AOT compilation. |
| Expansion-time AOT | Offline target compilation performed synchronously while a proc macro expands, with completed bytes embedded in returned Rust. |
| Extent expression | Static extent or expression over runtime scalar parameters. |
| Fallback | Semantically compatible alternative execution path used when no compiled variant applies. |
| Fusion visibility boundary | Limit that a frontend can optimize only semantics submitted in its semantic graph; a proc-macro invocation is one such boundary. |
| Applicability predicate | Runtime-checkable condition under which a program or region implementation may execute. |
| Guard | Checked predicate with provenance: semantic input, storage, schedule, target, or dispatch safety. |
| Implementation frontier | Bounded non-dominated region implementations returned for one candidate and target profile. |
| Index-domain predicate | Constraint restricting the mathematical points of an index region; it is part of map truth rather than a physical fallback guard. |
| Iteration domain | Cartesian coordinate space over which outputs are computed. |
| Kernel ABI | Ordered buffer and scalar parameters with types, roles, access modes, and binding locations. |
| Launch geometry | Grid and threadgroup dimensions derived from a schedule. |
| Logical coordinate | One index for each logical tensor axis. |
| Logical shape | Ordered axis extents independent of physical storage. |
| Manifest | Canonical metadata describing a bundle or kernel entry without being executable shader code. |
| Macro-local bundle | Bundle whose collection scope is one inline invocation, though it may contain many kernel entries and steps. |
| Kernel program | Executable dependency DAG of kernel stages, materializations, opaque calls, buffers, and launches. |
| Kernel schedule | Normalized mapping from one region's iteration/access representation onto target execution and memory hierarchy. |
| Live-device capability | Typed fact queried for one runtime device/context and used for preflight or routing, not portable semantic identity. |
| Map/scalar expression | Typed pointwise computation formed while lowering or fusing semantic operations into a region implementation. |
| Materialization | Allocating and storing an intermediate tensor rather than retaining it in a fused expression. |
| Materialization boundary | Kernel-program edge at which an intermediate tensor is stored. |
| Numerical contract | Operation semantics, optimization permissions, and execution guarantees taken together. |
| Numerical policy | Granular optimization permissions such as reassociation, contraction, and approximate intrinsics. |
| Numerical mode | Optional user-facing preset that expands into a complete numerical contract. |
| Opaque call | Physical kernel-program stage implementing a normatively defined semantic operation through an external/library implementation with explicit boundary, target, hazard, and cost contracts. |
| Operation | Atomic named semantic tensor computation consuming values and initially producing one or more individually typed values. |
| Operation definition | Registered versioned capabilities that define an operation's schema, semantics, verification, decomposition, optimization, and lowering support. |
| Operation key | Durable dialect, name, and semantic-version identity stored in semantic IR. |
| Operation registry snapshot | Immutable deterministic per-compilation mapping from semantic operation keys and capability-provider IDs to trusted implementations and revisions. |
| Provider revision | Stable provider-declared fingerprint for output-affecting implementation behavior, distinct from an operation's semantic version. |
| Program portfolio | Guarded alternatives containing complete kernel programs for one semantic graph. |
| Program result | Ordered named reference to a semantic value returned by the graph; it is not an `Output` operation. |
| Prepared-kernel capability | Fact available only after selecting and preparing one entry point/specialization for a live device. |
| Region candidate | Proposed semantic subgraph plus explicit boundary values, retained outputs, materialized edges, duplication policy, and contract. |
| Region partition | Compatible covering set of selected region candidates. |
| Resource estimate | Non-authoritative estimate such as register pressure, occupancy, or code size used for pruning and costing. |
| Resource requirements | Exact quantities or proven upper bounds such as threads, bindings, and local-memory bytes used for feasibility. |
| Reindex | Logical output-to-input coordinate transformation; it does not imply a storage copy. |
| Reduction domain | Coordinates combined to produce one logical output value. |
| Schedule trace | Non-authoritative history of scheduling transforms, parameters, preconditions, and rejections retained for explain/replay. |
| Root binding | Typed declaration mapping a semantic extent symbol to a static value, input dimension, interface parameter, or admitted target property. |
| Routing commit | Boundary after all route-sensitive launch preflight and final variant selection, before output/scratch acquisition or encoding; no later failure selects another plan or semantic fallback. |
| Semantic tensor graph | Public backend-neutral operation/value DAG describing tensor values and named program results as a function over explicit inputs and extent symbols. |
| Semantic authority | The single registered definition owning an operation key's meaning, schema, normative specification, and deterministic inference/validation contract. |
| Shape constraint | Equality, divisibility, interval, or factorization fact required by tensor semantics. |
| Shape environment | Scoped symbolic extent declarations, static/runtime source bindings, semantic constraints, and derived facts. |
| SIMD group | Hardware subgroup of lanes that execute and cooperate; Metal uses this term where CUDA commonly uses warp. |
| Scalable vector shape | Vector lane count expressed as a fixed minimum multiplied by a runtime-stable scale rather than one compile-time width. |
| Source origin | Diagnostic mapping from canonical IR back to frontend source. |
| Storage layout | Base offset and physical strides associated with a logical shape. |
| Target profile | Governed versioned compile guarantees, compatibility, execution/data-layout models, phased query schemas, feasibility rules, and separately identified tuning model used for physical planning. |
| Target property binding | Explicit semantic root binding to a stable, versioned property admitted initially from `CompileProfile` or `LiveDevicePreflight`; later physical phases cannot overwrite it. |
| Target requirement | Canonical bounded predicate over typed capability, candidate-resource, launch, ABI/layout, and binding/access facts required by a selected implementation, possibly deferred to a named safe preflight phase. |
| Tensor access map | Width-independent map from iteration coordinates and admitted parameters to logical tensor coordinates; storage addressing is derived separately. |
| Tensor expression | Pure semantic operation that produces a logical tensor value. |
| Tensor view | Logical shape, strides, and start position over an allocation. |
| Threadgroup | Threads dispatched together with shared synchronization and memory; Metal uses this term where CUDA commonly uses block. |
| Value | Individually typed semantic result with exactly one definition and zero or more consumers. |
| Variant | One complete kernel program plus applicability/routing contract for a semantic graph. |

Avoid using **layout** for both logical axis transformations and physical
strides. Use **reindex** or **axis transform** for the former and **storage
layout** for the latter.

Avoid **mega-kernel** in normative interfaces. It is useful conversationally,
but **fused kernel** or **fusion region** states the compiler concept without
implying that larger is always better.

Avoid unqualified **property** and **physical plan** where a more precise term
exists. Use boundary requirement/guarantee, target requirement, applicability
predicate, resource requirements/estimate, schedule invariant, cost estimate,
`RegionPartition`, `KernelSchedule`, or `KernelProgram` as appropriate.
