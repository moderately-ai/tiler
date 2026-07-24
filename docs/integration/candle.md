---
schema: "tiler-doc/v1"
id: "tiler.contract.candle-integration"
kind: "contract"
title: "Candle integration"
topics: ["integrations", "candle", "runtime", "metal"]
contract_status: "accepted"
implementation_status: "not-started"
evidence: ["tiler.research.runtime.execution-contract", "tiler.research.runtime.candle-post-wait", "tiler.research.runtime.semantic-validation", "tiler.research.apple-targets.numerical-behaviour"]
ticket: "synthesize-artifact-contracts"
---

# Candle integration

**Status:** accepted adapter contract; initial Candle API limits remain

## Ownership boundary

The Candle adapter consumes versioned Tiler artifacts. It owns Candle storage,
layout, allocation, command-stream, and fallback concerns; it does not own
compiler optimization or MSL generation.

The frontend macro passes an `EmbeddedBundle` backed by static manifest and
metallib byte literals. The adapter never reads the expansion compiler cache or
compiles MSL at runtime; it loads/caches Metal libraries and pipelines by bundle
identity.

The adapter implements the consumer-neutral
[runtime execution contract](../research/runtime/runtime-execution-contract.md).
It consumes a device-free validated envelope, binds it to one live
device/context, prepares every entry of one complete variant, and receives a
one-way `RoutingCommitted` authority before program allocation or encoding.

## Two-stage forward path

Fallback selection and artifact launch occur at different abstraction levels.

### Tensor-level preflight

Before applying a custom op, a frontend/runtime wrapper inspects Tensor-visible
device, dtype, shape, and layout facts; binds every semantic extent root from
input metadata, interface arguments, and admitted target-property providers;
then evaluates semantic requirements and available preflight guards. It chooses
either an ordered set of applicable compiled plan variants or the ordinary
Candle expression. This is where semantic fallback is safe and expressible.

Before calling `apply_op`, the wrapper asks the adapter to refine those variants
with live-device, library/function, prepared-pipeline, and route-sensitive
launch-preflight facts. It returns a `PreparedSelection` token keyed by the
bound semantic environment digest, live device identity, bundle and plan hash,
the complete ordered step/pipeline identities (each including entry point,
specialization, descriptor, archive/runtime mode), exact input-view/binding
fingerprint (storage identity/generation where available, dtype, shape, strides,
start/base offset, allocation length, accessible range, and required access),
and evaluated route-sensitive launch-fact digest, or a typed
capability miss. If every variant has a capability miss, the wrapper still owns
the Tensor expression and can choose ordinary Candle fallback. No output/
scratch allocation or encoding occurs before this selection.

The same bound semantic environment is passed to compiled and fallback paths.
Failure to bind a target property that affects output semantics is not a plan
miss: fallback is permitted only if it can realize that identical binding and
semantic result.

Conditions requiring actual plan-specific allocation facts are guaranteed by
the allocator contract or classified as post-`RoutingCommit` invariants. Their
failure returns an error rather than rebuilding a Tensor graph.

### Selected custom-op launch

For an already selected output-producing custom operation, the adapter:

1. converts Candle storage and `Layout` into runtime tensor-view descriptors;
2. constructs and validates the bound semantic extent environment;
3. computes and validates the output shape and semantic requirements;
4. validates every token field against the current inputs, device, plan,
   pipelines, and launch values, rejecting any stale/mismatched token before
   `RoutingCommit`, then consumes it without rerouting;
5. crosses `RoutingCommit` for that selected variant;
6. allocates output and declared temporary storage through the input
   `MetalDevice`/Candle allocator;
7. for each dependency-ordered step, binds allocation buffers and checked
   view-start metadata, packs scalars, evaluates dispatch, and encodes on
   Candle's current command encoder;
8. retains temporary storage through its last encoded GPU use;
9. returns `(MetalStorage, Shape)` without committing or synchronously waiting.

Output device matches the inputs, allocation arithmetic is checked, zero-size
behavior is explicit, and the plan fully initializes the one returned output.
Candle's current CustomOp return type does not represent multiple outputs.

Unary, binary, and ternary Candle custom-op traits may wrap a shared internal
launch object. The initial integration supports at most three independent
Tensor inputs. Larger fusion regions must be partitioned or require a future
Candle/generic packed-input extension; sharing launcher internals does not
change the public trait arity.

## Storage-layout contract

Candle storage is an allocation; `Layout` identifies the logical view within
it. The adapter must account for:

- rank and dimensions;
- element strides;
- start offset;
- dtype size and byte-offset conversion;
- contiguity class;
- maximum reachable element after composing logical access with the strided
  physical view;
- zero-sized views.

The initial fused variant requires contiguous inputs, but it must still
apply a nonzero contiguous start offset. Unsupported layouts fall back. Later
rank-specific affine-stride variants can pass dimensions, strides, and offsets
through metadata.

The adapter never uses full allocation length as logical tensor length and
never binds offset zero merely because it has the underlying buffer.

## Variant selection and fallback

The manifest's deterministic routing policy may select among:

```text
aligned vectorized contiguous variant
  -> scalar/tail-capable contiguous variant
  -> general affine-stride variant
```

If no artifact variant matches, the Tensor-level wrapper selects the existing
Candle operation pipeline outside the manifest/runtime launcher.

Failed preflight guards are normal and explainable. Pipeline preparation may
try another preflight-valid compiled plan before `RoutingCommit`. Only a typed
compatibility/capability rejection may route. Corrupt artifacts, schema or ABI
mismatches, dishonest capability providers, systemic runtime failures,
allocation failures, and all post-commit failures are errors; the adapter does
not mask them by trying another variant or risk fallback after partial work.

Library load, function lookup, and pipeline creation remain distinct stages.
A missing declared symbol is an artifact invariant, not an applicability miss.
A pipeline error permits another route only when a governed classifier proves a
typed capability miss before `RoutingCommit`; unknown/systemic errors fail
closed.

The Tensor-level wrapper retains enough information to execute the unfused
Candle expression when no generated variant applies. That fallback is valid
only when its numerical and autograd contract matches the requested semantics.

## Command-stream behavior

The adapter encodes into Candle's active command stream. It does not create a
private command buffer, commit, or call `wait_until_completed`. This preserves
ordering and overlap with surrounding Candle work.

Resource access modes come from the ABI so the encoder can declare read-only,
write-only, and read/write resources accurately.

Inputs, outputs, temporaries, metadata/argument storage, libraries, pipelines,
and any validation resources are retained through their exact final device use.
Encoding scope or a host reference count alone is not completion evidence.

An explicitly synchronous validation/readback path is an exception to the
ordinary asynchronous launch path. It must commit and wait for the exact
command buffer containing the validator and required copy/synchronization, then
observe a final `Completed` status before reading the CPU-visible validation
record. A final `Error` returns the command buffer's error and cannot select
fallback. The inspected Candle 0.11.0 `Commands::ensure_completed` does not
perform this post-wait terminal check, so that method is not sufficient until
the [verified gap](../research/runtime/candle-metal-post-wait-error-checking.md)
is fixed or the adapter supplies an equivalent checked boundary.

## Dtypes and numerical contract

Storage dtype, accumulator dtype, and output dtype are distinct fields.
Unsupported dtypes fail a guard before pipeline binding. Feature-dependent
types such as BF16 also require a compatible target artifact and device.

## Numerical scope across the Candle kernel boundary

**Fact — Tiler declares no Candle dependency, so this section describes an intended consumer rather than a resolved pin.** No manifest in this workspace names Candle: not the root `Cargo.toml`, not any of the six `crates/*/Cargo.toml`, not the two `prototypes/*/Cargo.toml`, and not any `spikes/**/Cargo.toml`. None of the nine checked-in `Cargo.lock` files contains a `candle` package. The adapter described above does not exist either — this contract's `implementation_status` is `not-started`, and the `implementation/candle` scope in `ticketsplease.toml` maps to a `crates/tiler-candle` that has not been admitted. The revision cited below was inspected directly, but it is an upstream revision of the consumer Tiler intends to support, not one Cargo resolves for it. [`repin-candle-numerical-scope-citation-at-adapter-admission`](../../tickets/repin-candle-numerical-scope-citation-at-adapter-admission.md) owns re-pinning it when that changes.

**Fact — the intended consumer compiles its own Metal kernels in the host process.** At `huggingface/candle` [`31f35b14`](https://github.com/huggingface/candle/blob/31f35b147389700ed2a178ee66a91c3cc25cc80d/candle-metal-kernels/src/kernel.rs), version 0.11.0 and the revision the rest of this corpus cites, `Kernels::load_library` at `candle-metal-kernels/src/kernel.rs:109` compiles each built-in kernel source through `new_library_with_source` at line 122 — that is, `newLibraryWithSource:options:error:` — and caches the resulting library by `Source`. [`MetalDevice::compile`](https://github.com/huggingface/candle/blob/31f35b147389700ed2a178ee66a91c3cc25cc80d/candle-core/src/metal_backend/device.rs) at `candle-core/src/metal_backend/device.rs:101` does the same at line 111 for a `ug`-generated kernel. Both reach the OS-resident runtime compiler that the Metal backend's [compiler-provenance section](../backends/metal.md#compiler-provenance-and-the-runtime-compiler) measures at a different build from the one an artifact's provenance names.

**Fact — the built-in kernels' math mode is chosen by the execution environment, not by any artifact.** `get_compile_options` at `candle-metal-kernels/src/kernel.rs:182` reads `CANDLE_METAL_ENABLE_FAST_MATH` with a default of `true` and, on macOS 15 or iOS 18 and later, sets `MTLMathMode::Fast` together with `MTLMathFloatingPointFunctions::Fast`; when that variable is falsy it sets `MTLMathMode::Relaxed` with `MTLMathFloatingPointFunctions::Precise`, and below those OS versions it sets the deprecated `setFastMathEnabled` instead. `MetalDevice::compile` passes `None` for its options and so takes `MTLCompileOptions` defaults, whose `mathFloatingPointFunctions` the [Apple GPU `f32` numerical behaviour](../research/apple-targets/numerical-behaviour.md) record documents as `Fast` from the macOS SDK 26.5 header. An environment variable and a runtime OS-version test therefore select a neighbouring kernel's math mode, and `load_library` evaluates both only on a cache miss, so the mode compiled into a cached library is the one that held at its first load in that process.

**Inference — a Tiler kernel and a Candle kernel in one command buffer differ on three independent axes.** They are produced by different compiler builds, under different math modes, fixed by different mechanisms at different times. Tiler's strict baseline for the qualified toolchain row is `-fmetal-math-mode=safe`, `-fmetal-math-fp32-functions=precise`, `-ffp-contract=off`, fixed at expansion time and carried in artifact identity; Candle's built-in default is the opposite corner of that same axis, fixed at first library load in the consumer's process. Nothing reconciles the two, and nothing needs to. They are separate compilations that happen to write to the same tensors.

**Fact — what a Tiler numerical claim covers.** A declared numerical realization, the toolchain provenance recorded with it, and [ADR 0076](../decisions/0076-declare-target-honourable-numerical-realizations.md)'s delivered-realization record are claims about the kernels Tiler emitted and compiled, and about nothing else. They do not extend to a Candle kernel that produced an input tensor, to a Candle kernel that consumes an output tensor, to a Candle kernel encoded into the same command buffer, or to any other Metal work in the process. This holds however the result is observed: a consumer reads one tensor out of one pipeline, but that tensor's value composes several numerical contracts and Tiler states one of them.

**Fact — this is not a defect in Candle and not a reason to distrust it.** Candle's kernels are Candle's contract, compiled by Candle's chosen compiler under Candle's chosen options, and nothing here says they are wrong, imprecise, or worse than an alternative. The point is narrower and structural: two correct numerical contracts do not compose into a third one automatically, and neither project's claim is evidence about the other's kernels. The [post-wait error-checking finding](../research/runtime/candle-metal-post-wait-error-checking.md) is a defect report against Candle; this section is not one.

**Inference — what a reference comparison over a mixed program compares.** A consumer who runs a Tiler-accelerated Candle program and diffs the end-to-end result against a CPU reference is measuring the composition, not Tiler's conformance. A divergence may originate in a Tiler kernel, in a neighbouring Candle kernel, or in the accumulation of both, and an end-to-end diff does not attribute it to any of the three. Attribution requires comparing at the boundary of the covered operations: the inputs and outputs of the fused custom op, against a reference evaluated for exactly those operations. ADR 0076's delivered-realization record is what tells such a comparison what to expect, and it describes only the covered operations.

**Proposal — the boundary is documented and reported, never checked at run time.** No adapter path detects, warns about, or rejects a mixed program, for two reasons. First, mixture is not an error condition but the only mode of use: the adapter is a Candle custom op, so every tensor reaching it was produced by Candle kernels and every output it returns is consumed by them. A predicate true on every reachable call carries no information, and rejecting on it would reject the product. Second, the condition is not observable where a check would have to sit. The adapter encodes into Candle's active command stream and does not own it, and because `load_library` caches by `Source`, reading `CANDLE_METAL_ENABLE_FAST_MATH` at adapter time would not establish what options an already-cached library was built with.

What is obligatory instead is that the claim carry its own scope. Wherever the adapter surfaces a numerical realization, a delivered-realization record, or a conformance claim — in explain output, in a diagnostic, or in a consumer-facing artifact record — it must also identify the operations that claim covers, so the scope cannot be read separately from the statement. The Diagnostics section below carries that obligation. This is a checkable property of the report, not of the program.

**Inference — a strict numerical contract removes the Candle fallback rather than weakening it.** Variant selection and fallback above already require that the unfused Candle expression match the requested semantics' numerical and autograd contract before it may be selected. With the compile-option fact above, a request whose realization Candle's own kernels do not deliver — a contraction-free or otherwise strict `f32` contract against a fast-math default — has no valid fallback, and the Tensor-level wrapper must fail closed naming the unmet realization rather than silently running the faster, differently rounded expression. That is the accepted rule applied, not a new one. Whether losing fallback is the right product behaviour for such a contract is an open decision owned by [`decide-strict-realization-fallback-availability`](../../tickets/decide-strict-realization-fallback-availability.md).

## Aliasing and mutation

The initial integration is out-of-place. In-place execution requires explicit
alias analysis proving that no future read observes overwritten data and that
the input/output index relationship is safe. It must not be used merely to fit
an existing custom-op hook.

## Autograd

A fused forward custom operation does not automatically provide gradients. For
the initial vertical slice, Tensor-level preflight bypasses fusion whenever the
operation must participate in tracked autograd, unless a concrete custom-op
backward implementation exists. A later adapter may carry a Rust backward
formula or separately compiled backward plan. Merely retaining a forward
fallback graph does not implement `CustomOp::bwd`.

Silently breaking autograd is not acceptable. Generated backward kernels are a
later capability, not a prerequisite for validating untracked forward
compilation.

## Diagnostics

Runtime errors and explain traces identify:

- semantic and scheduled kernel hashes;
- selected or rejected variant;
- failed guard and actual runtime value;
- artifact and target versions;
- evaluated binding offsets and launch geometry where safe;
- whether fallback was selected; and
- for any reported numerical realization, delivered-realization record, or
  conformance claim, the operations that claim covers.

The last item is not optional formatting. A realization reported without its
scope invites the misattribution the numerical-scope section above describes,
because the consumer sees one result and Tiler's claim covers part of it.

## Traceability

This adapter contract owns Candle-specific storage, preparation, encoding, and
fallback integration, and it owns what a consumer may conclude across the
Tiler/Candle kernel boundary. It does not own compiler planning or Metal
emission; the [Metal backend](../backends/metal.md) owns the compiler-provenance
facts the numerical-scope section builds on, and states there that the boundary
itself belongs here. The consumer-neutral execution evidence and accepted
routing decision are linked in frontmatter.
