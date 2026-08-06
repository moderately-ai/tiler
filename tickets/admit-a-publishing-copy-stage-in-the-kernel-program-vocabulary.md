---
id: admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary
title: Admit a publishing copy stage in the kernel-program vocabulary
status: todo
priority: p2
dependencies: []
related: [admit-elementwise-epilogues-over-a-materialized-intermediate, recognize-several-ordered-named-outputs-at-the-compiler-request-boundary]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer]
---
## User-visible outcome

A program that both publishes a value and consumes it — the conformance suite's own multi-output fixture, publishing `scaled` and reducing it into `reduced` — compiles, instead of refusing at the request boundary under `output-partition-overlap`.

## Why this exists

**Fact — every *region* the shape needs is now built.** `admit-elementwise-epilogues-over-a-materialized-intermediate` admitted an elementwise region reading `TensorRole::Intermediate` and writing `TensorRole::Output`: that is exactly the copy stage, and `crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs` measures the schedule vocabulary admitting it. Landing that ticket nevertheless left `a_published_and_consumed_intermediate_refuses_by_name` green, which is the fact this ticket exists for.

**Fact — the remaining wall is program-scope coverage, read from the verifier rather than inferred.** A stage that publishes a value another region computed claims *no* occurrence: the occurrence belongs to the producing region's walk, and claiming it twice would double-cover the semantic graph. `verify_partial_reductions` in `crates/tiler-ir/src/program/verify.rs` refuses every stage whose `coverage` is empty unless it is the declared combiner of a split:

```rust
if stage.coverage.is_empty()
    && !data.partial_reductions.iter().any(|split| split.combiner == ordinal(index))
{
    return Err(KernelProgramDiagnostic::UncoveringStage);
}
```

So the kernel-program vocabulary admits exactly one account for an uncovering dispatch, and a publishing copy is not it.

**Fact — the two compiler-side routes around it are both closed, checked by reading.** `crate::program::attribute_named_outputs` refuses a region that both materializes an edge and publishes a declared output (`AttributionFailure::MaterializesAndPublishes`), so the producing region cannot publish as a second stage without that widening too. And a *duplicating* cover — one region computing the value for the edge and another recomputing it for the publication — is refused by `CoverPolicy::governed`'s `CoverDuplicationAdmission::Forbidden` before it reaches assembly, which is a policy decision with its own owner.

**Inference — the widening is a declaration, not a relaxation.** The account a split declares is `PartialReduction`; the account a publishing copy needs is the analogous "this stage republishes value `v`, which stage `p` already covered". Adding it as a typed program-scope declaration keeps `UncoveringStage` refusing everything else, which is what makes the check still able to say no.

## Boundaries

- `crates/tiler-ir/**` for the declaration and its verification; `crates/tiler-compiler/**` for minting it during assembly and for relaxing `check_output_cover`'s `output-partition-overlap` in exactly the published-and-consumed direction.
- The other `output-partition-overlap` shape — two output keys naming one value — is **not** in scope and must keep refusing: `KernelProgramBuilder` refuses a second publication of one buffer, and no copy stage changes that.
- A kernel-program identity domain almost certainly moves with the new declaration. That is an identity step to execute completely, not to absorb.

## Closes when

`pipeline::conformance::a_published_and_consumed_intermediate_refuses_by_name` is replaced by an assertion that the program compiles and both published outputs bit-agree with the reference evaluator; a stage with empty coverage and no declaration still refuses by name, observed failing; and the compiler-facade gate row in `docs/correctness-and-testing.md` records the flip.
