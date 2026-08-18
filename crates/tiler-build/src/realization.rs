//! The one exhaustive translation of compiler evidence into the artifact record.
//!
//! `tiler-build` is the only crate that can see both authorities: `tiler-artifact`
//! depends on `tiler-ir` alone, `tiler-compiler` has no artifact edge, and this
//! crate already depends on both. That is the same derivation
//! `express-metal-honourability-in-the-shared-form` recorded for the Metal
//! projection, and it applies unchanged.
//!
//! # What "exhaustive" is protecting
//!
//! The translation forwards every subject, dimension, structured means, and
//! provenance value the compiler's borrowed view holds. It **never** reconstructs
//! evidence from flags, target names, neighbouring dtypes, profile digests, or
//! outer value shape. The reason is measured rather than stylistic: ADR 0076's
//! forcing measurement is that under `-fmetal-math-mode=relaxed` the emitted
//! module declares `air.compile.fast_math_disable` while every floating-point
//! operation in it carries a fast-math licence set, so a translator that inferred
//! a realization from any readable proxy would write the opposite of the truth
//! into the record a reference comparison then trusts.
//!
//! Nothing is *matched* here in the sense of a `match` over the vocabulary, and
//! that is the stronger form of the same property rather than a weaker one. The
//! shared `tiler_ir::numerics` types cross both boundaries by value —
//! [`DimensionBehaviour`], [`HonouringMeans`] with its relaxation payload,
//! [`NumericalObligationKey`], and [`FactSourceProvenance`] — so widening any of
//! them is a build error at their own total encoders rather than a silently short
//! translation here. A per-variant match would be a second vocabulary to keep in
//! step with the first.
//!
//! Dispositions are not translated at all — they are **derived** by
//! [`DeliveredRealizationBuilder::build`] from the obligations that arrive. A
//! translator that carried a disposition alongside the obligations would be
//! carrying the same claim twice, and the two copies could disagree.
//!
//! [`DimensionBehaviour`]: tiler_artifact::program::DimensionBehaviour
//! [`HonouringMeans`]: tiler_artifact::program::HonouringMeans
//! [`NumericalObligationKey`]: tiler_artifact::program::NumericalObligationKey
//! [`FactSourceProvenance`]: tiler_artifact::program::FactSourceProvenance

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    DeliveredRealizationBuilder, DeliveredRealizationError, DeliveredRealizationRecord,
    ScalarArithmeticSubject, TargetEvidenceDeclaration, TargetProfileRef,
};
use tiler_compiler::session::DeliveredRealizationView;

/// A typed rejection while translating compiler evidence into an artifact record.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RealizationTranslationError {
    /// The compiler's profile key or descriptor is not the artifact's.
    ///
    /// The check that makes this a *transfer* rather than a re-attribution: the
    /// record's facts are attributed to the profile that declared them, and the
    /// artifact pins one profile across its portfolio. A translation that
    /// silently relabelled a fact would produce a record whose provenance names
    /// a target the evidence never spoke about.
    ProfileDisagreement,
    /// One obligation named a subject the compiler view did not offer.
    ///
    /// The check that makes the translation a *proof* rather than a copy:
    /// `tiler-build` owes that the translated subject and obligation references
    /// agree with the compiler view it read, and an obligation whose subject is
    /// not in that view is a view the translator misread.
    ObligationSubjectNotOffered,
    /// The view offered no policy subject to govern the packaged entries.
    ///
    /// A selected scalar contract always produces exactly one complete subject,
    /// so this is a compiler that selected none.
    NoPolicySubject,
    /// The view offered several subjects and nothing decides which governs an entry.
    ///
    /// Fail-closed rather than picking the first. Which arithmetic contract
    /// governs a packaged entry is a compiler fact, and an entry's own
    /// [`NumericalRealization`] carries ten behaviour dimensions and no
    /// arithmetic type, so nothing reachable from here can decide it. When the
    /// compiler grows a per-entry statement, this becomes a translation of that
    /// statement rather than a refusal.
    ///
    /// [`NumericalRealization`]: tiler_ir::schedule::NumericalRealization
    AmbiguousEntrySubject {
        /// How many policy subjects the view offered.
        subjects: usize,
    },
    /// The artifact record refused the translated declaration.
    Record(DeliveredRealizationError),
}

impl From<DeliveredRealizationError> for RealizationTranslationError {
    fn from(value: DeliveredRealizationError) -> Self {
        Self::Record(value)
    }
}

impl fmt::Display for RealizationTranslationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProfileDisagreement => {
                formatter.write_str("the compiler evidence names another target profile")
            }
            Self::ObligationSubjectNotOffered => {
                formatter.write_str("an obligation names a subject the compiler view did not offer")
            }
            Self::NoPolicySubject => {
                formatter.write_str("the compiler view offered no scalar-arithmetic policy subject")
            }
            Self::AmbiguousEntrySubject { subjects } => write!(
                formatter,
                "{subjects} policy subjects offered and no per-entry statement decides which governs an entry",
            ),
            Self::Record(cause) => write!(formatter, "the artifact record refused: {cause}"),
        }
    }
}

impl Error for RealizationTranslationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Record(cause) => Some(cause),
            Self::ProfileDisagreement
            | Self::ObligationSubjectNotOffered
            | Self::NoPolicySubject
            | Self::AmbiguousEntrySubject { .. } => None,
        }
    }
}

/// Translates one plan's compiler evidence into the artifact record.
///
/// `entries` is the artifact's packaged-entry count, in the **declared** flat
/// space [`ArtifactProgramBuilder::declare_realization`] documents: every
/// ordinal `0..entries` is bound, because every packaged entry executes under the
/// selected contract and the artifact refuses a record that leaves one unbound.
///
/// The entry-to-subject association is the one claim the neutral artifact cannot
/// derive — an entry's `NumericalRealization` carries ten behaviour dimensions
/// and no arithmetic type — so this function states it from the compiler's own
/// subject table, and refuses when that table cannot decide it.
///
/// # Errors
///
/// Returns [`RealizationTranslationError`] for a profile the compiler evidence
/// does not name, an obligation naming a subject outside the view, a view with
/// no policy subject or with several and nothing deciding among them, or any
/// typed record rejection.
///
/// [`ArtifactProgramBuilder::declare_realization`]: tiler_artifact::program::ArtifactProgramBuilder::declare_realization
pub fn translate(
    view: DeliveredRealizationView<'_>,
    profile: &TargetProfileRef,
    entries: u32,
) -> Result<DeliveredRealizationRecord, RealizationTranslationError> {
    if view.profile_key() != profile.key.as_str()
        || view.profile_descriptor() != profile.descriptor.as_bytes()
    {
        return Err(RealizationTranslationError::ProfileDisagreement);
    }

    let mut builder = DeliveredRealizationBuilder::new(profile.clone());

    // Subjects first, complete over all eleven dimensions. Handing the dense
    // resolution array through rather than reading eleven named getters is what
    // keeps a widened vocabulary a build error at one array type instead of a
    // silently short translation.
    let offered: Vec<ScalarArithmeticSubject> = view
        .scalar_arithmetic()
        .map(|contract| contract.subject().clone())
        .collect();
    for contract in view.scalar_arithmetic() {
        builder
            .declare_scalar_arithmetic(contract.subject().identity(), *contract.resolutions())?;
    }

    for obligation in view.obligations() {
        if !offered.contains(obligation.subject()) {
            return Err(RealizationTranslationError::ObligationSubjectNotOffered);
        }
        let evidence = obligation.evidence();
        builder.require(
            &obligation.subject().identity(),
            obligation.dimension(),
            obligation.locus(),
            obligation.required(),
            TargetEvidenceDeclaration {
                declared: evidence.declared(),
                // Cloned structurally, relaxation payload included. There is no
                // path here that renders the means to a label and re-parses it —
                // `HonouringMeans::label` collapses every declared relaxation to
                // one string, so a record built from labels could not say *which*
                // relaxation made a requirement honourable.
                means: evidence.means().clone(),
                profile: profile.clone(),
                source: evidence.source().clone(),
            },
        )?;
    }

    let governing = match offered.as_slice() {
        [] => return Err(RealizationTranslationError::NoPolicySubject),
        [only] => only,
        several => {
            return Err(RealizationTranslationError::AmbiguousEntrySubject {
                subjects: several.len(),
            });
        }
    };
    for entry in 0..entries {
        builder.bind_entry(entry, &governing.identity())?;
    }

    Ok(builder.build()?)
}
