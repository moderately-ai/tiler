# 0039: Make integer overflow explicit in operation identity

**Status:** accepted

## Context

Fixed-width integer addition, subtraction, and multiplication can wrap,
saturate, report overflow, widen, or require that overflow never occur. These
contracts permit different rewrites and can produce observably different
results. Several tensor systems leave the choice underspecified, while compiler
IRs and programming languages demonstrate that all of the variants are useful.

Tiler cannot inherit Rust build-profile checks, backend-language behavior, or
LLVM poison without making one logical graph target- or build-dependent.

## Decision

Every canonical fixed-width integer arithmetic operation has an explicit,
versioned overflow semantic identity. The initial built-in families include:

- wrapping add, subtract, and multiply, evaluated modulo `2^N`;
- saturating add, subtract, and multiply, with bounds determined by the
  resolved signed or unsigned dtype;
- checked add, subtract, and multiply, producing the fixed-width result and an
  overflow predicate as explicit results; and
- widening add, subtract, and multiply, whose resolved result dtype represents
  the required wider mathematical range.

These are specialized semantic operations, not one generic integer operation
whose meaning comes from an ambient compiler setting. They may share traits,
verification, reference-evaluation code, and lowering machinery. A frontend
may provide an ergonomic `add` default that resolves to wrapping arithmetic,
but canonical admission records the selected family explicitly.

The list is not a sealed public enum or a claim that all variants are
implemented initially. The public operation-extension mechanism may introduce
additional versioned families with complete semantics and capabilities.
Existing family identities never change meaning when a new family is added.

Required-no-overflow is represented by a semantic precondition and proof or
runtime-validation obligation under ADRs 0021 and 0033. It is never silent
undefined behavior or poison. Whether it is exposed as a distinct ergonomic
operation, a verified refinement, or another API form remains evolvable. A
lowering may attach target `nsw`/`nuw`-like facts only after discharging the
corresponding proof obligation.

Overflow family participates in operation identity, reference evaluation,
rewrite preconditions, explain output, cost and feasibility analysis, artifact
identity, and conformance tests. Integer reduction combines values using an
explicit overflow family as well; accumulator dtype alone is insufficient.

## Consequences

- The same graph has stable overflow behavior across CPU/GPU and debug/release
  execution.
- Modular algebraic rewrites do not leak into saturating or checked operations.
- Checked arithmetic naturally exercises first-class multi-result operations.
- Widening remains explicit in the resolved signature and cannot be confused
  with saturation or overflow reporting.
- Sub-byte and future integer widths reuse the same families when their
  capability tables admit them.
- Importers of an underspecified source operation must select a declared import
  profile or reject it; Tiler does not guess.

## Alternatives considered

One generic `Add` with backend-defined overflow is compact but not portable.
Making wrapping implicit only in canonical IR would be deterministic, but would
hide a rewrite-critical choice and make other families look exceptional.
LLVM-style poison enables aggressive optimization but violates Tiler's
fail-fast, never-silently-wrong contract unless a proof has already established
the no-overflow precondition. Sealing a fixed Rust enum now would unnecessarily
constrain future operation families.
