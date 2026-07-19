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
- Which transcendental error metrics and operation subset are supported first?
- Which integer division/remainder and float-to-integer conversion families are
  supported first?
- Which built-in quantization scheme families and exact conversion contracts
  are enabled in the first product profile beyond the extensible affine model?

## Semantic graph and operation extensions

- Which mandatory Rust traits and canonical data structures form the first
  public experimental operation-definition API?
- How are external providers supplied to ordinary compiler sessions, and which
  discovery/declarative mechanism, if any, can make them visible to a separately
  compiled proc macro?
- Does one semantic authority exclusively own each `OpKey`, and how are
  additional lowering/schedule/cost providers selected without precedence
  ambiguity?
- What exact provider revision/fingerprint contract covers output-affecting
  callback changes?
- What bounded canonical attribute value model and schema/version policy is
  used?
- Which callback thread-safety, panic-containment, trust, and determinism
  obligations are public API?
- Which transactional rewrite API, recursion declarations, cycle detection,
  and application budgets are required?
- Are several named graph results and first-class multi-result operations both
  required in the first executable slice?
- Are program-result names semantic identity, diagnostic identity, or both?
- Which higher-level operations must decompose into a canonical core, and which
  may supply direct iteration/access lowering?
- How are extension semantic, verifier, rewrite, and lowering implementation
  changes fingerprinted for compiler and artifact identity?
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

- Is the canonical index type `u64`, `i64`, or target-dependent?
- When may a proven `u32` fast variant be emitted?
- Which extents are specialized versus passed at runtime?
- Are composed-axis factors required to be static initially?
- How are ellipses represented across possible runtime ranks?
- Is rank permanently static in the semantic graph, or when would dynamic-rank
  values become representable?
- When are data-dependent output shapes or device-produced indirect dispatch
  dimensions introduced, and what host/device `ShapeProgram` contract would
  they require?
- Are negative strides permanently out of scope or merely deferred?
- Which bounds are statically proven versus guarded at runtime?

## Fusion, planning, and scheduling

- What bounded search representation is simplest for the first release, and do
  exhaustive tiny-graph comparisons justify introducing a memo?
- When may shared work be duplicated?
- What source-size, live-value, or resource threshold caps a fusion region?
- Which reductions can coexist in one kernel?
- When are multi-output kernels introduced?
- Which facts must an optimized opaque call expose as boundary contracts,
  target requirements, resource envelopes, and costs?
- Which cost terms are hard constraints versus calibrated estimates?
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

- What is the minimal target-neutral artifact envelope, and which ABI,
  payload, reflection, and delivery fields belong only to backend profiles such
  as Metal?
- Does one `CompilationRequest` always compile one semantic graph for one or
  several target profiles, and how are multi-target outputs grouped?
- What serialization format is used for manifests and canonical IR?
- What direct-embedding size budget keeps rustc memory and expansion acceptable?
- Does the linker merge identical byte-string literals reliably enough to use,
  or is an explicit content-named section strategy eventually needed?
- What is the default compiler-cache location and cleanup/size policy?
- What minimum Rust version and lock implementation are supported, and how is
  automatic OS-lock release after process death tested?
- Which exact Metal toolchain facts enter the cache key?
- How should cache deletion interact with rustc incremental macro expansion?
- Does rust-analyzer need an analysis stub after cold/warm measurements?
- Which Apple toolchain details define reproducibility boundaries?
- When are macOS, iOS-device, and iOS-simulator bundles generated together?
- Can target discovery improve without relying on unstable proc-macro APIs?
- How is unsupported Apple cross-compilation diagnosed?
- When, if ever, does the serialized IR become a public compatibility promise?

## Candle runtime

- Which fusion cases require more than the initial three CustomOp inputs?
- Which general affine-strided layouts follow the initial contiguous-only path?
- Is fallback retained as a closure, a semantic plan, or reconstructed calls?
- Which tracked/autograd cases can eventually avoid the initial fusion bypass?
- Are pipeline objects cached in a Candle-specific or generic Metal runtime
  crate?
- What policy applies if pipeline construction fails after guards succeed?

## Project packaging

- Which component boundaries become separate crates in the first workspace?
- Which parts of the public experimental semantic graph are data APIs versus
  registered operation capability traits?
- Which APIs between the frontend proc macro and `tiler-metal-aot` remain
  private while artifact formats evolve?
- What minimum supported Rust version and Apple deployment targets apply?
