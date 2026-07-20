---
schema: "tiler-doc/v1"
id: "ADR-0065"
kind: "decision"
title: "Extract reference evaluation from the IR crate"
topics: ["rust", "reference", "dependencies", "semantics"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir"]
evidence: ["tiler.research.extensions.semantic-foundation-api-v2", "tiler.research.reference.normative-reference-slice"]
supersedes: ["ADR-0056"]
ticket: "prototype-reference-evaluator-crate"
---

# 0065: Extract reference evaluation from the IR crate

**Status:** accepted and implemented; supersedes ADR 0056 only for the
reusable-crate count and reference-evaluator placement

## Context

ADR 0056 selected four reusable crates before the reference evaluator existed.
The implementation now demonstrates that reference evaluation owns host tensor
storage, allocation, execution traversal, optional executable capabilities,
and conformance utilities. Keeping those in `tiler-ir` makes the foundational
representation crate also implement one consumer of that representation.

## Decision

Add a fifth reusable target-independent crate:

```text
tiler-reference -> tiler-ir
```

`tiler-ir` retains semantic representations, definitions, validation, and
canonical identity. `tiler-reference` owns host reference values, input
bindings, evaluator traversal, evaluation diagnostics, and the separately
frozen reference-capability registry.

Compiler, artifact, backend, and runtime production crates do not depend on
`tiler-reference`. Proof executables and tests may consume it explicitly.
Normative operation specifications remain authoritative; moving their
executable oracle does not transfer semantic ownership away from the IR
contract.

The frozen reference registry resolves the exact pair of semantic operation
key and ordered resolved operand/result signature. Its canonical provenance
also commits to the provider identity and capability revision. Standard and
external capabilities use the same transactional registration path; absence
is a typed capability error rather than evidence that the verified semantic
program is malformed.

## Consequences

- The crate graph mechanically enforces the intended producer/consumer
  direction.
- Multiple future reference engines do not enlarge `tiler-ir`.
- The prototype has one additional justified crate rather than speculative
  crates for every conceptual layer.
- Existing `tiler_ir::reference` paths are removed without deprecation because
  the workspace is unpublished and explicitly unstable.

## Alternatives considered

Keeping the evaluator as an IR module minimizes package count but preserves the
wrong dependency boundary. Moving it only into a proof executable prevents
reuse by optimizer, backend, and extension conformance tests.
