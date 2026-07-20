---
schema: "tiler-doc/v1"
id: "tiler.research.runtime.semantic-validation"
kind: "research"
title: "Semantic validation enforcement"
topics: ["runtime", "validation", "semantics", "fallback"]
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.candle-integration"]
adopted_by: ["ADR-0033", "ADR-0051"]
reproduced_by: ["tiler.spike.runtime"]
ticket: "spike-runtime-semantic-validation-enforcement"
---

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

### Witness dependencies

A witness is evidence about a particular logical subject, not a reusable
boolean. Its dependency identity contains at least:

- the stable precondition/predicate and obligation identity;
- the logical tensor or component-set identity and exact logical view;
- value version or compiler-established immutability provenance;
- producer-completion and coherence dependencies needed to observe that value;
- the validation mechanism and its completion/publication dependency edges.

Reuse requires equality or a proof of refinement for all of those fields.
Storage pointer identity is not value identity, and validating a base allocation
does not automatically validate a differently mapped view. A dependent result
consumes the witness edge even when the selected enforcement later erases it.

### Three commit boundaries

The execution contract distinguishes boundaries that an undifferentiated
"dispatch started" flag cannot express:

1. `RoutingCommit`: a complete semantically equivalent variant and any fallback
   have been selected after preparation.
2. `EnforcementCommit`: execution of the selected residual validation mechanism
   begins. From here, semantic or runtime failure is returned; another variant
   or fallback is not executed.
3. `PublicationCommit`: completed validation has produced a successful witness
   and private logical results become externally observable.

A proof-elided obligation has no runtime `EnforcementCommit`. Host validation
commits before reading authoritative values; device pre-scan commits before its
validation dispatch; transactional validation commits before combined
validation/compute. `PublicationCommit` must follow completion observation for
every device-produced witness.

## Enforcement mechanisms

**Proof-elided.** Only compiler-owned proof evidence discharges the obligation.
A caller assertion is not a proof under strict semantics. Runtime validation,
error storage, and synchronization cost are zero.

**Host scan.** The host may validate only when it can observe the authoritative
logical value after all producer/coherence dependencies. It walks canonical
logical-view order and may stop at the first violation. Result computation does
not start until success. Device-private storage may make this infeasible or add
a transfer/map cost.

**Device pre-scan.** A validation-only dispatch scans the logical view and
reduces failures into a deterministic error record. The runtime completes and
observes that dispatch before normal result dispatch. This adds a full read
pass, a dispatch, and a host-visible completion boundary, but needs no full
private result.

**Transactional device validation.** Validation and computation share a
dispatch, but every result and dependent effect stays in a declared private
transaction closure. After successful completion and error observation,
publication is either ownership promotion or an explicit copy/dispatch; the
mode is part of the plan. Failure discards private results. Initial support is
out-of-place: mutation requires shadow state or an undo protocol and is a
separate capability. Invalid lanes do not cancel other parallel work, so the
cost model includes potentially complete wasted computation.

An error-as-data design can later enlarge the transaction closure by threading
the witness through device work. It does not remove the rule that neither data
nor effects may escape independently of the witness.

### Completion observation

"Waited" is not sufficient. A device enforcement provider must, in order:

1. wait for or otherwise establish terminal completion;
2. inspect terminal execution status and errors *after* completion;
3. establish host visibility/coherence of the error record;
4. validate its framing and schema;
5. reduce it to the deterministic semantic success or error result.

An execution failure takes precedence over interpreting an error record whose
producer did not complete successfully. The separate Candle integration spike
tests whether its adapter provides these steps; core requires the contract and
does not encode Candle behavior.

### Deterministic errors

The semantic diagnostic priority is a canonical total order over
`(logical_linear_index, stable_error_code, obligation_ordinal)`. The logical
index is row-major over the declared logical view, never a physical storage
offset or worker order. Parallel enforcement uses an associative, commutative,
and idempotent minimum reduction; first-writer-wins is invalid.

The artifact declares an `ErrorRecordSpec` with schema/version, state,
obligation identity, logical index, stable code, and bounded optional detail.
A backend may pack the priority key for an atomic implementation only when the
declared index/code widths prove it lossless. The portable record contract does
not depend on one `u64` layout.

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

Capability is runtime-physical metadata, separate from semantic identity:

- host enforcement declares observable memory domains, supported
  dtype/predicate evaluation, producer completion, and coherence;
- pre-scan declares validation-kernel coverage, deterministic reduction,
  host-visible error records, and post-completion error observation;
- transactional enforcement declares private allocation, maximum transaction
  scope, permitted effects, publication modes, and completion observation.

The selected mechanism and capability provider schema/revision participate in
physical plan and artifact identity. Missing capability may choose another
semantically identical prepared plan before `RoutingCommit`; it cannot weaken
the precondition.

Hard feasibility is checked before costing. Cost inputs include:

| Mechanism | Principal cost inputs |
| --- | --- |
| Proof-elided | compile-time proof/search only; zero runtime work |
| Host scan | logical elements/bytes, view-index cost, producer wait/coherence, mapping or transfer, CPU bandwidth, expected first-error position |
| Device pre-scan | full logical read, validation dispatch, error-record clear/readback, queue completion, lost overlap, then result work |
| Transactional | predicate plus result work, full private bytes and lifetime, atomic contention, completion, invalid-input wasted work, publication promotion or copy |

Invalid-input probability and error position can affect expected cost, but
never legality or the returned semantics.

## Executable spike and measurements

[`spikes/runtime/semantic_validation_enforcement.rs`](../../../spikes/runtime/semantic_validation_enforcement.rs)
models the three commit boundaries, exact witness reuse, deterministic parallel
error reduction, completion failure, private publication, and no-fallback
behavior. Eight tests cover equivalent valid results, worker-order-independent
errors, failed private-result discard, post-completion status precedence,
witness version/view mismatch, and structural accounting.

The optimized CPU model was run on an Apple M4 Max with 14 logical CPUs,
macOS 27.0, and rustc 1.97.0. Values are median microseconds over nine runs for
64 Ki elements and five runs for larger inputs:

| Elements | Proof | Host scan | Parallel pre-scan | Transactional |
| ---: | ---: | ---: | ---: | ---: |
| 65,536 | 5 | 30 | 72 | 65 |
| 1,048,576 | 62 | 413 | 170 | 132 |
| 8,388,608 | 518 | 3,297 | 917 | 586 |

These timings validate the model and harness, not Metal/CUDA performance. The
portable findings are the accounting ratios: proof performs one compute read;
host and pre-scan perform two reads; pre-scan has two modeled dispatches and
one completion observation; transactional performs one read/dispatch but writes
one full private result before zero-copy ownership publication. Real device
measurements must calibrate bandwidth, launch, synchronization, atomic,
allocation, overlap-loss, and publication-copy coefficients independently.

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

Focused source inspection and an executable transition test now confirm that
the local Candle 0.11.0 `Commands::ensure_completed` checks command-buffer
status before waiting and can return success without rechecking a buffer that
transitions from committed/scheduled to error during the wait. Tiler cannot use
that result as a synchronous validation-readback completion proof until Candle
observes post-wait `Completed` versus `Error` and propagates the latter. See the
[post-wait verification](candle-metal-post-wait-error-checking.md).

Primary Apple sources: [command-buffer completion](https://developer.apple.com/documentation/metal/mtlcommandbuffer/waituntilcompleted%28%29),
[command-buffer errors](https://developer.apple.com/documentation/metal/mtlcommandbuffer/error),
and [resource synchronization](https://developer.apple.com/documentation/metal/resource-synchronization).

## Traceability

The result is adopted by ADRs 0033 and 0051 and the runtime-facing contracts.
The [runtime spike](../../../spikes/runtime/README.md) exercises enforcement
transitions; target-specific device implementations remain future work.
