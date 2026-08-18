//! Transactional builder for the target-neutral artifact program model.
//!
//! Construction follows the ADR 0071 discipline: a public transactional builder
//! with private storage, insertion-time checks for every locally decidable
//! invariant, and a consuming [`ArtifactProgramBuilder::build`] that runs
//! whole-artifact verification and returns an opaque
//! [`VerifiedArtifactProgram`] or the intact builder with typed diagnostics.
//!
//! Two things are taken rather than declared, so a producer cannot assert them.
//!
//! The **semantic subject** comes from a verified
//! [`SemanticProgram`]: its four-subject identity bundle and its ordered named
//! interface. Every packaged variant must realize that exact graph.
//!
//! The **program ABI** comes from the variant's own verified program and kernels: the builder replays the program's expression arena and derives the applicability guard, launch geometry, accessible byte offset and extent, binding target, element type, address space, access mode, alignment, and program role. A producer supplies only artifact-owned choices: deferred predicates, launch preconditions and zero-work policy, binding transport kinds, target and feasibility references, and backend entry selection.

use tiler_ir::program::abi::{PreparedEntryTargetRequirement, TargetPropertyRequirementRelation};
use tiler_ir::program::{
    MaterializedOrigin, MaterializedValueRef, StageRef, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{NumericalRealization, TensorRole};
use tiler_ir::semantic::{
    InputKey, OutputKey, ProviderIdentity, ResolvedValueType, SemanticIdentity, SemanticProgram,
};
use tiler_ir::shape::{Axis, Shape};

use tiler_ir::kernel::PlanDeterminismWitness;

use super::codec::{ArtifactEnvelope, PayloadContent, PayloadMetadata};
use super::environment::{
    PayloadPlanDeterminismReceipt, PlanDeterminismScope, TargetEnvironmentDeclaration,
};
use super::error::{
    AbiExprUse, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind, ArtifactLimitKind,
    ArtifactVerificationError, invalid_handle, limit,
};
use super::expr::{
    AbiBinaryOp, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue, AvailabilityPhase, ExprNode,
    binary_operand_type, evaluate, node_is_interface_only, node_phase, node_type,
    unary_operand_type,
};
use super::facts::AbiFactBinder;
use super::handles::{
    AbiExprId, ArtifactBuilderId, PayloadId, VariantId, next_artifact_builder_id,
};
use super::keys::{BackendKey, FeasibilityRuleSetRef, RepresentationKey, TargetProfileRef};
use super::model::{
    ArtifactExecutionPolicy, ArtifactProgramData, ArtifactSchema, BackendEntryRef,
    BackendPayloadDescriptor, BindingData, BindingKind, BindingTargetData, DeferredPredicateData,
    EntryData, ExtentOperandData, InterfaceComponentData, InterfaceEntryData, LaunchData,
    RoutingPolicy, SchemaVersion, SelectedProvider, StoredBackendEntry, VariantData,
    VerifiedArtifactProgram, encode_identity, packaged_entry_positions,
};
use super::realization::DeliveredRealizationRecord;
use super::requirement::RouteRequirement;
use super::{
    MAX_ABI_EXPRESSIONS, MAX_ARTIFACT_PAYLOADS, MAX_ARTIFACT_VARIANTS, MAX_DEFERRED_PREDICATES,
    MAX_DELIVERY_POSITIONS, MAX_ENTRY_BINDINGS, MAX_ENTRY_EXTENTS, MAX_ENVIRONMENT_PROVIDERS,
    MAX_LAUNCH_PRECONDITIONS, MAX_ROUTE_REQUIREMENTS, MAX_SELECTED_PROVIDERS, MAX_VARIANT_ENTRIES,
};

/// The complete frozen set of capability providers offered to one compilation.
///
/// This is a construction-time authority, not artifact content. It exists so
/// that a selected provider can be proven to have actually been offered; the
/// providers it lists but the plan never reaches are deliberately **not**
/// retained by the verified artifact and can therefore never enter its
/// identity (ADR 0072). Keeping the complete registry snapshot is a
/// compilation-request concern that lives outside the runtime artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilationEnvironment {
    available: Vec<ProviderIdentity>,
}

impl CompilationEnvironment {
    /// Freezes the providers offered to one compilation.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::StructuralLimit`] beyond the governed
    /// environment bound.
    pub fn new(
        providers: impl IntoIterator<Item = ProviderIdentity>,
    ) -> Result<Self, ArtifactBuildError> {
        let mut available: Vec<ProviderIdentity> = providers.into_iter().collect();
        limit(
            available.len(),
            MAX_ENVIRONMENT_PROVIDERS,
            ArtifactLimitKind::EnvironmentProviders,
        )?;
        available.sort_unstable_by(|left, right| {
            (left.namespace(), left.name(), left.revision()).cmp(&(
                right.namespace(),
                right.name(),
                right.revision(),
            ))
        });
        available.dedup();
        Ok(Self { available })
    }

    /// Returns the offered providers in canonical order.
    #[must_use]
    pub fn available(&self) -> &[ProviderIdentity] {
        &self.available
    }

    fn offers(&self, provider: &ProviderIdentity) -> bool {
        self.available.contains(provider)
    }
}

/// The unforgeable ordered semantic interface every packaged variant realizes.
#[derive(Clone, Debug)]
struct SemanticInterface {
    inputs: Vec<(InputKey, Shape, ResolvedValueType)>,
    outputs: Vec<(OutputKey, Shape, ResolvedValueType)>,
}

/// The portfolio-wide facts the first packaged variant establishes.
#[derive(Clone, Debug)]
struct PortfolioSubject {
    inputs: Vec<InterfaceEntryData<InputKey>>,
    outputs: Vec<InterfaceEntryData<OutputKey>>,
    numerical: NumericalRealization,
    profile: TargetProfileRef,
}

/// One deferred feasibility predicate a plan variant declares.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeferredPredicateSpec {
    /// Complete target-property requirement the builder turns into a predicate.
    pub requirement: PreparedEntryTargetRequirement,
    /// Declared program-stage ordinal whose prepared entry is queried.
    pub entry: u32,
}

/// One ABI binding a producer declares for an executable entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingSpec {
    /// Transport category of the binding.
    pub kind: BindingKind,
}

/// The launch contract a producer declares for an executable entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchSpec {
    /// Whether a zero-thread launch skips the dispatch entirely.
    pub zero_work_skips_dispatch: bool,
    /// Launch-instance preconditions evaluated before routing commits.
    pub preconditions: Vec<AbiExprId>,
}

/// One executable entry a producer declares for a program stage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntrySpec {
    /// ABI bindings in kernel buffer-parameter order.
    pub bindings: Vec<BindingSpec>,
    /// Launch contract of the entry.
    pub launch: LaunchSpec,
    /// Backend entry that realizes this entry.
    pub implementation: BackendEntryRef,
}

/// One complete plan variant a producer declares.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantSpec {
    /// Declared target profile the variant was assessed against.
    pub target_profile: TargetProfileRef,
    /// Feasibility rule set the variant was assessed under.
    pub feasibility_rules: FeasibilityRuleSetRef,
    /// Deferred feasibility predicates, each with its query authority.
    pub deferred_predicates: Vec<DeferredPredicateSpec>,
    /// Executable entries, one per stage of the variant's program.
    pub entries: Vec<EntrySpec>,
}

/// The variant ABI derived from the bound program.
///
/// `adopted` maps each program arena position to the handle `adopt_abi`
/// replayed it as; `offsets` holds the per-stage, per-access window-offset
/// literals `adopt_offsets` minted. They are one value because they answer one
/// question — what the program's own ABI says — and every entry check reads
/// both.
struct DerivedAbi {
    adopted: Vec<Option<AbiExprId>>,
    offsets: Vec<Vec<AbiExprId>>,
}

/// A transactional artifact-program builder with private storage.
#[derive(Clone, Debug)]
pub struct ArtifactProgramBuilder {
    owner: ArtifactBuilderId,
    semantic: SemanticIdentity,
    retained: super::retained::RetainedShapeEnvironment,
    interface: SemanticInterface,
    environment: CompilationEnvironment,
    providers: Vec<SelectedProvider>,
    payloads: Vec<BackendPayloadDescriptor>,
    payload_content: Vec<Option<PayloadContent>>,
    expressions: Vec<ExprNode>,
    /// Arena position of every node already interned, keyed by the node itself.
    interned: std::collections::HashMap<ExprNode, usize>,
    expression_types: Vec<AbiType>,
    expression_phases: Vec<AvailabilityPhase>,
    expression_interface_only: Vec<bool>,
    variants: Vec<VariantData>,
    /// The delivered-realization record a producer declared, if it has yet.
    ///
    /// The one `Option` in the wiring, and it is the *draft's* state rather than
    /// the artifact's: a transactional builder is incomplete until it is built,
    /// and [`Self::build`] turns the absence into
    /// [`ArtifactDiagnostic::MissingDeliveredRealization`] rather than into a
    /// third state the product carries.
    ///
    /// Held in the producer's **declared** entry space; `build` remaps it once.
    ///
    /// [`ArtifactDiagnostic::MissingDeliveredRealization`]: super::ArtifactDiagnostic::MissingDeliveredRealization
    realization: Option<DeliveredRealizationRecord>,
    subject: Option<PortfolioSubject>,
    /// Delivery positions the first accepted entry established, if any.
    ///
    /// `None` until a variant is accepted, so the first entry declared decides
    /// the count and every later one is measured against it rather than against
    /// a number this builder chose.
    delivery_positions: Option<usize>,
    routing: RoutingPolicy,
}

impl ArtifactProgramBuilder {
    /// Opens a builder bound to one verified semantic program and environment.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::BuilderIdentityExhausted`] when no fresh
    /// builder ownership identity remains.
    pub fn new(
        semantic: &SemanticProgram,
        environment: CompilationEnvironment,
    ) -> Result<Self, ArtifactBuildError> {
        let interface = read_semantic_interface(semantic)?;
        let owner =
            next_artifact_builder_id().ok_or(ArtifactBuildError::BuilderIdentityExhausted)?;
        Ok(Self {
            owner,
            semantic: semantic.semantic_identity().clone(),
            retained: super::retained::RetainedShapeEnvironment::project(semantic)?,
            interface,
            environment,
            providers: Vec::new(),
            payloads: Vec::new(),
            payload_content: Vec::new(),
            expressions: Vec::new(),
            interned: std::collections::HashMap::new(),
            expression_types: Vec::new(),
            expression_phases: Vec::new(),
            expression_interface_only: Vec::new(),
            variants: Vec::new(),
            realization: None,
            subject: None,
            delivery_positions: None,
            routing: RoutingPolicy::StablePriority,
        })
    }

    /// Records one capability provider the packaged plan actually reached.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::ProviderNotAvailable`] when the
    /// compilation environment never offered the provider,
    /// [`ArtifactBuildError::DuplicateSelectedProvider`] for a repeated
    /// selection, or a structural-limit error.
    pub fn select_provider(
        &mut self,
        selected: SelectedProvider,
    ) -> Result<(), ArtifactBuildError> {
        if !self.environment.offers(&selected.provider) {
            return Err(ArtifactBuildError::ProviderNotAvailable {
                provider: Box::new(selected.provider),
            });
        }
        if self.providers.contains(&selected) {
            return Err(ArtifactBuildError::DuplicateSelectedProvider {
                provider: Box::new(selected.provider),
            });
        }
        limit(
            self.providers.len().saturating_add(1),
            MAX_SELECTED_PROVIDERS,
            ArtifactLimitKind::SelectedProviders,
        )?;
        self.providers.push(selected);
        Ok(())
    }

    /// Declares one backend payload descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::DuplicatePayload`] for an identical
    /// descriptor, or a structural-limit error.
    pub fn push_payload(
        &mut self,
        descriptor: BackendPayloadDescriptor,
    ) -> Result<PayloadId, ArtifactBuildError> {
        if self.payloads.contains(&descriptor) {
            return Err(ArtifactBuildError::DuplicatePayload);
        }
        limit(
            self.payloads.len().saturating_add(1),
            MAX_ARTIFACT_PAYLOADS,
            ArtifactLimitKind::Payloads,
        )?;
        let id = PayloadId::from_len(self.owner, self.payloads.len()).ok_or(
            ArtifactBuildError::StructuralLimit {
                resource: ArtifactLimitKind::Payloads,
                actual: self.payloads.len().saturating_add(1),
                limit: MAX_ARTIFACT_PAYLOADS,
            },
        )?;
        self.payloads.push(descriptor);
        self.payload_content.push(None);
        Ok(id)
    }

    /// Declares the payload a backend compilation that has not run will produce.
    ///
    /// # Naming an object before it exists
    ///
    /// A [`PayloadMetadata`] is the compilation's *subject* — its source, its
    /// flags, its resolved toolchain, its entry mappings, its recorded target
    /// obligations — and the descriptor's content digest is derived from
    /// exactly those bytes, never from the emitted object. Every one of them is
    /// settled before the compiler is invoked, so this call is available on a
    /// cache *miss*: an artifact assembled through it carries the exact
    /// [`CanonicalArtifactProgramIdentity`](super::CanonicalArtifactProgramIdentity)
    /// the compiled artifact will carry, which is what lets a caller derive the
    /// key it will file the compiled result under before it pays for the
    /// compilation.
    ///
    /// [`Self::push_carried_payload`] is the same declaration once the object
    /// has arrived, and it delegates here rather than repeating the descriptor,
    /// so a pending payload and the carried payload of the same compilation
    /// cannot name two different backend payloads.
    ///
    /// # What the product is for
    ///
    /// The artifact this yields is *descriptor-only*: it names a backend object
    /// it does not contain. It is assembled to be identified rather than
    /// published. A descriptor-only payload carries no entry mapping, so
    /// `check_entry_mappings` cannot prove the artifact's executable entries
    /// reach a symbol — the obligation a carried payload does discharge — and
    /// publishing one would announce an object no compilation produced.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::DuplicatePayload`] for a descriptor this
    /// artifact already declares, a structural-limit error, or the identity
    /// error the digest constructor produced.
    #[allow(
        clippy::too_many_arguments,
        reason = "the accepted ADR 0013 carrier adds the optional target-environment declaration to the exact accepted declaration shape; a parameter struct would be a second spelling of `BackendPayloadDescriptor`"
    )]
    pub fn push_pending_payload(
        &mut self,
        backend: BackendKey,
        representation: RepresentationKey,
        payload_schema: SchemaVersion,
        compatibility: TargetProfileRef,
        execution_policy: ArtifactExecutionPolicy,
        environment: Option<TargetEnvironmentDeclaration>,
        metadata: &PayloadMetadata,
    ) -> Result<PayloadId, ArtifactBuildError> {
        self.push_payload(BackendPayloadDescriptor {
            backend,
            representation,
            payload_schema,
            digest: metadata.identity()?,
            compatibility,
            execution_policy,
            environment,
        })
    }

    /// Declares one backend payload and carries its content in the artifact.
    ///
    /// The compatibility contract is supplied rather than derived. The carried
    /// provenance names a backend-specific target string; `TargetProfileRef` is
    /// the neutral governed profile the artifact layer reasons about, and only
    /// the assembler knows which profile it compiled against.
    ///
    /// The descriptor's content digest is not supplied: it is *derived* from
    /// the exact canonical payload-metadata bytes, so a carried payload cannot
    /// claim a compilation subject other than the one it carries. That is the
    /// identity decision this layer encodes — a payload is content-addressed
    /// over its compilation inputs, and the emitted object travels opaquely
    /// under an integrity digest that artifact identity deliberately excludes.
    ///
    /// The descriptor is therefore the one [`Self::push_pending_payload`] would
    /// have built from the same subject, and this delegates to it so there is
    /// one construction rather than two that agree by inspection. The object is
    /// what this call adds, and the object is the part identity excludes.
    ///
    /// # Errors
    ///
    /// Returns the errors [`Self::push_pending_payload`] returns.
    #[allow(
        clippy::too_many_arguments,
        reason = "mirrors `push_pending_payload`, whose reasoned allow above is the shared rationale"
    )]
    pub fn push_carried_payload(
        &mut self,
        backend: BackendKey,
        representation: RepresentationKey,
        payload_schema: SchemaVersion,
        compatibility: TargetProfileRef,
        execution_policy: ArtifactExecutionPolicy,
        environment: Option<TargetEnvironmentDeclaration>,
        content: PayloadContent,
    ) -> Result<PayloadId, ArtifactBuildError> {
        let id = self.push_pending_payload(
            backend,
            representation,
            payload_schema,
            compatibility,
            execution_policy,
            environment,
            &content.metadata,
        )?;
        let position = self.payloads.len() - 1;
        self.payload_content[position] = Some(content);
        Ok(id)
    }

    /// Replays a verified program's ABI arena onto this builder's arena.
    ///
    /// Returns one slot per source position, `Some` exactly for the positions
    /// reached from `roots`. The builder deduplicates by content, so two source
    /// positions holding the same expression resolve to one handle — which is
    /// why this is a position map rather than a compacted list.
    ///
    /// # Why this belongs to the artifact layer
    ///
    /// A variant's runtime ABI must be the ABI of the program it carries. Until
    /// now the only consumer that achieved that did it by hand, in
    /// `prototypes/serial-sum-compile`, which made it a producer convention
    /// rather than a checked one: an assembler could package a variant whose
    /// accessible range is `UnsignedLiteral(24)` over a program whose own range
    /// is `rows * columns * 4`, and both would verify, because each is checked
    /// against the same third value and neither against the other. Under static
    /// shapes those coincide; under dynamic shapes they need not, and the
    /// artifact's expression is the one a runtime evaluates while the program's
    /// is the one identity folds.
    ///
    /// `bind-the-artifact-variant-abi-to-the-program-abi` completed the move: [`Self::push_variant`] always uses this replay for the guard, launch, and accessible extents, while offsets are minted from the program's byte windows. No caller-supplied field can restate those facts.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::ExpressionOutOfRange`] when a root or an
    /// operand names a position outside `arena`, and any structural-limit error
    /// the underlying pushes return.
    pub fn adopt_abi(
        &mut self,
        arena: &[ExprNode],
        roots: &[u32],
    ) -> Result<Vec<Option<AbiExprId>>, ArtifactBuildError> {
        let reachable = Self::reachable_from(arena, roots)?;
        let mut minted: Vec<Option<AbiExprId>> = vec![None; arena.len()];
        // Source order, so an operand is always minted before the node naming
        // it: a verified arena stores operands ahead of their users.
        for (at, node) in arena.iter().enumerate() {
            if !reachable[at] {
                continue;
            }
            let id = match node {
                ExprNode::Root(root) => self.push_root(root.clone())?,
                ExprNode::Unary { op, operand } => {
                    self.push_unary(*op, Self::resolve(&minted, *operand)?)?
                }
                ExprNode::Binary { op, left, right } => self.push_binary(
                    *op,
                    Self::resolve(&minted, *left)?,
                    Self::resolve(&minted, *right)?,
                )?,
                ExprNode::Select {
                    condition,
                    if_true,
                    if_false,
                } => self.push_select(
                    Self::resolve(&minted, *condition)?,
                    Self::resolve(&minted, *if_true)?,
                    Self::resolve(&minted, *if_false)?,
                )?,
            };
            minted[at] = Some(id);
        }
        Ok(minted)
    }

    /// Mints each stage access's window offset as its canonical literal.
    ///
    /// The program states an *expression* for the accessible extent and only a
    /// concrete [`ByteWindow`] for where the range starts, so the offset a
    /// binding row carries is derived from the bound program the way the
    /// guard, launch, and extent are — a producer cannot restate it. It stays
    /// an expression on the row rather than a plain number so a program that
    /// one day computes its window offset can carry that formula without a
    /// schema step.
    ///
    /// [`ByteWindow`]: tiler_ir::program::ByteWindow
    fn adopt_offsets(
        &mut self,
        program: &VerifiedKernelProgram,
    ) -> Result<Vec<Vec<AbiExprId>>, ArtifactBuildError> {
        let mut offsets = Vec::with_capacity(program.stages().len());
        for stage in program.stages() {
            let mut row = Vec::new();
            for access in stage.accesses() {
                row.push(self.push_root(AbiRoot::UnsignedLiteral(access.view().window().offset))?);
            }
            offsets.push(row);
        }
        Ok(offsets)
    }

    /// Marks every arena position reachable from a set of use sites.
    fn reachable_from(arena: &[ExprNode], roots: &[u32]) -> Result<Vec<bool>, ArtifactBuildError> {
        let mut reached = vec![false; arena.len()];
        let mut work: Vec<u32> = roots.to_vec();
        while let Some(node) = work.pop() {
            let at = usize::try_from(node).expect("u32 fits every supported host usize");
            if at >= arena.len() {
                return Err(ArtifactBuildError::ExpressionOutOfRange { position: node });
            }
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
        Ok(reached)
    }

    /// Resolves a source position to the handle it was replayed as.
    fn resolve(
        minted: &[Option<AbiExprId>],
        position: u32,
    ) -> Result<AbiExprId, ArtifactBuildError> {
        let at = usize::try_from(position).expect("u32 fits every supported host usize");
        minted
            .get(at)
            .copied()
            .flatten()
            .ok_or(ArtifactBuildError::ExpressionOutOfRange { position })
    }

    /// Declares one typed root fact of the shared ABI expression arena.
    ///
    /// The arena is canonically deduplicated: an identical expression returns
    /// the handle already minted for it, so the arena stays a function of
    /// content rather than of how often a producer rebuilt the same formula.
    ///
    /// # Errors
    ///
    /// Returns a structural-limit error.
    pub fn push_root(&mut self, root: AbiRoot) -> Result<AbiExprId, ArtifactBuildError> {
        self.push_node(ExprNode::Root(root))
    }

    /// Declares one unary operation over an existing expression.
    ///
    /// # Errors
    ///
    /// Returns a handle error, [`ArtifactBuildError::OperandType`] for a
    /// mistyped operand, or a structural-limit error.
    pub fn push_unary(
        &mut self,
        op: AbiUnaryOp,
        operand: AbiExprId,
    ) -> Result<AbiExprId, ArtifactBuildError> {
        let operand = self.resolve_expression(operand)?;
        self.expect_type(operand, unary_operand_type(op))?;
        self.push_node(ExprNode::Unary { op, operand })
    }

    /// Declares one binary operation over two existing expressions.
    ///
    /// # Errors
    ///
    /// Returns a handle error, [`ArtifactBuildError::OperandType`] for a
    /// mistyped operand, or a structural-limit error.
    pub fn push_binary(
        &mut self,
        op: AbiBinaryOp,
        left: AbiExprId,
        right: AbiExprId,
    ) -> Result<AbiExprId, ArtifactBuildError> {
        let left = self.resolve_expression(left)?;
        let right = self.resolve_expression(right)?;
        self.expect_type(left, binary_operand_type(op))?;
        self.expect_type(right, binary_operand_type(op))?;
        self.push_node(ExprNode::Binary { op, left, right })
    }

    /// Declares one conditional selection between two equally typed branches.
    ///
    /// Only the selected branch is evaluated, so a branch that would fail on a
    /// zero-sized bound is legal behind a guarding condition.
    ///
    /// # Errors
    ///
    /// Returns a handle error, [`ArtifactBuildError::OperandType`] for a
    /// non-predicate condition, [`ArtifactBuildError::SelectBranchType`] for
    /// disagreeing branches, or a structural-limit error.
    pub fn push_select(
        &mut self,
        condition: AbiExprId,
        if_true: AbiExprId,
        if_false: AbiExprId,
    ) -> Result<AbiExprId, ArtifactBuildError> {
        let condition = self.resolve_expression(condition)?;
        let if_true = self.resolve_expression(if_true)?;
        let if_false = self.resolve_expression(if_false)?;
        self.expect_type(condition, AbiType::Boolean)?;
        let (left, right) = (
            self.expression_types[usize_of(if_true)],
            self.expression_types[usize_of(if_false)],
        );
        if left != right {
            return Err(ArtifactBuildError::SelectBranchType {
                if_true: left,
                if_false: right,
            });
        }
        self.push_node(ExprNode::Select {
            condition,
            if_true,
            if_false,
        })
    }

    /// Declares one complete plan variant at the next routing rank.
    ///
    /// Every deferred predicate is minted from the complete typed
    /// prepared-entry requirement supplied by the producer. Its acquisition
    /// provider is part of that requirement rather than an artifact-selected
    /// lowering provider. This builder validates the assertion structurally; it
    /// cannot authenticate that an arbitrary caller copied the requirement and
    /// entry association from a compiler plan. The ordinary `tiler-build` path
    /// owns that stronger producer guarantee.
    ///
    /// # Errors
    ///
    /// Returns a handle error; a semantic-subject, interface, numerical, or
    /// target-profile disagreement; an entry, delivery, or binding cardinality
    /// error; an
    /// expression type, root-phase, or interface-root rejection; an accessible
    /// range, launch, or zero-work disagreement with the bound program; a
    /// kernel specialized on a bound input extent; a
    /// duplicate variant, deferred predicate, or launch precondition; a
    /// deferred requirement bound outside the program's entry range; or a
    /// structural-limit error.
    pub fn push_variant(
        &mut self,
        program: &VerifiedKernelProgram,
        spec: VariantSpec,
    ) -> Result<VariantId, ArtifactBuildError> {
        if program.semantic_graph_identity() != self.semantic.graph() {
            return Err(ArtifactBuildError::SemanticSubjectMismatch);
        }
        limit(
            self.variants.len().saturating_add(1),
            MAX_ARTIFACT_VARIANTS,
            ArtifactLimitKind::Variants,
        )?;
        let stages = program.stages().len();
        if stages != spec.entries.len() {
            return Err(ArtifactBuildError::EntryCardinality {
                expected: stages,
                actual: spec.entries.len(),
            });
        }
        limit(stages, MAX_VARIANT_ENTRIES, ArtifactLimitKind::Entries)?;
        // The program's own ABI, replayed onto this builder's arena. Every
        // expression below is taken from it rather than restated by a caller,
        // which is what makes a variant's runtime ABI *be* its program's rather
        // than merely agree with it at one value.
        let program_roots = program_abi_use_sites(program);
        let adopted = self.adopt_abi(program.abi_expressions(), &program_roots)?;
        let derived_guard = Self::resolve(&adopted, program.applicability_guard())?;
        let guard = self.check_use(
            derived_guard,
            AbiExprUse::ApplicabilityGuard,
            AbiType::Boolean,
            AvailabilityPhase::LiveDevicePreflight,
            false,
        )?;
        let deferred = self.check_deferred(&spec.deferred_predicates, stages)?;
        let derived = DerivedAbi {
            offsets: self.adopt_offsets(program)?,
            adopted,
        };
        let facts = static_facts(program);
        let entries = self.check_entries(program, &spec.entries, &facts, &derived)?;
        let subject = self.check_subject(program, &spec)?;
        let id = VariantId::from_len(self.owner, self.variants.len()).ok_or(
            ArtifactBuildError::StructuralLimit {
                resource: ArtifactLimitKind::Variants,
                actual: self.variants.len().saturating_add(1),
                limit: MAX_ARTIFACT_VARIANTS,
            },
        )?;
        if self.variants.iter().any(|existing| {
            existing.program.canonical_identity() == program.canonical_identity()
                && existing.guard == guard
        }) {
            return Err(ArtifactBuildError::DuplicateVariant);
        }
        self.subject = Some(subject);
        // Established only now, after every refusal above has passed: a variant
        // this call rejects must leave the artifact's delivery-position count
        // exactly where it found it, so a caller amending and retrying is not
        // measured against a count a refused variant set.
        if let Some(entry) = entries.first() {
            self.delivery_positions = Some(entry.implementation.payloads.len());
        }
        // Every cell opens `Unclaimed`. Only the proof-bound
        // [`Self::publish_plan`] can flip one, so no spec field, bool, or raw
        // declaration reaches a determinism claim.
        let scope = vec![
            PlanDeterminismScope::Unclaimed;
            entries
                .first()
                .map_or(0, |entry| entry.implementation.payloads.len())
        ];
        self.variants.push(VariantData {
            program: program.clone(),
            guard,
            profile: spec.target_profile,
            feasibility_rules: spec.feasibility_rules,
            deferred,
            route_requirements: Vec::new(),
            entries,
            scope,
        });
        Ok(id)
    }

    /// Declares one additional requirement the variant's route places on a live device.
    ///
    /// # Why this is a second call rather than a [`VariantSpec`] field
    ///
    /// A deferred predicate is minted *with* the plan, by the compiler that
    /// chose it, which is why it arrives in the spec. A route requirement is
    /// not: it states what the **emitted payload** consumes, and that is known
    /// only after backend emission, to a different producer stage from the one
    /// that assembles a variant. Taking it here lets that stage declare what it
    /// alone knows without restating a plan it did not make.
    ///
    /// # What this cannot decide
    ///
    /// Whether a requirement is *missing*. Zero rows is a legal, explicit state,
    /// so an omitted row and a route that genuinely needs none are the same
    /// artifact from here. Only a producer-owned exhaustive declaration of what
    /// the selected payload uses can tell them apart, and no such declaration
    /// reaches this layer. Everything that *is* locally decidable is decided:
    /// the row's own validity by its constructor, and subject uniqueness here.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::ForeignHandle`] or
    /// [`ArtifactBuildError::InvalidHandle`] for a variant this builder did not
    /// mint, [`ArtifactBuildError::DuplicateRouteRequirementSubject`] when the
    /// variant already constrains that subject, or
    /// [`ArtifactBuildError::StructuralLimit`] beyond
    /// [`MAX_ROUTE_REQUIREMENTS`].
    pub fn require_route(
        &mut self,
        variant: VariantId,
        requirement: RouteRequirement,
    ) -> Result<(), ArtifactBuildError> {
        let index = self.resolve_variant(variant)?;
        let subject = requirement.subject();
        if self.variants[index]
            .route_requirements
            .iter()
            .any(|existing| existing.subject() == subject)
        {
            return Err(ArtifactBuildError::DuplicateRouteRequirementSubject {
                subject: Box::new(subject),
            });
        }
        limit(
            self.variants[index]
                .route_requirements
                .len()
                .saturating_add(1),
            MAX_ROUTE_REQUIREMENTS,
            ArtifactLimitKind::RouteRequirements,
        )?;
        self.variants[index].route_requirements.push(requirement);
        Ok(())
    }

    /// Publishes the ADR 0013 `Plan` claim for one variant at one delivery position.
    ///
    /// The proof-bound join the accepted stability-subject carrier requires:
    /// an IR witness over exactly this variant's verified kernel program, and
    /// one backend receipt per entry's selected payload at this position, every
    /// receipt binding the witnessed program, the payload's compilation
    /// subject, the carried object's governed section digest, and one shared
    /// complete target-environment compatibility identity. Low-level
    /// construction cannot pass a bool or a raw declaration as proof, because
    /// both argument types are privately minted.
    ///
    /// Transactional: every obligation is checked before any cell moves, so a
    /// refused claim leaves the draft exactly as it was. Other positions may
    /// remain [`PlanDeterminismScope::Unclaimed`] — a missing provider for one
    /// family must not erase an independently supported family.
    ///
    /// # Errors
    ///
    /// Returns a handle error for a variant this builder did not mint;
    /// [`ArtifactBuildError::StructuralLimit`] on
    /// [`ArtifactLimitKind::DeliveryPositions`] for a position past the
    /// artifact's own; or the exact plan-determinism refusals —
    /// [`ArtifactBuildError::MissingPlanDeterminismWitness`],
    /// [`ArtifactBuildError::MissingTargetEnvironmentDeclaration`],
    /// [`ArtifactBuildError::MissingPayloadPlanDeterminismReceipt`],
    /// [`ArtifactBuildError::PlanDeterminismProgramMismatch`],
    /// [`ArtifactBuildError::PlanDeterminismPayloadMismatch`], and
    /// [`ArtifactBuildError::PlanDeterminismEnvironmentMismatch`].
    pub fn publish_plan(
        &mut self,
        variant: VariantId,
        delivery: usize,
        witness: &PlanDeterminismWitness<'_>,
        receipts: &[PayloadPlanDeterminismReceipt],
    ) -> Result<(), ArtifactBuildError> {
        let index = self.resolve_variant(variant)?;
        let positions = self.variants[index].scope.len();
        if delivery >= positions {
            return Err(ArtifactBuildError::StructuralLimit {
                resource: ArtifactLimitKind::DeliveryPositions,
                actual: delivery.saturating_add(1),
                limit: positions,
            });
        }
        let held = &self.variants[index];
        let program_identity = held.program.canonical_identity().as_bytes();
        if witness.kernel_program_identity().as_bytes() != program_identity {
            return Err(ArtifactBuildError::MissingPlanDeterminismWitness { variant: index });
        }
        let mut first: Option<(usize, &PayloadPlanDeterminismReceipt)> = None;
        for (entry, data) in held.entries.iter().enumerate() {
            let payload = usize_of(data.implementation.payloads[delivery]);
            let descriptor = &self.payloads[payload];
            let Some(declaration) = &descriptor.environment else {
                return Err(ArtifactBuildError::MissingTargetEnvironmentDeclaration {
                    variant: index,
                    delivery,
                    entry,
                });
            };
            let receipt = receipts
                .iter()
                .find(|receipt| *receipt.payload_subject() == descriptor.digest)
                .ok_or(ArtifactBuildError::MissingPayloadPlanDeterminismReceipt {
                    variant: index,
                    delivery,
                    entry,
                })?;
            if receipt.kernel_program_identity() != program_identity {
                return Err(ArtifactBuildError::PlanDeterminismProgramMismatch {
                    variant: index,
                    delivery,
                    entry,
                });
            }
            // The object binding. A `Plan` claim fixes exact executable
            // objects through the envelope digest, so the object must be
            // carried and must be the one the receipt bound.
            let Some(content) = &self.payload_content[payload] else {
                return Err(ArtifactBuildError::PlanDeterminismPayloadMismatch {
                    variant: index,
                    delivery,
                    entry,
                });
            };
            if *receipt.object_section_digest()
                != super::codec::payload_code_section_digest(&content.code)
            {
                return Err(ArtifactBuildError::PlanDeterminismPayloadMismatch {
                    variant: index,
                    delivery,
                    entry,
                });
            }
            // The receipt's identity must be the one this payload's own
            // declaration resolves to under its own profile, backend, and
            // representation; a self-pair reports the disagreement.
            let environment = receipt.environment();
            if environment.target_profile() != &descriptor.compatibility
                || environment.backend() != &descriptor.backend
                || environment.representation() != &descriptor.representation
                || environment.provider() != declaration.provider()
                || environment.descriptor_schema() != declaration.descriptor_schema()
                || environment.descriptor() != declaration.descriptor()
            {
                return Err(ArtifactBuildError::PlanDeterminismEnvironmentMismatch {
                    variant: index,
                    delivery,
                    first_entry: entry,
                    entry,
                });
            }
            match &first {
                None => first = Some((entry, receipt)),
                Some((first_entry, held_receipt)) => {
                    if environment.as_bytes() != held_receipt.environment().as_bytes() {
                        return Err(ArtifactBuildError::PlanDeterminismEnvironmentMismatch {
                            variant: index,
                            delivery,
                            first_entry: *first_entry,
                            entry,
                        });
                    }
                }
            }
        }
        self.variants[index].scope[delivery] = PlanDeterminismScope::Plan;
        Ok(())
    }

    /// Declares the numerical realization this artifact delivers.
    ///
    /// Required: [`Self::build`] refuses a draft that never called this. ADR
    /// 0076 item 4's measurement is why — under `-fmetal-math-mode=relaxed` the
    /// emitted module records `air.compile.fast_math_disable` while every
    /// floating-point operation carries a fast-math licence, so a consumer that
    /// inferred the realization from any readable proxy would read the opposite
    /// of the truth. An artifact without the record leaves that consumer with
    /// nothing but proxies.
    ///
    /// # A typed producer assertion, and what this layer can check
    ///
    /// This is the low-level seam. It validates the record against the artifact
    /// — profile equality, subject references, and agreement with every packaged
    /// entry's own realization statement — and it **cannot** authenticate that a
    /// caller transcribed a compiler's evidence rather than inventing it. The
    /// ordinary path is `tiler_build::realization::translate`, which forwards a
    /// borrowed compiler view without reconstruction; a caller reaching here
    /// directly is asserting the same thing without that provenance.
    ///
    /// # Which entry ordinal to state
    ///
    /// The **declared** one: a flat ordinal over (variant declaration rank,
    /// declared entry ordinal), counting entries across variants in the order
    /// they were pushed. A producer states the space it can see. [`Self::build`]
    /// remaps it once into the canonical stage-key space the envelope, the
    /// identity, and every reader use — the same treatment
    /// `DeferredPredicateSpec::entry` gets, and for the same reason: a declared
    /// ordinal is a transient fact of assembly order, and two artifacts that
    /// differ only in it are one artifact.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::RealizationRedeclared`] when a record was
    /// already declared, or [`ArtifactBuildError::RealizationProfileMismatch`]
    /// when the record names a profile the already-packaged variants do not.
    pub fn declare_realization(
        &mut self,
        record: DeliveredRealizationRecord,
    ) -> Result<(), ArtifactBuildError> {
        if self.realization.is_some() {
            return Err(ArtifactBuildError::RealizationRedeclared);
        }
        if let Some(subject) = &self.subject
            && subject.profile != *record.profile()
        {
            return Err(ArtifactBuildError::RealizationProfileMismatch);
        }
        self.realization = Some(record);
        Ok(())
    }

    /// Verifies the whole artifact and freezes it, or returns the intact builder.
    ///
    /// # How the intact builder is kept intact
    ///
    /// By **lending** rather than copying. The draft's tables move into the
    /// artifact data this verifies and move back before the failure path boxes
    /// the builder, so a producer carrying an `n`-byte compiled object is never
    /// charged `2n` for the possibility that its draft is wrong. Nothing
    /// observes the builder in between: this consumes it, and no method it calls
    /// afterwards reads `self`.
    ///
    /// # Errors
    ///
    /// Returns an [`ArtifactVerificationError`] carrying every whole-artifact
    /// diagnostic and the recoverable builder when verification fails, including
    /// [`ArtifactDiagnostic::MissingDeliveredRealization`] for a draft that
    /// declared no delivered-realization record.
    ///
    /// [`ArtifactDiagnostic::MissingDeliveredRealization`]: super::ArtifactDiagnostic::MissingDeliveredRealization
    pub fn build(mut self) -> Result<VerifiedArtifactProgram, ArtifactVerificationError> {
        let Some(declared) = self.realization.clone() else {
            // Nothing further can be assembled: every downstream stage reads a
            // record, so reporting the absence alone is the whole answer rather
            // than the first of several.
            return Err(ArtifactVerificationError {
                builder: Box::new(self),
                diagnostics: vec![ArtifactDiagnostic::MissingDeliveredRealization],
            });
        };
        let mut data = self.assemble(declared);
        let mut diagnostics = super::verify::verify_artifact(&data);
        // The one place the producer's declared entry ordinals become canonical
        // ones. Run after `verify_artifact` so a draft that is broken for a
        // simpler reason — an empty portfolio, say — reports that reason first.
        let positions = packaged_entry_positions(&data.variants);
        match data.realization.remap_entries(&positions) {
            Ok(canonical) => data.realization = canonical,
            Err(entry) => {
                diagnostics.push(ArtifactDiagnostic::DeliveredRealizationEntryOutOfRange {
                    entry,
                    entries: positions.len(),
                });
            }
        }
        if diagnostics.is_empty() {
            // Identity is derived from the canonical envelope so that the bytes
            // a producer stamps and the bytes a decoder re-derives come from
            // one encoder rather than from two that agree by inspection. The
            // record is cross-checked on that same envelope, for the same
            // reason: the decoder's own check runs there.
            match ArtifactEnvelope::project(&data).and_then(|envelope| {
                envelope.check_realization()?;
                encode_identity(&envelope)
            }) {
                Ok(identity) => return Ok(VerifiedArtifactProgram { data, identity }),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }
        self.reclaim(data);
        Err(ArtifactVerificationError {
            builder: Box::new(self),
            diagnostics,
        })
    }

    /// Assembles the artifact data by **taking** the draft's tables.
    ///
    /// Every table taken here is given back by [`Self::reclaim`], which is why
    /// this may move out of a draft [`Self::build`] has promised to return
    /// intact. The alternative is a copy of every carried backend object on
    /// every publication, and a compiled `metallib` is the whole reason the
    /// envelope carries opaque sections at all.
    ///
    /// Three fields are copied instead, each for a reason the lending does not
    /// reach. `semantic` is four identity digests with no empty value to leave
    /// behind. `inputs` and `outputs` are projections of `self.subject`, which
    /// also carries the numerical realization and target profile the data does
    /// not, so returning them would mean rebuilding a [`PortfolioSubject`]
    /// rather than restoring one. And `realization` is remapped from the
    /// producer's declared entry space into the canonical one inside `build`,
    /// so what the data ends up holding is deliberately not what the builder
    /// must keep.
    fn assemble(&mut self, realization: DeliveredRealizationRecord) -> ArtifactProgramData {
        let (inputs, outputs) = self.subject.as_ref().map_or_else(
            || (Vec::new(), Vec::new()),
            |subject| (subject.inputs.clone(), subject.outputs.clone()),
        );
        ArtifactProgramData {
            schema: ArtifactSchema::GOVERNED,
            semantic: self.semantic.clone(),
            retained: self.retained.clone(),
            routing: self.routing,
            inputs,
            outputs,
            providers: std::mem::take(&mut self.providers),
            payloads: std::mem::take(&mut self.payloads),
            payload_content: std::mem::take(&mut self.payload_content),
            expressions: std::mem::take(&mut self.expressions),
            expression_types: std::mem::take(&mut self.expression_types),
            variants: std::mem::take(&mut self.variants),
            realization,
        }
    }

    /// Returns everything [`Self::assemble`] took, leaving the draft as it was.
    ///
    /// Exact rather than reconstructed: these are the same allocations, so the
    /// recovered builder is the one the producer handed to `build` and not a
    /// value resembling it. The fields the builder holds and the data never saw
    /// — the interning map, expression phases and interface-only flags, the
    /// portfolio subject, the delivery-position count — are untouched
    /// throughout, which is what makes restoring the rest sufficient.
    ///
    /// The data is destructured exhaustively so a field added to
    /// [`ArtifactProgramData`] has to be classified here rather than silently
    /// left behind. That still cannot prove `assemble` and this agree about
    /// which fields were lent, so the codec suite's
    /// `a_recovered_builder_rebuilds_the_artifact_byte_for_byte` asserts the
    /// round trip on a draft carrying an object.
    fn reclaim(&mut self, data: ArtifactProgramData) {
        let ArtifactProgramData {
            // Nothing was lent for these: a constant, a `Copy` policy, the two
            // subject projections and the identity the builder kept its own of,
            // and the record `build` remapped out of the declared entry space.
            schema: _,
            routing: _,
            semantic: _,
            retained: _,
            inputs: _,
            outputs: _,
            realization: _,
            providers,
            payloads,
            payload_content,
            expressions,
            expression_types,
            variants,
        } = data;
        self.providers = providers;
        self.payloads = payloads;
        self.payload_content = payload_content;
        self.expressions = expressions;
        self.expression_types = expression_types;
        self.variants = variants;
    }

    /// Interns one expression node and returns the arena id that now denotes it.
    ///
    /// **Dedup is by shallow structural equality, and that decides deep
    /// structural equality by induction.** A node's operands are arena ids, not
    /// subtrees, and every operand was interned by this same function before the
    /// node naming it could be built — so two nodes with equal operand ids
    /// already denote equal subtrees, and equal shallow nodes therefore denote
    /// equal expressions. The same argument is written out at
    /// `KernelProgramBuilder::push_abi_node`.
    ///
    /// It replaces a scan that derived this node's full content key and compared
    /// it against every key already held. Because a key embeds copies of its
    /// operands' keys, that key is itself quadratic in the depth of the arena on
    /// a chain and doubles per level on a shared DAG, so the scan was cubic in
    /// bytes. The interned arena is byte-identical either way: this changes how
    /// a duplicate is *recognized*, not which nodes are duplicates.
    ///
    /// No key is derived here at all. The last reader of the builder's key table
    /// was the codec's canonical arena order, which now derives that order from
    /// `tiler_ir::program::abi::compare_expr_nodes`, so the table itself is gone
    /// and a producer no longer pays quadratic bytes to package an arena. The
    /// decoder reaches the same recognizer this function does, and its
    /// `parse_expressions` states the induction from the other side of the wire.
    fn push_node(&mut self, node: ExprNode) -> Result<AbiExprId, ArtifactBuildError> {
        if let Some(existing) = self.interned.get(&node).copied() {
            return AbiExprId::from_len(self.owner, existing).ok_or(
                ArtifactBuildError::StructuralLimit {
                    resource: ArtifactLimitKind::Expressions,
                    actual: existing,
                    limit: MAX_ABI_EXPRESSIONS,
                },
            );
        }
        limit(
            self.expressions.len().saturating_add(1),
            MAX_ABI_EXPRESSIONS,
            ArtifactLimitKind::Expressions,
        )?;
        let id = AbiExprId::from_len(self.owner, self.expressions.len()).ok_or(
            ArtifactBuildError::StructuralLimit {
                resource: ArtifactLimitKind::Expressions,
                actual: self.expressions.len().saturating_add(1),
                limit: MAX_ABI_EXPRESSIONS,
            },
        )?;
        self.expression_types
            .push(node_type(&node, &self.expression_types));
        self.expression_phases
            .push(node_phase(&node, &self.expression_phases));
        self.expression_interface_only.push(node_is_interface_only(
            &node,
            &self.expression_interface_only,
        ));
        self.interned.insert(node.clone(), self.expressions.len());
        self.expressions.push(node);
        Ok(id)
    }

    fn expect_type(&self, node: u32, expected: AbiType) -> Result<(), ArtifactBuildError> {
        let actual = self.expression_types[usize_of(node)];
        if actual == expected {
            Ok(())
        } else {
            Err(ArtifactBuildError::OperandType { expected, actual })
        }
    }

    /// Resolves one expression handle against a declared artifact use site.
    ///
    /// A use site fixes the value type, the latest phase its roots may require,
    /// and whether the expression must be computable from the bound semantic
    /// interface alone.
    fn check_use(
        &self,
        id: AbiExprId,
        use_site: AbiExprUse,
        expected: AbiType,
        admitted_through: AvailabilityPhase,
        interface_only: bool,
    ) -> Result<u32, ArtifactBuildError> {
        let node = self.resolve_expression(id)?;
        let actual = self.expression_types[usize_of(node)];
        if actual != expected {
            return Err(ArtifactBuildError::ExpressionType {
                use_site,
                expected,
                actual,
            });
        }
        let available_at = self.expression_phases[usize_of(node)];
        if available_at > admitted_through {
            return Err(ArtifactBuildError::RootPhaseEscape {
                use_site,
                available_at,
                admitted_through,
            });
        }
        if interface_only && !self.expression_interface_only[usize_of(node)] {
            return Err(ArtifactBuildError::NonInterfaceRoot { use_site });
        }
        Ok(node)
    }

    fn check_deferred(
        &mut self,
        specs: &[DeferredPredicateSpec],
        entries: usize,
    ) -> Result<Vec<DeferredPredicateData>, ArtifactBuildError> {
        limit(
            specs.len(),
            MAX_DEFERRED_PREDICATES,
            ArtifactLimitKind::DeferredPredicates,
        )?;
        let mut resolved = Vec::with_capacity(specs.len());
        for spec in specs {
            if usize_of(spec.entry) >= entries {
                return Err(ArtifactBuildError::DeferredQueryEntryOutOfRange {
                    entry: spec.entry,
                    entries,
                });
            }
            let predicate = self.mint_deferred_predicate(&spec.requirement)?;
            if resolved.iter().any(|held: &DeferredPredicateData| {
                held.predicate == predicate
                    && held.requirement == spec.requirement
                    && held.entry == spec.entry
            }) {
                return Err(ArtifactBuildError::DuplicateDeferredPredicate);
            }
            resolved.push(DeferredPredicateData {
                predicate,
                requirement: spec.requirement.clone(),
                entry: spec.entry,
            });
        }
        Ok(resolved)
    }

    fn mint_deferred_predicate(
        &mut self,
        requirement: &PreparedEntryTargetRequirement,
    ) -> Result<u32, ArtifactBuildError> {
        let query = requirement.query();
        let observed = self.push_root(AbiRoot::TargetProperty {
            key: query.key().clone(),
            phase: query.available_at(),
        })?;
        let required = self.push_root(AbiRoot::UnsignedLiteral(requirement.required()))?;
        let predicate = match requirement.relation() {
            TargetPropertyRequirementRelation::ObservedAtLeastRequired => {
                self.push_binary(AbiBinaryOp::LessOrEqual, required, observed)?
            }
            TargetPropertyRequirementRelation::ObservedEqualsRequired => {
                self.push_binary(AbiBinaryOp::Equal, required, observed)?
            }
            TargetPropertyRequirementRelation::RequiredImpliesObserved => {
                let zero = self.push_root(AbiRoot::UnsignedLiteral(0))?;
                let one = self.push_root(AbiRoot::UnsignedLiteral(1))?;
                let required_is_zero = self.push_binary(AbiBinaryOp::Equal, required, zero)?;
                let observed_nonzero = self.push_binary(AbiBinaryOp::LessOrEqual, one, observed)?;
                self.push_binary(AbiBinaryOp::Or, required_is_zero, observed_nonzero)?
            }
        };
        self.check_use(
            predicate,
            AbiExprUse::DeferredPredicate,
            AbiType::Boolean,
            query.available_at(),
            false,
        )
    }

    fn check_entries(
        &self,
        program: &VerifiedKernelProgram,
        specs: &[EntrySpec],
        facts: &AbiFacts,
        derived: &DerivedAbi,
    ) -> Result<Vec<EntryData>, ArtifactBuildError> {
        let mut resolved = Vec::with_capacity(specs.len());
        // The first entry any variant declares fixes the artifact's
        // delivery-position count; within this call the first entry of this
        // variant fixes it for the rest. Carried rather than re-read from
        // `self`, because `self.delivery_positions` is only updated once the
        // whole variant is accepted and a variant must be internally consistent
        // before it can establish anything.
        let mut positions = self.delivery_positions;
        for (index, (spec, stage)) in specs.iter().zip(program.stages()).enumerate() {
            let bindings = self.check_bindings(program, index, spec, stage, facts, derived)?;
            let input_extents = derive_extent_operands(index, stage)?;
            let launch = self.check_launch(index, &spec.launch, stage, facts, &derived.adopted)?;
            refuse_bound_extent_specialization(
                index,
                stage,
                &self.expressions,
                &bindings,
                &launch,
                &input_extents,
            )?;
            let declared = spec.implementation.payloads.len();
            if declared == 0 {
                return Err(ArtifactBuildError::EmptyDelivery { entry: index });
            }
            limit(
                declared,
                MAX_DELIVERY_POSITIONS,
                ArtifactLimitKind::DeliveryPositions,
            )?;
            match positions {
                Some(expected) if expected != declared => {
                    return Err(ArtifactBuildError::DeliveryCardinality {
                        entry: index,
                        expected,
                        actual: declared,
                    });
                }
                Some(_) => {}
                None => positions = Some(declared),
            }
            let mut payloads = Vec::with_capacity(declared);
            for payload in &spec.implementation.payloads {
                payloads.push(self.resolve_payload(*payload)?);
            }
            resolved.push(EntryData {
                bindings,
                input_extents,
                launch,
                implementation: StoredBackendEntry {
                    payloads,
                    entry_key: spec.implementation.entry_key.clone(),
                },
            });
        }
        Ok(resolved)
    }

    fn check_bindings(
        &self,
        program: &VerifiedKernelProgram,
        entry: usize,
        spec: &EntrySpec,
        stage: StageRef<'_>,
        facts: &AbiFacts,
        derived: &DerivedAbi,
    ) -> Result<Vec<BindingData>, ArtifactBuildError> {
        let buffers: Vec<_> = stage.kernel().buffers().collect();
        if buffers.len() != spec.bindings.len() {
            return Err(ArtifactBuildError::BindingCardinality {
                entry,
                expected: buffers.len(),
                actual: spec.bindings.len(),
            });
        }
        limit(
            spec.bindings.len(),
            MAX_ENTRY_BINDINGS,
            ArtifactLimitKind::EntryBindings,
        )?;
        let mut resolved = Vec::with_capacity(spec.bindings.len());
        // Retained so a repeated internal target can name the earlier slot it
        // aliases. Values compare by program identity and position, so this is
        // the addressed value itself rather than a structural resemblance to it.
        let mut internal: Vec<(usize, MaterializedValueRef<'_>)> = Vec::new();
        for (slot, ((binding, buffer), access)) in spec
            .bindings
            .iter()
            .zip(&buffers)
            .zip(stage.accesses())
            .enumerate()
        {
            let window = access.view().window();
            // Minted by `adopt_offsets` from this access's own window, so
            // unlike the extent below there is no producer statement to prove
            // against it — the check resolves the handle and types the use.
            let offset = self.check_use(
                derived.offsets[entry][slot],
                AbiExprUse::AccessibleOffset,
                AbiType::Unsigned,
                AvailabilityPhase::LiveDevicePreflight,
                true,
            )?;
            let node = self.check_use(
                Self::resolve(&derived.adopted, access.accessible_bytes())?,
                AbiExprUse::AccessibleBytes,
                AbiType::Unsigned,
                AvailabilityPhase::LiveDevicePreflight,
                true,
            )?;
            let computed = self.evaluate_static(node, AbiExprUse::AccessibleBytes, facts)?;
            if computed != window.length {
                return Err(ArtifactBuildError::AccessibleBytesDisagreement {
                    entry,
                    binding: slot,
                    expected: window.length,
                    actual: computed,
                });
            }
            let value = access.view().value();
            let target = binding_target(program, entry, slot, access.view())?;
            if matches!(target, BindingTargetData::Internal)
                && let Some((earlier, _)) = internal.iter().find(|(_, held)| *held == value)
            {
                return Err(ArtifactBuildError::AliasedInternalBinding {
                    entry,
                    binding: slot,
                    aliases: *earlier,
                });
            }
            if matches!(target, BindingTargetData::Internal) {
                internal.push((slot, value));
            }
            resolved.push(BindingData {
                kind: binding.kind,
                storage_scalar: value.storage_scalar(),
                access_type: buffer.element_type,
                component_role: value.component_role(),
                encoding: value.storage_encoding(),
                address_space: buffer.address_space,
                access: buffer.access,
                alignment: value.alignment(),
                target,
                accessible_offset: offset,
                accessible_bytes: node,
            });
        }
        Ok(resolved)
    }

    fn check_launch(
        &self,
        entry: usize,
        spec: &LaunchSpec,
        stage: StageRef<'_>,
        facts: &AbiFacts,
        adopted: &[Option<AbiExprId>],
    ) -> Result<LaunchData, ArtifactBuildError> {
        let stage_launch = stage.launch();
        let grid_threads = self.check_use(
            Self::resolve(adopted, stage_launch.grid_threads)?,
            AbiExprUse::LaunchThreads,
            AbiType::Unsigned,
            AvailabilityPhase::LiveDevicePreflight,
            true,
        )?;
        let threads_per_workgroup = self.check_use(
            Self::resolve(adopted, stage_launch.threads_per_workgroup)?,
            AbiExprUse::ThreadsPerWorkgroup,
            AbiType::Unsigned,
            AvailabilityPhase::LiveDevicePreflight,
            true,
        )?;
        let required = u64::from(stage.kernel().requirements().threads_per_workgroup);
        let declared = self.evaluate_static(
            threads_per_workgroup,
            AbiExprUse::ThreadsPerWorkgroup,
            facts,
        )?;
        if declared != required {
            return Err(ArtifactBuildError::LaunchDisagreement {
                entry,
                expected: required,
                actual: declared,
            });
        }
        let threads = self.evaluate_static(grid_threads, AbiExprUse::LaunchThreads, facts)?;
        if threads == 0 && !spec.zero_work_skips_dispatch {
            return Err(ArtifactBuildError::ZeroWorkPolicy { entry });
        }
        limit(
            spec.preconditions.len(),
            MAX_LAUNCH_PRECONDITIONS,
            ArtifactLimitKind::LaunchPreconditions,
        )?;
        let mut preconditions = Vec::with_capacity(spec.preconditions.len());
        for precondition in &spec.preconditions {
            let node = self.check_use(
                *precondition,
                AbiExprUse::LaunchPrecondition,
                AbiType::Boolean,
                AvailabilityPhase::LaunchPreflight,
                false,
            )?;
            if preconditions.contains(&node) {
                return Err(ArtifactBuildError::DuplicateLaunchPrecondition { entry });
            }
            preconditions.push(node);
        }
        Ok(LaunchData {
            grid_threads,
            threads_per_workgroup,
            zero_work_skips_dispatch: spec.zero_work_skips_dispatch,
            preconditions,
        })
    }

    /// Evaluates one interface-only expression against the declared static shapes.
    ///
    /// This is a compile-time consistency check, not a runtime evaluation: the facts are the program's own declared input shapes, and the derived accessible range or launch geometry must agree with the same program's static byte window or resource requirement.
    fn evaluate_static(
        &self,
        node: u32,
        use_site: AbiExprUse,
        facts: &AbiFacts,
    ) -> Result<u64, ArtifactBuildError> {
        match evaluate(&self.expressions, node, facts) {
            Ok(AbiValue::Unsigned(value)) => Ok(value),
            Ok(AbiValue::Boolean(_)) => Err(ArtifactBuildError::ExpressionType {
                use_site,
                expected: AbiType::Unsigned,
                actual: AbiType::Boolean,
            }),
            Err(cause) => Err(ArtifactBuildError::StaticEvaluation { use_site, cause }),
        }
    }

    fn check_subject(
        &self,
        program: &VerifiedKernelProgram,
        spec: &VariantSpec,
    ) -> Result<PortfolioSubject, ArtifactBuildError> {
        let (inputs, outputs) = project_interface(program, &self.interface)?;
        let numerical = program
            .stages()
            .next()
            .ok_or(ArtifactBuildError::EntryCardinality {
                expected: 1,
                actual: 0,
            })?
            .kernel()
            .numerical();
        if program
            .stages()
            .any(|stage| stage.kernel().numerical() != numerical)
        {
            return Err(ArtifactBuildError::NumericalContractMismatch);
        }
        if let Some(subject) = &self.subject {
            if subject.inputs != inputs || subject.outputs != outputs {
                return Err(ArtifactBuildError::InterfaceMismatch);
            }
            if subject.numerical != numerical {
                return Err(ArtifactBuildError::NumericalContractMismatch);
            }
            if subject.profile != spec.target_profile {
                return Err(ArtifactBuildError::TargetProfileMismatch);
            }
        }
        // The other direction of `declare_realization`'s check, because either
        // statement can be made first and the second one made is the one that
        // introduced the disagreement.
        if let Some(record) = &self.realization
            && *record.profile() != spec.target_profile
        {
            return Err(ArtifactBuildError::RealizationProfileMismatch);
        }
        Ok(PortfolioSubject {
            inputs,
            outputs,
            numerical,
            profile: spec.target_profile.clone(),
        })
    }

    /// Evaluates one draft expression, for tests that never package a variant.
    ///
    /// Evaluation is otherwise a property of the verified product; this exists
    /// so an expression-language test does not have to assemble a whole
    /// artifact around the node it is exercising.
    #[cfg(test)]
    pub(super) fn evaluate_draft_expression(
        &self,
        id: AbiExprId,
        facts: &AbiFacts,
    ) -> Result<AbiValue, super::expr::AbiEvaluationError> {
        let node = self
            .resolve_expression(id)
            .expect("a test evaluates a node this builder minted");
        evaluate(&self.expressions, node, facts)
    }

    fn resolve_expression(&self, id: AbiExprId) -> Result<u32, ArtifactBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ArtifactEntityKind::Expression, true));
        }
        if id.as_usize() >= self.expressions.len() {
            return Err(invalid_handle(ArtifactEntityKind::Expression, false));
        }
        Ok(id.index)
    }

    fn resolve_payload(&self, id: PayloadId) -> Result<u32, ArtifactBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ArtifactEntityKind::Payload, true));
        }
        if id.as_usize() >= self.payloads.len() {
            return Err(invalid_handle(ArtifactEntityKind::Payload, false));
        }
        Ok(id.index)
    }

    fn resolve_variant(&self, id: VariantId) -> Result<usize, ArtifactBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ArtifactEntityKind::Variant, true));
        }
        if id.as_usize() >= self.variants.len() {
            return Err(invalid_handle(ArtifactEntityKind::Variant, false));
        }
        Ok(id.as_usize())
    }
}

fn usize_of(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Reads the ordered named interface of a verified semantic program.
///
/// # Errors
///
/// Returns [`ArtifactBuildError::SymbolicSemanticInterface`] when an interface
/// extent names a declared `ShapeEnv` symbol. See that variant for why the
/// refusal is what keeps the envelope's three carried subjects sufficient.
fn read_semantic_interface(
    semantic: &SemanticProgram,
) -> Result<SemanticInterface, ArtifactBuildError> {
    let inputs = semantic
        .inputs()
        .map(|input| {
            let shape = static_interface_shape(semantic, input.value(), input.key().as_str())?;
            let value_type = semantic
                .value(input.value())
                .expect("a verified program resolves its own input value")
                .resolved_type()
                .clone();
            Ok((input.key().clone(), shape, value_type))
        })
        .collect::<Result<Vec<_>, ArtifactBuildError>>()?;
    let outputs = semantic
        .outputs()
        .map(|output| {
            let shape = static_interface_shape(semantic, output.value(), output.key().as_str())?;
            let value_type = semantic
                .value(output.value())
                .expect("a verified program resolves its own output value")
                .resolved_type()
                .clone();
            Ok((output.key().clone(), shape, value_type))
        })
        .collect::<Result<Vec<_>, ArtifactBuildError>>()?;
    Ok(SemanticInterface { inputs, outputs })
}

/// Returns one interface value's fixed shape, refusing a symbolic one.
///
/// The single place this builder narrows the semantic layer's total shape view,
/// so the refusal cannot be extended for the input side and forgotten for the
/// output side.
fn static_interface_shape(
    semantic: &SemanticProgram,
    value: tiler_ir::semantic::ValueId,
    interface: &str,
) -> Result<Shape, ArtifactBuildError> {
    semantic
        .shape(value)
        .expect("a verified program resolves its own interface value")
        .as_static()
        .cloned()
        .ok_or_else(|| ArtifactBuildError::SymbolicSemanticInterface {
            interface: interface.to_owned(),
        })
}

/// Projects the artifact's published interface from one variant's program.
///
/// The order is the semantic interface's, read from the semantic program the
/// variant realizes. The kernel program now carries the same order — whole-program
/// verification pins its published records to that interface under
/// `tiler.kernel-program.v11` — so this reads the authority directly rather than
/// re-deriving an order the layer below had discarded. The storage element type
/// comes from the materialized value the program binds, so the published
/// contract is a fact of the plan rather than a producer claim.
#[allow(
    clippy::type_complexity,
    reason = "the pair is the artifact's two interface projections and is destructured at its only call site"
)]
fn project_interface(
    program: &VerifiedKernelProgram,
    interface: &SemanticInterface,
) -> Result<
    (
        Vec<InterfaceEntryData<InputKey>>,
        Vec<InterfaceEntryData<OutputKey>>,
    ),
    ArtifactBuildError,
> {
    let mut inputs = Vec::with_capacity(interface.inputs.len());
    for (key, shape, value_type) in &interface.inputs {
        let values: Vec<_> = program
            .values()
            .filter(|value| match value.origin() {
                MaterializedOrigin::ProgramInput { key: bound } => bound == key,
                MaterializedOrigin::Internal => false,
            })
            .collect();
        let components = project_components(value_type, &values)?;
        inputs.push(InterfaceEntryData {
            key: key.clone(),
            shape: shape.clone(),
            logical_type: value_type.canonical_encoding().as_bytes().to_vec(),
            components,
        });
    }
    let mut outputs = Vec::with_capacity(interface.outputs.len());
    for (key, shape, value_type) in &interface.outputs {
        let values: Vec<_> = program
            .outputs()
            .filter(|output| output.key() == key)
            .map(tiler_ir::program::ProgramOutputRef::value)
            .collect();
        let components = project_components(value_type, &values)?;
        outputs.push(InterfaceEntryData {
            key: key.clone(),
            shape: shape.clone(),
            logical_type: value_type.canonical_encoding().as_bytes().to_vec(),
            components,
        });
    }
    Ok((inputs, outputs))
}

fn project_components(
    value_type: &ResolvedValueType,
    values: &[MaterializedValueRef<'_>],
) -> Result<Vec<InterfaceComponentData>, ArtifactBuildError> {
    let roles: Vec<_> = match value_type.encoded_numeric_parts() {
        None => vec![None],
        Some((_, contract)) => contract
            .components()
            .iter()
            .map(|component| Some(component.role()))
            .collect(),
    };
    roles
        .into_iter()
        .map(|role| {
            let value = values
                .iter()
                .copied()
                .find(|value| value.component_role() == role)
                .ok_or(ArtifactBuildError::InterfaceMismatch)?;
            Ok(InterfaceComponentData {
                role,
                shape: value.shape().clone(),
                resolved_type: value
                    .component_type()
                    .map(|value_type| value_type.canonical_encoding().as_bytes().to_vec()),
                storage_scalar: value.storage_scalar(),
                access_type: value.element_type(),
                encoding: value.storage_encoding(),
            })
        })
        .collect()
}

/// Derives what one binding slot addresses, from the plan rather than the producer.
///
/// This is the fact a decoded envelope cannot re-derive, so it is the one fact
/// most worth taking from the program instead of accepting as a claim. It reads
/// the same stage access the binding's element type, address space, access mode
/// and alignment already come from, so a producer cannot state a correspondence
/// its own plan contradicts.
///
/// The target names the whole addressed value; *where in it* the slot reaches is
/// the binding's own accessible offset and extent, so a slot addressing part of
/// a value is packageable rather than refused.
///
/// # Errors
///
/// Returns [`ArtifactBuildError::UnnameableBindingTarget`] when the addressed
/// value's role and origin disagree about whether its bytes cross the program
/// interface.
fn binding_target(
    program: &VerifiedKernelProgram,
    entry: usize,
    binding: usize,
    view: tiler_ir::program::ViewRef<'_>,
) -> Result<BindingTargetData, ArtifactBuildError> {
    let value = view.value();
    let unnameable = |role| ArtifactBuildError::UnnameableBindingTarget {
        entry,
        binding,
        role,
        external_origin: matches!(value.origin(), MaterializedOrigin::ProgramInput { .. }),
    };
    // Every combination of origin and role is written out rather than collapsed
    // behind a wildcard. The three rejecting arms are unreachable for a verified
    // program — `KernelProgramBuilder::push_value`'s `check_origin` admits only
    // the other three pairs — but that is another crate's builder rule rather
    // than something this match can rely on the type system to keep true, and
    // neither vocabulary is `#[non_exhaustive]`, so widening either must stop the
    // build here instead of falling into an arm chosen for a different case.
    match (value.origin(), value.role()) {
        (MaterializedOrigin::ProgramInput { key }, ValueRole::Input) => {
            Ok(BindingTargetData::ProgramInput(key.clone()))
        }
        (MaterializedOrigin::Internal, ValueRole::Output) => {
            let mut keys: Vec<OutputKey> = program
                .outputs()
                .filter(|output| output.value() == value)
                .map(|output| output.key().clone())
                .collect();
            if keys.is_empty() {
                // `verify_outputs` proves every output-role value is published,
                // so this is unreachable for a verified program. Refusing is
                // still the only fail-closed answer: the alternative is
                // encoding an empty key list a loader would read as "bind
                // nothing to a slot the kernel writes through".
                return Err(unnameable(ValueRole::Output));
            }
            keys.sort_unstable();
            Ok(BindingTargetData::ProgramOutput(keys))
        }
        (MaterializedOrigin::Internal, ValueRole::Temporary) => Ok(BindingTargetData::Internal),
        (MaterializedOrigin::ProgramInput { .. }, role @ ValueRole::Temporary)
        | (MaterializedOrigin::ProgramInput { .. }, role @ ValueRole::Output)
        | (MaterializedOrigin::Internal, role @ ValueRole::Input) => Err(unnameable(role)),
    }
}

/// Derives the live input-extent operand rows from the kernel the entry binds.
///
/// Callers do not supply a second list. Each kernel operand names an exact
/// region-local access position; this maps that position through the matching
/// stage access onto the program-interface key, and refuses an operand whose
/// access is absent, is not an input, or names an axis the input does not have.
fn derive_extent_operands(
    entry: usize,
    stage: StageRef<'_>,
) -> Result<Vec<ExtentOperandData>, ArtifactBuildError> {
    let kernel = stage.kernel();
    let declared = kernel.input_extents().len();
    limit(declared, MAX_ENTRY_EXTENTS, ArtifactLimitKind::EntryExtents)?;
    let buffers: Vec<_> = kernel.buffers().collect();
    let accesses: Vec<_> = stage.accesses().collect();
    let mut rows = Vec::with_capacity(declared);
    for parameter in kernel.input_extents() {
        let position = usize::try_from(parameter.access.get()).unwrap_or(usize::MAX);
        let Some((buffer, access)) = buffers.get(position).zip(accesses.get(position)) else {
            return Err(ArtifactBuildError::ExtentOperandUnbound {
                entry,
                access: parameter.access,
                axis: parameter.axis.get(),
            });
        };
        if buffer.tensor != TensorRole::Input {
            return Err(ArtifactBuildError::ExtentOperandUnbound {
                entry,
                access: parameter.access,
                axis: parameter.axis.get(),
            });
        }
        let value = access.view().value();
        let MaterializedOrigin::ProgramInput { key } = value.origin() else {
            return Err(ArtifactBuildError::ExtentOperandUnbound {
                entry,
                access: parameter.access,
                axis: parameter.axis.get(),
            });
        };
        let rank = value.shape().rank();
        if usize::try_from(parameter.axis.get()).unwrap_or(usize::MAX) >= rank {
            return Err(ArtifactBuildError::ExtentOperandAxis {
                entry,
                key: key.as_str().to_owned(),
                axis: parameter.axis.get(),
                rank,
            });
        }
        rows.push(ExtentOperandData {
            key: key.clone(),
            axis: parameter.axis,
            value_type: super::AbiType::Unsigned,
        });
    }
    Ok(rows)
}

/// Refuses a kernel that baked an extent the entry still treats as a binding.
///
/// Variant applicability guards and artifact-owned launch preconditions may
/// read a bound input extent — that is route-time selection or a host
/// predicate, not specialization. Accessible-range and launch-*geometry*
/// formulas that name `InputExtent` make the value a per-invocation binding
/// the payload or launch consumes. A kernel that has no matching live operand
/// and a nonzero baked `element_count` for that input has specialized on the
/// binding.
fn refuse_bound_extent_specialization(
    entry: usize,
    stage: StageRef<'_>,
    arena: &[ExprNode],
    bindings: &[BindingData],
    launch: &LaunchData,
    live_operands: &[ExtentOperandData],
) -> Result<(), ArtifactBuildError> {
    let mut named = Vec::new();
    for node in bindings
        .iter()
        .map(|binding| binding.accessible_bytes)
        .chain([launch.grid_threads, launch.threads_per_workgroup])
    {
        collect_input_extents(arena, node, &mut named);
    }
    named.sort_unstable();
    named.dedup();
    for (key, axis) in named {
        if live_operands
            .iter()
            .any(|operand| operand.key == key && operand.axis == axis)
        {
            continue;
        }
        let Some(element_count) = baked_element_count(stage, &key) else {
            continue;
        };
        if element_count == 0 {
            continue;
        }
        return Err(ArtifactBuildError::BoundExtentSpecialization {
            entry,
            key: key.as_str().to_owned(),
            axis: axis.get(),
            element_count,
        });
    }
    Ok(())
}

fn collect_input_extents(arena: &[ExprNode], node: u32, named: &mut Vec<(InputKey, Axis)>) {
    let Some(expression) = arena.get(usize_of(node)) else {
        return;
    };
    match expression {
        ExprNode::Root(AbiRoot::InputExtent { key, axis }) => {
            named.push((key.clone(), *axis));
        }
        ExprNode::Root(_) => {}
        ExprNode::Unary { operand, .. } => collect_input_extents(arena, *operand, named),
        ExprNode::Binary { left, right, .. } => {
            collect_input_extents(arena, *left, named);
            collect_input_extents(arena, *right, named);
        }
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => {
            collect_input_extents(arena, *condition, named);
            collect_input_extents(arena, *if_true, named);
            collect_input_extents(arena, *if_false, named);
        }
    }
}

fn baked_element_count(stage: StageRef<'_>, key: &InputKey) -> Option<u64> {
    let accesses: Vec<_> = stage.accesses().collect();
    let buffers: Vec<_> = stage.kernel().buffers().collect();
    accesses.iter().zip(&buffers).find_map(|(access, buffer)| {
        match access.view().value().origin() {
            MaterializedOrigin::ProgramInput { key: bound } if bound == key => {
                Some(buffer.element_count)
            }
            _ => None,
        }
    })
}

/// Every arena position the program's own ABI names.
///
/// The replay's roots, so the whole program ABI is adopted rather than the part
/// one variant happens to reference: a second variant over the same program
/// must resolve to the same handles.
#[allow(
    clippy::redundant_closure_for_method_calls,
    reason = "the receiver type is a private view of tiler-ir and naming it here would import a type this module otherwise never mentions"
)]
fn program_abi_use_sites(program: &VerifiedKernelProgram) -> Vec<u32> {
    let mut sites = vec![program.applicability_guard()];
    for stage in program.stages() {
        let launch = stage.launch();
        sites.push(launch.grid_threads);
        sites.push(launch.threads_per_workgroup);
        sites.extend(stage.accesses().map(|access| access.accessible_bytes()));
    }
    sites
}

/// Builds the declared-shape fact environment one program's ABI is checked in.
fn static_facts(program: &VerifiedKernelProgram) -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    for value in program.values() {
        if let MaterializedOrigin::ProgramInput { key } = value.origin() {
            binder
                .bind_input_shape(key, value.shape())
                .expect("a verified program declares each input exactly once");
        }
    }
    binder.build()
}
