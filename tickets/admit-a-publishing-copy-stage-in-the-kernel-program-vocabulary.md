---
id: admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary
title: Admit a publishing copy stage in the kernel-program vocabulary
status: done
priority: p2
dependencies: []
related: [admit-elementwise-epilogues-over-a-materialized-intermediate, recognize-several-ordered-named-outputs-at-the-compiler-request-boundary]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer]
---
## User-visible outcome

~~A program that both publishes a value and consumes it — the conformance suite's own multi-output fixture, publishing `scaled` and reducing it into `reduced` — compiles, instead of refusing at the request boundary under `output-partition-overlap`.~~

**Revised 2026-08-06 — premise falsified; see the Outcome.** The measured wall stack between the caller and that program is four deep, and the `tiler-ir` account this ticket names is the *last* of the four, unreachable today. The delivered outcome is: the stack measured rather than inferred, the four doc sites that made the remaining work look like a one-crate wiring job corrected, and the whole widening filed as [`lift-the-four-published-and-consumed-walls-together`](lift-the-four-published-and-consumed-walls-together.md).

## Why this exists

**~~Fact~~ — refuted 2026-08-06; necessary but not sufficient, see the Outcome. Every *region* the shape needs is now built.** `admit-elementwise-epilogues-over-a-materialized-intermediate` admitted an elementwise region reading `TensorRole::Intermediate` and writing `TensorRole::Output`: that is exactly the copy stage, and `crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs` measures the schedule vocabulary admitting it. Landing that ticket nevertheless left `a_published_and_consumed_intermediate_refuses_by_name` green, which is the fact this ticket exists for.

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

~~`pipeline::conformance::a_published_and_consumed_intermediate_refuses_by_name` is replaced by an assertion that the program compiles and both published outputs bit-agree with the reference evaluator; a stage with empty coverage and no declaration still refuses by name, observed failing; and the compiler-facade gate row in `docs/correctness-and-testing.md` records the flip.~~

**Revised 2026-08-06.** Not reachable from this ticket's premise; carried whole into [`lift-the-four-published-and-consumed-walls-together`](lift-the-four-published-and-consumed-walls-together.md). This ticket closes when the wall stack is measured rather than inferred, the doc claims that overstated the remaining work are corrected, and the widening is filed with its derivation. All three are delivered below.

## Outcome — premise falsified 2026-08-06; the `tiler-ir` account is the last of four walls, not the only one

**Why this stopped rather than shipped.** The ticket's "Why this exists" derived, from every *region* of the shape being expressible in the schedule vocabulary, that "only the program-scope account is missing" and that lifting the row was therefore a `tiler-ir` widening rather than a recognizer change. The premise is true and the conclusion is false. Each individual region is expressible — `materialized_intermediate_epilogue_wall.rs` measures that — but the compiler never mints the second dispatch that would need the account, and three refusals stand in front of it, all in `tiler-compiler`. `UncoveringStage` is unreachable today. Discovery stop, per AGENTS.md: measured, recorded, edged, and the reachable remainder delivered.

**Measurement — worktree at base `2ebe90cb`, 2026-08-06, pinned nightly, governed baseline profile.** Each wall disabled in turn (`if false && …`) and the next refusal read from `compile()`, against the fixture's program respelled with `SemanticProgramBuilder::try_standard`:

| # | Wall | Site | Reported as |
| --- | --- | --- | --- |
| 1 | recognition walks overlap | `check_output_cover`, `crates/tiler-compiler/src/request.rs` | `phase: "strategy", rule: "output-partition-overlap"` |
| 2 | one region materializes *and* publishes | `attribute_named_outputs`, `crates/tiler-compiler/src/program.rs` | `phase: "program-assembly", rule: "cover-named-output-attribution"` |
| 3 | nothing writes the published value | `derive_dependencies`, `crates/tiler-compiler/src/program.rs` | `phase: "program-assembly", rule: "internal-unwritten"` |
| 4 | the publishing stage covers no occurrence | `verify_partial_reductions`, `crates/tiler-ir/src/program/verify.rs` | `KernelProgramDiagnostic::UncoveringStage` |

Row 4 is **inferred from the verifier's text, not measured** — rows 1–3 stop the program first and no stage that would trigger it can be minted yet. Every perturbation was reverted and the tree confirmed clean (`git status --porcelain` empty) before any deliverable was written.

**Fact — rows 2 and 3 are the finding.** Between row 1 and row 2, recognition, region formation, cover enumeration, legality, selection and planning all *succeed*. The cover legally places `{constant, multiply}` as both the producer of the materialization edge the fold reads and the retainer of the named result `scaled`, and `{sum}` as the fold publishing `reduced`. Nothing in the region or cover vocabulary had to move. Row 3 is what the copy stage's absence actually is: with the attribution admitted, the scaling region's one owning write goes to the edge and `scaled` is an internal value no stage writes.

**Fact — this ticket's own "Boundaries" bullet was right about the site and wrong about its weight.** `attribute_named_outputs`' `MaterializesAndPublishes` is cited there as a *closed route around* the `tiler-ir` wall. It is not a route around it; it is the wall in front of it, and it is one recognizer rule from the surface rather than a profile away.

**Fact — the existing fixture cannot become the compiling assertion.** `a_published_and_consumed_intermediate_refuses_by_name` builds from `ExternalSemantics`, and `externally_registered_operations_require_their_own_realization_authority` pins that such a program refuses under `capability` / `semantic-authority-pairing`. Disabling row 1 alone reports exactly that — row 1 was masking it. The "Closes when" above could not have been met by editing that test's assertion; it needs a `try_standard` program.

**Inference — the surviving design, with its alternative eliminated rather than deferred.** The copy is a *second dispatch of the region that computed the value*, structurally identical to a split reduction's final pass. The alternative — the copy as a cover region of its own — is eliminated: `form_candidate` refuses an empty member set under `member-multiset` and `verify_candidate` under `membership`; a candidate's occurrence identity is derived from its members, so a coverless region has none; the anchored partitioner only chooses candidates from `containing[anchor]`; and `augment`, the only other entry, returns immediately under the governed non-duplicating policy. Admitting it means a second region-identity scheme and a weakened `verify_candidate`. The full derivation is in the successor ticket.

**Identity — no domain stepped, and that is the deliberate choice, not an omission.** A program-scope declaration section steps `PROGRAM_DOMAIN` to `tiler.kernel-program.v10` on the `v6` precedent (a new declaration section, encoded unconditionally). Landing that ahead of any producer pays a global step — invalidating every cached artifact identity, and moving the two pins in `the_standard_metal_path_publishes_its_recorded_identities` (`crates/tiler-build/src/metal_plan.rs`, outside this ticket's scopes) — for a vocabulary nothing can reach, and risks a second step if the minting site needs a field the declaration does not carry. The successor ticket carries the determination, the pin enumeration, and the `implementation/build` scope. The explain qualifier `request=689c3aefc30f48d3` was observed, not inherited: nothing on this branch touches a `Normalized*` type, an encoder arm, or a recognized program shape, so no request-subject sub-tag moves.

### What landed on this branch

Nothing that changes behaviour. No recognizer, verifier, region builder, encoder, or identity moved.

1. **Corrected claims — four doc sites that made unreachable work look reachable.** `select_supported_strategy`'s multi-output paragraph and `check_output_cover` in `crates/tiler-compiler/src/request.rs`, `attribute_named_outputs` in `crates/tiler-compiler/src/program.rs`, and `a_published_and_consumed_intermediate_refuses_by_name` in `crates/tiler-compiler/src/pipeline/conformance.rs`. Each said, in its own words, that lifting the row was a `tiler-ir` widening and not a recognizer change; each now carries the measured wall order and names the site that owns each wall. The conformance doc also records the `ExternalSemantics` defect, so the next worker does not spend the discovery again.
2. **The successor ticket**, [`lift-the-four-published-and-consumed-walls-together`](lift-the-four-published-and-consumed-walls-together.md), carrying the measurement table, the surviving design with its eliminated alternative, the six sites that must move, the identity determination with its pin enumeration, and a defensible split for the coordinator if one dispatch is too large.

**Scope.** `crates/tiler-compiler/**` (exclusive `implementation/compiler`) and `tickets/**` (shared `project/tickets`) only. No `crates/tiler-ir/**` file was edited — which is the finding, not an omission, and is why `implementation/ir` stayed declared but unused.

**Recommended board move after integration.** `done`, not `blocked`: the revised outcome above is fully supported and the remainder is a live ticket, so holding this one open would deadlock nothing but would misreport it. Do not read this as the published-and-consumed row being discharged — the gate still refuses, by name, and the successor owns the flip.
