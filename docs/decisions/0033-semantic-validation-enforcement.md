---
schema: "tiler-doc/v1"
id: "ADR-0033"
kind: "decision"
title: "Separate semantic validation from physical enforcement"
topics: ["numerics","validation","runtime"]
catalog_group: "dtypes-quantization"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics","tiler.contract.correctness-and-testing"]
evidence: ["tiler.research.numerics.affine-quantization-semantics"]
ticket: "define-initial-affine-quantization-semantics"
---

# 0033: Separate semantic validation from physical enforcement

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md) and [Correctness and testing](../correctness-and-testing.md).
- **Evidence:** [affine quantization semantics](../research/numerics/affine-quantization-semantics.md).
- **Work record:** [define-initial-affine-quantization-semantics](../../tickets/define-initial-affine-quantization-semantics.md).


## Context

Some strict semantic operations require predicates over runtime tensor values.
For example, strict affine quantization rejects NaN. Enforcing such a predicate
may require no work when compiler-proven, a host check, a device scan, or fused
detection whose private result is published only after success.

These mechanisms have different traffic, synchronization, temporary-storage,
and error-latency costs. Baking one mechanism into semantic IR would make
runtime capabilities part of program meaning. Treating the predicate as an
ordinary plan guard would instead allow invalid input to select a different
meaning.

## Decision

A semantic operation declares typed `SemanticPrecondition`s. Verification
either proves each precondition or emits a residual validation obligation.
Dependent semantic results conceptually require its validation witness.
The witness identifies the predicate/obligation, logical subject and exact
view, value version or immutability provenance, and required producer/coherence
dependencies. Pointer identity alone cannot justify reuse.

The physical plan selects an `EnforcementPlan` for every residual obligation
from mechanisms supported by its runtime profile, including:

- host-known validation;
- device pre-scan plus completion observation;
- fused detection into an error record with private result publication only
  after successful validation;
- a future error-as-data transaction whose result cannot escape independently;
- explicit unsupported classification.

Static proof removes the enforcement. The selected mechanism, error-record and
temporary roles, witness dependencies, observability point, transaction scope,
and publication boundary participate in plan, explanation, artifact, and cost
identity, but do not change semantic identity.

A failed semantic validation returns the operation's invalid-input error. It is
never an applicability miss and never authorizes a different semantic mapping.
Plan selection, pipeline preparation, and fallback selection complete before
device enforcement begins. Once device validation or transactional work begins,
ordinary fallback does not run. Private incomplete output may be discarded; no
logical result or dependent public work may escape before its witness succeeds.

Execution has three explicit boundaries. `RoutingCommit` fixes the prepared
variant and fallback. `EnforcementCommit` begins unresolved validation and
closes all alternate execution. `PublicationCommit` follows successful
completion observation and exposes the logical result. Proof-elided validation
has no runtime enforcement boundary. Host scans precede result work; device
pre-scans precede the result dispatch; transactional validation keeps the full
dependent effect closure private until publication.

Device completion observation means terminal completion, a post-completion
status/error check, error-record visibility/coherence, record validation, and
only then semantic interpretation. A failed producer cannot yield a trusted
semantic record. Initial transactions are out-of-place; mutation requires a
separate shadow/undo capability.

Validation covers the logical view, excluding padding, unused packed bits, and
unreachable allocation bytes. Diagnostics use deterministic logical-index and
error-code priority rather than schedule-dependent first-writer order. The
portable order is `(logical_linear_index, stable_error_code,
obligation_ordinal)` and parallel implementations must implement its minimum
without lossy packing.

An explicitly trusted assumption is a separate future semantic policy with its
own invalid-input behavior. It is not an enforcement plan for strict semantics.

## Consequences

- Compiler proofs eliminate validation cost without changing the graph.
- Runtimes can choose scans or transactions according to capability and cost.
- A narrow runtime may reject an unprovable strict operation without weakening
  its semantics or constraining compiler core.
- Transactional GPU validation requires private outputs, explicit completion,
  and publication machinery.
- Validation results can be reused only with sound immutability/version
  provenance.
- Runtime integrations need conformance tests for error timing, discarded
  output, dependent-work suppression, and no fallback after enforcement starts.
- Device pre-scan necessarily adds a read pass and completion boundary;
  transactional enforcement trades those for private result storage and
  potentially complete wasted compute on invalid input.

## Alternatives considered

Always pre-scanning is simple but can double traffic and force synchronization.
Always fusing a flag cannot report failure before computation and is unsafe when
output escapes. Treating validation as a plan guard confuses invalid input with
optimization applicability. Trusting the caller avoids cost but permits silent
semantic violation and therefore requires a distinct policy.
