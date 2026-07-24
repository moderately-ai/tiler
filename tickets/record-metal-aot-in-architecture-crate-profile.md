---
id: record-metal-aot-in-architecture-crate-profile
title: Record tiler-metal-aot in the architecture crate profile and target-metadata ownership
status: in-progress
priority: p2
dependencies: []
related: [choose-one-owner-for-apple-target-vocabulary, prototype-apple-aot-driver]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, architecture, metal]
claimed_from: todo
assignee: agent-record-metal-aot-in-architecture-crate-profile
lease_expires_at: 1784919376
---
`docs/architecture.md` has two gaps about `tiler-metal-aot` that a reader hits together, both found while deciding `choose-one-owner-for-apple-target-vocabulary`. Neither is in that ticket's scope (`implementation/metal`, `implementation/metal-aot`), so they are routed here.

**Fact — the accepted prototype packaging profile omits the crate entirely.** The "Accepted prototype packaging profile" section says "The prototype uses five reusable libraries and two non-published proof executables" and lists `tiler-ir`, `tiler-reference`, `tiler-artifact`, `tiler-compiler`, and `tiler-metal`. `Cargo.toml` has had six library members since `tiler-metal-aot` landed, and `scripts/check_workspace.py` pins it in `EXPECTED_MEMBERS`, `EXPECTED_WORKSPACE_DEPENDENCIES`, and `EXPECTED_DEPENDENCIES` with an empty dependency list. The profile block should name it, record `tiler-metal-aot -> []`, and record the development-only `tiler-metal` → `tiler-metal-aot` edge that `compile-golden-msl-through-the-aot-driver-in-the-gate` added and justified. Check whether the count sentence and ADR 0065's own text need the same correction, or whether the ADR is correct as of its acceptance and only this document is stale.

**Fact — the Component ownership table reads as if one crate owns all Metal target metadata.** The `tiler-metal` row says "Pure structured-kernel-to-MSL translation and Metal target metadata". `choose-one-owner-for-apple-target-vocabulary` decided that the MSL language version, Apple artifact family, and deployment minimum stay owned by *both* crates: `tiler-metal` owns what emitted source declares, `tiler-metal-aot` owns what a compiler invocation selects, and neither record subsumes the other. The table as written is the sentence a future reader would cite while "fixing" the duplication into a shared crate or into a dependency edge that destroys the driver's empty dependency closure — which is the failure mode that ticket exists to prevent. Distinguish the two rows' target-metadata ownership, and state that the correspondence between the two vocabularies is enforced by a total map in `crates/tiler-metal/src/target_correspondence.rs` rather than by a shared type.

**One decision this ticket owns: whether the ownership split needs an ADR.** `choose-one-owner-for-apple-target-vocabulary` deliberately did not write one. It changed no public surface — nothing moved, no signature changed, no namespace opened — so ADR 0075's always-ask categories do not reach it, and the reasoning is recorded on the types themselves plus three tickets. The case *for* an ADR is that the decision fixes a crate-ownership and dependency-direction boundary, which is the genre of ADR 0065 and ADR 0070, and that the alternative it rejects (a shared crate for three enums) is exactly what a future worker will propose again. The case *against* is that an ADR whose whole content is "these three types stay duplicated, here is the enforced correspondence" adds a fourth place the reasoning lives. Decide it here: either add `contracts/decisions` and draft the ADR for Tom's acceptance, or record in the Outcome why the contract update plus the type-level record is the complete answer. Do not leave it implicit.

**What closes this ticket.** Both edits landed in `docs/architecture.md`, the ADR question answered either way, `uv run --locked python scripts/docs.py render` rerun, and the complete gate green. Do not restate the vocabulary decision's full reasoning here; it lives on the types in `crates/tiler-metal/src/target.rs` and `crates/tiler-metal-aot/src/input.rs`, and duplicating it would create the second authority the documentation contract exists to prevent.
