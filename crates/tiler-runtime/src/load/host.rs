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
//! # One explicit backend choice per routing attempt
//!
//! An [`ExecutionEnvironment`] is deliberately one atomic choice, not one row
//! in a fallback set. The user or consumer selects the backend approach before
//! calling `DecodedProgram::preflight` or `DecodedProgram::prepare`, and the
//! loader validates this exact target-profile, backend-family, representation,
//! and dtype declaration. It never retries another family. A caller may inspect
//! a preflight refusal and explicitly begin a different attempt before any
//! routing commit, but that second attempt is application policy rather than a
//! silent loader fallback. Producer stable priority still chooses among
//! compatible variants within the one stated family.
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
//!
//! # Why dtype dispatchability is stated here and not derived from the profile
//!
//! The compile profile already owns this fact — `TargetProfileBuilder::
//! declare_dtype_dispatchability` records it, and its verdict participates in
//! the profile's complete descriptor, so two families disagreeing about one
//! dtype cannot share a descriptor. That is what makes a **separate** statement
//! here look redundant, and the reason it is not is that the two consumer paths
//! that construct an [`ExecutionEnvironment`] both restate the *producer's*
//! declaration as the host's, which makes [`ExecutionEnvironment::classify`]
//! tautological on exactly the routes that matter: `tiler::route`'s
//! `execution_environment` reads the macro-emitted route facts, and the Candle
//! prototype's `declared_route_environment` says of itself that it is
//! "producer-declared equality, NOT host-earned eligibility".
//!
//! So descriptor equality is the barrier for a host that states its own profile
//! honestly, and it is no barrier at all for one that restates the artifact's.
//! A dtype statement keyed by arithmetic type is what a loader can still refuse
//! on, and — unlike the descriptor — it can say *which* dtype and *which*
//! family, which is what an explain record needs. A descriptor mismatch tells a
//! reader to rebuild; a dtype refusal tells them a rebuild will not help.
//!
//! This crate is not a second authority over that fact any more than it is over
//! the target profile: it believes what the host states, and a host that states
//! it wrongly gets a wrong answer.
//!
//! # Why silence refuses
//!
//! A host that declares nothing about a dtype resolves
//! [`DTypeDispatchResolution::Unknown`], and an `Unknown` route is refused
//! exactly as an [`DTypeDispatchResolution::Unsupported`] one is. That is ADR
//! 0043's disposal of `Unknown` applied rather than amended — a predicate with
//! no admissible proof path is unknown, and an unknown candidate cannot enter an
//! executable frontier — and it is what the measured case requires: finding 26
//! of the Apple numerical-behaviour record has the iOS Simulator compiling and
//! linking every `bfloat` module and *then* failing pipeline creation, one phase
//! after ADR 0051's one-way routing commit. A loader that admitted silence would
//! discover that failure where no fallback is permitted.
//!
//! The compile profile's fourth resolution, `Deferred` — an exact declaration
//! that only resolves from a later phase — has no spelling here on purpose. It
//! is precisely the answer that arrives too late to route on, so a host holding
//! one states nothing and is refused, rather than being given a verdict this
//! stage cannot honour.

use std::collections::BTreeMap;

use tiler_artifact::program::{
    ArithmeticType, BackendKey, RepresentationKey, TargetProfileKey, TargetProfileRef,
};

/// What a loading host offers, stated rather than discovered.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExecutionEnvironment {
    /// The declared target profile this host offers, key and exact descriptor.
    pub target_profile: TargetProfileRef,
    /// The governed backend family this host can execute.
    pub backend: BackendKey,
    /// The governed executable representation this host can consume.
    pub representation: RepresentationKey,
    /// Which exact dtypes this host's target family can dispatch.
    ///
    /// A map rather than a list of rows, so one arithmetic type cannot carry two
    /// verdicts: a host that could state both would leave the loader picking one.
    /// An absent key is not a permissive answer — see the module documentation
    /// for why silence refuses.
    pub dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
}

/// What a host states about its target family's ability to dispatch one dtype.
///
/// Two values, mirroring the compile profile's own declaration vocabulary rather
/// than inventing a second one: a host derives this from the profile it offers,
/// and a third spelling here would be a fact nothing produced.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): a verdict
/// added here changes what a host must be able to say about itself, and that
/// must stop each host's build rather than reach a wildcard that guesses.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DTypeDispatch {
    /// The family can dispatch this dtype.
    Dispatchable,
    /// The family is stated to be unable to dispatch it.
    Unsupported,
}

/// How one dtype an artifact requires resolves against a host's statement.
///
/// Three answers rather than a boolean, because a caller acts differently on
/// each even though two of them refuse. `Unsupported` is a measured negative and
/// says a rebuild will not help; `Unknown` says nobody has measured this family
/// for this dtype, and the repair is to measure it. Collapsing them would report
/// an unmeasured family as a refuted one.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: this is a classification a
/// caller consumes to decide what to do next, so a later class must be able to
/// land additively.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DTypeDispatchResolution {
    /// The host states this family dispatches the dtype.
    Dispatchable,
    /// The host states this family does not.
    Unsupported,
    /// The host states nothing about this dtype.
    Unknown,
}

impl DTypeDispatchResolution {
    /// Returns whether this resolution permits a route to be dispatched.
    ///
    /// Only [`Self::Dispatchable`] does. Written as a match rather than a `!=`
    /// so a class added under convention 5a is classified deliberately here
    /// instead of defaulting to permitted — which is the answer that would route
    /// a program on a family nothing said could run it.
    #[must_use]
    pub const fn is_dispatchable(self) -> bool {
        match self {
            Self::Dispatchable => true,
            Self::Unsupported | Self::Unknown => false,
        }
    }
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
                declared: declared.key.clone(),
                host: self.target_profile.key.clone(),
            };
        }
        if declared.descriptor != self.target_profile.descriptor {
            return TargetCompatibility::DescriptorMismatch {
                key: declared.key.clone(),
            };
        }
        TargetCompatibility::Compatible
    }

    /// Resolves what this host states about dispatching one exact dtype.
    ///
    /// Total by construction: every arithmetic type resolves, and the absence of
    /// a declaration is [`DTypeDispatchResolution::Unknown`] rather than an
    /// `Option` a caller could unwrap into permission.
    #[must_use]
    pub fn classify_dtype(&self, arithmetic: ArithmeticType) -> DTypeDispatchResolution {
        match self.dtype_dispatch.get(&arithmetic) {
            Some(DTypeDispatch::Dispatchable) => DTypeDispatchResolution::Dispatchable,
            Some(DTypeDispatch::Unsupported) => DTypeDispatchResolution::Unsupported,
            None => DTypeDispatchResolution::Unknown,
        }
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
    ///
    /// **Labelled draft** under ADR 0075. `declared` and `host` are governed
    /// [`TargetProfileKey`] values, not erased strings.
    ProfileKeyMismatch {
        /// Governed profile key the artifact declares.
        declared: TargetProfileKey,
        /// Governed profile key this host offers.
        host: TargetProfileKey,
    },
    /// The family matches and the exact profile descriptor does not.
    ///
    /// Distinct from [`Self::ProfileKeyMismatch`] because it is the *same*
    /// target family under a descriptor this host does not offer, which is a
    /// rebuild rather than a wrong-artifact.
    ///
    /// **Labelled draft** under ADR 0075. `key` is a governed
    /// [`TargetProfileKey`], not an erased string.
    DescriptorMismatch {
        /// The governed profile key both sides agree on.
        key: TargetProfileKey,
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
    use super::{
        ArithmeticType, DTypeDispatch, DTypeDispatchResolution, ExecutionEnvironment,
        TargetCompatibility,
    };
    use std::collections::BTreeMap;
    use tiler_artifact::program::{
        BackendKey, RepresentationKey, TargetProfileDescriptorDigest, TargetProfileKey,
        TargetProfileRef,
    };

    fn profile_key(key: &str) -> TargetProfileKey {
        TargetProfileKey::new(key).expect("a governed profile key")
    }

    fn profile(key: &str, descriptor: &[u8]) -> TargetProfileRef {
        TargetProfileRef {
            key: profile_key(key),
            descriptor: TargetProfileDescriptorDigest::from_bytes(descriptor)
                .expect("a descriptor identity"),
        }
    }

    fn environment(key: &str, descriptor: &[u8]) -> ExecutionEnvironment {
        ExecutionEnvironment {
            target_profile: profile(key, descriptor),
            backend: BackendKey::new("tiler.metal").expect("a governed backend key"),
            representation: RepresentationKey::new("metallib").expect("a representation key"),
            dtype_dispatch: BTreeMap::new(),
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
                declared: profile_key("tiler.target.apple-m1"),
                host: profile_key("tiler.target.apple-m4"),
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
                key: profile_key("tiler.target.apple-m4"),
            },
        );
        assert!(!classification.is_compatible());
    }

    /// A declared dtype resolves to exactly the verdict the host stated.
    ///
    /// Both verdicts, over both arithmetic types the workspace produces, because
    /// a resolution that read one declaration for another would still answer.
    #[test]
    fn a_declared_dtype_resolves_to_the_stated_verdict() {
        let mut host = environment("tiler.target.apple-m4", b"descriptor-a");
        host.dtype_dispatch
            .insert(ArithmeticType::F32, DTypeDispatch::Dispatchable);
        host.dtype_dispatch
            .insert(ArithmeticType::Bf16, DTypeDispatch::Unsupported);
        assert_eq!(
            host.classify_dtype(ArithmeticType::F32),
            DTypeDispatchResolution::Dispatchable,
        );
        assert_eq!(
            host.classify_dtype(ArithmeticType::Bf16),
            DTypeDispatchResolution::Unsupported,
        );
    }

    /// A dtype the host said nothing about is unknown, and unknown is not
    /// dispatchable.
    ///
    /// The whole of the fail-closed property, asserted on the *class* and on
    /// [`DTypeDispatchResolution::is_dispatchable`] separately: a resolution that
    /// reported `Unknown` and still permitted a route would satisfy the first
    /// assertion alone.
    #[test]
    fn an_undeclared_dtype_is_unknown_and_refuses() {
        let mut host = environment("tiler.target.apple-m4", b"descriptor-a");
        host.dtype_dispatch
            .insert(ArithmeticType::F32, DTypeDispatch::Dispatchable);
        // Every arithmetic type the vocabulary defines except the one declared,
        // rather than a chosen example: a lookup that fell back to "the sole
        // declaration" would pass a single-case test.
        for arithmetic in ArithmeticType::ALL {
            if arithmetic == ArithmeticType::F32 {
                continue;
            }
            assert_eq!(
                host.classify_dtype(arithmetic),
                DTypeDispatchResolution::Unknown,
                "{} was never declared",
                arithmetic.canonical_type_key(),
            );
            assert!(!host.classify_dtype(arithmetic).is_dispatchable());
        }
        assert!(host.classify_dtype(ArithmeticType::F32).is_dispatchable());
    }

    /// A host stating nothing at all dispatches nothing at all.
    ///
    /// The state every consumer that has not yet derived this fact from its
    /// compile profile is in, asserted rather than left implicit: an empty
    /// declaration is a host that routes no program, which is the fail-closed
    /// direction and not a silently permissive default.
    #[test]
    fn a_silent_host_dispatches_nothing() {
        let host = environment("tiler.target.apple-m4", b"descriptor-a");
        for arithmetic in ArithmeticType::ALL {
            assert!(
                !host.classify_dtype(arithmetic).is_dispatchable(),
                "{} must not be dispatchable on a host that declared nothing",
                arithmetic.canonical_type_key(),
            );
        }
    }

    /// The two refusing resolutions are distinguishable from each other.
    ///
    /// The distinction is the substance of carrying a resolution at all: a
    /// measured negative says a rebuild will not help, and an unmeasured family
    /// says go and measure it. A caller that saw one class for both would be
    /// told the wrong repair half the time.
    #[test]
    fn a_refuted_dtype_and_an_unmeasured_one_are_separate_classes() {
        let mut refuted = environment("tiler.target.apple-m4", b"descriptor-a");
        refuted
            .dtype_dispatch
            .insert(ArithmeticType::Bf16, DTypeDispatch::Unsupported);
        let unmeasured = environment("tiler.target.apple-m4", b"descriptor-a");
        assert_ne!(
            refuted.classify_dtype(ArithmeticType::Bf16),
            unmeasured.classify_dtype(ArithmeticType::Bf16),
        );
        assert!(
            !refuted
                .classify_dtype(ArithmeticType::Bf16)
                .is_dispatchable()
        );
        assert!(
            !unmeasured
                .classify_dtype(ArithmeticType::Bf16)
                .is_dispatchable()
        );
    }
}
