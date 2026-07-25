//! What a loading host states about itself, and how a declared profile is
//! classified against it.
//!
//! # Why the host states its environment rather than the loader discovering it
//!
//! Discovery needs a device. The whole value of this crate is that a load is
//! decidable without one, so the host supplies the three facts a load depends
//! on — which target profile it offers, which backend family it can execute,
//! and which executable representation it can consume — and the loader treats
//! them as given. A host that states them wrongly gets a wrong answer, and that
//! is the correct division: the loader is not the authority on what a machine
//! is.
//!
//! # Why compatibility is a classification and not a boolean
//!
//! ADR 0043 requires a declared target profile to carry both a governed key and
//! an exact descriptor identity, because a profile key alone is not evidence
//! that a variant is legal on a device advertising the same key under a
//! different descriptor. The two ways of failing that check mean different
//! things: a different key is an artifact built for another target family, and
//! the host should look for a different artifact. The same key with a different
//! descriptor is an artifact built for *this* family against a profile
//! revision this host does not offer, and the host should rebuild or re-fetch.
//! Reporting both as `false` would erase the difference at exactly the moment a
//! caller needs it.

use tiler_artifact::program::{BackendKey, RepresentationKey, TargetProfileRef};

/// What a loading host offers, stated rather than discovered.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExecutionEnvironment {
    /// The declared target profile this host offers, key and exact descriptor.
    pub target_profile: TargetProfileRef,
    /// The governed backend family this host can execute.
    pub backend: BackendKey,
    /// The governed executable representation this host can consume.
    pub representation: RepresentationKey,
}

impl ExecutionEnvironment {
    /// Classifies one declared target profile against this host's own.
    ///
    /// Total by construction: the key is compared first because it decides
    /// which of the two remaining answers is meaningful, and equality of both
    /// halves is the only [`TargetCompatibility::Compatible`] result. There is
    /// no partial or best-effort class, because ADR 0043 makes the descriptor a
    /// feasibility input rather than a hint.
    #[must_use]
    pub fn classify(&self, declared: &TargetProfileRef) -> TargetCompatibility {
        if declared.key != self.target_profile.key {
            return TargetCompatibility::ProfileKeyMismatch {
                declared: declared.key.as_str().to_owned(),
                host: self.target_profile.key.as_str().to_owned(),
            };
        }
        if declared.descriptor != self.target_profile.descriptor {
            return TargetCompatibility::DescriptorMismatch {
                key: declared.key.as_str().to_owned(),
            };
        }
        TargetCompatibility::Compatible
    }
}

/// How one artifact's declared target profile relates to a host's own.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a. This is a classification a
/// caller consumes to decide what to do next, not a recognizer a reader
/// implements, so a later class — a profile compatible under a stated widening
/// rule, say — must be able to land additively rather than breaking every
/// match.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TargetCompatibility {
    /// The declared key and exact descriptor are both the host's own.
    Compatible,
    /// The artifact was built for a different target family entirely.
    ProfileKeyMismatch {
        /// Governed profile key the artifact declares.
        declared: String,
        /// Governed profile key this host offers.
        host: String,
    },
    /// The family matches and the exact profile descriptor does not.
    ///
    /// Distinct from [`Self::ProfileKeyMismatch`] because it is the *same*
    /// target family under a descriptor this host does not offer, which is a
    /// rebuild rather than a wrong-artifact.
    DescriptorMismatch {
        /// The governed profile key both sides agree on.
        key: String,
    },
}

impl TargetCompatibility {
    /// Returns whether this classification permits execution on the host.
    #[must_use]
    pub const fn is_compatible(&self) -> bool {
        matches!(self, Self::Compatible)
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionEnvironment, TargetCompatibility};
    use tiler_artifact::program::{
        BackendKey, RepresentationKey, TargetProfileDescriptorDigest, TargetProfileKey,
        TargetProfileRef,
    };

    fn profile(key: &str, descriptor: &[u8]) -> TargetProfileRef {
        TargetProfileRef {
            key: TargetProfileKey::new(key).expect("a governed profile key"),
            descriptor: TargetProfileDescriptorDigest::from_bytes(descriptor)
                .expect("a descriptor identity"),
        }
    }

    fn environment(key: &str, descriptor: &[u8]) -> ExecutionEnvironment {
        ExecutionEnvironment {
            target_profile: profile(key, descriptor),
            backend: BackendKey::new("tiler.metal").expect("a governed backend key"),
            representation: RepresentationKey::new("metallib").expect("a representation key"),
        }
    }

    /// Both halves equal is the only compatible answer.
    #[test]
    fn an_identical_profile_is_compatible() {
        let host = environment("tiler.target.apple-m4", b"descriptor-a");
        assert_eq!(
            host.classify(&profile("tiler.target.apple-m4", b"descriptor-a")),
            TargetCompatibility::Compatible,
        );
    }

    /// A different family is named as such rather than as a descriptor problem.
    #[test]
    fn a_different_family_is_a_key_mismatch() {
        let host = environment("tiler.target.apple-m4", b"descriptor-a");
        assert_eq!(
            host.classify(&profile("tiler.target.apple-m1", b"descriptor-a")),
            TargetCompatibility::ProfileKeyMismatch {
                declared: "tiler.target.apple-m1".to_owned(),
                host: "tiler.target.apple-m4".to_owned(),
            },
        );
    }

    /// The same key under a different descriptor is refused, and separately.
    ///
    /// This is the case ADR 0043 exists for: a key alone is not evidence, so an
    /// artifact naming this host's family against a descriptor it does not
    /// offer must not load. Asserting the *class* rather than a boolean is what
    /// keeps a caller able to tell a rebuild from a wrong artifact.
    #[test]
    fn the_same_key_under_another_descriptor_is_not_compatible() {
        let host = environment("tiler.target.apple-m4", b"descriptor-a");
        let classification = host.classify(&profile("tiler.target.apple-m4", b"descriptor-b"));
        assert_eq!(
            classification,
            TargetCompatibility::DescriptorMismatch {
                key: "tiler.target.apple-m4".to_owned(),
            },
        );
        assert!(!classification.is_compatible());
    }
}
