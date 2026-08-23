use super::super::{
    ApproximationEnvelope, ArithmeticType, CompilationRequest, ContractRejection,
    DimensionBehaviour, ExceptionalValueAssumption, ExceptionalValueDimensionKind,
    F32NumericalContractKey, IncoherentContract, MaterializationRounding,
    NumericalContractPreference, NumericalDimension, NumericalPermission, RequestError,
    StrictF32NumericalContract, SubnormalMode, TargetProfile, ValueDomainProvenance,
    VerifiedTargetResolution, canonical_contract_key, coherence, contract_key_arithmetic,
    contract_key_element_bytes, is_f32_contract_key, resolve_numerical_contract,
    verify_planned_request,
};
use super::support::program;
use tiler_ir::schedule::FlushedZeroSign;

/// Every resolution of every governed dimension, in canonical order.
///
/// The population is counted rather than described: an enumeration that
/// silently lost a resolution would make every claim below pass over a
/// smaller space than it names, which is the failure mode a uniform pass
/// hides. Each row's length is asserted where it is consumed.
fn statable_contracts() -> Vec<StrictF32NumericalContract> {
    let subnormals = [
        SubnormalMode::Preserve,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        },
    ];
    let permissions = [
        NumericalPermission::Forbidden,
        NumericalPermission::Permitted,
    ];
    let envelopes = [
        ApproximationEnvelope::Forbidden,
        ApproximationEnvelope::BackendElementary,
    ];
    let assumptions = [
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
        },
    ];
    let roundings = [MaterializationRounding::NearestTiesToEven];
    let mut contracts = Vec::new();
    for input in subnormals {
        for result in subnormals {
            for contraction in permissions {
                for reassociation in permissions {
                    for permutation in permissions {
                        for signed_zero in permissions {
                            for reciprocal_transform in permissions {
                                for approximate_intrinsics in envelopes {
                                    for nan_assumptions in assumptions {
                                        for infinity_assumptions in assumptions {
                                            for materialization_rounding in roundings {
                                                contracts.push(
                                                    StrictF32NumericalContract {
                                                        input_subnormals: input,
                                                        result_subnormals: result,
                                                        contraction,
                                                        reassociation,
                                                        permutation,
                                                        signed_zero,
                                                        reciprocal_transform,
                                                        approximate_intrinsics,
                                                        nan_assumptions,
                                                        infinity_assumptions,
                                                        materialization_rounding,
                                                        ..StrictF32NumericalContract::governed()
                                                    }
                                                    .keyed(),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    contracts
}

/// The size of the statable space, spelled as its factors.
///
/// Written as the product rather than as `2304` so a widened behaviour space
/// changes the expected count at the factor that moved, and a reader can
/// check the arithmetic against the vocabulary instead of trusting a
/// literal. The factors, in canonical dimension order: three subnormal
/// resolutions twice, two transform permissions five times, two
/// approximation envelopes, and two caller-statable exceptional-value
/// assumptions twice. Compiler-proven and runtime-validated provenance are
/// derived evidence, not caller-statements, and therefore are not keys in
/// this population.
/// Materialization rounding contributes no factor because it has exactly one
/// resolution, so its absence here is the note rather than a `* 1` term.
const STATABLE_CONTRACTS: usize = 3 * 3 * 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2;

/// The canonical key separates every statable contract from every other.
///
/// **Exhaustive finite evidence, not a sample.** The key is the contract's
/// standing identity: [`tiler_ir::index::NumericalContractIdentity`], the fusion
/// legality content identity, and the scheduled region's `profile_key` each
/// carry it *alone*, with no dimension beside it, so two contracts sharing a
/// key would give two stated meanings one artifact and one cache entry. The
/// space is finite and small enough to walk, so it is walked.
#[test]
fn the_canonical_key_is_injective_over_the_statable_space() {
    let contracts = statable_contracts();
    assert_eq!(
        contracts.len(),
        STATABLE_CONTRACTS,
        "the enumeration does not cover the space it names",
    );
    let mut keys: Vec<&str> = contracts.iter().map(|contract| contract.key).collect();
    let mut lengths: Vec<usize> = keys.iter().map(|key| key.len()).collect();
    lengths.sort_unstable();
    lengths.dedup();
    assert_eq!(
        lengths,
        [98, 100, 102],
        "the statable grammar reached an unexpected rendered length",
    );
    for contract in &contracts {
        let parsed = F32NumericalContractKey::try_from_str(contract.key)
            .expect("every statable compiler key is admitted by IR");
        assert_eq!(
            canonical_contract_key(contract).unwrap(),
            parsed.as_str(),
            "compiler and IR canonical encoders disagree"
        );
    }
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(
        keys.len(),
        STATABLE_CONTRACTS,
        "two statable contracts share one key",
    );
}

/// Every minted key is spelled in the governed key alphabet.
///
/// A key is compared byte for byte against one minted by a build that never
/// met this one and is printed in rejections a reader copies back out, so the
/// alphabet is part of what a key is: ASCII lowercase, digits, and `.`, with
/// no case, whitespace, or control byte. It is also carried into an explain
/// `SubjectKey`, which refuses anything longer than 255 bytes.
#[test]
fn every_minted_key_is_spelled_in_the_governed_alphabet() {
    let contracts = statable_contracts();
    assert_eq!(contracts.len(), STATABLE_CONTRACTS);
    for contract in contracts {
        let key = contract.key;
        assert!(
            crate::explain::SubjectKey::new(key).is_ok(),
            "{key} is not admissible as an explain subject",
        );
        assert!(key.len() <= 255, "{key} exceeds the explain key bound");
        assert!(
            key.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.'),
            "{key} leaves the governed key alphabet",
        );
        assert!(
            is_f32_contract_key(key),
            "{key} is not recognized as an f32 contract key",
        );
    }
}

/// The domain prefix test says no, and says it for the right reason.
///
/// Driven against cases that must fail, because a predicate that only ever
/// sees keys it accepts is indistinguishable from one that returns `true`.
#[test]
fn the_contract_key_domain_test_refuses_a_key_from_another_domain() {
    assert!(is_f32_contract_key(
        StrictF32NumericalContract::governed().key
    ));
    for refused in [
        "",
        tiler_ir::schedule::F32_NUMERICAL_CONTRACT_KEY_DOMAIN,
        "tiler.contract.f32.v20.0011",
        "tiler.contract.f16.v2.0011",
        "tiler.strict-f32.v1",
        crate::policy::UNKEYED_CONTRACT,
    ] {
        assert!(!is_f32_contract_key(refused), "{refused} was admitted");
    }
}

/// A key names its own width, and its width names its own element size.
///
/// **The `f32` answer is the pair's control.** Reporting two bytes for a BF16
/// key means nothing unless the binary32 key still reports four, because a
/// derivation that had simply stopped resolving would answer `None` for both
/// and pass a one-sided assertion by failing to be asked. And the refused
/// cases are what stop the width from being read off a textual prefix: each
/// is rejected by the IR-owned parse rather than admitted at some default.
#[test]
fn a_governed_contract_key_derives_its_own_width_and_element_size() {
    let f32_key = StrictF32NumericalContract::governed().key;
    let bf16_key = crate::session::NumericalContract::STRICT_BF16.key();
    assert_ne!(f32_key, bf16_key);

    assert_eq!(contract_key_arithmetic(f32_key), Some(ArithmeticType::F32));
    assert_eq!(
        contract_key_arithmetic(bf16_key),
        Some(ArithmeticType::Bf16)
    );
    assert_eq!(contract_key_element_bytes(f32_key), Some(4));
    assert_eq!(contract_key_element_bytes(bf16_key), Some(2));

    // A key under no governed domain answers `None` on both, which is what
    // makes an unregistered width report `Unknown` rather than continue at a
    // neighbour's size.
    for refused in [
        "",
        tiler_ir::schedule::F32_NUMERICAL_CONTRACT_KEY_DOMAIN,
        tiler_ir::schedule::BF16_NUMERICAL_CONTRACT_KEY_DOMAIN,
        "tiler.contract.f16.v2.0011",
        "tiler.contract.bf16.v10.0011",
        crate::policy::UNKEYED_CONTRACT,
    ] {
        assert_eq!(
            contract_key_arithmetic(refused),
            None,
            "{refused} was admitted"
        );
        assert_eq!(
            contract_key_element_bytes(refused),
            None,
            "{refused} was sized"
        );
    }
}

/// Omission resolves strict on every dimension, so it can never widen.
///
/// Checked against the *canonical dimension walk* rather than field by
/// field, so a dimension added to the vocabulary is covered by this claim the
/// moment it exists rather than when someone remembers to add a line.
#[test]
fn an_unstated_dimension_resolves_strict() {
    let strict = StrictF32NumericalContract::governed();
    for dimension in crate::target::honourability::CANONICAL_DIMENSIONS {
        let behaviour = strict.behaviour(dimension);
        let is_strict = match behaviour {
            DimensionBehaviour::Subnormals(mode) => mode == SubnormalMode::Preserve,
            DimensionBehaviour::Transform(permission) => {
                permission == NumericalPermission::Forbidden
            }
            DimensionBehaviour::Approximation(envelope) => {
                envelope == ApproximationEnvelope::Forbidden
            }
            DimensionBehaviour::ExceptionalValue(assumption) => {
                assumption == ExceptionalValueAssumption::MakeNoAssumption
            }
            DimensionBehaviour::Rounding(rounding) => {
                rounding == MaterializationRounding::NearestTiesToEven
            }
        };
        assert!(
            is_strict,
            "{} does not resolve strict when unstated",
            dimension.key()
        );
    }
}

/// A caller-stated absence on evidence it is not the author of is refused.
///
/// Both dimensions and both refused provenance classes, and the accepted
/// class beside them, so the check is shown saying yes and no rather than
/// only yes.
#[test]
fn an_absence_on_unfounded_provenance_is_incoherent() {
    for (dimension, apply) in [
        (
            ExceptionalValueDimensionKind::Nan,
            (|contract: &mut StrictF32NumericalContract, assumption: ExceptionalValueAssumption| {
                contract.nan_assumptions = assumption;
            }) as fn(&mut StrictF32NumericalContract, ExceptionalValueAssumption),
        ),
        (
            ExceptionalValueDimensionKind::Infinity,
            |contract, assumption| {
                contract.infinity_assumptions = assumption;
            },
        ),
    ] {
        for provenance in [
            ValueDomainProvenance::CompilerProven,
            ValueDomainProvenance::RuntimeValidated,
        ] {
            let mut contract = StrictF32NumericalContract::governed();
            apply(
                &mut contract,
                ExceptionalValueAssumption::AssumeAbsent { provenance },
            );
            assert_eq!(
                coherence(&contract),
                Err(IncoherentContract::UnfoundedValueDomainProvenance {
                    dimension,
                    provenance,
                }),
            );
            assert!(!contract.keyed().is_governed());
        }
        let mut declared = StrictF32NumericalContract::governed();
        apply(
            &mut declared,
            ExceptionalValueAssumption::AssumeAbsent {
                provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
            },
        );
        assert_eq!(coherence(&declared), Ok(()));
    }
}

/// The eliminated combinations are coherent, and each is named.
///
/// The enumeration on [`IncoherentContract`] is only refutable if the
/// combinations it *rejected as candidates* are driven: a later change that
/// quietly started refusing one of these would be narrowing what a caller may
/// state without anyone deciding to.
#[test]
fn the_eliminated_combinations_are_coherent() {
    let flush_preserving = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    };
    let flush_positive = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::AlwaysPositive,
    };
    let declared = ExceptionalValueAssumption::AssumeAbsent {
        provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
    };
    let cases: [(&str, StrictF32NumericalContract); 6] = [
        (
            "assumed-absent NaNs beside a canonical arithmetic NaN pattern",
            StrictF32NumericalContract {
                nan_assumptions: declared,
                ..StrictF32NumericalContract::governed()
            },
        ),
        (
            "one exceptional value assumed absent and the other not",
            StrictF32NumericalContract {
                infinity_assumptions: declared,
                ..StrictF32NumericalContract::governed()
            },
        ),
        (
            "permitted signed-zero elimination beside a sign-preserving flush",
            StrictF32NumericalContract {
                input_subnormals: flush_preserving,
                signed_zero: NumericalPermission::Permitted,
                ..StrictF32NumericalContract::governed()
            },
        ),
        (
            "forbidden signed-zero elimination beside an always-positive flush",
            StrictF32NumericalContract {
                result_subnormals: flush_positive,
                ..StrictF32NumericalContract::governed()
            },
        ),
        (
            "permitted contraction with forbidden reassociation",
            StrictF32NumericalContract {
                contraction: NumericalPermission::Permitted,
                ..StrictF32NumericalContract::governed()
            },
        ),
        (
            "permitted permutation with forbidden reassociation",
            StrictF32NumericalContract {
                permutation: NumericalPermission::Permitted,
                ..StrictF32NumericalContract::governed()
            },
        ),
    ];
    for (name, contract) in cases {
        assert_eq!(coherence(&contract), Ok(()), "{name} was refused");
    }
}

/// A key that does not describe the vector beside it is not admitted.
///
/// The direction that matters: a contract carrying a name from before a
/// dimension moved would otherwise reach a plan under a key that describes a
/// different meaning.
#[test]
fn a_contract_whose_key_does_not_describe_it_is_refused() {
    let strict = StrictF32NumericalContract::governed();
    assert!(strict.is_governed());
    let mutated = StrictF32NumericalContract {
        reassociation: NumericalPermission::Permitted,
        ..strict
    };
    assert!(
        !mutated.is_governed(),
        "a widened contract kept the strict key and was admitted",
    );
    assert!(mutated.keyed().is_governed());
    let unkeyed = crate::policy::strict_contract(
        ArithmeticType::F32,
        tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS,
    );
    assert!(!unkeyed.is_governed(), "an unkeyed contract was admitted");
}

/// A single-entry list and a bare contract behave identically.
///
/// The list is an additive generalization, not a second mechanism, so the
/// two spellings must produce the same verified request — including the same
/// request subject, which is what binds the caller's stated intent into
/// every explain record and receipt.
#[test]
fn a_single_entry_preference_and_a_bare_contract_are_the_same_request() {
    let program = program();
    let bare = verify_planned_request(CompilationRequest::governed_under(
        &program,
        StrictF32NumericalContract::governed(),
    ))
    .unwrap();
    let listed = verify_planned_request(CompilationRequest::governed_preferring(
        &program,
        NumericalContractPreference::ordered(vec![StrictF32NumericalContract::governed()]).unwrap(),
    ))
    .unwrap();
    assert_eq!(bare, listed);
    assert_eq!(
        bare.for_target(0).unwrap().subject(),
        listed.for_target(0).unwrap().subject(),
    );
}

/// Resolution follows the caller's stated order, never a ranking of its own.
///
/// The governed baseline honours both registered contracts, so whichever
/// entry the caller put first is the one that wins. That is the whole
/// property: nothing here prefers the strict entry because it is stricter or
/// the flushing entry because it is cheaper, because a cost may never rank
/// contracts against each other.
#[test]
fn a_preference_list_resolves_by_the_callers_order_and_never_by_rank() {
    let program = program();
    for (first, second) in [
        (
            StrictF32NumericalContract::governed(),
            StrictF32NumericalContract::governed_flush_to_zero(),
        ),
        (
            StrictF32NumericalContract::governed_flush_to_zero(),
            StrictF32NumericalContract::governed(),
        ),
    ] {
        let verified = verify_planned_request(CompilationRequest::governed_preferring(
            &program,
            NumericalContractPreference::ordered(vec![first, second]).unwrap(),
        ))
        .unwrap();
        assert!(matches!(
            verified.target_slots[0].resolution,
            VerifiedTargetResolution::Resolved {
                numerical_contract,
                ..
            } if numerical_contract == first
        ));
        let target = verified.for_target(0).unwrap();
        assert_eq!(target.numerical_contract(), first);
        // The whole stated list is retained, not only the winner: the
        // caller's fallback intent is what the list exists to record.
        assert_eq!(
            target.numerical_contracts().stated(),
            [first, second].as_slice()
        );
    }
}

/// Two lists that resolve alike but state different fallbacks are different
/// requests.
///
/// If the subject bound only the resolved contract, an explain trace and an
/// artifact would attribute one resolution to a preference they never saw.
#[test]
fn the_stated_preference_separates_requests_that_resolve_alike() {
    let program = program();
    let alone = verify_planned_request(CompilationRequest::governed_preferring(
        &program,
        NumericalContractPreference::ordered(vec![StrictF32NumericalContract::governed()]).unwrap(),
    ))
    .unwrap();
    let with_fallback = verify_planned_request(CompilationRequest::governed_preferring(
        &program,
        NumericalContractPreference::ordered(vec![
            StrictF32NumericalContract::governed(),
            StrictF32NumericalContract::governed_flush_to_zero(),
        ])
        .unwrap(),
    ))
    .unwrap();
    let alone = alone.for_target(0).unwrap();
    let with_fallback = with_fallback.for_target(0).unwrap();
    assert_eq!(
        alone.numerical_contract(),
        with_fallback.numerical_contract()
    );
    assert_ne!(
        alone.subject().canonical_explain_subject_bytes(),
        with_fallback.subject().canonical_explain_subject_bytes(),
    );
}

/// A request that states no contract does not compile, and says so.
///
/// The diagnostic names the contract as unstated rather than naming a
/// dimension: there is no default and no implicit strictest reading, so
/// there is no dimension the caller chose to report against.
#[test]
fn an_unstated_numerical_contract_is_refused_by_name() {
    let program = program();
    assert_eq!(
        NumericalContractPreference::ordered(Vec::new()),
        Err(RequestError::UnstatedNumericalContract)
    );
    let mut request = CompilationRequest::governed(&program);
    request.numerical_contracts.stated.clear();
    assert_eq!(
        verify_planned_request(request),
        Err(RequestError::UnstatedNumericalContract)
    );
}

/// A target that honours no stated entry rejects, naming every entry's cause.
///
/// The governed baseline deliberately declares nothing about the
/// always-positive flush, so a contract requiring it is `Undeclared` — the
/// fail-closed direction — rather than admitted. The rejection retains one
/// cause per stated entry, in the caller's order, so a two-entry preference
/// explains both entries rather than only the last.
#[test]
fn a_target_that_honours_no_stated_contract_rejects_with_a_cause_per_entry() {
    let program = program();
    let mut positive_flush = StrictF32NumericalContract::governed_flush_to_zero();
    positive_flush.input_subnormals = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::AlwaysPositive,
    };
    positive_flush.result_subnormals = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::AlwaysPositive,
    };
    // The contract must still be one this build registers, or the request
    // would be refused earlier for a different reason. It is not, so this
    // asserts the earlier refusal and then drives resolution directly.
    assert!(!positive_flush.is_governed());
    assert_eq!(
        verify_planned_request(CompilationRequest::governed_under(&program, positive_flush)),
        Err(RequestError::UnsupportedCapability {
            phase: "numerics",
            rule: "governed-contract-profile",
        })
    );

    let target = TargetProfile::governed();
    let error = resolve_numerical_contract(
        &NumericalContractPreference::ordered(vec![
            positive_flush,
            StrictF32NumericalContract::governed(),
        ])
        .unwrap(),
        // A profile that declares nothing at all: every dimension of every
        // entry is undeclared, so nothing may be admitted.
        &TargetProfile::governed_without_numerical_declarations(),
        // The fixture program is `f32`, which both stated entries resolve,
        // so applicability narrows nothing and every entry is asked.
        Some(ArithmeticType::F32),
    )
    .unwrap_err();
    let RequestError::NoResolvableNumericalContract {
        target_profile,
        rejections,
    } = error
    else {
        panic!("an unhonourable preference rejects by name");
    };
    assert_eq!(target_profile, *target.profile_key());
    assert_eq!(rejections.len(), 2, "one cause per stated entry");
    assert_eq!(rejections[0].contract_key(), positive_flush.key);
    assert_eq!(
        rejections[1].contract_key(),
        StrictF32NumericalContract::governed().key
    );
    for rejection in &rejections {
        assert!(matches!(rejection, ContractRejection::Undeclared { .. }));
        assert_eq!(
            rejection.dimension(),
            crate::target::honourability::NumericalDimension::InputSubnormals
        );
    }
}

/// The governed baseline resolves every registered contract, and its
/// declaration is what admits them.
#[test]
fn the_governed_baseline_honours_every_registered_contract() {
    let target = TargetProfile::governed();
    let expected = crate::target::honourability::CANONICAL_DIMENSIONS
        .into_iter()
        .filter(|dimension| crate::policy::is_consumable(*dimension))
        .count();
    for contract in StrictF32NumericalContract::named_profile() {
        let outcome = crate::physical::assess_contract(&target, contract).unwrap();
        let crate::target::feasibility::FeasibilityOutcome::Proven(evidence) = outcome else {
            panic!("the baseline honours {}", contract.key);
        };
        assert_eq!(
            evidence.honoured().len(),
            expected,
            "one per dimension an admitted operation can consume"
        );
        for honoured in evidence.honoured() {
            assert_eq!(
                honoured.means(),
                crate::target::honourability::HonouringMeans::SupportedExactly
            );
            assert_eq!(honoured.arithmetic(), contract.arithmetic);
            assert_eq!(honoured.profile().key(), target.profile_key().as_str());
        }
    }
}

/// A contract stating an arithmetic type the profile is silent about is
/// `Unknown`, never honoured by inheritance from a neighbouring type.
///
/// This is the measured case in miniature. One Apple profile flushes
/// subnormals in `f32` and preserves them in `f16`, so a declaration for one
/// width says nothing about the other; a resolver that fell back to a
/// neighbouring type's fact would report a conformance claim the hardware
/// contradicts.
#[test]
fn a_contract_for_an_undeclared_arithmetic_type_is_unknown() {
    let target = TargetProfile::governed();
    let mut contract = StrictF32NumericalContract::governed();
    contract.arithmetic = ArithmeticType::F16;
    let outcome = crate::physical::assess_contract(&target, contract).unwrap();
    let crate::target::feasibility::FeasibilityOutcome::Unknown(unknown) = outcome else {
        panic!("a profile silent about f16 cannot prove an f16 contract");
    };
    let first = unknown.dimensions().first().expect("a cause is reported");
    assert_eq!(first.arithmetic(), ArithmeticType::F16);
    assert_eq!(first.dimension(), NumericalDimension::InputSubnormals);
}

/// Every consumable contract dimension reaches the scheduled realization.
#[test]
fn realization_carries_every_consumable_contract_dimension() {
    let mut contract = StrictF32NumericalContract::governed();
    contract.permutation = NumericalPermission::Permitted;
    contract.signed_zero = NumericalPermission::Permitted;
    contract.nan_assumptions = ExceptionalValueAssumption::AssumeAbsent {
        provenance: tiler_ir::schedule::ValueDomainProvenance::CompilerProven,
    };
    contract.infinity_assumptions = ExceptionalValueAssumption::AssumeAbsent {
        provenance: tiler_ir::schedule::ValueDomainProvenance::RuntimeValidated,
    };
    let realization = contract.realization();
    assert_eq!(realization.permutation, contract.permutation);
    assert_eq!(realization.signed_zero, contract.signed_zero);
    assert_eq!(realization.nan_assumptions, contract.nan_assumptions);
    assert_eq!(
        realization.infinity_assumptions,
        contract.infinity_assumptions
    );
}
