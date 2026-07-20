# Open design questions

These questions are intentionally unresolved. Decisions should move into an
ADR when evidence is sufficient.

## Semantic and numerical policy

- Which ergonomic numerical-policy presets are exposed, and to which canonical
  per-operation ceilings and restrictions do they expand?
- Which built-in operations declare reassociation and commutativity
  capabilities under each resolved dtype/signature?
- After taxonomy review, which dtypes are recognized, representable,
  reference-evaluable, optimizable per operation, backend-realizable, and
  enabled in the first product profile?
- Which transcendental operation/dtype/contract tuples are enabled in the
  first vertically supported product profile?
- Which float-to-integer family, rounding, source-dtype, and destination-dtype
  tuples are enabled in the first product profile?
- Which built-in quantization scheme families and exact conversion contracts
  are enabled in the first product profile beyond the extensible affine model?

## Semantic graph and operation extensions

- What bounded canonical attribute value model and schema/version policy is
  used?
- Which transactional rewrite API, recursion declarations, cycle detection,
  and application budgets are required?
- Are several named graph results and first-class multi-result operations both
  required in the first executable slice?
- Which higher-level operations must decompose into a canonical core, and which
  may supply direct iteration/access lowering?
- May unknown operation keys round-trip through tooling even though they cannot
  be compiled or optimized?
- Which explicit effect/resource token model would be required before stateful,
  mutating, or hidden-random operations enter the semantic graph?
- Will a future `SemanticModule` add named graph functions, calls, recursion,
  and structured control-flow regions, or remain explicitly out of scope?
- Is differentiation a future optional operation capability, a frontend-owned
  transformation, or a separate compiler layer producing a backward semantic
  graph?

## Shapes and indexing

- Which extents are specialized versus passed at runtime?
- Are composed-axis factors required to be static initially?
- How are ellipses represented across possible runtime ranks?
- Is rank permanently static in the semantic graph, or when would dynamic-rank
  values become representable?
- When are data-dependent output shapes or device-produced indirect dispatch
  dimensions introduced, and what host/device `ShapeProgram` contract would
  they require?
- Which finite piecewise tensor-access maps justify extending the initial
  total affine/quasi-affine/semi-affine language?
- Which indirect gather/scatter relation and validation contract should admit
  tensor-data-derived indices?
- When does signed reachable-range analysis and runtime support justify
  enabling the currently deferred negative-stride ABI profile?

## Fusion, planning, and scheduling

- What bounded search representation is simplest for the first release, and do
  exhaustive tiny-graph comparisons justify introducing a memo?
- When may shared work be duplicated?
- What source-size, live-value, or resource threshold caps a fusion region?
- Which reductions can coexist in one kernel?
- When are multi-output kernels introduced?
- Which facts must an optimized opaque call expose as boundary contracts,
  target requirements, resource envelopes, and costs?
- Which governed capability keys and multivariate feasibility rules form the
  first Metal profile, and which remain backend extensions?
- What exact compatibility/versioning contract lets a declared profile cover
  several device families without overstating their common guarantee?
- Which runtime query phases and provider authorities are enabled in the first
  executable profile?
- Which compiler or prepared-kernel reports are authoritative enough to
  promote an estimate into a hard resource fact?
- What execution/threading and fixed/scalable vector contract defines the first
  future CPU profile?
- What normalized coordinate-map and loop/tile representation should
  `KernelSchedule` use?
- Which schedule transforms are stable enough for replayable explain traces?
- Which schedule fields are authoritative, derived-and-checked, or explain-only
  to avoid two independently editable truths?
- When does buffer assignment progress beyond one allocation per output and
  temporary to liveness-based reuse, suballocation, or in-place execution?
- What explicit placement/transfer plan would be needed for multi-device,
  sharded, distributed, or multi-queue programs?

## Artifacts and macro expansion

- Does one `CompilationRequest` always compile one semantic graph for one or
  several target profiles, and how are multi-target outputs grouped?
- What serialization format is used for manifests and canonical IR?
- Which additional Rust/LLVM/linker platforms must repeat the measured
  embedding matrix before changing the initial 1 MiB invocation and
  32-invocation/3.2 MiB package gates?
- What is the default compiler-cache location and cleanup/size policy?
- Does the workspace set MSRV 1.89 for standard-library file locking or audit an
  older-compatible lock adapter?
- What measured rust-analyzer cold/warm cost, if any, justifies a future stable
  optimization that preserves identical expansion semantics?
- What cross-machine and cross-toolchain evidence qualifies an Apple toolchain
  row beyond the measured same-host boundary?
- Which ergonomic explicit artifact-family profiles should frontends expose?
- How is unsupported Apple cross-compilation diagnosed?
- Which measured startup or size threshold would justify `MTLBinaryArchive`,
  offline pipeline binaries, or dynamic Metal libraries?
- When, if ever, does the serialized IR become a public compatibility promise?

## Structured kernel lowering

- What minimal conservative uniformity analysis is sufficient for the first
  workgroup reductions, and which schedule-derived uniformity proofs may it
  consume?
- When do asynchronous copies or split-phase barriers justify dependence tokens
  and a partial-order extension to the initial structured/phase model?
- Which target-specific operations require a later target-lowering IR rather
  than governed common-kernel operations?
- Which demonstrated workloads justify general CFGs, unrestricted pointers,
  calls, or richer aliasing beyond bounded structured tensor kernels?

## Candle runtime

- Which fusion cases require more than the initial three CustomOp inputs?
- Which general affine-strided layouts follow the initial contiguous-only path?
- Is fallback retained as a closure, a semantic plan, or reconstructed calls?
- Which tracked/autograd cases can eventually avoid the initial fusion bypass?
- Are pipeline objects cached in a Candle-specific or generic Metal runtime
  crate?

## Project packaging

- Which component boundaries become separate crates in the first workspace?
- Which parts of the public experimental semantic graph are data APIs versus
  registered operation capability traits?
- Which APIs between the frontend proc macro and `tiler-metal-aot` remain
  private while artifact formats evolve?
- What minimum supported Rust version and Apple deployment targets apply?
