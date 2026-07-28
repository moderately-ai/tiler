//! Compiles and encodes one real artifact envelope.
//!
//! This is the expensive step the expansion cache exists to spare, and it is a
//! genuine one: it runs `tiler-compiler`'s governed session and encodes the
//! result through `tiler-artifact`, so bytes produced here are accepted by the
//! real `decode_artifact` that
//! [`ExpansionCache`](tiler_cache::expansion::ExpansionCache) validates every hit
//! against.
//!
//! # Why nothing here is memoized
//!
//! A `OnceLock` around the compilation would make repeat expansions inside one
//! long-lived proc-macro server cheap — and would hide the exact quantity this
//! spike measures. The cache's claim is that it suppresses duplicate
//! *compilation*; an in-process memo would suppress it first and report a
//! success the cache did not earn.
//!
//! # What it does not use
//!
//! No Metal toolchain and no device. `tiler-compiler` depends only on
//! `tiler-ir`, so the whole path is host computation and the spike runs anywhere
//! the workspace builds. The payload is declared by descriptor rather than
//! carried, which is what keeps a real compiled object out of the picture — the
//! envelope is assembled to be identified, which is all a cache key and a
//! validation pass need.

use tiler_artifact::program::{
    AbiExprId, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey, BackendEntryRef,
    BackendKey, BackendPayloadDescriptor, BindingKind, BindingSpec, CapabilityKey,
    CompilationEnvironment, EntrySpec, FeasibilityRuleSetKey, FeasibilityRuleSetRef, LaunchSpec,
    PayloadDigest, RepresentationKey, SchemaVersion, SelectedProvider,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, VariantSpec,
    VerifiedArtifactProgram,
};
use tiler_compiler::session::{
    Compilation, NumericalContract, PlanAlternative, compile_governed,
};
use tiler_ir::program::abi::ExprNode;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Rows of the exercised program's input.
const ROWS: u64 = 4;
/// Columns of the exercised program's input.
const COLUMNS: u64 = 3;

/// Compiles, assembles, and encodes one artifact envelope.
///
/// # Panics
///
/// Panics when the governed program fails to compile or the assembled artifact
/// fails to verify or encode. Each of those is a defect in this spike or in the
/// workspace it builds against, never a cache outcome, so failing loudly here
/// keeps it distinguishable from the cache falling open.
#[must_use]
pub fn encoded_envelope() -> Vec<u8> {
    let semantic = serial_sum_program(ROWS, COLUMNS);
    let compilations = compile_governed(&semantic, NumericalContract::FlushSubnormalsToZeroF32)
        .expect("the governed program compiles");
    let compilation = compilations.first().expect("one governed target profile");
    let plan = compilation.selected().expect("a selected plan alternative");
    assemble(&semantic, compilation, plan)
        .encode()
        .expect("the envelope encodes")
}

/// Builds `sum((input * 1.0) + 0.0)` over the reduced axis.
fn serial_sum_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
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
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Packages one plan alternative and a declared payload as an artifact.
fn assemble(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
) -> VerifiedArtifactProgram {
    let profile = TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())
            .expect("the compiler mints a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )
        .expect("the compiler mints a profile descriptor"),
    };
    let rules = FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())
            .expect("the compiler mints a governed rule-set key"),
        revision: compilation.feasibility_rule_set_revision(),
    };

    let environment = CompilationEnvironment::new(
        plan.selected_capabilities()
            .map(|selected| selected.provider().clone()),
    )
    .expect("the offered providers compose an environment");
    let mut builder =
        ArtifactProgramBuilder::new(semantic, environment).expect("a builder identity remains");
    for selected in plan.selected_capabilities() {
        builder
            .select_provider(SelectedProvider {
                provider: selected.provider().clone(),
                capability: CapabilityKey::new(selected.capability_key())
                    .expect("the compiler mints a governed capability key"),
                capability_revision: selected.capability_revision(),
            })
            .expect("a selected provider was offered");
    }

    let payload = builder
        .push_payload(BackendPayloadDescriptor {
            backend: BackendKey::new("tiler.metal").expect("a governed backend key"),
            representation: RepresentationKey::new("metallib")
                .expect("a governed representation key"),
            payload_schema: SchemaVersion::new(1, 0),
            digest: PayloadDigest::from_bytes([0xe1, 0xe2, 0xe3])
                .expect("a bounded payload digest"),
            compatibility: profile.clone(),
            execution_policy: ArtifactExecutionPolicy::RequiresDeviceTranslation,
        })
        .expect("the declared payload is accepted");

    let abi = plan.abi();
    let program = abi.kernel_program();
    let minted = replay(&mut builder, abi.expressions(), &variant_roots(plan));
    let resolve = |position: u32| {
        minted[usize::try_from(position).expect("a bounded arena position fits a usize")]
            .expect("every use site names a position the variant's own roots reach")
    };

    let entries: Vec<EntrySpec> = abi
        .entries()
        .zip(program.stages())
        .map(|(entry, stage)| EntrySpec {
            bindings: entry
                .accessible_bytes()
                .map(|position| BindingSpec {
                    kind: BindingKind::Buffer,
                    accessible_bytes: resolve(position),
                })
                .collect(),
            launch: LaunchSpec {
                grid_threads: resolve(entry.grid_threads()),
                threads_per_workgroup: resolve(entry.threads_per_workgroup()),
                // Not a choice: every verified scheduled region carries it.
                zero_work_skips_dispatch: true,
                preconditions: Vec::new(),
            },
            implementation: BackendEntryRef {
                payload,
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )
                .expect("the packaged kernel identity fits a backend entry key"),
            },
        })
        .collect();

    builder
        .push_variant(
            program,
            VariantSpec {
                applicability_guard: resolve(abi.applicability_guard()),
                target_profile: profile,
                feasibility_rules: rules,
                deferred_predicates: Vec::new(),
                entries,
            },
        )
        .expect("the variant packages the plan it was built from");
    builder.build().expect("the assembled artifact verifies")
}

/// The arena positions one variant's own use sites name.
fn variant_roots(plan: PlanAlternative<'_>) -> Vec<u32> {
    let abi = plan.abi();
    let mut roots = vec![abi.applicability_guard()];
    for entry in abi.entries() {
        roots.extend(entry.accessible_bytes());
        roots.push(entry.grid_threads());
        roots.push(entry.threads_per_workgroup());
    }
    roots
}

/// Transliterates the reachable sub-DAG of one arena onto the builder's own.
///
/// Pruned to the variant's roots because the artifact layer refuses an arena
/// node no use site reaches, and the compiler's canonical graph serves both plan
/// alternatives. One forward pass suffices: operands precede the node naming
/// them, and the reachable set is operand-closed.
fn replay(
    builder: &mut ArtifactProgramBuilder,
    arena: &[ExprNode],
    roots: &[u32],
) -> Vec<Option<AbiExprId>> {
    let reachable = reachable_from(arena, roots);
    let mut minted: Vec<Option<AbiExprId>> = vec![None; arena.len()];
    let resolve = |minted: &[Option<AbiExprId>], position: u32| {
        minted[usize::try_from(position).expect("a bounded arena position fits a usize")]
            .expect("an operand precedes the node naming it")
    };
    for (position, node) in arena.iter().enumerate() {
        if !reachable[position] {
            continue;
        }
        let id = match node {
            ExprNode::Root(root) => builder.push_root(root.clone()),
            ExprNode::Unary { op, operand } => builder.push_unary(*op, resolve(&minted, *operand)),
            ExprNode::Binary { op, left, right } => {
                builder.push_binary(*op, resolve(&minted, *left), resolve(&minted, *right))
            }
            ExprNode::Select {
                condition,
                if_true,
                if_false,
            } => builder.push_select(
                resolve(&minted, *condition),
                resolve(&minted, *if_true),
                resolve(&minted, *if_false),
            ),
        }
        .expect("a well-typed compiler expression replays onto the artifact arena");
        minted[position] = Some(id);
    }
    minted
}

/// Marks every arena position reachable from a set of use sites.
fn reachable_from(arena: &[ExprNode], roots: &[u32]) -> Vec<bool> {
    let mut reached = vec![false; arena.len()];
    let mut work: Vec<u32> = roots.to_vec();
    while let Some(node) = work.pop() {
        let at = usize::try_from(node).expect("a bounded arena position fits a usize");
        if reached[at] {
            continue;
        }
        reached[at] = true;
        match &arena[at] {
            ExprNode::Root(_) => {}
            ExprNode::Unary { operand, .. } => work.push(*operand),
            ExprNode::Binary { left, right, .. } => {
                work.push(*left);
                work.push(*right);
            }
            ExprNode::Select {
                condition,
                if_true,
                if_false,
            } => {
                work.push(*condition);
                work.push(*if_true);
                work.push(*if_false);
            }
        }
    }
    reached
}
