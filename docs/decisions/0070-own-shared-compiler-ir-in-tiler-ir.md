---
schema: "tiler-doc/v1"
id: "ADR-0070"
kind: "decision"
title: "Own shared target-neutral compiler IR in tiler-ir"
topics: ["architecture", "ir", "rust", "dependencies"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.artifact-abi"]
evidence: ["tiler.research.program-planning.abi-expression-ownership", "tiler.research.kernel-ir.structured-kernel-ir-verifier"]
ticket: "prototype-shared-compiler-ir-ownership"
related: ["ADR-0071"]
---

# 0070: Own shared target-neutral compiler IR in tiler-ir

**Status:** accepted

## Context

The first target-neutral proof placed its scheduled-region, structured-kernel,
executable-program, and artifact-construction sketches in private test-only
`tiler-compiler` modules. That proved a bounded path, but neither a backend nor
an artifact consumer can depend on those representations without importing
optimizer internals. Moving the proof structs verbatim would instead publish
fixed serial-Sum cardinalities, `u8` arena positions, proof-specific kernel
bodies, and duplicated program/artifact authority.

The prototype also retained an unused `tiler-compiler -> tiler-artifact` Cargo
edge. Artifact construction follows compilation, so that edge points from a
producer toward a downstream encoding consumer and prevents the intended
independent relationship through shared IR.

## Decision

`tiler-ir` owns the experimental target-neutral representation and verifier
authority shared across compiler passes, backends, artifact codecs, and future
third-party plan producers:

- `index`: semantic-region references, symbolic iteration/scalar expressions,
  access relations, and verified index regions;
- `schedule`: target-neutral schedules, launch/resource requirements, and
  verified scheduled regions;
- `kernel`: structured kernel IR and schedule-refinement verification; and
- `program`: `AbiExpr`, pure checked evaluation, stage and buffer plans,
  verified kernel programs, and verified program portfolios.

Compiler-owned region candidates, alternatives, search state, target
feasibility assessment, costing, explanations, and selection evidence remain
in `tiler-compiler`. `tiler-artifact` owns wire encoding, compatibility,
runtime fact binding, failure classification, and backend-payload mappings; it
does not own a second editable program model. `tiler-metal` consumes verified
IR and artifact mappings without reconstructing semantic graph patterns.

The dependency direction is:

```text
tiler-ir        -> []
tiler-reference -> [tiler-ir]
tiler-compiler  -> [tiler-ir]
tiler-artifact  -> [tiler-ir]
tiler-metal     -> [tiler-ir, tiler-artifact]
```

The proof executable composes compiler, backend, and artifact crates. This
decision supersedes only ADR 0056's retained compiler-to-artifact edge; ADR
0065's reference-evaluator extraction remains unchanged.

Verified scheduled regions lower to verified structured kernels. Complete
kernel-program stages reference those verified implementations. A
compiler-private object that precedes structured lowering is a planning draft,
not a `KernelProgram`.

## Consequences

- Runtime and artifact code can validate executable programs without linking
  optimizer internals.
- Compiler strategies may evolve without becoming artifact schemas.
- IR extraction proceeds in dependency order; private proof-specific structs
  remain private until replaced by their generic verified representation.
- An empty module or public type alias is not implemented support. Each layer
  becomes usable only with its checked construction and authoritative verifier.

## Alternatives considered

Compiler ownership forces backends and runtimes to depend on optimizer
internals. Artifact ownership makes compilation depend on a downstream codec
or creates a cycle. A crate per IR level adds packaging boundaries before an
independent build or reuse requirement exists.
