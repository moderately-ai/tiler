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
use tiler_ir::schedule::NumericalRealization;
use tiler_ir::semantic::{
    InputKey, OutputKey, ProviderIdentity, ResolvedValueType, SemanticIdentity, SemanticProgram,
};
use tiler_ir::shape::Shape;

use super::codec::{ArtifactEnvelope, PayloadContent, PayloadMetadata};
use super::error::{
    AbiExprUse, ArtifactBuildError, ArtifactEntityKind, ArtifactLimitKind,
    ArtifactVerificationError, invalid_handle, limit,
};
use super::expr::{
    AbiBinaryOp, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue, AvailabilityPhase, ExprNode,
    binary_operand_type, evaluate, expr_key, node_is_interface_only, node_phase, node_type,
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
    EntryData, InterfaceComponentData, InterfaceEntryData, LaunchData, RoutingPolicy,
    SchemaVersion, SelectedProvider, StoredBackendEntry, VariantData, VerifiedArtifactProgram,
    encode_identity,
};
use super::{
    MAX_ABI_EXPRESSIONS, MAX_ARTIFACT_PAYLOADS, MAX_ARTIFACT_VARIANTS, MAX_DEFERRED_PREDICATES,
    MAX_ENTRY_BINDINGS, MAX_ENVIRONMENT_PROVIDERS, MAX_LAUNCH_PRECONDITIONS,
    MAX_SELECTED_PROVIDERS, MAX_VARIANT_ENTRIES,
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
    interface: SemanticInterface,
    environment: CompilationEnvironment,
    providers: Vec<SelectedProvider>,
    payloads: Vec<BackendPayloadDescriptor>,
    payload_content: Vec<Option<PayloadContent>>,
    expressions: Vec<ExprNode>,
    expression_keys: Vec<Vec<u8>>,
    /// Arena position of every node already interned, keyed by the node itself.
    interned: std::collections::HashMap<ExprNode, usize>,
    expression_types: Vec<AbiType>,
    expression_phases: Vec<AvailabilityPhase>,
    expression_interface_only: Vec<bool>,
    variants: Vec<VariantData>,
    subject: Option<PortfolioSubject>,
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
        let owner =
            next_artifact_builder_id().ok_or(ArtifactBuildError::BuilderIdentityExhausted)?;
        Ok(Self {
            owner,
            semantic: semantic.semantic_identity().clone(),
            interface: read_semantic_interface(semantic),
            environment,
            providers: Vec::new(),
            payloads: Vec::new(),
            payload_content: Vec::new(),
            expressions: Vec::new(),
            expression_keys: Vec::new(),
            interned: std::collections::HashMap::new(),
            expression_types: Vec::new(),
            expression_phases: Vec::new(),
            expression_interface_only: Vec::new(),
            variants: Vec::new(),
            subject: None,
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
    pub fn push_pending_payload(
        &mut self,
        backend: BackendKey,
        representation: RepresentationKey,
        payload_schema: SchemaVersion,
        compatibility: TargetProfileRef,
        execution_policy: ArtifactExecutionPolicy,
        metadata: &PayloadMetadata,
    ) -> Result<PayloadId, ArtifactBuildError> {
        self.push_payload(BackendPayloadDescriptor {
            backend,
            representation,
            payload_schema,
            digest: metadata.identity()?,
            compatibility,
            execution_policy,
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
    pub fn push_carried_payload(
        &mut self,
        backend: BackendKey,
        representation: RepresentationKey,
        payload_schema: SchemaVersion,
        compatibility: TargetProfileRef,
        execution_policy: ArtifactExecutionPolicy,
        content: PayloadContent,
    ) -> Result<PayloadId, ArtifactBuildError> {
        let id = self.push_pending_payload(
            backend,
            representation,
            payload_schema,
            compatibility,
            execution_policy,
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
    /// Every provider a deferred predicate names must already be selected, so
    /// [`ArtifactProgramBuilder::select_provider`] precedes this call.
    ///
    /// # Errors
    ///
    /// Returns a handle error; a semantic-subject, interface, numerical, or
    /// target-profile disagreement; an entry or binding cardinality error; an
    /// expression type, root-phase, or interface-root rejection; an accessible
    /// range, launch, or zero-work disagreement with the bound program; a
    /// duplicate variant, deferred predicate, or launch precondition; an
    /// unselected deferred authority; a non-deferred predicate phase; or a
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
        self.variants.push(VariantData {
            program: program.clone(),
            guard,
            profile: spec.target_profile,
            feasibility_rules: spec.feasibility_rules,
            deferred,
            entries,
        });
        Ok(id)
    }

    /// Verifies the whole artifact and freezes it, or returns the intact builder.
    ///
    /// # Errors
    ///
    /// Returns an [`ArtifactVerificationError`] carrying every whole-artifact
    /// diagnostic and the recoverable builder when verification fails.
    pub fn build(self) -> Result<VerifiedArtifactProgram, ArtifactVerificationError> {
        let data = self.assemble();
        let mut diagnostics = super::verify::verify_artifact(&data);
        if diagnostics.is_empty() {
            // Identity is derived from the canonical envelope so that the bytes
            // a producer stamps and the bytes a decoder re-derives come from
            // one encoder rather than from two that agree by inspection.
            match ArtifactEnvelope::project(&data).and_then(|envelope| encode_identity(&envelope)) {
                Ok(identity) => return Ok(VerifiedArtifactProgram { data, identity }),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }
        Err(ArtifactVerificationError {
            builder: Box::new(self),
            diagnostics,
        })
    }

    fn assemble(&self) -> ArtifactProgramData {
        let (inputs, outputs) = self.subject.as_ref().map_or_else(
            || (Vec::new(), Vec::new()),
            |subject| (subject.inputs.clone(), subject.outputs.clone()),
        );
        ArtifactProgramData {
            schema: ArtifactSchema::GOVERNED,
            semantic: self.semantic.clone(),
            routing: self.routing,
            inputs,
            outputs,
            providers: self.providers.clone(),
            payloads: self.payloads.clone(),
            payload_content: self.payload_content.clone(),
            expressions: self.expressions.clone(),
            expression_keys: self.expression_keys.clone(),
            expression_types: self.expression_types.clone(),
            variants: self.variants.clone(),
        }
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
        let key = expr_key(&node, &self.expression_keys);
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
        self.interned
            .insert(node.clone(), self.expression_keys.len());
        self.expression_keys.push(key);
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
        for (index, (spec, stage)) in specs.iter().zip(program.stages()).enumerate() {
            let bindings = self.check_bindings(program, index, spec, stage, facts, derived)?;
            let launch = self.check_launch(index, &spec.launch, stage, facts, &derived.adopted)?;
            let payload = self.resolve_payload(spec.implementation.payload)?;
            resolved.push(EntryData {
                bindings,
                launch,
                implementation: StoredBackendEntry {
                    payload,
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
}

fn usize_of(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Reads the ordered named interface of a verified semantic program.
fn read_semantic_interface(semantic: &SemanticProgram) -> SemanticInterface {
    let inputs = semantic
        .inputs()
        .map(|input| {
            let shape = semantic
                .shape(input.value())
                .expect("a verified program resolves its own input value")
                .clone();
            let value_type = semantic
                .value(input.value())
                .expect("a verified program resolves its own input value")
                .resolved_type()
                .clone();
            (input.key().clone(), shape, value_type)
        })
        .collect();
    let outputs = semantic
        .outputs()
        .map(|output| {
            let shape = semantic
                .shape(output.value())
                .expect("a verified program resolves its own output value")
                .clone();
            let value_type = semantic
                .value(output.value())
                .expect("a verified program resolves its own output value")
                .resolved_type()
                .clone();
            (output.key().clone(), shape, value_type)
        })
        .collect();
    SemanticInterface { inputs, outputs }
}

/// Projects the artifact's published interface from one variant's program.
///
/// The order is the semantic interface's, which the kernel program does not
/// retain. The storage element type comes from the materialized value the
/// program binds, so the published contract is a fact of the plan rather than a
/// producer claim.
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
