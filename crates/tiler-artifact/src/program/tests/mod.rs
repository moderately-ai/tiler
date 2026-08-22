//! Bounded tests for the target-neutral artifact program model.
//!
//! Fixtures package real verified kernel programs over real verified semantic
//! programs, so every rejection is a rejection of a plan that the shared IR
//! itself already accepted. Nothing here asserts that a kernel computes the
//! operations its stage covers; that remains compiler-owned evidence.

mod baked_extents;
mod bf16_pointwise;
mod construction;
mod environment_roles;
mod expressions;
mod extent_operands;
mod foreign_handles;
mod governed_keys;
mod identity_determinism;
mod identity_encoders;
mod insertion_rules;
mod plan_determinism;
mod provenance;
mod recorded_identity;
mod route_requirements;
mod stage_keys;
mod support;
mod whole_artifact_rules;

// The seven items this suite shares with `crate::proof::tests` are `pub(crate)`
// rather than `pub(super)`; the rest of the fixture set stays module-local. The
// proof sidecar associates with a *real* verified artifact, and a second
// hand-built one would be a second thing to keep correct.
pub(super) use support::artifacts::{
    Formulas, declare_realization, declare_realization_over, entry, formulas,
    partial_window_artifact, payload, prepared_requirement, profile, rules, selection,
    spare_provider, variant,
};
pub(crate) use support::artifacts::{
    artifact_with_selected_operations, build_artifact, default_artifact, lowering_provider,
    realization_record,
};
pub(super) use support::claims::{
    CLAIM_DESCRIPTOR, CLAIM_OBJECT, claim_declaration, claim_declaration_of, claim_payload_content,
    claimed_artifact, claimed_two_entry_artifact,
};
pub(crate) use support::encoded::strict_affine_u4_dequantize_artifact;
pub(super) use support::graphs::{
    BIAS_BITS, CANONICAL_NAN, ELEMENT_BYTES, build_graph, input_shape, output_shape, strict,
};
pub(crate) use support::graphs::{
    OTHER_SCALE_BITS, SCALE_BITS, build_graph_scaled, semantic_program,
};
pub(crate) use support::kernels::fused_program;
pub(super) use support::kernels::{SCRATCH_OFFSET, fused_kernel, partial_window_program};
pub(crate) use support::live::live_extent_program;
pub(super) use support::pointwise::{bf16_pointwise_artifact, f32_pointwise_artifact};
pub(crate) use support::routes::{requiring_artifact, route_feature, route_resource};
