//! What one scalar program's own algebra decides, before any topology reads it.
//!
//! Two things belong here and they are the same thing seen twice: the
//! boundary-tensor obligations a fold's contributor read and owning write
//! carry, and the topology-independent table [`split_family`] derives from the
//! scalar program alone. Keeping them out of the admissions is what makes a
//! family admitted by one topology and refused by another an explicit
//! [`ParallelFamily`] decision rather than a difference between independently
//! maintained match tables.

use crate::schedule::model::{ContributorOrder, ReductionPass, ScalarProgram, TensorRole};
use crate::shape::Axis;

/// Which boundary tensor one fold's contributor read is required to bind.
///
/// Two obligations rather than one role, because two different facts decide it.
/// A family whose scalar program *carries its own prologue* reads the original
/// input, since that is what the prologue applies to; a pass that folds values an
/// earlier dispatch staged reads the intermediate holding them. Both are exact,
/// and a region binding anything else is describing a different computation.
///
/// [`ScalarProgram::StrictSerialSum`] states neither. It says how contributors
/// combine and nothing about where they live, so `sum(x)` over a declared input
/// and the same fold over a materialized prologue's result are one scalar program
/// over two tensors. Requiring the intermediate would make the vocabulary unable
/// to express the first without an identity prologue region — a materialization,
/// and its observable rounding boundary, that no caller's program asked for — and
/// requiring the input would lose the second. Admitting both is what makes the
/// region's own access the thing that says which, rather than a rule guessing it
/// from the fold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ContributorTensor {
    /// This boundary tensor and no other.
    Exactly(TensorRole),
    /// Any input access; declared-interface association belongs to the compiler.
    DeclaredInput,
    /// The fold's declared contributor domain, wherever the plan placed it: the
    /// input tensor the program folds directly, or a materialized intermediate
    /// when a prologue region wrote it.
    DeclaredDomain,
}

impl ContributorTensor {
    /// Returns whether one read's boundary tensor discharges this obligation.
    pub(super) fn admits(self, tensor: TensorRole) -> bool {
        match self {
            Self::Exactly(required) => tensor == required,
            Self::DeclaredInput => matches!(tensor, TensorRole::Input),
            Self::DeclaredDomain => {
                tensor == TensorRole::Intermediate || Self::DeclaredInput.admits(tensor)
            }
        }
    }
}

/// Which boundary tensor one fold's owning write is required to commit to.
///
/// The write counterpart of [`ContributorTensor`], and it splits on a different
/// fact. A read's tensor is decided by the *scalar program*: a family carrying its
/// own prologue reads the original input, and a pass folding staged partials reads
/// the intermediate holding them. A write's tensor is decided by neither the
/// scalar program nor the family — no fold's algebra says whether its result is
/// the caller's answer or a value a later region consumes. That is a property of
/// the surrounding cover, so the vocabulary must let the region's own access state
/// it rather than fix it per family.
///
/// What *is* fixed is the write of a pass that exists only to stage. A split's
/// partial pass produces partials its final pass folds; they are not any output,
/// and a partial pass committing one to a declared program output would publish
/// an unfolded fragment as the program's answer. That pass therefore carries
/// [`Self::Exactly`] and every committing pass carries [`Self::CoverAssigned`],
/// which is the asymmetry a reader should be able to derive rather than discover.
///
/// Neither variant admits [`TensorRole::Input`]. A region writing a declared input
/// would mutate a tensor the caller owns, whatever it folded to get there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CommittedTensor {
    /// This boundary tensor and no other, because the pass's role in a split
    /// decides it rather than the cover.
    Exactly(TensorRole),
    /// Whichever of the two internal boundary tensors the cover assigned this
    /// region: a declared program output when the region publishes one, or a
    /// materialized intermediate when a later region consumes the value.
    CoverAssigned,
}

impl CommittedTensor {
    /// Returns whether one write's boundary tensor discharges this obligation.
    pub(super) fn admits(self, tensor: TensorRole) -> bool {
        match self {
            Self::Exactly(required) => tensor == required,
            Self::CoverAssigned => {
                matches!(tensor, TensorRole::Intermediate | TensorRole::Output)
            }
        }
    }
}

/// What one reduction family commits when its contributor domain is empty.
///
/// Two obligations rather than two values of one field. An identity-seeded family
/// names a bit pattern it commits; an identity-less one has no *empty-domain*
/// value it could commit, so what it owes is a *precondition on the domain*
/// instead of a constant — a statement about the empty case alone, and not about
/// whether the family's algebra has a neutral element, which
/// [`ScalarProgram::StrictSerialMaximum`] keeps apart. A typed enum for the
/// reason [`SplitFamily`] is a struct: the exhaustive match that decides it is
/// what forces a family added later to state which obligation it carries rather
/// than inherit whichever it resembles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EmptyDomainContract {
    /// The family commits these bits when the reduced domain is empty.
    Identity {
        /// Empty-reduction identity bit pattern the scalar program declares.
        bits: u32,
    },
    /// The family has no identity, so a non-empty domain is its precondition.
    NoIdentity,
}

/// Which topology asks for one reduction family's contributor tensor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FamilyTopology {
    /// The complete fold runs serially in one region.
    Serial,
    /// One pass of a split across dispatches.
    MultiPass(ReductionPass),
    /// The complete split runs cooperatively in one workgroup.
    Cooperative,
}

/// Which parallel forms one reduction family can realize.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ParallelFamily {
    /// The partial pass and cooperative tile are admitted; `final_pass` records
    /// whether this same scalar program can also combine staged partials.
    Split { final_pass: bool },
    /// Only the serial topology is meaningful for this scalar program.
    SerialOnly,
}

/// What one scalar program's own algebra decides about every reduction topology.
///
/// Derived once by [`split_family`] and read by the serial, multi-pass, and
/// cooperative admissions. A family admitted by one and not another is therefore
/// an explicit [`ParallelFamily`] decision rather than a difference between
/// independently maintained match tables.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SplitFamily<'a> {
    /// Reduced axes the scalar program declares.
    pub(super) axes: &'a [Axis],
    /// Contributor combination order the scalar program declares.
    pub(super) order: &'a ContributorOrder,
    /// The family's empty-domain obligation.
    pub(super) empty_domain: EmptyDomainContract,
    /// Whether splitting this family's contributor sequence spends the
    /// contract's reassociation permission.
    ///
    /// True for every sum here, and false for the pinned extrema family alone.
    /// The exception is the family's own algebra rather than a relaxation of any
    /// contract: `Maximum` is associative and commutative on *every* binary32
    /// input — NaN is absorbing and `-0.0 < +0.0` is a total order — so every
    /// tree over the same contributors returns the same bits and a split changes
    /// no observable value. Requiring the permission for it would make a legal
    /// split need one the operation never spends, which is exactly the asymmetry
    /// `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY` states against
    /// `SOFTMAX_F32_FACT_SUM_FOLD_ORDER`.
    ///
    /// The permission is still *recorded* and cross-checked against the region's
    /// declared realization whatever this says, exactly as
    /// [`ReductionTopology::Contraction`] records a permission it does not
    /// consume: a topology disagreeing with its own contract is incoherent
    /// however the fold behaves.
    ///
    /// [`ReductionTopology::Contraction`]: crate::schedule::model::ReductionTopology::Contraction
    pub(super) consumes_reassociation: bool,
    /// Boundary-tensor obligation the complete fold or first split level reads.
    ///
    /// There is deliberately no write counterpart on this struct, and the
    /// absence is the claim: a read's tensor varies by *family*, because a
    /// family's prologue is what decides whether it reads the original input,
    /// while a write's varies only by *pass* — every committing pass carries
    /// [`CommittedTensor::CoverAssigned`] and a split's staging pass carries
    /// [`CommittedTensor::Exactly`], identically for every family. Carrying it
    /// here would let a family declare a write target it has no authority over,
    /// and would invite two families to disagree about one cover's decision.
    pub(super) contributor_tensor: ContributorTensor,
    /// Parallel forms this family admits.
    pub(super) parallel: ParallelFamily,
}

impl SplitFamily<'_> {
    /// Derives the contributor tensor one topology reads, or refuses that form.
    ///
    /// The serial fold, split partial pass, and cooperative tile all read the
    /// family's own contributor domain. A final pass, when the family admits one,
    /// reads exactly the intermediate its partial pass staged. Nothing here reads
    /// [`Self::consumes_reassociation`], so deriving a serial tensor cannot give
    /// that field serial meaning.
    pub(super) const fn read_tensor(self, topology: FamilyTopology) -> Option<ContributorTensor> {
        match topology {
            FamilyTopology::Serial => Some(self.contributor_tensor),
            FamilyTopology::MultiPass(ReductionPass::Partial) | FamilyTopology::Cooperative => {
                match self.parallel {
                    ParallelFamily::Split { .. } => Some(self.contributor_tensor),
                    ParallelFamily::SerialOnly => None,
                }
            }
            FamilyTopology::MultiPass(ReductionPass::Final) => match self.parallel {
                ParallelFamily::Split { final_pass: true } => {
                    Some(ContributorTensor::Exactly(TensorRole::Intermediate))
                }
                ParallelFamily::Split { final_pass: false } | ParallelFamily::SerialOnly => None,
            },
        }
    }
}

/// Decides one family's empty-domain obligation against an optional count.
///
/// The identity-seeded arm requires the strict sum's `+0.0`, which every family
/// carrying an identity here shares — required at each admission rather than at
/// one of them, so a split cannot introduce a second empty-domain answer. The
/// identity-less arm requires a non-empty domain, which is what replaces the
/// constant the family has no correct value for.
///
/// `contributors` is absent only at the serial admission of an identity-seeded
/// fold, where the count is irrelevant to the identity check and deriving it
/// would impose a new canonical-axes and overflow obligation. Parallel
/// admissions already require a count for their split structure and pass it
/// through. The identity-less arm requires a present, nonzero count.
///
/// **Non-emptiness of the whole sequence is non-emptiness of every partition
/// under an exactly covering split**, which is why this needs no per-partition
/// statement and no `has_value` flag on the partials. The split contract fixes
/// `partitions * contributors_per_partition` (times the round count, for a tile)
/// as *exactly* the contributor count, and refuses a zero partition count; a
/// product of nonzero factors equalling a nonzero total forces every factor
/// nonzero, so each partition folds at least one contributor and each staged
/// partial is a real maximum. A carried `has_value` would be a runtime flag that
/// is constantly true — storage in every slot and a branch in every combine, for
/// a fact the verifier settles here.
///
/// **Exact coverage is a premise of that argument, not a detail of it.**
/// [`crate::schedule::model::ContributorPartition::covers`] still rejects anything
/// else, and [`verify_contributor_coverage`] keeps that meaning on
/// [`ContributorCoverage::Exact`]. A split covering a *padded* sequence has
/// partitions whose real contributors may be none, so the factor argument does
/// not reach it: [`ContributorCoverage::IdentityPadded`] states the family's
/// padding identity and the verifier derives two-sided neutrality before
/// admitting the split.
///
/// [`verify_contributor_coverage`]: super::coverage::verify_contributor_coverage
/// [`ContributorCoverage::Exact`]: crate::schedule::model::ContributorCoverage::Exact
/// [`ContributorCoverage::IdentityPadded`]: crate::schedule::model::ContributorCoverage::IdentityPadded
pub(super) const fn empty_domain_is_satisfied(
    contract: EmptyDomainContract,
    contributors: Option<u64>,
) -> bool {
    match contract {
        EmptyDomainContract::Identity { bits } => bits == 0.0_f32.to_bits(),
        EmptyDomainContract::NoIdentity => {
            matches!(contributors, Some(contributors) if contributors != 0)
        }
    }
}

/// Derives one reduction family's topology-independent algebraic facts.
///
/// The serial, multi-pass, and cooperative admissions all read this one table.
/// The first split level always reads `contributor_tensor`; a final pass, when
/// admitted, reads the intermediate that level staged. A fused or squared
/// prologue admits no final pass because applying it to a partial would apply it
/// twice. The epilogue-carrying fold is explicitly serial-only because its
/// epilogue applies to the complete fold rather than to a fragment.
pub(super) fn split_family(program: &ScalarProgram) -> Option<SplitFamily<'_>> {
    match program {
        ScalarProgram::StrictSerialSum {
            axes,
            order,
            empty_identity_bits,
            ..
        } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::Identity {
                bits: *empty_identity_bits,
            },
            consumes_reassociation: true,
            contributor_tensor: ContributorTensor::DeclaredDomain,
            parallel: ParallelFamily::Split { final_pass: true },
        }),
        ScalarProgram::FusedMultiplyAddSerialSum {
            axes,
            order,
            empty_identity_bits,
            contraction,
            ..
        } => (!contraction).then_some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::Identity {
                bits: *empty_identity_bits,
            },
            consumes_reassociation: true,
            contributor_tensor: ContributorTensor::DeclaredInput,
            parallel: ParallelFamily::Split { final_pass: false },
        }),
        ScalarProgram::SquaredSerialSum {
            axes,
            order,
            empty_identity_bits,
            ..
        } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::Identity {
                bits: *empty_identity_bits,
            },
            consumes_reassociation: true,
            contributor_tensor: ContributorTensor::DeclaredInput,
            parallel: ParallelFamily::Split { final_pass: false },
        }),
        ScalarProgram::SquaredSerialSumThenEpilogue {
            axes,
            order,
            empty_identity_bits,
            ..
        } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::Identity {
                bits: *empty_identity_bits,
            },
            consumes_reassociation: true,
            contributor_tensor: ContributorTensor::DeclaredInput,
            parallel: ParallelFamily::SerialOnly,
        }),
        ScalarProgram::StrictSerialMaximum { axes, order, .. } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::NoIdentity,
            consumes_reassociation: false,
            contributor_tensor: ContributorTensor::DeclaredInput,
            parallel: ParallelFamily::Split { final_pass: true },
        }),
        // No pointwise or decode program folds anything, and the contraction
        // owns a distinct two-read topology rather than this one-read family.
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::PointwiseBf16(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictTensorContraction { .. } => None,
    }
}
