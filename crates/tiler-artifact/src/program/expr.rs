//! The ABI expression domain, re-exported from where ADR 0068 places it.
//!
//! The domain type, its admitted roots, validation, canonical identity, and
//! authoritative checked evaluation live in [`tiler_ir::program::abi`]. This
//! module existed here until `relocate-abi-expressions-into-tiler-ir` moved it,
//! and it survives only as the name the rest of this crate already imports
//! through — so the relocation is one edit rather than thirteen.
//!
//! What this crate genuinely owns under ADR 0068 is unchanged and lives
//! elsewhere: the versioned wire encoding in [`super::codec`], runtime fact
//! binding and phase enforcement in [`super::facts`], failure classification in
//! [`super::error`], and backend-payload mappings in [`super::model`].
//!
//! `AbiExprUse` and `AbiExprView` are **not** re-exported here. They are
//! artifact-facing: one names a use site this crate validates, the other is a
//! read view over this crate's stored arena. Neither is part of the domain.

pub use tiler_ir::program::abi::{
    AbiBinaryOp, AbiEvaluationError, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue,
    AvailabilityPhase, ExprNode, MAX_TARGET_PROPERTY_KEY_BYTES, TargetPropertyKey,
    TargetPropertyKeyError, binary_operand_type, evaluate, node_is_interface_only, node_phase,
    node_type, unary_operand_type,
};
