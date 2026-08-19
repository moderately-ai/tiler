---
id: pin-the-retired-contraction-key-compile-refusal
title: Pin the retired contraction key's compile refusal
status: todo
priority: p3
dependencies: []
related: [replace-the-standard-contraction-key-with-the-accepted-successor]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, tests, correctness]
---
## User-visible outcome

The no-fallback half of the ADR 0112 replacement is held by a landed regression test rather than by inference: a `tiler::strict-tensor-contraction-f32@1` program, built through the public extension machinery, refuses at `compile()` with a typed rule and never falls back to the successor.

## Why this exists — filed 2026-08-19 at integration of the replacement migration

The migration's independent review (finding 3) demonstrated the refusal by a disposable probe: `SemanticRegistryBuilder::standard()` + `register_provider` with a retired-key extension provider + `SemanticProgramBuilder::apply` builds the old-key program publicly, and `compile()` refuses it with `UnsupportedCapability { rule: "semantic-authority-pairing" }` before a target-qualified explain trace. Nothing in-tree pins that refusal, so a future recognition change could silently re-admit the retired key.

## Required work

One integration test in `crates/tiler-compiler/tests/` following the review probe: build the old-key program via public machinery (mirror the extension-provider shape of `replacing_only_the_occurrence_key_moves_the_semantic_graph_identity` in `crates/tiler-ir/src/semantic/contraction/tests.rs`, using the public `OperationInferencer`/provider traits), drive it through the ordinary `compile()` entry with a governed target, and assert the exact typed refusal rule observed at the then-current base — quote the actual rule text from a run, do not assume `semantic-authority-pairing` survives verbatim. Control: the same program shape under the successor key compiles (the existing direct-path tests already hold this; cite one rather than duplicating it). Perturb the subject once — flip only the key to the successor spelling inside the new test's builder — and show the refusal disappears, proving the test reaches the key and not some other property.

## Closes when

The test lands green with the quoted refusal, its subject perturbation is recorded, and the migration ticket's artifact-cross correction note resolves its citation.
