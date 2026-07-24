---
id: widen-numerical-vocabulary-and-complete-identity
title: Widen the numerical vocabulary and complete its identity encoding
status: in-progress
priority: p1
dependencies: [accept-adr-0076-numerical-realizations]
related: [draft-target-honourable-numerical-contract-adr, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity]
claimed_from: todo
assignee: agent-widen-numerical-vocabulary-and-complete-identity
lease_expires_at: 1784929884
---
ADR 0076 items 1 and 6. This is the first of four ordered tickets and the only one that carries a latent correctness defect, so the two halves must land in one change.

## Vocabulary

`tiler_ir::schedule::SubnormalMode` has exactly one variant, `Preserve`, and `NumericalPermission` has exactly one, `Forbidden`. Two `NumericalRealization` values therefore cannot differ on any subnormal or permission dimension. Widen both to the behaviours accepted ADR 0019 and ADR 0011 already name — preservation and explicit flush-to-zero on each subnormal dimension, and a permitted form alongside `Forbidden`.

Keep `NumericalRealization`'s `input_subnormals` and `result_subnormals` independent. Coupling them is forbidden even when a target couples them; that is ADR 0019's operative sentence. Do not add a coupled "fast" spelling and do not express any dimension as a single boolean.

**A flush behaviour must state which zero it produces.** The measured Apple flush is sign-preserving: `0x80400000 * 2.0f` returns `0x80000000`, not `0x00000000`. `docs/numerical-semantics.md` already requires that "the zero sign follows the resolved signed-zero and subnormal contract rather than an ambient target mode", so a flush behaviour that leaves the sign unstated cannot be checked against measured hardware or reference-evaluated. Whether the sign is a field of the flush behaviour or resolved from the contract's signed-zero dimension is yours to choose; leaving it unstated is not.

## Identity — the defect this ticket must close

Three sibling encoders treat the realization three different ways, and only one of them fails closed when the vocabulary widens. Verify each by reading the file; do not conclude anything about them from a substring search.

- **`tiler_ir::kernel::model::push_numerical` is correct.** It encodes the profile key, canonical NaN bits, both subnormal modes, and both permissions, through `push_subnormal` and `push_permission` whose `match` arms are exhaustive over non-`#[non_exhaustive]` enums. Adding a variant is a build error there.
- **`tiler_ir::schedule::model::push_numerical` is the defect.** It encodes the profile key, the NaN bits, and the two `permits_*` booleans, and encodes **neither subnormal field**. Because those accessors are `!matches!(…)` expressions rather than matches, adding a variant compiles silently, and two regions that differ only in subnormal treatment receive the same `CanonicalScheduledRegionIdentity`. That is a cache-and-artifact correctness failure of exactly the class `AGENTS.md` singles out for special scrutiny, and it would not warn.
- **`crates/tiler-metal/src/emit.rs` guards a third way**, with irrefutable `let SubnormalMode::Preserve = mode;` bindings in `realization_requirements` and `record_subnormal_obligation` that become compile errors on widening. Those are outside this ticket's scope and will break your build; that is the guard working. Coordinate with `declare-metal-numerical-honourability` or add the scope.

The omission is masked today only because `profile_key` is encoded and every distinct contract has so far carried a distinct key. Relying on a key to stand in for the field values it names is an unstated invariant. Make `schedule::model::push_numerical` complete over every field and exhaustive per field, matching the kernel encoder, and update whatever scheduled-region identity fixtures that shifts.

## The second defect: `derive_requirements`

`tiler_ir::schedule::model::derive_requirements` computes `requires_strict_f32: !permits_reassociation() && !permits_contraction()` — reading contraction and reassociation and ignoring both subnormal fields entirely. Once the vocabulary widens, a subnormal-preserving contract that permits contraction and reassociation derives `requires_strict_f32 == false` and would be **admitted** on a target declaring no strict-`f32` support, because the obligation it needs was never in the predicate. Stop collapsing the realization into one summary bit; carry the region's declared realization forward per dimension so `select-numerical-contract-and-compose-feasibility` can compose it.

## The contract half

`docs/ir.md` states that "`IndexRegion` identity commits only to the canonical structural program: iteration and reduction domains, typed tensor boundaries, access maps, scalar operations and values, constraints, and ordered outputs", and its layered summary reads "`IndexRegion` commits to canonical iteration/scalar/access content." The numerical realization appears in neither enumeration — even though the implemented `IndexRegion` carries a `numerical: NumericalRealization` field and its encoder partially encodes it, and even though `ScalarProgram`'s own variants separately carry the canonical NaN bits and a contraction flag. So part of the realization sits inside "scalar content" while the region-level declaration sits outside it, and the contract and the implementation disagree about where it belongs in the identity layering. That disagreement is *why* the encoder can omit the field without contradicting anything written down.

State the answer in `docs/ir.md`. Repairing the encoder without stating where the realization sits in the layering would leave the next encoder free to make the same omission.

## Boundaries

Convention 5 applies: do not blanket-apply `#[non_exhaustive]`. These enums are matched to decide *support* and are encoded into identity across crate boundaries, so they must stay exhaustive — an unhandled variant must be a build error, which is the entire mechanism this ticket relies on.

Run the full gate. A widened enum with unchanged identity fixtures is expected to shift those fixtures; that shift is the evidence the encoding is now complete, so record the before and after rather than only asserting the new values.
