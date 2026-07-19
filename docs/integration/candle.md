# Candle integration

**Status:** proposed

The Candle adapter consumes versioned Tiler artifacts. It owns Candle storage,
layout, allocation, command-stream, and fallback concerns; it does not own
compiler optimization or MSL generation.

The frontend macro passes an `EmbeddedBundle` backed by static manifest and
metallib byte literals. The adapter never reads the expansion compiler cache or
compiles MSL at runtime; it loads/caches Metal libraries and pipelines by bundle
identity.

## Two-stage forward path

Fallback selection and artifact launch occur at different abstraction levels.

### Tensor-level preflight

Before applying a custom op, a frontend/runtime wrapper inspects Tensor-visible
device, dtype, shape, and layout facts; binds every semantic extent root from
input metadata, interface arguments, and admitted target-property providers;
then evaluates semantic requirements and available preflight guards. It chooses
either an ordered set of applicable compiled plan variants or the ordinary
Candle expression. This is where semantic fallback is safe and expressible.

The same bound semantic environment is passed to compiled and fallback paths.
Failure to bind a target property that affects output semantics is not a plan
miss: fallback is permitted only if it can realize that identical binding and
semantic result.

Guards requiring backend-only allocation facts are classified as launch-time
assertions. The integration should minimize these; failure after custom-op
selection normally returns an error rather than rebuilding a Tensor graph.

### Selected custom-op launch

For an already selected output-producing custom operation, the adapter:

1. converts Candle storage and `Layout` into runtime tensor-view descriptors;
2. constructs and validates the bound semantic extent environment;
3. computes and validates the output shape and semantic requirements;
4. validates launch-time guards and routes to a plan variant;
5. prepares all required per-device library/function/pipeline objects, trying a
   later preflight-valid plan only if preparation fails before device work;
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
- maximum reachable element under a strided access map;
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
try another preflight-valid compiled plan before allocation or encoding. Corrupt
artifacts, ABI mismatches, or failures after device work begins are errors; the
adapter does not risk executing fallback after partial work.

The Tensor-level wrapper retains enough information to execute the unfused
Candle expression when no generated variant applies. That fallback is valid
only when its numerical and autograd contract matches the requested semantics.

## Command-stream behavior

The adapter encodes into Candle's active command stream. It does not create a
private command buffer, commit, or call `wait_until_completed`. This preserves
ordering and overlap with surrounding Candle work.

Resource access modes come from the ABI so the encoder can declare read-only,
write-only, and read/write resources accurately.

## Dtypes and numerical contract

Storage dtype, accumulator dtype, and output dtype are distinct fields.
Unsupported dtypes fail a guard before pipeline binding. Feature-dependent
types such as BF16 also require a compatible target artifact and device.

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
- whether fallback was selected.
