---
id: answer-input-element-counts-as-the-declared-tensors-own-count
title: Answer input element counts as the declared tensor's own count, not the reading region's domain
status: todo
priority: p2
dependencies: []
related: [answer-per-ordinal-element-counts-only-for-ordinals-an-output-reads]
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [defect]
---
## User-visible outcome

`NormalizedOutput::input_elements_at` reports the declared input tensor's own element count for every arm, so an opaque call's `PerElementOf` scaling over a widened read is sized by the buffer the ABI binds rather than by the region domain that reads it.

## Why this exists (measured 2026-08-07 on `tkt/answer-per-ordinal-element-counts-only-for-ordinals-an-output-reads`)

The five arms do not agree about what they answer. `Contraction` returns `input_elements[ordinal]` and `Staged` returns `operand_elements`, both of which are the **operand tensor's own** count. `SerialSum` returns `input_elements` and `Pointwise` returns `elements`, both of which are the **region's iteration domain**. The two coincide for a dense read, because a dense read binds the region's domain to the tensor's shape, and diverge exactly for a widening structural read.

Measured against `frontier.rs`'s `shared_input_two_domain_request` fixture (`w: [2]`, `a: [2, 2]`, `scaled = a * broadcast(w)`), by printing each recognized output's answer:

```text
PROBE output 1 ordinal 0: reads=true elements=Some(4)
```

Declared input `0` is `w` and holds **2** elements; the pointwise arm answers **4**, which is the `[2, 2]` domain the broadcast widens into.

`max_input_elements` carries the same divergence in its `SerialSum`, `Pointwise`, and `Epilogue` arms, and it feeds `frontier.rs`'s structural cost estimate rather than a feasibility gate, so its consequence is a mis-sized estimate rather than a wrong refusal.

## The consequence, and why it is not the sibling ticket's

`resolve_work_items` binds `TensorRole::Input { ordinal }`, which names the **buffer** the ABI binds at that ordinal. Scaling a call's work by the consuming region's domain over-counts whenever a read widens, which is a confidently-wrong work count of the kind `WorkScaling` exists to prevent — the same class as the `Intermediate` arm's superseded `input_elements` substitution.

`answer-per-ordinal-element-counts-only-for-ordinals-an-output-reads` fixed *which ordinals* each arm answers for and deliberately did not touch *what* it answers; its own refusing-neighbour assertion rests on the two divergent counts, so this ticket must revisit that assertion rather than only the arms.

## The work

Decide the one meaning `input_elements_at` states — the declared tensor's own count is the reading its two exact arms and its consumer's binding already imply — and make the two single-shape arms derive it from the recognized read list's `LogicalAccess` operand shape rather than from the region domain. A declared input read twice at two relations needs the same agreement rule the `Staged` arm already applies. Restate `max_input_elements` on the same basis or record why an upper bound may stay a domain.

Revisit `frontier.rs`'s `a_bound_ordinal_resolves_from_the_output_that_reads_it`: with the tensor-count reading, the two outputs of the shared fixture agree on ordinal `0` at 2, so the "one input read at two domains" refusal needs a fixture whose disagreement survives — an epilogue chain reading one ordinal from both halves is the candidate.

## Closes when

Every arm answers the declared tensor's own count, the widened-read count is observed at 2 rather than 4, and each perturbation is watched failing once.
