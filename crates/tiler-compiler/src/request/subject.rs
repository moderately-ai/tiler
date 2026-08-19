//! The canonical request subject, and the exact bytes it is encoded to.
//!
//! Identity-bearing: `canonical_explain_subject_bytes` writes the
//! `tiler.compiler.request-subject.v6` preimage every explain record, receipt,
//! and pinned qualifier is bound to. The projection from a recognized shape to
//! its subject lives beside the encoder it feeds, because the two decide one
//! question together — which facts separate two requests — and a projection that
//! dropped a field the encoder never saw would give two programs one identity.
//!
//! Every encoder here is an exhaustive match rather than a discriminant cast, so
//! a widened vocabulary stops the build at the arm instead of silently restating
//! an already-encoded subject.

use super::*;

/// The exact request facts every explain record and receipt is bound to.
///
/// The installed lowering authority participates through its canonical registry
/// identity rather than the registry itself: the identity is comparable and
/// orderable while a registry holding provider implementations is neither, and
/// the identity already binds every authority the registry was frozen over.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedRequestSubject {
    normalized: NormalizedProgramSubject,
    pub(super) semantic_identity: SemanticIdentity,
    /// The caller's stated preference, retained beside the resolved contract.
    ///
    /// Both are bound because they answer different questions: the list is what
    /// the caller declared acceptable, and the resolved entry is what this
    /// target compiles under. Binding only the second would let two requests
    /// with different fallback intents share one subject.
    numerical_contracts: NumericalContractPreference,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: TargetProfile,
    capability_schema_version: u32,
    pub(super) lowering_registry: CanonicalLoweringRegistryIdentity,
    realization_registry: Box<[u8]>,
}

/// The subject projection of one fold's materialized contributor.
///
/// It carries the producer's own subject rather than a summary of it, for the
/// reason [`NormalizedEpilogueSubject`] does: a region of the producer's
/// partition binds against exactly the subject it would bind against if the
/// producer were the whole declared output, so [`crate::physical`]'s binding
/// recurses instead of restating each producing family's obligations again.
///
/// The continuation is carried whole rather than projected. It holds no graph
/// handle a subject must not bind — an expression, a read list, and the
/// graph-local member ordinals every other arm's member run already writes —
/// which is the same reason [`NormalizedOutputSubject::Pointwise`] carries its
/// recognized shape unprojected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MaterializedContributorSubject {
    producer: NormalizedOutputSubject,
    continuation: Option<ContributorContinuation>,
}

impl MaterializedContributorSubject {
    /// Returns the producer subject a region of the producing partition binds
    /// against.
    pub(crate) const fn producer(&self) -> &NormalizedOutputSubject {
        &self.producer
    }

    /// Returns the continuation between the producer and the fold, when an
    /// expression stands between the two.
    pub(crate) const fn continuation(&self) -> Option<&ContributorContinuation> {
        self.continuation.as_ref()
    }
}

/// The subject projection of one fold's contributor source.
///
/// Exhaustive for the reason [`SerialSumContributor`] is, and the encoder below
/// matches it rather than the fields it used to project: the `Materialized` arm
/// takes its own framed sub-tag, so no produced sum can be read as the
/// declared-input or pointwise-prologue neighbour whose bytes it would otherwise
/// share the grammar with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SerialSumContributorSubject {
    DeclaredInput(DeclaredInputOrdinal),
    PointwisePrologue {
        expression: PointwiseF32Expression,
        reads: Vec<(DeclaredInputOrdinal, LogicalAccess)>,
    },
    Materialized(Box<MaterializedContributorSubject>),
}

impl SerialSumContributorSubject {
    /// The pointwise prologue this fold computes before folding, when it has
    /// one.
    pub(crate) const fn prologue(&self) -> Option<&PointwiseF32Expression> {
        match self {
            Self::PointwisePrologue { expression, .. } => Some(expression),
            Self::DeclaredInput(_) | Self::Materialized(_) => None,
        }
    }

    /// The prologue region's reads, in access order, or empty when this fold
    /// has no prologue region.
    pub(crate) fn prologue_reads(&self) -> &[(DeclaredInputOrdinal, LogicalAccess)] {
        match self {
            Self::PointwisePrologue { reads, .. } => reads,
            Self::DeclaredInput(_) | Self::Materialized(_) => &[],
        }
    }

    /// The declared input ordinal the fold reads directly, or `None` when some
    /// region materializes its contributors.
    pub(crate) const fn declared_input(&self) -> Option<DeclaredInputOrdinal> {
        match self {
            Self::DeclaredInput(ordinal) => Some(*ordinal),
            Self::PointwisePrologue { .. } | Self::Materialized(_) => None,
        }
    }

    /// The materialized producer and its continuation, when another region
    /// computes this fold's contributors.
    pub(crate) fn materialized(&self) -> Option<&MaterializedContributorSubject> {
        match self {
            Self::Materialized(materialized) => Some(materialized),
            Self::DeclaredInput(_) | Self::PointwisePrologue { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSumSubject {
    input_keys: Vec<InputKey>,
    output_key: OutputKey,
    input_shape: Shape,
    output_shape: Shape,
    reduction_axes: Vec<Axis>,
    contributor: SerialSumContributorSubject,
    members: RecognizedSerialSumMembers,
    input_elements: u64,
    output_elements: u64,
}

/// The subject projection of one recognized ordered named output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedOutputSubject {
    SerialSum(NormalizedSerialSumSubject),
    Pointwise(NormalizedPointwise),
    /// Boxed for the reason [`NormalizedOutput::Contraction`] is.
    Contraction(Box<NormalizedContraction>),
    /// Boxed for the reason [`NormalizedOutput::Epilogue`] is.
    Epilogue(Box<NormalizedEpilogueSubject>),
    /// Boxed for the reason [`NormalizedOutput::Staged`] is.
    ///
    /// The occurrence's recognized shape is carried whole rather than projected:
    /// it holds no graph handle a subject must not bind, because the occurrence
    /// coordinate it does carry is the graph-local member ordinal every other
    /// arm's member run already writes. Its producer is the one part that *is*
    /// projected, into the subject's own recursive slot, for the reason
    /// [`NormalizedEpilogueSubject`] projects a chain's.
    Staged(Box<NormalizedStagedSubject>),
}

/// The subject projection of one recognized staged family.
///
/// It carries the producer's own subject rather than a summary of it, for the
/// reason [`NormalizedEpilogueSubject`] does: a region of the producer's
/// partition binds against exactly the subject it would bind against if the
/// producer were the whole declared output, so [`crate::physical`]'s binding
/// recurses instead of restating each producing family's obligations again.
///
/// **The occurrence copy's own [`NormalizedStaged::producer`] slot is cleared,
/// and the clearing is what keeps one fact in one place.** Carrying the producer
/// both as a recognized shape and as a subject would be two accounts of one
/// value, free to disagree; the recognized side is [`NormalizedProgram`]'s and
/// the subject side is this one's.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedStagedSubject {
    occurrence: Box<NormalizedStaged>,
    producer: Option<Box<NormalizedOutputSubject>>,
}

impl NormalizedStagedSubject {
    /// Returns the occurrence's own recognized shape.
    ///
    /// Its producer slot is cleared; [`Self::producer`] is where the producer
    /// travels.
    pub(crate) const fn occurrence(&self) -> &NormalizedStaged {
        &self.occurrence
    }

    /// Returns the producer subject a region of the producing partition binds
    /// against, or `None` when every operand binds a declared input.
    pub(crate) fn producer(&self) -> Option<&NormalizedOutputSubject> {
        self.producer.as_deref()
    }
}

/// The subject projection of one recognized elementwise epilogue chain.
///
/// It carries the producer's own subject rather than a summary of it, so a
/// region of the producer's partition binds against exactly the subject it would
/// bind against if the producer were the whole declared output — which is what
/// lets [`crate::physical`]'s binding recurse instead of restating each
/// producing family's obligations a second time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedEpilogueSubject {
    producer: Box<NormalizedOutputSubject>,
    input_keys: Vec<InputKey>,
    output_key: OutputKey,
    shape: Shape,
    expression: PointwiseF32Expression,
    reads: Vec<(BoundaryRead, LogicalAccess)>,
    members: Vec<SemanticStage>,
    elements: u64,
}

impl NormalizedEpilogueSubject {
    /// Returns the producer subject a region of the producing partition binds
    /// against.
    pub(crate) fn producer(&self) -> &NormalizedOutputSubject {
        &self.producer
    }
    /// Returns the epilogue region's iteration domain.
    pub(crate) const fn shape(&self) -> &Shape {
        &self.shape
    }
    /// Returns the recognized epilogue expression.
    pub(crate) const fn expression(&self) -> &PointwiseF32Expression {
        &self.expression
    }
    /// Returns the epilogue region's reads, in access order.
    pub(crate) fn reads(&self) -> &[(BoundaryRead, LogicalAccess)] {
        &self.reads
    }
    /// Returns the occurrences the epilogue region itself covers.
    pub(crate) fn members(&self) -> &[SemanticStage] {
        &self.members
    }
    /// Returns the epilogue region's published element count.
    pub(crate) const fn elements(&self) -> u64 {
        self.elements
    }
}

/// The recognized program as the request subject records it: one per ordered
/// named output, in declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedProgramSubject {
    outputs: Vec<NormalizedOutputSubject>,
}

impl NormalizedProgramSubject {
    /// Returns the recognized output subjects in declaration order.
    pub(crate) fn outputs(&self) -> &[NormalizedOutputSubject] {
        &self.outputs
    }
}

impl VerifiedRequestSubject {
    pub(crate) const fn normalized(&self) -> &NormalizedProgramSubject {
        &self.normalized
    }

    pub(crate) const fn numerical_contract(&self) -> StrictF32NumericalContract {
        self.numerical_contract
    }

    /// The retained verified semantic identity this subject binds.
    ///
    /// The live-row-major request binding decodes the identity's canonical
    /// shape-environment bytes through the public `decode_shape_env_subject`
    /// to prove a symbol's exact root; exposing the retained identity rather
    /// than a second stored association keeps the environment's one authority
    /// the identity bytes themselves (ADR 0070's boundary: shared schedule IR
    /// carries no `ShapeSymbol` or `ShapeEnvIdentity`).
    pub(crate) const fn semantic_identity(&self) -> &SemanticIdentity {
        &self.semantic_identity
    }

    pub(crate) fn canonical_explain_subject_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // The enclosing domain steps to `v6` because `SemanticIdentity` gained
        // its fifth subject, the shape environment, and this preimage
        // enumerates the subject set positionally: a `v5` reader would take the
        // environment identity's length frame for the output count. An
        // appends-only argument does not close for a field written *before* the
        // count, so this is a domain step.
        //
        // The earlier step to `v5` because the recognized program
        // became a *list* — one implementable region partition per ordered named
        // output — and the list is length-framed ahead of the arms. A `v4`
        // subject encoded exactly one arm with no count, so its first
        // post-identity byte is the arm sub-tag's own length frame while a `v5`
        // subject's is the output count. Nothing rules out a count that happens
        // to frame like a sub-tag length, so this is a domain step rather than
        // an appends-only re-tag: the per-tag injectivity argument that would
        // license the cheaper option does not close, and half a step is worse
        // than none.
        //
        // The earlier step to `v4` because the installed independent
        // semantic-realization authority now participates after lowering
        // authority. A v3 subject did not encode that field at all.
        //
        // The earlier step to `v3` rather than only the per-arm
        // sub-tags, because this recognizer moved two of the three arms' shapes
        // at once *and* gave the serial-sum arm its first sub-tag. A same-domain
        // re-tag would have to argue that a newly tagged arm cannot be read as
        // the untagged one it replaced — the old arm opened with a length-framed
        // input key, and a caller may name an input whatever it likes — and that
        // argument does not close. Stepping the domain makes the separation
        // structural instead.
        bytes.extend_from_slice(b"tiler.compiler.request-subject.v6\0");
        push_slice(&mut bytes, self.semantic_identity.graph().as_bytes());
        push_slice(
            &mut bytes,
            self.semantic_identity.reached_definitions().as_bytes(),
        );
        push_slice(
            &mut bytes,
            self.semantic_identity.admission_provenance().as_bytes(),
        );
        push_slice(
            &mut bytes,
            self.semantic_identity.registry_snapshot().as_bytes(),
        );
        push_slice(
            &mut bytes,
            self.semantic_identity.shape_environment().as_bytes(),
        );
        // The ordered named outputs, counted then written in declaration order.
        // The count is what keeps a two-output subject from framing like a
        // one-output subject followed by the contract encoding, and the order is
        // identity rather than presentation: two programs differing only in
        // which output is declared first are different programs, and the
        // semantic graph identity above already says so.
        push_len(&mut bytes, self.normalized.outputs.len());
        for normalized in &self.normalized.outputs {
            encode_output_subject(&mut bytes, normalized);
        }
        encode_contract(&mut bytes, self.numerical_contract);
        // The stated preference follows the resolved contract, length-framed and
        // in the caller's order, so a reordered list is a different subject.
        push_len(&mut bytes, self.numerical_contracts.stated().len());
        for contract in self.numerical_contracts.stated() {
            encode_contract(&mut bytes, *contract);
        }
        for budget in [
            self.budgets.semantic_values,
            self.budgets.semantic_operations,
            self.budgets.regions,
            self.budgets.host_expression_nodes,
            self.budgets.buffers,
            self.budgets.normalization_rewrites,
            self.budgets.region_members,
            self.budgets.region_boundary_outputs,
            self.budgets.region_live_values,
            self.budgets.region_candidates_per_seed,
            self.budgets.region_expansions,
            self.budgets.region_covers,
        ] {
            bytes.extend_from_slice(&budget.to_be_bytes());
        }
        for budget in [
            self.budgets.region_cover_expansions,
            self.budgets.physical_plan_combinations,
        ] {
            bytes.extend_from_slice(&budget.to_be_bytes());
        }
        bytes.extend_from_slice(self.target_profile.request_subject_bytes());
        bytes.extend_from_slice(&self.capability_schema_version.to_be_bytes());
        push_slice(&mut bytes, self.lowering_registry.as_bytes());
        push_slice(&mut bytes, &self.realization_registry);
        bytes
    }
}

/// Appends one recognized output's complete canonical subject encoding.
///
/// Recursive because a chain's producer, a staged occurrence's operand producer,
/// and a fold's materialized contributor are each themselves a recognized
/// output; the pointwise and contraction arms are flat.
///
/// **The recursion is bounded by the recognizer's sides rule rather than by this
/// function.** [`recognize_epilogue_producer`] is the one entry reached across a
/// materialization edge, and it hands `NoEdge` to both admissions — so a
/// producer's own contributor or staged operand may place no further edge, and
/// the deepest subject this function can be handed is a consumer, its producer,
/// and that producer's flat shape. `sum(sum(sum(x) * 2) * 2)` and
/// `(sum(sum(x) * 2)) * 3` are refused at recognition under
/// `reduction-contributor-depth`, so a chain of chains is not a subject that
/// reaches here. Lifting either `NoEdge` without first making this walk
/// iterative would put program depth on the host stack.
///
/// **Every member run below writes the occurrence ordinal of an attribution
/// atom and not its stage, and that is complete rather than lossy.** Every
/// recognized part in this module is minted at [`SemanticStage::first`] —
/// `plan_elementwise`, [`RecognizedSerialSumMembers::new`], the contraction
/// recognizer, and [`recognize_staged_family`] each mint one — so no two
/// subjects can differ in a stage ordinal and the ordinals alone separate them.
///
/// That holds for a family realized as a region *sequence* too, and the reason
/// is the recognizer's rather than this encoder's: a staged family's stage split
/// is region formation's to enumerate, so the recognized partition names the
/// occurrence and [`NormalizedOutput::owns_region_members`] answers for the
/// atoms formation minted. A recognizer that instead enumerated stages into a
/// partition would break the premise and would have to fold the stage into these
/// runs, which steps this domain's version and moves every pinned request
/// qualifier with it.
pub(super) fn encode_output_subject(bytes: &mut Vec<u8>, normalized: &NormalizedOutputSubject) {
    match normalized {
        // **Two sub-tags, chosen by the contributor source rather than by a
        // field's presence, and the split is the structural control.** A
        // materialized contributor has no producer slot in the
        // `serial-sum-f32.v3` grammar to write, so a subject encoded through
        // that arm would drop the producer entirely: two produced sums differing
        // only in what writes their contributors would collide, and the
        // separation from the *old* population would rest on nothing but an
        // accident of the unread-declared-input run. Matching the source here is
        // what makes the tag decide, before any payload is read.
        NormalizedOutputSubject::SerialSum(normalized) => match &normalized.contributor {
            SerialSumContributorSubject::Materialized(materialized) => {
                encode_produced_serial_sum(bytes, normalized, materialized);
            }
            SerialSumContributorSubject::DeclaredInput(_)
            | SerialSumContributorSubject::PointwisePrologue { .. } => {
                encode_serial_sum(bytes, normalized);
            }
        },
        NormalizedOutputSubject::Pointwise(normalized) => {
            // The sub-tag steps to `v4` because the arm gained each read's
            // access relation, and that fact is *load-bearing for
            // identity*: `a * w` with both inputs declared at the region's
            // shape and `a * broadcast(w)` widening a smaller `w` encode
            // the same input keys, the same result shape, the same
            // expression, and the same element count. Only the access maps
            // separate them, so a subject that omitted them would give two
            // different programs one identity — and leaning on the member
            // list to separate them would be exactly the unstated invariant
            // an identity encoder must not rest on.
            //
            // `v3` stepped because a fixed root family, child family,
            // association, and three leaves became the general expression
            // the recognizer now admits. A `v2` pointwise subject can never
            // be read as a `v3` one, and a `v3` one can never be read as a
            // `v4`.
            //
            // **A `bf16` program takes its own sub-tag rather than stepping
            // this one**, on the contraction, epilogue, and staged arms'
            // argument: an `f32` pointwise subject still encodes to exactly
            // the bytes it did, byte for byte, so no pinned request
            // qualifier moves for a program this vocabulary could already
            // express, and a reader that reaches `pointwise-bf16.v1` is
            // reading a subject the earlier vocabulary could not state.
            // The arithmetic is *in the tag* rather than beside it because
            // the node run that follows is a different vocabulary — sixteen
            // bit constants and four node kinds against thirty-two and seven
            // — so the two runs are not two spellings of one encoding.
            push_slice(
                bytes,
                match &normalized.expression {
                    RecognizedPointwise::F32(_) => b"pointwise-f32.v4".as_slice(),
                    RecognizedPointwise::Bf16(_) => b"pointwise-bf16.v1".as_slice(),
                },
            );
            push_len(bytes, normalized.input_keys.len());
            for key in &normalized.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, normalized.output_key.as_str().as_bytes());
            encode_explain_sourced_shape(bytes, &normalized.shape);
            match &normalized.expression {
                RecognizedPointwise::F32(expression) => {
                    encode_pointwise_expression(bytes, expression);
                }
                RecognizedPointwise::Bf16(expression) => {
                    encode_pointwise_bf16_expression(bytes, expression);
                }
            }
            push_len(bytes, normalized.members.len());
            for atom in &normalized.members {
                bytes.extend_from_slice(&atom.member().0.to_be_bytes());
            }
            bytes.extend_from_slice(&normalized.elements.to_be_bytes());
            encode_elementwise_reads(bytes, normalized.input_keys.len(), &normalized.reads);
        }
        // A third sub-tag rather than a step of the enclosing
        // `request-subject.v2` domain: neither existing arm's bytes move, so
        // a subject encoded before this variant existed still encodes to
        // exactly what it did, and a reader that reaches this tag is reading
        // a subject the earlier vocabulary could not express.
        NormalizedOutputSubject::Contraction(normalized) => {
            push_slice(bytes, b"contraction-f32.v1");
            push_len(bytes, normalized.input_keys.len());
            for key in &normalized.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, normalized.output_key.as_str().as_bytes());
            // Exactly two declarations made every previously admitted subject's
            // distinct, ascending read ordinals recoverably `0, 1`. Keep that
            // branch byte-for-byte unchanged. A wider declaration was refused
            // before this change, and its earlier framed key count selects this
            // fixed two-ordinal run unambiguously; the run separates every
            // two-input subset without moving `contraction-f32.v1`'s old bytes.
            if normalized.input_keys.len() > normalized.reads.len() {
                for read in &normalized.reads {
                    bytes.extend_from_slice(&read.input_ordinal.to_be_bytes());
                }
            }
            for read in &normalized.reads {
                encode_explain_shape(bytes, &read.shape);
            }
            encode_explain_shape(bytes, &normalized.output_shape);
            encode_explain_shape(bytes, &normalized.contracted_shape);
            // The canonical structure encoding, not a projection of it: the
            // index tuples are what ADR 0087 makes the operation's identity,
            // and two structures over one set of shapes are two programs.
            push_slice(bytes, normalized.structure.canonical_encoding().as_bytes());
            for read in &normalized.reads {
                push_len(bytes, read.operand_position);
            }
            push_len(bytes, normalized.members.len());
            for atom in &normalized.members {
                bytes.extend_from_slice(&atom.member().0.to_be_bytes());
            }
            for read in &normalized.reads {
                bytes.extend_from_slice(&read.elements.to_be_bytes());
            }
            bytes.extend_from_slice(&normalized.output_elements.to_be_bytes());
            bytes.extend_from_slice(&normalized.contracted_elements.to_be_bytes());
        }
        // A fourth sub-tag rather than a step of the enclosing
        // `request-subject.v5` domain, and the argument is the contraction
        // arm's: no existing arm's bytes move, so a subject encoded before this
        // variant existed still encodes to exactly what it did, and a reader
        // that reaches this tag is reading a subject the earlier vocabulary
        // could not express. The nested producer is written through this same
        // function, so a chain's producer encodes exactly as the standalone
        // output of that family would — which is what keeps the two spellings of
        // one fold from acquiring two identities.
        NormalizedOutputSubject::Epilogue(normalized) => {
            push_slice(bytes, b"epilogue-f32.v1");
            push_len(bytes, normalized.input_keys.len());
            for key in &normalized.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, normalized.output_key.as_str().as_bytes());
            encode_explain_shape(bytes, &normalized.shape);
            encode_pointwise_expression(bytes, &normalized.expression);
            encode_boundary_reads(bytes, &normalized.reads);
            push_len(bytes, normalized.members.len());
            for atom in &normalized.members {
                bytes.extend_from_slice(&atom.member().0.to_be_bytes());
            }
            bytes.extend_from_slice(&normalized.elements.to_be_bytes());
            encode_output_subject(bytes, &normalized.producer);
        }
        // A fifth sub-tag, on the contraction and epilogue arms' argument: no
        // existing arm's bytes move, so a subject encoded before this variant
        // existed still encodes to exactly what it did, and a reader that
        // reaches this tag is reading a subject the earlier vocabulary could not
        // express. The enclosing domain therefore does not step and no pinned
        // request qualifier moves.
        //
        // **The operation key and the attribute record are both identity, and
        // neither is redundant with the other.** Two families realized as region
        // sequences over the same shapes differ only in the key; two occurrences
        // of *one* family differ only in the record — `tiler::rms-norm-f32@1`'s
        // reduced axis and exact `eps` payload live there and are part of what
        // the occurrence computes. The record is written through
        // [`crate::region::encode_attributes`], the same canonical encoder
        // region content identity uses, so the two never disagree about what an
        // attribute value is.
        NormalizedOutputSubject::Staged(normalized) => {
            let occurrence = normalized.occurrence();
            // **The sub-tag steps to `v2`, and the step is forced.** An operand
            // entry used to open with its declared input ordinal and now opens
            // with the boundary-role tag that says whether there *is* one, so
            // every byte string this arm could already produce moves. The
            // per-tag injectivity argument that licenses a same-domain re-tag —
            // "no already-encodable subject's bytes move" — therefore does not
            // close, and half a step is worse than none.
            //
            // Only this arm's bytes move. The enclosing
            // `tiler.compiler.request-subject.v5` domain does not step, because
            // a program naming no staged family encodes exactly what it did, and
            // no pinned request qualifier encodes a staged subject:
            // `explain`'s `deterministic_trace_is_sealed_and_rendered_separately`
            // qualifies a multiply and `tiler-build`'s standard Metal goldens
            // qualify a reduction.
            push_slice(bytes, b"staged-family.v2");
            push_slice(bytes, occurrence.operation.namespace().as_bytes());
            push_slice(bytes, occurrence.operation.name().as_bytes());
            bytes.extend_from_slice(&occurrence.operation.semantic_version().to_be_bytes());
            push_slice(bytes, &occurrence.attributes);
            push_len(bytes, occurrence.input_keys.len());
            for key in &occurrence.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, occurrence.output_key.as_str().as_bytes());
            // The operand run: which boundary tensor supplies each operand, at
            // which shape and element count. Position is identity because the
            // family reads its operands by position — `rms_norm(x, w)` and
            // `rms_norm(w, x)` are different programs — the ordinals are the
            // program's own, which is what the ABI binds, and the role tag is
            // identity because `rms_norm(x, w)` and `rms_norm(matmul(a, b), w)`
            // agree on every other field of this entry.
            //
            // The two tags are the epilogue arm's, for the reason they are one
            // vocabulary rather than two: the same [`BoundaryRead`] is written,
            // and the arm's own sub-tag separates the two runs before either is
            // read.
            push_len(bytes, occurrence.operand_reads.len());
            for ((read, shape), elements) in occurrence
                .operand_reads
                .iter()
                .zip(&occurrence.operand_shapes)
                .zip(&occurrence.operand_elements)
            {
                encode_boundary_read(bytes, *read);
                encode_explain_shape(bytes, shape);
                bytes.extend_from_slice(&elements.to_be_bytes());
            }
            encode_explain_shape(bytes, &occurrence.output_shape);
            bytes.extend_from_slice(&occurrence.member.0.to_be_bytes());
            bytes.extend_from_slice(&occurrence.output_elements.to_be_bytes());
            // The producer, present exactly when some operand above is staged,
            // written through this same function so it encodes exactly as the
            // standalone output of its family would — the epilogue arm's own
            // property, and what keeps two spellings of one contraction from
            // acquiring two identities. The presence byte leads so the arm stays
            // self-delimiting.
            match normalized.producer() {
                Some(producer) => {
                    bytes.push(0x01);
                    encode_output_subject(bytes, producer);
                }
                None => bytes.push(0x00),
            }
        }
    }
}

/// Appends the canonical `serial-sum-produced-f32.v1` encoding of one fold whose
/// contributors another region materializes.
///
/// **A sixth sub-tag rather than a step of the enclosing
/// `tiler.compiler.request-subject.v6` domain, and rather than a widening of
/// `serial-sum-f32.v3`.** No existing arm's bytes move — a fold over a declared
/// input or a pointwise prologue still encodes to exactly what it did, byte for
/// byte, so the `domains.rs` request-subject pin row and every pinned request
/// qualifier hold — and a reader that reaches this tag is reading a subject the
/// earlier vocabulary could not express at all.
///
/// **Widening the old arm was the alternative, and it fails on both counts.**
/// Adding a trailing producer presence byte to every serial-sum subject moves the
/// bytes of every fold already encodable, which is the forced-not-chosen standard
/// the pointwise and staged arms are held to. And writing a produced sum through
/// the old grammar without such a byte would drop the producer entirely: two
/// produced sums differing only in what writes their contributors would collide,
/// and their separation from the old population would rest on the *accident* that
/// a dropped-producer forgery emits an [`UNREAD_DECLARED_INPUT_TAG`] for every
/// declared ordinal — a run no legal old subject produces, because every legal
/// fold reads at least one declared input in its own regions. Resting identity on
/// that is exactly what [`encode_elementwise_reads`]'s own comment forbids, so
/// the framed tag is the control and the marker run is not asked to be one.
///
/// The producer is written through [`encode_output_subject`], so a nested fold,
/// contraction, or staged occurrence encodes exactly as the standalone output of
/// its family would — the epilogue arm's property, and what keeps two spellings
/// of one producer from acquiring two identities. The continuation follows it
/// under a leading presence byte, so the arm stays self-delimiting whether or not
/// an expression stands between the producer and the fold.
fn encode_produced_serial_sum(
    bytes: &mut Vec<u8>,
    normalized: &NormalizedSerialSumSubject,
    materialized: &MaterializedContributorSubject,
) {
    push_slice(bytes, b"serial-sum-produced-f32.v1");
    push_len(bytes, normalized.input_keys.len());
    for key in &normalized.input_keys {
        push_slice(bytes, key.as_str().as_bytes());
    }
    push_slice(bytes, normalized.output_key.as_str().as_bytes());
    encode_explain_shape(bytes, &normalized.input_shape);
    encode_explain_shape(bytes, &normalized.output_shape);
    push_len(bytes, normalized.reduction_axes.len());
    for axis in &normalized.reduction_axes {
        bytes.extend_from_slice(&axis.get().to_be_bytes());
    }
    // The reduction's own occurrences only. This arm's pointwise part is empty
    // by construction — a declared-input prologue is a *different* contributor
    // source — so writing it would be a framed zero stating nothing, and the
    // continuation's occurrences travel with the continuation below.
    push_len(bytes, normalized.members.reduction().len());
    for atom in normalized.members.reduction() {
        bytes.extend_from_slice(&atom.member().0.to_be_bytes());
    }
    bytes.extend_from_slice(&normalized.input_elements.to_be_bytes());
    bytes.extend_from_slice(&normalized.output_elements.to_be_bytes());
    encode_output_subject(bytes, materialized.producer());
    match materialized.continuation() {
        Some(continuation) => {
            bytes.push(0x01);
            encode_pointwise_expression(bytes, &continuation.expression);
            encode_boundary_reads(bytes, &continuation.reads);
            push_len(bytes, continuation.members.len());
            for atom in &continuation.members {
                bytes.extend_from_slice(&atom.member().0.to_be_bytes());
            }
        }
        None => bytes.push(0x00),
    }
}

/// Appends one region's [`BoundaryRead`] run, in access order.
///
/// Both halves of each entry are identity: two regions whose reads bind the same
/// tensors in a different order serve different expression leaves from the same
/// buffers, and a staged read and a declared-input read at the same position bind
/// different buffers. One definition rather than three, because the epilogue
/// arm, the staged arm's operand run, and a fold's continuation write the same
/// vocabulary and each arm's own sub-tag separates the runs before any is read.
fn encode_boundary_reads(bytes: &mut Vec<u8>, reads: &[(BoundaryRead, LogicalAccess)]) {
    push_len(bytes, reads.len());
    for (read, map) in reads {
        encode_boundary_read(bytes, *read);
        encode_access_relation(bytes, map);
    }
}

/// Appends one boundary tensor role under its canonical tag.
fn encode_boundary_read(bytes: &mut Vec<u8>, read: BoundaryRead) {
    match read {
        BoundaryRead::Staged => bytes.push(0x01),
        BoundaryRead::Input(ordinal) => {
            bytes.push(0x02);
            bytes.extend_from_slice(&ordinal.to_be_bytes());
        }
    }
}

/// Appends the canonical `serial-sum-f32.v3` encoding of one fold whose
/// contributors are a declared input or a pointwise prologue over declared
/// inputs.
///
/// **These bytes do not move**, which is what keeps the fold this vocabulary
/// could already express at the request qualifier it already had.
fn encode_serial_sum(bytes: &mut Vec<u8>, normalized: &NormalizedSerialSumSubject) {
    // **The sub-tag holds at `v3` although the arm gained an
    // absent prologue, and the forced-not-chosen standard is what
    // decides that.** A prologue is written below as its framed
    // node run, and
    // `tiler_ir::schedule::PointwiseF32ExpressionBuilder::build`
    // refuses an expression with no node — so every subject this
    // arm could encode before carries a node count of at least
    // one at that position. Writing the absent prologue as a
    // framed *zero* therefore occupies a byte string no
    // previously encodable subject can produce, and the run stays
    // self-delimiting: a count of zero ends the prologue and a
    // count of `n` is followed by exactly `n` nodes and the root.
    // Per-tag injectivity closes, no already-encodable subject's
    // bytes move, and a step would restate every pin for a
    // separation the encoding already has.
    //
    // The earlier step to `v3` was forced, and by the shape this
    // one is not: the access-relation run is written at the arm's
    // *end*, so a `v2` subject and a `v3` one carrying no maps
    // would have differed only by a trailing framed zero, and a
    // reader with the old framing would have consumed the
    // following output's tag as this arm's payload.
    push_slice(bytes, b"serial-sum-f32.v3");
    push_len(bytes, normalized.input_keys.len());
    for key in &normalized.input_keys {
        push_slice(bytes, key.as_str().as_bytes());
    }
    push_slice(bytes, normalized.output_key.as_str().as_bytes());
    encode_explain_shape(bytes, &normalized.input_shape);
    encode_explain_shape(bytes, &normalized.output_shape);
    push_len(bytes, normalized.reduction_axes.len());
    for axis in &normalized.reduction_axes {
        bytes.extend_from_slice(&axis.get().to_be_bytes());
    }
    match normalized.contributor.prologue() {
        Some(prologue) => encode_pointwise_expression(bytes, prologue),
        None => push_len(bytes, 0),
    }
    for members in [
        normalized.members.pointwise(),
        normalized.members.reduction(),
    ] {
        push_len(bytes, members.len());
        for atom in members {
            bytes.extend_from_slice(&atom.member().0.to_be_bytes());
        }
    }
    bytes.extend_from_slice(&normalized.input_elements.to_be_bytes());
    bytes.extend_from_slice(&normalized.output_elements.to_be_bytes());
    // The declared inputs this fold's own regions read: the prologue
    // region's read list, or — when there is no prologue region — the
    // fold's own contributor read. One run rather than two fields
    // because what the arm must separate is *which* declared inputs
    // this output reads, and `sum(a)` and `sum(b)` over the same two
    // declarations agree on every other field here.
    //
    // The contributor read's relation is spelled `LinearIdentity`
    // rather than the `ReductionContributor` the region carries, and
    // that is not one fact encoded twice: the arm has already written
    // the contributor domain, the published domain, and the canonical
    // reduction axes, which is everything that relation is derived
    // from. What the entry contributes is the ordinal — and the run
    // omits a lone dense read, so what reaches the bytes is the marker
    // for each declared input the fold does not read.
    //
    // The materialized source never reaches here: it takes its own tag,
    // where the producer is written rather than dropped, so this run's
    // two sources are the two the enclosing match admits.
    let contributor = normalized
        .contributor
        .declared_input()
        .map(|ordinal| [(ordinal, LogicalAccess::LinearIdentity)]);
    let reads = contributor
        .as_ref()
        .map_or(normalized.contributor.prologue_reads(), <[_; 1]>::as_slice);
    encode_elementwise_reads(bytes, normalized.input_keys.len(), reads);
}

/// Appends one numerical contract's complete canonical encoding.
///
/// Complete over every dimension and exhaustive per dimension: each dimension is
/// written through [`StrictF32NumericalContract::behaviour`] and
/// [`DimensionBehaviour::encode`], whose matches are exhaustive over every
/// behaviour space, and the dimensions are walked in
/// [`crate::target::honourability::CANONICAL_DIMENSIONS`] order. The contract key is
/// encoded beside the field values it names and never in place of them, and the
/// arithmetic type keying every resolution is encoded too — two contracts that
/// resolve the same dimensions for different dtypes are different contracts
/// (ADR 0076 item 6).
///
/// Walking the canonical order rather than listing fields is what makes adding a
/// dimension a build error at `behaviour` instead of a silent omission here.
pub(super) fn encode_contract(bytes: &mut Vec<u8>, contract: StrictF32NumericalContract) {
    push_slice(bytes, contract.key.as_bytes());
    bytes.push(contract.arithmetic.tag());
    bytes.extend_from_slice(&contract.canonical_arithmetic_nan_bits.to_be_bytes());
    push_len(
        bytes,
        crate::target::honourability::CANONICAL_DIMENSIONS.len(),
    );
    for dimension in crate::target::honourability::CANONICAL_DIMENSIONS {
        bytes.push(dimension.tag());
        contract.behaviour(dimension).encode(bytes);
    }
}

// The transform-permission tag is `tiler_ir::numerics`'. It was duplicated
// here and in the artifact's own encoder; the request subject,
// the target-profile descriptor, and the delivered-realization record all
// encode the same behaviours, so one definition is what keeps them from
// drifting. Both remain exhaustive matches rather than discriminant casts, for
// the reason the relocated definitions record: a cast reads whatever ordinal a
// variant happens to occupy, so adding or reordering one would silently restate
// every encoded subject (ADR 0074 convention 5b).
pub(crate) use tiler_ir::numerics::permission_tag;

/// Appends one recognized elementwise expression's complete canonical encoding.
///
/// **Complete, and structural rather than summarized.** The node run is written
/// in the expression's own canonical order with each node's operand ordinals, so
/// two expressions that differ in association, in which leaf an operand reads,
/// or in the sharing of a subexpression encode differently — all three are
/// different binary32 functions, and a subject that could not tell them apart
/// would let one artifact stand for two programs.
///
/// The per-node tag is an exhaustive match rather than a discriminant cast, for
/// the reason the relocated tag encoders record: a node added to the
/// vocabulary must stop
/// the build here rather than silently encode under a neighbour's tag.
///
/// **The leading count is never zero**, because a `PointwiseF32Expression` is
/// constructible only through a builder that refuses an empty node run. The
/// serial-sum subject arm relies on that to spell an *absent* prologue as a
/// framed zero without a sub-tag step, so a vocabulary change admitting a
/// node-free expression would have to move that arm's tag.
fn encode_pointwise_expression(bytes: &mut Vec<u8>, expression: &PointwiseF32Expression) {
    push_len(bytes, expression.nodes().len());
    for node in expression.nodes() {
        match node {
            PointwiseF32Node::Input { access } => {
                bytes.push(0x01);
                bytes.extend_from_slice(&access.get().to_be_bytes());
            }
            PointwiseF32Node::Constant { bits } => {
                bytes.push(0x02);
                bytes.extend_from_slice(&bits.to_be_bytes());
            }
            PointwiseF32Node::Add { lhs, rhs } => encode_binary_node(bytes, 0x03, *lhs, *rhs),
            PointwiseF32Node::Multiply { lhs, rhs } => encode_binary_node(bytes, 0x04, *lhs, *rhs),
            PointwiseF32Node::Divide { lhs, rhs } => encode_binary_node(bytes, 0x05, *lhs, *rhs),
            PointwiseF32Node::Exp { argument } => {
                bytes.push(0x06);
                bytes.extend_from_slice(&argument.index().to_be_bytes());
            }
            PointwiseF32Node::Rsqrt { argument } => {
                bytes.push(0x07);
                bytes.extend_from_slice(&argument.index().to_be_bytes());
            }
        }
    }
    bytes.extend_from_slice(&expression.root().index().to_be_bytes());
}

/// Appends one recognized `bf16` elementwise expression's canonical encoding.
///
/// Structurally the same run [`encode_pointwise_expression`] writes and
/// deliberately not the same function: the node vocabularies are different types
/// with different payload widths, and a shared encoder would have to erase one of
/// them to a common shape. The tags overlap by design — the two runs never share
/// a byte string because the arm's own sub-tag separates them before either is
/// read — and a `bf16` constant is written as its sixteen bits rather than
/// widened, so two constants differing only above bit fifteen cannot exist to be
/// confused.
///
/// Written as an exhaustive match for the reason its `f32` sibling is: a node
/// added to the `bf16` vocabulary must stop the build here rather than encode
/// under a neighbour's tag.
fn encode_pointwise_bf16_expression(bytes: &mut Vec<u8>, expression: &PointwiseBf16Expression) {
    push_len(bytes, expression.nodes().len());
    for node in expression.nodes() {
        match node {
            tiler_ir::schedule::PointwiseBf16Node::Input { access } => {
                bytes.push(0x01);
                bytes.extend_from_slice(&access.get().to_be_bytes());
            }
            tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => {
                bytes.push(0x02);
                bytes.extend_from_slice(&bits.to_be_bytes());
            }
            tiler_ir::schedule::PointwiseBf16Node::Add { lhs, rhs } => {
                bytes.push(0x03);
                bytes.extend_from_slice(&lhs.index().to_be_bytes());
                bytes.extend_from_slice(&rhs.index().to_be_bytes());
            }
            tiler_ir::schedule::PointwiseBf16Node::Multiply { lhs, rhs } => {
                bytes.push(0x04);
                bytes.extend_from_slice(&lhs.index().to_be_bytes());
                bytes.extend_from_slice(&rhs.index().to_be_bytes());
            }
        }
    }
    bytes.extend_from_slice(&expression.root().index().to_be_bytes());
}

/// Appends one ordered binary expression node under its canonical tag.
fn encode_binary_node(
    bytes: &mut Vec<u8>,
    tag: u8,
    lhs: tiler_ir::schedule::PointwiseF32NodeId,
    rhs: tiler_ir::schedule::PointwiseF32NodeId,
) {
    bytes.push(tag);
    bytes.extend_from_slice(&lhs.index().to_be_bytes());
    bytes.extend_from_slice(&rhs.index().to_be_bytes());
}

/// The run entry naming a declared input this region's read list does not read.
///
/// It occupies the relation slot, and the slot's tag space is
/// [`encode_access_relation`]'s: that function writes `0x01`, `0x02`, `0x03`,
/// `0x05`, or the wildcard `0x00`, and nothing else, so `0x04` is a byte no run
/// could carry before this entry existed. That disjointness is the whole
/// argument holding `pointwise-f32.v4` and `serial-sum-f32.v3` where they are —
/// a relation added to that encoder must take the wildcard or a tag above this
/// one, never this one.
pub(super) const UNREAD_DECLARED_INPUT_TAG: u8 = 0x04;

/// Request-subject tag for the labelled-draft parametric broadcast carrier.
///
/// Above [`UNREAD_DECLARED_INPUT_TAG`] so the unread-input marker and this
/// relation cannot share a byte. Crate-internal; the `tiler.compiler.request-subject.v6`
/// domain does not step because previously encodable maps keep their payloads.
pub(super) const PARAMETRIC_BROADCAST_ACCESS_TAG: u8 = 0x05;

/// Encodes which declared inputs one whole-program, prologue, or fold region
/// reads, and how.
///
/// The count leads, then each entry gives an input ordinal and either its
/// relation or [`UNREAD_DECLARED_INPUT_TAG`]. **One read of an ordinal,
/// addressing densely, is written as nothing**: the ordinal's absence from the
/// run is that read's canonical spelling, so the empty run means "every declared
/// input is read once, densely".
///
/// **A declared input read by no leaf is written explicitly, and that entry is
/// what keeps the projection injective across this widening.** The recovery rule
/// is "an ordinal absent from the run has one dense read", and its premise was
/// the `elementwise-reads` completeness rule
/// `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs` lifted.
/// Without the marker, three declared inputs and two dense reads would encode
/// alike whether the pair read is `{0, 1}`, `{0, 2}`, or `{1, 2}` — one arm, one
/// byte string, three programs — and leaning on the enclosing subject's graph
/// identity to separate them is exactly the unstated invariant an identity
/// encoder must not rest on.
///
/// **The step is therefore not forced, and this is why.** Writing every read
/// positionally would separate them too, but it moves the bytes of every subject
/// already encodable — an all-dense complete read list writes an empty run today
/// and would write one entry per input — so it costs both sub-tags a version and
/// every governed compilation its request qualifier. The marker moves nothing:
/// a program reading every declared input emits no marker at all, so its bytes
/// are what they were, and a byte string carrying one is a subject the earlier
/// vocabulary could not express. Per-tag injectivity closes, so
/// `pointwise-f32.v4` and `serial-sum-f32.v3` hold rather than step.
///
/// **The recovery, stated so a reader can refute it.** The declared input count
/// is written earlier in the same arm. For each ordinal in `0..declared`: it is
/// read not at all when the run carries its marker; it has one dense read when
/// the run does not name it; and it has exactly its `k` run entries, in run
/// order, otherwise. The two byte strings that would be ambiguous — a lone entry
/// writing `LinearIdentity`, and a marker beside any other entry for one ordinal
/// — are the two this projection never emits.
///
/// The relation is written through a per-variant tag and its own framed payload,
/// so two reads differing in operand shape, result shape, or any decode differ
/// in these bytes. The two structural relations get distinct tags for the reason
/// they are distinct variants: a bijection and a replication are different facts
/// about what a read consumes.
///
/// `declared` is a count rather than an ordinal list because the arm writes the
/// declared input keys immediately before, in the same order. It saturates at
/// [`u32::MAX`] rather than truncating, which no request reaches:
/// [`check_program_budgets`](super::verify::check_program_budgets) bounds a program's declared inputs far below it, so
/// the saturation is unreachable rather than a collision this encoder tolerates.
pub(super) fn encode_elementwise_reads(
    output: &mut Vec<u8>,
    declared: usize,
    reads: &[(DeclaredInputOrdinal, LogicalAccess)],
) {
    let written = || {
        reads
            .iter()
            .enumerate()
            .filter(|(position, (ordinal, map))| {
                *map != LogicalAccess::LinearIdentity
                    || reads
                        .iter()
                        .enumerate()
                        .any(|(other, (seen, _))| other != *position && seen == ordinal)
            })
    };
    let declared = u32::try_from(declared).unwrap_or(u32::MAX);
    let unread =
        || (0..declared).filter(|ordinal| !reads.iter().any(|(seen, _)| seen.get() == *ordinal));
    push_len(output, written().count() + unread().count());
    for (_, (ordinal, map)) in written() {
        output.extend_from_slice(&ordinal.to_be_bytes());
        encode_access_relation(output, map);
    }
    for ordinal in unread() {
        output.extend_from_slice(&ordinal.to_be_bytes());
        output.push(UNREAD_DECLARED_INPUT_TAG);
    }
}

/// Appends one read's access relation under its canonical per-variant tag.
///
/// Split out of [`encode_elementwise_reads`] because an epilogue's read list
/// writes every position unconditionally, so it needs the relation without that
/// run's canonical omission. One definition is what keeps the two spellings from
/// drifting into two tag vocabularies.
///
/// [`LogicalAccess::LinearIdentity`] carries its own tag rather than falling
/// through the wildcard, which is what keeps the dense read distinguishable from
/// a relation this encoder refuses. Both callers reach it: an epilogue read that
/// interposes no relation, and the dense half of a declared input read twice.
///
/// **The named tag space is `0x01`/`0x02`/`0x03`/`0x05`, and `0x04` remains
/// reserved for the unread-input marker.** [`UNREAD_DECLARED_INPUT_TAG`] occupies
/// this slot's `0x04` in [`encode_elementwise_reads`]'s run precisely because no
/// relation can write it here. The parametric carrier takes `0x05` rather than
/// the refusal `0x00`, so two sourced mappings cannot share explain identity.
/// A later relation takes the wildcard or a tag above `0x05`, never `0x04`.
///
/// The domain does not step: `0x05` is a previously unencodable population, and
/// every already-encodable map keeps its bytes. The encoding stays crate-internal.
pub(super) fn encode_access_relation(output: &mut Vec<u8>, map: &LogicalAccess) {
    match map {
        LogicalAccess::ReindexBijection {
            operand_shape,
            result_shape,
            axes,
        } => {
            output.push(0x01);
            encode_explain_shape(output, operand_shape);
            encode_explain_shape(output, result_shape);
            encode_explain_axis_decodes(output, axes);
        }
        LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape,
            axes,
        } => {
            output.push(0x02);
            encode_explain_shape(output, operand_shape);
            encode_explain_shape(output, result_shape);
            encode_explain_axis_decodes(output, axes);
        }
        LogicalAccess::LinearIdentity => output.push(0x03),
        LogicalAccess::ParametricBroadcast {
            operand_shape,
            mapping,
            environment,
        } => {
            output.push(PARAMETRIC_BROADCAST_ACCESS_TAG);
            encode_explain_sourced_shape(output, operand_shape);
            push_slice(output, mapping.canonical_encoding().as_bytes());
            push_slice(output, environment.as_bytes());
        }
        // No other relation can be recorded here. The arm is a refusal to encode
        // rather than a wildcard tag, so a relation added later cannot silently
        // share one of these tags.
        _ => output.push(0x00),
    }
}

/// Encodes one framed run of operand-axis coordinate decodes.
fn encode_explain_axis_decodes(output: &mut Vec<u8>, axes: &[AxisDecode]) {
    push_len(output, axes.len());
    for decode in axes {
        output.extend_from_slice(&decode.divisor.to_be_bytes());
        output.extend_from_slice(&decode.modulus.to_be_bytes());
        output.push(u8::from(decode.mirrored));
    }
}

pub(super) fn encode_explain_shape(output: &mut Vec<u8>, shape: &Shape) {
    push_len(output, shape.rank());
    for extent in shape.extents() {
        output.extend_from_slice(&extent.get().to_be_bytes());
    }
}

/// Encodes one recognized pointwise domain without moving a wholly literal subject's bytes.
///
/// A static boundary takes the existing [`encode_explain_shape`] path, so every
/// previously encodable pointwise subject keeps its qualifier. A symbolic
/// boundary writes tagged extents; that population had no subject before this
/// admission.
fn encode_explain_sourced_shape(output: &mut Vec<u8>, shape: &SourcedShape) {
    if let Some(shape) = shape.as_static() {
        encode_explain_shape(output, shape);
        return;
    }
    push_len(output, shape.rank());
    for extent in shape.extents() {
        match extent {
            SourcedExtent::Static(extent) => {
                output.push(0x01);
                output.extend_from_slice(&extent.get().to_be_bytes());
            }
            SourcedExtent::Symbol(symbol) => {
                output.push(0x02);
                push_slice(output, symbol.scope().as_bytes());
                push_slice(output, symbol.name().as_bytes());
            }
            _ => {
                // A new source kind must extend this encoder. The reserved tag
                // keeps an unknown spelling from colliding with Static or Symbol.
                output.push(0x00);
            }
        }
    }
}

impl NormalizedSerialSumSubject {
    pub(crate) const fn input_shape(&self) -> &Shape {
        &self.input_shape
    }
    pub(crate) const fn output_shape(&self) -> &Shape {
        &self.output_shape
    }
    pub(crate) fn reduction_axes(&self) -> &[Axis] {
        &self.reduction_axes
    }
    /// Where this fold's contributors come from.
    pub(crate) const fn contributor(&self) -> &SerialSumContributorSubject {
        &self.contributor
    }
    /// The recognized elementwise prologue the fold's contributors come from, or
    /// `None` when a declared input or another region's result supplies them.
    pub(crate) const fn prologue(&self) -> Option<&PointwiseF32Expression> {
        self.contributor.prologue()
    }
    /// The prologue region's reads, in access order; empty when there is no
    /// prologue region.
    pub(crate) fn prologue_reads(&self) -> &[(DeclaredInputOrdinal, LogicalAccess)] {
        self.contributor.prologue_reads()
    }
    /// The declared input ordinal the fold reads directly, or `None` when some
    /// region materializes its contributors.
    pub(crate) const fn contributor_input(&self) -> Option<DeclaredInputOrdinal> {
        self.contributor.declared_input()
    }
    pub(crate) const fn members(&self) -> &RecognizedSerialSumMembers {
        &self.members
    }
    pub(crate) const fn input_elements(&self) -> u64 {
        self.input_elements
    }
    pub(crate) const fn output_elements(&self) -> u64 {
        self.output_elements
    }
}

#[derive(Clone, Copy)]
pub(super) struct VerifiedRequestAuthorities<'a> {
    pub(super) installed: &'a CompilerCapabilitySnapshot,
    pub(super) realization_laws: &'a FrozenIndexRealizationLawRegistry,
}

pub(super) fn request_subject(
    normalized: &NormalizedProgram,
    semantic_identity: &SemanticIdentity,
    numerical_contracts: &NumericalContractPreference,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: &TargetProfile,
    authorities: VerifiedRequestAuthorities<'_>,
) -> VerifiedRequestSubject {
    #[cfg(test)]
    crate::workcount::REQUEST_SUBJECT_REBUILDS.record();
    let normalized = NormalizedProgramSubject {
        outputs: normalized.outputs().iter().map(output_subject).collect(),
    };
    VerifiedRequestSubject {
        normalized,
        semantic_identity: semantic_identity.clone(),
        numerical_contracts: numerical_contracts.clone(),
        numerical_contract,
        budgets,
        target_profile: target_profile.clone(),
        capability_schema_version: authorities.installed.schema_version,
        lowering_registry: authorities.installed.registry_identity().clone(),
        realization_registry: authorities
            .realization_laws
            .identity()
            .as_bytes()
            .to_vec()
            .into_boxed_slice(),
    }
}

/// Projects one recognized output into the subject the request is bound to.
///
/// Recursive because an epilogue chain carries a whole further recognized output
/// inside it; every other arm is a flat projection.
/// Projects one recognized fold's contributor source into the subject's own.
///
/// Recursive through the materialized arm, because the producer is a whole
/// further recognized output and binds against exactly the subject it would bind
/// against standing alone. The other two arms are flat projections of the facts
/// the fold's own regions are built from.
fn contributor_subject(contributor: &SerialSumContributor) -> SerialSumContributorSubject {
    match contributor {
        SerialSumContributor::DeclaredInput(ordinal) => {
            SerialSumContributorSubject::DeclaredInput(*ordinal)
        }
        SerialSumContributor::PointwisePrologue { expression, reads } => {
            SerialSumContributorSubject::PointwisePrologue {
                expression: expression.clone(),
                reads: reads.clone(),
            }
        }
        SerialSumContributor::Materialized(materialized) => {
            SerialSumContributorSubject::Materialized(Box::new(MaterializedContributorSubject {
                producer: output_subject(&materialized.producer),
                continuation: materialized.continuation.clone(),
            }))
        }
    }
}

pub(super) fn output_subject(normalized: &NormalizedOutput) -> NormalizedOutputSubject {
    match normalized {
        NormalizedOutput::SerialSum(normalized) => {
            NormalizedOutputSubject::SerialSum(NormalizedSerialSumSubject {
                input_keys: normalized.input_keys.clone(),
                output_key: normalized.output_key.clone(),
                input_shape: normalized.input_shape.clone(),
                output_shape: normalized.output_shape.clone(),
                reduction_axes: normalized.reduction_axes.clone(),
                contributor: contributor_subject(&normalized.contributor),
                members: normalized.members.clone(),
                input_elements: normalized.input_elements,
                output_elements: normalized.output_elements,
            })
        }
        NormalizedOutput::Pointwise(normalized) => {
            NormalizedOutputSubject::Pointwise(normalized.clone())
        }
        NormalizedOutput::Contraction(normalized) => {
            NormalizedOutputSubject::Contraction(normalized.clone())
        }
        NormalizedOutput::Staged(normalized) => {
            let mut occurrence = normalized.clone();
            // Cleared here rather than left duplicated: the producer travels in
            // the subject's own recursive slot beside it, and two copies of one
            // recognized shape are two accounts of one fact.
            let producer = occurrence
                .producer
                .take()
                .map(|producer| Box::new(output_subject(&producer)));
            NormalizedOutputSubject::Staged(Box::new(NormalizedStagedSubject {
                occurrence,
                producer,
            }))
        }
        NormalizedOutput::Epilogue(chain) => {
            NormalizedOutputSubject::Epilogue(Box::new(NormalizedEpilogueSubject {
                producer: Box::new(output_subject(&chain.producer)),
                input_keys: chain.input_keys.clone(),
                output_key: chain.output_key.clone(),
                shape: chain.shape.clone(),
                expression: chain.expression.clone(),
                reads: chain.reads.clone(),
                members: chain.members.clone(),
                elements: chain.elements,
            }))
        }
    }
}
