# Semantic validation enforcement

**Status:** researched architecture supporting ADR 0033

## Problem

Some semantic operations are valid only when runtime tensor values satisfy a
predicate. Strict affine quantization, for example, rejects NaN. Shape/layout
guards known to the host are cheap, but proving a predicate over device data
may require a complete scan, synchronization, or transactional result.

The semantic contract must not prescribe one enforcement algorithm, and a
physical optimizer must not reinterpret invalid semantic input as an
inapplicable fast path.

## Precedents

MLIR Shape separates constraint witnesses from the operations that assume them;
unresolved constraints can lower to runtime assertion logic. MLIR runtime
verification generates explicit checks with effects. JAX `checkify` instead
functionalizes errors into data threaded through compiled computation, but
observing/throwing the error introduces a host boundary and missed observation
can lose the error. TensorFlow `CheckNumerics` provides a concrete GPU pre-scan:
it scans, copies a small indicator to the host, and completes only after the
check. CUDA device-side assertions show why a fused flag is not fail-before-
compute: other threads continue and output must be treated as invalid.

Primary sources: [MLIR Shape dialect](https://mlir.llvm.org/docs/Dialects/ShapeDialect/),
[MLIR runtime verification](https://mlir.llvm.org/docs/Passes/#generate-runtime-verification),
[JAX checkify](https://docs.jax.dev/en/latest/debugging/checkify_guide.html),
[TensorFlow CheckNumerics](https://www.tensorflow.org/api_docs/python/tf/debugging/check_numerics),
and [PyTorch CUDA device assertions](https://github.com/pytorch/pytorch/blob/main/c10/cuda/CUDADeviceAssertion.h).

## Architecture

A semantic operation declares a typed `SemanticPrecondition`. Verification
either proves it or emits a residual validation obligation. Consumers depend on
a conceptual validation witness; proven witnesses erase before execution.

The physical planner may realize an unresolved obligation through:

- a host-known check;
- a device pre-scan followed by completion observation;
- fused detection writing private output plus an error record, with publication
  only after successful validation;
- a future error-as-data transaction whose outputs cannot escape without the
  error dependency;
- rejection when the runtime profile cannot provide the required observability.

These alternatives have different cost, synchronization, temporary-storage,
and error-latency properties but implement the same semantic identity. The
selected `EnforcementPlan` is recorded in plan/explain/artifact identity.

An explicitly trusted assumption is not an enforcement plan for strict
semantics. It is a separate versioned policy with its own invalid-input
contract and is deferred.

## Publication and failure rules

- Static contradiction rejects compilation; a static proof removes enforcement.
- Semantic validation failure returns the operation's invalid-input error. It
  never selects another numerical mapping or plan.
- Plan selection, pipeline preparation, and every fallback decision complete
  before device enforcement begins.
- A fused validator writes only private output/scratch. The result and dependent
  public work remain unpublished until the validation witness succeeds.
- Once device validation or transactional work begins, later implementation or
  device failure is returned; ordinary fallback does not run.
- A runtime may support operation-local transactions first and larger pure
  region transactions later. The boundary is explicit.
- Validation scans the logical view, not padding, unreachable allocation bytes,
  or unused packed bits.

Error diagnostics use deterministic priority, such as the smallest logical
index plus stable error code, rather than schedule-dependent first-writer order.

## Cost and capability

Validation is a physical computation with traffic, dispatch, synchronization,
temporary, and compilation costs. The planner considers proof first and may
choose among supported enforcement alternatives. A runtime profile advertises
which observability and transaction mechanisms it implements.

Small component tensors such as scales and zero points may be cheap to validate
on the host or with a pre-scan. Full data predicates can dominate a
bandwidth-bound operation. That cost can make a strict operation unsupported in
a narrow runtime profile, but it cannot weaken its semantics.

Validation results may be reused only with sound immutability/version
provenance. Storage pointer identity alone is insufficient for mutable data.

## Metal/Candle evidence

Metal exposes completion handlers, status/error, and blocking completion
observation. A device flag in shared memory still requires synchronization
before the host can return a semantic error; private memory requires a copy to a
CPU-visible resource. A later dispatch may read a flag and suppress its writes,
but already encoded direct dispatches are not retroactively canceled.

The current Candle Metal adapter returns tensors after encoding without an
operation-level fallible completion boundary. A validator-plus-readback or
transactional result therefore needs explicit integration work. This is a
runtime-profile feasibility issue, not a core semantic restriction.

Local inspection also found that Candle's current `Commands::ensure_completed`
checks command-buffer status before waiting and does not appear to recheck
status/error after a committed or scheduled buffer completes. This requires a
focused upstream/source validation before Tiler relies on the path.

Primary Apple sources: [command-buffer completion](https://developer.apple.com/documentation/metal/mtlcommandbuffer/waituntilcompleted%28%29),
[command-buffer errors](https://developer.apple.com/documentation/metal/mtlcommandbuffer/error),
and [resource synchronization](https://developer.apple.com/documentation/metal/resource-synchronization).
