#![allow(
    clippy::wildcard_imports,
    reason = "every child here is one layer of `request`, not a separate concept: \
the module was one file, its layers share one import surface, and each child \
globs that surface from the root exactly as `pipeline`'s children do. \
Enumerating the parent's imports per child would restate the same forty names \
thirteen times and would have to be restated again on every change. The child \
globs are `super::*` only — the parent form the workspace macro-boundary scan \
admits — while the root itself enumerates every carried item explicitly"
)]

//! The compilation request boundary.
//!
//! One caller-submitted program, contract preference, budget set, target list,
//! and installed authority go in; a recognized program, one verified slot per
//! target, and the canonical request subject every later stage is bound to come
//! out. Nothing below this boundary may widen what the caller stated, and
//! nothing here may decide what a target implements.
//!
//! The children are the boundary's layers rather than separate concepts, so this
//! root holds their shared import surface and re-exports each at its own
//! declared visibility:
//!
//! - [`authority`] and [`contract`] and [`budget`] — what a caller submits.
//! - [`verify`] — the phase order admission checks it in.
//! - [`recognize`], [`elementwise`], [`structural`], [`folded`], [`graph`] — the
//!   walk that decides which region partition implements each declared output.
//! - [`normal_form`] — the shapes that walk produces.
//! - [`subject`] — the identity-bearing projection and encoding of those shapes.
//! - [`verified`] — the admitted request and its per-target views.
//! - [`refusal`] — why any of the above said no.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::sync::{Mutex, OnceLock, PoisonError};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexRealizationLaw,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    AccessOrdinal, AxisDecode, LogicalAccess, PointwiseBf16Expression,
    PointwiseBf16ExpressionBuilder, PointwiseBf16Value, PointwiseF32Expression,
    PointwiseF32ExpressionBuilder, PointwiseF32Node, PointwiseF32Value, TensorRole,
    interpret_parametric_broadcast, mapping_names_a_symbol,
    parametric_broadcast_read_is_admissible,
};
use tiler_ir::semantic::{
    BF16_CONSTANT_BITS_ATTRIBUTE, BROADCAST_AXIS_MAPPING_ATTRIBUTE, Bf16, BroadcastAxisMapping,
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalIntegerWidth, CanonicalValueView,
    ContractionIndex, ContractionIndexStructure, F32, F32_CONSTANT_BITS_ATTRIBUTE, InputKey, OpKey,
    OperationAttributes, OutputKey, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE,
    REINDEX_MAPPING_ATTRIBUTE, ReindexForm, ReindexFormKind, ResolvedValueType, SemanticIdentity,
    SemanticProgram, TypeKey, ValueId, add_bf16_op, add_f32_op, broadcast_f32_op, constant_bf16_op,
    constant_f32_op, multiply_bf16_op, multiply_f32_op, reindex_f32_op, silu_f32_op,
    strict_serial_sum_f32_op, tensor_contraction_f32_op,
};
use tiler_ir::shape::{Axis, Extent, ExtentSources, Shape, SourcedExtent, SourcedShape};

// The numerical-realization vocabulary is target-neutral and owned by the shared
// IR (ADR 0070); the compiler contract references it rather than duplicating it.
pub(crate) use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, MaterializationRounding,
    NumericalPermission, SubnormalMode, ValueDomainProvenance,
};
use tiler_ir::schedule::{
    Bf16NumericalContractKey, F32NumericalContractKey, NumericalContractKeyError,
};

use crate::capability::{
    CanonicalLoweringRegistryIdentity, FrozenLoweringCapabilityRegistry,
    LoweringCapabilityRevision, LoweringCapabilitySubject,
};
use crate::elementary::{PointwiseExpressionSink, silu_point_body};
use crate::governed::{governed_lowering_capabilities, governed_scalars};
use crate::policy::UnrepresentableDimension;
use crate::region::{SemanticMemberId, SemanticStage};
use crate::target::DTypeDispatchabilityResolution;
use crate::target::honourability::{
    DeferredDimension, DimensionBehaviour, NumericalDimension, NumericalRequirement,
    UndeclaredDimension, UnhonouredDimension,
};
pub(crate) use crate::target::{TargetProfile, TargetProfileKey};

mod authority;
mod budget;
mod contract;
mod elementwise;
mod folded;
mod graph;
mod normal_form;
mod recognize;
mod refusal;
mod structural;
mod subject;
mod verified;
mod verify;

// One explicit import block per child, carrying each item at its own declared
// visibility rather than at the re-export's, so the surface reachable as
// `crate::request::_` is exactly what it was while this module was one file.
// Explicit names rather than globs because the workspace macro-boundary scan
// admits only parent globs: a non-parent glob could import an untracked macro
// name, so the spine enumerates what it carries. `budget` is the only child
// holding `pub` items — `crate::session` re-exports `BudgetRefusal` and
// `BudgetResource` onto the public surface. `pub(super)` items appear here as
// plain imports so siblings reach them through their own `use super::*`; a
// wider spelling would assert a reach no item has.
pub(crate) use authority::{
    CompilationRequest, CompilerCapabilitySnapshot, LoweringProviderIdentity,
};
use authority::{REQUEST_SCHEMA_VERSION, carries_program_environment};
pub(crate) use budget::DeterministicBudgets;
use budget::check_budget;
pub use budget::{BudgetRefusal, BudgetResource};
pub(crate) use contract::{
    ExceptionalValueDimensionKind, IncoherentContract, MAX_NUMERICAL_CONTRACT_PREFERENCES,
    NumericalContractPreference, StrictF32NumericalContract, coherence, contract_key_element_bytes,
};
use elementwise::{
    ElementwiseLeaves, ElementwiseRefusal, RecognizedElementwise, constant_family,
    declared_ordinal, plan_elementwise, recognize_elementwise, recognize_epilogue,
    recognize_pointwise,
};
use folded::{
    StagedOperandAdmission, materializes_its_result, normalize_contraction,
    recognize_epilogue_producer, recognize_reduction, recognize_staged_family,
};
use graph::{
    check_canonical_reduction_axes, constant_bits, element_count_u64, producer, producer_for_value,
    reduction_axes, sourced_shape, sourced_shape_ref, static_shape, static_shape_ref,
    unsupported_symbolic_extent,
};
pub(crate) use normal_form::{
    BoundaryRead, DeclaredInputOrdinal, NormalizedContraction, NormalizedContractionRead,
    NormalizedEpilogue, NormalizedOutput, NormalizedPointwise, NormalizedProgram,
    NormalizedSerialSum, NormalizedStaged, RecognizedPointwise, RecognizedSerialSumMembers,
};
pub(crate) use recognize::recognized_arithmetic;
use recognize::{recognized_program_arithmetic, select_supported_strategy};
pub(crate) use refusal::{ContractRejection, DTypeDispatchRefusalDisposition, RequestError};
use refusal::{mismatch, unsupported};
use structural::{is_structural_family, recognize_structural_read};
pub(crate) use subject::{
    NormalizedEpilogueSubject, NormalizedOutputSubject, NormalizedSerialSumSubject,
    VerifiedRequestSubject, permission_tag,
};
use subject::{VerifiedRequestAuthorities, request_subject};
pub(crate) use verified::{
    VerifiedCompilationRequest, VerifiedRequest, VerifiedTargetRequest, VerifiedTargetResolution,
    VerifiedTargetSlot,
};
pub(crate) use verify::verify_request;
use verify::{require_elementary_accuracy, verify_program};

// The `#[cfg(test)]` half of the carried surface: items the children gate to
// test builds, plus items whose only cross-module consumers are the test
// modules below. A glob carried these silently; explicit imports must state
// the gate or the lib build would deny them as unused.
#[cfg(test)]
pub(crate) use contract::is_f32_contract_key;
#[cfg(test)]
use contract::{canonical_contract_key, contract_key_arithmetic};
#[cfg(test)]
use recognize::{check_output_cover, published_and_consumed_overlap, recognize_program_outputs};
#[cfg(test)]
use subject::{
    PARAMETRIC_BROADCAST_ACCESS_TAG, UNREAD_DECLARED_INPUT_TAG, encode_access_relation,
    encode_elementwise_reads, encode_explain_shape, encode_output_subject, output_subject,
};
#[cfg(test)]
pub(crate) use verified::verify_planned_request;
#[cfg(test)]
use verify::{canonical_program_value_types, check_program_budgets, resolve_numerical_contract};

#[cfg(test)]
mod subject_budget;
#[cfg(test)]
mod tests;
