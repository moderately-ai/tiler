//! The verified request, its ordered per-target slots, and the single-target
//! view every later stage compiles from.
//!
//! Admission mints one slot per structurally admitted target, each carrying that
//! target's resolved contract and its own request-subject authority, so a
//! profile that honours nothing is an ordered outcome rather than a reason to
//! discard its companions. `for_target` is the one place a slot becomes a
//! compilable request, and it re-derives the authority before it does.

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedCompilationRequest {
    pub(super) normalized: NormalizedProgram,
    pub(super) semantic_identity: SemanticIdentity,
    pub(super) numerical_contracts: NumericalContractPreference,
    pub(super) budgets: DeterministicBudgets,
    /// Ordered target receipts minted at verification.
    ///
    /// Profile, resolved contract, and authority travel as one slot so no later
    /// stage can recover their association by comparing whole profile values or
    /// by indexing several parallel vectors.
    pub(super) target_slots: Vec<VerifiedTargetSlot>,
    pub(super) capabilities: CompilerCapabilitySnapshot,
    pub(super) realization_laws: FrozenIndexRealizationLawRegistry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedTargetSlot {
    pub(super) target_profile: TargetProfile,
    pub(super) resolution: VerifiedTargetResolution,
}

/// Contract resolution retained for one structurally admitted target.
///
/// A target that cannot honour any stated contract is still a verified member
/// of the request. Keeping that outcome in its ordered slot lets later
/// orchestration report it beside successful companions instead of aborting the
/// batch before those companions are considered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VerifiedTargetResolution {
    Resolved {
        numerical_contract: StrictF32NumericalContract,
        authority: Box<VerifiedRequestSubject>,
    },
    Rejected(RequestError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedTargetRequest {
    pub(super) normalized: NormalizedProgram,
    pub(super) semantic_identity: SemanticIdentity,
    numerical_contracts: NumericalContractPreference,
    numerical_contract: StrictF32NumericalContract,
    pub(super) budgets: DeterministicBudgets,
    pub(super) target_profile: TargetProfile,
    pub(super) capabilities: CompilerCapabilitySnapshot,
    realization_laws: FrozenIndexRealizationLawRegistry,
    authority: VerifiedRequestSubject,
}

impl VerifiedTargetRequest {
    pub(crate) const fn normalized(&self) -> &NormalizedProgram {
        &self.normalized
    }

    /// Returns the recognized output implementing one cover region's members.
    ///
    /// Every per-region authority below this boundary asks this rather than
    /// asking the request for "the" strategy: with several declared outputs
    /// there is no such thing, and a region's members are exactly the fact that
    /// says which output's partition it belongs to.
    pub(crate) fn output_for_region(
        &self,
        members: &[SemanticStage],
    ) -> Option<(usize, &NormalizedOutput)> {
        self.normalized.output_for_region(members)
    }

    /// Returns the recognized output at one declared position.
    ///
    /// # Panics
    ///
    /// Panics when the position names no declared output, which is invalid
    /// compiler output rather than a caller error: every position handed to
    /// this accessor came from [`Self::output_for_region`] resolving a region
    /// this same request recognized.
    pub(crate) fn output_at(&self, position: usize) -> &NormalizedOutput {
        self.normalized
            .output_at(position)
            .expect("a resolved output position names a recognized output")
    }

    /// Returns the one recognized output of a single-output request.
    ///
    /// **No compile-path derivation reads this, and the `output-arity` guard
    /// that used to justify it is gone.** Relaxing that guard surfaced no caller
    /// to convert, which is the fact worth recording: every per-region authority
    /// on the compile path already resolves through
    /// [`crate::physical::spell_region`] and [`Self::output_at`], and the two
    /// whole-program constructors that still call this —
    /// [`crate::physical::build_scheduled_regions`] and
    /// [`crate::physical::build_fused_scheduled_region`] — are retained as the
    /// single definition of each canonical region and are reached only from
    /// tests. This accessor is what lets those, and the fixtures around them,
    /// name a one-output program's shape without repeating a destructuring.
    ///
    /// # Panics
    ///
    /// Panics for a request whose program declares other than one declared
    /// output. That is now a *reachable* state — the boundary admits ordered
    /// multi-output programs — so the panic is the guarantee: a fixture or
    /// constructor that grows a second output fails loudly here rather than
    /// silently asserting about the first.
    pub(crate) fn sole_output(&self) -> &NormalizedOutput {
        let [output] = self.normalized.outputs() else {
            panic!("this derivation is for a request declaring exactly one output");
        };
        output
    }

    /// The sole recognized output's serial-sum shape, for fixtures.
    ///
    /// `#[cfg(test)]`, and the three below with it. Compile-path code resolves
    /// the output a region belongs to through [`Self::output_for_region`]; these
    /// exist so a fixture that built a one-output program can name its shape
    /// without repeating `sole_output()` at every assertion. They carry the
    /// same panic as [`Self::sole_output`], which is what makes a fixture that
    /// grew a second output fail loudly rather than assert about the first.
    #[cfg(test)]
    pub(crate) fn serial_sum(&self) -> &NormalizedSerialSum {
        self.sole_output().serial_sum()
    }

    #[cfg(test)]
    pub(crate) fn pointwise(&self) -> Option<&NormalizedPointwise> {
        self.sole_output().pointwise()
    }

    #[cfg(test)]
    pub(crate) fn contraction(&self) -> Option<&NormalizedContraction> {
        self.sole_output().contraction()
    }

    /// The request subject this target compiles under.
    ///
    /// **A borrow of the stored authority, not a reconstruction.** The subject
    /// is a pure function of fields that are private and never mutated after
    /// `for_target` verified them, so rebuilding it per call reproduced a value
    /// this type already holds — and it was called once per proposal, per
    /// region, per cover.
    ///
    /// [`Self::reconstructs_its_authority`] is the separate operation that
    /// re-derives and compares; the two were one method, so every reader paid
    /// the verifier's cost.
    pub(crate) const fn subject(&self) -> &VerifiedRequestSubject {
        &self.authority
    }

    /// Re-derives the subject from this request's fields and compares it to the
    /// stored authority.
    ///
    /// Deliberately **not** what [`Self::subject`] does. This is the tamper
    /// check, it costs a full reconstruction, and it is named so a caller
    /// choosing it is choosing the cost. A reader that only wants the subject
    /// wants the borrow.
    pub(crate) fn reconstructs_its_authority(&self) -> bool {
        request_subject(
            &self.normalized,
            &self.semantic_identity,
            &self.numerical_contracts,
            self.numerical_contract,
            self.budgets,
            &self.target_profile,
            VerifiedRequestAuthorities {
                installed: &self.capabilities,
                realization_laws: &self.realization_laws,
            },
        ) == self.authority
    }

    /// The one contract this target compiles under, resolved from the caller's
    /// stated preference before any planning began.
    pub(crate) const fn numerical_contract(&self) -> StrictF32NumericalContract {
        self.numerical_contract
    }

    /// The caller's stated preference, in the caller's order.
    ///
    /// It is bound into the request subject, and therefore into every explain
    /// record and receipt, already; this accessor exists so a consumer can *read*
    /// the fallback intent rather than only distinguish two requests by it.
    pub(crate) fn numerical_contracts(&self) -> &NumericalContractPreference {
        &self.numerical_contracts
    }

    pub(crate) const fn budgets(&self) -> DeterministicBudgets {
        self.budgets
    }

    pub(crate) const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }

    pub(crate) const fn capabilities(&self) -> &CompilerCapabilitySnapshot {
        &self.capabilities
    }

    pub(crate) const fn realization_laws(&self) -> &FrozenIndexRealizationLawRegistry {
        &self.realization_laws
    }

    pub(crate) const fn semantic_identity(&self) -> &SemanticIdentity {
        &self.semantic_identity
    }

    /// Rebinds only the profile for downstream tamper-check fixtures.
    #[cfg(test)]
    pub(crate) fn with_target_profile_for_test(mut self, target_profile: TargetProfile) -> Self {
        self.target_profile = target_profile;
        self
    }
}

impl VerifiedCompilationRequest {
    pub(crate) fn target_slots(&self) -> &[VerifiedTargetSlot] {
        &self.target_slots
    }

    /// Returns the verified target indexes used by receipt-mutation fixtures.
    #[cfg(test)]
    pub(crate) fn target_profiles(&self) -> Vec<usize> {
        (0..self.target_slots.len()).collect()
    }

    /// Returns the verified deterministic budgets bound to this request.
    pub(crate) const fn budgets(&self) -> DeterministicBudgets {
        self.budgets
    }

    /// Re-admits one semantic candidate for an already verified target group.
    ///
    /// The outer request has already admitted the nonempty unique target set,
    /// contract vocabulary, capability pairing, and request schema. Candidate
    /// readmission therefore rechecks only the candidate program and remints
    /// target-local authorities for the named resolved slots. Repeating the
    /// outer admission here would let an unrelated target rejection erase this
    /// contract group.
    pub(crate) fn readmit_candidate(
        &self,
        program: &SemanticProgram,
        target_indexes: &[usize],
    ) -> Result<Self, RequestError> {
        // Authorities before recognition, for the reason [`verify_request`]
        // states at the same pair of statements.
        let Ok(realization_laws) = FrozenIndexRealizationLawRegistry::from_semantic(
            program.semantic_registry().clone(),
            self.capabilities.scalars.clone(),
        ) else {
            return unsupported("capability", "semantic-authority-pairing");
        };
        if self.capabilities.lowering.semantic_snapshot()
            != program.semantic_registry().snapshot_identity()
        {
            return unsupported("capability", "semantic-authority-pairing");
        }
        let (normalized, semantic_identity) =
            verify_program(program, self.budgets, &realization_laws)?;
        let mut target_slots = Vec::with_capacity(target_indexes.len());
        for target_index in target_indexes {
            let slot = self
                .target_slots
                .get(*target_index)
                .ok_or(RequestError::UnverifiedTargetSelection)?;
            let VerifiedTargetResolution::Resolved {
                numerical_contract, ..
            } = &slot.resolution
            else {
                return Err(RequestError::UnverifiedTargetSelection);
            };
            // Rechecked rather than inherited from the resolved slot. The
            // obligation is a property of the candidate's operation multiset,
            // and a rewrite that introduced a family this target cannot realize
            // would otherwise inherit an admission granted to a program that did
            // not contain it. Today's algebraic rules preserve the multiset, so
            // this cannot fire — which is exactly why it is a check rather than
            // a comment, and why its failure is invalid compiler output rather
            // than a candidate silently dropped.
            require_elementary_accuracy(program, &slot.target_profile)?;
            let authority = request_subject(
                &normalized,
                &semantic_identity,
                &self.numerical_contracts,
                *numerical_contract,
                self.budgets,
                &slot.target_profile,
                VerifiedRequestAuthorities {
                    installed: &self.capabilities,
                    realization_laws: &realization_laws,
                },
            );
            target_slots.push(VerifiedTargetSlot {
                target_profile: slot.target_profile.clone(),
                resolution: VerifiedTargetResolution::Resolved {
                    numerical_contract: *numerical_contract,
                    authority: Box::new(authority),
                },
            });
        }
        Ok(Self {
            normalized,
            semantic_identity,
            numerical_contracts: self.numerical_contracts.clone(),
            budgets: self.budgets,
            target_slots,
            capabilities: self.capabilities.clone(),
            realization_laws,
        })
    }

    pub(crate) fn for_target(
        &self,
        target_index: usize,
    ) -> Result<VerifiedTargetRequest, RequestError> {
        let Some(slot) = self.target_slots.get(target_index) else {
            return Err(RequestError::UnverifiedTargetSelection);
        };
        let (numerical_contract, authority) = match &slot.resolution {
            VerifiedTargetResolution::Resolved {
                numerical_contract,
                authority,
            } => (numerical_contract, authority),
            VerifiedTargetResolution::Rejected(error) => return Err(error.clone()),
        };
        let current_authority = request_subject(
            &self.normalized,
            &self.semantic_identity,
            &self.numerical_contracts,
            *numerical_contract,
            self.budgets,
            &slot.target_profile,
            VerifiedRequestAuthorities {
                installed: &self.capabilities,
                realization_laws: &self.realization_laws,
            },
        );
        if !numerical_contract.is_governed() || authority.as_ref() != &current_authority {
            return Err(RequestError::UnverifiedTargetSelection);
        }
        Ok(VerifiedTargetRequest {
            normalized: self.normalized.clone(),
            semantic_identity: self.semantic_identity.clone(),
            numerical_contracts: self.numerical_contracts.clone(),
            numerical_contract: *numerical_contract,
            budgets: self.budgets,
            target_profile: slot.target_profile.clone(),
            capabilities: self.capabilities.clone(),
            realization_laws: self.realization_laws.clone(),
            authority: current_authority,
        })
    }
}

impl VerifiedTargetSlot {
    pub(crate) const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }

    pub(crate) const fn resolution(&self) -> &VerifiedTargetResolution {
        &self.resolution
    }
}

/// The outcome of admitting one compilation request.
///
/// Two variants because a request can be completely refused *before* a strategy
/// is chosen, and the alternative shapes are both worse: an optional recognized
/// program inside [`VerifiedCompilationRequest`] would make every later stage
/// carry a case that cannot occur once a target resolved, and forcing
/// recognition to run anyway would report a recognizer limitation for a request
/// no target admitted.
pub(crate) enum VerifiedRequest {
    /// At least one target admitted the request, so the program was recognized.
    Planned(Box<VerifiedCompilationRequest>),
    /// Every requested target refused, in the caller's order.
    Refused(Vec<VerifiedTargetSlot>),
}

/// Admits a request whose fixture profile is expected to resolve.
///
/// The crate's own fixtures state a contract their governed profile honours, so
/// the planned outcome is the one under test at every one of these call sites
/// and an unexpected complete refusal should fail loudly rather than be pattern
/// matched away. Tests that assert a *refused* request call [`verify_request`]
/// directly and match the variant.
#[cfg(test)]
pub(crate) fn verify_planned_request(
    request: CompilationRequest<'_>,
) -> Result<VerifiedCompilationRequest, RequestError> {
    verify_request(request).map(|verified| {
        verified
            .planned()
            .expect("the fixture profile admits the stated contract")
    })
}

impl VerifiedRequest {
    /// The ordered target slots, whichever way the request was admitted.
    pub(crate) fn target_slots(&self) -> &[VerifiedTargetSlot] {
        match self {
            Self::Planned(request) => request.target_slots(),
            Self::Refused(slots) => slots,
        }
    }

    /// The planned request, or `None` when every target refused.
    pub(crate) fn planned(self) -> Option<VerifiedCompilationRequest> {
        match self {
            Self::Planned(request) => Some(*request),
            Self::Refused(_) => None,
        }
    }
}
