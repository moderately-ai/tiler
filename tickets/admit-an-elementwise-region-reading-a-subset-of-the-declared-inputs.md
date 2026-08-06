---
id: admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs
title: Admit an elementwise region reading a subset of the declared inputs
status: todo
priority: p2
dependencies: []
related: [admit-ordered-multi-output-programs-at-the-compiler-request-boundary, recognize-several-ordered-named-outputs-at-the-compiler-request-boundary, admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api]
---
## User-visible outcome

A program declaring several ordered named outputs whose recognition walks read *different* subsets of the declared input tensors compiles, so an output's region binds the inputs its own expression reads rather than every input the program declares.

## Why this exists

**Fact — the check is whole-program and the region it protects is per-output.** `recognize_elementwise` (`crates/tiler-compiler/src/request.rs`) is handed `declared`, the program's *entire* declared input list, and ends with `if reads.len() != declared.len() { return mismatch("elementwise-reads"); }`. `recognize_pointwise` and `recognize_reduction` both build that list as `program.inputs().map(|input| input.value()).collect()`, and `NormalizedPointwise::input_keys` is likewise every declared input key. So an output whose expression reads two of three declared inputs is refused, under a rule whose own comment states the property it is protecting: "Every declared input must be read. One that is not would bind a buffer the kernel never loads, and the expression's own dense-ordinal rule would refuse the assembled expression anyway".

**Inference — the rule was correct while a program had one output, and multi-output admission is what made it wrong.** With one declared output, an input no output reaches is dropped by `SemanticProgramBuilder` before the program is frozen — `request::tests::every_refusal_names_its_unrecognized_property` records exactly that for the `input-arity` row ("a frozen program drops the unused declaration"). So the whole-program input list and the single output's read set always agreed, and no fixture could distinguish them. [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) relaxed both arity guards, and two outputs can now split the declared inputs between them while every input is still read by *some* output.

**Measurement (2026-08-05, at the multi-output landing).** `crates/tiler-compiler/tests/multi_output_boundary.rs`'s `two_outputs_reading_disjoint_declared_inputs_refuse_for_their_unread_buffers` compiles `doubled = a + a` and `squared = b * b` over two declared `[4]` inputs, under every stated numerical contract, and observes `phase: "strategy", rule: "elementwise-reads"`. Its accepted neighbour is `two_output_region` in the same file — `product = a * b` and `sum = a + b`, the same two inputs, the same two families, the same independence — which compiles, so the refusal is about the *subset* and not about the second output.

## Boundaries

- This is about which inputs one output's region binds, not about input arity. [`admit-multi-input-elementwise-programs-at-the-compiler-boundary`](admit-multi-input-elementwise-programs-at-the-compiler-boundary.md) landed the general multi-input expression and is not to be re-opened.
- **`TensorRole::Input { ordinal }` is program-scoped, and must stay so.** `CoverAssembly::from_plan` checks `ordinal < semantic.input_count()` and binds `AssemblyBinding::Input(ordinal)` against the program's declared interface, so a region reading inputs 0 and 2 should carry those ordinals rather than a region-local renumbering. A renumbering would make the same expression at two sites carry different bytes and would have to be undone at assembly.
- What has to move together is the *expression's* leaf numbering and the region's bound input list: `PointwiseF32Expression` uses dense `InputOrdinal`s from zero, so the projection needs a stated map from the expression's leaf ordinals to the program's input ordinals, and that map is part of what the region's identity must fold. Decide whether it belongs on `NormalizedPointwise`/`NormalizedSerialSum` or on the built region before editing either.
- Whether the whole-program obligation "every declared input is read by *some* output" survives is a separate question from this one, and it should: an input no output reaches is a buffer the caller binds and no kernel loads. If it survives, it belongs beside `check_output_cover` in `select_supported_strategy`, which is where the other whole-program obligations moved.

## Required failure-path evidence

Each observed failing against an accepted neighbour: the disjoint-input two-output program above must compile rather than refuse under `elementwise-reads`; a program declaring an input *no* output's walk reads must still refuse, so the obligation is relocated rather than deleted; a region built for an output reading a subset must bind the program input ordinals its expression actually reads, checked against a fixture where those ordinals are not `0..n`; and two programs whose outputs read different subsets must keep distinct region identities.

## An unrecorded `tiler-ir` prerequisite, discharged 2026-08-06

**Fact — this ticket was filed dispatchable over a wall a crate down, and that wall is now gone.** Closes-when item 1 requires the region built for an output to bind exactly the program input ordinals its expression reads, and item 4's failure evidence names "a fixture where those ordinals are not `0..n`". `tiler_ir::schedule`'s `verify_pointwise_region` refused precisely that: it required read access `i` to be `TensorRole::Input { ordinal: i }` at every position, so a region binding inputs `0` and `2` was rejected by the intrinsic verifier no matter what the recognizer produced. [`admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`](admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary.md) separated the access position from the declared input the role names, for the epilogue's sake, and the rule is now that the reads name *strictly ascending* declared inputs — a gap between ordinals is admitted, a repeat or a descent is not.

**Inference — the second Boundaries bullet is now enforced by the verifier rather than only asserted here.** That bullet states `TensorRole::Input { ordinal }` is program-scoped and a region-local renumbering would have to be undone at assembly. The widening reached the same conclusion from the other direction and `crates/tiler-ir/src/schedule/builder.rs`'s `reads_bind_boundary_tensors_in_order` records the derivation; `read_accesses_must_name_strictly_ascending_declared_inputs` pins the non-prefix admission with a region reading declared inputs `0`, `1`, and `7`.

What this does *not* discharge is anything in this ticket's own scope: `recognize_elementwise` still requires `reads.len() == declared.len()`, `NormalizedPointwise::input_keys` is still every declared input key, and the leaf-ordinal-to-input-ordinal map item 1 asks for still has to be carried and folded into identity. The wall this ticket would have hit *after* doing that work is what moved.

## Closes when

1. Each recognized output carries the declared inputs its own walk reads, with an explicit map from the expression's dense leaf ordinals to the program's input ordinals, and the region built from it binds exactly those.
2. The whole-program obligation is relocated rather than removed, with the removal-shaped perturbation observed failing.
3. `two_outputs_reading_disjoint_declared_inputs_refuse_for_their_unread_buffers` is flipped from a refusal to a compilation, and `crates/tiler-compiler/tests/multi_output_boundary.rs`'s header paragraph naming this wall is rewritten to record what moved.
4. Any identity consequence of the ordinal map is executed as a complete step at its owning layer, with every moved pin enumerated — or shown by recomputation not to move.
