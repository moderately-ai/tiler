//! Bounded tests for the target-neutral artifact envelope codec.
//!
//! Round-tripping is the weakest evidence a codec can offer, so it is the
//! smallest part of this suite. Two stronger properties carry most of the
//! weight.
//!
//! **Canonical form.** Two artifacts with equal identity must encode to equal
//! bytes, and an envelope that is well formed but not canonical must be refused
//! rather than normalized on the way in. The order-independence cases prove the
//! first; the forged-model cases prove the second.
//!
//! **Fail-closed under a competent adversary.** Corrupting bytes and watching a
//! digest reject them proves very little, because a forger recomputes digests.
//! The forged-model cases therefore build a *structurally invalid envelope*,
//! encode it — which stamps a correct manifest digest, correct section digests,
//! and a correct identity for whatever it now says — and require the decoder to
//! reject it anyway, with the named cause. The byte-level cases separately
//! prove that an incompetent corruption cannot slip through either.

mod binding_targets;
mod byte_corruption;
mod canonical_order;
mod carried_payloads;
mod carriers;
mod expression_arena;
mod extent_operands;
mod forged_models;
mod hot_path;
mod payload_sections;
mod plan_determinism;
mod provenance;
mod round_trip;
mod route_requirements;
mod section_descriptors;
mod selected_providers;
mod subgroup;
mod support;
mod vocabularies;
