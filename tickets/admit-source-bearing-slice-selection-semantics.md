---
id: admit-source-bearing-slice-selection-semantics
title: Admit source-bearing Slice selection semantics
status: review
priority: p1
dependencies: [decide-the-source-bearing-slice-offset-boundary]
related: [admit-a-position-selecting-slice-for-the-rotary-table, evaluate-retained-shape-relations-before-routing-commit]
scopes: [implementation/ir, implementation/reference, implementation/compiler, contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, slice, symbolic-extents, semantics, reference]
claimed_from: todo
assignee: worker-admit-source-bearing-slice
lease_expires_at: 1786633149
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

## Fact audit at `1913da6f8373cdb29a80a002ab2f69b1488c8e33`

- **Verified — the accepted public product is `Window { offset: SourcedExtent, extent: Extent }`.** [`decide-the-source-bearing-slice-offset-boundary`](decide-the-source-bearing-slice-offset-boundary.md) is `done` and names that exact spelling. No third shape was invented.
- **Verified — at this base the window offset was still literal-only.** `crates/tiler-ir/src/semantic/slice.rs`, anchors `pub enum SliceAxisSelection` and `offset: u64`, stored a literal. `decode_axis`, anchor `(SLICE_RELATION_SYMBOLIC_WINDOW, _)`, refused the reserved relation before relation-specific fields. `SliceF32::infer` called `request.static_operand_shape(0)`.
- **Verified — `SourcedExtent` was already the crate's one constant-or-symbol vocabulary.** `crates/tiler-ir/src/shape/sourced.rs`, anchors `pub enum SourcedExtent` and `Self::Static(_) => 0x01`. ShapeEnv remains the only binding authority.
- **Verified — the coordinator's unverified Fact was true, not drifted.** Purpose unchanged.
- **Scopes added before editing:** `implementation/compiler` for the mechanical `SourcedExtent::as_static` adaptation and identity-pin re-derivation; `contracts/foundation` and `contracts/navigation` for the live `docs/ir.md` and support-matrix repairs. No artifact or metal scope. Symbolic index refinement remains excluded.

## Outcome

`SliceAxisSelection::Window` now carries `offset: SourcedExtent`. Construction is still shape-independent. `SliceSelection::apply` proves `offset + extent <= available_axis` and the proper-window rule against the program's exact `ShapeEnv`. Restricted axes keep the explicit window extent; untouched axes keep their sourced extents.

Literal window bytes stay `CanonicalValue::unsigned_u64` under `tiler.slice-selection.v1`. A symbolic offset is an injective `bytes` payload of `SourcedExtent::Symbol`; a static extent encoded as those bytes is malformed, so a literal has one spelling. `tiler::slice-f32@1` is preserved. `symbolic-window` remains reserved and is not a second window variant.

Reference evaluation derives one immutable `ExtentBindingContext` from the program environment and declared inputs. Callbacks resolve through that context and cannot accept a second cursor scalar. `Static` and `InputDimension` have a complete authority path; `InterfaceParameter` and `TargetProperty` refuse by named kind.

The Slice law and governed lowering still realize only a `SourcedExtent::Static` offset. A symbolic offset is `unsupported("slice-symbolic-offset")` there.

**Support-matrix row.** `tiler::slice-f32@1` stays **R5**. The row now includes source-bearing window offsets at semantic and reference. The strided form and symbolic lowering stay R1. No dtype-maturity cell moved. A semantic/reference widening without lowering is not a later rung.

**Identity blast radius.** `tiler.slice-selection.v1` and `tiler::slice-f32@1` did not step. Standard semantic provider `tiler::standard-semantics` 7 → 8 (inferencer and participation are behaviour the projection's participation tag plus Rust logic both record). Slice reference capability 7 → 8; other standard-reference capabilities stay at 7. Law encoding tag and slice law revision stay 1. Re-derived pins: registry snapshot `e2e2b84254505cfe…`, law-registry identity `0b8eba7dfbbdb33c…`, every standard law-row digest (provider revision is in the sidecar), slice law row `2a352358c72d1d4c…`, explain request `17e0dd47e48b7c18`.

**Quoted perturbation failures.**

- Foreign symbol: `slice.selection.undeclared-symbol: slice/0::ghost is not declared by this program's shape environment`
- Late phase: `slice.selection.source-too-late: slice/0::c is available at PreparedKernelPreflight, after LiveDevicePreflight`
- Unproved interval (`C` in `[0, 64]` against a 64-extent axis and extent 6): `the window on axis 0 names offset slice/0::c and extent 6 against available 64, and this program's shape environment does not prove the window stays inside that axis for every admitted model`
- Proved overflow (`C` in `[60, 64]`): `slice.selection.out-of-bounds`
- Root-binding change (`InputDimension` vs `Static(4)` on the same symbol): graph identity equal, fifth subject `shape_environment` unequal
- Two symbols that may resolve to the same number: canonical encodings unequal
- Reference `InterfaceParameter`: `reference.extent.unsupported-binding: slice-ref/0::c is a interface-parameter and this evaluator has no authenticated value source for that kind`

**Public surface.** `SourcedExtent` was already public. `SliceAxisSelection` lost `Copy` because a symbol is not `Copy`. `Window.offset` is the accepted product. `SliceAxisSelection::static_window` is a convenience constructor. `SliceSelection::apply` and `ExtentBindingContext` are labelled drafts until Tom accepts their exact included and excluded surface; the accepted decision already names the `Window` product and the reference binding-context obligation.

**Checks.** `cargo test -p tiler-ir` green. `cargo test -p tiler-reference` green. `cargo test -p tiler-compiler --lib` green. `cargo clippy -p tiler-ir -p tiler-reference --all-targets -- -D warnings` clean. `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-ir -p tiler-reference --no-deps` clean. `tkt lint`, `git diff --check`, and `tkt guard --base main --format json` recorded at commit time.
