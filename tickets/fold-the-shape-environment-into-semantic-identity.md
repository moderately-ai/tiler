---
id: fold-the-shape-environment-into-semantic-identity
title: Fold the shape environment into semantic identity as a fifth subject
status: todo
priority: p1
dependencies: [carry-a-sourced-shape-on-semantic-values]
related: [carry-symbolic-extents-into-the-semantic-program, compose-the-complete-expansion-cache-subject]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, identity, correctness]
---
## User-visible outcome

Two programs spelled identically over differently constrained or differently bound environments have different identities, so no cache can serve one for the other, and graph meaning stays separable from binding provenance.

## Why this exists

**Fact.** `SemanticIdentity` owns exactly four subjects and `SemanticGraphIdentity`'s documentation states that it "identifies graph meaning" with provider implementations, registry snapshots, and compilation provenance "deliberately excluded". Reproduce with `grep -n "pub struct SemanticIdentity" -A 6 crates/tiler-ir/src/semantic/identity.rs`.

**Fact.** `encode_shape` writes the rank and then eight raw big-endian bytes per extent with no discriminator, under domain `tiler.semantic-graph.v2\0`. A tagged encoding therefore changes a wholly static program's bytes even though its meaning does not — the same situation that moved `tiler.index-region.v8` to `v9`.

**Inference.** Folding the whole `ShapeEnvIdentity` into `graph` would put root-binding provenance into a subject documented to exclude that class of fact, and the accepted three-identity table puts binding on the interface side. [The symbolic-semantic-extents record](../docs/research/shapes/symbolic-semantic-extents.md) runs the elimination.

## Implementation keys

- Add the environment as a fifth `SemanticIdentity` subject, keeping the private fields and the absence of a public constructor, so component-wise assembly from different programs stays impossible.
- Decide whether the subject is optional. Optional makes "this program declares no symbols" and "this program has an empty environment" two states; total over an empty-environment identity makes them one. Run the elimination and state it; do not leave both readings live.
- Encode a symbolic extent as its *symbol*, never a resolved value, matching `SourcedExtent::encode` and for the same stated reason: folding a bound value collapses graph identity into specialized identity.
- Advance `tiler.semantic-graph.v2` to `v3`. Do **not** advance `tiler.shape-env.v3`: no byte a shape environment encodes changes, and a domain that advanced anyway would make two identical subjects carry different domains.
- Fold the environment identity exactly once. The artifact program subject and the expansion cache's `ComposedSubject` already carry the semantic subjects, so no cache facet, artifact section, or crate dependency is added. A third facet would be the second-authority failure `compose-the-complete-expansion-cache-subject` eliminated.
- Update `docs/ir.md`'s constraint-and-proof-context statement about what canonical identity includes, in the same change.

## Evidence

- One structure over two environments differing only in a constraint gives two identities; the same environment twice gives one.
- One structure over two environments differing only in which input axis binds the symbol gives two *environment* subjects and one *graph* subject — the split this ticket exists for, and the assertion that fails under the rejected alternative.
- A symbolic program and the literal program its environment determines have different identities, matching the index layer's own assertion that a boundary sized by a symbol is a different program from one sized by that symbol's value.
- Every pinned identity recomputed on the merged tree, with the exact check for pinned fixtures stated and run.

## Public boundary

The fifth accessor on `SemanticIdentity`, the identity domain advance, and the `docs/ir.md` sentence it corrects.
