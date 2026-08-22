---
id: re-prove-the-live-extent-operand-association-at-decode
title: Re-prove the live-extent operand association at decode
status: in-progress
priority: p0
dependencies: []
related: [accept-the-live-extent-artifact-envelope-row, bind-frozen-live-extent-bytes-at-declared-backend-transports, associate-live-extent-operands-with-symbolic-semantic-interface-axes, package-the-admitted-live-schedule-into-a-symbolic-kernel-program]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, artifact, codec, fail-closed]
claimed_from: todo
assignee: worker-assoc
lease_expires_at: 1787427335
---
## User-visible outcome

A decoded artifact carrying a live input-extent operand row proves the same association its builder proved: the named interface axis is symbolic, its symbol is declared in the artifact's one retained environment, and that symbol is rooted at exactly that `(key, axis)` input dimension. Bytes no builder could have written are refused rather than routed.

## The gap, audited at `489cc3553965ef87d053cc15a11279a9e00b4ab4`

**Fact — construction proves four arms; decode proves one.** `check_extent_operand_association` (`crates/tiler-artifact/src/program/builder.rs`, anchor `fn check_extent_operand_association`) refuses a static axis (`ExtentOperandStaticAxis`), a symbol the retained environment does not bind (`ExtentOperandForeignSymbol`), a symbol rooted anywhere other than an input dimension (`ExtentOperandUnsourcedSymbol`), and a symbol rooted at a *different* input dimension (`ExtentOperandSourceMismatch`). The codec's error vocabulary has no counterpart to the last three: `grep -n "ExtentOperand" crates/tiler-artifact/src/program/codec/error.rs` names only `UnknownExtentOperandKey`, `ExtentOperandAxis`, `ExtentOperandType`, `ExtentOperandStaticAxis`, and `ExtentOperandTransport`. `grep -rn "ExtentOperandSourceMismatch\|ExtentOperandForeignSymbol\|ExtentOperandUnsourcedSymbol" --include="*.rs" crates/` returns sites in `program/builder.rs`, `program/error.rs`, and `program/tests/extent_operands.rs` only — nothing under `codec/`.

**Fact — the gap opened at `tiler.artifact-program.v21`, and nothing carried the arms across.** `check_extent_operand_static_axes` (`crates/tiler-artifact/src/program/codec/validate.rs`) used to refuse *every* declared row, because the decoded interface grammar carried only literal extents; its own doc records the change ("Per-axis since"). While it was blanket, the three missing arms were unreachable and their absence was harmless. The per-axis narrowing made them reachable and they were never added.

**Fact — `check_interface_symbol_coherence` deliberately does not close it.** Its doc states "A symbol on an axis the environment roots elsewhere is not one either", which is correct for the *interface* (the admitted same-shape population has every input wearing `n` while one roots it) and is exactly the case an operand row must not name.

**Fact — the one positive decode case in the tree is outside the construction-admitted population.** `a_well_placed_live_extent_row_over_a_symbolic_axis_is_admitted` (`crates/tiler-artifact/src/program/codec/tests/extent_operands.rs`) sets the row to key `input`, axis 1, and sets `envelope.inputs[0].extents[1]` to `forged_symbol("T")`. `forged_retained_environment` (`crates/tiler-artifact/src/program/codec/tests/forged_models.rs`) binds `T` to `BindingSource::Static(Extent::new(2))`. Construction refuses that exact combination as `ExtentOperandUnsourcedSymbol { source: "static 2" }` — the arm `a_live_operand_on_a_symbol_without_an_input_source_refuses` pins. Decode admits it, and the test asserts the admission.

**Fact — two source comments claim the missing checks exist.** `crates/tiler-artifact/src/program/codec/validate.rs` writes "a symbolic one is the case the row exists for and passes on to the association checks", and `crates/tiler-artifact/src/program/codec/error.rs` writes "passes to the association checks". There are no association checks after that point. Both are claims about current behaviour and are false.

**Inference — the consequence is a fail-open bind.** `bind_extent_parameters` (`crates/tiler-runtime/src/load.rs`) freezes `facts.input_extent(operand.key(), operand.axis())` onto transport `binding_count + ordinal`. Preflight's only shape check is `evaluate_retained_shape_relations`, which evaluates declared `SemanticInputConstraint`s and nothing else, so two interface axes wearing one symbol are not proven equal unless the author declared that constraint. A decoded artifact whose row names a non-root axis therefore binds an unrelated quantity into the kernel's loop bound, with no refusal on any path.

## Required work

- Add the three missing decode-side arms to `codec::validate`, with their own `ArtifactCodecError` variants, sited after `check_extent_operand_static_axes` so each narrower structural refusal stays reachable. The authority is the envelope's own retained environment and published interface; no second table.
- Correct the two false comments named above; state what decode proves and what it does not.
- Repair `a_well_placed_live_extent_row_over_a_symbolic_axis_is_admitted` so its admitted case is one a builder could produce — a symbol rooted at the row's own `(key, axis)` input dimension — and add a refusal case for each new arm.

## Required evidence

- Each new arm perturbed separately, subject not assertion, with the quoted failure text. A perturbation that reddens every arm at once does not show which is load-bearing.
- One negative control that the narrower structural refusals still fire first: a row that is both misordered and mis-rooted must still report `NonCanonicalOrder`.
- One control that the literal-at-a-rooted-axis population `tiler.artifact-program.v17` pins stays representable, so the new arms do not re-break `invocation_bindings_do_not_enter_artifact_identity`.
- State whether any identity or schema value moves. **Expected: none** — these are refusals over already-encoded fields, adding no byte to either encoder — but rederive rather than copy.
- Targeted `tiler-artifact` and `tiler-runtime` tests, Clippy, rustdoc, `tkt lint`, `git diff --check`, exact-base guard, and the repository gate.

## Recommended graph edge

`bind-frozen-live-extent-bytes-at-declared-backend-transports` should depend on this ticket. That p0 makes a backend consume the frozen bytes at the declared transport; consuming them faithfully from a row decode never associated is what turns this gap into an executed wrong result. The edge is the coordinator's to add.

## Closes when

No decoded artifact can carry a live-extent operand row that `ArtifactProgramBuilder` would have refused, each arm is watched failing on its own refusal, and no source comment claims a check that does not run.
