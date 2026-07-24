//! Shared closure-convenience shape for ADR 0071 checked builders.
//!
//! Every target-neutral IR layer in this crate follows the same construction
//! discipline: a transactional builder checks local invariants on each
//! insertion, and a consuming `build()` runs whole-object verification before
//! returning an opaque verified product. [`CheckedBuildError`] and
//! [`build_checked`] let each layer offer a closure-based convenience that
//! delegates to that exact path without duplicating it and without erasing the
//! layer's concrete error types into one lossy error.

use std::error::Error;
use std::fmt;

/// One composed failure from a checked closure-based builder convenience.
///
/// ADR 0071 builders raise two structurally distinct failures that must never
/// be collapsed into a single lossy error:
///
/// - an insertion-time *admission* rejection, raised while the authoring
///   closure (or the initial builder construction) populates the mutable draft;
///   and
/// - a whole-object *verification* failure, raised by the consuming `build()`
///   step, which additionally carries every deterministic diagnostic and
///   recoverable builder ownership.
///
/// The variant keeps both kinds separate and fully typed so a caller can react
/// to a rejected insertion differently from a recoverable verification
/// diagnostic. It is generic over each layer's own two error types rather than a
/// universal untyped error, so later schedule, kernel, and program builders
/// reuse the same shape without erasing their concrete admission and
/// verification types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedBuildError<Admission, Verification> {
    /// The builder rejected construction or an insertion before verification.
    Admission(Admission),
    /// Consuming whole-object verification rejected the assembled draft.
    Verification(Verification),
}

impl<Admission, Verification> fmt::Display for CheckedBuildError<Admission, Verification>
where
    Admission: fmt::Display,
    Verification: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admission(error) => write!(f, "builder admission failed: {error}"),
            Self::Verification(error) => write!(f, "whole-object verification failed: {error}"),
        }
    }
}

impl<Admission, Verification> Error for CheckedBuildError<Admission, Verification>
where
    Admission: Error + 'static,
    Verification: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            Self::Verification(error) => Some(error),
        }
    }
}

/// Runs a closure-based convenience over an already-constructed checked builder.
///
/// This is the shared mechanism behind each layer's public closure convenience
/// (for example [`crate::index::IndexRegionBuilder::build_with`]). It scopes the
/// mutable draft to `assemble`, then consumes it through `verify`, mapping the
/// two failure kinds onto the matching [`CheckedBuildError`] variant. Because
/// `assemble` only borrows the draft, it can never reach the consuming `verify`
/// step itself; the verified product escapes solely as the successful return
/// value.
pub(crate) fn build_checked<Builder, Verified, Admission, Verification>(
    mut builder: Builder,
    assemble: impl FnOnce(&mut Builder) -> Result<(), Admission>,
    verify: impl FnOnce(Builder) -> Result<Verified, Verification>,
) -> Result<Verified, CheckedBuildError<Admission, Verification>> {
    assemble(&mut builder).map_err(CheckedBuildError::Admission)?;
    verify(builder).map_err(CheckedBuildError::Verification)
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;
    use std::fmt;

    use super::{CheckedBuildError, build_checked};

    /// A toy builder standing in for a future schedule/kernel/program layer,
    /// with admission and verification error types distinct from the index
    /// layer's. It proves the shared shape composes for any checked builder
    /// without an untyped abstraction.
    #[derive(Default)]
    struct ToyBuilder {
        entries: u32,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct ToyAdmission;
    impl fmt::Display for ToyAdmission {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("toy admission")
        }
    }
    impl std::error::Error for ToyAdmission {}

    #[derive(Debug, Eq, PartialEq)]
    struct ToyVerification;
    impl fmt::Display for ToyVerification {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("toy verification")
        }
    }
    impl std::error::Error for ToyVerification {}

    impl ToyBuilder {
        fn push(&mut self, admit: bool) -> Result<(), ToyAdmission> {
            if admit {
                self.entries += 1;
                Ok(())
            } else {
                Err(ToyAdmission)
            }
        }
        fn build(self) -> Result<u32, ToyVerification> {
            if self.entries == 0 {
                Err(ToyVerification)
            } else {
                Ok(self.entries)
            }
        }
    }

    #[test]
    fn closure_path_matches_manual_construction() {
        let via_closure = build_checked(
            ToyBuilder::default(),
            |builder| {
                builder.push(true)?;
                builder.push(true)
            },
            ToyBuilder::build,
        )
        .unwrap();

        let mut manual = ToyBuilder::default();
        manual.push(true).unwrap();
        manual.push(true).unwrap();
        assert_eq!(via_closure, manual.build().unwrap());
    }

    #[test]
    fn admission_failure_keeps_its_typed_variant() {
        let error = build_checked(
            ToyBuilder::default(),
            |builder| builder.push(false),
            ToyBuilder::build,
        )
        .unwrap_err();
        assert_eq!(error, CheckedBuildError::Admission(ToyAdmission));
        assert_eq!(error.source().unwrap().to_string(), "toy admission");
    }

    #[test]
    fn verification_failure_keeps_its_typed_variant() {
        let error = build_checked(
            ToyBuilder::default(),
            |_| -> Result<(), ToyAdmission> { Ok(()) },
            ToyBuilder::build,
        )
        .unwrap_err();
        assert_eq!(error, CheckedBuildError::Verification(ToyVerification));
        assert_eq!(error.source().unwrap().to_string(), "toy verification");
    }
}
