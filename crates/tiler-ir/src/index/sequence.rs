//! Ordered multi-region canonical realizations.
//!
//! A single [`VerifiedIndexRegion`] proves one region structurally safe, but a
//! family whose canonical realization is a reduction feeding an elementwise pass
//! over the reduction's materialized result is *two* regions with a value handed
//! between them. This module owns the ordered chain, the explicit contract for
//! each handed value, and the canonical identity the refinement verifier compares
//! sequences by.
//!
//! **The chain is derived and checked, never declared and believed.** A caller
//! supplies the ordered regions and, for each region input boundary, the source
//! the value comes from; [`VerifiedIndexRegionSequence::try_new`] then proves the
//! chain well formed. That mirrors what
//! `tiler_compiler::frontier::derive_subprogram_boundary_contract` does one layer
//! down for a scheduled subprogram: a non-final stage's owning write is the next
//! stage's input, a value handed on and never read is refused, and only the final
//! stage's writes leave the sequence. The two layers are deliberately separate
//! IRs — that one operates on `VerifiedScheduledRegion` and this one on
//! `VerifiedIndexRegion` — so this is a model mirrored, not a mechanism reused.
//!
//! The public surface here is a concrete draft pending Tom's review; see
//! `tickets/accept-the-multi-region-index-realization-surface.md`.

use core::fmt;
use std::error::Error;

use crate::identity::{push_len, push_slice};
use crate::semantic::ResolvedValueType;
use crate::shape::Shape;

use super::{TensorRole, VerifiedIndexRegion, VerifiedTensorId};

const REGION_SEQUENCE_IDENTITY_TAG: &[u8] = b"tiler.ir.index-region-sequence.v1\0";

/// Maximum ordered stages admitted by one canonical realization.
///
/// The closed law vocabulary emits at most two stages. The ceiling is stated
/// independently and generously so that a hand-built sequence exceeding what any
/// law can produce still refuses by a named bound rather than by exhausting
/// memory in the identity encoder.
pub const MAX_INDEX_REGION_SEQUENCE_STAGES: usize = 64;

/// Where one region input boundary's value comes from.
///
/// Declared per input boundary rather than inferred, because inference has no
/// answer when two boundaries agree on element type and shape — exactly the case
/// a normalization presents, whose value and weight operands are both the
/// normalized shape at the same width.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StagedInputSource {
    /// The occurrence's expanded input boundary at this ordered position.
    ///
    /// The position indexes the *component-expanded* semantic inputs, the same
    /// list operand binding walks, so an encoded compound operand names one
    /// position per component in its contract order.
    Occurrence(usize),
    /// The value the named earlier stage handed on.
    Intermediate(usize),
}

/// One value produced by a stage and consumed by the stage that follows it.
///
/// Every field is a checked fact about the two regions, not a caller assertion:
/// [`VerifiedIndexRegionSequence::try_new`] reads the producing output boundary
/// and the consuming input boundary out of the regions themselves and refuses
/// when they disagree.
///
/// **Ownership.** An intermediate belongs to the sequence. It is neither an
/// occurrence input nor an occurrence result: no [`StagedInputSource::Occurrence`]
/// names it, and it is produced by a non-final stage, whose writes therefore
/// never leave the sequence.
///
/// **Lifetime.** It is created by [`Self::producer`] and dead after
/// [`Self::consumer`], which is required to be the immediately following stage
/// and the only stage that reads it. A value handed further down the chain would
/// have to stay live across a stage that does not mention it, which the
/// sequence deliberately cannot express rather than leaving the retention
/// implied by stage order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedIntermediate {
    producer: usize,
    producer_output: VerifiedTensorId,
    consumer: usize,
    consumer_input: VerifiedTensorId,
    value_type: ResolvedValueType,
    shape: Shape,
}

impl StagedIntermediate {
    /// Returns the ordered stage that produces this value.
    #[must_use]
    pub const fn producer(&self) -> usize {
        self.producer
    }
    /// Returns the producing stage's output tensor boundary.
    #[must_use]
    pub const fn producer_output(&self) -> VerifiedTensorId {
        self.producer_output
    }
    /// Returns the ordered stage that consumes this value.
    #[must_use]
    pub const fn consumer(&self) -> usize {
        self.consumer
    }
    /// Returns the consuming stage's input tensor boundary.
    #[must_use]
    pub const fn consumer_input(&self) -> VerifiedTensorId {
        self.consumer_input
    }
    /// Returns the element type both boundaries agree on.
    #[must_use]
    pub const fn value_type(&self) -> &ResolvedValueType {
        &self.value_type
    }
    /// Returns the static shape both boundaries agree on.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// Canonical identity of one ordered multi-region realization.
///
/// **A one-stage sequence's identity is its region's identity, byte for byte.**
/// That is deliberate rather than incidental: it is what keeps every receipt
/// minted for a single-region law unchanged by this vocabulary's arrival. A
/// sequence of two or more stages is written under its own domain tag and can
/// therefore never collide with a region identity, which carries a different one
/// in the same leading position.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalIndexRegionSequenceIdentity(Box<[u8]>);

impl CanonicalIndexRegionSequenceIdentity {
    /// Returns the canonical sequence identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An ordered chain of verified regions realizing one semantic occurrence.
///
/// The final stage is stored beside the earlier ones rather than at the end of
/// one list, so "a realization has at least one region" is a property of the
/// type rather than an invariant every reader must re-establish.
#[derive(Clone, Debug)]
pub struct VerifiedIndexRegionSequence {
    leading: Vec<VerifiedIndexRegion>,
    last: VerifiedIndexRegion,
    sources: Vec<Vec<StagedInputSource>>,
    intermediates: Vec<StagedIntermediate>,
    identity: CanonicalIndexRegionSequenceIdentity,
}

impl PartialEq for VerifiedIndexRegionSequence {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for VerifiedIndexRegionSequence {}

impl VerifiedIndexRegionSequence {
    /// Builds the one-stage sequence realizing an occurrence in a single region.
    ///
    /// Its input boundaries are sourced positionally from the occurrence, which
    /// is the binding rule single-region refinement has always used.
    #[must_use]
    pub fn single(region: VerifiedIndexRegion) -> Self {
        let sources = vec![
            region
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Input)
                .enumerate()
                .map(|(position, _)| StagedInputSource::Occurrence(position))
                .collect(),
        ];
        let identity = CanonicalIndexRegionSequenceIdentity(
            region.canonical_identity().as_bytes().to_vec().into(),
        );
        Self {
            leading: Vec::new(),
            last: region,
            sources,
            intermediates: Vec::new(),
            identity,
        }
    }

    /// Builds and proves an ordered multi-stage chain.
    ///
    /// `sources` carries one entry per stage, and within a stage one entry per
    /// input tensor boundary in the region's own boundary order.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRegionSequenceError`] when the stage population is outside
    /// its bound, a source list does not match its stage's input boundaries, a
    /// handed value is never read or is read by a stage other than the one that
    /// immediately follows its producer, a non-final stage does not produce
    /// exactly the one value it hands on, or a producing and consuming boundary
    /// disagree on element type or shape.
    pub fn try_new(
        stages: Vec<VerifiedIndexRegion>,
        sources: Vec<Vec<StagedInputSource>>,
    ) -> Result<Self, IndexRegionSequenceError> {
        if stages.is_empty() {
            return Err(IndexRegionSequenceError::Empty);
        }
        if stages.len() > MAX_INDEX_REGION_SEQUENCE_STAGES {
            return Err(IndexRegionSequenceError::TooManyStages {
                actual: stages.len(),
                limit: MAX_INDEX_REGION_SEQUENCE_STAGES,
            });
        }
        if sources.len() != stages.len() {
            return Err(IndexRegionSequenceError::SourceArity {
                stage: stages.len().min(sources.len()),
            });
        }

        let inputs_of = |stage: &VerifiedIndexRegion| {
            stage
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Input)
                .map(|tensor| {
                    (
                        tensor.id(),
                        tensor.value_type().clone(),
                        tensor.shape().as_static().cloned(),
                    )
                })
                .collect::<Vec<_>>()
        };

        let mut intermediates = Vec::with_capacity(stages.len().saturating_sub(1));
        // The value the previous stage handed on, and the stage that produced
        // it. `None` at the head of the chain, and cleared by the stage that
        // consumes it so a second consumer finds nothing to bind.
        let mut handoff: Option<(usize, VerifiedTensorId, ResolvedValueType, Shape)> = None;

        for (position, stage) in stages.iter().enumerate() {
            let inputs = inputs_of(stage);
            let stage_sources = &sources[position];
            if stage_sources.len() != inputs.len() {
                return Err(IndexRegionSequenceError::SourceArity { stage: position });
            }
            let mut owed = handoff.take();
            for (slot, source) in stage_sources.iter().enumerate() {
                let (tensor, value_type, shape) = &inputs[slot];
                match source {
                    StagedInputSource::Occurrence(_) => {}
                    StagedInputSource::Intermediate(producer) => {
                        let Some((owed_producer, owed_output, owed_type, owed_shape)) = &owed
                        else {
                            // Either nothing was handed on, or this stage already
                            // consumed it. Both are the same defect: an
                            // intermediate has exactly one reader.
                            return Err(IndexRegionSequenceError::UnavailableIntermediate {
                                stage: position,
                                slot,
                            });
                        };
                        if producer != owed_producer {
                            return Err(IndexRegionSequenceError::UnavailableIntermediate {
                                stage: position,
                                slot,
                            });
                        }
                        let Some(shape) = shape else {
                            return Err(IndexRegionSequenceError::SymbolicIntermediate {
                                stage: position,
                                slot,
                            });
                        };
                        if value_type != owed_type || shape != owed_shape {
                            return Err(IndexRegionSequenceError::IntermediateInterface {
                                stage: position,
                                slot,
                            });
                        }
                        intermediates.push(StagedIntermediate {
                            producer: *owed_producer,
                            producer_output: *owed_output,
                            consumer: position,
                            consumer_input: *tensor,
                            value_type: value_type.clone(),
                            shape: shape.clone(),
                        });
                        owed = None;
                    }
                }
            }
            if owed.is_some() {
                // A value published by the previous stage that this one never
                // reads is staged with no consumer: a leak whose owner nothing
                // downstream could name.
                return Err(IndexRegionSequenceError::IntermediateNeverRead {
                    producer: position.saturating_sub(1),
                });
            }

            if position + 1 == stages.len() {
                continue;
            }
            let mut outputs = stage
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Output);
            let Some(output) = outputs.next() else {
                return Err(IndexRegionSequenceError::NotChained { stage: position });
            };
            if outputs.next().is_some() {
                // Only the final stage's writes leave the sequence, and one
                // handed value has one reader, so a non-final stage publishing
                // two values has nothing coherent to hand on.
                return Err(IndexRegionSequenceError::NotChained { stage: position });
            }
            let Some(shape) = output.shape().as_static().cloned() else {
                return Err(IndexRegionSequenceError::SymbolicIntermediate {
                    stage: position,
                    slot: 0,
                });
            };
            handoff = Some((position, output.id(), output.value_type().clone(), shape));
        }

        let identity = encode_sequence_identity(&stages, &sources);
        let mut leading = stages;
        let last = leading.pop().ok_or(IndexRegionSequenceError::Empty)?;
        Ok(Self {
            leading,
            last,
            sources,
            intermediates,
            identity,
        })
    }

    /// Returns the ordered stages.
    pub fn stages(&self) -> impl Iterator<Item = &VerifiedIndexRegion> {
        self.leading.iter().chain(std::iter::once(&self.last))
    }

    /// Returns every stage before the final one, in order.
    #[must_use]
    pub fn leading_stages(&self) -> &[VerifiedIndexRegion] {
        &self.leading
    }

    /// Returns the number of ordered stages, which is never zero.
    #[must_use]
    pub fn stage_count(&self) -> usize {
        self.leading.len().saturating_add(1)
    }

    /// Returns whether this realization has exactly one stage.
    #[must_use]
    pub fn is_single_stage(&self) -> bool {
        self.leading.is_empty()
    }

    /// Returns one ordered stage, when the ordinal names one.
    #[must_use]
    pub fn stage(&self, stage: usize) -> Option<&VerifiedIndexRegion> {
        match self.leading.get(stage) {
            Some(region) => Some(region),
            None if stage == self.leading.len() => Some(&self.last),
            None => None,
        }
    }

    /// Returns the ordered input sources of one stage, when the stage exists.
    #[must_use]
    pub fn stage_sources(&self, stage: usize) -> Option<&[StagedInputSource]> {
        self.sources.get(stage).map(Vec::as_slice)
    }

    /// Returns every checked handed value in producing order.
    #[must_use]
    pub fn intermediates(&self) -> &[StagedIntermediate] {
        &self.intermediates
    }

    /// Returns the stage whose writes leave the sequence.
    #[must_use]
    pub const fn final_stage(&self) -> &VerifiedIndexRegion {
        &self.last
    }

    /// Returns the exact canonical sequence identity.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalIndexRegionSequenceIdentity {
        &self.identity
    }
}

/// Encodes the canonical identity of an ordered chain.
///
/// **Injectivity.** A one-stage chain is written as its region's identity
/// verbatim, and every longer chain is written under
/// [`REGION_SEQUENCE_IDENTITY_TAG`]. A region identity carries its own distinct
/// domain tag as its first bytes, so the two preimages are disjoint and no
/// multi-stage chain can spell a region. Within the tagged form, the stage count,
/// every region identity, and every source list are length-prefixed and written
/// in order, so a truncated, extended, or reordered chain — and a chain whose
/// stages are identical but whose inputs are sourced differently — each render
/// distinct bytes.
fn encode_sequence_identity(
    stages: &[VerifiedIndexRegion],
    sources: &[Vec<StagedInputSource>],
) -> CanonicalIndexRegionSequenceIdentity {
    if let [only] = stages {
        return CanonicalIndexRegionSequenceIdentity(
            only.canonical_identity().as_bytes().to_vec().into(),
        );
    }
    let mut bytes = REGION_SEQUENCE_IDENTITY_TAG.to_vec();
    push_len(&mut bytes, stages.len());
    for stage in stages {
        push_slice(&mut bytes, stage.canonical_identity().as_bytes());
    }
    push_len(&mut bytes, sources.len());
    for stage_sources in sources {
        push_len(&mut bytes, stage_sources.len());
        for source in stage_sources {
            match source {
                StagedInputSource::Occurrence(position) => {
                    bytes.push(1);
                    push_len(&mut bytes, *position);
                }
                StagedInputSource::Intermediate(producer) => {
                    bytes.push(2);
                    push_len(&mut bytes, *producer);
                }
            }
        }
    }
    CanonicalIndexRegionSequenceIdentity(bytes.into_boxed_slice())
}

/// Why an ordered multi-region chain was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexRegionSequenceError {
    /// A realization retains no stage at all.
    Empty,
    /// The stage population exceeds the governed ceiling.
    TooManyStages {
        /// Supplied stage count.
        actual: usize,
        /// Maximum admitted stage count.
        limit: usize,
    },
    /// A stage's declared input sources do not match its input boundaries.
    SourceArity {
        /// Ordered stage whose source list disagreed.
        stage: usize,
    },
    /// An input claims a handed value that is not available to it.
    ///
    /// Either nothing was handed on, the named producer is not the immediately
    /// preceding stage, or this stage already consumed the one value it was
    /// handed.
    UnavailableIntermediate {
        /// Ordered consuming stage.
        stage: usize,
        /// Ordered input boundary within that stage.
        slot: usize,
    },
    /// A producing and consuming boundary disagree on element type or shape.
    IntermediateInterface {
        /// Ordered consuming stage.
        stage: usize,
        /// Ordered input boundary within that stage.
        slot: usize,
    },
    /// A handed boundary exposes no static shape in this bounded profile.
    SymbolicIntermediate {
        /// Ordered stage carrying the boundary.
        stage: usize,
        /// Ordered boundary position within that stage.
        slot: usize,
    },
    /// A stage handed a value on that the following stage never read.
    IntermediateNeverRead {
        /// Ordered stage that published the unread value.
        producer: usize,
    },
    /// A non-final stage does not publish exactly the one value it hands on.
    NotChained {
        /// Ordered stage that cannot hand a value on.
        stage: usize,
    },
}

impl fmt::Display for IndexRegionSequenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a realization retains no stage"),
            Self::TooManyStages { actual, limit } => {
                write!(formatter, "{actual} stages exceed the limit of {limit}")
            }
            Self::SourceArity { stage } => {
                write!(
                    formatter,
                    "stage {stage} declares a source list that does not match its input boundaries"
                )
            }
            Self::UnavailableIntermediate { stage, slot } => {
                write!(
                    formatter,
                    "stage {stage} input {slot} claims an unavailable handed value"
                )
            }
            Self::IntermediateInterface { stage, slot } => {
                write!(
                    formatter,
                    "stage {stage} input {slot} disagrees with the handed boundary"
                )
            }
            Self::SymbolicIntermediate { stage, slot } => {
                write!(
                    formatter,
                    "stage {stage} boundary {slot} exposes no static shape"
                )
            }
            Self::IntermediateNeverRead { producer } => {
                write!(formatter, "stage {producer} handed a value nothing reads")
            }
            Self::NotChained { stage } => {
                write!(
                    formatter,
                    "stage {stage} does not publish exactly one value to hand on"
                )
            }
        }
    }
}

impl Error for IndexRegionSequenceError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{
        DomainRole, FrozenScalarRegistry, IndexRegionBuilder, ScalarAttributes,
        multiply_f32_scalar_op,
    };
    use crate::semantic::F32;
    use crate::shape::{Extent, Shape};

    fn scalars() -> FrozenScalarRegistry {
        FrozenScalarRegistry::standard().expect("the standard scalar registry is coherent")
    }

    /// Emits `out[i] = mul(in[0][i], in[last][i])` over `[extent]`.
    ///
    /// `inputs` names each input boundary's extent, and the product reads the
    /// first and the last of them, so a one-input region squares and a two-input
    /// region multiplies its two operands.
    fn product_region(inputs: &[u64], extent: u64, outputs: usize) -> VerifiedIndexRegion {
        let mut builder =
            IndexRegionBuilder::new(scalars()).expect("the standard registry admits a builder");
        let point = builder
            .dimension(DomainRole::Parallel, Extent::new(extent))
            .expect("one parallel dimension");
        let coordinate = builder
            .dimension_expr(point)
            .expect("its induction variable");
        let tensors = inputs
            .iter()
            .map(|input| {
                builder
                    .tensor(
                        crate::index::TensorRole::Input,
                        F32::resolved_type(),
                        Shape::from_dims([*input]),
                    )
                    .expect("an input boundary")
            })
            .collect::<Vec<_>>();
        let left = builder
            .read(tensors[0], &[point], &[coordinate])
            .expect("the first operand read");
        let right = builder
            .read(
                *tensors.last().expect("at least one input"),
                &[point],
                &[coordinate],
            )
            .expect("the last operand read");
        let product = builder
            .apply(
                multiply_f32_scalar_op(),
                ScalarAttributes::empty(),
                &[left, right],
            )
            .expect("the governed multiply applies")
            .get(0)
            .expect("multiply yields one result");
        for _ in 0..outputs {
            let output = builder
                .tensor(
                    crate::index::TensorRole::Output,
                    F32::resolved_type(),
                    Shape::from_dims([extent]),
                )
                .expect("an output boundary");
            let write = builder
                .write(output, &[point], &[coordinate])
                .expect("a complete write");
            builder.output(write, product).expect("an output root");
        }
        builder.build().expect("the region verifies")
    }

    fn square() -> VerifiedIndexRegion {
        product_region(&[4], 4, 1)
    }

    fn consumer() -> VerifiedIndexRegion {
        product_region(&[4, 4], 4, 1)
    }

    fn chained_sources() -> Vec<Vec<StagedInputSource>> {
        vec![
            vec![StagedInputSource::Occurrence(0)],
            vec![
                StagedInputSource::Occurrence(1),
                StagedInputSource::Intermediate(0),
            ],
        ]
    }

    #[test]
    fn a_one_stage_sequence_is_its_region_byte_for_byte() {
        let region = square();
        let expected = region.canonical_identity().as_bytes().to_vec();
        let sequence = VerifiedIndexRegionSequence::single(region);
        assert_eq!(sequence.identity().as_bytes(), expected);
        assert!(sequence.is_single_stage());
        assert_eq!(sequence.stage_count(), 1);
        assert!(sequence.intermediates().is_empty());
    }

    #[test]
    fn a_chained_pair_records_its_handed_value() {
        let sequence =
            VerifiedIndexRegionSequence::try_new(vec![square(), consumer()], chained_sources())
                .expect("the fold's output is the pass's second input");
        assert_eq!(sequence.stage_count(), 2);
        assert!(!sequence.is_single_stage());
        let [intermediate] = sequence.intermediates() else {
            panic!("a two-stage chain hands exactly one value on")
        };
        assert_eq!(intermediate.producer(), 0);
        assert_eq!(intermediate.consumer(), 1);
        assert_eq!(intermediate.shape(), &Shape::from_dims([4]));
        assert_eq!(intermediate.value_type(), &F32::resolved_type());
        // The handed value is the producer's own output boundary and the
        // consumer's own input boundary, not a re-declaration of either.
        assert_eq!(
            intermediate.producer_output(),
            sequence
                .stage(0)
                .expect("stage zero")
                .tensors()
                .find(|tensor| tensor.role() == TensorRole::Output)
                .expect("the fold publishes one value")
                .id()
        );
    }

    /// A handed value nothing reads, and a read of a value nothing handed.
    #[test]
    fn an_unread_or_unavailable_handed_value_refuses() {
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(
                vec![square(), consumer()],
                vec![
                    vec![StagedInputSource::Occurrence(0)],
                    vec![
                        StagedInputSource::Occurrence(1),
                        StagedInputSource::Occurrence(2),
                    ],
                ],
            ),
            Err(IndexRegionSequenceError::IntermediateNeverRead { producer: 0 })
        );
        // Naming a producer that is not the immediately preceding stage keeps
        // the value alive across a stage that never mentions it.
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(
                vec![square(), consumer()],
                vec![
                    vec![StagedInputSource::Occurrence(0)],
                    vec![
                        StagedInputSource::Occurrence(1),
                        StagedInputSource::Intermediate(1),
                    ],
                ],
            ),
            Err(IndexRegionSequenceError::UnavailableIntermediate { stage: 1, slot: 1 })
        );
        // One handed value has one reader: a second claim finds nothing.
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(
                vec![square(), consumer()],
                vec![
                    vec![StagedInputSource::Occurrence(0)],
                    vec![
                        StagedInputSource::Intermediate(0),
                        StagedInputSource::Intermediate(0),
                    ],
                ],
            ),
            Err(IndexRegionSequenceError::UnavailableIntermediate { stage: 1, slot: 1 })
        );
    }

    #[test]
    fn a_handed_boundary_disagreeing_on_shape_refuses() {
        // The fold publishes `[2]`; the pass's handed slot is `[4]`. Both
        // regions verify on their own, which is the point: what disagrees is the
        // composition, and only the chain check looks at it.
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(
                vec![product_region(&[2], 2, 1), consumer()],
                chained_sources(),
            ),
            Err(IndexRegionSequenceError::IntermediateInterface { stage: 1, slot: 1 })
        );
    }

    #[test]
    fn a_non_final_stage_publishing_two_values_refuses() {
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(
                vec![product_region(&[4], 4, 2), consumer()],
                chained_sources(),
            ),
            Err(IndexRegionSequenceError::NotChained { stage: 0 })
        );
    }

    #[test]
    fn a_source_list_that_does_not_match_its_stage_refuses() {
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(vec![square()], Vec::new()),
            Err(IndexRegionSequenceError::SourceArity { stage: 0 })
        );
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(
                vec![square(), consumer()],
                vec![
                    vec![StagedInputSource::Occurrence(0)],
                    vec![StagedInputSource::Intermediate(0)],
                ],
            ),
            Err(IndexRegionSequenceError::SourceArity { stage: 1 })
        );
    }

    #[test]
    fn an_empty_or_over_wide_realization_refuses() {
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(Vec::new(), Vec::new()),
            Err(IndexRegionSequenceError::Empty)
        );
        let region = square();
        let stages = MAX_INDEX_REGION_SEQUENCE_STAGES + 1;
        assert_eq!(
            VerifiedIndexRegionSequence::try_new(vec![region; stages], vec![Vec::new(); stages],),
            Err(IndexRegionSequenceError::TooManyStages {
                actual: stages,
                limit: MAX_INDEX_REGION_SEQUENCE_STAGES,
            })
        );
    }

    /// The chain is ordered, and the order is part of what the identity says.
    ///
    /// **A reversed chain is not malformed, and this is where that is recorded.**
    /// Running the pass first and the fold second composes perfectly well —
    /// every shape lines up and every handed value is read — so nothing
    /// structural refuses it. What it is not is *this operation's* realization,
    /// and the authority for that is
    /// [`ResolvedIndexRealization::verify_sequence`](crate::index::ResolvedIndexRealization::verify_sequence),
    /// which compares the whole chain's identity against the law's own. This
    /// test's job is to prove the identity separates the two orders, because a
    /// chain identity that did not would make that comparison blind to order.
    #[test]
    fn reversing_or_rewiring_a_chain_changes_its_identity() {
        let forward =
            VerifiedIndexRegionSequence::try_new(vec![square(), consumer()], chained_sources())
                .expect("the fold-then-pass order chains");
        let reversed = VerifiedIndexRegionSequence::try_new(
            vec![consumer(), square()],
            vec![
                vec![
                    StagedInputSource::Occurrence(0),
                    StagedInputSource::Occurrence(1),
                ],
                vec![StagedInputSource::Intermediate(0)],
            ],
        )
        .expect("the reversed order is structurally well formed, and that is the finding");
        assert_ne!(forward.identity(), reversed.identity());

        // Two chains over identical regions, differing only in which occurrence
        // input the pass's first slot reads. Nothing about either region says
        // which, so the sources are the only thing separating them — and the
        // identity has to carry that or the two realizations are one.
        let wired =
            VerifiedIndexRegionSequence::try_new(vec![square(), consumer()], chained_sources())
                .expect("the ordinary wiring chains");
        let rewired = VerifiedIndexRegionSequence::try_new(
            vec![square(), consumer()],
            vec![
                vec![StagedInputSource::Occurrence(0)],
                vec![
                    StagedInputSource::Occurrence(0),
                    StagedInputSource::Intermediate(0),
                ],
            ],
        )
        .expect("reading the same occurrence input twice also chains");
        assert_eq!(
            wired.stage(1).map(VerifiedIndexRegion::canonical_identity),
            rewired
                .stage(1)
                .map(VerifiedIndexRegion::canonical_identity),
            "the rewiring must leave both regions identical, or this proves nothing"
        );
        assert_ne!(wired.identity(), rewired.identity());
    }

    #[test]
    fn a_chain_is_never_confusable_with_one_of_its_regions() {
        let chained =
            VerifiedIndexRegionSequence::try_new(vec![square(), consumer()], chained_sources())
                .expect("the pair chains");
        for stage in chained.stages() {
            assert_ne!(
                chained.identity().as_bytes(),
                stage.canonical_identity().as_bytes()
            );
            assert_ne!(
                chained.identity().as_bytes(),
                VerifiedIndexRegionSequence::single(stage.clone())
                    .identity()
                    .as_bytes()
            );
        }
        assert!(
            chained
                .identity()
                .as_bytes()
                .starts_with(REGION_SEQUENCE_IDENTITY_TAG)
        );
    }
}
