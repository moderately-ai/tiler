//! Decoding artifact bytes into a validated, device-free program record.
//!
//! # The three stages, and why they are three types
//!
//! [`DecodedProgram::decode`] takes bytes and returns a fully validated read
//! view, or a typed rejection naming the class of failure.
//! [`DecodedProgram::preflight`] takes a host's stated
//! [`ExecutionEnvironment`], the identity of the program the caller expects, and
//! the ABI facts the caller has bound, discharges every remaining obligation
//! this loader can decide, and returns a [`Preflight`]. [`Preflight::commit`]
//! consumes it and is infallible.
//!
//! Nothing allocates, touches a device, or is irreversible before the commit,
//! and nothing can refuse after it. That is ADR 0051's one-way routing commit
//! expressed as three types rather than as a rule to remember.
//!
//! The validation is [`tiler_artifact`]'s, not this crate's.
//! [`decode_artifact`] proves framing, manifest and section digests, component
//! schemas, canonical order, expression-arena closure, required-feature
//! support, and — last — that the identity re-derived from the decoded content
//! equals the one the manifest carries. A rejection never yields a partially
//! validated view, so holding a [`DecodedProgram`] *is* the evidence that the
//! bytes passed every one of those checks.
//!
//! # Why the rejection is reclassified rather than passed through
//!
//! [`ArtifactCodecFailure`] already classifies the codec's own boundaries, and
//! this module keeps every one of those distinctions by carrying the value
//! whole in [`LoadRejection::Artifact`]. It does not flatten them into strings
//! and it does not add a class the codec already draws. The reclassification
//! exists so that a *host* failure — an incompatible profile, an artifact that
//! is not the one this process expected, an object this build cannot execute —
//! is a different variant from a damaged file, because the two mean different
//! things to do next and collapsing them would make a version skew look like
//! corruption.
//!
//! # How a route is chosen
//!
//! Routing is driven by what the artifact declares rather than by a search over
//! its tables. Variants are tried in declaration order — which is meaning under
//! [`RoutingPolicy::StablePriority`] — and the first whose applicability guard
//! evaluates true against the caller's facts is selected. That variant's entry
//! names the payload descriptor realizing it, and that descriptor names its own
//! object section. Neither association is inferred from a count.
//!
//! **Two refusals this module documented are retracted**, because both described
//! gaps that `expose-the-dispatch-record-on-a-decoded-artifact` closed. It
//! refused more than one packaged variant on the grounds that "a guard is
//! reachable only through a `VerifiedArtifactProgram` that no decode produces",
//! and more than one payload descriptor on the grounds that "the descriptor-to-
//! section map is not published". `DecodedVariant::applicability_guard` and
//! `DecodedArtifact::payload_object` publish precisely those, so both refusals
//! are gone rather than restated, and `LoadRejection` no longer has a class for
//! either.
//!
//! # What is still refused, and why each reason survives
//!
//! - **A variant whose entry count is not one.** An envelope carries each
//!   stage's canonical identity rather than the program's dependency graph, so
//!   declaration order is not execution order and a multi-entry variant cannot
//!   be sequenced from an artifact alone. Unreachable today, because the decoder
//!   already refuses such an envelope through
//!   `tiler.artifact.feature.multi-stage-program`; kept because this loader must
//!   not be correct only by another layer's refusal.
//! - **A variant with deferred feasibility predicates.** Answering one means
//!   querying the provider it names, and this crate holds no provider registry.
//!   Ignoring them would route past a feasibility condition the producer
//!   deliberately left open.
//! - **Every guard false.** An artifact whose own guards exclude the bound facts
//!   has nothing applicable to route to, and taking a variant anyway is how a
//!   plan gets executed on a host it was proven not to fit.
//! - **Any execution policy other than a native image.** Device translation is
//!   by definition not device-free.

mod host;
mod route;

pub use host::{ExecutionEnvironment, TargetCompatibility};
pub use route::{Preflight, RoutedBinding, RoutedDispatch, RoutedLaunch};

use tiler_artifact::program::{
    AbiEvaluationError, AbiFacts, AbiValue, ArtifactCodecFailure, ArtifactExecutionPolicy,
    BackendPayloadDescriptor, CanonicalArtifactProgramIdentity, DecodedArtifact, DecodedEntry,
    DecodedExpr, DecodedInput, DecodedOutput, DecodedVariant, RoutingPolicy, SectionView,
    decode_artifact,
};

use std::error::Error;
use std::fmt;

/// One artifact's bytes, decoded and fully validated by the artifact layer.
///
/// Accessors rather than fields, and deliberately no `From`/`Deref` onto
/// [`DecodedArtifact`]: this crate's job is to add host-relative obligations on
/// top of a decode, and handing out the raw view would let a caller skip them
/// while still appearing to have gone through the runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedProgram {
    decoded: DecodedArtifact,
}

impl DecodedProgram {
    /// Decodes and validates one encoded artifact envelope.
    ///
    /// # Errors
    ///
    /// Returns [`LoadRejection::Artifact`] carrying the codec's own
    /// classification of the first boundary that refused.
    pub fn decode(bytes: &[u8]) -> Result<Self, LoadRejection> {
        decode_artifact(bytes)
            .map(|decoded| Self { decoded })
            .map_err(LoadRejection::Artifact)
    }

    /// Returns the identity re-derived from this artifact's decoded content.
    ///
    /// Never read from the manifest: [`decode_artifact`] derived it from
    /// content and refused when it disagreed with the manifest's copy, so a
    /// forged envelope cannot present a chosen identity here.
    #[must_use]
    pub fn identity(&self) -> CanonicalArtifactProgramIdentity {
        self.decoded.identity()
    }

    /// Returns the governed features this artifact requires of a reader.
    ///
    /// Informational at this point rather than a gate: the codec already
    /// refused any feature this build cannot supply, so a
    /// [`DecodedProgram`] never carries an unsupported one. It is exposed so a
    /// host can log or report what an artifact needed.
    #[must_use]
    pub fn required_features(&self) -> &[String] {
        self.decoded.features()
    }

    /// Returns the policy by which this artifact's variants are chosen among.
    #[must_use]
    pub fn routing_policy(&self) -> RoutingPolicy {
        self.decoded.routing()
    }

    /// Returns the number of packaged plan variants, in routing priority order.
    #[must_use]
    pub fn variant_count(&self) -> usize {
        self.decoded.variant_count()
    }

    /// Returns the named program inputs in semantic interface order.
    ///
    /// The interface a host binds storage *to*: a routed binding addressing
    /// [`BindingTarget::ProgramInput`] names one of these keys, and the extents
    /// bound from these shapes are the free variables of every launch and
    /// accessible-range formula the artifact carries.
    ///
    /// [`BindingTarget::ProgramInput`]: tiler_artifact::program::BindingTarget::ProgramInput
    #[must_use]
    pub fn inputs(&self) -> impl ExactSizeIterator<Item = DecodedInput<'_>> {
        self.decoded.inputs()
    }

    /// Returns the named program outputs in semantic interface order.
    #[must_use]
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = DecodedOutput<'_>> {
        self.decoded.outputs()
    }

    /// Returns the carried backend payload descriptors in canonical order.
    #[must_use]
    pub fn payloads(&self) -> &[BackendPayloadDescriptor] {
        self.decoded.payloads()
    }

    /// Returns every framed section this artifact carries.
    #[must_use]
    pub fn sections(&self) -> impl ExactSizeIterator<Item = SectionView<'_>> {
        self.decoded.sections()
    }

    /// Discharges every obligation this loader can decide, before any commit.
    ///
    /// The order is chosen so that the first refusal is the most useful one.
    /// Identity is checked first: if these are not the bytes of the artifact the
    /// caller expects, no later answer about them is worth reporting. Variant
    /// selection follows, then the target profiles the variant and its payload
    /// separately declare, then how the object reaches an executable state, then
    /// the object itself, and last the geometry and bindings, which are the only
    /// obligations that depend on the caller's facts.
    ///
    /// `expected` is the caller's own artifact identity — the one it obtained by
    /// building this artifact, or recorded when it cached these bytes. This is
    /// the binding-by-identity path a decoded envelope supports: it proves the
    /// loaded bytes *are* that artifact without reconstructing anything, because
    /// [`Self::decode`] already re-derived the identity from content rather than
    /// reading it from the manifest. Its strength is exactly the strength of
    /// whatever recorded it. An identity re-read from these same bytes is a
    /// tautology, and this method cannot tell the difference, so a caller that
    /// passes [`Self::identity`] here has checked nothing.
    ///
    /// `facts` are the ABI facts the caller has bound — input extents, target
    /// properties. They are taken here rather than after the commit because
    /// evaluating a guard, a launch extent, or an accessible byte range can
    /// *fail*, and ADR 0051 forbids a refusal after the routing commit.
    /// Evaluating them in [`RoutedDispatch`] instead would move a failure past
    /// the point where a fallback is still permitted, which is the one thing the
    /// commit exists to prevent.
    ///
    /// # Errors
    ///
    /// Returns the [`LoadRejection`] naming the first obligation that failed.
    /// Nothing has been allocated or committed when it does.
    ///
    /// # Panics
    ///
    /// Panics if the decoded artifact contradicts an invariant
    /// [`decode_artifact`] already proved: an entry naming a payload the
    /// artifact does not declare, an expression whose evaluated value disagrees
    /// with its validated type, or a transport mapping that is not one slot per
    /// binding. Each would be a defect in `tiler-artifact` rather than a caller
    /// error, which is why none is a returned variant a caller would handle.
    pub fn preflight(
        &self,
        environment: &ExecutionEnvironment,
        expected: &CanonicalArtifactProgramIdentity,
        facts: &AbiFacts,
    ) -> Result<Preflight<'_>, LoadRejection> {
        let identity = self.identity();
        if &identity != expected {
            return Err(LoadRejection::ProgramMismatch {
                expected: expected.clone(),
                loaded: identity,
            });
        }

        let variant = self.select_variant(facts)?;

        let classification = environment.classify(variant.target_profile());
        if !classification.is_compatible() {
            return Err(LoadRejection::IncompatibleTarget {
                declaration: TargetDeclaration::Variant,
                classification,
            });
        }

        let entry = accept_entry(variant)?;
        let position = entry.payload();
        let payload = self
            .decoded
            .payloads()
            .get(position)
            .expect("a decode proved every entry names a payload the artifact declares");
        if payload.backend != environment.backend
            || payload.representation != environment.representation
        {
            return Err(LoadRejection::UnexecutablePayload {
                declared_backend: payload.backend.as_str().to_owned(),
                declared_representation: payload.representation.as_str().to_owned(),
                host_backend: environment.backend.as_str().to_owned(),
                host_representation: environment.representation.as_str().to_owned(),
            });
        }

        // The payload's own compatibility contract, classified separately from
        // the variant's. `BackendPayloadDescriptor::compatibility` exists
        // because two variants declaring different profiles may realize their
        // entries through one payload, so deriving either from the other is the
        // inference the artifact layer records that field to forbid.
        let classification = environment.classify(&payload.compatibility);
        if !classification.is_compatible() {
            return Err(LoadRejection::IncompatibleTarget {
                declaration: TargetDeclaration::Payload,
                classification,
            });
        }

        // Exhaustive rather than a wildcard: `ArtifactExecutionPolicy` is
        // deliberately not `#[non_exhaustive]` (ADR 0074 convention 5b), so a
        // policy added to the artifact layer is a build failure here instead of
        // being silently treated as directly loadable.
        match payload.execution_policy {
            ArtifactExecutionPolicy::NativeImage => {}
            policy @ ArtifactExecutionPolicy::RequiresDeviceTranslation => {
                return Err(LoadRejection::UndeliverableExecutionPolicy { policy });
            }
        }

        // One condition asked three ways: a descriptor-only payload carries no
        // object, publishes no entry symbol, and publishes no transport mapping.
        // All three are answered from the descriptor position the entry named.
        let (Some(object), Some(symbol), Some(transports)) = (
            self.decoded.payload_object(position),
            entry.backend_symbol(),
            entry.transport_slots(),
        ) else {
            return Err(LoadRejection::ObjectNotCarried);
        };

        let launch = evaluate_launch(entry, facts)?;
        let bindings = place_bindings(entry, transports, facts)?;

        Ok(Preflight {
            identity,
            payload,
            object,
            entry,
            symbol,
            launch,
            bindings,
        })
    }

    /// Selects the first packaged variant whose applicability guard holds.
    ///
    /// Declaration order *is* the priority order, so the walk stops at the first
    /// guard that evaluates true rather than scoring the survivors. A variant is
    /// never selected for being the only one: cardinality is not a guard, and an
    /// artifact whose every guard is false is refused.
    fn select_variant(&self, facts: &AbiFacts) -> Result<DecodedVariant<'_>, LoadRejection> {
        // Exhaustive rather than a wildcard: `RoutingPolicy` is deliberately not
        // `#[non_exhaustive]` (ADR 0074 convention 5b), so a policy added to the
        // artifact layer is a build failure here instead of silently reusing
        // stable-priority selection.
        match self.decoded.routing() {
            RoutingPolicy::StablePriority => {}
        }
        for variant in self.decoded.variants() {
            let subject = AbiSubject::ApplicabilityGuard {
                variant: variant.routing_rank(),
            };
            if boolean(variant.applicability_guard(), subject, facts)? {
                return Ok(variant);
            }
        }
        Err(LoadRejection::NoApplicableVariant {
            packaged: self.decoded.variant_count(),
        })
    }
}

/// Returns the one entry a selected variant dispatches, or why it has none.
///
/// Separate from [`DecodedProgram::select_variant`] because the two answer
/// different questions. Selection asks which variant *applies*; this asks
/// whether the applicable one is something a device-free loader can carry out,
/// and its two refusals are properties of the selected variant rather than
/// reasons to try the next one. Falling through to a lower-priority variant here
/// would silently substitute a plan the producer ranked below the one whose
/// guard held.
fn accept_entry(variant: DecodedVariant<'_>) -> Result<DecodedEntry<'_>, LoadRejection> {
    let rank = variant.routing_rank();
    let deferred = variant.deferred_predicates().len();
    if deferred > 0 {
        return Err(LoadRejection::UnansweredDeferredPredicates {
            variant: rank,
            deferred,
        });
    }

    let mut entries = variant.entries();
    let declared = entries.len();
    if declared != 1 {
        return Err(LoadRejection::UnroutableEntries {
            variant: rank,
            entries: declared,
        });
    }
    Ok(entries
        .next()
        .expect("an iterator reporting one item yields one item"))
}

/// Evaluates one entry's launch geometry and proves its preconditions hold.
fn evaluate_launch(
    entry: DecodedEntry<'_>,
    facts: &AbiFacts,
) -> Result<RoutedLaunch, LoadRejection> {
    for (index, precondition) in entry.launch_preconditions().enumerate() {
        let subject = AbiSubject::LaunchPrecondition { index };
        if !boolean(precondition, subject, facts)? {
            return Err(LoadRejection::LaunchPrecondition { index });
        }
    }
    Ok(RoutedLaunch {
        grid_threads: unsigned(entry.launch_threads(), AbiSubject::LaunchThreads, facts)?,
        threads_per_workgroup: unsigned(
            entry.threads_per_workgroup(),
            AbiSubject::ThreadsPerWorkgroup,
            facts,
        )?,
        zero_work_skips_dispatch: entry.zero_work_skips_dispatch(),
    })
}

/// Places each ABI binding on its backend transport and sizes it.
///
/// `transports[slot]` is the mapping the carried payload declares, and a decode
/// proved it covers every slot, so the pairing is read rather than assumed.
fn place_bindings<'a>(
    entry: DecodedEntry<'a>,
    transports: &[u32],
    facts: &AbiFacts,
) -> Result<Vec<RoutedBinding<'a>>, LoadRejection> {
    let mut placed = Vec::with_capacity(entry.bindings().len());
    for binding in entry.bindings() {
        let slot = binding.slot();
        placed.push(RoutedBinding {
            binding,
            transport: *transports
                .get(slot)
                .expect("a decode proved one transport slot per ABI binding"),
            accessible_bytes: unsigned(
                binding.accessible_bytes(),
                AbiSubject::AccessibleBytes { slot },
                facts,
            )?,
        });
    }
    Ok(placed)
}

/// Evaluates one expression a decode proved unsigned.
fn unsigned(
    expression: DecodedExpr<'_>,
    subject: AbiSubject,
    facts: &AbiFacts,
) -> Result<u64, LoadRejection> {
    match expression
        .evaluate(facts)
        .map_err(|error| LoadRejection::AbiEvaluation { subject, error })?
    {
        AbiValue::Unsigned(value) => Ok(value),
        AbiValue::Boolean(_) => {
            panic!("a decode proved {subject} has an unsigned type, and it evaluated to a boolean")
        }
    }
}

/// Evaluates one expression a decode proved boolean.
fn boolean(
    expression: DecodedExpr<'_>,
    subject: AbiSubject,
    facts: &AbiFacts,
) -> Result<bool, LoadRejection> {
    match expression
        .evaluate(facts)
        .map_err(|error| LoadRejection::AbiEvaluation { subject, error })?
    {
        AbiValue::Boolean(value) => Ok(value),
        AbiValue::Unsigned(_) => {
            panic!("a decode proved {subject} has a boolean type, and it evaluated to an unsigned")
        }
    }
}

/// Which of an artifact's expressions an evaluation failure was reported for.
///
/// A failure there is about the *facts the caller bound* rather than about the
/// artifact, so a caller has to be able to tell which formula went unanswered:
/// an unbound input extent under a launch count and one under a binding's byte
/// range are the same error class and two different mistakes.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: an expression class added
/// to the dispatch record lands here additively.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AbiSubject {
    /// One plan variant's applicability guard, by routing rank.
    ApplicabilityGuard {
        /// Zero-based routing rank of the variant whose guard was evaluated.
        variant: usize,
    },
    /// The routed entry's total launch thread count.
    LaunchThreads,
    /// The routed entry's per-workgroup thread count.
    ThreadsPerWorkgroup,
    /// One launch-instance precondition, by declaration position.
    LaunchPrecondition {
        /// Zero-based position among the entry's preconditions.
        index: usize,
    },
    /// One binding's minimum accessible byte range, by ABI slot.
    AccessibleBytes {
        /// Zero-based ABI slot of the binding whose range was evaluated.
        slot: usize,
    },
}

impl fmt::Display for AbiSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApplicabilityGuard { variant } => {
                write!(formatter, "variant {variant}'s applicability guard")
            }
            Self::LaunchThreads => formatter.write_str("the launch thread count"),
            Self::ThreadsPerWorkgroup => formatter.write_str("the per-workgroup thread count"),
            Self::LaunchPrecondition { index } => write!(formatter, "launch precondition {index}"),
            Self::AccessibleBytes { slot } => {
                write!(formatter, "the accessible byte range of ABI slot {slot}")
            }
        }
    }
}

/// Which declaration of a target profile a compatibility refusal was about.
///
/// Two are classified and they are separate declarations. Reporting only the
/// classification would leave a host unable to tell a plan *assessed* for
/// another profile from an object *compiled* for one, and those are different
/// things to go and fix.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TargetDeclaration {
    /// The profile the selected plan variant was assessed against.
    Variant,
    /// The profile the carried payload's own bytes were built for.
    Payload,
}

impl fmt::Display for TargetDeclaration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variant => formatter.write_str("the selected variant"),
            Self::Payload => formatter.write_str("the carried payload"),
        }
    }
}

/// Why one artifact was not accepted for execution on this host.
///
/// The classes answer different questions, which is the whole reason there is
/// more than one. Bytes the artifact layer refused, an artifact that is not the
/// one this process expected, a host that cannot honour a declared target
/// profile, an artifact whose own guards exclude this host, and a carried object
/// this build cannot execute are different things to do next; reporting them as
/// one would make a stale cache entry indistinguishable from a corrupt file.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a later obligation lands
/// as a new class rather than by widening an existing one's meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LoadRejection {
    /// The artifact layer refused the bytes, with its own classification.
    ///
    /// Carried whole rather than restated. The codec draws five distinctions —
    /// malformed, integrity, unsupported, invalid, limit — and this crate is
    /// not a better authority on which of them applies.
    Artifact(ArtifactCodecFailure),
    /// The bytes are a valid artifact, and not the one the caller expected.
    ///
    /// The whole substance of binding by identity. Both identities are carried
    /// because a caller that logs only "mismatch" cannot tell a stale cache
    /// entry from a mixed-up path.
    ProgramMismatch {
        /// Identity the caller expected these bytes to have.
        expected: CanonicalArtifactProgramIdentity,
        /// Identity re-derived from the bytes that were actually loaded.
        loaded: CanonicalArtifactProgramIdentity,
    },
    /// Every packaged variant's applicability guard evaluated false.
    ///
    /// The artifact is well formed and nothing it packages applies to the facts
    /// this host bound. Distinct from an incompatible target profile: what
    /// excluded it is the producer's own guard rather than the host's profile.
    NoApplicableVariant {
        /// How many plan variants the artifact packages.
        packaged: usize,
    },
    /// The selected variant defers feasibility predicates this loader cannot
    /// answer.
    ///
    /// Answering one means querying the provider the predicate names, and this
    /// crate holds no provider registry. Refusing is the fail-closed form;
    /// routing past an open feasibility condition is not.
    UnansweredDeferredPredicates {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// How many predicates it defers.
        deferred: usize,
    },
    /// The selected variant does not dispatch as exactly one entry.
    ///
    /// An envelope carries each stage's canonical identity and not the
    /// program's dependency graph, so declaration order is not execution order
    /// and a multi-entry variant cannot be sequenced from an artifact alone.
    UnroutableEntries {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// How many executable entries it declares.
        entries: usize,
    },
    /// The routed entry's payload is not one this host stated it can execute.
    UnexecutablePayload {
        /// Governed backend family key the packaged payload declares.
        declared_backend: String,
        /// Governed executable representation key the packaged payload declares.
        declared_representation: String,
        /// Governed backend family key this host stated.
        host_backend: String,
        /// Governed executable representation key this host stated.
        host_representation: String,
    },
    /// A declared target profile is not this host's.
    ///
    /// Carries which declaration refused and how it relates to the host's own,
    /// so a caller can distinguish an artifact for another target family from
    /// one for this family under a profile descriptor the host does not offer.
    IncompatibleTarget {
        /// Which of the two declarations was classified.
        declaration: TargetDeclaration,
        /// How the declared profile relates to the host's own.
        classification: TargetCompatibility,
    },
    /// The payload needs a delivery step this device-free loader cannot perform.
    UndeliverableExecutionPolicy {
        /// The policy the payload declares.
        policy: ArtifactExecutionPolicy,
    },
    /// The artifact names its payload and does not carry the object bytes.
    ///
    /// A descriptor-only payload is well formed; it just cannot be executed
    /// from this artifact alone, and it publishes no entry symbol or transport
    /// mapping either.
    ObjectNotCarried,
    /// An ABI expression could not be evaluated against the caller's facts.
    ///
    /// Always about the facts rather than the artifact: a decode already proved
    /// every operand's type and that no root escapes its availability phase, so
    /// what remains is an extent or property the caller did not bind, or an
    /// arithmetic boundary the bound values crossed.
    AbiEvaluation {
        /// Which of the artifact's expressions was being evaluated.
        subject: AbiSubject,
        /// The artifact layer's own account of why it could not be evaluated.
        error: AbiEvaluationError,
    },
    /// A launch-instance precondition evaluated false.
    ///
    /// The formula was answerable and its answer forbids this launch. Separate
    /// from [`Self::AbiEvaluation`] because an unanswered precondition is a
    /// caller that bound too little, and a false one is a launch the artifact
    /// itself declares invalid.
    LaunchPrecondition {
        /// Zero-based position of the precondition that did not hold.
        index: usize,
    },
}

impl fmt::Display for LoadRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artifact(failure) => write!(formatter, "runtime.artifact: {failure}"),
            Self::ProgramMismatch { expected, loaded } => write!(
                formatter,
                "runtime.program-mismatch: expected an artifact of {} identity bytes, loaded one \
                 of {}, and they differ",
                expected.as_bytes().len(),
                loaded.as_bytes().len(),
            ),
            Self::NoApplicableVariant { packaged } => write!(
                formatter,
                "runtime.no-applicable-variant: none of the {packaged} packaged variant(s) has an \
                 applicability guard that holds for the bound facts",
            ),
            Self::UnansweredDeferredPredicates { variant, deferred } => write!(
                formatter,
                "runtime.deferred-predicates: variant {variant} defers {deferred} feasibility \
                 predicate(s), and a device-free loader queries no provider",
            ),
            Self::UnroutableEntries { variant, entries } => write!(
                formatter,
                "runtime.unroutable-entries: variant {variant} declares {entries} entries, and an \
                 envelope carries no execution order to sequence them by",
            ),
            Self::UnexecutablePayload {
                declared_backend,
                declared_representation,
                host_backend,
                host_representation,
            } => write!(
                formatter,
                "runtime.unexecutable-payload: the routed entry is realized by a \
                 {declared_backend}/{declared_representation} payload and this host states \
                 {host_backend}/{host_representation}",
            ),
            Self::IncompatibleTarget {
                declaration,
                classification,
            } => write!(
                formatter,
                "runtime.incompatible-target: {declaration} declares a profile this host does not \
                 offer: {classification:?}",
            ),
            Self::UndeliverableExecutionPolicy { policy } => write!(
                formatter,
                "runtime.undeliverable: a device-free loader cannot deliver {policy:?}",
            ),
            Self::ObjectNotCarried => formatter.write_str(
                "runtime.object-absent: the artifact names its payload and carries no object",
            ),
            Self::AbiEvaluation { subject, error } => write!(
                formatter,
                "runtime.abi-evaluation: {subject} could not be evaluated: {error}",
            ),
            Self::LaunchPrecondition { index } => write!(
                formatter,
                "runtime.launch-precondition: precondition {index} does not hold for the bound \
                 facts",
            ),
        }
    }
}

impl Error for LoadRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Artifact(failure) => Some(failure),
            Self::AbiEvaluation { error, .. } => Some(error),
            Self::ProgramMismatch { .. }
            | Self::NoApplicableVariant { .. }
            | Self::UnansweredDeferredPredicates { .. }
            | Self::UnroutableEntries { .. }
            | Self::UnexecutablePayload { .. }
            | Self::IncompatibleTarget { .. }
            | Self::UndeliverableExecutionPolicy { .. }
            | Self::ObjectNotCarried
            | Self::LaunchPrecondition { .. } => None,
        }
    }
}

impl From<ArtifactCodecFailure> for LoadRejection {
    fn from(value: ArtifactCodecFailure) -> Self {
        Self::Artifact(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{AbiSubject, DecodedProgram, LoadRejection, TargetDeclaration};
    use std::error::Error;
    use tiler_artifact::program::ArtifactCodecFailure;

    /// Bytes that are not an artifact at all are refused as malformed.
    ///
    /// The class matters more than the refusal: a host that cannot tell "this
    /// is not a Tiler artifact" from "this artifact is damaged" cannot decide
    /// whether to look for a different file or to re-fetch this one.
    #[test]
    fn foreign_bytes_are_malformed_rather_than_damaged() {
        let rejection = DecodedProgram::decode(b"not a Tiler artifact at all")
            .expect_err("foreign bytes are not an artifact");
        assert!(
            matches!(
                rejection,
                LoadRejection::Artifact(ArtifactCodecFailure::Malformed { .. }),
            ),
            "expected a malformed classification, got {rejection}",
        );
    }

    /// An empty input is refused rather than treated as an empty artifact.
    #[test]
    fn empty_bytes_are_refused() {
        assert!(DecodedProgram::decode(&[]).is_err());
    }

    /// The rejection keeps the codec's own failure reachable as its source.
    ///
    /// Asserted because the alternative — formatting the cause into a string —
    /// is the easy way to write this type and destroys a caller's ability to
    /// match on what actually happened.
    #[test]
    fn a_rejection_preserves_the_codec_failure_it_classifies() {
        let rejection =
            DecodedProgram::decode(b"short").expect_err("five bytes are not an artifact");
        let LoadRejection::Artifact(failure) = &rejection else {
            panic!("bytes that are not an artifact are an artifact-layer rejection: {rejection}");
        };
        assert!(
            rejection.to_string().contains(&failure.to_string()),
            "the display form must not lose the boundary that refused",
        );
        assert!(
            rejection.source().is_some(),
            "the classified codec failure must stay reachable as a source",
        );
    }

    /// Each ABI subject names a distinct expression in its display form.
    ///
    /// The subject is the whole value of the evaluation rejection: the error it
    /// carries is one class for an unbound extent under a launch count and
    /// under a byte range, so a subject that read the same for both would leave
    /// a caller unable to tell which formula went unanswered.
    #[test]
    fn every_abi_subject_names_a_distinct_expression() {
        let rendered: Vec<String> = [
            AbiSubject::ApplicabilityGuard { variant: 0 },
            AbiSubject::LaunchThreads,
            AbiSubject::ThreadsPerWorkgroup,
            AbiSubject::LaunchPrecondition { index: 0 },
            AbiSubject::AccessibleBytes { slot: 0 },
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        for (position, text) in rendered.iter().enumerate() {
            assert!(
                !rendered[..position].contains(text),
                "{text:?} is not distinguishable from an earlier subject",
            );
        }
    }

    /// The two target declarations are distinguishable in a refusal.
    ///
    /// A plan assessed for another profile and an object compiled for one are
    /// different repairs, and `TargetCompatibility` alone cannot separate them:
    /// a descriptor mismatch carries the key both sides agree on and nothing
    /// about which declaration carried it.
    #[test]
    fn a_target_refusal_names_which_declaration_it_is_about() {
        assert_ne!(
            TargetDeclaration::Variant.to_string(),
            TargetDeclaration::Payload.to_string(),
        );
    }
}
