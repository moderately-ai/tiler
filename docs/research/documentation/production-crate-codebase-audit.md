---
schema: "tiler-doc/v1"
id: "tiler.research.documentation.production-crate-codebase-audit"
kind: "research"
title: "Production crate architecture and maintainability audit"
topics: ["architecture", "maintainability", "rust", "progressive-disclosure"]
catalog_group: "documentation-governance"
research_status: "complete"
disposition: "pending"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.architecture"]
---

# Production crate architecture and maintainability audit

## Question

Which parts of the production Rust crates should be improved, refactored, or
split so that the repository remains generally flat while readers encounter
detail progressively?

## Scope and snapshot

This is a static architecture and maintainability audit of `crates/*` only.
It deliberately excludes `prototypes/`, `spikes/`, generated build output, and
the suitability of prototype code for promotion.

The detailed review was performed on the repository state leading to
`d43dca054178f3343675826543a0f0e67ea0ec49` and rechecked on
`b428e228890c5a64bb18e131e8a1cc68fbe68ed5` on 2026-07-27. The latter snapshot
contained 120,252 lines in Rust source files under production crates, 929 Rust
test functions, and 40 source files with an inline `mod tests` module. Those
counts describe size and concentration; they are not quality metrics.

No build, test, benchmark, or platform measurement is evidence for this report.
The review inspected declarations, construction sites, call sites, tests,
dependency edges, accepted ADRs, and normative contracts. Runtime performance
and unexecuted failure paths remain measurement boundaries.

## Overall conclusion

The crate graph is sound. The main concepts are separated at durable ownership
boundaries: semantic and program IR, compiler planning, artifacts, cache,
runtime, reference evaluation, Metal lowering, and offline Metal compilation.
The repository should keep these crate boundaries and its shallow top-level
shape.

The primary maintainability problem is local concentration inside a small
number of files. Several files combine the public entrypoint, validation,
algorithms, diagnostics, conformance machinery, and large test suites. The
highest-value restructuring is therefore one additional level beneath those
existing modules, not new crates and not a deep directory tree.

## Findings requiring correctness work

### Runtime commit authority is weaker than the accepted contract

**Fact.** The runtime path separates preflight from committed execution, but
the committed operation consumes a preflight result rather than a one-shot
authority that makes reuse impossible by construction. The decoded program is
also clonable, and preflight is callable through a shared reference.

**Inference.** The API documents the one-way routing boundary required by
[ADR 0051](../../decisions/0051-make-runtime-routing-commit-one-way.md), but its
types do not fully enforce that a successful preflight can authorize exactly
one committed attempt. Correct adapters can obey the contract; accidental
retry or fallback remains easier to express than the architecture intends.

**Proposal.** Introduce a consumed execution authority only when the public
runtime boundary is reviewed. Keep device objects and consumer fallback policy
outside `tiler-runtime`.

### Cache publication outcomes can obscure successful publication

**Fact.** The expansion-cache protocol can finish the immutable rename and then
encounter a later observation failure. Some such paths return `Uncached` and
retain a `publication_refusal`, even though the entry may already have been
published successfully.

**Inference.** A caller can receive an outcome whose name and explanation
describe failed publication while another process can observe the published
entry. That weakens explainability and makes accounting unreliable.

**Proposal.** Model “published, later observation unavailable” separately from
“publication refused.” Once the atomic publication point succeeds, later
diagnostics must not rewrite that historical fact.

### Offline tool identity and tool execution are not one operation

**Fact.** Metal AOT preflight resolves absolute `metal` and `metallib` paths and
records those identities. The compilation path still invokes tools through
`xcrun` by their bare names.

**Inference.** The bytes can be produced by a different executable than the one
whose resolved path was recorded if tool selection changes between preflight
and execution.

**Proposal.** Execute the exact resolved tools whose identities enter
provenance, or redefine and record the authoritative `xcrun` selection input.
Do not preserve two independently mutable notions of tool identity.

### Stage coverage omits index-refinement identity

**Fact.** Kernel-program stage coverage binds scheduled regions and their
realization, but the coverage identity does not carry the index-refinement
identity used to justify the structured kernel body.

**Inference.** Two stage bodies can agree on their coarse scheduled region while
being justified by different index refinements. The current identity surface
cannot distinguish those proofs.

**Proposal.** Bind stage coverage to the refinement identity before artifacts
or caches treat the stage coverage as complete executable identity.

**Resolved in the tree; accepted 2026-08-05.**
[`bind-stage-coverage-to-index-refinement-identity`](../../../tickets/bind-stage-coverage-to-index-refinement-identity.md)
made each covered occurrence a `CoveredOccurrence` carrying the reached-only
executable-coverage identity of the receipt that proved it, folded by both the
kernel program's own encoder and the artifact's independently serialized stage
key — `tiler.kernel-program.v9` and `tiler.artifact-program.stage.v3`. The
exact proof-bound stage-coverage surface was accepted by Tom at the live
decision review recorded in
[`accept-the-proof-bound-stage-coverage-public-boundary`](../../../tickets/accept-the-proof-bound-stage-coverage-public-boundary.md).
Acceptance makes it current pre-alpha vocabulary, not a stabilized published
API. The inference above is now false of the tree: two stage bodies justified by
different index refinements have different program and artifact identities.
What the finding did not anticipate is *which* identity to fold. The receipt's
complete identity restates the frozen registry snapshots, so folding it would
have made an unused provider revision invalidate an otherwise identical
artifact; the reached-only projection is folded instead.

### Artifact schema evolution needed an explicit major transition

**Fact.** During the audit, the program envelope's variant layout changed while
the manifest schema still reported the preceding major version.

**Inference.** A decoder could interpret incompatible bytes under an unchanged
schema identity.

**Result.** The rechecked snapshot reports program manifest schema `4.0`.
This specific issue is resolved in the later snapshot. It remains a useful
review rule: a byte-layout incompatibility and its major schema transition must
land together.

## High-value structural refactors

### `tiler-ir/src/index/builder.rs`

**Fact.** At 4,613 lines, the file owns public construction, reduction,
canonical compaction, proof-related validation, identity formation, and tests.

**Inference.** The reader must absorb several different abstraction levels to
find one operation. The file is large because responsibilities accumulated, not
because the index model needs another crate.

**Proposal.** Preserve `tiler_ir::index::builder` as the public disclosure point
and use this shallow internal layout:

```text
index/builder/
  mod.rs
  access.rs
  reducer.rs
  proof.rs
  compact.rs
  identity.rs
  tests.rs
```

`mod.rs` should explain the construction lifecycle, own re-exports, and keep
the common path legible. Files should be divided by invariant and phase, not by
arbitrary line count.

### `tiler-compiler/src/pipeline.rs`

**Fact.** The 5,118-line file combines the compiler entrypoint, transactional
rewrite flow, alternative construction, trace production, verification,
conformance support, and roughly half its size in tests.

**Inference.** The important orchestration path is hidden among mechanisms and
fixtures. This is the clearest progressive-disclosure failure in the compiler.

**Proposal.** Keep `pipeline` as one public concept and split beneath it:

```text
pipeline/
  mod.rs
  transaction.rs
  alternatives.rs
  trace.rs
  verify.rs
  conformance.rs
  tests.rs
```

The root should read as the compilation story. Transactional mutation,
enumeration, explanation, and conformance should remain separately inspectable.

### `tiler-reference/src/lib.rs`

**Fact.** The 4,100-line crate root contains public tensor representation,
semantic registration and evaluation, scalar dispatch, index evaluation,
arithmetic, errors, and a large inline test suite.

**Inference.** A crate root that implements most of the crate conceals which
pieces constitute the public reference contract and which are mechanisms.

**Proposal.** Retain a small `lib.rs` that introduces and re-exports:

```text
tensor.rs
semantic_registry.rs
semantic_eval.rs
scalar_registry.rs
index_eval.rs
arithmetic.rs
error.rs
tests.rs
```

Do not merge the semantic and scalar registries merely because both dispatch
behavior. They govern different identities and extension obligations.

### Compiler target concerns

**Fact.** Target requests and physical-planning types depend on each other
across several top-level compiler modules. Target description, feasibility,
honourability, and assessment are spread across the crate.

**Inference.** The public compiler surface exposes implementation navigation
cost, and dependency direction is harder to see than the architecture requires.

**Proposal.** Form one private, shallow target cluster:

```text
target/
  mod.rs
  profile.rs
  feasibility.rs
  honourability.rs
  assessment.rs
```

This is an organizational boundary, not a new universal target IR and not
permission to move hardware facts into the logical graph.

### Embedded test concentration

**Fact.** Forty production source files contain inline test modules. Together
the embedded test sections account for roughly twenty-one thousand lines.

**Inference.** Moving tests is a lower-risk first step that reveals production
module boundaries without changing public APIs or algorithms.

**Proposal.** Extract large test modules before splitting production logic.
Keep small invariant-local tests beside their implementation when proximity is
more valuable than file-size reduction.

## Secondary findings

These issues are narrower than the findings above but should be considered when
their owning areas are next changed:

- Program-envelope decoding performs avoidable cloning of manifest and payload
  data. A bounded view during validation followed by deliberate ownership would
  reduce peak memory without weakening framing limits.
- Stage projection repeatedly searches stage collections. Explicit ordinal
  indexes would make both identity and complexity easier to reason about.
- Some edge-limit failures are classified through a nearby node-limit
  diagnostic. Limits should identify the resource that was actually exceeded.
- Debug-oriented `Display` implementations can produce ambiguous
  user-visible explanations. Stable diagnostic text should be designed rather
  than inherited from debug formatting.
- Metal AOT errors retain important distinctions internally but flatten some
  causal detail into strings. Preserve typed tool, phase, exit, and output
  structure through the public error boundary.
- “Usable metallib” language exceeds what offline compilation proves. A
  successful linker output is a compiled artifact, not evidence of runtime
  compatibility on every declared deployment target.
- Multi-target compilation can stop at the first failure even when the caller
  needs per-target delivery outcomes. Batch reporting should preserve each
  requested target's result.
- Governed fact fields are validated internally but are not exposed as a shared
  vocabulary. Consumers should not have to reproduce the closed field set.
- Whole-file dead-code allowances hide narrower maturity boundaries. Prefer
  item- or module-level admissions with an explicit reason.
- Comments that refer to removed Python documentation checks are stale. Current
  documentation maintenance is manual and claims must say so.

## Known identity gates

Two cross-component bindings deserve continued scrutiny:

- The artifact ABI and the executable program ABI need an exact, validated
  relationship, including the expressions that determine binding sizes and
  offsets.
- A cache subject must be bound to the provenance and identity of the payload
  it carries. A valid entry digest is not by itself proof that the requested
  subject and returned artifact are the same computation.

These are not arguments for a universal identity object. Each layer should own
its identity, and crossings should carry explicit verified bindings.

## What should remain unchanged

- Keep the present crate boundaries generally flat.
- Keep the runtime consumer- and device-object-neutral.
- Keep offline Metal tool discovery out of ordinary compiler and runtime
  consumers.
- Keep reference evaluation independent enough to check compiler behavior.
- Keep hard feasibility separate from estimated cost.
- Keep semantic, scalar, capability, and provider registries distinct where
  their governed identities differ.
- Keep stage-oriented program structure visible; do not hide it behind a
  universal execution graph.

## Recommended order

1. Correct identity and one-way-commit gaps before making their affected
   surfaces stable.
2. Extract the largest test modules.
3. Split the index builder, compiler pipeline, and reference crate root beneath
   their existing public modules.
4. Consolidate private compiler target concerns without changing the public
   semantic model.
5. Address secondary diagnostic, allocation, and batch-reporting issues when
   their owning modules are already in scope.

Each structural step should preserve public paths unless a separately reviewed
boundary change justifies otherwise. File movement alone is not evidence of
better architecture; the success criterion is that the common path is visible
first and each deeper file owns a coherent invariant.

## Disposition

This report is an audit and a prioritized refactoring map. Its proposals are not
accepted public API decisions and do not authorize production implementation.
Correctness findings should close through their existing or newly reconciled
tickets; structural work should be scoped only after the affected public
boundaries receive the review required by repository policy.
