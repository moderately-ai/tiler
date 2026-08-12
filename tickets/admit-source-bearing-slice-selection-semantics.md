---
id: admit-source-bearing-slice-selection-semantics
title: Admit source-bearing Slice selection semantics
status: todo
priority: p1
dependencies: [decide-the-source-bearing-slice-offset-boundary]
related: [admit-a-position-selecting-slice-for-the-rotary-table, evaluate-retained-shape-relations-before-routing-commit]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, slice, symbolic-extents, semantics, reference]
---
# Admit source-bearing Slice selection semantics

## Goal

One `tiler::slice-f32@1` window can name either a literal or a `ShapeSymbol` offset through `SourcedExtent`, while `ShapeEnv` remains the only binding authority. Semantic construction, inference, and reference evaluation prove the window against that exact environment and refuse unavailable or unproved sources without defaults.

## Work

- Re-audit every Fact in the accepted decision at the exact implementation base before editing; repair this ticket if source drift changes the purpose.
- Replace the literal-only window offset with the accepted `SourcedExtent` product while preserving every existing literal canonical byte. Add injective symbolic encoding, total decode/encode handling, and typed observation without introducing a second `symbolic-window` relation.
- Validate a selection against the program's exact `ShapeEnv` and sourced operand shape. Prove checked `offset + extent <= available_axis` and the existing proper-window rule for every admitted model; retain explicit result extent on restricted axes and sourced extents on untouched axes. Refuse foreign, undeclared, late, overflowing, or unproved sources by distinct typed causes.
- Extend reference evaluation with one authenticated, immutable extent-binding context derived from the exact program and declared inputs. Callbacks never accept a duplicate cursor scalar. Support only binding kinds with a complete authority path and refuse the rest explicitly.
- Audit and update the normative definition, standard semantic registry/provider revision, reference capability revision, facts, documentation, and every resulting identity pin. Preserve `tiler::slice-f32@1` and `tiler.slice-selection.v1` only after proving old literal bytes remain unchanged and new symbolic values are injective.
- Keep index refinement, live kernel operands, artifact/runtime delivery, the rotary consumer, strided windows, and tensor-element-driven dynamic Slice outside this ticket.

## Acceptance

- Literal selections retain byte-for-byte canonical encodings and existing result behavior.
- Static and `InputDimension` source offsets pass only when the same environment proves their bounds; symbol, root-binding, phase, and interval perturbations each fail under their named cause.
- Two symbols resolving to the same runtime number remain identity-distinct; changing a symbol's binding source moves the fifth semantic subject.
- Reference evaluation obtains the offset from the authoritative binding and returns the same result as the literal neighbour. Supplying no binding, a foreign binding, or a second inconsistent spelling is impossible or refused before evaluation.
- Complete source-first Fact verdict, identity blast radius, targeted IR/reference tests, subject perturbations with failure text, rustdoc, Clippy, `tkt lint`, `git diff --check`, `tkt guard`, and the required repository gate are recorded.
