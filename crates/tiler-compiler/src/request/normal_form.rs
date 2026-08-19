//! The recognized normal forms: one implementable region partition per ordered
//! named output.
//!
//! Each `Normalized*` shape is what recognition produced and what every
//! per-region authority below the boundary resolves against — the occurrences a
//! walk claimed, the tensors it reads, and the domains it iterates. The shapes
//! answer questions *about* a recognized program; the walk that builds them is
//! `recognize` and its per-shape siblings, and the projection identity binds is
//! `subject`.

use super::*;

/// One tensor's position in the declared program-input interface.
///
/// This coordinate is compiler-private because it belongs to the checked
/// semantic request, not to a schedule access list. Keeping it distinct from
/// [`AccessOrdinal`] prevents a local read position from being reused as an ABI
/// binding when a region reads a sparse subset or repeats one declared input.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct DeclaredInputOrdinal(u32);

impl DeclaredInputOrdinal {
    pub(crate) const fn new(ordinal: u32) -> Self {
        Self(ordinal)
    }

    pub(crate) const fn get(self) -> u32 {
        self.0
    }

    pub(super) const fn to_be_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}

#[cfg(test)]
impl PartialEq<u32> for DeclaredInputOrdinal {
    fn eq(&self, other: &u32) -> bool {
        self.get() == *other
    }
}

/// The recognized serial-sum occurrences as canonical region member sets.
///
/// The strategy recognizer already walks the verified program to identify these
/// operations, so the exact occurrences it matched are retained instead of being
/// re-encoded as a fixed role vocabulary downstream. Only the ascending member
/// sets are retained: two programs that `tiler-ir` gives one canonical graph
/// identity may store the prologue's constants in either order, and the
/// recognized coverage must not depend on which spelling the caller authored. A
/// shared constant simply contributes one member instead of two.
///
/// **The prologue set is empty exactly when the fold has no prologue.** A
/// reduction whose contributor tensor is a declared input — `sum(x)` — claims one
/// occurrence and needs one region, so its partition has one part and the empty
/// part is not a member set any cover region may match. That is a fact about the
/// program rather than a degenerate case: [`NormalizedSerialSum::prologue`] is
/// `None` for it, and every derivation that would spell a prologue region reads
/// the option rather than the emptiness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecognizedSerialSumMembers {
    pointwise: Vec<SemanticStage>,
    reduction: Vec<SemanticStage>,
}

impl RecognizedSerialSumMembers {
    /// Binds the recognized prologue's occurrences and the reduction's own.
    pub(super) fn new(pointwise: Vec<SemanticStage>, reduction: u32) -> Self {
        let mut pointwise = pointwise;
        pointwise.sort_unstable();
        pointwise.dedup();
        Self {
            pointwise,
            reduction: vec![SemanticStage::first(SemanticMemberId(reduction))],
        }
    }

    /// Returns the pointwise prologue members in ascending order.
    pub(crate) fn pointwise(&self) -> &[SemanticStage] {
        &self.pointwise
    }

    /// Returns the reduction members in ascending order.
    pub(crate) fn reduction(&self) -> &[SemanticStage] {
        &self.reduction
    }

    /// Returns every recognized member in ascending order.
    pub(crate) fn all(&self) -> Vec<SemanticStage> {
        let mut members: Vec<_> = self
            .pointwise
            .iter()
            .chain(&self.reduction)
            .copied()
            .collect();
        members.sort_unstable();
        members.dedup();
        members
    }
}

/// A verified N-input, one-output `f32` program whose output is a strict serial
/// reduction of a recognized elementwise contributor expression.
///
/// **The prologue is a general expression, not a template.** It is whatever
/// [`recognize_elementwise`] found between the declared inputs and the
/// reduction's operand — any depth, any mix of the recognized families, any
/// number of declared inputs, and shared reads. `input_keys` and `inputs` are
/// parallel and in declaration order, which is the order the expression's input
/// ordinals index and the order the assembled program binds its buffers in.
///
/// **And it is optional, because `sum(x)` has none.** A fold whose operand is a
/// declared input computes nothing before the fold, so there is no expression to
/// carry and no region to build for one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSum {
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// The contributor domain: the shape the prologue writes and the fold reads.
    pub(crate) input_shape: Shape,
    pub(crate) output_shape: Shape,
    pub(crate) reduction_axes: Vec<Axis>,
    /// The recognized elementwise prologue the fold's contributors come from.
    ///
    /// `None` when the fold's operand is a declared input tensor. That is the
    /// typed statement of "there is no prologue region here", and it is what every
    /// prologue-spelling derivation asks: an identity expression standing in for
    /// the absence would let a cover spell a copy kernel whose materialization —
    /// and whose rounding boundary — the caller's program never asked for.
    pub(crate) prologue: Option<PointwiseF32Expression>,
    /// The prologue region's reads, in access order, or empty when there is no
    /// prologue.
    ///
    /// Empty exactly when `prologue` is `None`, for the reason that field is
    /// `None`: a fold over a declared input has no prologue region, so there is
    /// no read list to state and an inhabited one would describe a region no
    /// cover places.
    pub(crate) prologue_reads: Vec<(DeclaredInputOrdinal, LogicalAccess)>,
    /// The declared input ordinal the fold reads directly, or `None` when a
    /// prologue region materializes its contributors.
    ///
    /// **`Some` exactly when [`Self::prologue`] is `None`, and it is the
    /// recognized ordinal rather than zero.** A prologue-less fold's own read is
    /// the one access no read list describes — `prologue_reads` belongs to a
    /// region this program does not have — so without this field the physical
    /// layer had nothing but the declared arity to derive the contributor buffer
    /// from, and derived `Input { ordinal: 0 }`. That was right while every
    /// elementwise walk read every declared input, because such a program
    /// declared exactly one; `sum(b)` beside an independent `a * a` declares two
    /// and folds the second.
    pub(crate) contributor_input: Option<DeclaredInputOrdinal>,
    pub(crate) members: RecognizedSerialSumMembers,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) pointwise_result: ValueId,
    pub(crate) output: ValueId,
    pub(crate) input_elements: u64,
    pub(crate) output_elements: u64,
}

impl NormalizedSerialSum {
    /// The occurrences a prologue region would cover, when this fold has one.
    ///
    /// `None` rather than an empty slice for a fold over a declared input, so a
    /// cover region covering no occurrence cannot match the part. The two answers
    /// are the same bytes and different facts: "the prologue claims nothing" is a
    /// state no recognized program is in, and treating it as one is how an empty
    /// member set would acquire a region.
    pub(crate) fn prologue_members(&self) -> Option<&[SemanticStage]> {
        self.prologue.as_ref().map(|_| self.members.pointwise())
    }
}

/// One recognized per-point expression, in the arithmetic its program states.
///
/// **The arithmetic is carried rather than assumed, and the two vocabularies are
/// separate types rather than one width-tagged one.** A per-point body is a
/// function on a *specific* format — `x * 3.0` rounds differently in binary32 and
/// in `bf16`, and a `bf16` constant is a sixteen-bit pattern that no
/// [`PointwiseF32Node::Constant`] payload can hold — so `tiler_ir::schedule`
/// gives each width its own expression type and its own scheduled-region
/// spelling. This enum is what lets one recognizer walk produce either.
///
/// Every consumer matches it exhaustively rather than projecting it to a tag, so
/// a third admitted width is a build error at each site instead of an expression
/// silently spelled as one of these two.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RecognizedPointwise {
    F32(PointwiseF32Expression),
    Bf16(PointwiseBf16Expression),
}

impl RecognizedPointwise {
    /// The `f32` expression this recognition holds, or a refusal naming the
    /// width it holds instead.
    ///
    /// **Defence in depth rather than a live gate.** The one caller is the fold's
    /// prologue walk, which is entered only from `tiler::strict-serial-sum-f32@1`
    /// and therefore states `f32` at the call site; the refusal exists so that a
    /// fold family admitted at another width fails loudly here instead of the
    /// prologue silently acquiring a spelling its region cannot carry.
    pub(super) fn into_f32(self) -> Result<PointwiseF32Expression, RequestError> {
        match self {
            Self::F32(expression) => Ok(expression),
            Self::Bf16(_) => mismatch("prologue-arithmetic"),
        }
    }

    /// The `f32` expression a fixture asserted about, for the crate's own tests.
    ///
    /// Panics for the other width, like [`NormalizedOutput::serial_sum`] and its
    /// siblings: a fixture whose recognized width changed should fail loudly
    /// here rather than have its assertion quietly skipped.
    #[cfg(test)]
    pub(super) fn f32(&self) -> &PointwiseF32Expression {
        match self {
            Self::F32(expression) => expression,
            Self::Bf16(_) => panic!("the fixture recognizes an f32 expression"),
        }
    }

    /// The `bf16` expression a fixture asserted about, for the crate's own tests.
    #[cfg(test)]
    pub(super) fn bf16(&self) -> &PointwiseBf16Expression {
        match self {
            Self::Bf16(expression) => expression,
            Self::F32(_) => panic!("the fixture recognizes a bf16 expression"),
        }
    }
}

/// A verified N-input, one-output elementwise program.
///
/// `input_keys` and `inputs` are parallel and in the program's declaration
/// order, which is the order the expression's input ordinals index and the order
/// the assembled program binds its buffers in. One sourced `shape` governs every
/// input and the output. A wholly literal boundary still answers
/// [`SourcedShape::as_static`] and then a single element count sizes the region;
/// a symbolic boundary keeps the symbols as written and leaves `elements` at
/// zero, because a launch geometry is not a number anyone authored.
///
/// **`expression` is the recognized program, not a projection of it.** It is the
/// general per-point expression vocabulary rather than a fixed leaf count and
/// association, so what the recognizer admits is bounded by what the physical
/// expression can spell rather than by a shape it was taught. It also carries
/// the arithmetic the program is stated in, because that is what decides which
/// scheduled-region scalar program realizes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedPointwise {
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    pub(crate) shape: SourcedShape,
    pub(crate) expression: RecognizedPointwise,
    pub(crate) members: Vec<SemanticStage>,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) output: ValueId,
    pub(crate) elements: u64,
    /// The region's reads, in access order.
    ///
    /// One entry per expression input leaf, naming the declared input ordinal it
    /// binds and the relation it addresses that tensor with. Ordinals do not
    /// descend and one may appear twice — once densely and once through a
    /// relation — which is how `a * permute(a)` is spelled: two leaves meaning
    /// two different tensors derived from one declared input.
    pub(crate) reads: Vec<(DeclaredInputOrdinal, LogicalAccess)>,
}

/// One declared-input read of a verified binary tensor contraction.
///
/// The declaration ordinal is the ABI binding, while `operand_position` is the
/// position in the contraction occurrence and its canonical index structure.
/// Keeping both beside the operand's value, shape, and count prevents a
/// region-local renumbering from silently changing which program tensor a
/// structure operand reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedContractionRead {
    pub(crate) input_ordinal: DeclaredInputOrdinal,
    pub(crate) shape: Shape,
    pub(crate) elements: u64,
    pub(crate) value: ValueId,
    pub(crate) operand_position: usize,
}

/// A verified binary tensor-contraction `f32` shape over two distinct declared
/// inputs and one semantic result.
///
/// **The structure is carried whole, not projected.** ADR 0087 makes the
/// canonical index structure the operation's identity, so a normalization that
/// kept only the extents it happened to need would let two different structures
/// over the same shapes share a request subject. `reads` is ordered by strictly
/// ascending declared-input ordinal. Each entry names the structure operand it
/// supplies; the complete declaration remains in `input_keys` so those ordinals
/// keep their program-wide meaning when another output reads an input this
/// contraction does not.
///
/// `output_shape` and `contracted_shape` are derived from the structure and the
/// operand shapes rather than read from the graph, and the derived output shape
/// is required to equal the program's own: the semantic inferencer already
/// proved them equal at construction, so a disagreement here is invalid state
/// and is refused rather than resolved in favour of either side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedContraction {
    /// The complete declared-input list, in program declaration order.
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// The two distinct operand reads, ordered by declared input ordinal.
    pub(crate) reads: [NormalizedContractionRead; 2],
    pub(crate) output_shape: Shape,
    /// Row-major shape of the contracted iteration space, ascending by
    /// canonical contracted index.
    pub(crate) contracted_shape: Shape,
    pub(crate) structure: ContractionIndexStructure,
    pub(crate) members: Vec<SemanticStage>,
    pub(crate) output: ValueId,
    pub(crate) output_elements: u64,
    /// Points of the contracted iteration space; the fold length per output.
    pub(crate) contracted_elements: u64,
}

/// Which boundary tensor one recognized read binds.
///
/// **Local access position and declared association are separate facts in every
/// recognized shape.** [`canonical_input_reads`](super::elementwise::canonical_input_reads) orders a whole-program or
/// prologue run by declaration group and dense-before-mapped relation, while
/// omitting unread declarations and preserving distinguishable repeated reads.
/// Leaf position and declared ordinal therefore need not coincide. An epilogue
/// additionally reads the value an earlier region staged. In both cases the
/// *position* of a read — the leaf it serves — and the *tensor* it binds are
/// independent. `tiler_ir::schedule`'s `reads_bind_boundary_tensors_in_order`
/// checks only each fieldless boundary category; it has no declared-interface
/// association to resolve. `crate::program::CoverAssembly::from_plan` constructs
/// the exact [`AccessOrdinal`] from the read position and projects it through
/// [`crate::physical::VerifiedScheduledRegion::declared_input_at`]'s retained,
/// checked request subject.
///
/// **Two recognized shapes carry it, and the separation is the same fact in
/// both.** An epilogue's read list names the tensor each expression leaf binds;
/// a staged family's operand run names the tensor each *occurrence operand*
/// binds, because
/// [`admit-a-staged-family-that-reads-a-materialized-intermediate`](../../../tickets/admit-a-staged-family-that-reads-a-materialized-intermediate.md)
/// made an operand's source a recognition-time property. One vocabulary rather
/// than two is what keeps `crate::program::CoverAssembly::from_plan` resolving
/// one kind of role against the cover, and what keeps
/// [`Self::tensor`] the single statement of the mapping onto
/// [`TensorRole`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundaryRead {
    /// The value the producer region staged, bound to the materialization edge
    /// the cover hands this region.
    ///
    /// Carries no ordinal because [`TensorRole::Intermediate`] carries none: a
    /// region reads at most one staged value, which is exactly what makes the
    /// unordinalled role sufficient and a second staged read inadmissible.
    Staged,
    /// The declared program input at this ordinal.
    Input(DeclaredInputOrdinal),
}

impl BoundaryRead {
    /// Returns the boundary tensor this read binds.
    pub(crate) const fn tensor(self) -> TensorRole {
        match self {
            Self::Staged => TensorRole::Intermediate,
            Self::Input(_) => TensorRole::Input,
        }
    }

    /// Returns the declared input ordinal this read binds, or `None` for the
    /// staged one.
    pub(crate) const fn declared_ordinal(self) -> Option<DeclaredInputOrdinal> {
        match self {
            Self::Staged => None,
            Self::Input(ordinal) => Some(ordinal),
        }
    }
}

/// A verified `f32` program output that is an elementwise expression over a
/// value an earlier region produces.
///
/// **The chain is the recognized shape, not two shapes that happen to compose.**
/// `matmul(a, b) * 2.0` and `sum(x * x) * scale` are one declared output each,
/// and neither the contraction nor the fold publishes anything: their result is
/// a materialization edge some cover places, and the epilogue is the region that
/// consumes it. Carrying the producer *inside* this shape is what makes "which
/// recognized partition does this region belong to" answerable for both halves
/// from one place, and what lets every region builder, cost, and subject binding
/// the producing family already has apply to the producer unchanged.
///
/// **The producer is a folding family, and only ever those two.** A pointwise
/// producer is not a materialization boundary at all — its occurrences are part
/// of the epilogue's own walk, and fusing them is the whole point of the
/// expression vocabulary. [`recognize_epilogue_producer`] is where that is
/// enforced, and the `NormalizedOutput` typing here is a convenience for the
/// consumers rather than a claim that any variant may appear.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedEpilogue {
    /// The producer whose staged result this epilogue reads.
    ///
    /// Its `output_key` is *this* chain's published key. The producer names no
    /// key of its own — it publishes nothing — and the field means "the ordered
    /// named output the partition this shape belongs to publishes", which is the
    /// producer's own key exactly when the producer is the whole output.
    pub(crate) producer: Box<NormalizedOutput>,
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// The epilogue region's iteration domain, which is the published shape.
    pub(crate) shape: Shape,
    pub(crate) expression: PointwiseF32Expression,
    /// One entry per expression input leaf, in access order.
    ///
    /// Parallel to the region's reads: leaf `i` is served by entry `i`, which
    /// names the boundary tensor it binds and the relation it addresses that
    /// tensor with. Exactly one entry is [`BoundaryRead::Staged`].
    pub(crate) reads: Vec<(BoundaryRead, LogicalAccess)>,
    /// The occurrences the epilogue region itself covers.
    pub(crate) members: Vec<SemanticStage>,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) output: ValueId,
    pub(crate) elements: u64,
}

/// A verified `f32` value produced by one occurrence of a registered family
/// whose realization law realizes a region *sequence*.
///
/// **The classification is the law's, not a family list's.** What makes an
/// occurrence this shape is that
/// [`FrozenIndexRealizationLawRegistry::family_realizes_region_sequence`]
/// answers true for its operation key — so every family the registry carries a
/// multi-region law for is recognized by the same arm, and a family added to the
/// registry becomes recognizable without a line here. No operation key is named
/// in the recognizer, which is what keeps the capability the general one.
///
/// **It carries the occurrence and not the stage split, because the stage split
/// belongs to region formation.** Tom's Option A′ decision made
/// [`crate::region::RegionGraph::with_realizations`] the authority that reads
/// each law's realized sequence and enumerates one candidate per stage; a
/// recognizer that also enumerated them would be a second account of one fact,
/// and the two would have to agree about stage counts, sources, and handed
/// values for either to mean anything. So this shape claims
/// [`SemanticStage::first`] — the occurrence — and
/// [`NormalizedOutput::owns_region_members`] answers for whichever stage atoms
/// formation actually minted.
///
/// **One operand may be a value another region materializes, and the shape says
/// which.** `rms_norm(matmul(a, b), w)` reads its first operand across a
/// materialization edge rather than from a declared buffer, so
/// [`Self::operand_reads`] carries a per-operand [`BoundaryRead`] and
/// [`Self::producer`] carries the recognized shape whose regions write it. Both
/// are recognition-time facts and both live here, which is the resolution
/// [`admit-a-staged-family-that-reads-a-materialized-intermediate`](../../../tickets/admit-a-staged-family-that-reads-a-materialized-intermediate.md)
/// chose over deriving the operand's source from the cover's materialization
/// edges: an operand supplied by no declared input and no recognizable producer
/// is a property of the *program*, and a stage that discovered it later could
/// only report it as a cover it could not assemble.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedStaged {
    /// The recognized shape producing the value one operand reads, when an
    /// operand is a materialized intermediate.
    ///
    /// `None` exactly when every entry of [`Self::operand_reads`] is
    /// [`BoundaryRead::Input`]. It is carried for the reason
    /// [`NormalizedEpilogue::producer`] is carried: the producing occurrences
    /// belong to *this* output's walk — nothing else claims them, and
    /// [`check_output_cover`](super::recognize::check_output_cover) refuses a program with an occurrence no walk
    /// claimed — so the region a cover places for them has to be spelled from a
    /// recognized shape this partition holds.
    ///
    /// Its `output_key` is this occurrence's own, for the reason
    /// [`NormalizedEpilogue::producer`] states: a producer publishes nothing and
    /// the field means the ordered named output the partition it belongs to
    /// publishes.
    pub(crate) producer: Option<Box<NormalizedOutput>>,
    /// The registered family this occurrence belongs to.
    pub(crate) operation: OpKey,
    /// The law the registry carries for that family.
    ///
    /// **Read once, here, from the same registry row the admission above reads.**
    /// A scheduled region for one of these stages has to know what the stage
    /// computes — which axes it folds, which payload its epilogue carries — and
    /// that is the law's content rather than the occurrence's shape: a `[2, 2]`
    /// operand reduced to `[2]` names two different reductions, so no derivation
    /// from shapes can recover it. Carrying the law is what lets the physical
    /// layer be written against the closed law vocabulary — one arm per law, a
    /// fail-closed wildcard for the rest — instead of against a family list.
    ///
    /// It is *not* a second account of the realization. The stage count, each
    /// stage's reads, and the handed values stay
    /// [`crate::region::RegionGraph::with_realizations`]'s, read off the law's own
    /// realized sequence; this field is the law itself, which is one value with
    /// one owner however many readers it has.
    pub(crate) law: IndexRealizationLaw,
    /// The occurrence's attribute record in canonical bytes.
    ///
    /// Carried whole rather than projected. `tiler::rms-norm-f32@1` declares its
    /// reduced axis and its exact `eps` payload here, both of which are part of
    /// what the occurrence computes, and a subject that dropped them would give
    /// two different normalizations one identity.
    pub(crate) attributes: Box<[u8]>,
    /// The same record, typed.
    ///
    /// Beside the canonical bytes rather than instead of them, because the two
    /// serve different readers and neither derives the other cheaply: identity
    /// binds the bytes, and the law names the *fields* it interprets by
    /// identifier, which only the typed record can answer. They cannot disagree —
    /// the bytes are this record's own canonical encoding.
    pub(crate) attribute_record: OperationAttributes,
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// Boundary tensor supplying each occurrence operand, in operand order.
    ///
    /// At most one entry is [`BoundaryRead::Staged`], and the bound is the
    /// unordinalled [`TensorRole::Intermediate`]'s rather than a simplification:
    /// a second staged operand — one value read twice, or two different
    /// materialized values — has nothing to say which edge each read binds.
    /// [`recognize_staged_family`]'s `staged-operand-conflict` refusal is that
    /// boundary and [`Self::producer`] is the one edge that survives it.
    pub(crate) operand_reads: Vec<BoundaryRead>,
    /// Operand shapes, in operand order.
    pub(crate) operand_shapes: Vec<Shape>,
    /// The published shape of the occurrence's one result.
    pub(crate) output_shape: Shape,
    /// The occurrence this walk claimed.
    pub(crate) member: SemanticMemberId,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) output: ValueId,
    /// Operand element counts, in operand order.
    pub(crate) operand_elements: Vec<u64>,
    pub(crate) output_elements: u64,
}

impl NormalizedStaged {
    /// Returns whether one region's members are exactly stages of this
    /// occurrence.
    ///
    /// **Narrower than [`NormalizedOutput::owns_region_members`] on purpose, and
    /// the difference is the producer.** That predicate answers for every region
    /// of this output's partition, which since a staged operand became
    /// recognizable includes the producer's own regions; this one answers only
    /// for the stages *this* occurrence realizes as. [`crate::physical`] asks
    /// this before it names a stage spelling, because a producer region resolved
    /// to this output belongs to the producer's family and not to the law's two
    /// stages.
    ///
    /// The stage list is deliberately not enumerated here: region formation
    /// decided how many stages there are and which atoms exist, so asking for a
    /// stage list would be a second account of that decision, free to disagree
    /// with the candidates actually enumerated. An empty member set is no region
    /// of this occurrence.
    pub(crate) fn owns_stage_members(&self, members: &[SemanticStage]) -> bool {
        !members.is_empty() && members.iter().all(|atom| atom.member() == self.member)
    }

    /// Returns each operand that binds a declared input, as `(ordinal, count)`.
    ///
    /// The staged operand is skipped rather than reported at some ordinal: its
    /// element count sizes a materialization edge, not a declared buffer, and a
    /// caller scaling work over a declared input must not receive it.
    fn declared_operands(&self) -> impl Iterator<Item = (DeclaredInputOrdinal, u64)> + '_ {
        self.operand_reads
            .iter()
            .zip(&self.operand_elements)
            .filter_map(|(read, elements)| Some((read.declared_ordinal()?, *elements)))
    }
}

/// One recognized ordered named program output, and the region partition that
/// implements it.
///
/// **A property of one output, not of the program.** Each variant carries the
/// occurrences its own walk claimed, partitioned into the parts a region can be
/// spelled from — one part for the two single-region shapes, the prologue and
/// the fold for a reduction. [`NormalizedProgram`] holds one of these per
/// declared output, in declaration order, so "which strategy implements this
/// cover region" is answered by the part whose members the region covers rather
/// than by asking which whole-program template matched.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedOutput {
    SerialSum(NormalizedSerialSum),
    Pointwise(NormalizedPointwise),
    /// Boxed because a contraction carries two operand shapes, an output shape,
    /// a contracted shape, and a validated index structure — roughly twice the
    /// serial sum's payload — and every value of this enum would otherwise pay
    /// for the widest variant.
    Contraction(Box<NormalizedContraction>),
    /// An elementwise expression over a value a folding region stages.
    ///
    /// Boxed because it carries a whole further recognized output inside it,
    /// which would otherwise make every value of this enum the size of two.
    Epilogue(Box<NormalizedEpilogue>),
    /// One occurrence of a registered family realized as a region sequence, and
    /// the shape producing the value one operand reads when that operand is a
    /// materialized intermediate.
    ///
    /// Boxed because it carries an operand-indexed shape list, an element-count
    /// list, the occurrence's canonical attribute bytes, and a whole further
    /// recognized output, none of which the other variants pay for.
    Staged(Box<NormalizedStaged>),
}

impl NormalizedOutput {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) | Self::Staged(_) => {
                panic!("request is not a serial-sum program")
            }
        }
    }

    pub(crate) const fn try_serial_sum(&self) -> Option<&NormalizedSerialSum> {
        match self {
            Self::SerialSum(normalized) => Some(normalized),
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) | Self::Staged(_) => None,
        }
    }

    pub(crate) const fn pointwise(&self) -> Option<&NormalizedPointwise> {
        match self {
            Self::SerialSum(_) | Self::Contraction(_) | Self::Epilogue(_) | Self::Staged(_) => None,
            Self::Pointwise(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn contraction(&self) -> Option<&NormalizedContraction> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Epilogue(_) | Self::Staged(_) => None,
            Self::Contraction(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn epilogue(&self) -> Option<&NormalizedEpilogue> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Contraction(_) | Self::Staged(_) => {
                None
            }
            Self::Epilogue(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn staged(&self) -> Option<&NormalizedStaged> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) => {
                None
            }
            Self::Staged(normalized) => Some(normalized),
        }
    }

    /// Returns whether some recognized read of this output is the labelled-draft
    /// parametric broadcast carrier.
    fn carries_parametric_broadcast(&self) -> bool {
        match self {
            Self::Pointwise(normalized) => normalized
                .reads
                .iter()
                .any(|(_, map)| access_is_parametric_broadcast(map)),
            Self::SerialSum(normalized) => normalized
                .prologue_reads
                .iter()
                .any(|(_, map)| access_is_parametric_broadcast(map)),
            Self::Epilogue(chain) => {
                chain
                    .reads
                    .iter()
                    .any(|(_, map)| access_is_parametric_broadcast(map))
                    || chain.producer.carries_parametric_broadcast()
            }
            Self::Staged(normalized) => normalized
                .producer
                .as_deref()
                .is_some_and(Self::carries_parametric_broadcast),
            Self::Contraction(_) => false,
        }
    }

    /// Returns the recognized shape one *producer* region of this output is
    /// built from.
    ///
    /// A chain's producer regions — the fold, its prologue, its split passes,
    /// its cooperative tile, the contraction — are spelled from the producer's
    /// own recognized shape, so every derivation that would otherwise read the
    /// chain asks this instead and reaches the same value it reaches for a
    /// standalone output. The epilogue region is the one part that is not built
    /// from it, and [`crate::physical::RegionSpellingKind::Epilogue`] is what
    /// distinguishes it.
    ///
    /// **It takes the region's members, and it has to since a staged family may
    /// read a materialized intermediate.** Such an output holds two recognized
    /// shapes whose regions a cover both places — the occurrence's own two
    /// stages, and its producer's partition — so "the producer shape" is not a
    /// property of the output alone. The epilogue arm descends unconditionally
    /// because its own region is never spelled through here; the staged arm
    /// descends exactly when the members are not the occurrence's stages, which
    /// is the same question [`crate::physical::spell_region`] answered to reach
    /// this call.
    pub(crate) fn producer_shape_for(&self, members: &[SemanticStage]) -> &Self {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Contraction(_) => self,
            Self::Epilogue(chain) => chain.producer.producer_shape_for(members),
            Self::Staged(normalized) => normalized
                .producer
                .as_deref()
                .filter(|_| !normalized.owns_stage_members(members))
                .map_or(self, |producer| producer.producer_shape_for(members)),
        }
    }

    /// Returns the element count of one declared input tensor this output
    /// reads.
    ///
    /// Per ordinal rather than one shared count, because a contraction's two
    /// operands generally have different extents. Every arm answers only for an
    /// ordinal *this* output reads and `None` for one it does not, so a caller
    /// that names an ordinal this output never loads gets a refusal instead of
    /// another tensor's size.
    ///
    /// **The count is the declared tensor's own, never the iteration domain of
    /// a region that reads it.** Its consumer binds an exact [`AccessOrdinal`]
    /// and projects that access through the retained checked request subject to
    /// this private declared ordinal. A count taken from the reading region
    /// would scale a call by an iteration space rather than by the tensor the
    /// caller passed. The
    /// two coincide for a dense read — [`plan_elementwise`] refuses a leaf at
    /// any shape but the region's — and diverge exactly for a widening
    /// structural read: `a * broadcast(w)` over a `[2]` weight iterates `[2, 2]`
    /// and must still answer `2` for the ordinal `w` occupies.
    ///
    /// So every arm derives from an *operand* shape. `Contraction` reads its
    /// per-ordinal operand counts and `Staged` its operand run, both of which
    /// are declared operands' own extents. The three arms holding elementwise
    /// read lists ask [`read_tensor_elements`] per read, which answers a
    /// structural read from the relation's own operand shape and declines a
    /// relation it cannot size.
    ///
    /// **The read lists are the gate, not the declared arity, and that is a
    /// per-read truth rather than a per-program one.** `input_keys` is the whole
    /// program's declaration list, so a bound on its length says nothing about
    /// which of those inputs this output's walk reached; the premise that every
    /// declared input of a reduced program is read at the contributor domain
    /// held only while every walk had to read every declared input. Since a walk
    /// may read a subset, an output iterating one domain would otherwise
    /// volunteer its own count for an ordinal only a sibling loads, and
    /// [`NormalizedProgram::agreed_input_elements_at`] would read that
    /// volunteered count as a disagreement and refuse a call the reading output
    /// sizes exactly. [`Self::reads_declared_input`] states the same fact for
    /// the callers that need only the predicate, and the two must keep agreeing
    /// about which ordinals a walk reached.
    ///
    /// **Agreement or nothing wherever one ordinal has several reads**, for the
    /// reason [`NormalizedProgram::agreed_input_elements_at`] states: a half
    /// that reads the ordinal and cannot size it must refuse rather than be
    /// overruled by a half that can. Whether a half *reads* the ordinal is asked
    /// separately from what it answers, so an unsizable read is a `Some(None)`
    /// the fold sees rather than an abstention it drops.
    ///
    /// The disagreement is unreachable for the relations recognized today: every
    /// route above resolves to the shape the program declared the ordinal at, so
    /// the answer is a function of the ordinal alone. That is exhaustive finite
    /// evidence over five arms and three admitted access maps rather than a
    /// proof about relations not yet spelled, which is why the fold stays.
    /// `every_arm_answers_the_declared_tensors_own_count` is what says so, and
    /// says no when an arm reintroduces a domain.
    pub(crate) fn input_elements_at(&self, ordinal: DeclaredInputOrdinal) -> Option<u64> {
        match self {
            // A prologue region's reads address declared tensors from the
            // contributor domain; a prologue-less fold's own contributor read
            // addresses the declared tensor directly, and `input_shape` is that
            // operand's own shape rather than a domain standing in for it. The
            // two sources are mutually exclusive — `prologue_reads` is inhabited
            // exactly when `contributor_input` is `None` — and are folded
            // together anyway so neither has to restate the exclusion.
            Self::SerialSum(normalized) => agreed(
                normalized
                    .prologue_reads
                    .iter()
                    .filter(|(read, _)| *read == ordinal)
                    .map(|(_, map)| read_tensor_elements(map, normalized.input_elements))
                    .chain(
                        (normalized.contributor_input == Some(ordinal))
                            .then_some(Some(normalized.input_elements)),
                    ),
            )
            .flatten(),
            Self::Pointwise(normalized) => agreed(
                normalized
                    .reads
                    .iter()
                    .filter(|(read, _)| *read == ordinal)
                    .map(|(_, map)| read_tensor_elements(map, normalized.elements)),
            )
            .flatten(),
            // A contraction's two explicit reads are a subset of the complete
            // declared interface, so the ordinal map — not declaration length
            // or read position — gates the count.
            Self::Contraction(normalized) => normalized
                .reads
                .iter()
                .find(|read| read.input_ordinal == ordinal)
                .map(|read| read.elements),
            // A chain reads a declared input from its producer, from its
            // epilogue, or from both. Whether each half reads it is asked before
            // what it answers, so a half that reads the ordinal without being
            // able to size it contributes a `None` the fold compares rather than
            // an absence it cannot tell from silence. Neither half reading is
            // the arm's own spelling of "this chain does not read that ordinal".
            Self::Epilogue(chain) => agreed(
                chain
                    .producer
                    .reads_declared_input(ordinal)
                    .then(|| chain.producer.input_elements_at(ordinal))
                    .into_iter()
                    .chain(
                        chain
                            .reads
                            .iter()
                            .any(|(read, _)| *read == BoundaryRead::Input(ordinal))
                            .then(|| {
                                agreed(
                                    chain
                                        .reads
                                        .iter()
                                        .filter(|(read, _)| *read == BoundaryRead::Input(ordinal))
                                        .map(|(_, map)| read_tensor_elements(map, chain.elements)),
                                )
                                .flatten()
                            }),
                    ),
            )
            .flatten(),
            // The operand run is the occurrence's read list, so an operand
            // binding a declared input answers that operand tensor's own count
            // and the staged operand answers nothing — its count sizes a
            // materialization edge rather than a declared buffer.
            //
            // Agreement or nothing, over the occurrence's own operands *and* its
            // producer's answer, for the reason the chain arm above and
            // [`NormalizedProgram::agreed_input_elements_at`] both state: two
            // claimants that cannot name one extent for a tensor give a
            // work-scaling caller no single answer, and the producer is asked
            // only when it reads the ordinal so its silence costs nothing.
            Self::Staged(normalized) => agreed(
                normalized
                    .declared_operands()
                    .filter(|(operand, _)| *operand == ordinal)
                    .map(|(_, elements)| Some(elements))
                    .chain(
                        normalized
                            .producer
                            .as_deref()
                            .filter(|producer| producer.reads_declared_input(ordinal))
                            .map(|producer| producer.input_elements_at(ordinal)),
                    ),
            )
            .flatten(),
        }
    }

    /// Returns whether some region of this output's partition reads one
    /// declared input tensor.
    ///
    /// **Read from the recognized read lists, not from the declared arity.** An
    /// output's regions bind the inputs its own walk reached, so "this program
    /// declares three inputs" says nothing about which of them this output
    /// loads. It is exhaustive over the recognized shapes rather than projected
    /// through one of them, because each carries the fact differently: an
    /// elementwise region in its read list, a fold in its prologue's read list
    /// *or* its own contributor ordinal, a contraction in its operand count, and
    /// a chain in both halves of the chain.
    pub(super) fn reads_declared_input(&self, ordinal: DeclaredInputOrdinal) -> bool {
        match self {
            Self::Pointwise(normalized) => {
                normalized.reads.iter().any(|(read, _)| *read == ordinal)
            }
            Self::SerialSum(normalized) => {
                normalized.contributor_input == Some(ordinal)
                    || normalized
                        .prologue_reads
                        .iter()
                        .any(|(read, _)| *read == ordinal)
            }
            Self::Contraction(normalized) => normalized
                .reads
                .iter()
                .any(|read| read.input_ordinal == ordinal),
            Self::Epilogue(chain) => {
                chain
                    .reads
                    .iter()
                    .any(|(read, _)| *read == BoundaryRead::Input(ordinal))
                    || chain.producer.reads_declared_input(ordinal)
            }
            // The recognized operand map, not the declared arity: a staged
            // occurrence binds one read per operand and a program may declare an
            // input it never names. Its producer's regions are part of this
            // output's partition too, so a declared input only the producer
            // reaches is one this output reads.
            Self::Staged(normalized) => {
                normalized
                    .declared_operands()
                    .any(|(operand, _)| operand == ordinal)
                    || normalized
                        .producer
                        .as_deref()
                        .is_some_and(|producer| producer.reads_declared_input(ordinal))
            }
        }
    }

    /// Returns the largest declared input element count this output reads.
    ///
    /// **Declared tensors' own counts, on the same basis
    /// [`Self::input_elements_at`] states**, so the two accessors cannot
    /// disagree about what a "declared input element count" is. A widening read
    /// used to make this report the reading region's domain, which is the
    /// iteration space rather than any tensor the ABI binds.
    ///
    /// **A read whose relation [`read_tensor_elements`] declines contributes the
    /// reading region's domain rather than refusing**, and the asymmetry with
    /// [`Self::input_elements_at`] is deliberate: this feeds structural cost
    /// estimates alone — [`NormalizedProgram::max_input_elements`] records the
    /// caller — and a maximum that refused would turn an estimate into a
    /// feasibility gate. It is unreachable for the three maps recognized today,
    /// and where a fourth reached it the domain would be an estimate rather than
    /// a bound: it happens to equal the operand count for a bijection and to
    /// exceed it for a replication, but a narrowing relation would sit the other
    /// side and nothing here would say so.
    pub(crate) fn max_input_elements(&self) -> u64 {
        match self {
            // Same two sources the reading arm folds, and for the same reason:
            // a prologue region's reads, or a prologue-less fold's own
            // contributor read of the declared tensor.
            Self::SerialSum(normalized) => normalized
                .prologue_reads
                .iter()
                .map(|(_, map)| {
                    read_tensor_elements(map, normalized.input_elements)
                        .unwrap_or(normalized.input_elements)
                })
                .chain(
                    normalized
                        .contributor_input
                        .map(|_| normalized.input_elements),
                )
                .max()
                .unwrap_or_default(),
            Self::Pointwise(normalized) => normalized
                .reads
                .iter()
                .map(|(_, map)| {
                    read_tensor_elements(map, normalized.elements).unwrap_or(normalized.elements)
                })
                .max()
                .unwrap_or_default(),
            Self::Contraction(normalized) => normalized
                .reads
                .iter()
                .map(|read| read.elements)
                .max()
                .unwrap_or_default(),
            // The epilogue's declared-input reads only: a chain whose epilogue
            // reads only the staged value reads no declared input there, and
            // reporting its domain would overstate what this output reads.
            Self::Epilogue(chain) => chain.producer.max_input_elements().max(
                chain
                    .reads
                    .iter()
                    .filter(|(read, _)| read.declared_ordinal().is_some())
                    .map(|(_, map)| {
                        read_tensor_elements(map, chain.elements).unwrap_or(chain.elements)
                    })
                    .max()
                    .unwrap_or_default(),
            ),
            // The declared-input operands only, and the producer's own answer
            // beside them: a staged operand's count is a materialization edge's
            // extent, and reporting it here would overstate the largest declared
            // input this output reads.
            Self::Staged(normalized) => normalized
                .declared_operands()
                .map(|(_, elements)| elements)
                .max()
                .unwrap_or_default()
                .max(
                    normalized
                        .producer
                        .as_deref()
                        .map_or(0, NormalizedOutput::max_input_elements),
                ),
        }
    }

    pub(crate) const fn output_elements(&self) -> u64 {
        match self {
            Self::SerialSum(normalized) => normalized.output_elements,
            Self::Pointwise(normalized) => normalized.elements,
            Self::Contraction(normalized) => normalized.output_elements,
            Self::Epilogue(chain) => chain.elements,
            Self::Staged(normalized) => normalized.output_elements,
        }
    }

    /// Returns every occurrence this output's walk claimed, in ascending order.
    pub(crate) fn members(&self) -> Vec<SemanticStage> {
        match self {
            Self::SerialSum(normalized) => normalized.members.all(),
            Self::Pointwise(normalized) => normalized.members.clone(),
            Self::Contraction(normalized) => normalized.members.clone(),
            Self::Epilogue(chain) => {
                let mut members = chain.producer.members();
                members.extend_from_slice(&chain.members);
                members.sort_unstable();
                members.dedup();
                members
            }
            // The occurrence, once, and its producer's own claim when an operand
            // is a materialized intermediate. A staged family's realization
            // stages are region formation's to enumerate — see
            // [`NormalizedStaged`] — and claiming them here would state the same
            // split twice and make [`check_output_cover`](super::recognize::check_output_cover)'s per-occurrence
            // accounting count a realization choice as program work. The
            // producer's occurrences are program work and are claimed by this
            // walk alone, exactly as a chain's producer's are.
            Self::Staged(normalized) => {
                let mut members = normalized
                    .producer
                    .as_deref()
                    .map_or_else(Vec::new, NormalizedOutput::members);
                members.push(SemanticStage::first(normalized.member));
                members.sort_unstable();
                members.dedup();
                members
            }
        }
    }

    /// Returns whether one region's exact member set is a part of this output's
    /// partition, so that a region spelled from it covers this output's work and
    /// no other's.
    ///
    /// The reduction's *whole* partition is a part in its own right: the fused
    /// spelling realizes the prologue and the fold in one region, which is the
    /// one case where a part is the union of two others.
    ///
    /// A prologue-less fold's partition has one part, and the prologue part is
    /// asked for through [`NormalizedSerialSum::prologue_members`] rather than by
    /// comparing against an empty set: a region covering no occurrence would
    /// otherwise resolve to a prologue this program does not have, and every
    /// derivation downstream would build one. Like the same distinction in
    /// [`crate::physical::spell_region`], it is defence in depth rather than a
    /// live gate — no cover this search places carries an empty member set.
    ///
    /// **Crate-visible because the region vocabulary asks it rather than
    /// restating it.** *Which* region spells a member set is
    /// [`crate::physical::spell_region`]'s question, decided against the region
    /// vocabulary; whether the member set is this output's at all is this one,
    /// decided against the recognized partition. A physical arm answering the
    /// second for itself would be a second account of the partition, free to
    /// disagree with the account [`NormalizedProgram::output_for_region`] and
    /// [`check_output_cover`](super::recognize::check_output_cover) read.
    pub(crate) fn owns_region_members(&self, members: &[SemanticStage]) -> bool {
        match self {
            Self::SerialSum(normalized) => {
                normalized
                    .prologue_members()
                    .is_some_and(|prologue| members == prologue)
                    || members == normalized.members.reduction()
                    || members == normalized.members.all()
            }
            Self::Pointwise(normalized) => members == normalized.members,
            Self::Contraction(normalized) => members == normalized.members,
            // The epilogue's own part, or any part of the producer's partition.
            // The chain as a whole is deliberately *not* a part: no scheduled
            // region computes a fold and an expression over its result, so a
            // cover grouping both has no spelling and must be declined rather
            // than resolved to this output.
            Self::Epilogue(chain) => {
                members == chain.members || chain.producer.owns_region_members(members)
            }
            // Every region whose atoms are all stages of this one occurrence —
            // which is [`NormalizedStaged::owns_stage_members`], and which states
            // why no stage list is enumerated here — or any part of the
            // producer's partition when an operand is a materialized
            // intermediate. The two are disjoint by construction: a producer's
            // atoms name a different occurrence, so no member set can be both.
            Self::Staged(normalized) => {
                normalized.owns_stage_members(members)
                    || normalized
                        .producer
                        .as_deref()
                        .is_some_and(|producer| producer.owns_region_members(members))
            }
        }
    }

    #[cfg(test)]
    fn serial_sum_mut(&mut self) -> &mut NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) | Self::Staged(_) => {
                panic!("the fixture is a serial sum")
            }
        }
    }
}

/// The recognized program: one implementable region partition per ordered named
/// output, in the program's own declaration order.
///
/// **A list rather than one whole-program strategy, and the difference is what
/// makes several ordered outputs statable at all.** Recognition used to read
/// `outputs().next()`, classify that one occurrence, and require the resulting
/// walk to cover the program exactly, so a second declared output was either
/// outside the walk — leaving the program uncovered — or inside it, where one
/// region's owning write would have had to serve two publications. Each output
/// now carries its own walk, and the *program*-wide obligation moved to the
/// relation between them: the walks partition the occurrences, so every
/// occurrence is claimed exactly once and every published value has one region
/// that owns its write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedProgram {
    pub(super) outputs: Vec<NormalizedOutput>,
}

impl NormalizedProgram {
    /// Returns the recognized outputs in the program's declaration order.
    pub(crate) fn outputs(&self) -> &[NormalizedOutput] {
        &self.outputs
    }

    /// Returns the recognized output whose partition contains one region's exact
    /// member set, with its declaration position.
    ///
    /// This is the lookup every per-region authority below the boundary asks:
    /// the members are the region's own coverage, and the partition they belong
    /// to is what says which shape, domain, and expression the region realizes.
    /// A member set belonging to no output's partition is `None` — a region
    /// covering occurrences from two outputs' walks, or covering part of one
    /// part, has no recognized implementation and is refused by name rather than
    /// spelled against whichever partition happened to be first.
    ///
    /// **Two outputs may own one member set, and declaration order decides.**
    /// [`check_output_cover`](super::recognize::check_output_cover) admits exactly one overlap — a walk that is one
    /// whole part of a longer walk's partition, publishing the value that part
    /// hands across the boundary — and both claimants are then recognitions of
    /// the same value over the same occurrences, so they resolve to the same
    /// region. [`crate::physical::spell_region`] carries the derivation and
    /// `both_claimants_of_a_published_and_consumed_part_spell_one_region` is the
    /// check that says no if that ever stops holding.
    pub(crate) fn output_for_region(
        &self,
        members: &[SemanticStage],
    ) -> Option<(usize, &NormalizedOutput)> {
        self.outputs
            .iter()
            .enumerate()
            .find(|(_, output)| output.owns_region_members(members))
    }

    /// Returns the recognized output at one declared position.
    pub(crate) fn output_at(&self, position: usize) -> Option<&NormalizedOutput> {
        self.outputs.get(position)
    }

    /// Returns the element count of one declared input tensor, when every
    /// recognized output that *reads* it agrees on the count.
    ///
    /// **Agreement or nothing, because the caller is sizing work.** The count
    /// scales a call over the tensor bound to that ordinal, and answering from
    /// whichever output claimed first would let one claimant's number stand for
    /// a tensor another claimant sizes differently — the confidently-wrong
    /// verdict a work-scaling resolution exists to prevent. A disagreement
    /// therefore yields `None` and the caller refuses, exactly as it does for an
    /// ordinal no input occupies.
    ///
    /// **Defence in depth rather than a live gate, and the distinction is worth
    /// stating.** Two outputs used to read one declared input at different
    /// domains — a reduction at its contributor shape, an elementwise sibling at
    /// its own — and this fold is what refused the pair. Since
    /// [`NormalizedOutput::input_elements_at`] answers the declared tensor's own
    /// count on every arm, that count is a function of the ordinal alone and no
    /// recognizable program reaches the refusal. The fold stays because the
    /// property it enforces is one an added arm or access relation could break,
    /// and a wrong work count is not a failure a later stage catches.
    /// `every_arm_answers_the_declared_tensors_own_count` records the reasoning
    /// and `a_bound_access_resolves_through_the_checked_request_subject` records the
    /// perturbation that makes this fold refuse again.
    ///
    /// **The fold ranges over the reading outputs, and it has to.** An output
    /// that never loads the ordinal has nothing to say about it, but [`agreed`]
    /// compares `Option`s, so a silent output's `None` is a *value* that
    /// disagrees with every count rather than an abstention — and a program
    /// whose two outputs iterate disjoint inputs would refuse every ordinal,
    /// each of which exactly one output sizes. Filtering them out before the
    /// fold is what makes silence cost nothing.
    ///
    /// The filter asks [`NormalizedOutput::reads_declared_input`] rather than
    /// whether the output produced a count, and the difference is load-bearing:
    /// an output that *does* read the ordinal and still cannot name one domain
    /// for it — an epilogue chain whose producer and epilogue read it at two
    /// domains — answers `None`, and that is a genuine disagreement the fold
    /// must keep. Filtering on the answer would drop exactly that refusal and
    /// let a sibling's count stand for a chain that has no single one.
    ///
    /// The two `None`s are flattened deliberately: "the reading outputs
    /// disagree" and "no output reads that ordinal" are different findings, and
    /// this accessor's caller acts identically on both — it refuses.
    pub(crate) fn agreed_input_elements_at(&self, ordinal: DeclaredInputOrdinal) -> Option<u64> {
        agreed(
            self.outputs
                .iter()
                .filter(|output| output.reads_declared_input(ordinal))
                .map(|output| output.input_elements_at(ordinal)),
        )
        .flatten()
    }

    /// Returns the largest declared input element count over every output.
    ///
    /// A structural proxy for the widest thing a plan for this request could
    /// stage, which is what `GovernedPhysicalProvider::propose`'s cost estimate
    /// wants. Deliberately a maximum rather than an agreement: a cost may be an
    /// upper bound over the whole request, and a cost that refused would turn an
    /// estimate into a feasibility gate. That one caller is the whole reason
    /// [`NormalizedOutput::max_input_elements`] substitutes a domain for a
    /// relation it cannot size instead of declining.
    pub(crate) fn max_input_elements(&self) -> u64 {
        self.outputs
            .iter()
            .map(NormalizedOutput::max_input_elements)
            .max()
            .unwrap_or_default()
    }

    /// Returns the largest published element count over every output.
    ///
    /// A maximum for the reason [`Self::max_input_elements`] is one: its callers
    /// are structural cost estimates, never feasibility.
    pub(crate) fn max_output_elements(&self) -> u64 {
        self.outputs
            .iter()
            .map(NormalizedOutput::output_elements)
            .max()
            .unwrap_or_default()
    }

    /// Returns every attribution atom any output's walk claimed, ascending.
    ///
    /// The walks partition the program's occurrences — [`check_output_cover`](super::recognize::check_output_cover)
    /// proves it — so this is the program's whole operation set and the
    /// deduplication is the invariant being relied on rather than a repair.
    pub(crate) fn all_members(&self) -> Vec<SemanticStage> {
        let mut members: Vec<SemanticStage> = self
            .outputs
            .iter()
            .flat_map(NormalizedOutput::members)
            .collect();
        members.sort_unstable();
        members.dedup();
        members
    }

    /// Returns the first authored symbol any recognized pointwise output still names.
    ///
    /// Same-shape elementwise is the population this boundary admits with
    /// extents left symbolic. Later schedule construction that needs a fixed
    /// launch geometry reads this rather than folding
    /// [`ExtentSources::determined`].
    pub(crate) fn first_symbolic_extent(&self) -> Option<SourcedExtent> {
        self.outputs.iter().find_map(|output| match output {
            NormalizedOutput::Pointwise(pointwise) => pointwise
                .shape
                .extents()
                .find(|extent| extent.as_static().is_none()),
            _ => None,
        })
    }

    /// Returns whether any recognized read is the labelled-draft parametric
    /// broadcast carrier.
    ///
    /// The compile path uses this to take the carrier to physical selection
    /// instead of refusing it under the generic symbolic-extent schedule gate.
    /// A provider that cannot implement the carrier then declines by name.
    pub(crate) fn carries_parametric_broadcast(&self) -> bool {
        self.outputs
            .iter()
            .any(NormalizedOutput::carries_parametric_broadcast)
    }

    /// Returns every *occurrence* any output's walk claimed, ascending.
    ///
    /// The projection of [`Self::all_members`] onto operations, for the
    /// authorities whose subject is the occurrence rather than a stage of it:
    /// an occurrence resolves one lowering capability and carries one refinement
    /// receipt however many regions realize it, so asking per atom would resolve
    /// one capability twice and mint two receipts for one proof obligation.
    pub(crate) fn all_occurrences(&self) -> Vec<SemanticMemberId> {
        let mut members: Vec<SemanticMemberId> = self
            .all_members()
            .into_iter()
            .map(SemanticStage::member)
            .collect();
        members.dedup();
        members
    }

    #[cfg(test)]
    pub(super) fn serial_sum_mut(&mut self) -> &mut NormalizedSerialSum {
        let [output] = self.outputs.as_mut_slice() else {
            panic!("the fixture declares one output");
        };
        output.serial_sum_mut()
    }
}

/// Returns the element count of the tensor one recognized elementwise read
/// addresses, given the domain of the region performing it.
///
/// **The tensor's own count, which is what separates it from the domain.** An
/// elementwise region's read list carries a relation per read, and a widening
/// one addresses fewer elements than the region iterates: `broadcast(w)` over a
/// `[2]` weight into a `[2, 2]` domain reads two elements four times. Callers
/// sizing a call over the buffer the ABI binds want the two, so the count is
/// taken from the relation rather than from the region.
///
/// A dense read answers the domain because the recognizer made them equal:
/// [`plan_elementwise`] refuses an elementwise leaf whose shape is not the
/// region's, so the domain *is* that tensor's count rather than standing in for
/// it. A structural read answers its operand shape, and
/// [`recognize_structural_read`] admits a structural occurrence only over a
/// value this walk reads — a declared input or the staged value — so that
/// operand shape is the declared tensor's own.
///
/// **The wildcard declines rather than guesses.** [`LogicalAccess`] is
/// `#[non_exhaustive]`, so a relation added upstream reaches it; naming a count
/// for a relation whose operand extent is unknown is exactly the confidently
/// wrong work count a refusal exists to prevent. The named arms are the ones
/// that would be reached first and are listed so a reader can see which
/// relations are being declined on purpose.
fn read_tensor_elements(map: &LogicalAccess, domain_elements: u64) -> Option<u64> {
    match map {
        LogicalAccess::LinearIdentity => Some(domain_elements),
        // The overflow refusal is unreachable through a recognized program:
        // `recognize_structural_read` took this shape from the declared value,
        // and `element_count_u64` already multiplied the same extents when the
        // shape's own arm minted its count. It declines rather than saturating
        // because a saturated count is a work count nothing derived.
        LogicalAccess::ReindexBijection { operand_shape, .. }
        | LogicalAccess::BroadcastReplication { operand_shape, .. } => {
            tiler_ir::schedule::element_count(operand_shape).ok()
        }
        // A sourced operand answers only when every extent is already a
        // literal. Folding `ExtentSources::determined` here would size the
        // read from a bound value the request must not specialize.
        LogicalAccess::ParametricBroadcast { operand_shape, .. } => operand_shape
            .as_static()
            .and_then(|shape| tiler_ir::schedule::element_count(shape).ok()),
        LogicalAccess::ScalarBroadcast
        | LogicalAccess::PackedU4LsbZeroTail { .. }
        | LogicalAccess::ReductionContributor { .. }
        | LogicalAccess::ContractionOperand { .. }
        | _ => None,
    }
}

/// Returns whether `map` is the labelled-draft parametric broadcast carrier.
const fn access_is_parametric_broadcast(map: &LogicalAccess) -> bool {
    matches!(map, LogicalAccess::ParametricBroadcast { .. })
}

/// Returns the one value every entry carries, or `None` when they disagree.
///
/// An empty sequence answers `None` rather than a vacuous value: a program with
/// no recognized output has nothing to report, and reporting a default would be
/// an answer nothing derived.
fn agreed<T: Eq>(values: impl IntoIterator<Item = T>) -> Option<T> {
    let mut values = values.into_iter();
    let first = values.next()?;
    values.all(|value| value == first).then_some(first)
}
