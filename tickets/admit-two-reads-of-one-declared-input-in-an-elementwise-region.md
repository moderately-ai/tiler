---
id: admit-two-reads-of-one-declared-input-in-an-elementwise-region
title: Admit two reads of one declared input in an elementwise region
status: todo
priority: p2
dependencies: []
related: [admit-elementwise-epilogues-over-a-materialized-intermediate, admit-the-structural-families-into-the-scheduled-region-vocabulary]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer]
---
## User-visible outcome

A program whose elementwise expression reads one declared input *both* densely and through a structural relation — `a * permute(a)` — compiles, instead of refusing at the request boundary under `structural-access-conflict`.

## Why this exists

**Fact — this refusal replaced a silently wrong result, measured.** At `912b6058`, `out = a * permute(a)` over `a = [[1, 2], [4, 8]]` compiled and returned `[1, 16, 4, 64]`, which is `permute(a) * permute(a)`; the reference evaluator gives `[1, 8, 8, 64]`. The region binds one read per declared input and the expression's two `Input { ordinal: 0 }` nodes share it, so the *mapped* relation served both leaves. `admit-elementwise-epilogues-over-a-materialized-intermediate` closed it fail-closed — the refusal is now named — and this ticket is the widening that would admit the program instead.

Reproduce the refusal with `cargo nextest run -p tiler-compiler -E 'test(every_refusal_names_its_unrecognized_property)'`, whose `structural-access-conflict` row carries the fixture and the measured wrong values.

**Fact — admitting it is a widening rather than a repair.** The region would need *two* reads binding one declared input, one dense and one mapped. `tiler_ir::schedule`'s `reads_bind_boundary_tensors_in_order` refuses that by name today — declared input ordinals must ascend *strictly*, and two reads of ordinal 0 do not — and it refuses it for a stated reason: "two reads naming one input would bind one buffer twice while the leaf that meant another tensor went unbound". Relaxing it means the region's read list and the program's declared interface stop being the same list, which `CoverAssembly::from_plan` currently relies on.

**Inference — the expression side is already general.** `plan_elementwise` keys leaves by *value*, and an epilogue already numbers its leaves by access position rather than by declared ordinal, so a second read of one input is expressible in the recognizer's own vocabulary. What is missing is the schedule-side admission and the assembly-side attribution.

## Boundaries

- `crates/tiler-ir/**` for the read-list rule and whatever attributes a repeated ordinal; `crates/tiler-compiler/**` for the read list and the assembly binding.
- The *other* `structural-access-conflict` shape — one leaf carrying two *different* structural relations — is a separate question and may stay refused.

## Closes when

`a * permute(a)` compiles and bit-agrees with the reference evaluator; a region binding two reads of one input with no attribution still refuses by name, observed failing; and the fixture in `every_refusal_names_its_unrecognized_property` moves from the refusal inventory to an accepted row.
