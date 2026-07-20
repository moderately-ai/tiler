---
schema: "tiler-doc/v1"
id: "ADR-0031"
kind: "decision"
title: "Reject NaN in strict affine quantization"
topics: ["numerics","quantization","nan"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.affine-quantization-semantics"]
ticket: "define-initial-affine-quantization-semantics"
---

# 0031: Reject NaN in strict affine quantization

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [affine quantization semantics](../research/numerics/affine-quantization-semantics.md).
- **Work record:** [define-initial-affine-quantization-semantics](../../tickets/define-initial-affine-quantization-semantics.md).


## Context

An ordinary affine integer code range has no intrinsic NaN representation.
Existing tensor specifications do not establish one portable result for
quantizing a NaN. Possible behaviors include rejecting the input, mapping it to
the zero point, mapping it to an endpoint, or using a scheme-defined reserved
code. These choices are observably different.

Silently selecting a mapping in backend lowering would violate Tiler's typed
conversion contract. Treating every mapping as the same strict operation would
also make reference evaluation and fallback disagree.

## Decision

The initial strict affine `Quantize` conversion rejects every NaN input as an
invalid semantic runtime value. It does not map NaN to the zero point, an
endpoint, or an arbitrary code.

NaN behavior is a required field of the quantization conversion family.
Alternative behaviors, such as `NaNToZeroPoint` or a scheme-defined reserved
code, are separate explicitly selected typed families. They participate in
semantic graph, explanation, plan, artifact, and cache identity. A backend
cannot substitute one behavior for another.

For statically known constants, NaN rejection occurs during graph verification.
For dynamic inputs, NaN absence must be compiler-proven or runtime-validated
before the strict conversion successfully commits an observable result. A
failed validation is a semantic input error, not a physical-plan applicability
miss. It does not authorize fallback to another NaN mapping.

The runtime may choose a validated implementation strategy consistent with the
partial-execution contract, including a prevalidation pass or a transactional
conversion that withholds the result until validation succeeds. That physical
choice cannot weaken the semantic rejection rule.

## Consequences

- Strict quantization never silently turns exceptional data into numerical
  zero or an endpoint.
- Dynamic unproven inputs may require validation work.
- Frontends importing a different NaN convention must select a distinct
  conversion family explicitly.
- Reference evaluation and backend conformance include qNaN and sNaN rejection
  vectors.
- Fallback can change implementation but not exceptional-value meaning.

## Alternatives considered

Mapping NaN to the zero point is deterministic and inexpensive but silently
changes an exceptional value into valid numerical zero. Endpoint mapping is
equally arbitrary for a full affine code range. A reserved code is appropriate
only for a scheme that defines one and therefore belongs to that scheme's
explicit contract rather than generic strict affine quantization.
