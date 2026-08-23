//! The pipeline's unit and integration tests.
//!
//! Split out of `pipeline.rs` so the module root reads as the compilation
//! story rather than as orchestration followed by sixteen hundred lines of
//! fixtures. `conformance` is deliberately a separate sibling: it drives the
//! public `compile()` entry point only, and mixing it in here would blur the
//! line between a test that may reach a stage-local constructor and one that
//! may not.
//!
//! # Mapping rule
//!
//! Split from one 10,138-line file
//! (`split-the-compiler-pipeline-test-monolith-by-orchestration-phase`) by
//! subject: each child module holds the tests for one compiled-program
//! property (a family of `compile()` calls sharing a fixture shape and an
//! observation being made about the result), plus any fixture used only by
//! that property. A fixture used by tests in more than one child lives in
//! [`support`] instead, so it is defined exactly once. `support` is not
//! organized by production module (`planning.rs`, `trace.rs`, `verify.rs`,
//! ...) because most fixtures here are reused across many of those -- the
//! interpretation harness (`KirMachine` and friends) backs nearly every
//! bit-for-bit reference comparison in the suite, regardless of which
//! compiler stage the comparison is really about.

use super::*;

use crate::explain::ExplainDisposition;
use crate::frontier::{PhysicalImplementationProvider, PhysicalProposalKind};
use crate::physical::{AccessOrdinal, RegionId, TensorRole};
use crate::request::{
    CompilerCapabilitySnapshot, NumericalContractPreference, StrictF32NumericalContract,
    TargetProfile,
};
use std::collections::BTreeMap;
use tiler_ir::kernel::{BinaryOp, CompareOp, ConvertOp, KernelConstant, OperationView, UnaryOp};
use tiler_ir::program::abi::{AvailabilityPhase, TargetPropertyRequirementRelation};
use tiler_ir::program::{DependencyReasonView, ValueRole};
use tiler_ir::schedule::RegionProgram;
use tiler_ir::semantic::{
    Bf16, Bf16Add, Bf16Constant, Bf16Multiply, CANONICAL_F32_ARITHMETIC_NAN_BITS, ContractionIndex,
    ContractionIndexStructure, F32, F32Add, F32Constant, F32Multiply, F32TensorContraction,
    InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

mod bf16;
mod contract_groups;
mod contraction_direct_path;
mod cover_completeness;
mod determinism;
mod epilogue_budgets;
mod explain_exhaustiveness;
mod explain_rendering;
mod frontier_census;
mod normalization_and_capability;
mod opaque_calls;
mod portfolio_verification;
mod produced_folds;
mod reduction_frontier_split;
mod reduction_strategy_selection;
mod reference_matching;
mod rejection_attribution;
mod search_budgets;
mod split_program_assembly;
mod staged_rms;
mod structured_body_matching;
mod support;
mod target_dispatch;
mod tree_feasibility;
mod tree_participant_matching;
mod workgroup_tree_admission;

// The six fixtures `conformance.rs` reaches through `super::tests::{name}` keep
// their original pub(super) visibility across the split: they are re-exported
// here at the same reach (`pipeline::tests::name`) rather than left reachable
// only one hop short of it.
pub(super) use support::{
    bits_of, f32_tensor, interpret_fused, interpret_fused_inputs, reduction_loop, tensor_bits,
};
