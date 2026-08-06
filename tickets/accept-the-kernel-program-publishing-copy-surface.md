---
id: accept-the-kernel-program-publishing-copy-surface
title: Accept the kernel-program publishing-copy surface
status: done
priority: p2
dependencies: []
related: [lift-the-four-published-and-consumed-walls-together]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [public-boundary, compiler-api]
---
## What Tom is being asked to accept

The exact public surface `tiler_ir::program` gained in `lift-the-four-published-and-consumed-walls-together`, which merged **labelled a draft**. Nothing outside `tiler-ir` may treat its spelling as settled until this node closes, and only Tom closes it.

```rust
/// One published copy of a value another stage computed.
pub struct PublishingCopy {
    pub source_stage: StageId,
    pub publisher: StageId,
    pub source: MaterializedValueId,
    pub published: MaterializedValueId,
}

/// A read-only view of one publishing-copy contract.
pub struct PublishingCopyRef<'a> { /* private fields */ }
impl<'a> PublishingCopyRef<'a> {
    pub fn source_stage(self) -> StageRef<'a>;
    pub fn publisher(self) -> StageRef<'a>;
    pub fn source(self) -> MaterializedValueRef<'a>;
    pub fn published(self) -> MaterializedValueRef<'a>;
}

impl KernelProgramBuilder {
    pub fn push_publishing_copy(&mut self, copy: PublishingCopy)
        -> Result<(), KernelProgramBuildError>;
}

impl VerifiedKernelProgram {
    pub fn publishing_copies(&self)
        -> impl ExactSizeIterator<Item = PublishingCopyRef<'_>>;
}

pub const MAX_PROGRAM_PUBLISHING_COPIES: usize = 4_096;

// KernelProgramBuildError
DuplicatePublishingCopy,

// ProgramLimitKind
PublishingCopies,

// KernelProgramDiagnostic
CopiedSourceNotInitializedBySourceStage,
CopiedSourceNotReadByPublisher,
PublishedCopyNotOutput,
PublishedCopyNotWrittenByPublisher,
PublishedCopyExtentMismatch,
```

`KernelProgramDiagnostic::UncoveringStage`'s meaning also widened: it now admits a stage covering no occurrence under **two** declared accounts — a split's combiner, as before, and a publishing copy's publisher. The variant, its rule code `uncovering-stage`, and its refusal for anything undeclared are unchanged.

## Why this exists, and the evidence behind the shape

**Fact.** A program that both publishes a value and consumes it downstream cannot express that with one write: `ValueRole` is exclusive, so the producing stage's owning write goes to the temporary its consumer reads across, and a second dispatch writes the published value. That second dispatch computes no operation of the bound graph, so `UncoveringStage` rejected it. This is structurally the same situation `PartialReduction` already solved for a split reduction's final pass, one fold up, and the surface is deliberately shaped like it.

**Fact.** `PublishingCopy` is not derivable from the entities already folded into program identity: a dispatch reading one value and writing another is the same stage, value, and edge set whether or not the program declares it to be a publication. It is therefore folded into `tiler.kernel-program.v10`, on the `v6` precedent, rather than checked away.

**Inference — the eliminated alternative.** Inferring the copy from structure (a stage that covers nothing, reads one value and writes an output, therefore *is* a copy) was rejected: it converts a producer's mistake into a silent admission, which is exactly the fail-closed line the uncovering-stage rule exists to hold. The declaration is a positive claim a producer makes and the verifier checks.

Verified end to end by `tiler-compiler`'s `pipeline::conformance::a_published_and_consumed_intermediate_compiles_and_agrees`: the governed published-and-consumed program compiles through the ordinary `compile()` path and both published outputs bit-agree with `ReferenceEvaluator::standard()`.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects the surface. Record the acceptance sentence with who, the date, and the venue; on acceptance, remove the draft labels from `PublishingCopy`, `PublishingCopyRef`, and `push_publishing_copy`, and record the surface in `docs/ir.md` beside `CoveredOccurrence`'s own proposal paragraph.

## Decided — accepted

Accepted by Tom on 2026-08-06 at the morning decision review in the coordination session, witnessed first-hand by the coordinator, with the evidence packet this node carries. Acceptance is not stabilization; the surface is accepted pre-alpha vocabulary.

## Sweep correction — 2026-08-06

The acceptance sweep above was executed incompletely and the gap was found by the vocabulary audit hours later: of the three draft labels the obligation names, only `PublishingCopy`'s moved — `PublishingCopyRef` (`model.rs:1464`) and `push_publishing_copy` (`builder.rs:712`) kept "Draft public surface" while pointing at a type recording acceptance, and the `docs/ir.md` paragraph was never written. Root cause: the sweep grep matched one label pattern and its single hit was treated as the population — the uniform-pass failure the process warns about. All three labels and the `docs/ir.md` paragraph are corrected in this change; the audit's finding is the evidence trail.
