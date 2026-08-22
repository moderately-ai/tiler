//! The request boundary's unit tests and control populations.
//!
//! Held together rather than split per child because most of what they assert is
//! about the boundary as a whole: a program is built, admitted, recognized, and
//! its subject encoded, and the assertion is usually about the relation between
//! two of those layers. Splitting the file along the children would put the
//! fixture builders in one module and the claims they support in another.
//!
//! Several populations here are exhaustive rather than sampled — the statable
//! contract space, the budget-resource vocabulary, every recognizer refusal rule
//! — and each states its own size beside itself so an enumeration that stopped
//! covering its domain fails at the count instead of passing over less.

use std::sync::Arc;

use super::*;
use tiler_ir::schedule::FlushedZeroSign;
use tiler_ir::semantic::{
    Bf16Add, Bf16Constant, Bf16Multiply, BroadcastAxisSource, CanonicalValue, CanonicalValueKind,
    F32Add, F32Broadcast, F32Constant, F32Gather, F32Multiply, F32RmsNorm, NormativeDefinitionRef,
    OperationArity, OperationAttributeSchema, OperationAttributes, OperationConformance,
    OperationDefinition, OperationDefinitionFacts, OperationEffect, OperationInferenceError,
    OperationInferencer, OperationSchema, ProviderDiagnosticCode, ProviderIdentity, RegistryError,
    ResolvedValueType, SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, StrictSerialF32Sum, TypeDefinitionFacts, ValueFact,
    ValueTypeDefinition, ValueTypeDefinitionKey, gather_index_resolved_type,
};
use tiler_ir::shape::{
    BindingSource, ExtentRelation, ExtentTerm, FactProvenance, GuardApplicability, RootBinding,
    SemanticInputConstraint, ShapeEnv, ShapeEnvBuilder, ShapeSymbol, SymbolScope, VariantGuard,
};

fn diagnostic_code(value: &str) -> ProviderDiagnosticCode {
    ProviderDiagnosticCode::new(value).unwrap()
}

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

pub(super) fn program() -> SemanticProgram {
    program_with_builder(SemanticProgramBuilder::try_standard().unwrap())
}

/// One ordinary governed gather occurrence over the admitted F32/U32 signature.
fn gather_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let source = builder
        .input::<F32>(InputKey::new("source").unwrap(), Shape::from_dims([4, 2]))
        .unwrap();
    let index = builder
        .input_resolved(
            InputKey::new("index").unwrap(),
            Shape::from_dims([3]),
            gather_index_resolved_type(),
        )
        .unwrap();
    let gathered = F32Gather::apply(&mut builder, source, index, Axis::new(0)).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), gathered)
        .unwrap();
    builder.build().unwrap()
}

fn program_with_builder(mut builder: SemanticProgramBuilder) -> SemanticProgram {
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let pointwise = F32Add::apply(&mut builder, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, pointwise, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

/// Builds one whole-program elementwise fixture and its expected nodes.
///
/// `(first * second) + third` over three declared inputs. It is deliberately
/// *not* a shape the superseded template could spell: two of its leaves are
/// distinct input tensors rather than constants, and the old recognizer
/// demanded exactly one input.
fn three_input_elementwise() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let root = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// A normalization over `[2, 2]` reduced on axis one, optionally scaled.
///
/// `weighted` decides which of the two shapes the ticket names is built: the
/// family as the whole declared output, and the family as a program stage a
/// later elementwise pass consumes.
fn normalization_program(weighted: bool, eps_bits: u32) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let value = builder
        .input::<F32>(InputKey::new("value").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("weight").unwrap(), shape)
        .unwrap();
    let normalized =
        tiler_ir::semantic::F32RmsNorm::apply(&mut builder, value, weight, Axis::new(1), eps_bits)
            .unwrap();
    let root = if weighted {
        F32Multiply::apply(&mut builder, normalized, value).unwrap()
    } else {
        normalized
    };
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// A registered family whose law realizes a region sequence is a program
/// stage, both as the declared output and as a chain's producer.
///
/// **The recognition is the law's and the partition is the occurrence's, and
/// both halves are asserted.** `tiler::rms-norm-f32@1` reaches this arm
/// because its registered `IndexRealizationLaw` realizes a region *sequence*
/// — no operation key appears in the recognizer — and what the recognized
/// part claims is the occurrence, once, because region formation is the
/// authority that enumerates the stages. `owns_region_members` therefore
/// answers for whichever stage atoms formation minted, which is what lets a
/// cover region covering one stage resolve to this output at all.
///
/// Watched failing under a deliberate perturbation: removing the
/// `laws.family_realizes_region_sequence(operation.key())` disjunct from
/// `plan_elementwise`'s folding discovery refuses the weighted program under
/// `operation-set`, which is the wall this ticket moved.
#[test]
fn a_registered_staged_family_is_recognized_as_a_program_stage() {
    let eps = 1.0e-6_f32.to_bits();

    // The family as the whole declared output.
    let whole = normalization_program(false, eps);
    let NormalizedOutput::Staged(staged) = recognize(&whole).unwrap() else {
        panic!("a family whose law realizes a region sequence is a staged stage")
    };
    assert_eq!(staged.operation, tiler_ir::semantic::rms_norm_f32_op());
    assert_eq!(staged.member, SemanticMemberId(0));
    assert_eq!(
        staged.operand_reads,
        [
            BoundaryRead::Input(DeclaredInputOrdinal::new(0)),
            BoundaryRead::Input(DeclaredInputOrdinal::new(1))
        ]
    );
    assert_eq!(staged.producer, None);
    assert_eq!(staged.output_shape, Shape::from_dims([2, 2]));
    assert_eq!(staged.output_elements, 4);
    assert!(
        !staged.attributes.is_empty(),
        "the occurrence's axis and eps record reaches the recognized shape"
    );

    // The family as a program stage a later pass consumes: the walk names
    // the value the chain materializes and the producer is this shape.
    let weighted = normalization_program(true, eps);
    let NormalizedOutput::Epilogue(chain) = recognize(&weighted).unwrap() else {
        panic!("an elementwise pass over a staged family's result is a chain")
    };
    let NormalizedOutput::Staged(producer) = chain.producer.as_ref() else {
        panic!("the chain's producer is the staged family")
    };
    assert_eq!(producer.member, SemanticMemberId(0));
    assert_eq!(chain.members, [SemanticStage::first(SemanticMemberId(1))]);

    // The partition: the occurrence once, and every region whose atoms are
    // stages of it.
    let output = NormalizedOutput::Staged(producer.clone());
    assert_eq!(
        output.members(),
        [SemanticStage::first(SemanticMemberId(0))]
    );
    let fold = SemanticStage::first(SemanticMemberId(0));
    let pass = fold.next_stage();
    assert!(output.owns_region_members(&[fold]));
    assert!(output.owns_region_members(&[pass]));
    assert!(output.owns_region_members(&[fold, pass]));
    assert!(
        !output.owns_region_members(&[]),
        "an empty member set is no region of this occurrence"
    );
    assert!(
        !output.owns_region_members(&[fold, SemanticStage::first(SemanticMemberId(1))]),
        "a region straddling the consumer belongs to no single part"
    );
}

/// A staged family reading a value another region *computes* refuses by name.
///
/// **This is the neighbour that keeps the widening below attributable, and
/// its rule survives the widening with a narrower meaning.** A multiply's
/// result is no materialization edge — [`materializes_its_result`] is the one
/// statement of where an edge may sit, and it says the expression vocabulary
/// evaluates a multiply per point — so admitting it here would be a second
/// account of that fact, and materializing it would add exactly the
/// observable rounding boundary the caller's program never asked for. Only
/// the operand differs between this program and
/// [`a_staged_family_reading_a_materialized_intermediate_is_recognized`].
///
/// Watched failing under a deliberate perturbation: replacing
/// `materializes_its_result(&root, laws)` with `true` admits the walk to
/// [`recognize_epilogue_producer`], which refuses the same program under
/// `operation-set` — a true statement about the producing family and not
/// about this occurrence's operand, and the reason the guard states the
/// operand rule itself.
#[test]
fn a_staged_family_reading_a_computed_value_refuses_by_name() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let value = builder
        .input::<F32>(InputKey::new("value").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("weight").unwrap(), shape)
        .unwrap();
    let doubled = F32Multiply::apply(&mut builder, value, value).unwrap();
    let normalized = tiler_ir::semantic::F32RmsNorm::apply(
        &mut builder,
        doubled,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), normalized)
        .unwrap();
    let program = builder.build().unwrap();
    assert_eq!(recognize(&program).unwrap_err(), "staged-operand");
}

/// A normalization over a materialized contraction result, optionally with
/// a trailing elementwise pass and optionally normalizing that result twice.
///
/// `ab,bc->ac` over `a` and `b`, with an independent third `[2, 2]` input
/// `w` serving as the normalization weight. The contraction's two reads are
/// therefore a strict subset of the complete interface in the ordinary
/// `rms_norm(matmul(a, b), w)` spelling.
fn contraction_fed_normalization(passed: bool, doubly_staged: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let left = builder
        .input::<F32>(InputKey::new("a").unwrap(), shape.clone())
        .unwrap();
    let right = builder
        .input::<F32>(InputKey::new("b").unwrap(), shape.clone())
        .unwrap();
    let independent_weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), shape)
        .unwrap();
    let structure = ContractionIndexStructure::new(
        [
            vec![ContractionIndex::new(0), ContractionIndex::new(1)],
            vec![ContractionIndex::new(1), ContractionIndex::new(2)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .expect("ab,bc->ac is an admitted structure");
    let product =
        tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, left, right)
            .unwrap();
    let weight = if doubly_staged {
        product
    } else {
        independent_weight
    };
    let normalized = tiler_ir::semantic::F32RmsNorm::apply(
        &mut builder,
        product,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    let root = if passed {
        F32Multiply::apply(&mut builder, normalized, independent_weight).unwrap()
    } else {
        normalized
    };
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// A staged family reading a materialized intermediate is recognized, and
/// the operand's boundary role is the recognized shape's.
///
/// **The admission this ticket exists for.** `rms_norm(matmul(a, b), w)`
/// reads its first operand across a materialization edge, which used to be
/// refused under `staged-operand` because nothing in the recognized staged
/// shape could record that operand zero is served by an edge rather than by a
/// declared buffer. Both halves are asserted, because either alone would be
/// consistent with a defect: the operand run names the boundary tensor per
/// operand, and the producer is carried so that the contraction's occurrence
/// is claimed by this output's walk — without which [`check_output_cover`]
/// refuses the program under `operation-set` for an occurrence no walk owns.
///
/// The partition is asserted too, on both sides of the edge, because it is
/// what lets a cover place two regions here: the occurrence's own stages and
/// the contraction's part are all this output's, and a set mixing them is
/// nobody's.
///
/// Watched failing under a deliberate perturbation: dropping the
/// `producer` field from [`NormalizedOutput::members`]'s staged arm — so the
/// walk claims only its own occurrence — refuses this program under
/// `operation-set`, which is exactly the coverage obligation the producer is
/// carried to discharge.
#[test]
fn a_staged_family_reading_a_materialized_intermediate_is_recognized() {
    let program = contraction_fed_normalization(false, false);
    assert_eq!(program.operation_count(), 2);
    let recognized = recognize(&program).expect("the staged operand is recognized");
    let NormalizedOutput::Staged(staged) = &recognized else {
        panic!("a normalization output recognizes as a staged family")
    };
    // The operand's source, carried by the recognized shape: operand zero is
    // the edge and operand one is the independent third declared input.
    assert_eq!(
        staged.operand_reads,
        [
            BoundaryRead::Staged,
            BoundaryRead::Input(DeclaredInputOrdinal::new(2))
        ]
    );
    assert_eq!(staged.member, SemanticMemberId(1));
    // The producer, recognized as the shape a standalone contraction output
    // would be, so every region builder the contraction already has applies
    // to it unchanged.
    let producer = staged
        .producer
        .as_deref()
        .expect("a staged operand carries the shape producing it");
    assert!(producer.contraction().is_some());
    assert_eq!(
        producer.members(),
        [SemanticStage::first(SemanticMemberId(0))]
    );

    // The whole partition: the contraction's part, and the occurrence's own
    // stages. The population is counted, so an assertion about the parts is
    // an assertion about the whole program's occurrences.
    assert_eq!(recognized.members().len(), program.operation_count());
    let fold = SemanticStage::first(SemanticMemberId(1));
    for part in [
        vec![SemanticStage::first(SemanticMemberId(0))],
        vec![fold],
        vec![fold.next_stage()],
    ] {
        assert!(
            recognized.owns_region_members(&part),
            "{part:?} is a part of this output's partition",
        );
    }
    assert!(
        !recognized.owns_region_members(&[SemanticStage::first(SemanticMemberId(0)), fold]),
        "a region straddling the materialization edge is no part",
    );

    // Which declared input each side reads, and at which count. Both are
    // read by the occurrence's own operand run *and* by the producer, and
    // the two agree at `[2, 2]`, so the accessor answers rather than
    // refusing.
    for ordinal in [0, 1, 2] {
        assert!(recognized.reads_declared_input(DeclaredInputOrdinal::new(ordinal)));
        assert_eq!(
            recognized.input_elements_at(DeclaredInputOrdinal::new(ordinal)),
            Some(4),
        );
    }
    assert!(!recognized.reads_declared_input(DeclaredInputOrdinal::new(3)));
    assert_eq!(recognized.max_input_elements(), 4);

    // **The boundary this widening does not move, asserted rather than
    // implied.** The consuming stage would read the occurrence's operand
    // edge *and* the value the producing stage handed it, and
    // `TensorRole::Intermediate` carries no ordinal, so
    // [`crate::physical::staged_plan`] declines the occurrence outright. Its
    // control is the same law over two declared operands, whose plan exists
    // — without which this `None` would be evidence that the plan derivation
    // had stopped working rather than evidence about the edge.
    assert_eq!(crate::physical::staged_plan(staged), None);
    let declared = normalization_program(false, 1.0e-6_f32.to_bits());
    let NormalizedOutput::Staged(control) = recognize(&declared).unwrap() else {
        panic!("a normalization output recognizes as a staged family")
    };
    assert!(crate::physical::staged_plan(&control).is_some());
}

/// One fixture of [`every_arm_answers_the_declared_tensors_own_count`] and
/// everything asserted about it.
///
/// Named rather than a tuple so each column reads as the claim it is: the
/// rows carry six columns each, and in a positional literal an exchanged
/// pair of `u64`s looks like a passing row.
struct CountRow {
    label: &'static str,
    /// The arm the fixture must reach, so a row whose recognition moved
    /// stops standing for the arm it names.
    arm: &'static str,
    output: NormalizedOutput,
    /// The iteration domain the widening read is *not* answered at, or
    /// `None` where the row has no widening read — for the two arms that
    /// hold no elementwise read list, and for the bare fold whose one read
    /// is dense.
    domain: Option<u64>,
    /// The count each declared ordinal must resolve to, in declaration
    /// order. Its length is the declared arity.
    counts: &'static [Option<u64>],
    max: u64,
}

/// Every arm of [`NormalizedOutput::input_elements_at`] answers the declared
/// tensor's own element count, and none answers a reading region's domain.
///
/// **The two numbers coincide unless a read widens, so most rows carry a
/// widening one.** A `[2]` weight broadcast into a `[2, 2]` region iterates
/// four points and holds two elements; an arm answering `4` would scale an
/// opaque call by the iteration space rather than by the buffer whose exact
/// access projects to that declared ordinal, which is the confidently
/// wrong work count [`crate::call_declaration::WorkScaling`] exists to
/// prevent. Each row therefore states the domain beside the counts and
/// refuses to run if they are equal, so a row that had no widening to get
/// wrong cannot pass for one that did.
///
/// **The rows are counted against the arms.** "Every arm" is the claim, so
/// the population is asserted to reach all five rather than described as
/// doing so; a variant added without a row fails here rather than shipping
/// unexamined. [`NormalizedOutput::reads_declared_input`] is asserted beside
/// every count because the two are separate statements of which ordinals a
/// walk reached, and
/// [`NormalizedProgram::agreed_input_elements_at`] refuses when they drift.
///
/// **Watched failing once each, every perturbation on the subject rather
/// than on an assertion, and each quoted by the row that caught it:**
///
/// - Restoring [`NormalizedOutput::input_elements_at`]'s pointwise arm to
///   `normalized.elements`, the reading region's domain it answered before:
///   *a sole widened pointwise read: ordinal 0 is not the declared tensor's
///   own count — left `Some(4)`, right `Some(2)`*.
/// - Restoring [`NormalizedOutput::max_input_elements`]'s pointwise arm to
///   the same domain, perturbed alone so the count rows still pass: *a sole
///   widened pointwise read: the largest declared input count this output
///   reads — left `4`, right `2`*. The two arms are perturbed separately
///   because together the first fires and hides the second.
/// - Restoring the serial-sum arm to `normalized.input_elements`: *a widened
///   read in a fold's prologue: ordinal 0 — left `Some(4)`, right
///   `Some(2)`*.
/// - Restoring the epilogue arm's consumed half to `chain.elements`: *a
///   widened read in a chain's epilogue: ordinal 1 — left `Some(4)`, right
///   `Some(2)`*.
/// - Dropping the serial-sum arm's `contributor_input` term, which is the
///   one read no read list describes: *a prologue-less fold's own
///   contributor read: ordinal 0 — left `None`, right `Some(6)`*.
/// - Answering [`read_tensor_elements`]'s structural arms with
///   `domain_elements` instead of the operand shape, which is the single
///   statement the three widening rows share: the first of them fires, *a
///   sole widened pointwise read: ordinal 0 — left `Some(4)`, right
///   `Some(2)`*, and each later row fires in turn once its predecessor is
///   admitted.
#[test]
fn every_arm_answers_the_declared_tensors_own_count() {
    // A `[2]` operand replicated across a leading axis into `[2, 2]`: the
    // read addresses two elements over a domain of four, which is the whole
    // difference these rows are about.
    let widen = |builder: &mut SemanticProgramBuilder, operand: tiler_ir::semantic::Value<F32>| {
        let mapping = tiler_ir::semantic::BroadcastAxisMapping::new(
            [Extent::new(2), Extent::new(2)],
            [
                tiler_ir::semantic::BroadcastAxisSource::Replicate,
                tiler_ir::semantic::BroadcastAxisSource::FromOperand(Axis::new(0)),
            ],
        )
        .expect("one replicated axis over a rank-one operand is an admitted relation");
        tiler_ir::semantic::F32Broadcast::apply(builder, &mapping, operand)
            .expect("the standard registry admits the broadcast family")
    };
    let weight = |builder: &mut SemanticProgramBuilder| {
        builder
            .input::<F32>(InputKey::new("w").unwrap(), Shape::from_dims([2]))
            .unwrap()
    };

    // `w + w` over the widened read alone: one declared input, read only
    // through the relation, so this is the row where the maximum moves too.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let w = weight(&mut builder);
    let widened = widen(&mut builder, w);
    let root = F32Add::apply(&mut builder, widened, widened).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let sole_widened_read = builder.build().unwrap();

    // `a * broadcast(w)`: the widened read beside a dense one, so the two
    // ordinals must answer different counts from one region.
    let mixed_program = |folded: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let w = weight(&mut builder);
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let widened = widen(&mut builder, w);
        let scaled = F32Multiply::apply(&mut builder, a, widened).unwrap();
        let root = if folded {
            StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap()
        } else {
            scaled
        };
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };

    // `sum(a, axis 1)`: no prologue, so the fold's own contributor read is
    // the one access no read list describes. Nothing widens here, and the
    // row is what keeps that term live.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let root = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let bare_fold = builder.build().unwrap();

    // `sum(a, axis 2) * broadcast(w)`: the producer folds ordinal `0` at its
    // own twelve-element shape and the epilogue widens ordinal `1` over a
    // four-point domain, so one chain carries both halves.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2, 3]))
        .unwrap();
    let w = weight(&mut builder);
    let reduced = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(2)]).unwrap();
    let widened = widen(&mut builder, w);
    let root = F32Multiply::apply(&mut builder, reduced, widened).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let widened_epilogue = builder.build().unwrap();

    let arm = |output: &NormalizedOutput| match output {
        NormalizedOutput::SerialSum(_) => "serial-sum",
        NormalizedOutput::Pointwise(_) => "pointwise",
        NormalizedOutput::Contraction(_) => "contraction",
        NormalizedOutput::Epilogue(_) => "epilogue",
        NormalizedOutput::Staged(_) => "staged",
        NormalizedOutput::Gather(_) => "gather",
    };
    let rows: [CountRow; 7] = [
        CountRow {
            label: "a sole widened pointwise read",
            arm: "pointwise",
            output: recognize(&sole_widened_read).expect("a widened read is an elementwise region"),
            domain: Some(4),
            counts: &[Some(2)],
            max: 2,
        },
        CountRow {
            label: "a widened pointwise read beside a dense one",
            arm: "pointwise",
            output: recognize(&mixed_program(false))
                .expect("a widened read is an elementwise region"),
            domain: Some(4),
            counts: &[Some(2), Some(4)],
            max: 4,
        },
        CountRow {
            label: "a widened read in a fold's prologue",
            arm: "serial-sum",
            output: recognize(&mixed_program(true)).expect("a widened prologue read is recognized"),
            domain: Some(4),
            counts: &[Some(2), Some(4)],
            max: 4,
        },
        CountRow {
            label: "a prologue-less fold's own contributor read",
            arm: "serial-sum",
            output: recognize(&bare_fold).expect("a fold over a declared input is recognized"),
            domain: None,
            counts: &[Some(6)],
            max: 6,
        },
        CountRow {
            label: "a widened read in a chain's epilogue",
            arm: "epilogue",
            output: recognize(&widened_epilogue).expect("a widened epilogue read is recognized"),
            domain: Some(4),
            counts: &[Some(12), Some(2)],
            max: 12,
        },
        CountRow {
            label: "a contraction's two operands",
            arm: "contraction",
            output: recognize(&contraction_program(false))
                .expect("a binary contraction is recognized"),
            domain: None,
            counts: &[Some(6), Some(12)],
            max: 12,
        },
        CountRow {
            label: "a staged family's operand run",
            arm: "staged",
            output: recognize(&normalization_program(false, 1.0e-6_f32.to_bits()))
                .expect("a normalization is recognized"),
            domain: None,
            counts: &[Some(4), Some(4)],
            max: 4,
        },
    ];
    let reached: BTreeSet<&str> = rows.iter().map(|row| row.arm).collect();
    assert_eq!(
        reached.len(),
        5,
        "the rows reach {reached:?}, which is not every arm of the accessor",
    );

    for CountRow {
        label,
        arm: expected_arm,
        output,
        domain,
        counts,
        max,
    } in rows
    {
        assert_eq!(
            arm(&output),
            expected_arm,
            "{label}: the fixture recognized as another arm, so the row proves nothing about \
             the one it names",
        );
        if let Some(domain) = domain {
            assert!(
                counts.iter().any(|count| *count != Some(domain)),
                "{label}: every count equals the domain of {domain}, so this row cannot \
                 observe the difference it exists for",
            );
        }
        for (ordinal, expected) in counts.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).expect("the fixtures declare few inputs");
            assert_eq!(
                output.input_elements_at(DeclaredInputOrdinal::new(ordinal)),
                *expected,
                "{label}: ordinal {ordinal} is not the declared tensor's own count",
            );
            assert_eq!(
                output.reads_declared_input(DeclaredInputOrdinal::new(ordinal)),
                expected.is_some(),
                "{label}: ordinal {ordinal} — the predicate and the count disagree about what \
                 this walk reads",
            );
        }
        let past = u32::try_from(counts.len()).expect("the fixtures declare few inputs");
        assert_eq!(
            output.input_elements_at(DeclaredInputOrdinal::new(past)),
            None,
            "{label}: an ordinal past the declaration produced a count",
        );
        assert_eq!(
            output.max_input_elements(),
            max,
            "{label}: the largest declared input count this output reads",
        );
    }
}

/// The two shapes a staged operand still refuses, each by its own name.
///
/// **Both are asserted rather than left implicit, because one admitted shape
/// reads as general support unless its boundary is stated.** Their admitted
/// neighbour is
/// [`a_staged_family_reading_a_materialized_intermediate_is_recognized`]'s
/// program, which differs from each by exactly the property named:
///
/// - *A second operand supplied by a materialization edge.*
///   `rms_norm(m, m)` gives one occurrence two `TensorRole::Intermediate`
///   reads, and that role carries no ordinal, so nothing says which edge each
///   binds. `staged-operand-conflict`.
/// - *An occurrence already at the far side of an edge reading its own.*
///   `rms_norm(matmul(a, b), w) * w` makes the normalization an epilogue
///   chain's producer, so admitting its operand edge would be a recognized
///   chain two materialization boundaries deep. `staged-operand-depth`, the
///   depth rule's one guard, stated at [`StagedOperandAdmission`].
///
/// Each was watched failing before it was restored: with the
/// `producer.is_some()` guard deleted the first program is recognized with
/// two `BoundaryRead::Staged` operands and one producer, and with the
/// `StagedOperandAdmission::NoEdge` guard deleted the second is recognized as
/// a two-boundary chain — both admissions no region vocabulary here can
/// spell.
///
/// **The second perturbation was rerun on 2026-08-08 and its cost measured**,
/// because "no region vocabulary can spell it" is a claim about a stage this
/// assertion cannot see. Handing `recognize_epilogue_producer`'s call site
/// `OneEdge` recognizes the program as
/// `Epilogue { producer: Staged { producer: Some(Contraction), operand_reads:
/// [Staged, Input(2)] } }` — a well-formed nesting — and this row is the
/// *only* one of the crate's 784 tests that moves. End to end the program
/// then refuses `NoFeasiblePlan` rather than compiling.
/// `crates/tiler-compiler/tests/recognized_chain_depth_boundary.rs` holds
/// that measurement and the trigger that reopens it.
#[test]
fn a_staged_operand_still_refuses_a_second_edge_and_a_deeper_chain() {
    assert_eq!(
        recognize(&contraction_fed_normalization(false, true)).unwrap_err(),
        "staged-operand-conflict",
    );
    assert_eq!(
        recognize(&contraction_fed_normalization(true, false)).unwrap_err(),
        "staged-operand-depth",
    );
}

/// The staged subject separates an edge-fed operand from a declared one, and
/// separates a carried producer from an absent one.
///
/// **Two claims, each isolated, because either alone would pass on the
/// other's evidence.** The occurrence's own operand run and the producer are
/// two facts the `staged-family.v2` arm writes, and a forgery that moved both
/// at once would be separated by whichever the encoder still carried — the
/// exact way a check stops exercising its shape while staying green.
///
/// Each forgery therefore moves exactly one field of the *same* recognized
/// value, leaving every operand shape, element count, key, member ordinal and
/// published shape identical. Neither forgery is a value the recognizer
/// produces; that is what makes them drivable at all, and it is the same
/// device the request-subject mutation tests above use.
///
/// Watched failing under two deliberate perturbations, one per claim:
/// dropping the role tag from `encode_output_subject`'s staged arm makes the
/// first pair equal, and dropping its producer run makes the second pair
/// equal.
#[test]
fn a_staged_subject_separates_an_edge_fed_operand_from_a_declared_one() {
    let program = contraction_fed_normalization(false, false);
    let normalized = select_supported_strategy(&program, &laws_of(&program)).unwrap();
    let [recognized] = normalized.outputs() else {
        panic!("the fixture declares one output");
    };
    let encoded = |output: &NormalizedOutput| {
        let mut bytes = Vec::new();
        encode_output_subject(&mut bytes, &output_subject(output));
        bytes
    };
    let forge = |edit: fn(&mut NormalizedStaged)| {
        let mut forged = recognized.clone();
        let NormalizedOutput::Staged(staged) = &mut forged else {
            panic!("a normalization output recognizes as a staged family")
        };
        edit(staged);
        encoded(&forged)
    };
    assert_ne!(
        encoded(recognized),
        forge(|staged| {
            staged.operand_reads[0] = BoundaryRead::Input(DeclaredInputOrdinal::new(0));
        }),
        "the operand's boundary role is part of what the occurrence reads",
    );
    assert_ne!(
        encoded(recognized),
        forge(|staged| staged.producer = None),
        "the shape writing the edge is part of what this partition computes",
    );
}

/// Two occurrences differing only in `eps` bind different request subjects.
///
/// The attribute record is what separates them: both programs declare the
/// same keys, the same shapes, the same operand map, the same member, and
/// the same element counts, so a staged subject arm that omitted the record
/// would give two different normalizations one identity. Watched failing
/// under a deliberate perturbation: dropping the attribute run from
/// `encode_output_subject`'s staged arm makes the two subjects equal.
#[test]
fn a_staged_subject_separates_two_occurrences_differing_only_in_eps() {
    let subject_bytes = |eps_bits: u32| {
        let program = normalization_program(false, eps_bits);
        let normalized = select_supported_strategy(&program, &laws_of(&program)).unwrap();
        let mut bytes = Vec::new();
        for output in normalized.outputs() {
            encode_output_subject(&mut bytes, &output_subject(output));
        }
        bytes
    };
    let first = subject_bytes(1.0e-6_f32.to_bits());
    let second = subject_bytes(1.0e-5_f32.to_bits());
    assert_ne!(
        first, second,
        "the occurrence's eps payload is part of what the staged stage computes"
    );
}

/// Builds the five-node `input * scale + bias` expression a forgery swaps in.
/// Replaces one recognized fold's prologue expression, leaving its reads alone.
///
/// The mutation is the *subject's*, which is what makes the receipt check that
/// follows a perturbation rather than an assertion edit: the recomputed request
/// subject carries the forged expression whole, and no other fact moves.
fn forge_prologue(normalized: &mut NormalizedProgram, expression: PointwiseF32Expression) {
    let SerialSumContributor::PointwisePrologue {
        expression: recognized,
        ..
    } = &mut normalized.serial_sum_mut().contributor
    else {
        panic!("the fixture folds a pointwise prologue over declared inputs");
    };
    *recognized = expression;
}

fn affine_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(scale_bits).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(bias_bits).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// The realization-law authority recognition consults, for one fixture.
///
/// Paired with the governed scalar profile, which is what the compile path
/// pairs. A fixture that registers its own operations has a semantic
/// authority the governed scalars were never frozen over, so it is paired
/// with the empty scalar registry built over *its* semantic authority
/// instead — recognition asks this registry one question, whether a family's
/// registered law realizes a region sequence, and that reads the semantic
/// law rows alone.
fn laws_of(program: &SemanticProgram) -> FrozenIndexRealizationLawRegistry {
    let semantic = program.semantic_registry().clone();
    FrozenIndexRealizationLawRegistry::from_semantic(
        semantic.clone(),
        governed_scalars().expect("the governed scalar profile is coherent"),
    )
    .or_else(|_| {
        FrozenIndexRealizationLawRegistry::from_semantic(
            semantic.clone(),
            tiler_ir::index::ScalarRegistryBuilder::new(semantic).freeze(),
        )
    })
    .expect("a law authority over the fixture's own semantic authority coheres")
}

/// A gather stops first at exact target dispatch, then at governed lowering.
///
/// The second compile changes only the target's exact U32 dispatch fact. It
/// keeps the semantic program byte-for-byte identical, so the advance from the
/// target-local `DTypeNotDispatchable` refusal to the next layer pins the
/// request boundary's ordered diagnostic layers without granting Gather a
/// production target claim or a planning route.
///
/// **The second expectation moved from `dtype-recognized` to
/// `missing-capability`, and the move is this lane's landing.** The U32 index is
/// no longer refused by whole-program arithmetic recognition: it is exempt by
/// operand position, the gather recognizer resolves the output, and the request
/// verifies. The next authority that has nothing to say about a gather is the
/// governed lowering registry, which carries no gather capability row — so the
/// program now reaches `phase: "lowering"` and stops there.
///
/// That is still fail-closed, and deliberately so: no gather acquires a schedule,
/// kernel, artifact, cache, or dispatch route from this lane. Two named
/// authorities stand between this refusal and one that could — the governed row
/// itself, and `RegionVocabularyWall::GatherProofUnavailable`, which is what
/// physical planning answers once a row exists.
///
/// Watched failing under a deliberate subject perturbation: removing the
/// U32 row from `governed_with_gather_index_dispatch_for_test` makes the
/// second compile return the same target-local refusal as the first.
#[test]
fn a_governed_gather_refuses_at_dispatch_before_governed_lowering() {
    let program = gather_program();
    let product = crate::pipeline::compile(CompilationRequest::governed(&program))
        .expect("a target-local refusal is an ordinary compilation product");
    let [outcome] = product.targets.as_slice() else {
        panic!("the governed request carries one target outcome");
    };
    assert_eq!(
        outcome.failure(),
        Some(&crate::pipeline::CompileError::NoFeasiblePlan(
            crate::pipeline::NoFeasiblePlanError::Request(RequestError::DTypeNotDispatchable {
                target_profile: TargetProfile::governed().profile_key().clone(),
                resolved_type: Box::new(gather_index_resolved_type()),
                disposition: DTypeDispatchRefusalDisposition::Unknown,
            })
        )),
        "the governed target answers for the exact U32 index type before recognition",
    );

    let mut widened = CompilationRequest::governed(&program);
    widened.target_profiles = vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    // The refusal is compared by its phase and rule alone. A `CompileError`
    // reaching this layer carries a whole explain trace, and comparing the
    // error whole would pin every byte of that trace here — a fixture-shaped
    // assertion that would move for reasons unrelated to the ordered layers
    // this test is about.
    let advanced = crate::pipeline::compile(widened).expect_err("the widened request refuses");
    assert_eq!(
        planning_capability_rule(&advanced)
            .unwrap_or_else(|| panic!("the widened request refused with {advanced:?}")),
        ("lowering", "missing-capability"),
        "an exact U32 dispatch fact advances the same program past recognition to \
governed lowering, which carries no gather capability row",
    );
}

/// The real output recognizer resolves a Gather to its own recognized shape.
///
/// **This assertion is the inverse of the one it replaces, and the inversion is
/// the landing.** It previously required `operation-set` — the refusal a walk
/// reports for an occurrence no recognizer claims — because no gather arm
/// existed. `recognize_gather` is that arm, so the same fixture through the same
/// real realization-law authority and output walk now produces a recognized
/// shape, and leaving the old expectation in place would have left the suite
/// asserting the opposite of the tree.
///
/// Every field is checked against the fixture rather than the shape being merely
/// destructured, because the fields are what the request subject binds: the two
/// declared ordinals are the ADR 0108 amendment's checked association, and the
/// result shape is derived here rather than read from the graph.
///
/// Watched failing under a deliberate subject perturbation: swapping
/// `gather_program`'s two declared inputs so the U32 index is declared first
/// moves `source_input`/`index_input` to `1`/`0` and reddens this exact
/// assertion — the ordinals are read from declaration position, not assumed.
#[test]
fn the_real_request_recognizer_resolves_a_gather_to_its_own_shape() {
    let program = gather_program();
    let recognized = recognize_program_outputs(&program, &laws_of(&program), ArithmeticType::F32)
        .expect("the real output walk recognizes a gather");
    let [output] = recognized.outputs() else {
        panic!("the fixture declares one output");
    };
    let gather = output.gather().expect("the recognized shape is a gather");
    assert_eq!(gather.source_input, 0, "the source is declared first");
    assert_eq!(gather.index_input, 1, "the index is declared second");
    assert_eq!(gather.source_shape, Shape::from_dims([4, 2]));
    assert_eq!(gather.index_shape, Shape::from_dims([3]));
    assert_eq!(
        gather.result_shape,
        Shape::from_dims([3, 2]),
        "the index shape splices into the source at the gathered axis",
    );
    assert_eq!(gather.axis, Axis::new(0));
    assert_eq!(
        gather.index_access,
        AccessOrdinal::new(1),
        "the address read is canonical local access 1",
    );
    assert_eq!(
        [
            gather.source_elements,
            gather.index_elements,
            gather.result_elements
        ],
        [8, 3, 6],
    );
}

/// A gather whose source is not a declared program input is refused by name.
///
/// The accepted surface admits only declared program inputs as either gather
/// operand. This drives the `gather-operand-input` refusal specifically, rather
/// than letting such a program fall through to a neighbouring rule.
///
/// **The perturbation is on the subject, and it is one edge.** The fixture is
/// [`gather_program`] with a single `F32Multiply` interposed on the source, so
/// the gathered-from value is computed rather than declared while its type,
/// shape, the index operand, the axis, and the gather occurrence itself are all
/// unchanged. Removing that one multiply restores the recognized shape the test
/// above asserts, which is what shows this refusal is about the operand's source
/// and not about the family.
#[test]
fn a_gather_reading_a_computed_source_is_refused_under_operand_input() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let source = builder
        .input::<F32>(InputKey::new("source").unwrap(), Shape::from_dims([4, 2]))
        .unwrap();
    let index = builder
        .input_resolved(
            InputKey::new("index").unwrap(),
            Shape::from_dims([3]),
            gather_index_resolved_type(),
        )
        .unwrap();
    let one = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let computed = F32Multiply::apply(&mut builder, source, one).unwrap();
    let gathered = F32Gather::apply(&mut builder, computed, index, Axis::new(0)).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), gathered)
        .unwrap();
    let program = builder.build().unwrap();
    assert_eq!(
        recognize_program_outputs(&program, &laws_of(&program), ArithmeticType::F32),
        Err(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "gather-operand-input",
        }),
    );
}

/// Recognizes one program through the whole boundary, or reports the rule.
///
/// Answers with the sole recognized output, because every fixture reaching
/// it declares one; [`recognize_outputs`] is the multi-output form.
fn recognize(program: &SemanticProgram) -> Result<NormalizedOutput, &'static str> {
    strategy_rule(select_supported_strategy(program, &laws_of(program))).map(|recognized| {
        let [output] = recognized.outputs() else {
            panic!("the fixture declares one output");
        };
        output.clone()
    })
}

/// Recognizes one program's ordered named outputs, or reports the rule.
///
/// Drives [`recognize_program_outputs`] directly rather than through
/// [`select_supported_strategy`], so a refusal this helper returns is one
/// the walks themselves produced. The two program-wide properties the
/// boundary checks before them are asserted rather than reported, which is
/// what makes that attribution exact.
fn recognize_outputs(program: &SemanticProgram) -> Result<NormalizedProgram, &'static str> {
    assert_ne!(program.input_count(), 0, "the fixture declares an input");
    assert!(
        program
            .values()
            .all(|value| value.resolved_type() == &F32::resolved_type()),
        "the fixture is f32 throughout",
    );
    strategy_rule(recognize_program_outputs(
        program,
        &laws_of(program),
        ArithmeticType::F32,
    ))
}

/// Reduces one recognition outcome to the strategy rule it refused under.
fn strategy_rule(
    outcome: Result<NormalizedProgram, RequestError>,
) -> Result<NormalizedProgram, &'static str> {
    outcome.map_err(|error| match error {
        RequestError::UnsupportedCapability {
            phase: "strategy",
            rule,
        } => rule,
        other => panic!("recognition refuses under the strategy phase, got {other:?}"),
    })
}

#[derive(Clone, Copy)]
enum TestOperation {
    Constant,
    Binary,
    Sum,
}

impl OperationInferencer for TestOperation {
    fn infer(
        &self,
        request: tiler_ir::semantic::OperationInferenceRequest<'_>,
        outputs: &mut tiler_ir::semantic::OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let attributes = request.attributes();
        match self {
            Self::Constant => {
                outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
            }
            Self::Binary => {
                let left = request.static_operand_shape(0)?;
                let right = request.static_operand_shape(1)?;
                let shape = if left.rank() == 0 {
                    right.clone()
                } else if right.rank() == 0 || left == right {
                    left.clone()
                } else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.binary.shape"),
                        "operands must have equal shapes or include one scalar",
                    )
                    .unwrap());
                };
                outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
            }
            Self::Sum => {
                let Some(CanonicalValueView::Sequence(values)) = attributes
                    .get(REDUCTION_AXES_ATTRIBUTE)
                    .map(CanonicalValue::view)
                else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.sum.axes"),
                        "sum axes must be a sequence",
                    )
                    .unwrap());
                };
                let axes = values
                    .iter()
                    .map(|value| match value.view() {
                        CanonicalValueView::Unsigned {
                            width: CanonicalIntegerWidth::Bits32,
                            bits,
                        } => u32::try_from(bits).map(Axis::new).map_err(|_| {
                            OperationInferenceError::new(
                                diagnostic_code("test.sum.axis-width"),
                                "sum axis exceeds u32",
                            )
                            .unwrap()
                        }),
                        _ => Err(OperationInferenceError::new(
                            diagnostic_code("test.sum.axis-kind"),
                            "sum axes must be u32 values",
                        )
                        .unwrap()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                outputs.try_push(ValueFact::new(
                    F32::resolved_type(),
                    request.static_operand_shape(0)?.without_axes(&axes),
                ))
            }
        }
    }
}

struct GovernedTestSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for GovernedTestSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "governed-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_marked_value_type::<F32>(
            ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(
                    TypeKey::new("tiler", "f32", 1).expect("the test F32 key is valid"),
                ),
                NormativeDefinitionRef::new("test binary32 semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ),
            F32::resolved_type(),
        )?;
        register_test_operation(
            registrar,
            constant_f32_op(),
            0,
            [OperationAttributeSchema::required(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValueKind::FloatBits,
            )],
            TestOperation::Constant,
        )?;
        register_test_operation(registrar, multiply_f32_op(), 2, [], TestOperation::Binary)?;
        register_test_operation(registrar, add_f32_op(), 2, [], TestOperation::Binary)?;
        register_test_operation(
            registrar,
            strict_serial_sum_f32_op(),
            1,
            [OperationAttributeSchema::required(
                REDUCTION_AXES_ATTRIBUTE,
                CanonicalValueKind::Sequence,
            )],
            TestOperation::Sum,
        )
    }
}

fn register_test_operation<const N: usize>(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: OpKey,
    operands: u32,
    attributes: [OperationAttributeSchema; N],
    inferencer: TestOperation,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        key,
        OperationSchema::new(
            OperationArity::exact(operands),
            OperationArity::exact(1),
            attributes,
        )
        .expect("the test operation schema is valid"),
        NormativeDefinitionRef::new("test governed operation semantics")?,
        OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
        OperationConformance::new(CanonicalValue::boolean(true)),
        OperationEffect::Pure,
        Arc::new(inferencer),
    ))
}

fn governed_test_program(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::new();
    registry
        .register_provider(&GovernedTestSemantics { revision })
        .unwrap();
    program_with_builder(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

struct UnusedSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for UnusedSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "unused-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(
                TypeKey::new("tiler-test", "unused", 1).expect("the test key is valid"),
            ),
            NormativeDefinitionRef::new("unused test semantics")?,
            TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
        ))
    }
}

fn program_with_unused_provider(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision })
        .unwrap();
    program_with_builder(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

fn request_with_matching_empty_capabilities(program: &SemanticProgram) -> CompilationRequest<'_> {
    let scalars =
        tiler_ir::index::ScalarRegistryBuilder::new(program.semantic_registry().clone()).freeze();
    let lowering = crate::capability::LoweringCapabilityRegistryBuilder::new(
        program.semantic_registry().clone(),
        scalars.clone(),
    )
    .unwrap()
    .freeze();
    let mut request = CompilationRequest::governed(program);
    request.capabilities = CompilerCapabilitySnapshot::new(lowering, scalars);
    request
}

#[test]
fn governed_request_selects_the_supported_serial_sum_strategy() {
    let program = program();
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let [recognized] = verified.normalized.outputs() else {
        panic!("the fixture declares one output");
    };
    let normalized = recognized.serial_sum();
    assert_eq!(normalized.input_shape, Shape::from_dims([2, 3]));
    assert_eq!(normalized.output_shape, Shape::from_dims([2]));
    assert_eq!(normalized.reduction_axes, [Axis::new(1)]);
    assert_eq!(normalized.input_elements, 6);
    assert_eq!(normalized.output_elements, 2);
    assert_eq!(normalized.input_keys, [InputKey::new("input").unwrap()]);
    // The prologue is the recognized expression, not two constants: it is
    // `input * 2.0 + 1.0` in the physical node vocabulary, and the affine
    // pair the fused region needs is recovered from it rather than stored
    // beside it.
    let prologue = normalized
        .contributor
        .prologue()
        .expect("a fold over a computed contributor has a prologue");
    assert_eq!(prologue.input_count(), 1);
    assert!(matches!(
        prologue.nodes(),
        [
            PointwiseF32Node::Input { .. },
            PointwiseF32Node::Constant { bits: scale },
            PointwiseF32Node::Multiply { .. },
            PointwiseF32Node::Constant { bits: bias },
            PointwiseF32Node::Add { .. },
        ] if *scale == 2.0_f32.to_bits() && *bias == 1.0_f32.to_bits()
    ));
    assert_eq!(
        verified
            .target_slots
            .iter()
            .map(|slot| &slot.target_profile)
            .collect::<Vec<_>>(),
        [&TargetProfile::governed()]
    );
}

/// The composed program: a multi-input elementwise expression feeding a
/// strict serial reduction.
///
/// **This is the shape no normalization matched.** The superseded serial-sum
/// template demanded exactly one declared input and the exact four- or
/// five-operation `x * scale + bias` prologue; the superseded pointwise
/// template refused anything containing a reduction. `sum((a * b) + c)` over
/// three declared inputs is neither, and it is admitted here on the strength
/// of its occurrences: two recognized elementwise families composing into one
/// expression, feeding one recognized reduction.
#[test]
fn a_multi_input_elementwise_expression_feeding_a_reduction_is_recognized() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let biased = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    let program = builder.build().unwrap();

    let NormalizedOutput::SerialSum(recognized) =
        recognize(&program).expect("the composed program is recognized")
    else {
        panic!("a program whose output is a reduction recognizes as one");
    };
    assert_eq!(recognized.input_keys.len(), 3);
    assert_eq!(recognized.input_shape, Shape::from_dims([2, 3]));
    assert_eq!(recognized.output_shape, Shape::from_dims([2]));
    assert_eq!(
        recognized
            .contributor
            .prologue()
            .expect("a fold over a computed contributor has a prologue")
            .input_count(),
        3,
        "one leaf per declared input tensor",
    );
    // Three elementwise occurrences in the prologue is exactly two — the
    // multiply and the add — with no constant, and the reduction is the
    // third occurrence of the program.
    assert_eq!(recognized.members.pointwise().len(), 2);
    assert_eq!(recognized.members.all().len(), program.operation_count());
    // No fused spelling exists: `FusedMultiplyAddSerialSum` applies one
    // scalar constant and one scalar bias, and this prologue applies neither.
    let verified = verify_planned_request(CompilationRequest::governed(&program))
        .unwrap()
        .for_target(0)
        .unwrap();
    assert_eq!(
        crate::physical::fused_prologue_constants(verified.sole_output()),
        None
    );
}

/// A reduction over a declared input is recognized with no prologue.
///
/// `sum(x)` is the simplest fold there is, and it used to be the one shape
/// this recognizer refused for a wall *below* it: `verify_access_and_semantics`
/// required a `ScalarProgram::StrictSerialSum` region's contributor access to
/// read `TensorRole::Intermediate`, so a region folding the input directly was
/// rejected as malformed. That arm now admits the fold's declared contributor
/// domain, and the absence of a prologue is recorded as `None` rather than as
/// an identity expression — which is what keeps a cover from spelling the copy
/// kernel the refusal existed to avoid.
///
/// Its neighbour is the same fold with one elementwise occurrence between the
/// input and the sum, asserted beside it so the `None` is attributable to the
/// missing prologue rather than to the fold.
#[test]
fn a_reduction_over_a_declared_input_is_recognized_with_no_prologue() {
    let fold = |prologue: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let contributor = if prologue {
            let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
            F32Multiply::apply(&mut builder, input, scale).unwrap()
        } else {
            input
        };
        let sum = StrictSerialF32Sum::apply(&mut builder, contributor, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    };
    let bare = fold(false);
    assert_eq!(bare.operation_count(), 1);
    let Ok(NormalizedOutput::SerialSum(recognized)) = recognize(&bare) else {
        panic!("a fold over a declared input is recognized as a serial sum");
    };
    // The source is *named* rather than inferred from absent fields: the arm
    // itself is what says this fold reads a declared input, and it carries the
    // recognized ordinal.
    assert!(matches!(
        recognized.contributor,
        SerialSumContributor::DeclaredInput(ordinal) if ordinal == 0
    ));
    assert_eq!(recognized.contributor.prologue(), None);
    assert_eq!(recognized.contributor.prologue_reads(), []);
    // One part, not two: the empty prologue part is not a member set a cover
    // region may match, which is what `prologue_members` states.
    assert_eq!(recognized.prologue_members(), None);
    assert_eq!(recognized.continuation_members(), None);
    assert_eq!(recognized.members.reduction().len(), 1);

    let neighbour = fold(true);
    let Ok(NormalizedOutput::SerialSum(recognized)) = recognize(&neighbour) else {
        panic!("a fold over a computed contributor is recognized as a serial sum");
    };
    assert!(matches!(
        recognized.contributor,
        SerialSumContributor::PointwisePrologue { .. }
    ));
    assert_eq!(recognized.prologue_members().map(<[_]>::len), Some(2));
    assert_eq!(recognized.continuation_members(), None);
}

/// Elementwise recognition follows the graph, not a taught depth or arity.
///
/// Each shape below was refused by the superseded template, and each was
/// refused for the *leaf count* rather than for anything about what it
/// computes: the old recognizer admitted exactly two operations over exactly
/// three leaves in one of two associations.
#[test]
fn elementwise_recognition_admits_depth_sharing_and_multiple_inputs() {
    // Three declared inputs and a mixed multiply-then-add chain.
    let three = three_input_elementwise();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&three).expect("a three-input expression is recognized")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    assert_eq!(
        recognized.input_keys,
        [
            InputKey::new("a").unwrap(),
            InputKey::new("b").unwrap(),
            InputKey::new("c").unwrap(),
        ],
    );
    assert_eq!(recognized.expression.f32().input_count(), 3);
    assert_eq!(recognized.members.len(), three.operation_count());

    // A four-deep chain: `((a * 2.0) + b) * ((a * 2.0) + b)`, whose shared
    // subexpression is one node rather than two. Depth and sharing are both
    // beyond what a three-leaf template could spell.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let first = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let second = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, first, scale).unwrap();
    let shifted = F32Add::apply(&mut builder, scaled, second).unwrap();
    let root = F32Multiply::apply(&mut builder, shifted, shifted).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let deep = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&deep).expect("a deep shared expression is recognized")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    assert_eq!(recognized.expression.f32().input_count(), 2);
    assert_eq!(recognized.members.len(), deep.operation_count());
    assert_eq!(
        recognized.expression.f32().nodes().len(),
        6,
        "the shared `(a * 2.0) + b` is one node, not two",
    );

    // One input read at two leaves, which binds one read access.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let constant = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let doubled = F32Add::apply(&mut builder, input, input).unwrap();
    let root = F32Add::apply(&mut builder, doubled, constant).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let repeated = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&repeated).expect("a repeated read is recognized")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    assert_eq!(recognized.expression.f32().input_count(), 1);
    assert_eq!(recognized.input_keys.len(), 1);
}

/// The recognizer admits a `bf16` program and mints its own vocabulary.
///
/// **The wall this replaces refused every program carrying a non-`f32`
/// value under `dtype-f32`, before a subject was normalized**, so no
/// `NormalizedProgram` for one could exist and nothing downstream could be
/// asked about it. Recognition now derives the program's one arithmetic type
/// and walks it with the same authority the `f32` walk uses — the same
/// classification, the same shape checks, the same leaf ordering — and only
/// the minting differs.
///
/// The expression is asserted whole rather than by node count alone: the
/// constant leaf carries the *sixteen* declared payload bits, which is the
/// one place a widened `f32` reading would show up as a number no `bf16`
/// program stated.
#[test]
fn a_bf16_program_is_recognized_in_its_own_expression_vocabulary() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    // `3.0` in bf16, whose sixteen bits are not the low half of any binary32
    // pattern this walk could have read instead.
    let scale = Bf16Constant::apply(&mut builder, 0x4040).unwrap();
    let scaled = Bf16Multiply::apply(&mut builder, input, scale).unwrap();
    let bias = Bf16Constant::apply(&mut builder, 0x8000).unwrap();
    let root = Bf16Add::apply(&mut builder, scaled, bias).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), root)
        .unwrap();
    let program = builder.build().unwrap();

    let NormalizedOutput::Pointwise(recognized) =
        recognize(&program).expect("a bf16 elementwise program is recognized")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    let expression = recognized.expression.bf16();
    assert_eq!(expression.input_count(), 1);
    // The population, counted: every occurrence the program declares is
    // claimed, so an assertion about the expression is an assertion about
    // the whole program rather than about a prefix of it.
    assert_eq!(recognized.members.len(), program.operation_count());
    assert_eq!(
        expression.nodes().len(),
        5,
        "one input leaf, two constants, the multiply, and the add",
    );
    let constants: Vec<u16> = expression
        .nodes()
        .iter()
        .filter_map(|node| match node {
            tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
            _ => None,
        })
        .collect();
    assert_eq!(
        constants,
        [0x4040, 0x8000],
        "the constants are the declared bf16 payloads, not a widened reading",
    );
    assert_eq!(
        recognized.reads,
        vec![(DeclaredInputOrdinal::new(0), LogicalAccess::LinearIdentity)],
        "one dense read of the one declared input",
    );
}

/// Constant occurrence identity reaches the initial recognizer and mint.
///
/// Each pair computes `x * 2 + 2` in its own arithmetic. The only authored
/// difference is whether the add reuses the exact constant value consumed by
/// the multiply or consumes a second constant occurrence with the same
/// payload. Semantic construction, elementwise planning, and minting all
/// preserve that difference for both arithmetic widths the compiler
/// currently recognizes. This drives `recognize` directly: ordinary
/// compilation normalizes equal pure constants before candidate readmission,
/// as the normalization and pipeline regressions assert separately.
///
#[test]
fn equal_constant_occurrences_remain_distinct_through_initial_recognition() {
    fn f32_program(repeat_occurrence: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let scaled = F32Multiply::apply(&mut builder, input, two).unwrap();
        let addend = if repeat_occurrence {
            F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap()
        } else {
            two
        };
        let root = F32Add::apply(&mut builder, scaled, addend).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    fn bf16_program(repeat_occurrence: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let two = Bf16Constant::apply(&mut builder, 0x4000).unwrap();
        let scaled = Bf16Multiply::apply(&mut builder, input, two).unwrap();
        let addend = if repeat_occurrence {
            Bf16Constant::apply(&mut builder, 0x4000).unwrap()
        } else {
            two
        };
        let root = Bf16Add::apply(&mut builder, scaled, addend).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    fn recognized_pointwise(program: &SemanticProgram) -> RecognizedPointwise {
        let NormalizedOutput::Pointwise(recognized) =
            recognize(program).expect("the compiler recognizes the elementwise program")
        else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        assert_eq!(
            recognized.members.len(),
            program.operation_count(),
            "the expression must cover every semantic occurrence",
        );
        recognized.expression
    }

    let shared_f32 = f32_program(false);
    let repeated_f32 = f32_program(true);
    assert_eq!(shared_f32.operation_count(), 3);
    assert_eq!(repeated_f32.operation_count(), 4);
    let RecognizedPointwise::F32(shared_f32_expression) = recognized_pointwise(&shared_f32) else {
        panic!("an f32 program must mint the f32 pointwise vocabulary");
    };
    let RecognizedPointwise::F32(repeated_f32_expression) = recognized_pointwise(&repeated_f32)
    else {
        panic!("an f32 program must mint the f32 pointwise vocabulary");
    };
    assert_eq!(shared_f32_expression.nodes().len(), 4);
    assert_eq!(repeated_f32_expression.nodes().len(), 5);
    assert_eq!(
        shared_f32_expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                PointwiseF32Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [2.0_f32.to_bits()],
    );
    assert_eq!(
        repeated_f32_expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                PointwiseF32Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [2.0_f32.to_bits(), 2.0_f32.to_bits()],
        "the extra node is a second equal-payload constant occurrence",
    );
    assert_ne!(shared_f32_expression, repeated_f32_expression);

    let shared_bf16 = bf16_program(false);
    let repeated_bf16 = bf16_program(true);
    assert_eq!(shared_bf16.operation_count(), 3);
    assert_eq!(repeated_bf16.operation_count(), 4);
    let RecognizedPointwise::Bf16(shared_bf16_expression) = recognized_pointwise(&shared_bf16)
    else {
        panic!("a bf16 program must mint the bf16 pointwise vocabulary");
    };
    let RecognizedPointwise::Bf16(repeated_bf16_expression) = recognized_pointwise(&repeated_bf16)
    else {
        panic!("a bf16 program must mint the bf16 pointwise vocabulary");
    };
    assert_eq!(shared_bf16_expression.nodes().len(), 4);
    assert_eq!(repeated_bf16_expression.nodes().len(), 5);
    assert_eq!(
        shared_bf16_expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [0x4000],
    );
    assert_eq!(
        repeated_bf16_expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [0x4000, 0x4000],
        "the extra node is a second equal-payload constant occurrence",
    );
    assert_ne!(shared_bf16_expression, repeated_bf16_expression);

    let VerifiedRequest::Refused(refusals) = verify_request(CompilationRequest::governed_under(
        &repeated_bf16,
        crate::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16.resolve(),
    ))
    .expect("the governed target refusal is a target-local outcome") else {
        panic!("the governed target declares no bf16 dispatch row");
    };
    let [refusal] = refusals.as_slice() else {
        panic!("the governed request carries one target and one refusal");
    };
    let VerifiedTargetResolution::Rejected(refusal) = &refusal.resolution else {
        panic!("the governed target slot is refused");
    };
    assert_eq!(
        *refusal,
        RequestError::DTypeNotDispatchable {
            target_profile: TargetProfile::governed().profile_key().clone(),
            resolved_type: Box::new(Bf16::resolved_type()),
            disposition: DTypeDispatchRefusalDisposition::Unknown,
        },
        "the governed request stops at dtype dispatch before target-specific recognition",
    );
}

/// The two refusals the `dtype-f32` rule split into name different findings.
///
/// **`dtype-recognized` and `dtype-uniform` are not one rule renamed.** The
/// first says this build states no per-point vocabulary for a width the
/// program uses; the second says the program uses two widths at once, which
/// no single scheduled region can carry however well each width is
/// supported. Each is exercised by a program that fails only it, and the
/// admitted neighbours above are what keep the pair from passing for a
/// recognizer that refused everything.
#[test]
fn a_mixed_width_program_and_an_unspelled_width_refuse_by_different_names() {
    // Two recognized widths in one program: the quantized carrier is `bf16`
    // and its declared sibling is `f32`, so no one arithmetic governs it.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let narrow = builder
        .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let wide = builder
        .input::<F32>(InputKey::new("y").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let narrow_sum = Bf16Add::apply(&mut builder, narrow, narrow).unwrap();
    let wide_sum = F32Add::apply(&mut builder, wide, wide).unwrap();
    builder
        .output(OutputKey::new("narrow").unwrap(), narrow_sum)
        .unwrap();
    builder
        .output(OutputKey::new("wide").unwrap(), wide_sum)
        .unwrap();
    let mixed = builder.build().unwrap();
    assert_eq!(
        recognize(&mixed),
        Err("dtype-uniform"),
        "a program of two widths has no single scalar program",
    );

    // One width this build spells no per-point body in: the strict-affine
    // encoded carrier, a registered value type that names no arithmetic type
    // at all.
    let published = |program: &SemanticProgram| recognize(program);
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let codes = builder
        .input::<tiler_ir::semantic::StrictAffineU4>(
            InputKey::new("codes").unwrap(),
            Shape::from_dims([2, 3]),
        )
        .unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), codes)
        .unwrap();
    let encoded = builder.build().unwrap();
    assert_eq!(
        published(&encoded),
        Err("dtype-recognized"),
        "a value type this build states no per-point vocabulary for is named as such",
    );

    // The neighbour that attributes that refusal to the *width* rather than
    // to the shape: the same program in a recognized width publishes a
    // declared input, which is refused one rule later under `operation-set`.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let value = builder
        .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), value)
        .unwrap();
    let published_input = builder.build().unwrap();
    assert_eq!(
        published(&published_input),
        Err("operation-set"),
        "the shape alone refuses under its own rule, so the width is what the U4 program \
         was refused for",
    );
}

/// Every refusal names the exact property that was not recognized.
///
/// The table is the ticket's contract: recognition generalizes, admission
/// does not become silent. Each row is a program the boundary refuses, the
/// rule it refuses under, and — through the accepted neighbour built beside
/// it — a demonstration that the rule can say yes as well as no.
#[test]
fn every_refusal_names_its_unrecognized_property() {
    let shape = || Shape::from_dims([2, 3]);

    // `input-arity`: an all-constant graph has no output-reachable input,
    // and a frozen program drops the unused declaration. The neighbour is
    // the same expression with one leaf replaced by the declared tensor.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let _input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let first = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let second = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let root = F32Add::apply(&mut builder, first, second).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let all_constant = builder.build().unwrap();
    assert_eq!(all_constant.input_count(), 0);
    assert_eq!(recognize(&all_constant).unwrap_err(), "input-arity");

    // `output-partition-overlap`: two named outputs one walk would have to
    // publish, because the second names a value the first's walk consumes.
    // The neighbour is the same graph naming only the root, which recognizes
    // — so the rule reads the *sharing* rather than the second output. This
    // row replaced an `output-arity` row: the arity guard is gone, and what
    // refuses this program is the partition obligation it actually violates.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, constant).unwrap();
    let root = F32Add::apply(&mut builder, scaled, constant).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder
        .output(OutputKey::new("partial").unwrap(), scaled)
        .unwrap();
    let two_outputs = builder.build().unwrap();
    assert_eq!(two_outputs.output_count(), 2);
    assert_eq!(
        recognize(&two_outputs).unwrap_err(),
        "output-partition-overlap",
    );

    // **Admitted, and this row is the one the structural widening flipped.**
    // A transposition over a declared input becomes the *read map* of the
    // region that consumes it, so `tiler::reindex-f32@1` is recognized
    // rather than refused. The derived relation is asserted rather than
    // merely the admission: a recognizer that admitted the family and bound
    // a dense read would compile the wrong tensor, which is precisely the
    // failure a bare `is_ok()` here would not see.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let permuted = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
            .expect("a two-axis transposition is an admitted form"),
        input,
    )
    .expect("the standard registry admits the reindex family");
    builder
        .output(OutputKey::new("result").unwrap(), permuted)
        .unwrap();
    let structural = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&structural).expect("a transposition of a declared input is a mapped read")
    else {
        panic!("a reindex over a declared input is an elementwise region");
    };
    // `shape()` is `[2, 3]`, so the transposed result is `[3, 2]` with
    // suffix products `[2, 1]`. Operand axis 1 takes result axis 0's window
    // and operand axis 0 takes result axis 1's, which is the transposition
    // written as a decode per operand axis.
    assert_eq!(
        recognized.reads,
        vec![(
            DeclaredInputOrdinal::new(0),
            LogicalAccess::ReindexBijection {
                operand_shape: Shape::from_dims([2, 3]),
                result_shape: Shape::from_dims([3, 2]),
                axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 3)],
            },
        )],
    );

    // `structural-operand`: the family is admitted, and what is refused is a
    // structural occurrence over a *computed* value. The region binds one
    // read per declared input and has no access to bind an intermediate it
    // also produces, so this refuses by name rather than materializing the
    // intermediate — which would add exactly the observable rounding
    // boundary the family's admission excludes. It is the neighbour that
    // keeps the row above attributable: both are reindexes, and only the
    // operand differs.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let doubled = tiler_ir::semantic::F32Silu::apply(&mut builder, input)
        .expect("the standard registry admits the silu family");
    let permuted = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
            .expect("a two-axis transposition is an admitted form"),
        doubled,
    )
    .expect("the standard registry admits the reindex family");
    builder
        .output(OutputKey::new("result").unwrap(), permuted)
        .unwrap();
    let computed = builder.build().unwrap();
    assert_eq!(recognize(&computed).unwrap_err(), "structural-operand");

    // **Admitted, and this row moved here from the refusal inventory.** One
    // declared input read *both* densely and through a relation was refused
    // under `structural-access-conflict`, because the region bound one read
    // per declared input and the expression's two `Input { ordinal: 0 }`
    // nodes shared it — so the mapped relation served both leaves and
    // `a * permute(a)` over `[[1, 2], [4, 8]]` compiled to `[1, 16, 4, 64]`,
    // which is `permute(a) * permute(a)`, where the reference evaluator
    // gives `[1, 8, 8, 64]`. The region now binds two reads of ordinal `0`,
    // and the read list is asserted rather than the admission: a recognizer
    // that admitted the program and bound one read would compile exactly the
    // wrong tensor that a bare `is_ok()` would not see.
    //
    // What still refuses is the pair with no canonical order between its two
    // members — two *structural* relations on one input — which is the
    // neighbour that keeps the admission attributable.
    let mixed = |second_dense: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let reindex = |builder: &mut SemanticProgramBuilder,
                       form: &tiler_ir::semantic::ReindexForm| {
            tiler_ir::semantic::F32Reindex::apply(builder, form, a)
                .expect("the standard registry admits the reindex family")
        };
        let transposed = reindex(
            &mut builder,
            &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
                .expect("a two-axis transposition is an admitted form"),
        );
        let second = if second_dense {
            a
        } else {
            reindex(
                &mut builder,
                &tiler_ir::semantic::ReindexForm::reverse_axis(Axis::new(0))
                    .expect("an axis reversal is an admitted form"),
            )
        };
        let root = F32Multiply::apply(&mut builder, second, transposed).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };
    let NormalizedOutput::Pointwise(recognized) = recognize(&mixed(true))
        .expect("one declared input may be read densely and through a relation")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    // The dense read leads and the mapped one follows, which is the pair's
    // canonical order and the only one the region verifier admits.
    assert_eq!(
        recognized.reads,
        vec![
            (DeclaredInputOrdinal::new(0), LogicalAccess::LinearIdentity),
            (
                DeclaredInputOrdinal::new(0),
                LogicalAccess::ReindexBijection {
                    operand_shape: Shape::from_dims([2, 2]),
                    result_shape: Shape::from_dims([2, 2]),
                    axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 2)],
                },
            ),
        ],
    );
    assert_eq!(recognized.expression.f32().input_count(), 2);
    assert_eq!(
        recognize(&mixed(false)).unwrap_err(),
        "structural-access-conflict",
    );

    // `structural-access-conflict` again, and this is the *other* half of the
    // widening's boundary: the twice-read tensor is the value an earlier
    // region staged rather than a declared input. What admits the pair above
    // is the ordinal saying which tensor each read binds, and
    // `TensorRole::Intermediate` carries none — so a second staged read has
    // nothing to attribute it to a second materialization edge. Its accepted
    // neighbour is `s * s`, which reads the staged value once and differs by
    // exactly the read that would have no attribution.
    let staged = |mapped: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let folded = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(1)]).unwrap();
        let second = if mapped {
            tiler_ir::semantic::F32Reindex::apply(
                &mut builder,
                &tiler_ir::semantic::ReindexForm::reverse_axis(Axis::new(0))
                    .expect("an axis reversal is an admitted form"),
                folded,
            )
            .expect("the standard registry admits the reindex family")
        } else {
            folded
        };
        let root = F32Multiply::apply(&mut builder, folded, second).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };
    assert!(matches!(
        recognize(&staged(false)),
        Ok(NormalizedOutput::Epilogue(_)),
    ));
    assert_eq!(
        recognize(&staged(true)).unwrap_err(),
        "structural-access-conflict",
    );

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let activated = tiler_ir::semantic::F32Silu::apply(&mut builder, input)
        .expect("the standard registry admits the silu family");
    builder
        .output(OutputKey::new("result").unwrap(), activated)
        .unwrap();
    let unary = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&unary).expect("the activation projects into the expression vocabulary")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    // One occurrence, one declared input read once, and the composition's
    // seven nodes: the projection is the shared body's, not a per-shape one.
    assert_eq!(
        recognized.members,
        vec![SemanticStage::first(SemanticMemberId(0))]
    );
    assert_eq!(recognized.expression.f32().input_count(), 1);
    assert_eq!(recognized.expression.f32().nodes().len(), 7);

    // A contraction with a reachable elementwise epilogue is a *chain*, not
    // a refusal, and the bare contraction beside it is what makes the
    // difference attributable: the two programs differ by exactly the
    // epilogue, and the recognized shape differs by exactly the consumer
    // region.
    let contraction = contraction_program(false);
    assert!(matches!(
        recognize(&contraction),
        Ok(NormalizedOutput::Contraction(_))
    ));
    let with_epilogue = contraction_program(true);
    let Ok(NormalizedOutput::Epilogue(chain)) = recognize(&with_epilogue) else {
        panic!("an elementwise expression over a contraction result is a chain");
    };
    assert!(matches!(*chain.producer, NormalizedOutput::Contraction(_)));
    assert_eq!(
        chain.reads.len(),
        1,
        "the epilogue reads only the staged value"
    );
    assert_eq!(chain.reads[0].0, BoundaryRead::Staged);

    // The one side the discovery used to refuse: a fold whose *contributors*
    // cross a materialization boundary. The producer was already recognized;
    // what was missing was a place on `NormalizedSerialSum` to retain it, and
    // the contributor source is that place — so `sum(sum(x) * 2)` is now the
    // admitted subject rather than the wall.
    //
    // The declared-input neighbour is the same fold over the same scaling of
    // the *declared input*, so the difference between them is exactly where the
    // scaled value comes from — which is what the contributor source names.
    let folded_prologue = |nested: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let (contributors, axis) = if nested {
            let inner = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
            (
                F32Multiply::apply(&mut builder, inner, scale).unwrap(),
                Axis::new(0),
            )
        } else {
            (
                F32Multiply::apply(&mut builder, input, scale).unwrap(),
                Axis::new(1),
            )
        };
        let outer = StrictSerialF32Sum::apply(&mut builder, contributors, [axis]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), outer)
            .unwrap();
        builder.build().unwrap()
    };
    let Ok(NormalizedOutput::SerialSum(declared)) = recognize(&folded_prologue(false)) else {
        panic!("a fold over a pointwise prologue is recognized as a serial sum");
    };
    assert!(matches!(
        declared.contributor,
        SerialSumContributor::PointwisePrologue { .. }
    ));
    let Ok(NormalizedOutput::SerialSum(produced)) = recognize(&folded_prologue(true)) else {
        panic!("a fold over a materialized producer is recognized as a serial sum");
    };
    let SerialSumContributor::Materialized(materialized) = &produced.contributor else {
        panic!("a fold over a nested reduction names a materialized contributor");
    };
    assert!(matches!(
        materialized.producer,
        NormalizedOutput::SerialSum(_)
    ));
    let continuation = materialized
        .continuation
        .as_ref()
        .expect("the `* 2` between the producer and the fold is a continuation");
    assert_eq!(
        continuation
            .reads
            .iter()
            .filter(|(read, _)| *read == BoundaryRead::Staged)
            .count(),
        1,
        "the continuation reads exactly the value the producer staged",
    );

    // The source names the contributor relation rather than the producer
    // family: a staged family reaches the same arm a nested reduction does,
    // and — because the fold's contributor *is* the produced value — with no
    // continuation rather than a synthesized identity one.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let value = builder
        .input::<F32>(InputKey::new("value").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("weight").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let normalized = F32RmsNorm::apply(
        &mut builder,
        value,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, normalized, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), reduced)
        .unwrap();
    let staged_contributor = builder.build().unwrap();
    let Ok(NormalizedOutput::SerialSum(staged)) = recognize(&staged_contributor) else {
        panic!("a fold over a staged family is recognized as a serial sum");
    };
    let SerialSumContributor::Materialized(materialized) = &staged.contributor else {
        panic!("a fold over a staged family names a materialized contributor");
    };
    assert!(matches!(materialized.producer, NormalizedOutput::Staged(_)));
    assert_eq!(
        materialized.continuation, None,
        "the fold's contributor *is* the produced value, so no region stands between them",
    );

    // `reduction-contributor-depth`: the same shape one materialization
    // boundary deeper, where the fold's producer is itself a fold across an
    // edge. The rule names how deep the chain runs rather than a carrier the
    // normal form lacks, and it is the sides rule that reports it — the
    // producer is recognized through `recognize_epilogue_producer`, which
    // hands `NoEdge`.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2, 2]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let inner = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(2)]).unwrap();
    let scaled = F32Multiply::apply(&mut builder, inner, scale).unwrap();
    let middle = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
    let rescaled = F32Multiply::apply(&mut builder, middle, scale).unwrap();
    let outer = StrictSerialF32Sum::apply(&mut builder, rescaled, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), outer)
        .unwrap();
    let too_deep = builder.build().unwrap();
    assert_eq!(
        recognize(&too_deep).unwrap_err(),
        "reduction-contributor-depth"
    );

    // Width, not depth, and it keeps its own rule: a contributor walk reaching
    // a *second, different* materialized value has nothing to say which edge
    // each read binds, so it reports `operation-set` after the retain rather
    // than taking the first fold and dropping the second.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let first = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let second = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let left = StrictSerialF32Sum::apply(&mut builder, first, [Axis::new(1)]).unwrap();
    let right = StrictSerialF32Sum::apply(&mut builder, second, [Axis::new(1)]).unwrap();
    let paired = F32Multiply::apply(&mut builder, left, right).unwrap();
    let folded = StrictSerialF32Sum::apply(&mut builder, paired, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), folded)
        .unwrap();
    let two_edges = builder.build().unwrap();
    assert_eq!(recognize(&two_edges).unwrap_err(), "operation-set");
}

/// Two ordered named outputs whose producers share no occurrence.
///
/// `product = a * b` and `sum = a + b` over the same two declared inputs.
/// The independence is the point: neither output's walk reaches the other's
/// producer, which is exactly the branch the superseded single-output
/// recognition refused under `operation-set` — one walk covered one of the
/// two operations and the program had two.
fn independent_two_output_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let first = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let second = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, first, second).unwrap();
    let sum = F32Add::apply(&mut builder, first, second).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// Recognition names one implementable region partition per ordered output.
///
/// **The wall this ticket was filed for, observed gone.** The recognition
/// used to read one output, classify it, and require that one walk to cover
/// the program; a second declared output outside the walk therefore refused
/// under `operation-set`, which is what the measurement at `3adc0689`
/// recorded when both arity guards were relaxed. The same program now
/// recognizes into two partitions, each naming its own output key,
/// expression, and members — and the members are disjoint, which is what
/// makes each one a region a cover can place without two regions claiming
/// one occurrence.
///
/// The whole boundary is asserted beside the walk, because the two together
/// are what the claim needs: the same program recognizes into two partitions
/// *and* clears [`select_supported_strategy`], which used to refuse it under
/// `output-arity` before any occurrence was classified. That guard is gone,
/// so the two derivations now agree rather than contradicting each other.
#[test]
fn recognizing_several_ordered_named_outputs_names_one_partition_each() {
    let program = independent_two_output_program();
    assert_eq!(program.output_count(), 2);
    assert_eq!(program.operation_count(), 2);

    let recognized = recognize_outputs(&program).expect("both outputs are recognized");
    let [product, sum] = recognized.outputs() else {
        panic!("one recognized partition per declared output, in declaration order");
    };
    let product = product
        .pointwise()
        .expect("a multiply is an elementwise output");
    let sum = sum.pointwise().expect("an add is an elementwise output");
    assert_eq!(product.output_key, OutputKey::new("product").unwrap());
    assert_eq!(sum.output_key, OutputKey::new("sum").unwrap());
    // Each walk claims exactly its own producer, and the two sets are
    // disjoint: together they partition the program's occurrences.
    assert_eq!(product.members.len(), 1);
    assert_eq!(sum.members.len(), 1);
    assert_ne!(product.members, sum.members);
    assert_eq!(recognized.all_members().len(), program.operation_count());
    // Two different binary32 functions over the same two reads, so the
    // partitions are distinguished by what they compute and not only by
    // which occurrence they name.
    assert_ne!(product.expression, sum.expression);

    // The same recognition reached through the ordinary boundary, which is
    // where the arity guard stood. Compared by the same fields rather than
    // by whole-value equality, for the reason
    // `two_programs_differing_only_in_output_order_recognize_differently`
    // gives about `ValueId` carrying its graph.
    let admitted =
        select_supported_strategy(&program, &laws_of(&program)).expect("the boundary admits it");
    assert_eq!(
        admitted
            .outputs()
            .iter()
            .map(NormalizedOutput::members)
            .collect::<Vec<_>>(),
        vec![product.members.clone(), sum.members.clone()],
    );
}

/// A cover region resolves to the output whose partition owns it.
///
/// This is the lookup `crate::physical::spell_region` performs, exercised
/// on the one shape that can distinguish it from the whole-program question
/// it replaced: with two declared outputs, "which expression does this
/// region compute" has two answers and the members are what choose between
/// them. The straddling case is the one that must say no — a region covering
/// both outputs' occurrences computes two published results from one owning
/// write, and no scheduled region does that.
#[test]
fn a_region_resolves_to_the_output_whose_partition_owns_it() {
    let program = independent_two_output_program();
    let recognized = recognize_outputs(&program).expect("both outputs are recognized");
    let [first, second] = recognized.outputs() else {
        panic!("one recognized partition per declared output");
    };
    let first_members = first.members();
    let second_members = second.members();

    assert_eq!(
        recognized
            .output_for_region(&first_members)
            .map(|(at, _)| at),
        Some(0),
    );
    assert_eq!(
        recognized
            .output_for_region(&second_members)
            .map(|(at, _)| at),
        Some(1),
    );
    // The check can say no, in both of the ways a cover can get it wrong: a
    // region straddling the two partitions, and a region covering neither.
    let straddling = recognized.all_members();
    assert_eq!(straddling.len(), 2);
    assert!(recognized.output_for_region(&straddling).is_none());
    assert!(recognized.output_for_region(&[]).is_none());
}

/// The whole-program cover check was widened, not removed, and says no.
///
/// **Both arms are driven against a case that must fail.** The accepted
/// neighbour is the recognized two-output partition itself; each perturbation
/// takes exactly one property away from it.
///
/// *Removal-shaped.* Dropping one occurrence from a walk leaves an
/// occurrence no output claims, which is work the assembled program would
/// silently not compute. Removing the check rather than widening it is
/// exactly what would admit this, so the perturbation is the removal.
///
/// *Overlap-shaped.* Adding one walk's occurrence to another's makes the two
/// partitions claim it twice, which is the shape where one region's owning
/// write would have to serve both a materialization edge and a publication.
#[test]
fn the_output_partition_check_can_say_no_in_both_directions() {
    let program = independent_two_output_program();
    let recognized = recognize_outputs(&program).expect("both outputs are recognized");
    let outputs = recognized.outputs().to_vec();
    // The control: unperturbed, the walks partition the occurrences.
    assert_eq!(check_output_cover(&program, &outputs), Ok(()));

    let mut uncovered = outputs.clone();
    let NormalizedOutput::Pointwise(dropped) = &mut uncovered[1] else {
        panic!("the fixture's second output is elementwise");
    };
    dropped.members.clear();
    assert_eq!(
        check_output_cover(&program, &uncovered),
        mismatch("operation-set"),
        "an occurrence covered by no walk was admitted",
    );

    let mut overlapping = outputs.clone();
    let claimed = outputs[0].members();
    let NormalizedOutput::Pointwise(widened) = &mut overlapping[1] else {
        panic!("the fixture's second output is elementwise");
    };
    widened.members.extend_from_slice(&claimed);
    widened.members.sort_unstable();
    assert_eq!(
        check_output_cover(&program, &overlapping),
        mismatch("output-partition-overlap"),
        "one occurrence claimed by two walks was admitted",
    );
}

/// Two output keys naming one value still refuse under the partition rule.
///
/// **This is the neighbour of the admitted overlap, and it differs from it
/// by exactly the property [`published_and_consumed_overlap`] requires.**
/// Three shapes, each observed refusing under the partition rule rather than
/// being admitted and dropped a layer down:
///
/// - Two output keys naming *one* value. The two walks are equal rather than
///   one being a strict subset of the other, so there is no shorter walk to
///   publish and no boundary to publish at. Whichever region owns that
///   value's write publishes once, and
///   `tiler_ir::program::KernelProgramBuilder` refuses a second publication
///   of one buffer.
/// - A publication *inside* one recognized part. `product` is consumed by
///   the add that `biased` names, and a pointwise walk fusing the multiply
///   and the add has no region boundary between them — the subset is not a
///   *part*, which is the conjunct `owns_region_members` decides.
/// - A published value nothing outside the part reads. This one is stated
///   against [`published_and_consumed_overlap`] directly rather than as a
///   program, and that is a fact worth recording rather than a convenience:
///   for every program the recognizer admits, the value a part publishes
///   *is* the value crossing its boundary, so the conjunct is defence in
///   depth against a future recognizer rather than a live gate. Stating the
///   member sets is what makes it drivable at all.
///
/// Their admitted neighbour is the published-and-consumed program that
/// `crate::pipeline::conformance`'s
/// `a_published_and_consumed_intermediate_compiles_and_agrees` compiles,
/// which differs from each by exactly one of those conjuncts.
#[test]
fn an_output_key_pair_naming_one_value_still_refuses_by_name() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let first = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let second = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, first, second).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder
        .output(OutputKey::new("alias").unwrap(), product)
        .unwrap();
    let colliding = builder.build().unwrap();
    assert_eq!(colliding.output_count(), 2);
    assert_eq!(colliding.operation_count(), 1);
    assert_eq!(
        recognize_outputs(&colliding).unwrap_err(),
        "output-partition-overlap",
    );

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let other = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, input, other).unwrap();
    let biased = F32Add::apply(&mut builder, product, other).unwrap();
    builder
        .output(OutputKey::new("biased").unwrap(), biased)
        .unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    let mid_walk = builder.build().unwrap();
    assert_eq!(
        recognize_outputs(&mid_walk).unwrap_err(),
        "output-partition-overlap",
    );

    // The admitted neighbour, at this same boundary: `scaled` is a strict
    // subset of the fold's walk, is exactly its recognized prologue part,
    // and is the value the fold reads across the boundary.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("reduced").unwrap(), reduced)
        .unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    let published_and_consumed = builder.build().unwrap();
    let recognized = recognize_outputs(&published_and_consumed).expect("the overlap is admitted");
    let claimed: Vec<Vec<SemanticStage>> = recognized
        .outputs()
        .iter()
        .map(NormalizedOutput::members)
        .collect();
    assert_eq!(
        published_and_consumed_overlap(&published_and_consumed, recognized.outputs(), &claimed),
        Some((1, 0)),
    );

    // The crossing conjunct, driven against a stated member set: the shorter
    // walk is the fold's *reduction* part rather than its prologue part — a
    // part in its own right, and still a strict subset — but the value the
    // second output publishes is the multiply's, which no occurrence outside
    // that part reads. Every other conjunct is unchanged.
    let reduction_part = vec![claimed[0].last().copied().expect("the fold claims members")];
    assert_eq!(
        published_and_consumed_overlap(
            &published_and_consumed,
            recognized.outputs(),
            &[claimed[0].clone(), reduction_part],
        ),
        None,
    );
}

/// Two declared inputs and one expression naming both of the outer ones.
///
/// `product = a * c` and `doubled = b + b` over three declared `[2, 3]`
/// inputs. The first walk reads ordinals `0` and `2`, which is deliberately
/// not a prefix and not contiguous: a region-local renumbering would give
/// its two leaves reads `0` and `1` and the assembled program would multiply
/// `a * b`, and every other recognized fact would agree.
fn non_contiguous_subset_program(outer: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                .unwrap()
        })
        .collect();
    let (paired, doubled) = if outer { (2, 1) } else { (1, 2) };
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[paired]).unwrap();
    let sum = F32Add::apply(&mut builder, inputs[doubled], inputs[doubled]).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder
        .output(OutputKey::new("doubled").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

/// A walk reading a subset carries the program's ordinals, not its own.
///
/// **The read list is the map this ticket asked for.** `mint_elementwise`
/// numbers the expression's leaves by position in the canonical read order,
/// and the read at that position names the declared input ordinal it binds,
/// so `reads` *is* the leaf-ordinal-to-input-ordinal correspondence and
/// nothing further had to be carried. What changed is that it is no longer
/// the identity on `0..declared`.
///
/// The neighbour swaps which of the two later inputs each output reads, so
/// the recognized ordinals move with the program while the expression, the
/// declared keys, the domain, and the member sets all stay put — which is
/// what makes the assertion about the read list rather than about the
/// program being recognized at all.
#[test]
fn a_walk_reading_a_subset_carries_the_program_input_ordinals_it_reached() {
    for (outer, expected) in [(true, 2_u32), (false, 1)] {
        let program = non_contiguous_subset_program(outer);
        assert_eq!(program.input_count(), 3);
        let recognized = recognize_outputs(&program).expect("a subset walk is recognized");
        let [product, doubled] = recognized.outputs() else {
            panic!("the fixture declares two outputs");
        };
        let NormalizedOutput::Pointwise(product) = product else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        let NormalizedOutput::Pointwise(doubled) = doubled else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        // The declared interface stays whole: the ordinals index it, so a
        // region reading two of three inputs still resolves against all
        // three at assembly.
        assert_eq!(product.input_keys.len(), 3);
        assert_eq!(
            product.reads,
            vec![
                (DeclaredInputOrdinal::new(0), LogicalAccess::LinearIdentity),
                (
                    DeclaredInputOrdinal::new(expected),
                    LogicalAccess::LinearIdentity
                ),
            ],
        );
        assert_eq!(product.expression.f32().input_count(), 2);
        // The other output reads the remaining input at one leaf, twice.
        let other = if outer { 1 } else { 2 };
        assert_eq!(
            doubled.reads,
            vec![(
                DeclaredInputOrdinal::new(other),
                LogicalAccess::LinearIdentity
            )]
        );
        assert_eq!(doubled.expression.f32().input_count(), 1);
    }
}

/// A declared input no output reads is refused at program scope.
///
/// **The removal-shaped perturbation, and it has to be forged.** The
/// obligation `canonical_input_reads` used to state per walk moved to
/// [`check_output_cover`], and no program the public builder can construct
/// reaches it: a frozen program retains only output-reachable values, the
/// `operation-set` rule claims every retained occurrence for some walk, and
/// every way a walk consumes an operand records a read of it. So the check
/// is driven against a recognized program whose read list has had one entry
/// removed — which is exactly the state deleting the check would admit —
/// and its unforged neighbour is asserted to pass, so a check that refused
/// everything would fail here too.
#[test]
fn a_declared_input_no_output_reads_is_refused_at_program_scope() {
    let program = non_contiguous_subset_program(true);
    let recognized = recognize_outputs(&program).expect("a subset walk is recognized");
    assert_eq!(check_output_cover(&program, recognized.outputs()), Ok(()));

    let mut forged = recognized.clone();
    let NormalizedOutput::Pointwise(product) = &mut forged.outputs[0] else {
        panic!("the first declared output is elementwise");
    };
    product.reads.retain(|(ordinal, _)| *ordinal != 2);
    assert_eq!(
        check_output_cover(&program, &forged.outputs),
        mismatch("input-set"),
    );
}

/// A fold retains whichever declared input its contributor names.
///
/// The two programs have the same declaration, output families, shapes, and
/// operation order. The contributor ordinal is the relevant difference, and
/// it reaches both normalization and the output-subject bytes rather than
/// being renumbered to the fold region's only read.
#[test]
fn a_fold_over_a_later_declared_input_retains_its_ordinal() {
    let folded = |first: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let inputs: Vec<_> = ["a", "b"]
            .into_iter()
            .map(|key| {
                builder
                    .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                    .unwrap()
            })
            .collect();
        let (folded, doubled) = if first { (0, 1) } else { (1, 0) };
        let sum = StrictSerialF32Sum::apply(&mut builder, inputs[folded], [Axis::new(1)]).unwrap();
        let pair = F32Add::apply(&mut builder, inputs[doubled], inputs[doubled]).unwrap();
        builder
            .output(OutputKey::new("folded").unwrap(), sum)
            .unwrap();
        builder
            .output(OutputKey::new("doubled").unwrap(), pair)
            .unwrap();
        builder.build().unwrap()
    };
    let recognized = [
        recognize_outputs(&folded(true)).expect("a fold over input zero"),
        recognize_outputs(&folded(false)).expect("a fold over input one"),
    ];
    let mut encoded = Vec::new();
    for (ordinal, outputs) in recognized.iter().enumerate() {
        let [normalized, _] = outputs.outputs() else {
            panic!("the fixture declares two outputs");
        };
        let NormalizedOutput::SerialSum(fold) = normalized else {
            panic!("a reduction output recognizes as a serial sum");
        };
        assert_eq!(
            fold.contributor,
            SerialSumContributor::DeclaredInput(DeclaredInputOrdinal::new(
                u32::try_from(ordinal).unwrap()
            ))
        );

        let mut bytes = Vec::new();
        encode_output_subject(&mut bytes, &output_subject(normalized));
        encoded.push(bytes);
    }
    assert_ne!(encoded[0], encoded[1]);
}

/// The read run separates two subsets and leaves a complete one empty.
///
/// **Both halves of the sub-tag determination, driven at the encoder.** The
/// complete read list writes the framed zero it has always written, which is
/// the "no already-encodable subject's bytes move" half; the three
/// two-element subsets of three declared inputs write three different runs,
/// which is the injectivity half the marker exists for. Without the marker
/// all three would be that same framed zero, and one arm would encode three
/// programs.
#[test]
fn the_read_run_marks_unread_declared_inputs_and_leaves_a_complete_list_empty() {
    let dense = |ordinal| {
        (
            DeclaredInputOrdinal::new(ordinal),
            LogicalAccess::LinearIdentity,
        )
    };
    let run = |reads: &[(DeclaredInputOrdinal, LogicalAccess)]| {
        let mut bytes = Vec::new();
        encode_elementwise_reads(&mut bytes, 3, reads);
        bytes
    };
    // The framed zero every already-encodable subject wrote, byte for byte.
    assert_eq!(run(&[dense(0), dense(1), dense(2)]), vec![0_u8; 8]);
    // One marker, naming the ordinal no leaf read.
    let mut expected = vec![0_u8; 7];
    expected.push(1);
    expected.extend_from_slice(&1_u32.to_be_bytes());
    expected.push(UNREAD_DECLARED_INPUT_TAG);
    assert_eq!(run(&[dense(0), dense(2)]), expected);
    // The three subsets of the same size are three distinct runs, which is
    // the collision the marker closes.
    let subsets = [
        run(&[dense(0), dense(1)]),
        run(&[dense(0), dense(2)]),
        run(&[dense(1), dense(2)]),
    ];
    for (position, first) in subsets.iter().enumerate() {
        for second in &subsets[position + 1..] {
            assert_ne!(first, second);
        }
    }
}

/// Both claimants of a published-and-consumed part resolve to one region.
///
/// **This is the check behind the decided tie-break.**
/// [`NormalizedProgram::output_for_region`] scans in declaration order and
/// takes the first match, and the admitted overlap makes two outputs own one
/// member set — so "first" is only correct because the two claimants are
/// recognitions of one value over one occurrence set and therefore spell the
/// same region. That argument is worth less than a check that says no when
/// it stops holding, which is what this is: the same member set is resolved
/// against each claimant in turn, and the two regions the physical layer
/// builds from those resolutions are compared whole.
///
/// The two spellings are reached through different arms — the fold's
/// prologue part and the pointwise output's own walk — so an agreement here
/// is about the recognitions rather than about one code path being called
/// twice.
#[test]
fn both_claimants_of_a_published_and_consumed_part_spell_one_region() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("reduced").unwrap(), reduced)
        .unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    let program = builder.build().unwrap();
    let recognized = recognize_outputs(&program).expect("the overlap is admitted");
    let [fold, publication] = recognized.outputs() else {
        panic!("one recognized partition per declared output");
    };
    let shared = publication.members();

    // Both own it, which is the state the tie-break exists for.
    assert!(fold.owns_region_members(&shared));
    assert!(publication.owns_region_members(&shared));
    assert_eq!(
        recognized.output_for_region(&shared).map(|(at, _)| at),
        Some(0),
        "the first declared claimant is the one the scan returns",
    );

    // And they spell one region. Compared through the request the physical
    // layer actually reads, at the write the cover assigns a published-and-
    // consumed region.
    let request = verify_planned_request(CompilationRequest::governed(&program))
        .unwrap()
        .for_target(0)
        .unwrap();
    let staging = crate::physical::RegionWrite::MaterializedAndPublished;
    let (from_fold, fold_members) = crate::physical::pointwise_region(&request, fold, staging);
    let (from_publication, publication_members) =
        crate::physical::pointwise_region(&request, publication, staging);
    assert_eq!(from_fold, from_publication);
    assert_eq!(fold_members, publication_members);
    assert_eq!(fold_members, shared);
}

/// Output order reaches the recognized program, not only the semantic graph.
///
/// Two programs holding the same operations and the same two output keys,
/// differing only in which `output()` call came first, recognize into lists
/// that are unequal *and* unequal in order — the first entry of one is the
/// second entry of the other. The request subject encodes that list
/// length-framed in this order, so a permuted declaration cannot reach one
/// subject; the semantic half of the same claim is pinned in
/// `crates/tiler-compiler/tests/multi_output_boundary.rs`.
///
/// **The subject half is asserted here too, and it was not reachable until
/// `output-arity` was relaxed:** a subject is minted only for a request the
/// boundary admitted, and that guard admitted no two-output program at all.
/// Both orders now mint one, and the two subjects name their outputs in the
/// order their programs declared them.
///
/// **Measurement boundary, and it is a limit on what any test here can
/// claim.** The subject's *output list* is compared against the program's
/// declared keys, not its canonical bytes. The previous version of this
/// comment predicted the encoded form would become checkable once the guard
/// moved, and it does not: the subject folds the semantic graph identity,
/// output order is already part of that identity, and no two programs can
/// differ *only* in the recognized list — so two subjects' bytes differ
/// whatever the list order, observed by sorting the arms in
/// [`VerifiedRequestSubject::canonical_explain_subject_bytes`] and watching
/// the inequality still hold. A check that cannot say no is not evidence.
/// The list comparison is anchored to the declared keys for the same reason:
/// comparing the two subjects only to each other survives a list reversed
/// for both, which was also observed.
///
/// The recognized entries are compared by the fields the subject encodes
/// rather than by the whole recognized value, because a [`ValueId`] carries
/// the graph it was built in: two separately built programs never share one,
/// so whole-value equality would report a difference this test is not about
/// and would hold whatever the order.
#[test]
fn two_programs_differing_only_in_output_order_recognize_differently() {
    fn ordered(product_first: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let first = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let second = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, first, second).unwrap();
        let sum = F32Add::apply(&mut builder, first, second).unwrap();
        let product_key = OutputKey::new("product").unwrap();
        let sum_key = OutputKey::new("sum").unwrap();
        if product_first {
            builder.output(product_key, product).unwrap();
            builder.output(sum_key, sum).unwrap();
        } else {
            builder.output(sum_key, sum).unwrap();
            builder.output(product_key, product).unwrap();
        }
        builder.build().unwrap()
    }

    /// The per-output facts the request subject encodes, in list order.
    fn encoded(recognized: &NormalizedProgram) -> Vec<(OutputKey, Vec<SemanticStage>)> {
        recognized
            .outputs()
            .iter()
            .map(|output| {
                let pointwise = output.pointwise().expect("an elementwise output");
                (pointwise.output_key.clone(), pointwise.members.clone())
            })
            .collect()
    }

    let product_first = encoded(&recognize_outputs(&ordered(true)).expect("recognized"));
    let sum_first = encoded(&recognize_outputs(&ordered(false)).expect("recognized"));
    assert_ne!(
        product_first, sum_first,
        "output order must reach the recognized program, not only presentation",
    );
    assert_eq!(product_first[0], sum_first[1]);
    assert_eq!(product_first[1], sum_first[0]);
    // The check can say no: re-declaring the same order reproduces the
    // recognition, so the inequality above is about the order and not about
    // rebuilding the program.
    assert_eq!(
        product_first,
        encoded(&recognize_outputs(&ordered(true)).expect("recognized")),
    );

    // The same claim about the *subject*, minted through the ordinary
    // boundary rather than from the walk alone, and anchored to the
    // program's own declared order rather than only to the other subject.
    // Comparing the two subjects to each other is not enough: a subject list
    // reversed for *both* programs still swaps entry for entry, so that
    // relation holds while the interface is backwards. The declared keys are
    // the fixed point a reversal moves away from.
    for product_first in [true, false] {
        let program = ordered(product_first);
        let declared: Vec<OutputKey> = program
            .outputs()
            .map(|output| output.key().clone())
            .collect();
        let request = verify_planned_request(CompilationRequest::governed(&program))
            .expect("the boundary admits an ordered two-output program");
        let request = request.for_target(0).expect("one governed target");
        let subject: Vec<OutputKey> = request
            .subject()
            .normalized()
            .outputs()
            .iter()
            .map(|output| match output {
                NormalizedOutputSubject::Pointwise(normalized) => normalized.output_key.clone(),
                _ => panic!("both outputs of the fixture are elementwise"),
            })
            .collect();
        assert_eq!(
            subject, declared,
            "the request subject does not name the outputs in declaration order",
        );
    }
}

/// Builds a binary contraction, optionally with an elementwise epilogue.
fn contraction_program(epilogue: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let left = builder
        .input::<F32>(InputKey::new("left").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let right = builder
        .input::<F32>(InputKey::new("right").unwrap(), Shape::from_dims([3, 4]))
        .unwrap();
    // `ab,bc->ac`: the ordinary matrix product, stated as the index
    // structure the operation's identity is.
    let structure = ContractionIndexStructure::new(
        [
            vec![ContractionIndex::new(0), ContractionIndex::new(1)],
            vec![ContractionIndex::new(1), ContractionIndex::new(2)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .expect("ab,bc->ac is an admitted structure");
    let product =
        tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, left, right)
            .unwrap();
    let root = if epilogue {
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        F32Multiply::apply(&mut builder, product, scale).unwrap()
    } else {
        product
    };
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// A contraction over one of the three two-input subsets of one declaration.
///
/// The independent output retains the skipped input without entering the
/// contraction walk. All input shapes and occurrence positions are equal
/// across fixtures, so the read ordinals are the only contraction-subject
/// field that changes.
fn contraction_subset_program(pair: [usize; 2]) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), shape.clone())
                .unwrap()
        })
        .collect();
    let structure = ContractionIndexStructure::new(
        [
            vec![ContractionIndex::new(0), ContractionIndex::new(1)],
            vec![ContractionIndex::new(1), ContractionIndex::new(2)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .expect("ab,bc->ac is an admitted structure");
    let product = tiler_ir::semantic::F32TensorContraction::apply(
        &mut builder,
        &structure,
        inputs[pair[0]],
        inputs[pair[1]],
    )
    .unwrap();
    let skipped = (0..3)
        .find(|ordinal| !pair.contains(ordinal))
        .expect("two of three inputs leave one skipped");
    let retained = F32Add::apply(&mut builder, inputs[skipped], inputs[skipped]).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder
        .output(OutputKey::new("retained").unwrap(), retained)
        .unwrap();
    builder.build().unwrap()
}

/// The three subsets are distinguished by the contraction arm itself.
///
/// This drives [`encode_output_subject`] directly, excluding the enclosing
/// semantic graph identity that would distinguish separately built programs
/// whatever this arm encoded. It also pins both read predicates for the
/// skipped ordinal, so restoring dense indexing or a declaration-length
/// predicate makes the first non-prefix subset fail independently.
#[test]
fn contraction_subjects_separate_all_two_input_subsets_of_three_declarations() {
    let pairs = [[0_u32, 1_u32], [0, 2], [1, 2]];
    let mut subjects = Vec::new();
    for pair in pairs {
        let program = contraction_subset_program([
            usize::try_from(pair[0]).unwrap(),
            usize::try_from(pair[1]).unwrap(),
        ]);
        let recognized = recognize_outputs(&program).expect("both outputs are recognized");
        let NormalizedOutput::Contraction(contraction) = &recognized.outputs()[0] else {
            panic!("the first output is the contraction");
        };
        assert_eq!(contraction.input_keys.len(), 3);
        assert_eq!(
            contraction
                .reads
                .iter()
                .map(|read| read.input_ordinal)
                .collect::<Vec<_>>(),
            pair,
        );
        let skipped = (0..3).find(|ordinal| !pair.contains(ordinal)).unwrap();
        for ordinal in pair {
            assert!(
                recognized.outputs()[0].reads_declared_input(DeclaredInputOrdinal::new(ordinal))
            );
            assert_eq!(
                recognized.outputs()[0].input_elements_at(DeclaredInputOrdinal::new(ordinal)),
                Some(4),
            );
        }
        assert!(!recognized.outputs()[0].reads_declared_input(DeclaredInputOrdinal::new(skipped)));
        assert_eq!(
            recognized.outputs()[0].input_elements_at(DeclaredInputOrdinal::new(skipped)),
            None,
        );

        let mut bytes = Vec::new();
        encode_output_subject(&mut bytes, &output_subject(&recognized.outputs()[0]));
        subjects.push(bytes);
    }
    for (position, first) in subjects.iter().enumerate() {
        for second in &subjects[position + 1..] {
            assert!(first != second, "two declared-input subsets collided");
        }
    }
}

/// The conditional ordinal run does not move an old contraction subject.
///
/// The helper is the exact pre-widening `contraction-f32.v1` arm, projected
/// through the new read records. Equality therefore checks every byte of an
/// already-admitted two-declaration subject, not merely its tag or digest.
#[test]
fn a_two_declaration_contraction_keeps_its_v1_subject_bytes() {
    let program = contraction_program(false);
    let recognized = recognize(&program).expect("the contraction is recognized");
    let NormalizedOutput::Contraction(normalized) = &recognized else {
        panic!("the output is a contraction");
    };
    assert_eq!(
        normalized
            .reads
            .iter()
            .map(|read| read.input_ordinal)
            .collect::<Vec<_>>(),
        [0, 1],
    );

    let mut legacy = Vec::new();
    push_slice(&mut legacy, b"contraction-f32.v1");
    push_len(&mut legacy, normalized.input_keys.len());
    for key in &normalized.input_keys {
        push_slice(&mut legacy, key.as_str().as_bytes());
    }
    push_slice(&mut legacy, normalized.output_key.as_str().as_bytes());
    for read in &normalized.reads {
        encode_explain_shape(&mut legacy, &read.shape);
    }
    encode_explain_shape(&mut legacy, &normalized.output_shape);
    encode_explain_shape(&mut legacy, &normalized.contracted_shape);
    push_slice(
        &mut legacy,
        normalized.structure.canonical_encoding().as_bytes(),
    );
    for read in &normalized.reads {
        push_len(&mut legacy, read.operand_position);
    }
    push_len(&mut legacy, normalized.members.len());
    for atom in &normalized.members {
        legacy.extend_from_slice(&atom.member().0.to_be_bytes());
    }
    for read in &normalized.reads {
        legacy.extend_from_slice(&read.elements.to_be_bytes());
    }
    legacy.extend_from_slice(&normalized.output_elements.to_be_bytes());
    legacy.extend_from_slice(&normalized.contracted_elements.to_be_bytes());

    let mut current = Vec::new();
    encode_output_subject(&mut current, &output_subject(&recognized));
    assert_eq!(current, legacy, "an existing v1 subject moved bytes");
}

#[test]
fn invalid_pointwise_arity_shape_and_dtype_fail_at_semantic_admission() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let tensor = builder
        .input::<F32>(InputKey::new("tensor").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    assert!(
        builder
            .apply(
                add_f32_op(),
                OperationAttributes::empty(),
                &[tensor.erase()],
            )
            .is_err(),
        "the semantic schema refuses invalid builtin arity before normalization",
    );

    let other_shape = builder
        .input::<F32>(InputKey::new("other").unwrap(), Shape::from_dims([3, 2]))
        .unwrap();
    assert!(
        F32Add::apply(&mut builder, tensor, other_shape).is_err(),
        "the semantic inferencer refuses incompatible shapes before normalization",
    );

    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision: 1 })
        .unwrap();
    let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
    let foreign = builder
        .input_resolved(
            InputKey::new("foreign").unwrap(),
            Shape::from_dims([2, 3]),
            ResolvedValueType::nominal(TypeKey::new("tiler-test", "unused", 1).unwrap()),
        )
        .unwrap();
    let scalar = F32Constant::apply(&mut builder, 1.0_f32.to_bits())
        .unwrap()
        .erase();
    assert!(
        builder
            .apply(
                add_f32_op(),
                OperationAttributes::empty(),
                &[foreign, scalar],
            )
            .is_err(),
        "the semantic authority refuses a non-f32 builtin operand before normalization",
    );
}

#[test]
fn program_dispatch_types_are_exact_canonical_and_unique() {
    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision: 1 })
        .unwrap();
    let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
    let f32 = builder
        .input::<F32>(InputKey::new("f32").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scalar = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let foreign_type = ResolvedValueType::nominal(TypeKey::new("tiler-test", "unused", 1).unwrap());
    let foreign = builder
        .input_resolved(
            InputKey::new("foreign").unwrap(),
            Shape::from_dims([2, 3]),
            foreign_type.clone(),
        )
        .unwrap();
    builder
        .output(OutputKey::new("f32-output").unwrap(), f32)
        .unwrap();
    builder
        .output(OutputKey::new("scalar-output").unwrap(), scalar)
        .unwrap();
    builder
        .output_resolved(OutputKey::new("foreign-output").unwrap(), foreign)
        .unwrap();
    let program = builder.build().unwrap();

    let actual = canonical_program_value_types(&program);
    assert_eq!(actual.len(), 2, "repeated F32 values are deduplicated");
    assert!(actual.contains(&F32::resolved_type()));
    assert!(actual.contains(&foreign_type));
    assert!(actual.windows(2).all(|pair| {
        pair[0].canonical_encoding().as_bytes() < pair[1].canonical_encoding().as_bytes()
    }));
}

#[test]
fn request_rejects_profile_and_budget_mismatches_stably() {
    let program = program();
    let mut request = CompilationRequest::governed(&program);
    request.budgets.semantic_operations = 4;
    assert_eq!(
        verify_planned_request(request),
        Err(RequestError::BudgetExceeded {
            resource: BudgetResource::SemanticOperations,
            limit: 4,
            reported: 5,
        })
    );

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), input)
        .unwrap();
    let invalid = builder.build().unwrap();
    assert_eq!(
        verify_planned_request(CompilationRequest::governed(&invalid)),
        Err(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "operation-set",
        })
    );
}

/// Builds a program declaring exactly `inputs` inputs and `outputs` ordered
/// named outputs over `operations` occurrences, so a budget's `reported` value
/// can be placed on either side of its bound.
///
/// Every occurrence is one `f32` add producing one value, so
/// `value_count() == inputs + operations`. That is the same identity the
/// decoder layer has — no occurrence in it produces more than one value —
/// and it is the identity `semantic_values` is sized against. The chain
/// consumes every declared input before it starts re-reading the last, so no
/// declared input is left unreached.
///
/// The outputs are the chain's last `outputs` accumulator values, so the
/// output arity moves without moving any of the other three counts: that
/// independence is what lets a probe exceed exactly one of the five bounds.
fn budget_probe(inputs: usize, operations: usize, outputs: usize) -> SemanticProgram {
    assert!(inputs >= 2, "the chain's first add needs two operands");
    assert!(
        operations >= inputs - 1,
        "fewer adds than inputs would leave a declared input unreached",
    );
    assert!(
        (1..=operations).contains(&outputs),
        "each declared output publishes one of the chain's own results",
    );
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let declared: Vec<_> = (0..inputs)
        .map(|index| {
            builder
                .input::<F32>(
                    InputKey::new(format!("input{index}")).unwrap(),
                    Shape::from_dims([2, 3]),
                )
                .unwrap()
        })
        .collect();
    let mut accumulator = declared[0];
    let mut results = Vec::with_capacity(operations);
    for step in 0..operations {
        let operand = declared[(step + 1).min(inputs - 1)];
        accumulator = F32Add::apply(&mut builder, accumulator, operand).unwrap();
        results.push(accumulator);
    }
    for (ordinal, result) in results[operations - outputs..].iter().enumerate() {
        builder
            .output(OutputKey::new(format!("result{ordinal}")).unwrap(), *result)
            .unwrap();
    }
    let program = builder.build().unwrap();
    assert_eq!(program.input_count(), inputs);
    assert_eq!(program.operation_count(), operations);
    assert_eq!(program.output_count(), outputs);
    assert_eq!(program.value_count(), inputs + operations);
    program
}

/// Each widened budget refuses the program one step past it, and the
/// decoder layer's own measured counts are admitted.
///
/// The five program-scoped bounds are sized to that layer, so the admitted
/// neighbours are its two measured rows exactly — eighteen declared inputs
/// and three ordered named outputs over sixty-two occurrences and eighty
/// values at the decode row, and over fifty-eight and seventy-six at the
/// prefill row — and the decode row sits *on* all five bounds rather than
/// under them.
///
/// Refusals are observed through [`verify_program`], which is the entry the
/// budgets guard; admission is observed at [`check_program_budgets`],
/// because clearing the budget gate is the whole of what a budget can
/// promise. `verify_program` still refuses the layer's *shape* at the
/// recognizer under a rule this widening deliberately does not touch, so an
/// admitted probe here is evidence about size and about nothing else.
/// Every budget resource carries its own stable key.
///
/// A duplicate would make two budgets indistinguishable everywhere the key
/// is what travels — the rule key of a request refusal, the resource key of
/// an explain record, the reason code of a failure detail — so a caller told
/// which budget refused would be told the wrong one, silently.
///
/// The population is sized by `variant_count` rather than written out, so a
/// budget added to the vocabulary and not to `ALL` fails the build here
/// rather than shrinking the set this test checks while it still reports no
/// duplicate. The census is printed for the same reason: "nothing ran" must
/// not be able to look green.
#[test]
fn every_budget_resource_key_is_distinct() {
    let keys: BTreeSet<&'static str> = BudgetResource::ALL
        .iter()
        .map(|resource| resource.key())
        .collect();
    assert_eq!(
        keys.len(),
        BudgetResource::ALL.len(),
        "two budget resources share a stable key: {keys:?}",
    );
    assert_eq!(
        BudgetResource::ALL.len(),
        15,
        "the vocabulary changed size; every dependent claim about it needs re-reading",
    );
}

/// The three internal stop vocabularies map onto the shared one injectively.
///
/// Each `resource()` is exhaustive, so `rustc` already proves it total. What
/// it cannot prove is that two internal budgets do not land on one public
/// row, which would report a region stop as a cover stop or the reverse.
///
/// [`crate::cover::CoverBudgetResource::Refusals`] is deliberately absent
/// from the image: it refuses no compilation, and its `None` is what keeps
/// that exclusion typed rather than an inequality at the consuming site.
#[test]
fn the_stop_vocabularies_map_onto_distinct_shared_resources() {
    let region = [
        crate::region::RegionBudgetResource::Members,
        crate::region::RegionBudgetResource::BoundaryOutputs,
        crate::region::RegionBudgetResource::LiveValues,
        crate::region::RegionBudgetResource::CandidatesPerSeed,
        crate::region::RegionBudgetResource::Expansions,
    ];
    let mut image: Vec<BudgetResource> = region.iter().map(|stop| stop.resource()).collect();
    image.extend(
        [
            crate::cover::CoverBudgetResource::Covers,
            crate::cover::CoverBudgetResource::Expansions,
            crate::cover::CoverBudgetResource::Refusals,
        ]
        .iter()
        .filter_map(|stop| stop.truncating_resource()),
    );
    image.push(crate::selection::PlanBudgetResource::Combinations.resource());

    let distinct: BTreeSet<BudgetResource> = image.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        image.len(),
        "two stops share one row: {image:?}"
    );
    assert_eq!(
        image.len(),
        8,
        "five region stops, two cover stops, one plan stop"
    );
    assert!(
        crate::cover::CoverBudgetResource::Refusals
            .truncating_resource()
            .is_none(),
        "the explanation budget refuses no compilation and holds no row",
    );

    // Every one of the eight is a search or shape stop reached after a
    // target is consulted. The five program-scoped and two report-only
    // explain rows are exactly the ones no stop vocabulary maps onto.
    for resource in BudgetResource::ALL {
        let outside_stop_vocabularies = matches!(
            resource,
            BudgetResource::SemanticValues
                | BudgetResource::SemanticOperations
                | BudgetResource::Regions
                | BudgetResource::HostExpressionNodes
                | BudgetResource::Buffers
                | BudgetResource::ExplainDetailRecords
                | BudgetResource::ExplainDetailCanonicalBytes
        );
        assert_eq!(
            outside_stop_vocabularies,
            !distinct.contains(&resource),
            "{resource:?} is claimed by both a stop vocabulary and another refusal authority",
        );
    }
}

/// Every resource reports exactly one of the four provenances, and the
/// population is sized from the type.
///
/// Categories are defined by how the compared number was produced, not by
/// whether it can be described abstractly as a bound. An exact completed
/// count is mathematically both an upper and a lower bound; a conservative
/// envelope computed before selection is not a reachable plan's demand; a
/// search stop is a floor on unexplored work, not the budget success needs;
/// a construction stop is an exact attempted prefix, not the complete
/// trace's demand.
///
/// The match is wildcard-free over [`BudgetResource::ALL`], which is itself
/// sized by `variant_count`, so a sixteenth resource is a build error here
/// rather than a census that still reports four classes over a smaller set.
#[test]
fn every_budget_resource_reports_exactly_one_provenance() {
    let mut exact = 0usize;
    let mut envelope = 0usize;
    let mut search = 0usize;
    let mut construction = 0usize;
    for resource in BudgetResource::ALL {
        let expected = match resource {
            BudgetResource::SemanticValues
            | BudgetResource::SemanticOperations
            | BudgetResource::RegionMembers
            | BudgetResource::RegionBoundaryOutputs
            | BudgetResource::RegionLiveValues => BudgetRefusal::ExactDemand,
            BudgetResource::Regions
            | BudgetResource::HostExpressionNodes
            | BudgetResource::Buffers => BudgetRefusal::PlanningUpperBound,
            BudgetResource::RegionCandidatesPerSeed
            | BudgetResource::RegionExpansions
            | BudgetResource::RegionCovers
            | BudgetResource::RegionCoverExpansions
            | BudgetResource::PhysicalPlanCombinations => BudgetRefusal::SearchLowerBound,
            BudgetResource::ExplainDetailRecords | BudgetResource::ExplainDetailCanonicalBytes => {
                BudgetRefusal::ConstructionLowerBound
            }
        };
        assert_eq!(
            resource.refusal(),
            expected,
            "{resource:?} reports the wrong provenance",
        );
        match expected {
            BudgetRefusal::ExactDemand => exact += 1,
            BudgetRefusal::PlanningUpperBound => envelope += 1,
            BudgetRefusal::SearchLowerBound => search += 1,
            BudgetRefusal::ConstructionLowerBound => construction += 1,
        }
    }
    assert_eq!(
        (
            exact,
            envelope,
            search,
            construction,
            exact + envelope + search + construction,
        ),
        (5, 3, 5, 2, BudgetResource::ALL.len()),
        "provenance census changed; re-read every dependent claim. exact={exact} envelope={envelope} search={search} construction={construction} total={}",
        BudgetResource::ALL.len(),
    );
    eprintln!(
        "budget-resource provenance census: exact={exact} envelope={envelope} search={search} construction={construction} total={}",
        BudgetResource::ALL.len(),
    );
}

#[test]
fn each_widened_budget_refuses_the_program_one_step_past_it() {
    let governed = DeterministicBudgets::governed();

    for (inputs, operations) in [(18, 62), (18, 58)] {
        assert_eq!(
            check_program_budgets(&budget_probe(inputs, operations, 3), governed),
            Ok(()),
            "the decoder layer's measured row {inputs}/{operations} is admitted",
        );
    }

    // Exceeding `semantic_values` alone is not expressible: the bound is
    // exactly the eighteen inputs plus the sixty-two occurrences, so one
    // more value is one more input or one more occurrence. Which resource is
    // reported is therefore the check order's guarantee rather than an
    // accident, and it is the first one.
    assert_eq!(
        verify_program(
            &budget_probe(19, 62, 3),
            governed,
            &laws_of(&budget_probe(19, 62, 3))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::SemanticValues,
            limit: 80,
            reported: 81,
        }),
    );

    assert_eq!(
        verify_program(
            &budget_probe(17, 63, 3),
            governed,
            &laws_of(&budget_probe(17, 63, 3))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::SemanticOperations,
            limit: 62,
            reported: 63,
        }),
    );

    // One further declared output is four further dispatches, and it is the
    // *only* one of these five probes that moves along the output axis. It
    // exceeds all three derived bounds at once — sixteen dispatches,
    // fifty-five expression nodes, and thirty-four buffers — and `regions`
    // is the one that reports, which is the check order's guarantee again.
    assert_eq!(
        verify_program(
            &budget_probe(18, 62, 4),
            governed,
            &laws_of(&budget_probe(18, 62, 4))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::Regions,
            limit: 12,
            reported: 16,
        }),
    );

    assert_eq!(
        verify_program(
            &budget_probe(19, 18, 3),
            governed,
            &laws_of(&budget_probe(19, 18, 3))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::HostExpressionNodes,
            limit: 51,
            reported: 53,
        }),
    );

    // `buffers` is reached only once the bound that shadows it moves, and
    // the shadowing is a property of the two bounds rather than of this
    // test: both are derived from the declared input count and both are
    // tight at eighteen, so a nineteen-input program exceeds them together
    // and the earlier check reports. The perturbation widens
    // `host_expression_nodes` to exactly what nineteen inputs and three
    // outputs need and leaves `buffers` at its governed value, so what is
    // observed refusing is the governed bound.
    let unshadowed = DeterministicBudgets {
        host_expression_nodes: 53,
        ..governed
    };
    assert_eq!(
        verify_program(
            &budget_probe(19, 18, 3),
            unshadowed,
            &laws_of(&budget_probe(19, 18, 3))
        )
        .err(),
        Some(RequestError::BudgetExceeded {
            resource: BudgetResource::Buffers,
            limit: 30,
            reported: 31,
        }),
    );
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

#[test]
fn request_requires_a_nonempty_unique_target_set() {
    let program = program();
    let mut empty = CompilationRequest::governed(&program);
    empty.target_profiles.clear();
    assert_eq!(
        verify_planned_request(empty),
        Err(RequestError::EmptyTargetSet)
    );

    let mut duplicate = CompilationRequest::governed(&program);
    duplicate.target_profiles.push(TargetProfile::governed());
    assert_eq!(
        verify_planned_request(duplicate),
        Err(RequestError::DuplicateTargetProfile)
    );
}

#[test]
fn verified_request_receipts_reject_post_verification_mutation() {
    let program = program();
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let mut forged = verified.clone();
    forged.budgets.buffers += 1;
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    let mut forged = verified.clone();
    forged.capabilities = CompilerCapabilitySnapshot::without_capabilities();
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    let mut forged = verified.clone();
    forged.target_slots[0].target_profile =
        TargetProfile::governed_without_numerical_declarations();
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    let mut forged = verified.clone();
    forged.semantic_identity = program_with_unused_provider(7).semantic_identity().clone();
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    // The recognized prologue's scale changed. It is the mutation that used
    // to be a `scale_bits` edit: the subject now carries the whole
    // expression, so a forged prologue is a forged expression.
    let mut forged = verified.clone();
    forge_prologue(
        &mut forged.normalized,
        affine_expression(3.0_f32.to_bits(), 1.0_f32.to_bits()),
    );
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    let mut forged = verified;
    forged.normalized.serial_sum_mut().output_key = OutputKey::new("forged").unwrap();
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );
}

#[test]
fn verified_target_receipt_detects_every_governed_subject_mutation_class() {
    let program = program();
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let target = verified.for_target(0).unwrap();

    let mut forged = target.clone();
    forged.target_profile = TargetProfile::governed_without_numerical_declarations();
    assert!(!forged.reconstructs_its_authority());

    let mut forged = target.clone();
    forged.capabilities = CompilerCapabilitySnapshot::without_capabilities();
    assert!(!forged.reconstructs_its_authority());

    let mut forged = target.clone();
    forged.budgets.regions += 1;
    assert!(!forged.reconstructs_its_authority());

    let mut forged = target.clone();
    forged.semantic_identity = program_with_unused_provider(11).semantic_identity().clone();
    assert!(!forged.reconstructs_its_authority());

    // One constant of the recognized prologue flipped. The expression is
    // rebuilt rather than edited in place, because it is opaque by
    // construction — which is exactly what makes the subject bind it whole.
    let mut forged = target.clone();
    forge_prologue(
        &mut forged.normalized,
        affine_expression(2.0_f32.to_bits(), 1.0_f32.to_bits() ^ 1),
    );
    assert!(!forged.reconstructs_its_authority());

    let mut forged = target;
    forged.normalized.serial_sum_mut().input_keys = vec![InputKey::new("forged").unwrap()];
    assert!(!forged.reconstructs_its_authority());
}

#[test]
fn used_provider_revision_changes_admission_and_snapshot_subjects() {
    let first = governed_test_program(1);
    let second = governed_test_program(2);
    let first = verify_planned_request(request_with_matching_empty_capabilities(&first)).unwrap();
    let second = verify_planned_request(request_with_matching_empty_capabilities(&second)).unwrap();

    assert_eq!(
        first.semantic_identity.graph(),
        second.semantic_identity.graph()
    );
    assert_eq!(
        first.semantic_identity.reached_definitions(),
        second.semantic_identity.reached_definitions()
    );
    assert_ne!(
        first.semantic_identity.admission_provenance(),
        second.semantic_identity.admission_provenance()
    );
    assert_ne!(
        first.semantic_identity.registry_snapshot(),
        second.semantic_identity.registry_snapshot()
    );
}

#[test]
fn unused_provider_revision_changes_only_the_snapshot_subject() {
    let first = program_with_unused_provider(1);
    let second = program_with_unused_provider(2);
    let first = verify_planned_request(request_with_matching_empty_capabilities(&first)).unwrap();
    let second = verify_planned_request(request_with_matching_empty_capabilities(&second)).unwrap();

    assert_eq!(
        first.semantic_identity.graph(),
        second.semantic_identity.graph()
    );
    assert_eq!(
        first.semantic_identity.reached_definitions(),
        second.semantic_identity.reached_definitions()
    );
    assert_eq!(
        first.semantic_identity.admission_provenance(),
        second.semantic_identity.admission_provenance()
    );
    assert_ne!(
        first.semantic_identity.registry_snapshot(),
        second.semantic_identity.registry_snapshot()
    );
}

fn request_symbol(name: &str) -> ShapeSymbol {
    ShapeSymbol::new(SymbolScope::new("program/0").unwrap(), name).unwrap()
}

fn request_axis_binding(input: &str, axis: u32) -> RootBinding {
    RootBinding::new(
        BindingSource::InputDimension {
            input: InputKey::new(input).unwrap(),
            axis: Axis::new(axis),
        },
        AvailabilityPhase::LiveDevicePreflight,
        FactProvenance::RuntimeValidated,
    )
    .unwrap()
}

fn request_environment(bound_to: Option<u64>) -> Arc<ShapeEnv> {
    request_environment_rooted("a", bound_to)
}

/// The fixture environment with `n` rooted at `input[0]`.
///
/// The root input is a parameter because the live source projection must
/// follow the environment's exact root rather than the first declared
/// input, and only a fixture that can move the root can watch that.
fn request_environment_rooted(input: &str, bound_to: Option<u64>) -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let declared = request_symbol("n");
    draft.declare(declared.clone()).unwrap();
    draft
        .bind(&declared, request_axis_binding(input, 0))
        .unwrap();
    if let Some(value) = bound_to {
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::equal(ExtentTerm::Symbol(declared), ExtentTerm::Constant(value)),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
    }
    Arc::new(draft.build().unwrap())
}

/// An environment whose `n` is rooted at an interface parameter, not an
/// input dimension — a valid authored program outside the admitted live
/// population.
fn interface_parameter_environment() -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let declared = request_symbol("n");
    draft.declare(declared.clone()).unwrap();
    draft
        .bind(
            &declared,
            RootBinding::new(
                BindingSource::InterfaceParameter {
                    key: tiler_ir::shape::InterfaceParameterKey::new("len").unwrap(),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    Arc::new(draft.build().unwrap())
}

/// `(a * b) + c` over three rank-one `f32` inputs of one sourced extent.
fn three_input_elementwise_with(
    environment: Option<Arc<ShapeEnv>>,
    extents: &[SourcedExtent],
) -> SemanticProgram {
    let mut builder = match environment {
        Some(environment) => {
            SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap()
        }
        None => SemanticProgramBuilder::try_standard().unwrap(),
    };
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input_sourced::<F32>(InputKey::new(key).unwrap(), extents.to_vec())
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let root = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

fn symbolic_three_input_elementwise(bound_to: Option<u64>) -> SemanticProgram {
    three_input_elementwise_with(
        Some(request_environment(bound_to)),
        &[SourcedExtent::Symbol(request_symbol("n"))],
    )
}

fn literal_three_input_elementwise(extent: u64) -> SemanticProgram {
    three_input_elementwise_with(None, &[SourcedExtent::Static(Extent::new(extent))])
}

fn first_symbolic_extent(program: &SemanticProgram) -> SourcedExtent {
    program
        .inputs()
        .next()
        .and_then(|input| program.shape(input.value()).ok())
        .and_then(|shape| shape.extents().find(|extent| extent.as_static().is_none()))
        .expect("the symbolic fixture names at least one symbol")
}

fn scheduled_symbolic_extent(error: &crate::pipeline::CompileError) -> Option<&SourcedExtent> {
    match error {
        crate::pipeline::CompileError::UnsupportedCapability(
            RequestError::UnsupportedSymbolicExtent {
                phase: "schedule",
                rule: "symbolic-extent",
                extent,
            },
        ) => Some(extent),
        crate::pipeline::CompileError::Explained { source, .. } => {
            scheduled_symbolic_extent(source)
        }
        _ => None,
    }
}

/// Same-shape elementwise is admitted through strategy with extents left symbolic.
///
/// Watched failing under a deliberate perturbation: restoring
/// `static_shape` in `recognize_elementwise_output` makes this program
/// refuse as `UnsupportedSymbolicExtent { phase: "strategy" }` before a
/// `NormalizedProgram` exists.
#[test]
fn a_symbolic_elementwise_program_is_recognized_with_its_symbols() {
    let program = symbolic_three_input_elementwise(None);
    let request = CompilationRequest::governed(&program);
    assert!(
        std::ptr::eq(
            request
                .shape_environment
                .expect("a symbolic program carries its environment")
                .environment(),
            program
                .extent_sources()
                .expect("the constructed program owns its environment")
                .environment(),
        ),
        "the request must carry the program's own environment, not a second one",
    );
    let verified = verify_planned_request(request)
        .expect("same-shape symbolic elementwise must pass strategy selection");
    assert_eq!(
        verified.normalized.first_symbolic_extent(),
        Some(SourcedExtent::Symbol(request_symbol("n"))),
    );
    let pointwise = verified
        .normalized
        .outputs()
        .first()
        .and_then(NormalizedOutput::pointwise)
        .expect("the fixture is whole-program elementwise");
    assert_eq!(pointwise.shape.as_static(), None);
    assert_eq!(
        pointwise.shape.extents().collect::<Vec<_>>(),
        vec![SourcedExtent::Symbol(request_symbol("n"))],
    );
}

/// The verified target request of one symbolic fixture, and its program.
fn symbolic_target(bound_to: Option<u64>) -> (SemanticProgram, VerifiedTargetRequest) {
    let program = symbolic_three_input_elementwise(bound_to);
    let target = verify_planned_request(CompilationRequest::governed(&program))
        .expect("the admitted symbolic population verifies")
        .for_target(0)
        .expect("one governed target");
    (program, target)
}

/// The canonical live region and members of one symbolic target request.
fn live_region_of(
    target: &VerifiedTargetRequest,
) -> (
    crate::physical::ScheduledRegion,
    Vec<crate::region::SemanticStage>,
) {
    let [output] = target.normalized().outputs() else {
        panic!("the fixture declares one output");
    };
    crate::physical::pointwise_region(target, output, crate::physical::RegionWrite::ProgramOutput)
}

/// A still-unsupported symbolic population names the extent at schedule;
/// the literal neighbour compiles.
///
/// The admitted rank-one population no longer reaches this refusal — its
/// own test below proves the decline moved to program assembly — so the
/// subjects here are the populations the accepted surface leaves refused:
/// a mixed-rank domain, a symbol rooted at an interface parameter, and a
/// root input the region never reads densely. Each is perturbed
/// independently of the parametric-broadcast exception, so a missing
/// broadcast is provably not the only way a symbol reaches a plan.
#[test]
fn unsupported_symbolic_populations_keep_the_named_schedule_refusal() {
    // Rank two with one symbolic axis: same-shape, recognized, refused.
    let mixed_rank = three_input_elementwise_with(
        Some(request_environment(None)),
        &[
            SourcedExtent::Symbol(request_symbol("n")),
            SourcedExtent::Static(Extent::new(4)),
        ],
    );
    let extent = first_symbolic_extent(&mixed_rank);
    match crate::pipeline::compile(CompilationRequest::governed(&mixed_rank)) {
        Err(error) => assert_eq!(
            scheduled_symbolic_extent(&error),
            Some(&extent),
            "a higher-rank symbolic domain must keep the schedule refusal, got {error}"
        ),
        Ok(_) => panic!("a higher-rank symbolic domain must keep the schedule refusal"),
    }

    // A non-input root: the environment roots `n` at an interface
    // parameter, which has no accepted runtime input-axis realization.
    let parameter_rooted = three_input_elementwise_with(
        Some(interface_parameter_environment()),
        &[SourcedExtent::Symbol(request_symbol("n"))],
    );
    let extent = first_symbolic_extent(&parameter_rooted);
    match crate::pipeline::compile(CompilationRequest::governed(&parameter_rooted)) {
        Err(error) => assert_eq!(
            scheduled_symbolic_extent(&error),
            Some(&extent),
            "a non-input-rooted symbol must keep the schedule refusal, got {error}"
        ),
        Ok(_) => panic!("a non-input-rooted symbol must keep the schedule refusal"),
    }

    // A root input the region never reads: `b + c` declares `a` and roots
    // `n` there, but no dense read realizes `a[0]`, so there is no access
    // for the source marker to sit on.
    let unread_root = {
        let mut builder =
            SemanticProgramBuilder::try_standard_with_shape_environment(request_environment(None))
                .unwrap();
        let inputs: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|key| {
                builder
                    .input_sourced::<F32>(
                        InputKey::new(key).unwrap(),
                        vec![SourcedExtent::Symbol(request_symbol("n"))],
                    )
                    .unwrap()
            })
            .collect();
        let root = F32Add::apply(&mut builder, inputs[1], inputs[2]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };
    let extent = first_symbolic_extent(&unread_root);
    match crate::pipeline::compile(CompilationRequest::governed(&unread_root)) {
        Err(error) => assert_eq!(
            scheduled_symbolic_extent(&error),
            Some(&extent),
            "an unread root input must keep the schedule refusal, got {error}"
        ),
        Ok(_) => panic!("an unread root input must keep the schedule refusal"),
    }

    let literal = literal_three_input_elementwise(4);
    crate::pipeline::compile(CompilationRequest::governed(&literal))
        .expect("the literal neighbour of the symbolic elementwise program still compiles");
}

/// A distinct proved-equal symbol does not widen the exact-shape population.
///
/// The environment declares `m`, roots it at `b[0]`, and proves `n == m`;
/// the recognizer still compares exact `SourcedShape`, so the program is
/// refused at strategy rather than admitted through the live schedule on a
/// solver fact.
#[test]
fn a_proved_equal_symbol_does_not_widen_the_admitted_population() {
    let environment = {
        let mut draft = ShapeEnvBuilder::new();
        let n = request_symbol("n");
        let m = request_symbol("m");
        draft.declare(n.clone()).unwrap();
        draft.declare(m.clone()).unwrap();
        draft.bind(&n, request_axis_binding("a", 0)).unwrap();
        draft.bind(&m, request_axis_binding("b", 0)).unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::equal(ExtentTerm::Symbol(n), ExtentTerm::Symbol(m)),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        Arc::new(draft.build().unwrap())
    };
    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let a = builder
        .input_sourced::<F32>(
            InputKey::new("a").unwrap(),
            vec![SourcedExtent::Symbol(request_symbol("n"))],
        )
        .unwrap();
    let b = builder
        .input_sourced::<F32>(
            InputKey::new("b").unwrap(),
            vec![SourcedExtent::Symbol(request_symbol("m"))],
        )
        .unwrap();
    let root = F32Add::apply(&mut builder, a, b).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let program = builder.build().unwrap();
    let refusal = verify_planned_request(CompilationRequest::governed(&program))
        .expect_err("a differently spelled proved-equal symbol must refuse at recognition");
    assert_eq!(
        refusal.to_string(),
        "compile.unsupported.strategy.elementwise-shape: no installed capability can \
         compile this valid semantic program",
        "neither spelling equality nor proves_equal may widen the exact-shape population"
    );
}

/// The admitted population forms the verified source-bound live schedule.
///
/// The accepted fieldless spelling, end to end at the physical layer: a
/// rank-zero static outer domain of one work item, the exact root-realizing
/// read carrying `LiveRowMajorSource` at the decoded `a[0]` root, every
/// other read and the final write the fieldless consumer, one derived
/// input-extent operand, checked request binding, feasibility, and a
/// lowered kernel consuming exactly that operand. The retained request
/// still names the authored `n`, and a binding that proves `n == 4`
/// changes none of the schedule bytes while the literal `[4]` neighbour's
/// differ — the specialization boundary held from both sides.
#[test]
fn the_admitted_symbolic_population_forms_a_verified_source_bound_live_schedule() {
    use tiler_ir::schedule::LogicalAccess;

    let (_, target) = symbolic_target(None);
    assert_eq!(
        target.normalized().first_symbolic_extent(),
        Some(SourcedExtent::Symbol(request_symbol("n"))),
        "the retained compiler request still names the authored symbol",
    );
    let root = crate::physical::decode_live_extent_root(
        target.semantic_identity().shape_environment().as_bytes(),
        &request_symbol("n"),
        tiler_ir::schedule::RegionId::new(0),
    )
    .expect("the retained identity bytes decode to the root");
    assert_eq!(root.input, InputKey::new("a").unwrap());
    assert_eq!(root.axis, Axis::new(0));

    let (region, members) = live_region_of(&target);
    assert_eq!(region.index.iteration_shape.rank(), 0, "empty static outer");
    assert_eq!(region.schedule.work_items, 1, "one static outer invocation");
    let maps: Vec<LogicalAccess> = region
        .index
        .accesses
        .iter()
        .map(|access| access.map.clone())
        .collect();
    assert_eq!(
        maps,
        vec![
            LogicalAccess::LiveRowMajorSource {
                inner_axis: Axis::new(0)
            },
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajor,
        ],
        "one source marker on the root read, fieldless consumers elsewhere, \
         the final write included",
    );

    let verified =
        crate::physical::verify_schedule_with_feasibility(region.clone(), members.clone(), &target)
            .expect("the source-bound live schedule verifies and binds");
    assert_eq!(
        tiler_ir::schedule::live_input_extents(verified.region()),
        vec![(tiler_ir::schedule::AccessOrdinal::new(0), Axis::new(0))],
        "the marker is the region's one runtime extent operand",
    );
    let kernel = crate::physical::lower_structured_kernel(&verified)
        .expect("the live schedule lowers to a verified kernel");
    let operands: Vec<_> = kernel.input_extents().collect();
    assert_eq!(operands.len(), 1);
    assert_eq!(
        operands[0].access,
        tiler_ir::schedule::AccessOrdinal::new(0)
    );
    assert_eq!(operands[0].axis, Axis::new(0));

    // The bound-symbol neighbour: `n == 4` proved, schedule bytes exact.
    let (_, bound_target) = symbolic_target(Some(4));
    let (bound_region, bound_members) = live_region_of(&bound_target);
    let bound = crate::physical::verify_schedule_with_feasibility(
        bound_region,
        bound_members,
        &bound_target,
    )
    .expect("a bound symbol still verifies as the symbol");
    assert_eq!(
        verified.canonical_identity().as_bytes(),
        bound.canonical_identity().as_bytes(),
        "a binding that proves n == 4 must not move the schedule bytes",
    );

    // The literal `[4]` neighbour is a different schedule.
    let literal = literal_three_input_elementwise(4);
    let literal_target = verify_planned_request(CompilationRequest::governed(&literal))
        .unwrap()
        .for_target(0)
        .unwrap();
    let (literal_region, literal_members) = live_region_of(&literal_target);
    let literal_verified = crate::physical::verify_schedule_with_feasibility(
        literal_region,
        literal_members,
        &literal_target,
    )
    .expect("the literal neighbour verifies");
    assert_ne!(
        verified.canonical_identity().as_bytes(),
        literal_verified.canonical_identity().as_bytes(),
        "the live schedule and the baked [4] schedule are different subjects",
    );
}

/// The source marker projects to the environment's root, never the first
/// input.
///
/// Rebinding `n` to `c[0]` with the access order unchanged moves the
/// marker to read position 2; positions 0 and 1 become fieldless
/// consumers.
#[test]
fn the_source_marker_follows_the_environment_root_not_the_first_input() {
    use tiler_ir::schedule::LogicalAccess;

    let program = three_input_elementwise_with(
        Some(request_environment_rooted("c", None)),
        &[SourcedExtent::Symbol(request_symbol("n"))],
    );
    let target = verify_planned_request(CompilationRequest::governed(&program))
        .unwrap()
        .for_target(0)
        .unwrap();
    let (region, members) = live_region_of(&target);
    let maps: Vec<LogicalAccess> = region
        .index
        .accesses
        .iter()
        .map(|access| access.map.clone())
        .collect();
    assert_eq!(
        maps,
        vec![
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajorSource {
                inner_axis: Axis::new(0)
            },
            LogicalAccess::LiveRowMajor,
        ],
        "the marker moved to access 2 with the root, not stayed first",
    );
    crate::physical::verify_schedule_with_feasibility(region, members, &target)
        .expect("the c-rooted live schedule verifies and binds");
}

/// A hand-built region whose marker sits on the wrong access is refused as
/// `request-binding`: intrinsic verification cannot see the semantic root,
/// and the compiler binding proves it independently.
#[test]
fn a_forged_source_marker_position_fails_request_binding() {
    use tiler_ir::schedule::LogicalAccess;

    let (_, target) = symbolic_target(None);
    let (mut region, members) = live_region_of(&target);
    // The fixture's root is `a[0]`, access 0. Nominate `b` instead.
    region.index.accesses[0].map = LogicalAccess::LiveRowMajor;
    region.index.accesses[1].map = LogicalAccess::LiveRowMajorSource {
        inner_axis: Axis::new(0),
    };
    let refusal = crate::physical::verify_schedule_with_feasibility(region, members, &target)
        .expect_err("a marker off the decoded root must not bind");
    assert_eq!(
        refusal.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
        "equal runtime values cannot replace the exact a[0] authority",
    );
}

/// A launch minted over the determined representative extent cannot bind
/// the symbolic subject: plan specialization stays forbidden and its
/// refusal stays reachable.
#[test]
fn a_specialized_representative_launch_fails_request_binding() {
    let (_, target) = symbolic_target(Some(4));
    // The literal `[4]` region a folding formation step would mint.
    let literal = literal_three_input_elementwise(4);
    let literal_target = verify_planned_request(CompilationRequest::governed(&literal))
        .unwrap()
        .for_target(0)
        .unwrap();
    let (specialized, members) = live_region_of(&literal_target);
    assert_eq!(
        specialized.schedule.work_items, 4,
        "the specialized region launches over the bound value",
    );
    let refusal = crate::physical::verify_schedule_with_feasibility(specialized, members, &target)
        .expect_err("a [4] launch must not bind the symbolic subject");
    assert_eq!(
        refusal.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
        "ExtentSources::determined never supplies schedule geometry",
    );
}

/// Truncated and bad-domain identity-subject bytes fail as the existing
/// compiler `request-binding` through the production decode mapping.
#[test]
fn corrupted_identity_subject_bytes_fail_as_request_binding() {
    let (_, target) = symbolic_target(None);
    let bytes = target
        .semantic_identity()
        .shape_environment()
        .as_bytes()
        .to_vec();
    let region = tiler_ir::schedule::RegionId::new(0);

    let truncated = crate::physical::decode_live_extent_root(
        &bytes[..bytes.len() - 1],
        &request_symbol("n"),
        region,
    )
    .expect_err("truncated identity bytes must not decode");
    assert_eq!(
        truncated.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
        "a truncated subject is the existing compiler request-binding refusal",
    );

    let mut bad_domain = bytes.clone();
    // The domain separator is length-framed at the front; flipping a byte
    // inside it is a subject from another domain.
    bad_domain[8] ^= 0xff;
    let bad = crate::physical::decode_live_extent_root(&bad_domain, &request_symbol("n"), region)
        .expect_err("bad-domain identity bytes must not decode");
    assert_eq!(
        bad.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
        "a bad-domain subject is the existing compiler request-binding refusal",
    );

    // An absent symbol on well-formed bytes is the same fail-closed rule:
    // no arm defaults an environment or selects another binding.
    let absent = crate::physical::decode_live_extent_root(
        &bytes,
        &ShapeSymbol::new(SymbolScope::new("program/0").unwrap(), "absent").unwrap(),
        region,
    )
    .expect_err("an undeclared symbol must not resolve a root");
    assert_eq!(
        absent.to_string(),
        "schedule.intrinsic.request-binding: region 0 rejected",
    );
}

/// A bound symbol is not folded into the compiled product.
///
/// The environment pins `n` to 4. The program still names the symbol, the
/// request still carries that environment, and compilation forms the live
/// schedule as the symbol — never a `[4]` plan.
///
/// **The value-never-enters-identity assertions below are unchanged; the wall
/// assertion beside them is what moved.** This used to close by requiring
/// `compile()` to decline at `program-assembly.named-output-symbolic`, which
/// stood only because packaging could not represent the shape-environment
/// subject. `tiler.kernel-program.v13` folds it, so the population packages and
/// the claim strengthens from "declines rather than compiling as 4" to the
/// directly testable "compiles, and what it compiles is not the `[4]` program".
/// The exact schedule-identity claim (bound and unbound schedule bytes equal,
/// literal `[4]` bytes different) remains
/// `the_admitted_symbolic_population_forms_a_verified_source_bound_live_schedule`'s.
#[test]
fn a_compiled_plan_does_not_fold_a_bound_extent_value() {
    let bound = symbolic_three_input_elementwise(Some(4));
    let extent = first_symbolic_extent(&bound);
    assert_eq!(
        extent,
        SourcedExtent::Symbol(request_symbol("n")),
        "a constraint that pins n to 4 must not rewrite the authored shape",
    );
    for value in bound.values() {
        assert_eq!(
            value.shape().as_static(),
            None,
            "no authored boundary may collapse to the bound value",
        );
    }
    let verified = verify_planned_request(CompilationRequest::governed(&bound))
        .expect("a bound symbol is still a recognized symbolic program");
    assert_eq!(
        verified.normalized.first_symbolic_extent(),
        Some(extent.clone()),
        "recognition must keep the authored symbol, not the bound value 4",
    );
    let compiled = crate::pipeline::compile(CompilationRequest::governed(&bound))
        .expect("a bound symbol packages as the symbol");
    let literal = literal_three_input_elementwise(4);
    let literal_compiled = crate::pipeline::compile(CompilationRequest::governed(&literal))
        .expect("the literal [4] neighbour still compiles");
    assert_ne!(
        packaged_program_identity(&compiled),
        packaged_program_identity(&literal_compiled),
        "a program that names n, even with n proved equal to 4, is not the [4] program",
    );

    // The packaged boundary keeps the symbol rather than the proved value: the
    // covered boundary is the zero-extent convention, and 4 appears nowhere in
    // it. A packaging step that folded the bound value would size this at 4.
    let packaged = packaged_program(&compiled);
    for value in packaged.core().values() {
        assert!(
            value
                .shape()
                .extents()
                .iter()
                .all(|extent| extent.get() != 4),
            "no packaged value may be sized by the bound extent value",
        );
    }

    // Where the live quantity *is* carried: as an `InputExtent` root over the
    // environment's own decoded root, resolved at live preflight from the
    // caller's buffer. This is the positive half of the assertion above — a
    // packaging step that folded the bound value would have no reason to
    // declare this root, and one that dropped the quantity entirely would leave
    // the accessible ranges sized by nothing.
    let rooted = packaged.core().abi_expressions().iter().any(|node| {
        matches!(
            node,
            tiler_ir::program::abi::ExprNode::Root(tiler_ir::program::abi::AbiRoot::InputExtent {
                key,
                axis,
            }) if key.as_str() == "a" && axis.get() == 0,
        )
    });
    assert!(
        rooted,
        "the live extent must be carried as a root over the environment's decoded root",
    );
}

/// The verified kernel program one compiled target packaged.
fn packaged_program(
    compiled: &crate::pipeline::CompilationProduct,
) -> &crate::program::KernelProgram {
    &compiled.targets[0]
        .compiled()
        .expect("the governed target compiled")
        .portfolio
        .alternatives[0]
        .program
}

/// The canonical kernel-program identity bytes one compiled target packaged.
fn packaged_program_identity(compiled: &crate::pipeline::CompilationProduct) -> Vec<u8> {
    packaged_program(compiled)
        .core()
        .canonical_identity()
        .as_bytes()
        .to_vec()
}

/// The admitted symbolic population passes schedule formation *and* packaging,
/// and its packaged boundary keeps the symbol.
///
/// **This replaces
/// `the_admitted_symbolic_population_declines_at_program_assembly_not_schedule`.**
/// Two walls fell in turn for this population and the retired names of both are
/// recorded here so a later reader can tell which one a regression restored:
/// the schedule-geometry refuse `UnsupportedSymbolicExtent { phase: "schedule",
/// rule: "symbolic-extent" }` went when the source-bound live schedule landed,
/// and the packaging refuse `program-assembly.named-output-symbolic` went at
/// `tiler.kernel-program.v13`, which folds the shape-environment subject so a
/// symbolic program's identity is complete rather than under-keyed.
///
/// Watched failing under two independent deliberate perturbations, each showing
/// a different assertion below is load-bearing. Restoring the unconditional
/// schedule gate makes this fail at the `scheduled_symbolic_extent` assertion
/// with `compile.schedule.symbolic-extent: program/0::n is a symbolic extent
/// this capability cannot plan over`. Restoring `named-output-symbolic` as an
/// unconditional refusal in `CoverAssembly::from_plan` makes it fail at the
/// `compile` call with `compile.unsupported.program-assembly.named-output-symbolic:
/// no installed capability can compile this valid semantic program`.
#[test]
fn the_admitted_symbolic_population_packages_a_verified_kernel_program() {
    let symbolic = symbolic_three_input_elementwise(None);
    crate::region::RegionGraph::from_program(&symbolic)
        .expect("region-graph construction must record a sourced boundary");
    crate::region::form_region_candidates(
        &symbolic,
        crate::request::DeterministicBudgets::governed(),
        crate::request::StrictF32NumericalContract::governed(),
    )
    .expect("region formation must accept the admitted symbolic population");

    let compiled = match crate::pipeline::compile(CompilationRequest::governed(&symbolic)) {
        Ok(compiled) => compiled,
        Err(error) => {
            assert_eq!(
                scheduled_symbolic_extent(&error),
                None,
                "the admitted population must pass the schedule gate, got {error}"
            );
            panic!("the admitted population must package, got {error}");
        }
    };

    // The packaged program's identity folds the program's own environment
    // subject beside its graph. Read rather than asserted by construction: a
    // fold that dropped the subject leaves this needle absent. The domain step
    // itself is pinned where the domain is declared — `tiler_ir`'s
    // `the_program_domain_separator_is_what_distinguishes_the_reinterpreting_steps`
    // and its `PINNED_IDENTITY_DOMAINS` row — rather than restated here, because
    // this crate's own pin census admits only the domains it declares.
    let identity = packaged_program_identity(&compiled);
    let subject = symbolic
        .semantic_identity()
        .shape_environment()
        .as_bytes()
        .to_vec();
    assert!(
        identity
            .windows(subject.len())
            .any(|window| window == subject.as_slice()),
        "the packaged identity must carry the program's own environment subject",
    );

    let literal = literal_three_input_elementwise(4);
    crate::pipeline::compile(CompilationRequest::governed(&literal))
        .expect("the literal neighbour still compiles");
}

/// The packaging population is exactly the admitted one, counted rather than
/// argued.
///
/// **The lift is condition-shaped, not population-shaped**, so the check that
/// matters is that making representation total did not widen *what compiles*.
/// The census walks every symbolic fixture this module can author and requires
/// each to compile exactly when `admits_source_bound_live_schedule` says the
/// request is admitted — including the parametric-broadcast carrier, which the
/// schedule gate lets past on its own separate arm and which must therefore
/// still decline at physical selection rather than falling into packaging.
///
/// The population is printed rather than trusted, and a floor is asserted, so a
/// fixture list that silently stopped covering its subject cannot look green.
#[test]
fn the_packaging_population_is_exactly_the_admitted_population() {
    let cases: Vec<(&str, SemanticProgram)> = vec![
        ("admitted-unbound", symbolic_three_input_elementwise(None)),
        (
            "admitted-bound-4",
            symbolic_three_input_elementwise(Some(4)),
        ),
        (
            "root-at-b",
            three_input_elementwise_with(
                Some(request_environment_rooted("b", None)),
                &[SourcedExtent::Symbol(request_symbol("n"))],
            ),
        ),
        (
            "interface-parameter-root",
            three_input_elementwise_with(
                Some(interface_parameter_environment()),
                &[SourcedExtent::Symbol(request_symbol("n"))],
            ),
        ),
        (
            "unread-root-input",
            three_input_elementwise_with(
                Some(request_environment_rooted("missing", None)),
                &[SourcedExtent::Symbol(request_symbol("n"))],
            ),
        ),
        (
            "parametric-broadcast",
            parametric_broadcast_only_program(
                parametric_broadcast_environment("n", (1, 32_768), None),
                "n",
            ),
        ),
    ];
    assert!(
        cases.len() >= 6,
        "the census must keep every symbolic arm the accepted surface names",
    );

    let mut admitted = 0_usize;
    let mut packaged = 0_usize;
    for (label, program) in &cases {
        let target = verify_planned_request(CompilationRequest::governed(program))
            .ok()
            .and_then(|verified| verified.for_target(0).ok());
        let admits = target
            .as_ref()
            .is_some_and(crate::physical::admits_source_bound_live_schedule);
        let compiles = crate::pipeline::compile(CompilationRequest::governed(program)).is_ok();
        println!("packaging census: {label}: admitted={admits} packaged={compiles}");
        admitted += usize::from(admits);
        packaged += usize::from(compiles);
        assert_eq!(
            admits, compiles,
            "{label}: the packaging population must equal the admitted population",
        );
    }
    // Three admitted: the unbound fixture, its constraint-bound neighbour, and
    // the one whose root moved to `b` — a root the region still reads densely.
    // Three refused: a root that is not an input dimension at all, a root input
    // the region never reads, and the parametric-broadcast carrier, which the
    // schedule gate lets past on its own arm and which must therefore decline at
    // physical selection rather than falling into packaging.
    assert_eq!(
        (admitted, packaged),
        (3, 3),
        "the census must exercise both answers, not one of them six times",
    );
}

/// Dropping the program's environment is a pairing refusal, not a schema one.
#[test]
fn dropping_the_program_environment_is_a_pairing_refusal() {
    let program = symbolic_three_input_elementwise(None);
    let mut request = CompilationRequest::governed(&program);
    request.shape_environment = None;
    match verify_request(request) {
        Err(RequestError::MismatchedShapeEnvironment) => {}
        Ok(_) => panic!("dropping the environment must refuse, got a verified request"),
        Err(error) => panic!("dropping the environment must be a pairing refusal, got {error}"),
    }
    assert_eq!(
        RequestError::MismatchedShapeEnvironment.to_string(),
        "compile.request.shape-environment: request must carry the program's own environment",
    );
}

fn parametric_broadcast_environment(
    symbol: &str,
    interval: (u64, u64),
    guard: Option<u64>,
) -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let declared = request_symbol(symbol);
    draft.declare(declared.clone()).unwrap();
    draft.bind(&declared, request_axis_binding("a", 0)).unwrap();
    draft
        .require(SemanticInputConstraint::new(
            ExtentRelation::interval(ExtentTerm::Symbol(declared.clone()), interval.0, interval.1)
                .unwrap(),
            FactProvenance::FrontendRequired,
        ))
        .unwrap();
    if let Some(value) = guard {
        draft
            .guard(VariantGuard::new(
                ExtentRelation::equal(ExtentTerm::Symbol(declared), ExtentTerm::Constant(value)),
                GuardApplicability::Schedule,
            ))
            .unwrap();
    }
    Arc::new(draft.build().unwrap())
}

/// `a * broadcast(w)` over `a: f32[n, 4]` and `w: f32[4]`.
fn parametric_broadcast_program(
    environment: Arc<ShapeEnv>,
    pad: &str,
) -> (SemanticProgram, BroadcastAxisMapping) {
    let pad_symbol = request_symbol(pad);
    let mapping = BroadcastAxisMapping::new(
        [
            SourcedExtent::Symbol(pad_symbol),
            SourcedExtent::Static(Extent::new(4)),
        ],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .expect("a symbolic rank-pad mapping is context-free");
    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let activation = builder
        .input_sourced::<F32>(
            InputKey::new("a").unwrap(),
            vec![
                SourcedExtent::Symbol(request_symbol(pad)),
                SourcedExtent::Static(Extent::new(4)),
            ],
        )
        .unwrap();
    let weight = builder
        .input_sourced::<F32>(
            InputKey::new("w").unwrap(),
            vec![SourcedExtent::Static(Extent::new(4))],
        )
        .unwrap();
    let widened = F32Broadcast::apply(&mut builder, &mapping, weight)
        .expect("the sourced mapping applies against the program's environment");
    let root = F32Multiply::apply(&mut builder, activation, widened).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    (builder.build().unwrap(), mapping)
}

/// A single sourced broadcast, so lowering has only the parametric
/// occurrence to refine. The fused `a * broadcast(w)` neighbour still
/// exercises recognition; its multiply keeps a static index law.
fn parametric_broadcast_only_program(environment: Arc<ShapeEnv>, pad: &str) -> SemanticProgram {
    let mapping = BroadcastAxisMapping::new(
        [
            SourcedExtent::Symbol(request_symbol(pad)),
            SourcedExtent::Static(Extent::new(4)),
        ],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .expect("a symbolic rank-pad mapping is context-free");
    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let weight = builder
        .input_sourced::<F32>(
            InputKey::new("w").unwrap(),
            vec![SourcedExtent::Static(Extent::new(4))],
        )
        .unwrap();
    let widened = F32Broadcast::apply(&mut builder, &mapping, weight)
        .expect("the sourced mapping applies against the program's environment");
    builder
        .output(OutputKey::new("result").unwrap(), widened)
        .unwrap();
    builder.build().unwrap()
}

fn recognized_parametric_read(program: &SemanticProgram) -> LogicalAccess {
    let verified = verify_planned_request(CompilationRequest::governed(program))
        .expect("a sourced broadcast must pass strategy selection");
    let pointwise = verified
        .normalized
        .outputs()
        .first()
        .and_then(NormalizedOutput::pointwise)
        .expect("the fixture is whole-program elementwise");
    pointwise
        .reads
        .iter()
        .map(|(_, map)| map.clone())
        .find(|map| matches!(map, LogicalAccess::ParametricBroadcast { .. }))
        .expect("recognition must retain the parametric carrier")
}

/// Builds a gather fixture over stated shapes and a stated gathered axis.
fn gather_program_over(source: [u64; 2], index: [u64; 1], axis: u32) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let source = builder
        .input::<F32>(InputKey::new("source").unwrap(), Shape::from_dims(source))
        .unwrap();
    let index = builder
        .input_resolved(
            InputKey::new("index").unwrap(),
            Shape::from_dims(index),
            gather_index_resolved_type(),
        )
        .unwrap();
    let gathered = F32Gather::apply(&mut builder, source, index, Axis::new(axis)).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), gathered)
        .unwrap();
    builder.build().unwrap()
}

/// The gather source relation takes its own request tag and encodes injectively.
///
/// **The tag is checked against the whole named space rather than against one
/// neighbour.** `encode_access_relation` writes `0x01`, `0x02`, `0x03`, `0x05`,
/// and the refusal `0x00`, and `UNREAD_DECLARED_INPUT_TAG` occupies `0x04` in
/// the run this encoder's output sits inside — so a gather taking any of those
/// would either collide with a relation or forge the unread-input marker. `0x06`
/// is the first value above all of them.
///
/// **`0x06` is deliberately not the schedule layer's `0x0C` for the same
/// relation.** Tag spaces here are per-frame, so the two frames each assign
/// their own next free value; this assertion pins the request frame's, and the
/// schedule frame's is pinned in `tiler-ir`.
///
/// Watched failing under three separate subject perturbations, each on the
/// encoder rather than on the assertion:
/// writing the gather at `PARAMETRIC_BROADCAST_ACCESS_TAG` collapses the first
/// assertion; writing it at `UNREAD_DECLARED_INPUT_TAG` collapses the second;
/// and swapping the source and index shape frames collapses the third, because
/// the two shapes differ.
#[test]
fn the_gather_source_relation_takes_its_own_request_tag_and_encodes_injectively() {
    let relation = |axis: u32, index_access: u32| LogicalAccess::GatherSource {
        source_shape: Shape::from_dims([4, 2]),
        result_shape: Shape::from_dims([3, 2]),
        axis: Axis::new(axis),
        index_access: AccessOrdinal::new(index_access),
        index_shape: Shape::from_dims([3]),
    };
    let encode = |map: &LogicalAccess| {
        let mut bytes = Vec::new();
        encode_access_relation(&mut bytes, map);
        bytes
    };

    let gather = encode(&relation(0, 1));
    assert_eq!(
        gather.first().copied(),
        Some(0x06),
        "the gather source relation takes the request frame's next free tag",
    );

    // **The distinctness check is derived from the encoder, not compared to the
    // literal above, and the separation is deliberate.** Asserting that the
    // gather's tag differs from each named constant would be unreachable by
    // pigeonhole: any perturbation of the gather tag trips the pin first, so
    // those assertions could never be the ones to fail and would prove only that
    // the pin runs. Collecting the encoder's *own* answers and requiring them
    // pairwise distinct fails independently — moving `LinearIdentity` onto
    // `0x06` reddens this while leaving the pin above green.
    //
    // `LinearIdentity` and the gather are the two relations constructible here
    // without a program fixture; the parametric carrier's tag is asserted from
    // its constant, and the reindex and replication tags are covered by the
    // pinned subject goldens elsewhere in this module.
    let mut tags: Vec<u8> = [
        encode(&LogicalAccess::LinearIdentity),
        encode(&LogicalAccess::ScalarBroadcast),
        gather.clone(),
    ]
    .iter()
    .filter_map(|bytes| bytes.first().copied())
    // The refusal tag is written for every relation this encoder declines, so
    // several relations legitimately share it and it is not part of the
    // distinct population.
    .filter(|tag| *tag != 0x00)
    .collect();
    let written = tags.len();
    tags.sort_unstable();
    tags.dedup();
    assert_eq!(
        tags.len(),
        written,
        "two encodable access relations share a request-subject tag: {tags:?}",
    );
    assert!(
        !tags.contains(&UNREAD_DECLARED_INPUT_TAG),
        "no relation may forge the unread-declared-input marker: {tags:?}",
    );
    assert!(
        !tags.contains(&PARAMETRIC_BROADCAST_ACCESS_TAG),
        "these relations are not the parametric carrier: {tags:?}",
    );

    // Each member the relation carries separates two encodings on its own.
    assert_ne!(
        gather,
        encode(&relation(1, 1)),
        "the gathered axis is identity",
    );
    assert_ne!(
        gather,
        encode(&relation(0, 2)),
        "the owned address read's local ordinal is identity",
    );
}

/// The `gather-f32.v1` output subject separates every field it carries.
///
/// **The arm is encoded directly rather than through the whole request subject,
/// and that is what makes this test able to fail.** A request subject opens with
/// the semantic graph identity, which already separates any two *programs* that
/// differ in a gather's axis or shapes — so a whole-subject comparison stays
/// green with a field dropped from this arm entirely, and would be asserting the
/// graph identity rather than the projection. Leaning on the enclosing subject
/// to separate arms is exactly the unstated invariant
/// [`encode_elementwise_reads`]'s own documentation forbids resting identity on.
///
/// **Two of these perturbations are unreachable from any program**, which is the
/// other reason the shape is a forge rather than a fixture pair. Declaration
/// order fixes `source_input`/`index_input`, and canonical access order fixes
/// `index_access` at one, so swapping the declared association or moving the
/// owned address ordinal cannot be expressed by authoring a different program.
/// The association swap is the load-bearing one: it is the ADR 0108
/// schedule-clause amendment's central claim that the checked
/// declared-input association lives *here*, in the compiler-private request
/// subject, and nowhere in shared schedule identity.
///
/// Watched failing under a deliberate subject perturbation: dropping
/// `normalized.axis` from `encode_output_subject`'s gather arm reddens the first
/// row with `the gathered axis must move the subject`, while leaving the whole
/// request subject's own goldens green — which is the defect the direct
/// encoding exists to catch.
#[test]
fn a_gather_output_subject_separates_every_field_it_carries() {
    let program = gather_program_over([4, 4], [4], 0);
    let normalized = select_supported_strategy(&program, &laws_of(&program))
        .expect("the gather fixture is recognized");
    let [recognized] = normalized.outputs() else {
        panic!("the fixture declares one output");
    };
    let encoded = |output: &NormalizedOutput| {
        let mut bytes = Vec::new();
        encode_output_subject(&mut bytes, &output_subject(output));
        bytes
    };
    let forge = |edit: fn(&mut NormalizedGather)| {
        let mut forged = recognized.clone();
        let NormalizedOutput::Gather(gather) = &mut forged else {
            panic!("the fixture recognizes as a gather");
        };
        edit(gather);
        encoded(&forged)
    };

    let base = encoded(recognized);
    assert!(!base.is_empty(), "the gather arm encodes a subject");

    for (label, forged) in [
        (
            "the gathered axis",
            forge(|gather| gather.axis = Axis::new(1)),
        ),
        (
            "the declared source/index association",
            forge(|gather| std::mem::swap(&mut gather.source_input, &mut gather.index_input)),
        ),
        (
            "the owned address read's local ordinal",
            forge(|gather| gather.index_access = AccessOrdinal::new(2)),
        ),
        (
            "the source shape",
            forge(|gather| gather.source_shape = Shape::from_dims([5, 4])),
        ),
        (
            "the index shape",
            forge(|gather| gather.index_shape = Shape::from_dims([3])),
        ),
        (
            "the result shape",
            forge(|gather| gather.result_shape = Shape::from_dims([4, 5])),
        ),
        (
            "the claimed occurrence",
            forge(|gather| gather.member = SemanticMemberId(gather.member.0 + 1)),
        ),
        (
            "the source element count",
            forge(|gather| gather.source_elements += 1),
        ),
        (
            "the index element count",
            forge(|gather| gather.index_elements += 1),
        ),
        (
            "the result element count",
            forge(|gather| gather.result_elements += 1),
        ),
    ] {
        assert_ne!(base, forged, "{label} must move the subject");
    }
}

/// A gather takes its own output sub-tag, and no other arm's bytes move.
///
/// The sub-tag is what keeps `tiler.compiler.request-subject.v6` from stepping:
/// a gather is a subject the earlier vocabulary could not express at all, so
/// every previously encodable output still encodes to exactly what it did. The
/// second half of that claim is carried by the module's existing pinned
/// subjects, which this lane did not touch and which pass unchanged; this states
/// the first half.
#[test]
fn a_gather_output_subject_takes_its_own_sub_tag() {
    let program = gather_program_over([4, 2], [3], 0);
    let normalized = select_supported_strategy(&program, &laws_of(&program))
        .expect("the gather fixture is recognized");
    let [recognized] = normalized.outputs() else {
        panic!("the fixture declares one output");
    };
    let mut bytes = Vec::new();
    encode_output_subject(&mut bytes, &output_subject(recognized));
    let tag = b"gather-f32.v1";
    assert!(
        bytes
            .windows(tag.len())
            .any(|window| window == tag.as_slice()),
        "the gather arm writes its own framed sub-tag",
    );
    for other in [
        b"pointwise-f32.v4".as_slice(),
        b"contraction-f32.v1".as_slice(),
        b"serial-sum-f32.v3".as_slice(),
        b"epilogue-f32.v1".as_slice(),
        b"staged-family.v2".as_slice(),
    ] {
        assert!(
            !bytes.windows(other.len()).any(|window| window == other),
            "a gather subject must not carry another arm's sub-tag",
        );
    }
}

/// A recognized gather has no governed region spelling, and the wall says why.
///
/// **This is the lane's stopping point, stated as a typed answer rather than as
/// an absence.** The occurrence is recognized, the request subject binds it, and
/// the schedule layer defines both `LogicalAccess::GatherSource` and its paired
/// `BoundsProofKind::GatherSource`. What physical planning cannot do is obtain
/// the `GatherIndexBoundsProof` that proof variant carries: it is minted only by
/// the index layer's verifier-private deriver, it binds a
/// `CanonicalIndexRegionIdentity`, and the refinement that holds one is not
/// reachable from a provider's `ImplementationContext`.
///
/// **What it would take for this to say something else.** The wall is reached
/// only for a member set that is exactly the gather occurrence's, so the two
/// ways it stops answering are a recognizer that stops producing
/// `NormalizedOutput::Gather` and a `spell_output` arm that returns a spelling
/// instead. The second is the intended future change, and this assertion is what
/// will require the lane that makes it to state the new answer here.
///
/// Watched failing under a deliberate subject perturbation: returning
/// `RegionVocabularyWall::PartialCoverage` from the gather arm reddens this with
/// `left: PartialCoverage  right: GatherProofUnavailable`, and passing a member
/// set that is not the occurrence's falls through to `PartialCoverage` instead —
/// which is what shows the wall is decided for this occurrence rather than
/// reported for every unspellable cover.
#[test]
fn a_recognized_gather_has_no_governed_region_spelling() {
    let program = gather_program();
    let mut request = CompilationRequest::governed(&program);
    request.target_profiles = vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    let planned = verify_planned_request(request).expect("the fixture admits a planned request");
    let target = planned
        .for_target(0)
        .expect("the U32-capable profile admits the fixture");
    let [output] = target.normalized().outputs() else {
        panic!("the fixture declares one output");
    };
    let members = output.members();
    assert_eq!(members.len(), 1, "a gather claims exactly one occurrence");

    assert_eq!(
        crate::physical::spell_region(
            &target,
            &members,
            crate::physical::RegionWrite::ProgramOutput,
        ),
        Err(crate::physical::RegionVocabularyWall::GatherProofUnavailable),
        "a gather's own member set is declined by name, not reported as partial coverage",
    );
    assert_eq!(
        crate::physical::RegionVocabularyWall::GatherProofUnavailable.reason(),
        "gather-proof-unavailable",
    );

    // A member set that is not this occurrence's falls through to the caller's
    // own wall, which is what separates "this region cannot be built" from
    // "this cover names occurrences no output owns".
    let foreign = [crate::region::SemanticStage::first(
        crate::region::SemanticMemberId(members[0].member().0 + 1),
    )];
    assert_eq!(
        crate::physical::spell_region(
            &target,
            &foreign,
            crate::physical::RegionWrite::ProgramOutput,
        ),
        Err(crate::physical::RegionVocabularyWall::PartialCoverage),
    );
}

/// The recognized-output vocabulary is sized from its own enum.
///
/// A hand-written six would be satisfied by an enumeration that had stopped
/// covering the type. `variant_count` makes a widened vocabulary a build error
/// at this line instead, which is the property every consumer's exhaustive match
/// already has and which this states for the population as a whole.
#[test]
fn the_recognized_output_vocabulary_is_sized_from_its_type() {
    assert_eq!(
        std::mem::variant_count::<NormalizedOutput>(),
        6,
        "the recognized output vocabulary changed size; every dependent claim \
about it needs re-reading",
    );
}

fn request_subject_bytes(program: &SemanticProgram) -> Vec<u8> {
    verify_planned_request(CompilationRequest::governed(program))
        .expect("the fixture admits a planned request")
        .for_target(0)
        .expect("the governed profile admits the fixture")
        .subject()
        .canonical_explain_subject_bytes()
}

fn planning_capability_rule(
    error: &crate::pipeline::CompileError,
) -> Option<(&'static str, &'static str)> {
    match error {
        crate::pipeline::CompileError::UnsupportedCapability(
            RequestError::UnsupportedCapability { phase, rule },
        ) => Some((*phase, *rule)),
        crate::pipeline::CompileError::Explained { source, .. } => planning_capability_rule(source),
        _ => None,
    }
}

/// One symbolic broadcast program reaches selection with its mapping and
/// environment unchanged.
///
/// Watched failing under a deliberate perturbation: restoring the static
/// domain gate in `plan_elementwise` refuses this program as
/// `UnsupportedSymbolicExtent { phase: "strategy" }` before a
/// `NormalizedProgram` exists.
#[test]
fn a_parametric_broadcast_program_is_recognized_with_its_carrier() {
    let environment = parametric_broadcast_environment("n", (1, 32_768), None);
    let identity = environment.identity().clone();
    let (program, mapping) = parametric_broadcast_program(environment, "n");
    let request = CompilationRequest::governed(&program);
    assert!(
        std::ptr::eq(
            request
                .shape_environment
                .expect("a symbolic program carries its environment")
                .environment(),
            program
                .extent_sources()
                .expect("the constructed program owns its environment")
                .environment(),
        ),
        "the request must carry the program's own environment, not a second one",
    );
    let verified =
        verify_planned_request(request).expect("a sourced broadcast must pass strategy selection");
    assert!(verified.normalized.carries_parametric_broadcast());
    let LogicalAccess::ParametricBroadcast {
        operand_shape,
        mapping: retained,
        environment: named,
    } = recognized_parametric_read(&program)
    else {
        panic!("recognition must retain ParametricBroadcast, not a concrete neighbour");
    };
    assert_eq!(
        operand_shape.extents().collect::<Vec<_>>(),
        vec![SourcedExtent::Static(Extent::new(4))],
    );
    assert_eq!(retained, mapping);
    assert_eq!(named, identity);
    assert_eq!(
        verified.normalized.first_symbolic_extent(),
        Some(SourcedExtent::Symbol(request_symbol("n"))),
    );
}

/// Perturbing a bound value does not change semantic, normalized-program, or
/// request identity.
///
/// The two programs share declarations, root bindings, and the positivity
/// interval. They differ only in a schedule variant guard pinning `n` to 4
/// or 10. Guards are outside `ShapeEnvIdentity`, so a compiler that folded
/// the pin into `BroadcastReplication` would be the thing that moved the
/// identities.
#[test]
fn a_bound_value_change_does_not_move_parametric_broadcast_identity() {
    let four = parametric_broadcast_program(
        parametric_broadcast_environment("n", (1, 32_768), Some(4)),
        "n",
    )
    .0;
    let ten = parametric_broadcast_program(
        parametric_broadcast_environment("n", (1, 32_768), Some(10)),
        "n",
    )
    .0;
    assert_eq!(four.semantic_identity(), ten.semantic_identity());
    assert_eq!(
        recognized_parametric_read(&four),
        recognized_parametric_read(&ten),
        "recognition must keep the same carrier; a fold to BroadcastReplication would move",
    );
    assert_eq!(
        request_subject_bytes(&four),
        request_subject_bytes(&ten),
        "request identity must not move with a bound value the environment does not author",
    );
    for program in [&four, &ten] {
        let LogicalAccess::ParametricBroadcast { operand_shape, .. } =
            recognized_parametric_read(program)
        else {
            panic!("a bound value must not fold the carrier into a concrete neighbour");
        };
        assert_eq!(operand_shape.as_static(), Some(&Shape::from_dims([4])));
        assert_eq!(
            program.extent_sources().and_then(
                |sources| sources.determined(&SourcedExtent::Symbol(request_symbol("n")))
            ),
            None,
            "a variant guard must not determine the authored symbol",
        );
    }
}

/// A provider lacking parametric support declines by the named capability
/// rule, not a static-signature or generic unsupported mask.
///
/// Watched failing under a deliberate perturbation: leaving the generic
/// symbolic-extent schedule refuse in front of physical selection reports
/// `phase: "schedule", rule: "symbolic-extent"` instead of the provider's
/// `parametric-broadcast` rule.
#[test]
fn a_provider_lacking_parametric_support_declines_by_named_rule() {
    let program = parametric_broadcast_only_program(
        parametric_broadcast_environment("n", (1, 32_768), None),
        "n",
    );
    crate::region::RegionGraph::from_program(&program)
        .expect("region-graph construction must record a sourced broadcast");
    crate::region::form_region_candidates(
        &program,
        DeterministicBudgets::governed(),
        StrictF32NumericalContract::governed(),
    )
    .expect("region formation must accept the parametric population");
    match crate::pipeline::compile(CompilationRequest::governed(&program)) {
        Err(error) => {
            assert_eq!(
                planning_capability_rule(&error),
                Some(("planning", "parametric-broadcast")),
                "a provider without parametric support must decline that named rule, got {error}"
            );
        }
        Ok(_) => panic!("a provider without parametric support must decline, got a product"),
    }

    let literal = literal_three_input_elementwise(4);
    crate::pipeline::compile(CompilationRequest::governed(&literal))
        .expect("the literal neighbour still compiles");
}

/// Two parametric mappings that differ in one pad symbol produce different
/// request-subject bytes. Concrete reindex and broadcast keep tags `0x01`
/// and `0x02`.
///
/// Watched failing under a deliberate perturbation: writing the parametric
/// carrier as `0x02` makes the two encodings share a tag with
/// `BroadcastReplication`.
#[test]
fn parametric_broadcast_request_subject_tag_is_injective() {
    let n_program = parametric_broadcast_program(
        parametric_broadcast_environment("n", (1, 32_768), None),
        "n",
    )
    .0;
    let t_env = {
        let mut draft = ShapeEnvBuilder::new();
        let declared = request_symbol("t");
        draft.declare(declared.clone()).unwrap();
        draft.bind(&declared, request_axis_binding("a", 0)).unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(declared), 1, 32_768).unwrap(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        Arc::new(draft.build().unwrap())
    };
    let t_program = parametric_broadcast_program(t_env, "t").0;
    assert_ne!(
        request_subject_bytes(&n_program),
        request_subject_bytes(&t_program),
        "two pad symbols must not share request-subject bytes",
    );

    let mut parametric_bytes = Vec::new();
    let parametric = recognized_parametric_read(&n_program);
    encode_access_relation(&mut parametric_bytes, &parametric);
    assert_eq!(
        parametric_bytes.first().copied(),
        Some(PARAMETRIC_BROADCAST_ACCESS_TAG),
        "the parametric carrier must take tag 0x05, not the refusal 0x00",
    );

    let concrete = BroadcastAxisMapping::new(
        [Extent::new(2), Extent::new(2)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .unwrap();
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), Shape::from_dims([2]))
        .unwrap();
    let widened = F32Broadcast::apply(&mut builder, &concrete, weight).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), widened)
        .unwrap();
    let concrete_program = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&concrete_program).expect("a literal broadcast is still BroadcastReplication")
    else {
        panic!("a literal broadcast is an elementwise region");
    };
    let (_, LogicalAccess::BroadcastReplication { .. }) = &recognized.reads[0] else {
        panic!("a wholly literal mapping must stay BroadcastReplication");
    };
    let mut concrete_bytes = Vec::new();
    encode_access_relation(&mut concrete_bytes, &recognized.reads[0].1);
    assert_eq!(concrete_bytes.first().copied(), Some(0x02));
    assert_ne!(
        parametric_bytes.first(),
        concrete_bytes.first(),
        "colliding the parametric tag with BroadcastReplication loses injectivity",
    );
}

// ---------------------------------------------------------------------------
// The materialized serial-sum contributor: admission, identity, and the walls
// beside it
// ---------------------------------------------------------------------------

/// `sum(input * 2.0, [cols])` — the pointwise-prologue neighbour of every
/// produced fold below, and the shape the `tiler-build` Metal goldens qualify.
fn pointwise_prologue_fold() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let folded = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), folded)
        .unwrap();
    builder.build().unwrap()
}

/// `sum(input, [cols])` — the declared-input neighbour.
fn declared_input_fold() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let folded = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), folded)
        .unwrap();
    builder.build().unwrap()
}

/// `sum(sum(input, [cols]) * 2.0, [rows])` — a produced fold with a
/// continuation, over the same declaration as the two neighbours above.
fn produced_fold_with_continuation() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let inner = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    let scaled = F32Multiply::apply(&mut builder, inner, scale).unwrap();
    let outer = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), outer)
        .unwrap();
    builder.build().unwrap()
}

/// Renders one encoded subject as lowercase hex, for a pinned comparison.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut rendered, byte| {
        let _ = write!(rendered, "{byte:02x}");
        rendered
    })
}

/// Encodes one program's sole recognized output subject.
fn encoded_subject(program: &SemanticProgram) -> Vec<u8> {
    let recognized = recognize(program).expect("the fixture is recognized");
    let mut bytes = Vec::new();
    encode_output_subject(&mut bytes, &output_subject(&recognized));
    bytes
}

/// The two neighbour arms' `serial-sum-f32.v3` bytes did not move.
///
/// **Pinned to exact bytes rather than compared structurally**, because what the
/// accepted carrier promised is that these two subjects encode to *what they
/// already did* — so the enclosing `tiler.compiler.request-subject.v6` domain
/// does not step, the `domains.rs` pin row stays where it is, and every governed
/// compilation keeps its request qualifier. A structural assertion would pass
/// through exactly the change that breaks that promise.
///
/// The two values below were captured at base `441f3215` — before the
/// contributor source existed — by running this test in a detached worktree at
/// that commit. They are recorded rather than derived for the reason
/// `tiler-build`'s standard Metal pins are: the point is that they do **not**
/// move. A change here is either a deliberate identity revision, which must
/// step the sub-tag and restate every pin in the commit that states why, or the
/// defect this test exists to catch.
#[test]
fn the_declared_input_and_pointwise_prologue_arms_keep_their_exact_bytes() {
    const DECLARED_INPUT: &str = "000000000000001173657269616c2d73756d2d6633322e763300000000000000010000000000000005696e7075740000000000000006726573756c740000000000000002000000000000000200000000000000040000000000000001000000000000000200000000000000010000000100000000000000000000000000000000000000000000000100000000000000000000000800000000000000020000000000000000";
    const POINTWISE_PROLOGUE: &str = "000000000000001173657269616c2d73756d2d6633322e763300000000000000010000000000000005696e7075740000000000000006726573756c74000000000000000200000000000000020000000000000004000000000000000100000000000000020000000000000001000000010000000000000003010000000002400000000400000000000000010000000200000000000000020000000000000001000000000000000100000002000000000000000800000000000000020000000000000000";

    assert_eq!(
        hex(&encoded_subject(&declared_input_fold())),
        DECLARED_INPUT
    );
    assert_eq!(
        hex(&encoded_subject(&pointwise_prologue_fold())),
        POINTWISE_PROLOGUE,
    );
}

/// A produced fold takes its own framed sub-tag, and the *tag* is what separates
/// it — not the unread-declared-input marker run.
///
/// **The forgery is built as bytes rather than inferred.** The subject that a
/// forger would have to write to claim `serial-sum-f32.v3` for a produced fold is
/// constructed here by perturbing only the recognized contributor source and
/// re-encoding, so both byte strings are real encoder output over the same fold.
/// The assertion is then about where they diverge: the framed tag, at position
/// zero, before any payload is read.
///
/// **That distinction is the whole point.** A dropped-producer forgery pushed
/// through the old grammar emits an `encode_elementwise_reads` run of *only*
/// unread markers, which no legal old subject produces — so the two strings would
/// differ even with no tag split, and a test resting on that would pass while the
/// separation rested on an accident of one program's declaration count. Resting
/// identity on an unstated invariant is what `encode_elementwise_reads`'s own
/// comment forbids, so what is asserted is the structural control.
#[test]
fn a_produced_fold_cannot_encode_under_the_old_serial_sum_tag() {
    let framed = |tag: &[u8]| {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, tag);
        bytes
    };
    let produced_tag = framed(b"serial-sum-produced-f32.v1");
    let retained_tag = framed(b"serial-sum-f32.v3");

    let recognized = recognize(&produced_fold_with_continuation()).expect("the produced fold");
    let mut produced = Vec::new();
    encode_output_subject(&mut produced, &output_subject(&recognized));
    assert!(
        produced.starts_with(&produced_tag),
        "the materialized arm must open with its own framed tag",
    );

    // The forgery: the same fold, claiming the neighbour's grammar. Only the
    // contributor source moves, so every other fact the arm writes is the
    // produced fold's own.
    let mut forged = recognized.clone();
    forged.serial_sum_mut().contributor =
        SerialSumContributor::DeclaredInput(DeclaredInputOrdinal::new(0));
    let mut forged_bytes = Vec::new();
    encode_output_subject(&mut forged_bytes, &output_subject(&forged));
    assert!(
        forged_bytes.starts_with(&retained_tag),
        "the forgery must be real encoder output under the tag it claims",
    );
    assert_ne!(produced, forged_bytes);
    // The separation is *inside the framed tag*, so it holds for every produced
    // fold rather than only for the ones whose declaration count happens to emit
    // a marker run. The two tag strings differ in length, so the divergence
    // lands in the eight-byte length prefix — the first field either arm writes,
    // and one no payload can reach past.
    let shared = produced
        .iter()
        .zip(&forged_bytes)
        .take_while(|(left, right)| left == right)
        .count();
    assert!(
        shared < retained_tag.len(),
        "the two arms must diverge inside the framed tag, before any payload; \
         they share {shared} of the retained tag's {} bytes",
        retained_tag.len(),
    );

    // And the producer itself is bound: two produced folds differing only in
    // what writes their contributors are different subjects. Un-repaired, an
    // encoder that dropped the producer would collide them.
    let mut other_producer = recognized.clone();
    let SerialSumContributor::Materialized(materialized) =
        &mut other_producer.serial_sum_mut().contributor
    else {
        panic!("the fixture folds a materialized contributor");
    };
    let NormalizedOutput::SerialSum(inner_fold) = &mut materialized.producer else {
        panic!("the fixture's producer is a fold");
    };
    inner_fold.reduction_axes = vec![Axis::new(0)];
    let mut other_bytes = Vec::new();
    encode_output_subject(&mut other_bytes, &output_subject(&other_producer));
    assert_ne!(
        produced, other_bytes,
        "the producer is written through the recursion, so a different producer is a different subject",
    );
}

/// The continuation's presence is written, so omitting it is a different
/// subject rather than the same one.
///
/// A produced fold whose contributor *is* the produced value carries no
/// continuation; one with an expression between the two carries a presence byte
/// and the epilogue read vocabulary. The two must not share a byte string, or a
/// forgery could drop the continuation and keep the fold's identity.
#[test]
fn a_produced_folds_continuation_presence_is_bound() {
    let with_continuation =
        recognize(&produced_fold_with_continuation()).expect("the produced fold");
    let mut present = Vec::new();
    encode_output_subject(&mut present, &output_subject(&with_continuation));

    let mut without = with_continuation.clone();
    let SerialSumContributor::Materialized(materialized) =
        &mut without.serial_sum_mut().contributor
    else {
        panic!("the fixture folds a materialized contributor");
    };
    materialized.continuation = None;
    let mut absent = Vec::new();
    encode_output_subject(&mut absent, &output_subject(&without));

    assert_ne!(present, absent);
    assert!(
        absent.len() < present.len(),
        "the absent continuation writes its presence byte and nothing else",
    );
    // The presence byte is the last byte of the shorter encoding, and it is
    // `0x00`; the longer one carries `0x01` at that position and the framed
    // expression after it. A forgery that truncated the payload but kept the
    // byte would still be a different subject.
    assert_eq!(absent.last().copied(), Some(0x00));
    assert_eq!(present.get(absent.len() - 1).copied(), Some(0x01));
}

/// A produced fold's `sum(rms_norm(x, w))` shape retains a staged producer and
/// no synthesized continuation, and a produced sum's fold stays resolvable.
#[test]
fn a_produced_folds_partition_claims_the_producer_and_the_continuation() {
    let recognized = recognize(&produced_fold_with_continuation()).expect("the produced fold");
    let NormalizedOutput::SerialSum(fold) = &recognized else {
        panic!("a produced fold recognizes as a serial sum");
    };
    let SerialSumContributor::Materialized(materialized) = &fold.contributor else {
        panic!("the fixture folds a materialized contributor");
    };

    // Every occurrence is claimed exactly once, which is what
    // `check_output_cover` requires: the inner fold, the constant, the multiply,
    // and the outer fold.
    assert_eq!(
        recognized.members().len(),
        produced_fold_with_continuation().operation_count(),
    );

    // The three parts are disjoint and none of them is the pointwise prologue's.
    assert_eq!(fold.prologue_members(), None);
    let continuation = fold
        .continuation_members()
        .expect("the `* 2` is a continuation region");
    assert!(
        !fold
            .members
            .pointwise()
            .iter()
            .any(|atom| continuation.contains(atom)),
        "a continuation member must never enter the declared-input prologue part",
    );
    assert!(
        !fold
            .members
            .all()
            .iter()
            .any(|atom| continuation.contains(atom)),
        "the fused affine candidate is prologue-union-fold, so it must not claim the continuation",
    );
    for part in [
        fold.members.reduction(),
        continuation,
        &materialized.producer.members(),
    ] {
        assert!(
            recognized.owns_region_members(part),
            "each part of the fold's partition must resolve to this output",
        );
    }
    // The continuation union the fold is *not* a part: no scheduled region
    // computes an expression over a staged value and a fold of its result.
    let mut grouped: Vec<SemanticStage> = continuation.to_vec();
    grouped.extend_from_slice(fold.members.reduction());
    grouped.sort_unstable();
    grouped.dedup();
    assert!(
        !recognized.owns_region_members(&grouped),
        "grouping the continuation with the fold is declined, not flattened",
    );
}
