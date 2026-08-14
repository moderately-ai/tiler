//! The governed scale-then-reduce program and the compile helpers this spike uses.

use std::collections::BTreeSet;

use tiler_compiler::physical_provider::{
    InstalledPhysicalProviders, PhysicalImplementationProvider,
};
use tiler_compiler::session::{
    Compilation, CompileFailureClass, CompileRequest, NumericalContract, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// The numerical contract the decision ticket names as the governed strict
/// contract. Split and tree strategies remain declined under it because
/// reassociation is forbidden.
pub const CONTRACT: NumericalContract = NumericalContract::STRICT_F32;

/// Distinct region subjects `one_compile_enumerates_each_distinct_region_subject_once`
/// pins for this program. The harness re-measures rather than trusting the pin,
/// and the population check fails if the observed count moves.
pub const GOVERNED_FIVE_OP_DISTINCT_SUBJECTS: u64 = 17;

/// The governed scale-then-reduce program at one shape.
///
/// This is `crates/tiler-compiler/src/hot_path.rs`'s `program`, rebuilt against
/// the public semantic builder so the spike never reaches a crate-private
/// helper. Five operations: two constants, a multiply, an add, and a strict
/// serial sum.
#[must_use]
pub fn five_op_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("a valid input key"),
            Shape::from_dims([rows, columns]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum applies");
    builder
        .output(OutputKey::new("result").expect("a valid output key"), sum)
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// A one-multiply pointwise program used only as a perturbation subject.
///
/// Fewer occurrences than [`five_op_program`], so a check pinned at
/// [`GOVERNED_FIVE_OP_DISTINCT_SUBJECTS`] must fail when this is compiled in
/// its place.
#[must_use]
pub fn tiny_pointwise_program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("a valid input key"),
            Shape::from_dims([4, 3]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).expect("the scale applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    builder
        .output(
            OutputKey::new("result").expect("a valid output key"),
            product,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// A tensor add chain whose ordered regrouping creates a second semantic
/// candidate only for targets resolving a reassociating contract.
#[must_use]
pub fn tensor_add_chain() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("a valid input key"),
            Shape::from_dims([2, 2]),
        )
        .expect("the input binds");
    let first =
        F32Constant::apply(&mut builder, 1.0e20_f32.to_bits()).expect("the first constant applies");
    let second = F32Constant::apply(&mut builder, (-1.0e20_f32).to_bits())
        .expect("the second constant applies");
    let left = F32Add::apply(&mut builder, input, first).expect("the first add applies");
    let root = F32Add::apply(&mut builder, left, second).expect("the second add applies");
    builder
        .output(OutputKey::new("result").expect("a valid output key"), root)
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Outcome of one compile through the public session.
#[derive(Debug)]
pub struct Compiled {
    /// The public failure class, when planning refused.
    pub failure: Option<CompileFailureClass>,
    /// Rendered explain bytes retained on the successful or failed path.
    pub explain_bytes: usize,
    /// Retained complete-plan alternatives.
    pub alternatives: usize,
    /// Offered physical-provider identities, governed first.
    pub offered: usize,
    /// Distinct selected physical-provider explain subjects.
    pub selected_providers: usize,
    /// Target slots returned in caller order.
    pub target_keys: Vec<String>,
    /// Successful target slots.
    pub successes: usize,
    /// Resolved numerical contract key for every successful slot.
    pub resolved_contracts: Vec<String>,
}

/// Compiles `program` against `profile` and the stated physical environment.
pub fn compile_with(
    program: &SemanticProgram,
    profile: &TargetProfile,
    providers: &InstalledPhysicalProviders<'_>,
) -> Compiled {
    compile_request(program, [CONTRACT], [profile.clone()], providers)
}

/// Compiles one complete public request, preserving target order and resolving
/// every stated numerical-contract group inside the same compiler call.
pub fn compile_request(
    program: &SemanticProgram,
    contracts: impl IntoIterator<Item = NumericalContract>,
    profiles: impl IntoIterator<Item = TargetProfile>,
    providers: &InstalledPhysicalProviders<'_>,
) -> Compiled {
    let targets = TargetRequest::new(profiles).expect("the census target request is valid");
    let request = match CompileRequest::preferring(program, contracts, targets) {
        Ok(request) => request.with_physical_providers(providers.clone()),
        Err(failure) => return summarize_request_failure(&failure),
    };
    let batch = match compile(request) {
        Ok(batch) => batch,
        Err(failure) => return summarize_request_failure(&failure),
    };

    let mut summary = Compiled {
        failure: None,
        explain_bytes: 0,
        alternatives: 0,
        offered: 0,
        selected_providers: 0,
        target_keys: Vec::new(),
        successes: 0,
        resolved_contracts: Vec::new(),
    };
    let mut selected = BTreeSet::new();
    for slot in batch.into_targets() {
        let (profile, outcome) = slot.into_parts();
        summary
            .target_keys
            .push(profile.profile_key().as_str().to_owned());
        match outcome {
            Ok(compilation) => {
                let one = summarize_ok(&compilation);
                summary.successes += 1;
                summary.explain_bytes += one.explain_bytes;
                summary.alternatives += one.alternatives;
                summary.offered = summary.offered.max(one.offered);
                summary
                    .resolved_contracts
                    .push(compilation.resolved_numerical_contract_key().to_owned());
                selected.extend(compilation.alternatives().flat_map(|alternative| {
                    alternative
                        .selected_physical_providers()
                        .map(|provider| provider.provider_explain_subject().to_owned())
                        .collect::<Vec<_>>()
                }));
            }
            Err(failure) => {
                summary.failure.get_or_insert(failure.class());
                summary.explain_bytes +=
                    failure.explain().map_or(0, |report| report.render().len());
            }
        }
    }
    summary.selected_providers = selected.len();
    summary
}

/// Compiles with only the governed physical provider.
pub fn compile_governed_only(program: &SemanticProgram, profile: &TargetProfile) -> Compiled {
    compile_with(program, profile, &InstalledPhysicalProviders::governed())
}

/// Compiles with caller-installed providers beside the governed one.
pub fn compile_installed<'a>(
    program: &SemanticProgram,
    profile: &TargetProfile,
    installed: impl IntoIterator<Item = &'a dyn PhysicalImplementationProvider>,
) -> Compiled {
    let environment = InstalledPhysicalProviders::installed(installed)
        .expect("the stated identities install beside the governed provider");
    compile_with(program, profile, &environment)
}

fn summarize_ok(compilation: &Compilation) -> Compiled {
    let explain_bytes = compilation.explain().render().len();
    let alternatives = compilation.alternatives().len();
    let offered = compilation.offered_physical_providers().len();
    let selected_providers = compilation
        .alternatives()
        .flat_map(|alternative| {
            alternative
                .selected_physical_providers()
                .map(|selected| selected.provider_explain_subject().to_owned())
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>()
        .len();
    Compiled {
        failure: None,
        explain_bytes,
        alternatives,
        offered,
        selected_providers,
        target_keys: vec![
            compilation
                .target_profile()
                .profile_key()
                .as_str()
                .to_owned(),
        ],
        successes: 1,
        resolved_contracts: vec![compilation.resolved_numerical_contract_key().to_owned()],
    }
}

fn summarize_request_failure(failure: &tiler_compiler::session::CompileFailure) -> Compiled {
    Compiled {
        failure: Some(failure.class()),
        explain_bytes: failure.explain().map_or(0, |report| report.render().len()),
        alternatives: 0,
        offered: 0,
        selected_providers: 0,
        target_keys: Vec::new(),
        successes: 0,
        resolved_contracts: Vec::new(),
    }
}
