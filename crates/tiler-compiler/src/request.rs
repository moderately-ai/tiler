#![allow(
    clippy::wildcard_imports,
    reason = "every child here is one layer of `request`, not a separate concept: \
the module was one file, its layers share one import surface, and each child \
globs that surface from the root exactly as `pipeline`'s children do. \
Enumerating the parent's imports per child would restate the same forty names \
thirteen times and would have to be restated again on every change. The globs \
are scoped to one parent whose contents sit in the same directory, and the \
root's own re-export globs carry each item at its own declared visibility"
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

// One glob per child, carrying each item at its own declared visibility rather
// than at the re-export's, so the surface reachable as `crate::request::_` is
// exactly what it was while this module was one file. `budget` is the only child
// holding `pub` items — `crate::session` re-exports `BudgetRefusal` and
// `BudgetResource` onto the public surface — and `elementwise`, `folded`,
// `graph`, and `structural` hold nothing above `pub(super)`, so their globs are
// plain imports: a wider spelling there would assert a reach no item has.
pub(crate) use authority::*;
pub use budget::*;
pub(crate) use contract::*;
use elementwise::*;
use folded::*;
use graph::*;
pub(crate) use normal_form::*;
pub(crate) use recognize::*;
pub(crate) use refusal::*;
use structural::*;
pub(crate) use subject::*;
pub(crate) use verified::*;
pub(crate) use verify::*;

#[cfg(test)]
mod subject_budget;
#[cfg(test)]
mod tests;
