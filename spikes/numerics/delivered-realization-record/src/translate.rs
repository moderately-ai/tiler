//! **Proposed `tiler_build::realization`** — the one exhaustive translation.
//!
//! `tiler-build` is the only crate that can see both authorities: `tiler-artifact`
//! depends on `tiler-ir` alone, `tiler-compiler` has no artifact edge, and
//! `tiler-build` already depends on both. That is the same derivation
//! `express-metal-honourability-in-the-shared-form` recorded for the Metal
//! projection, and it applies unchanged.
//!
//! # What "exhaustive" is protecting
//!
//! The translation matches every subject, dimension, disposition, structured
//! means, and provenance variant. It **never** reconstructs evidence from flags,
//! target names, neighbouring dtypes, profile digests, or outer value shape. The
//! reason is measured rather than stylistic: ADR 0076's forcing measurement is
//! that under `-fmetal-math-mode=relaxed` the emitted module declares
//! `air.compile.fast_math_disable` while every floating-point operation carries a
//! fast-math licence, so a translator that inferred a realization from any
//! readable proxy would write the opposite of the truth into the record a
//! reference comparison then trusts.
//!
//! Dispositions are not translated at all — they are **derived** by the artifact
//! builder from the obligations that arrive. A translator that carried a
//! disposition alongside the obligations would be carrying the same claim twice,
//! and the two copies could disagree.

use tiler_artifact::program::TargetProfileRef;

use crate::compiler_view::DeliveredRealizationView;
use crate::record::{
    DeliveredRealizationBuilder, DeliveredRealizationError, DeliveredRealizationRecord,
    TargetEvidenceDeclaration,
};
use crate::shared::ScalarArithmeticSubject;

/// A typed rejection while translating compiler evidence into an artifact record.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RealizationTranslationError {
    /// The compiler's profile key or descriptor is not a governed artifact
    /// identity.
    MalformedProfile,
    /// One obligation named a subject the compiler view did not offer.
    ///
    /// This is the check that makes the translation a *proof* rather than a copy:
    /// `tiler-build` owes that the translated subject and obligation references
    /// agree with the compiler view it read, and an obligation whose subject is
    /// not in that view is a view the translator misread.
    ObligationSubjectNotOffered,
    /// The artifact record refused the translated declaration.
    Record(DeliveredRealizationError),
}

impl From<DeliveredRealizationError> for RealizationTranslationError {
    fn from(value: DeliveredRealizationError) -> Self {
        Self::Record(value)
    }
}

/// Translates one plan's compiler evidence into the artifact record.
///
/// `entry_subjects` binds each packaged entry ordinal to the subject governing
/// it. `tiler-build` states it because the compiler knows which contract governs
/// which stage and the neutral artifact cannot derive it: an entry's
/// `NumericalRealization` carries eight behaviour dimensions and no arithmetic
/// type.
///
/// # Errors
///
/// Returns [`RealizationTranslationError`] for a malformed profile identity, an
/// obligation naming a subject outside the view, or any typed record rejection.
pub fn translate(
    view: DeliveredRealizationView<'_>,
    profile: &TargetProfileRef,
    entry_subjects: &[(u32, ScalarArithmeticSubject)],
) -> Result<DeliveredRealizationRecord, RealizationTranslationError> {
    if view.profile_key() != profile.key.as_str()
        || view.profile_descriptor() != profile.descriptor.as_bytes()
    {
        return Err(RealizationTranslationError::MalformedProfile);
    }

    let mut builder = DeliveredRealizationBuilder::new(profile.clone());

    // Subjects first, complete over all eleven dimensions. Walking the dense
    // resolution array rather than eleven named getters is what keeps a widened
    // vocabulary a build error at one array literal instead of a silently short
    // translation.
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
                // path here that renders the means to a key and re-parses it.
                means: evidence.means().clone(),
                profile: profile.clone(),
                source: evidence.source().clone(),
            },
        )?;
    }

    for (entry, subject) in entry_subjects {
        if !offered.contains(subject) {
            return Err(RealizationTranslationError::ObligationSubjectNotOffered);
        }
        builder.bind_entry(*entry, &subject.identity())?;
    }

    Ok(builder.build()?)
}
