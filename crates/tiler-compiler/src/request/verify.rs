//! Admission: the phase order one compilation request is checked in.
//!
//! Request-scoped properties first — environment pairing, schema, contract
//! representability and admission, target uniqueness, program budgets — then
//! every structurally admitted target resolves its own contract, and only then
//! is the program recognized. The order is the point: honourability is a
//! property of the stated contract and the target's declaration, so asking what
//! this build can *plan* first would attribute a build limitation to a request
//! whose stated meaning the target already cannot deliver.

use super::*;

pub(crate) fn verify_request(
    request: CompilationRequest<'_>,
) -> Result<VerifiedRequest, RequestError> {
    if !carries_program_environment(request.shape_environment, request.program) {
        return Err(RequestError::MismatchedShapeEnvironment);
    }
    if request.capabilities.schema_version != REQUEST_SCHEMA_VERSION {
        return Err(RequestError::UnsupportedRequestVersion);
    }
    // The registry itself is deliberately unconstrained: an externally
    // registered lowering provider is exactly what this boundary admits. What is
    // constrained is that the request pairs the registry with the same scalar
    // authority its capabilities were admitted against, because every resolved
    // provider is driven through — and revalidated under — that snapshot.
    if request.capabilities.lowering.scalar_snapshot()
        != request.capabilities.scalars.snapshot_identity()
    {
        return unsupported("capability", "scalar-authority-pairing");
    }
    if request.target_profiles.is_empty() {
        return Err(RequestError::EmptyTargetSet);
    }
    if request.numerical_contracts.stated().is_empty() {
        return Err(RequestError::UnstatedNumericalContract);
    }
    // Representability is checked before admission and before any target is
    // consulted. Before a target, because it is a property of this build rather
    // than of a profile: a dimension an admitted operation can consume and no
    // scheduled region can record would give two meanings one identity on every
    // target at once, and reporting that as an unhonourable dimension would
    // attribute a build limitation to a declaration that never spoke about it.
    // Before admission, because it is the more specific of two true statements —
    // an unrealizable contract is also unregistered, and "this build cannot
    // realize a permitted signed-zero dimension" names the reason while "this
    // contract is not one this build registers" only names the consequence.
    for contract in request.numerical_contracts.stated() {
        if let Some(cause) = crate::policy::unrepresentable_dimension(contract) {
            return Err(RequestError::UnrepresentableNumericalDimension { cause });
        }
    }
    if request
        .numerical_contracts
        .stated()
        .iter()
        .any(|contract| !contract.is_governed())
    {
        // Names the profile rather than one contract: a caller stating an
        // unadmitted contract has not violated the strict one, it has named a
        // contract this build does not register.
        return unsupported("numerics", "governed-contract-profile");
    }
    let mut target_keys: Vec<_> = request
        .target_profiles
        .iter()
        .map(TargetProfile::profile_key)
        .collect();
    target_keys.sort_unstable();
    if target_keys.windows(2).any(|keys| keys[0] == keys[1]) {
        return Err(RequestError::DuplicateTargetProfile);
    }
    // Budgets before targets, because exceeding one is a property of the
    // submitted program that no target outcome can make admissible. Recognition
    // is deliberately *not* here — see the phase comment below.
    check_program_budgets(request.program, request.budgets)?;
    let dispatch_types = canonical_program_value_types(request.program);
    // The arithmetic every stated contract is measured against, and `None` for a
    // program whose value types this build states no contract vocabulary for.
    //
    // **`ok()` rather than `?`, and the discarded refusal is deliberate.** The
    // recognizer reports the same finding under `dtype-recognized` in its own
    // phase, *after* every target has answered, and hoisting it here would move
    // a build limitation ahead of the target's own dtype-dispatch and
    // honourability refusals — the exact ordering the phase split below exists to
    // prevent. What is hoisted is only the *applicability* narrowing, which needs
    // an arithmetic to compare against and simply does not apply without one.
    let program_arithmetic = recognized_program_arithmetic(request.program).ok();
    // Applicability before targets, for the reason representability is checked
    // before them: a contract stated for another width is not a question this
    // profile — or any profile — can answer, because a contract's arithmetic is
    // part of its identity and a target's honourability rows are keyed by
    // subject. Only the *complete* absence of an applicable entry refuses here;
    // a preference naming this program's width alongside another's resolves
    // against the applicable entries and reports their own causes.
    if let Some(program) = program_arithmetic
        && !request
            .numerical_contracts
            .stated()
            .iter()
            .any(|contract| contract.arithmetic == program)
    {
        return Err(RequestError::NoApplicableNumericalContract {
            program,
            stated: request
                .numerical_contracts
                .stated()
                .iter()
                .map(|contract| (contract.key, contract.arithmetic))
                .collect(),
        });
    }

    // Resolve every structurally admitted target independently. A profile that
    // honours no stated contract is a target-local outcome, not a reason to
    // discard the other ordered slots. Intrinsic profile/authority failures
    // remain outer request errors because no target outcome can make malformed
    // input valid.
    //
    // **This runs before the program is recognized, and the order is the whole
    // point of the phase split.** Honourability is a property of the stated
    // contract and the target's own declaration; it does not depend on which
    // physical strategy this build happens to be able to spell. Recognition
    // answers a different question — what this build can *plan* — so asking it
    // first attributes a build limitation to a request whose stated meaning the
    // target already cannot deliver. That was not hypothetical: while the
    // recognizer refused every non-`f32` program under one `dtype-f32` rule, a
    // profile's measured `bf16` subnormal row could never produce the refusal it
    // exists to produce, and the missing answer read as a missing target fact
    // rather than as a boundary in the wrong order. The rule is gone and the
    // order is what keeps its lesson: a width this build cannot *spell* is still
    // reported after the target has answered for the width it cannot *dispatch*.
    //
    // Each of the three checks below keeps its former relative order, so nothing
    // about which refusal a rejected target reports has moved.
    let target_resolutions = request
        .target_profiles
        .iter()
        .map(|target| {
            let structural = require_compile_profile_dispatch(target, &dispatch_types)
                .and_then(|()| require_elementary_accuracy(request.program, target));
            match structural {
                Ok(()) => match resolve_numerical_contract(
                    &request.numerical_contracts,
                    target,
                    program_arithmetic,
                ) {
                    Ok(numerical_contract) => Ok(Ok(numerical_contract)),
                    Err(error @ RequestError::NoResolvableNumericalContract { .. }) => {
                        Ok(Err(error))
                    }
                    Err(error) => Err(error),
                },
                // Both structural refusals are target-local: another requested
                // profile may dispatch the dtype, or declare the elementary
                // realization, that this one does not.
                Err(
                    error @ (RequestError::DTypeNotDispatchable { .. }
                    | RequestError::UnrealizedElementaryAccuracy { .. }),
                ) => Ok(Err(error)),
                Err(error) => Err(error),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Nothing left to plan: no requested target admitted the request, so there
    // is no program for a strategy to be chosen for. Returning the ordered
    // refusals here is what keeps the more specific statement — *this target
    // declares it cannot honour this dimension* — from being replaced by the
    // recognizer's, which would be true and would answer a question the caller
    // cannot act on while every target still refuses.
    if target_resolutions.iter().all(Result::is_err) {
        return Ok(VerifiedRequest::Refused(
            request
                .target_profiles
                .iter()
                .zip(target_resolutions)
                .map(|(target_profile, resolution)| VerifiedTargetSlot {
                    target_profile: target_profile.clone(),
                    resolution: VerifiedTargetResolution::Rejected(
                        resolution.expect_err("every resolution is an error in this branch"),
                    ),
                })
                .collect(),
        ));
    }

    // The authorities the recognized program's subject is bound to, then
    // recognition.
    //
    // **The realization-law authority now precedes recognition, and it has to.**
    // Recognition asks it whether an occurrence's registered law realizes a
    // region *sequence*, which is what admits a staged family as a program
    // stage, so an authority that does not cohere is not one recognition may
    // consult. The order change is confined to a program that fails both: it
    // used to report the recognizer's rule and now reports the pairing, which is
    // the more specific of the two statements — recognition's answer under an
    // incoherent authority would not be evidence about the program at all. The
    // semantic-snapshot pairing keeps its own position between them, and both
    // report the same rule, so a program failing only one is unmoved.
    let Ok(realization_laws) = FrozenIndexRealizationLawRegistry::from_semantic(
        request.program.semantic_registry().clone(),
        request.capabilities.scalars.clone(),
    ) else {
        return unsupported("capability", "semantic-authority-pairing");
    };
    if request.capabilities.lowering.semantic_snapshot()
        != request.program.semantic_registry().snapshot_identity()
    {
        return unsupported("capability", "semantic-authority-pairing");
    }
    let normalized = select_supported_strategy(request.program, &realization_laws)?;
    let semantic_identity = request.program.semantic_identity().clone();
    let target_slots = request
        .target_profiles
        .iter()
        .zip(target_resolutions)
        .map(|(target, resolution)| VerifiedTargetSlot {
            target_profile: target.clone(),
            resolution: match resolution {
                Ok(numerical_contract) => VerifiedTargetResolution::Resolved {
                    numerical_contract,
                    authority: Box::new(request_subject(
                        &normalized,
                        &semantic_identity,
                        &request.numerical_contracts,
                        numerical_contract,
                        request.budgets,
                        target,
                        VerifiedRequestAuthorities {
                            installed: &request.capabilities,
                            realization_laws: &realization_laws,
                        },
                    )),
                },
                Err(error) => VerifiedTargetResolution::Rejected(error),
            },
        })
        .collect();
    Ok(VerifiedRequest::Planned(Box::new(
        VerifiedCompilationRequest {
            normalized,
            semantic_identity,
            numerical_contracts: request.numerical_contracts,
            budgets: request.budgets,
            target_slots,
            capabilities: request.capabilities,
            realization_laws,
        },
    )))
}

/// Returns every exact value type in canonical byte order, without duplicates.
pub(super) fn canonical_program_value_types(program: &SemanticProgram) -> Vec<ResolvedValueType> {
    let mut resolved_types = program
        .values()
        .map(|value| value.resolved_type().clone())
        .collect::<Vec<_>>();
    resolved_types.sort_by(|left, right| {
        left.canonical_encoding()
            .as_bytes()
            .cmp(right.canonical_encoding().as_bytes())
    });
    resolved_types.dedup();
    resolved_types
}

/// Requires the target to realize every registered elementary accuracy contract
/// this program's operations carry.
///
/// **Asked of the whole program rather than of the recognized members, and the
/// two are the same set here.** Recognition already requires the members it
/// matched to cover the program exactly — that is what `operation-set` refuses —
/// so walking the program's operations reaches every recognized occurrence and
/// nothing else, without threading the recognizer's member vector through a
/// question that is about operations rather than about regions.
///
/// **Asked per target, before any numerical contract is resolved.** The
/// obligation is the registered operation's and is fixed; no contract a caller
/// can state widens or waives it, so resolving one first would order two
/// independent rejections without making either more specific.
pub(super) fn require_elementary_accuracy(
    program: &SemanticProgram,
    target: &TargetProfile,
) -> Result<(), RequestError> {
    let operations: Vec<OpKey> = program
        .operations()
        .map(|operation| operation.key().clone())
        .collect();
    crate::target::accuracy::assess_program_elementary_accuracy(operations.iter(), target).map_err(
        |refusal| RequestError::UnrealizedElementaryAccuracy {
            operation: refusal.operation().clone(),
            target_profile: target.profile_key().clone(),
            reason: refusal.diagnostic_code(),
            undischarged_half: refusal.undischarged_half(),
            undischarged_class: refusal.undischarged_class(),
            candidates: refusal.candidates().to_vec().into_boxed_slice(),
        },
    )
}

/// Requires an exact compile-profile dispatch fact for every program value type.
pub(super) fn require_compile_profile_dispatch(
    target: &TargetProfile,
    resolved_types: &[ResolvedValueType],
) -> Result<(), RequestError> {
    for resolved_type in resolved_types {
        let disposition =
            match target.dtype_dispatchability(resolved_type, AvailabilityPhase::CompileProfile) {
                DTypeDispatchabilityResolution::Dispatchable => continue,
                DTypeDispatchabilityResolution::Unsupported => {
                    DTypeDispatchRefusalDisposition::Unsupported
                }
                DTypeDispatchabilityResolution::Deferred { available_at } => {
                    DTypeDispatchRefusalDisposition::Deferred { available_at }
                }
                DTypeDispatchabilityResolution::Unknown => DTypeDispatchRefusalDisposition::Unknown,
            };
        return Err(RequestError::DTypeNotDispatchable {
            target_profile: target.profile_key().clone(),
            resolved_type: Box::new(resolved_type.clone()),
            disposition,
        });
    }
    Ok(())
}

/// Verifies the program-scoped portion shared by outer admission and semantic
/// candidate readmission.
pub(super) fn verify_program(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<(NormalizedProgram, SemanticIdentity), RequestError> {
    check_program_budgets(program, budgets)?;
    Ok((
        select_supported_strategy(program, laws)?,
        program.semantic_identity().clone(),
    ))
}

/// Checks every deterministic budget this request's program must fit.
///
/// Separated from recognition so outer admission can run it in its own phase:
/// exceeding a budget is a property of the submitted program that no target
/// outcome can excuse, while recognition is a statement about what this build
/// can plan and is asked only of a request some target admitted. Readmission
/// keeps both together, because a rewritten candidate has to clear each again.
pub(super) fn check_program_budgets(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
) -> Result<(), RequestError> {
    check_budget(
        BudgetResource::SemanticValues,
        budgets.semantic_values,
        program.value_count(),
    )?;
    check_budget(
        BudgetResource::SemanticOperations,
        budgets.semantic_operations,
        program.operation_count(),
    )?;
    // The largest shape this profile may assemble, not the smallest it might:
    // the request is admitted before any plan is chosen, so a budget that only
    // admitted the two-region materialized program would let a request through
    // and then refuse the split at assembly, reporting a caller's request as a
    // compiler-output defect.
    //
    // Four dispatches per *declared output*, because a region count belongs to a
    // plan and each ordered named output carries its own producer chain:
    // prologue, partial, final, and the elementwise epilogue that reads the
    // fold's staged result. Four is that chain's measured stage count
    // (`crate::pipeline::tests::the_widest_assembled_plan_is_the_split_reduction_with_its_epilogue`,
    // whose reassociation-forbidding neighbour attributes the fourth stage to
    // the split rather than to the epilogue), and it is the widest chain the
    // recognizer can spell for one output. The outputs' walks partition the
    // program's occurrences — `check_output_cover` proves it — so their chains
    // are disjoint region sets and the assembled plan's stage count is their
    // sum.
    //
    // It was the bare literal `4` while recognition could name only one output,
    // and the sentence that justified spelling it — that the widest plan this
    // profile assembles is that chain whatever the program declares — stopped
    // being true when multi-output admission landed: two independent chains pass
    // every other budget and assemble seven or eight stages against a bound of
    // four, which is exactly the request the boundary admitted and assembly then
    // refused.
    check_budget(
        BudgetResource::Regions,
        budgets.regions,
        program.output_count().saturating_mul(4),
    )?;
    // Derived from the declared arity rather than spelled, because it is an
    // upper bound over every plan the request could reach and the widest of
    // those grows with *both* arities: three program-scoped nodes — the element
    // width, the workgroup width, and the applicability guard — one element
    // count and one byte count per declared input, and per declared output its
    // own pair together with its chain's staged partial tensor's pair.
    //
    // One input and one output reach nine, which is what the split program
    // declared when this was a literal. The bound is deliberately loose: the
    // two-input contraction's own demand is nine by a different route — it
    // declares no partial tensor and one further input — and the widest
    // one-input chain declares seven, because an upper bound over every
    // reachable plan cannot also be each plan's exact count.
    check_budget(
        BudgetResource::HostExpressionNodes,
        budgets.host_expression_nodes,
        program
            .input_count()
            .saturating_mul(2)
            .saturating_add(program.output_count().saturating_mul(4))
            .saturating_add(3),
    )?;
    // The widest buffer count any plan for this request could reach: every
    // declared input, and four per declared output — the prologue's materialized
    // temporary, a split's staged partial tensor, the fold's staged result that
    // an elementwise epilogue reads, and the output itself. A standalone
    // elementwise output binds only the last of the four and a contraction only
    // its output, so this bounds them too, which is what lets it be checked
    // before a strategy has been chosen.
    //
    // The per-output four is measured rather than enumerated from the
    // vocabulary, by
    // `crate::pipeline::tests::the_widest_assembled_plan_binds_four_buffers_per_declared_output`:
    // it was three while the enumeration stopped at the split's partial tensor,
    // which under-reported the epilogue's staged read by one for every output —
    // one declared input reaches five values, not four.
    check_budget(
        BudgetResource::Buffers,
        budgets.buffers,
        program
            .input_count()
            .saturating_add(program.output_count().saturating_mul(4)),
    )?;
    Ok(())
}

/// Resolves a caller's ordered preference against one target's declaration.
///
/// The first stated entry every one of whose dimensions the target honours wins.
/// The order is the caller's; nothing here reorders, scores, or blends the
/// entries, and no entry is admitted on a weakened reading of itself.
///
/// # Errors
///
/// Returns [`RequestError::NoResolvableNumericalContract`] carrying one
/// canonical-first cause per stated entry, in the caller's order, when no entry
/// resolves. A malformed profile is an intrinsic contract violation rather than
/// a resolution outcome and surfaces as
/// [`RequestError::UnsupportedCapability`].
pub(super) fn resolve_numerical_contract(
    preference: &NumericalContractPreference,
    target: &TargetProfile,
    program_arithmetic: Option<ArithmeticType>,
) -> Result<StrictF32NumericalContract, RequestError> {
    let mut rejections = Vec::new();
    for contract in preference.stated() {
        // A contract about another width is skipped rather than rejected,
        // because it was never asked: `verify_request` has already refused a
        // preference in which *every* entry is inapplicable, so reaching here
        // means some applicable entry exists and this one simply is not it.
        // Pushing a rejection would report a profile declining a question no
        // profile was put.
        if program_arithmetic.is_some_and(|program| contract.arithmetic != program) {
            continue;
        }
        let outcome = crate::physical::assess_contract(target, *contract).map_err(|_| {
            RequestError::UnsupportedCapability {
                phase: "numerics",
                rule: "target-profile-malformed",
            }
        })?;
        match outcome {
            crate::target::feasibility::FeasibilityOutcome::Proven(_) => return Ok(*contract),
            crate::target::feasibility::FeasibilityOutcome::Rejected(rejection) => {
                // The representative is the canonical-first unhonourable
                // dimension; a contract-only proposal has no capability
                // requirements, so it is always a numerical cause.
                if let crate::target::feasibility::RejectionCause::Numerical(cause) =
                    rejection.representative()
                {
                    rejections.push(ContractRejection::Unhonourable {
                        contract_key: contract.key,
                        cause,
                    });
                }
            }
            crate::target::feasibility::FeasibilityOutcome::Unknown(unknown) => {
                rejections.extend(unknown.dimensions().first().map(|cause| {
                    ContractRejection::Undeclared {
                        contract_key: contract.key,
                        cause: cause.clone(),
                    }
                }));
            }
            crate::target::feasibility::FeasibilityOutcome::Deferred(deferred) => {
                rejections.extend(deferred.dimensions().first().map(|cause| {
                    ContractRejection::Deferred {
                        contract_key: contract.key,
                        cause: cause.clone(),
                    }
                }));
            }
        }
    }
    Err(RequestError::NoResolvableNumericalContract {
        target_profile: target.profile_key().clone(),
        rejections,
    })
}
