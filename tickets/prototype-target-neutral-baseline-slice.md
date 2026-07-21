---
id: prototype-target-neutral-baseline-slice
title: Compile serial Sum into a verified materialized baseline plan
status: in-progress
priority: p0
dependencies: [prototype-semantic-reference-slice]
related: [prototype-artifact-slice]
scopes: [implementation/compiler, implementation/artifact, implementation/ir]
shared_scopes: [project/tickets, contracts/optimizer, contracts/artifacts, contracts/foundation, contracts/numerics]
paths: [Cargo.lock]
tags: [implementation, prototype, compiler, vertical-slice]
claimed_from: todo
assignee: codex
lease_expires_at: 1784606188
---
Compile the accepted immutable semantic program into one complete verified
materialized baseline. The bounded `CompilationRequest` must make its static
shape environment, numerical contract, frozen operation capabilities,
deterministic budgets, and conservative prototype target profile explicit; the
compiler must not obtain target facts from ambient Metal state. Target-neutral
describes representation ownership: physical requirements remain explicit, but
no Metal source, binary, compiler object, or live device enters the plan.

Exercise the complete lowering and verifier path without implementing the
optimization under test:

- verify and deterministically normalize the semantic request;
- form one pointwise region for multiply/add and one strict-`Sum` region,
  preserving their observable materialization boundary;
- derive canonical iteration and access maps with read-bounds and unique-write
  proofs for both regions;
- assess hard feasibility separately from cost and apply the fixed
  one-thread-per-output schedules;
- construct and verify a two-stage `KernelProgram` with one initialized
  cross-kernel intermediate, conservative non-aliasing `BufferPlan`, typed ABI
  roles, checked guard/launch expressions, numerical realization, target
  requirements, and one-way routing states; and
- refine both scheduled entries into verified structured kernel IR and produce
  stable explanations for every accepted or rejected fixed-profile condition.

The output is an in-memory compiler product: the verified two-stage
`KernelProgram`, both verified structured kernels, and its manifest-ready
artifact construction plan. Golden and negative tests must cover deterministic
output, complete semantic-result coverage, malformed references, invalid
access/schedule/program refinements, uninitialized or aliased buffers, resource
infeasibility, and stable diagnostics. Do not fuse pointwise work into `Sum`,
emit MSL, invoke `xcrun`, encode the final bundle, dispatch a device, or
introduce general alternative search, a calibrated cost model, or a public wire
format.

Before consequential `tiler-ir`, `tiler-compiler`, or `tiler-artifact` public
modules, traits, and call sites are hardened, present the bounded interface
draft to Tom. Internally stage the work as request/profile verification,
access/schedule/KIR lowering, then whole-program buffer/ABI/routing
construction; do not split by crate unless branch duration becomes unsafe. The
conservative prototype target profile is a named, versioned fixture whose
identity is refined by later artifact and live-device evidence rather than
silently replaced by ambient Metal facts.
