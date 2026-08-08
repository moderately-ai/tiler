---
id: correct-the-slice-normative-definition-and-recompute-compiler-identities
title: Correct the Slice normative definition and recompute compiler identities
status: done
priority: p1
dependencies: [correct-the-symbolic-coefficient-era-index-vocabulary-claims, pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach]
related: [admit-a-position-selecting-slice-for-the-rotary-table]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, identity, correction]
---
## Why this exists

`SLICE_F32_NORMATIVE_DEFINITION` still says that no index-expression variant can carry a bound extent symbol in a coordinate position and that semantic value facts carry static extents. Both premises are false after the symbolic-coefficient and sourced-semantic-result-shape work landed. Unlike nearby comments and diagnostics, this string is identity-bearing and cannot be corrected as an isolated prose edit.

## Starting evidence, stale until re-read at this ticket's base

- **Verified at `a6eab57fa7d44a5be4c2e7990906d6f60e577b4c`.** `crates/tiler-ir/src/semantic/slice.rs`, anchor `SLICE_F32_NORMATIVE_DEFINITION`, contains the stale source clauses `no index-expression` and `carries static extents`, and `register_standard_slice` registers them through `OperationDefinition::new`.
- **Verified at that base.** `crates/tiler-ir/src/semantic/registry.rs`, anchor `fn encode_operation_definition`, length-frames the normative definition into `fn encode_definition_projection` and `fn encode_registered_operation`; the latter reaches the snapshot through `fn compute_identity`.
- **Imprecise as written because it named no population.** Compiler code consumes the standard-registry subject and the semantic bundle, but exactly one affected literal compiler pin exists: the `request=4e10437fec85d7b1` qualifier in `explain::tests::deterministic_trace_is_sealed_and_rendered_separately`. The other affected compiler identities are re-derived values: the frozen lowering registry, realization-law registry, fusion-legality occurrence proof, index-realization authority and resolution, complete refinement receipt, compiler refinement content and occurrence, verified request, compilation/explain subject, fusion numerical evidence, and program alternative. The reached-only executable-coverage, selected physical content, kernel-program, and artifact populations do not move for currently lowerable non-Slice programs, because they exclude the full registry snapshot under ADR 0072 and no governed lowering capability exists for Slice.
- **Imprecise as written about the refusal point.** `SliceAxisSelection::Window` has `offset: u64`; `decode_axis` has already decoded the relation name when it refuses `symbolic-window`, but refuses before parsing any relation-specific fields; `SliceF32::infer` later calls `request.static_operand_shape(0)` for literal bounds and result inference. General `ValueFact` shape carriage is sourced.

The worker's first deliverable is a per-Fact verdict at the exact base, including the complete ticket, `slice.rs`, registry encoding, all compiler consumers and pins, and the identity-domain census this ticket depends on. Repair this ticket before editing if the propagation graph or affected population differs.

## Re-audited propagation and version boundary — 2026-08-08

The normative bytes have two distinct semantic routes. A program that reaches Slice moves its `SemanticDefinitionProjectionIdentity`; any residual semantic-precondition identity in such a program also moves because that identity frames the program's reached definitions. Every program built against the standard registry moves only the registry-snapshot component of its `SemanticIdentity`, whether or not it reaches Slice. Semantic graph, admission-provenance, and shape-environment identities do not move.

The full standard snapshot then reaches the lowering-registry identity directly and through the pooled capability authorities, the realization-law-registry identity, fusion-legality occurrence proofs, complete index-refinement authorities and receipts, compiler refinement identities, verified request subjects, explain identities, fusion numerical evidence, and program-alternative identities. Governed lowering carries twenty capabilities and none is Slice, so no governed capability's reached-definition projection moves. The scalar-registry snapshot and realization-law sidecar do not move either.

This is a value change inside an existing length-framed field, not a grammar change. Keep `tiler::slice-f32@1`, the standard-semantics provider revision, every semantic/compiler/refinement domain separator, the request-subject `v6` domain, and the explain renderer version unchanged. The compiler identity census remains twenty-five distinct direct domains and twenty-six source occurrences; its five source-sized guards must remain green.

One nearby claim is stale but outside this ticket: the pooling rationale at `crates/tiler-compiler/src/capability.rs`, anchor `Why a pool and not an inline copy`, quotes an undated five-capability/1,496-byte/15,583-byte measurement. The governed population is now twenty and the audit measured a 43,741-byte standard snapshot and 137,779-byte lowering identity. Preserve that discovery for coordinator follow-up rather than silently repairing unrelated historical measurement prose here.

## Outcome

Correct the normative definition so it states the actual Slice-family boundary without claiming all semantic values are static. Recompute every reached semantic identity, registry subject, compiler expectation, fixture, and pin affected by those bytes on the merged tree. Keep the identity-domain grammar unchanged unless the encoding grammar itself changes: moving a value within an existing framed normative-definition field requires recomputation, not a separator-version step.

Add or strengthen a correctness-bearing assertion that reaches the normative bytes and identity consumers. Perturb the subject, not the assertion, and record the failure text before restoring it.

## Non-goals and stop conditions

Do not add the reserved symbolic-window relation, choose attribute versus operand carriage, change Slice inference, or widen a public boundary. If correcting the text exposes a need to change the semantic encoding grammar rather than its value, stop and map that domain step separately. If an affected identity has no owning pin authority, stop and create it rather than updating an ad hoc expectation.

## Closes when

The normative bytes match current behavior; every affected identity and pin is enumerated and recomputed on the final merged base; unaffected domains are justified; subject perturbations prove the checks reach both the definition and its compiler consumers; package checks, doctests, Clippy, rustdoc with warnings denied, formatting, `tkt lint`, citations, and `tkt guard` pass.

## Worker evidence — 2026-08-08

The correction changes only `SLICE_F32_NORMATIVE_DEFINITION`: sourced extents remain generally available to semantic value facts, while Slice's admitted window grammar carries a literal offset and literal extent and no source-bearing selection field. The definition now states the exact decoder boundary — after the relation name is decoded, before any relation-specific field is parsed — and the later static-operand-shape requirement for bounds and result inference. No symbolic relation, schema, inference path, reference semantics, or public surface changed.

The exact compiler pin census remains one literal:

```text
$ rg -n 'request=[0-9a-f]{16}' crates/tiler-compiler/src crates/tiler-compiler/tests
crates/tiler-compiler/src/explain.rs:3883:                "tiler-explain-v7 request=7ba3d77a66f04638\n",
```

That qualifier was recomputed from `4e10437fec85d7b1` to `7ba3d77a66f04638`. The derived identities named above move through their standard-registry component and remain computed rather than independently pinned. Reached-only executable coverage, selected physical content, kernel programs, and artifacts remain unchanged for the currently lowerable non-Slice population. The compiler-domain guard remains twenty-five distinct domains in twenty-six source occurrences, with all five guard tests passing; no separator, operation version, provider revision, request-subject version, or explain-renderer version moved.

Four negative controls perturbed the normative subject while leaving assertions and pins untouched, then restored it:

1. Reordering `semantic value facts generally can carry` to `semantic value facts can generally carry` made the direct IR test fail with `the normative reference omits the current literal-versus-sourced boundary clause "semantic value facts generally can carry sourced extents"`.
2. Removing `only` from the admitted-window grammar made the compiler Slice proof test fail with `the compiler proof does not carry the corrected Slice selection grammar`, and its printed `ReachedDefinition` contained the perturbed normative bytes.
3. With the real correction present and the old qualifier still pinned, the explain test failed with `left: "tiler-explain-v7 request=7ba3d77a66f04638\\n..."` and `right: "tiler-explain-v7 request=4e10437fec85d7b1\\n..."`.
4. After updating that sole pin, changing `For an admitted literal` to `For each admitted literal` moved the computed qualifier again: the explain test failed with `left: "tiler-explain-v7 request=4193766c9b388a02\\n..."` and `right: "tiler-explain-v7 request=7ba3d77a66f04638\\n..."`.

Package verification on the restored final tree:

- `cargo check -p tiler-ir` and `cargo check -p tiler-compiler`: passed.
- `cargo nextest run -p tiler-ir`: 989 passed; `cargo nextest run -p tiler-compiler`: 808 passed, one configured skip.
- `cargo test -p tiler-ir --doc`: 17 passed and one ignored across both doctest runs; `cargo test -p tiler-compiler --doc`: 13 passed.
- `cargo clippy -p tiler-ir --all-targets -- -D warnings` and the corresponding `tiler-compiler` command: passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-ir --no-deps` and the corresponding `tiler-compiler` command: passed.
- `cargo nextest run -p tiler-compiler -E 'test(domains::)'`: five passed.

The stale five-capability/measurement comment in `capability.rs` remains unchanged and is a coordinator follow-up, not hidden work in this identity correction.
