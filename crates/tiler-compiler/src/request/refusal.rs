//! Why a request was refused, in the vocabulary a caller acts on.
//!
//! One typed error per distinct finding, and the two constructors every
//! authority in this boundary reports through. The arms are deliberately not
//! collapsed: a target that declares it cannot honour a dimension, a target that
//! declares nothing about it, a dimension this build cannot realize at all, and
//! a program shape no installed capability can plan are four different
//! statements with four different remedies.

use super::*;

/// Why one stated numerical contract could not be resolved on one target.
///
/// The three arms are three different claims and are deliberately not collapsed:
/// a declared refusal, an absent declaration, and a declaration that has not yet
/// become available are not the same thing, and reporting the second or third as
/// a rejection would assert knowledge the profile never supplied.
///
/// A stated contract about *another* arithmetic type produces no arm here at
/// all, and deliberately: it was never asked of the target, because a contract's
/// arithmetic is part of its identity and a target's rows are keyed by subject,
/// so there is no declaration of this profile's that could answer for it.
/// [`RequestError::NoApplicableNumericalContract`] is that refusal, and it is
/// program-scoped rather than target-local for the same reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ContractRejection {
    /// The target declares it cannot honour a required behaviour.
    Unhonourable {
        contract_key: &'static str,
        cause: UnhonouredDimension,
    },
    /// Nothing the profile declares speaks to a required behaviour, so the
    /// dimension is `Unknown` in ADR 0043's sense and fails closed.
    Undeclared {
        contract_key: &'static str,
        cause: UndeclaredDimension,
    },
    /// The declaration exists only from a later availability phase, so it cannot
    /// resolve the contract at the compile profile.
    Deferred {
        contract_key: &'static str,
        cause: DeferredDimension,
    },
}

impl ContractRejection {
    /// The contract whose resolution this rejection explains.
    pub(crate) const fn contract_key(&self) -> &'static str {
        match self {
            Self::Unhonourable { contract_key, .. }
            | Self::Undeclared { contract_key, .. }
            | Self::Deferred { contract_key, .. } => contract_key,
        }
    }

    /// The dimension the resolution failed on.
    pub(crate) fn dimension(&self) -> NumericalDimension {
        match self {
            Self::Unhonourable { cause, .. } => cause.dimension(),
            Self::Undeclared { cause, .. } => cause.dimension(),
            Self::Deferred { cause, .. } => cause.dimension(),
        }
    }

    /// The arithmetic type the resolution failed for.
    ///
    /// Reported beside the dimension because one profile can honour a dimension
    /// in one arithmetic type and refuse it in another — the measured Apple row
    /// preserves subnormals in `f16` and flushes them in `f32` — so a rejection
    /// naming only the dimension would be false about the other type.
    pub(crate) fn arithmetic(&self) -> ArithmeticType {
        match self {
            Self::Unhonourable { cause, .. } => cause.arithmetic(),
            Self::Undeclared { cause, .. } => cause.arithmetic(),
            Self::Deferred { cause, .. } => cause.arithmetic(),
        }
    }

    /// The complete resolved semantic type the resolution failed for.
    pub(crate) fn resolved_type(&self) -> &ResolvedValueType {
        match self {
            Self::Unhonourable { cause, .. } => cause.resolved_type(),
            Self::Undeclared { cause, .. } => cause.resolved_type(),
            Self::Deferred { cause, .. } => cause.resolved_type(),
        }
    }

    /// The behaviour the contract required on that dimension.
    pub(crate) const fn required(&self) -> DimensionBehaviour {
        match self {
            Self::Unhonourable { cause, .. } => cause.required(),
            Self::Undeclared { cause, .. } => cause.required(),
            Self::Deferred { cause, .. } => cause.required(),
        }
    }
}

impl fmt::Display for ContractRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {} in {} requires {}",
            self.contract_key(),
            self.dimension().key(),
            self.arithmetic().canonical_type_key(),
            self.required().key()
        )?;
        match self {
            Self::Unhonourable { cause, .. } => {
                write!(formatter, ", target declares {}", cause.means().label())?;
                if let Some(honoured) = cause.honoured() {
                    write!(formatter, " and honours {}", honoured.key())?;
                }
                write!(formatter, " (profile {})", cause.profile().key())
            }
            Self::Undeclared { .. } => formatter.write_str(", target declares nothing"),
            Self::Deferred { cause, .. } => write!(
                formatter,
                ", target declares it only from a later phase ({:?})",
                cause.phase()
            ),
        }
    }
}

/// Why one exact program dtype cannot be dispatched at compile profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DTypeDispatchRefusalDisposition {
    /// The profile explicitly refuses the exact type.
    Unsupported,
    /// The first exact fact becomes available only at a later phase.
    Deferred { available_at: AvailabilityPhase },
    /// No fact names the exact type at any phase.
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RequestError {
    UnsupportedRequestVersion,
    /// The request carried a shape environment that is not the program's own.
    ///
    /// Two environments over one program is the ambiguity
    /// [`tiler_ir::index::IndexRegionBuilder::new_with_shape_environment`]
    /// exists to prevent. Dropping the program's environment, or attaching a
    /// different one, is this refusal rather than
    /// [`Self::UnsupportedRequestVersion`]: the schema is current and the
    /// authority that is wrong is the environment pairing.
    MismatchedShapeEnvironment,
    EmptyTargetSet,
    DuplicateTargetProfile,
    UnverifiedTargetSelection,
    /// The caller stated no numerical contract at all.
    ///
    /// Distinct from every rejection that names a dimension: there is no default
    /// and no implicit strictest reading, so the diagnostic says the contract is
    /// unstated rather than reporting a dimension the caller never chose.
    UnstatedNumericalContract,
    /// The same exact numerical contract appeared more than once.
    DuplicateNumericalContract,
    /// The preference exceeded the number of distinct public contracts.
    TooManyNumericalContracts {
        actual: usize,
        max: usize,
    },
    /// No contract in the caller's stated order is about this program's
    /// arithmetic type.
    ///
    /// **Program-scoped, and checked before any target is consulted**, for the
    /// reason [`Self::UnrepresentableNumericalDimension`] is: it is a property of
    /// the request rather than of a profile. A contract's arithmetic is part of
    /// its identity (ADR 0076 item 6) and a target's honourability rows are keyed
    /// by subject, so a `bf16` program stated under an `f32` contract is not a
    /// question any profile can answer — the `f32` rows would answer honestly
    /// about a width the program does not use, and the program would compile
    /// under a meaning nobody stated for it.
    ///
    /// Distinct from [`Self::NoResolvableNumericalContract`], which reports that
    /// the target *was* asked and declined. Nothing here proposes a substitute:
    /// only the caller may state what its program means.
    NoApplicableNumericalContract {
        /// The arithmetic every value of the submitted program carries.
        program: ArithmeticType,
        /// Each stated contract's key and the arithmetic it resolves, in the
        /// caller's own order.
        stated: Vec<(&'static str, ArithmeticType)>,
    },
    /// No contract in the caller's stated order resolves on this target.
    ///
    /// Every stated entry's first canonical failure is retained, in the caller's
    /// order, so the diagnostic explains the whole preference rather than only
    /// its last entry. Nothing here proposes a substitute contract: only the
    /// caller may change what its program means.
    NoResolvableNumericalContract {
        target_profile: TargetProfileKey,
        rejections: Vec<ContractRejection>,
    },
    /// One exact program value type cannot be dispatched on this target at the
    /// compile-profile phase.
    DTypeNotDispatchable {
        target_profile: TargetProfileKey,
        resolved_type: Box<ResolvedValueType>,
        disposition: DTypeDispatchRefusalDisposition,
    },
    /// A stated contract resolves a dimension this build cannot realize.
    ///
    /// Distinct from every target rejection above, and deliberately so: those say
    /// *this target* cannot do what the caller asked, and this says *no target
    /// could*, because the scheduled-region IR has nowhere to record which
    /// resolution was chosen and two contracts differing only there would reach
    /// one region. Reporting it as an unhonourable dimension would attribute a
    /// build limitation to a profile that never claimed anything about it.
    UnrepresentableNumericalDimension {
        cause: UnrepresentableDimension,
    },
    /// A deterministic budget refused a demand.
    ///
    /// The carrier of request and planning budget refusals from their four
    /// authorities. Explain-detail capacity is deliberately absent: its hard
    /// build constants are not request fields, and its distinct outer carrier
    /// keeps that request-wide abort out of candidate-local budget retry.
    /// `limit` and `reported` are `u64` because the internal stop records are:
    /// the two search budgets `DeterministicBudgets` declares as `u64` cannot
    /// be narrowed to `u32` without reporting a saturated number as though it
    /// were the declared bound, and a `usize` demand would make the width of a
    /// public refusal a property of the host.
    ///
    /// Whether `reported` is an exact demand, a planning envelope, or a search
    /// lower bound is not uniform across the vocabulary and is read from
    /// [`BudgetResource::refusal`].
    BudgetExceeded {
        resource: BudgetResource,
        limit: u64,
        reported: u64,
    },
    UnsupportedCapability {
        phase: &'static str,
        rule: &'static str,
    },
    /// A strategy or later capability stated over fixed extents met a symbolic one.
    ///
    /// Named by the extent as written, never by a specialized value. A bound
    /// symbol is still this refusal: specializing it into the logical plan is a
    /// physical-planning decision this boundary must not make. Distinct from
    /// [`Self::UnsupportedCapability`]: that variant names a handle, signature,
    /// or other rule that happened to observe the shape, which is the
    /// mis-attribution this refusal exists to close.
    UnsupportedSymbolicExtent {
        phase: &'static str,
        /// The capability that cannot plan over the extent.
        rule: &'static str,
        /// The extent as written. Never a value the environment determines.
        extent: SourcedExtent,
    },
    /// The target declares no realization refining a registered elementary
    /// accuracy contract this program's operations carry.
    ///
    /// Distinct from every dimension rejection above. Those resolve a *generic*
    /// numerical freedom the caller stated; this one is an ADR 0042 accuracy
    /// contract the registered operation itself carries, which no contract a
    /// caller can state weakens or waives. It is a target-local hard rejection,
    /// so a companion profile that does declare a refining realization still
    /// compiles.
    UnrealizedElementaryAccuracy {
        /// The elementary family whose registered contract went unsatisfied.
        operation: OpKey,
        /// The profile that was asked.
        target_profile: TargetProfileKey,
        /// Stable diagnostic code of the refusing reason.
        ///
        /// Carried rather than re-derived so the public failure key and the
        /// refusal that produced it cannot disagree; the three reasons — no
        /// installed realization at all, an installed one that could not be
        /// proved to refine, and a refining one whose evidence cannot discharge —
        /// are different findings and keep different keys.
        reason: &'static str,
        /// The failing half, when `reason` is undischarged evidence.
        undischarged_half: Option<crate::target::accuracy::ElementaryEvidenceHalf>,
        /// The failing evidence class, when `reason` is undischarged evidence.
        undischarged_class: Option<tiler_ir::semantic::accuracy::ConformanceEvidenceClass>,
        /// Declared same-operation candidates in canonical order.
        ///
        /// Empty when nothing was installed. Several unrefined or undischarged
        /// rows appear here in the same order the profile stores them after
        /// canonicalization, so the public refusal cannot depend on insertion
        /// order.
        candidates: Box<[crate::target::accuracy::ElementaryAccuracyCandidate]>,
    },
    ShapeProductOverflow {
        role: &'static str,
    },
}

impl fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRequestVersion => {
                formatter.write_str("compile.request.schema: unsupported request schema")
            }
            Self::MismatchedShapeEnvironment => formatter.write_str(
                "compile.request.shape-environment: request must carry the program's own environment",
            ),
            Self::EmptyTargetSet => formatter
                .write_str("compile.request.targets.empty: at least one target is required"),
            Self::DuplicateTargetProfile => formatter
                .write_str("compile.request.targets.duplicate: target profile keys must be unique"),
            Self::UnverifiedTargetSelection => formatter.write_str(
                "compile.request.targets.selection: target was not verified by the request",
            ),
            Self::UnstatedNumericalContract => formatter.write_str(
                "compile.request.numerics.unstated: a resolved numerical contract is required",
            ),
            Self::DuplicateNumericalContract => formatter.write_str(
                "compile.request.numerics.duplicate: numerical contracts must be distinct",
            ),
            Self::TooManyNumericalContracts { actual, max } => write!(
                formatter,
                "compile.request.numerics.too-many: {actual} contracts exceeds maximum {max}"
            ),
            Self::NoApplicableNumericalContract { program, stated } => {
                write!(
                    formatter,
                    "compile.request.numerics.inapplicable: no stated contract resolves {}",
                    program.canonical_type_key()
                )?;
                for (key, arithmetic) in stated {
                    write!(
                        formatter,
                        "; {key} resolves {}",
                        arithmetic.canonical_type_key()
                    )?;
                }
                Ok(())
            }
            Self::NoResolvableNumericalContract {
                target_profile,
                rejections,
            } => {
                write!(
                    formatter,
                    "compile.request.numerics.unhonourable: target {target_profile} honours no stated contract"
                )?;
                for rejection in rejections {
                    write!(formatter, "; {rejection}")?;
                }
                Ok(())
            }
            Self::DTypeNotDispatchable {
                target_profile,
                resolved_type,
                disposition,
            } => write!(
                formatter,
                "compile.request.dtype.dispatch: target {target_profile} cannot dispatch exact type {:?} at compile profile: {disposition:?}",
                resolved_type.canonical_encoding().as_bytes(),
            ),
            Self::UnrepresentableNumericalDimension { cause } => write!(
                formatter,
                "compile.request.numerics.unrepresentable: {} in {} requires {}, this build realizes only {} and {} can consume it",
                cause.dimension().key(),
                cause.arithmetic().canonical_type_key(),
                cause.required().key(),
                cause.realized().key(),
                cause.consumed_by(),
            ),
            Self::BudgetExceeded {
                resource,
                limit,
                reported,
            } => write!(
                formatter,
                "compile.budget.{}: {reported} exceeds deterministic limit {limit}",
                resource.key()
            ),
            Self::UnsupportedCapability { phase, rule } => {
                write!(
                    formatter,
                    "compile.unsupported.{phase}.{rule}: no installed capability can compile this valid semantic program"
                )
            }
            Self::UnsupportedSymbolicExtent {
                phase,
                rule,
                extent,
            } => write!(
                formatter,
                "compile.{phase}.{rule}: {extent} is a symbolic extent this capability cannot plan over"
            ),
            Self::UnrealizedElementaryAccuracy {
                operation,
                target_profile,
                reason,
                undischarged_half,
                undischarged_class,
                candidates: _,
            } => {
                write!(
                    formatter,
                    "{reason}: target {target_profile} declares no realization that both refines and discharges the registered accuracy contract of {operation}"
                )?;
                match (undischarged_half, undischarged_class) {
                    (Some(half), Some(class)) => write!(
                        formatter,
                        "; {half} evidence class {class} cannot discharge a hard requirement"
                    ),
                    _ => Ok(()),
                }
            }
            Self::ShapeProductOverflow { role } => write!(
                formatter,
                "compile.shape.{role}.element-count: static element count exceeds u64"
            ),
        }
    }
}

impl Error for RequestError {}

pub(super) fn mismatch<T>(rule: &'static str) -> Result<T, RequestError> {
    unsupported("strategy", rule)
}

pub(super) fn unsupported<T>(phase: &'static str, rule: &'static str) -> Result<T, RequestError> {
    Err(RequestError::UnsupportedCapability { phase, rule })
}
