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
//! # The retention contract
//!
//! **A published value may be read by more than one later stage, and its
//! retention is recorded rather than implied.** The record is
//! [`StagedIntermediate`]: one per *read*, each naming the producing stage, the
//! reading stage, the two boundaries, and — on every record of the same published
//! value — the last stage across which that value stays live
//! ([`StagedIntermediate::retained_through`]). So an intermediate's lifetime is
//! `producer..=retained_through`, and it is a checked fact of the chain rather
//! than something a reader infers from stage order.
//!
//! It is *derived* from the declared readers, which is this module's own rule
//! rather than an exception to it: a caller declares where each input boundary's
//! value comes from and [`VerifiedIndexRegionSequence::try_new`] proves the
//! result well formed, so the lifetime that follows from those declarations is
//! computed and recorded rather than separately asserted. A separately declared
//! span would be a second authority over one fact, and the two could disagree.
//!
//! Three rules bound it, and they are what keep the chain a chain:
//!
//! - a source names an **earlier** stage, never this one and never a later one,
//!   so the reader graph is acyclic and every read follows its production;
//! - a **non-final stage publishes exactly one value**, so "the value stage `p`
//!   published" names one thing; a stage that hands two values on is a separate
//!   capability this vocabulary does not have;
//! - a published value has **at least one** reader, checked over the whole chain
//!   rather than at the following stage, because a value read three stages later
//!   is not unread at the stage in between.
//!
//! This is what the softmax needs and what the reduction-then-pass shape did not:
//! `tiler::softmax-f32@1` publishes its exponentials, folds them to a denominator,
//! and then reads *both* in its final pass. The alternative — letting a non-final
//! stage hand several values on — reaches the same staging only by copying the
//! exponentials through the folding stage verbatim, which puts a materialization
//! that is no part of what the operation means into a region's canonical identity.
//!
//! The public surface here is a concrete draft pending Tom's review; see
//! `tickets/accept-the-multi-region-index-realization-surface.md` for the
//! originally accepted shape and
//! `tickets/accept-the-multi-reader-index-realization-retention.md` for this
//! widening.

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
    /// The value the named earlier stage published.
    ///
    /// Any earlier stage, not only the immediately preceding one: the ordinal is
    /// what makes a value read several stages after it was produced expressible,
    /// and it is load-bearing rather than derivable. Reading the same published
    /// value at two boundaries — of one stage or of two — is two reads of one
    /// value and is admitted; the module header states the retention contract
    /// that follows.
    Intermediate(usize),
}

/// One read of a value one stage published and a later stage consumes.
///
/// Every field is a checked fact about the two regions, not a caller assertion:
/// [`VerifiedIndexRegionSequence::try_new`] reads the producing output boundary
/// and the consuming input boundary out of the regions themselves and refuses
/// when they disagree.
///
/// **The record is per read, not per value.** A published value read by two
/// stages — or twice by one — yields two records agreeing on [`Self::producer`],
/// [`Self::producer_output`], [`Self::value_type`], [`Self::shape`], and
/// [`Self::retained_through`], and differing in [`Self::consumer`] and
/// [`Self::consumer_input`]. That granularity is not new: this record has always
/// named one *consuming boundary*, and a chain in which every published value has
/// one reader — which is every chain any registered law spells — yields exactly
/// the records it always did.
///
/// **Ownership.** An intermediate belongs to the sequence. It is neither an
/// occurrence input nor an occurrence result: no [`StagedInputSource::Occurrence`]
/// names it, and it is produced by a non-final stage, whose writes therefore
/// never leave the sequence.
///
/// **Lifetime.** It is created by [`Self::producer`] and dead after
/// [`Self::retained_through`], which is the last stage that reads it and is never
/// earlier than [`Self::consumer`]. A value staying live across a stage that does
/// not mention it is exactly what this record expresses, and the span is stated
/// here rather than left to be inferred from stage order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedIntermediate {
    producer: usize,
    producer_output: VerifiedTensorId,
    consumer: usize,
    consumer_input: VerifiedTensorId,
    retained_through: usize,
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
    /// Returns the ordered stage this record's read happens at.
    ///
    /// One of possibly several readers; [`Self::retained_through`] is the last of
    /// them and is what bounds the value's lifetime.
    #[must_use]
    pub const fn consumer(&self) -> usize {
        self.consumer
    }
    /// Returns the consuming stage's input tensor boundary.
    #[must_use]
    pub const fn consumer_input(&self) -> VerifiedTensorId {
        self.consumer_input
    }
    /// Returns the last ordered stage across which this value stays live.
    ///
    /// The maximum [`Self::consumer`] over every read of the same published
    /// value, so the value's lifetime is `producer()..=retained_through()`. It is
    /// equal to [`Self::consumer`] exactly when this record is the value's last
    /// read, and therefore on every record of a value with one reader.
    #[must_use]
    pub const fn retained_through(&self) -> usize {
        self.retained_through
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
    /// published value is never read or is claimed by a stage at or before the one
    /// that published it, a non-final stage does not produce exactly the one value
    /// it publishes, or a producing and consuming boundary disagree on element
    /// type or shape.
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

        // What each non-final stage publishes, collected before any source is
        // read. A source names a producer by ordinal rather than by adjacency, so
        // it must be answered against what that stage actually published rather
        // than against whichever value happened to be in flight when the walk
        // reached the reader.
        let mut published: Vec<(VerifiedTensorId, ResolvedValueType, Shape)> =
            Vec::with_capacity(stages.len().saturating_sub(1));
        for (position, stage) in stages.iter().enumerate() {
            if position + 1 == stages.len() {
                break;
            }
            let mut outputs = stage
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Output);
            let Some(output) = outputs.next() else {
                return Err(IndexRegionSequenceError::NotChained { stage: position });
            };
            if outputs.next().is_some() {
                // Only the final stage's writes leave the sequence, and
                // `Intermediate(p)` names one value, so a non-final stage
                // publishing two has nothing coherent for a reader to name.
                // Widening *that* is a separate capability from widening how many
                // stages may read one value, and this vocabulary has only the
                // second.
                return Err(IndexRegionSequenceError::NotChained { stage: position });
            }
            let Some(shape) = output.shape().as_static().cloned() else {
                return Err(IndexRegionSequenceError::SymbolicIntermediate {
                    stage: position,
                    slot: 0,
                });
            };
            published.push((output.id(), output.value_type().clone(), shape));
        }

        let mut intermediates = Vec::new();
        let mut reads = vec![0_usize; published.len()];
        for (position, stage) in stages.iter().enumerate() {
            let inputs = inputs_of(stage);
            let stage_sources = &sources[position];
            if stage_sources.len() != inputs.len() {
                return Err(IndexRegionSequenceError::SourceArity { stage: position });
            }
            for (slot, source) in stage_sources.iter().enumerate() {
                let (tensor, value_type, shape) = &inputs[slot];
                match source {
                    StagedInputSource::Occurrence(_) => {}
                    StagedInputSource::Intermediate(producer) => {
                        // A value is readable only after the stage that published
                        // it. With adjacency gone this bound is what keeps the
                        // reader graph acyclic, and it also rules out the final
                        // stage as a producer, whose writes leave the sequence.
                        let Some((owed_output, owed_type, owed_shape)) =
                            published.get(*producer).filter(|_| *producer < position)
                        else {
                            return Err(IndexRegionSequenceError::UnavailableIntermediate {
                                stage: position,
                                slot,
                            });
                        };
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
                            producer: *producer,
                            producer_output: *owed_output,
                            consumer: position,
                            consumer_input: *tensor,
                            // Filled once every read is known: the span is a
                            // property of the value, and no single read can see
                            // it.
                            retained_through: position,
                            value_type: value_type.clone(),
                            shape: shape.clone(),
                        });
                        reads[*producer] = reads[*producer].saturating_add(1);
                    }
                }
            }
        }

        // Checked over the whole chain rather than at the following stage: a
        // value read three stages later is not unread at the stage in between.
        if let Some(producer) = reads.iter().position(|count| *count == 0) {
            // A published value nothing reads is staged with no consumer: a leak
            // whose owner nothing downstream could name.
            return Err(IndexRegionSequenceError::IntermediateNeverRead { producer });
        }

        // The retention span, derived from the declared readers and then recorded
        // on every record of the value it belongs to.
        let mut retained = vec![0_usize; published.len()];
        for read in &intermediates {
            retained[read.producer] = retained[read.producer].max(read.consumer);
        }
        for read in &mut intermediates {
            read.retained_through = retained[read.producer];
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
///
/// **Injectivity over the widened source vocabulary, and why no byte moved to get
/// it.** [`StagedInputSource::Intermediate`] now admits any earlier producer
/// rather than only the immediately preceding stage, so the ordinal it carries is
/// load-bearing where it used to be redundant — two chains over identical regions
/// whose final stage reads a value published two stages back rather than one are
/// different realizations and must not encode alike. They do not: the producer
/// ordinal is written in full under tag `2`, exactly as it always was, and
/// `push_len` is injective over the whole `usize` range rather than over the range
/// the chain rules happened to admit. So this function is unchanged, and *because*
/// it is unchanged every chain that was expressible before encodes byte for byte
/// as it did — the admitted preimage set widened while the map did not. The
/// widening is therefore identity-neutral by construction rather than by survey,
/// and `the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte` in
/// [`super::law`] pins the claim over exact bytes anyway.
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
    /// An input claims a published value that is not available to it.
    ///
    /// The named producer is not an earlier stage: it is this stage itself, a
    /// later one, or an ordinal no stage occupies. Every earlier stage is
    /// non-final and therefore published exactly one value, so that bound is the
    /// whole of the condition — reading a value several stages after it was
    /// published is admitted, and reading one before it exists is not.
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
    /// A stage published a value that no later stage reads.
    ///
    /// Checked over the whole chain, so a value first read several stages later
    /// is not this refusal.
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
                    "stage {stage} input {slot} claims a value no earlier stage published"
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
                write!(
                    formatter,
                    "stage {producer} published a value no later stage reads"
                )
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
        DomainRole, FrozenScalarRegistry, IndexRegionBuilder, ScalarAttributes, add_f32_scalar_op,
        constant_f32_scalar_op, multiply_f32_scalar_op,
    };
    use crate::semantic::{
        CanonicalField, CanonicalValue, F32, F32_CONSTANT_BITS_ATTRIBUTE, TypeKey,
    };
    use crate::shape::{Extent, Shape};

    fn scalars() -> FrozenScalarRegistry {
        FrozenScalarRegistry::standard().expect("the standard scalar registry is coherent")
    }

    /// Emits `out[r] = fold(+, in[r, *])` over `[rows, columns]`.
    ///
    /// A real reduction rather than a pointwise stand-in, because the softmax's
    /// staging is characterized by the *shapes* its stages hand across: a fold
    /// publishes one value per row where the pass it feeds is one per point, and a
    /// chain built from shape-preserving regions alone could not tell the two
    /// apart.
    fn row_fold_region(rows: u64, columns: u64) -> VerifiedIndexRegion {
        let mut builder =
            IndexRegionBuilder::new(scalars()).expect("the standard registry admits a builder");
        let row = builder
            .dimension(DomainRole::Parallel, Extent::new(rows))
            .expect("the kept dimension");
        let column = builder
            .dimension(DomainRole::Reduction, Extent::new(columns))
            .expect("the folded dimension");
        let row_coordinate = builder.dimension_expr(row).expect("its induction variable");
        let column_coordinate = builder
            .dimension_expr(column)
            .expect("its induction variable");
        let input = builder
            .tensor(
                crate::index::TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([rows, columns]),
            )
            .expect("the folded boundary");
        let output = builder
            .tensor(
                crate::index::TensorRole::Output,
                F32::resolved_type(),
                Shape::from_dims([rows]),
            )
            .expect("the published boundary");
        let contributor = builder
            .read(input, &[row, column], &[row_coordinate, column_coordinate])
            .expect("one contributor per point of the folded axis");
        let seed = builder
            .apply(
                constant_f32_scalar_op(),
                exact_f32_attributes(0.0_f32.to_bits()),
                &[],
            )
            .expect("the governed constant applies")
            .get(0)
            .expect("a constant yields one result");
        let folded = builder
            .reduce(&[column], &[seed], &[contributor], |body| {
                let state = body.state(0).expect("one state");
                let value = body.contributor(0).expect("one contributor");
                let accumulated = body
                    .apply(
                        add_f32_scalar_op(),
                        ScalarAttributes::empty(),
                        &[state, value],
                    )?
                    .get(0)
                    .expect("the governed add yields one result");
                body.yield_values(&[accumulated])
            })
            .expect("the fold builds")
            .get(0)
            .expect("a one-state fold yields one result");
        let write = builder
            .write(output, &[row], &[row_coordinate])
            .expect("a complete write");
        builder.output(write, folded).expect("an output root");
        builder.build().expect("the fold region verifies")
    }

    /// Emits `out[r, c] = mul(full[r, c], per_row[r])` over `[rows, columns]`.
    ///
    /// Its boundary order is `(full, per row)`, which is the order its stage's
    /// sources are declared in and therefore the order the chain binds them.
    fn row_pointwise_region(rows: u64, columns: u64) -> VerifiedIndexRegion {
        let mut builder =
            IndexRegionBuilder::new(scalars()).expect("the standard registry admits a builder");
        let row = builder
            .dimension(DomainRole::Parallel, Extent::new(rows))
            .expect("the row dimension");
        let column = builder
            .dimension(DomainRole::Parallel, Extent::new(columns))
            .expect("the column dimension");
        let row_coordinate = builder.dimension_expr(row).expect("its induction variable");
        let column_coordinate = builder
            .dimension_expr(column)
            .expect("its induction variable");
        let full = builder
            .tensor(
                crate::index::TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([rows, columns]),
            )
            .expect("the per-point boundary");
        let per_row = builder
            .tensor(
                crate::index::TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([rows]),
            )
            .expect("the per-row boundary");
        let output = builder
            .tensor(
                crate::index::TensorRole::Output,
                F32::resolved_type(),
                Shape::from_dims([rows, columns]),
            )
            .expect("the result boundary");
        let element = builder
            .read(full, &[row, column], &[row_coordinate, column_coordinate])
            .expect("the per-point read");
        let scale = builder
            .read(per_row, &[row], &[row_coordinate])
            .expect("the per-row read");
        let product = builder
            .apply(
                multiply_f32_scalar_op(),
                ScalarAttributes::empty(),
                &[element, scale],
            )
            .expect("the governed multiply applies")
            .get(0)
            .expect("multiply yields one result");
        let write = builder
            .write(output, &[row, column], &[row_coordinate, column_coordinate])
            .expect("a complete write");
        builder.output(write, product).expect("an output root");
        builder.build().expect("the pass region verifies")
    }

    /// Returns the governed constant's attribute record for one exact payload.
    fn exact_f32_attributes(bits: u32) -> ScalarAttributes {
        ScalarAttributes::new(
            CanonicalValue::record([CanonicalField::new(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValue::float_bits(
                    TypeKey::new("tiler", "f32", 1).expect("the governed f32 key is valid"),
                    bits.to_be_bytes(),
                )
                .expect("an exact binary32 payload is canonical"),
            )])
            .expect("a one-field record is canonical"),
        )
        .expect("a canonical record is valid scalar attributes")
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
        // An ordinal no stage occupies: only non-final stages publish, so a
        // two-stage chain has exactly one producer to name.
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
    }

    /// A value read at or before the stage that publishes it.
    ///
    /// **Separate from the out-of-range case, and the separation is the finding.**
    /// A two-stage chain cannot exhibit this: naming stage one there is already
    /// an ordinal no producer occupies, so an implementation with no ordering
    /// bound at all still refuses it and the assertion proves nothing about the
    /// bound. Three stages give both references a *live* producer whose element
    /// type and shape agree — so the only thing standing between them and
    /// acceptance is the rule that a value is readable strictly after it is
    /// published, which is what keeps the reader graph acyclic now that adjacency
    /// no longer does.
    #[test]
    fn a_value_read_at_or_before_its_producing_stage_refuses() {
        // Self-reference: stage one claims its own published value. Forward
        // reference: the head of the chain claims a value published later. The
        // refusal is compared through `expect_err` rather than against an `Ok`
        // pattern so a regression reports the refusal it produced instead of
        // rendering three verified regions.
        for (sources, expected) in [
            (
                vec![
                    vec![StagedInputSource::Occurrence(0)],
                    vec![
                        StagedInputSource::Occurrence(1),
                        StagedInputSource::Intermediate(1),
                    ],
                    vec![
                        StagedInputSource::Intermediate(0),
                        StagedInputSource::Intermediate(1),
                    ],
                ],
                IndexRegionSequenceError::UnavailableIntermediate { stage: 1, slot: 1 },
            ),
            (
                vec![
                    vec![StagedInputSource::Intermediate(1)],
                    vec![
                        StagedInputSource::Occurrence(0),
                        StagedInputSource::Intermediate(0),
                    ],
                    vec![
                        StagedInputSource::Occurrence(1),
                        StagedInputSource::Intermediate(1),
                    ],
                ],
                IndexRegionSequenceError::UnavailableIntermediate { stage: 0, slot: 0 },
            ),
        ] {
            assert_eq!(
                VerifiedIndexRegionSequence::try_new(
                    vec![square(), consumer(), consumer()],
                    sources,
                )
                .map(|_| ())
                .expect_err("a value is readable only after the stage that published it"),
                expected
            );
        }
        // The same three regions and the same boundaries, wired forward: this is
        // what the two chains above would have been had the bound not refused
        // them, and it is admitted. Without it the refusals prove only that
        // *something* about those chains was wrong.
        let forward = VerifiedIndexRegionSequence::try_new(
            vec![square(), consumer(), consumer()],
            vec![
                vec![StagedInputSource::Occurrence(0)],
                vec![
                    StagedInputSource::Occurrence(1),
                    StagedInputSource::Intermediate(0),
                ],
                vec![
                    StagedInputSource::Occurrence(2),
                    StagedInputSource::Intermediate(1),
                ],
            ],
        )
        .expect("a strictly forward three-stage chain is well formed");
        assert_eq!(forward.stage_count(), 3);
    }

    /// One published value read at two boundaries of one stage.
    ///
    /// **This is the refusal the retention widening removes, asserted as an
    /// admission.** Before it, a second claim on a handed value found the slot
    /// cleared and reported `UnavailableIntermediate`; a value with two readers
    /// is now what the model expresses. The two records agree on everything about
    /// the value and differ only in the boundary each read binds.
    #[test]
    fn one_published_value_read_twice_by_one_stage_chains() {
        let sequence = VerifiedIndexRegionSequence::try_new(
            vec![square(), consumer()],
            vec![
                vec![StagedInputSource::Occurrence(0)],
                vec![
                    StagedInputSource::Intermediate(0),
                    StagedInputSource::Intermediate(0),
                ],
            ],
        )
        .expect("squaring the published value is two reads of one value");
        let [first, second] = sequence.intermediates() else {
            panic!("two boundaries reading one value are two records")
        };
        assert_eq!(first.producer(), second.producer());
        assert_eq!(first.producer_output(), second.producer_output());
        assert_eq!(first.consumer(), 1);
        assert_eq!(second.consumer(), 1);
        assert_ne!(first.consumer_input(), second.consumer_input());
        assert_eq!(first.retained_through(), 1);
        assert_eq!(second.retained_through(), 1);
    }

    /// The softmax's staging, expressible at the sequence layer.
    ///
    /// **This is the chain `tiler::softmax-f32@1` needs, and the one wall this
    /// module was the second half of.** Its four stages are the pinned formula's:
    /// `S0` folds the row maximum `m`; `S1` reads the scores and `m` and publishes
    /// the exponentials `e`; `S2` folds `e` into the denominator `d`; and `S3`
    /// reads `e` **again**, alongside `d`, and writes the row. `e` is the value
    /// that survives a stage that produces something else, which is what every
    /// one of the four stagings the ticket enumerated ran aground on.
    ///
    /// The regions here stand for those five steps in their *boundaries* — the
    /// interfaces are where the chain checks anything — while the arithmetic they
    /// carry is the fold and the product the other tests in this module use. The
    /// law that emits the softmax's actual scalar programs is a separate piece of
    /// work; what is proved here is that the shape it must produce is now
    /// expressible and checked rather than refused.
    #[test]
    fn the_softmax_staging_publishing_the_exponentials_chains() {
        let sequence = VerifiedIndexRegionSequence::try_new(
            vec![
                // S0: x[3,4] -> m[3]
                row_fold_region(3, 4),
                // S1: x[3,4], m[3] -> e[3,4]
                row_pointwise_region(3, 4),
                // S2: e[3,4] -> d[3]
                row_fold_region(3, 4),
                // S3: e[3,4], d[3] -> r[3,4]
                row_pointwise_region(3, 4),
            ],
            vec![
                vec![StagedInputSource::Occurrence(0)],
                vec![
                    StagedInputSource::Occurrence(0),
                    StagedInputSource::Intermediate(0),
                ],
                vec![StagedInputSource::Intermediate(1)],
                vec![
                    StagedInputSource::Intermediate(1),
                    StagedInputSource::Intermediate(2),
                ],
            ],
        )
        .expect("the exponentials survive the denominator's fold");

        assert_eq!(sequence.stage_count(), 4);
        // Four reads of three published values: `m` once, `e` twice, `d` once.
        let reads = sequence
            .intermediates()
            .iter()
            .map(|read| (read.producer(), read.consumer(), read.retained_through()))
            .collect::<Vec<_>>();
        assert_eq!(reads, [(0, 1, 1), (1, 2, 3), (1, 3, 3), (2, 3, 3)]);

        // The retention claim, stated rather than inferred: `e` is published by
        // stage one and stays live through stage three, across stage two — which
        // reads it, and publishes something else entirely.
        let exponentials = sequence
            .intermediates()
            .iter()
            .filter(|read| read.producer() == 1)
            .collect::<Vec<_>>();
        assert_eq!(exponentials.len(), 2);
        for read in &exponentials {
            assert_eq!(read.retained_through(), 3);
            assert_eq!(read.shape(), &Shape::from_dims([3, 4]));
            assert_eq!(
                read.producer_output(),
                exponentials[0].producer_output(),
                "both reads name the one boundary stage one published"
            );
        }
        // The denominator is a different value of a different shape, so the two
        // reads at stage three are not one value read twice.
        let [denominator] = sequence
            .intermediates()
            .iter()
            .filter(|read| read.producer() == 2)
            .collect::<Vec<_>>()[..]
        else {
            panic!("the denominator has one reader")
        };
        assert_eq!(denominator.shape(), &Shape::from_dims([3]));
        assert_ne!(
            denominator.producer_output(),
            exponentials[0].producer_output()
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
