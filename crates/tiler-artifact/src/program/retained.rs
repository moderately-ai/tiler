//! Private lossless artifact carrier of the semantic identity's fifth subject.
//!
//! Construction projects a verified [`ShapeEnv`], regenerates
//! [`ShapeEnvIdentity`] bytes exactly, and refuses a mismatch. The carried
//! bytes are the one authority for both artifact identity and decoded
//! evaluation. Invocation values never enter those bytes.

use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{InputKey, SemanticProgram};
use tiler_ir::shape::{
    Axis, BindingSource, ExtentRelation, ExtentTerm, RootBinding, SemanticInputConstraint,
    ShapeEnv, ShapeEnvBuilder, ShapeEnvIdentity, ShapeEnvSubjectError, ShapeSymbol,
    decode_shape_env_subject, encode_shape_env_subject,
};

use super::error::ArtifactBuildError;
use super::expr::AbiFacts;
use super::model::InterfaceEntryData;

/// Lossless artifact representation of one shape environment's identity subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedShapeEnvironment {
    bindings: Vec<(ShapeSymbol, RootBinding)>,
    constraints: Vec<SemanticInputConstraint>,
    bytes: Vec<u8>,
}

impl RetainedShapeEnvironment {
    /// Projects the verified environment a semantic program carries.
    ///
    /// A program built without an environment contributes the empty subject's
    /// bytes — the same identity `SemanticIdentity` already reports.
    pub(crate) fn project(program: &SemanticProgram) -> Result<Self, ArtifactBuildError> {
        let empty;
        let environment = if let Some(sources) = program.extent_sources() {
            sources.environment()
        } else {
            empty = empty_environment();
            &empty
        };
        Self::from_verified(environment, program.semantic_identity().shape_environment())
    }

    /// Projects one verified environment and refuses any identity mismatch.
    pub(crate) fn from_verified(
        environment: &ShapeEnv,
        expected: &ShapeEnvIdentity,
    ) -> Result<Self, ArtifactBuildError> {
        refuse_interface_parameter(environment.bindings())?;
        let bindings: Vec<(ShapeSymbol, RootBinding)> = environment
            .bindings()
            .map(|(symbol, binding)| (symbol.clone(), binding.clone()))
            .collect();
        let constraints: Vec<SemanticInputConstraint> =
            environment.constraints().cloned().collect();
        let bytes = encode_shape_env_subject(&bindings, &constraints);
        if bytes != expected.as_bytes() || bytes != environment.identity().as_bytes() {
            return Err(ArtifactBuildError::RetainedShapeEnvironmentIdentityMismatch);
        }
        Self::from_canonical_bytes(bytes)
            .map_err(|_| ArtifactBuildError::RetainedShapeEnvironmentIdentityMismatch)
    }

    /// Rebuilds the view from carried bytes and revalidates the encoding.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, ShapeEnvSubjectError> {
        Self::from_canonical_bytes(bytes.to_vec())
    }

    fn from_canonical_bytes(bytes: Vec<u8>) -> Result<Self, ShapeEnvSubjectError> {
        let decoded = decode_shape_env_subject(&bytes)?;
        for (symbol, binding) in &decoded.bindings {
            if let BindingSource::InterfaceParameter { key } = binding.source() {
                return Err(ShapeEnvSubjectError::UnsupportedBindingSource {
                    symbol: symbol.clone(),
                    source: format!("interface-parameter `{key}`"),
                });
            }
        }
        Ok(Self {
            bindings: decoded.bindings,
            constraints: decoded.constraints,
            bytes,
        })
    }

    /// Returns the canonical identity bytes this carrier is the authority for.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the decoded root bindings, in the canonical symbol order.
    ///
    /// The artifact's one shape-environment authority. A live input-extent
    /// operand is associated against exactly these bindings, so a symbol from
    /// any other environment has no row here and fails closed.
    pub(crate) fn bindings(&self) -> &[(ShapeSymbol, RootBinding)] {
        &self.bindings
    }

    /// Evaluates every retained semantic input constraint against bound facts.
    pub(crate) fn evaluate(
        &self,
        facts: &AbiFacts,
        inputs: &[InterfaceEntryData<InputKey>],
    ) -> Result<(), RetainedShapeRelationFailure> {
        for constraint in &self.constraints {
            evaluate_relation(constraint.relation(), &self.bindings, facts, inputs)?;
        }
        Ok(())
    }
}

fn empty_environment() -> ShapeEnv {
    ShapeEnvBuilder::new()
        .build()
        .expect("an environment with no symbol and no constraint is always verifiable")
}

fn refuse_interface_parameter<'a>(
    bindings: impl IntoIterator<Item = (&'a ShapeSymbol, &'a RootBinding)>,
) -> Result<(), ArtifactBuildError> {
    for (symbol, binding) in bindings {
        if let BindingSource::InterfaceParameter { key } = binding.source() {
            return Err(ArtifactBuildError::UnsupportedRetainedBindingSource {
                symbol: symbol.to_string(),
                source: format!("interface-parameter `{key}`"),
            });
        }
    }
    Ok(())
}

fn evaluate_relation(
    relation: &ExtentRelation,
    bindings: &[(ShapeSymbol, RootBinding)],
    facts: &AbiFacts,
    inputs: &[InterfaceEntryData<InputKey>],
) -> Result<(), RetainedShapeRelationFailure> {
    match relation {
        ExtentRelation::Equal { left, right } => {
            let left_value = evaluate_term(left, bindings, facts, inputs)?;
            let right_value = evaluate_term(right, bindings, facts, inputs)?;
            if left_value == right_value {
                Ok(())
            } else {
                Err(unsatisfied(
                    relation,
                    bindings,
                    facts,
                    &[left_value, right_value],
                    &format!("observed {left_value} != {right_value}"),
                ))
            }
        }
        ExtentRelation::AdditiveEquality {
            sum, left, right, ..
        } => {
            let sum_value = evaluate_term(sum, bindings, facts, inputs)?;
            let left_value = evaluate_term(left, bindings, facts, inputs)?;
            let right_value = evaluate_term(right, bindings, facts, inputs)?;
            let added = left_value.checked_add(right_value).ok_or_else(|| {
                overflow(
                    relation,
                    bindings,
                    facts,
                    &[sum_value, left_value, right_value],
                    &format!("{left_value} + {right_value} overflows u64"),
                )
            })?;
            if sum_value == added {
                Ok(())
            } else {
                Err(unsatisfied(
                    relation,
                    bindings,
                    facts,
                    &[sum_value, left_value, right_value],
                    &format!("observed {sum_value} != {left_value} + {right_value}"),
                ))
            }
        }
        ExtentRelation::Divisible { dividend, divisor } => {
            let dividend_value = evaluate_term(dividend, bindings, facts, inputs)?;
            if dividend_value % divisor.get() == 0 {
                Ok(())
            } else {
                Err(unsatisfied(
                    relation,
                    bindings,
                    facts,
                    &[dividend_value],
                    &format!("observed {divisor} does not divide {dividend_value}"),
                ))
            }
        }
        ExtentRelation::NonNegativeDifference {
            minuend,
            subtrahend,
        } => {
            let minuend_value = evaluate_term(minuend, bindings, facts, inputs)?;
            let subtrahend_value = evaluate_term(subtrahend, bindings, facts, inputs)?;
            if minuend_value >= subtrahend_value {
                Ok(())
            } else {
                Err(unsatisfied(
                    relation,
                    bindings,
                    facts,
                    &[minuend_value, subtrahend_value],
                    &format!("observed {minuend_value} - {subtrahend_value} < 0"),
                ))
            }
        }
        ExtentRelation::Interval { term, lower, upper } => {
            let value = evaluate_term(term, bindings, facts, inputs)?;
            if value >= *lower && value <= *upper {
                Ok(())
            } else {
                Err(unsatisfied(
                    relation,
                    bindings,
                    facts,
                    &[value],
                    &format!("observed {value} is outside [{lower}, {upper}]"),
                ))
            }
        }
        ExtentRelation::Factorization { product, factors } => {
            let product_value = evaluate_term(product, bindings, facts, inputs)?;
            let mut factor_values = Vec::with_capacity(factors.len());
            let mut observed = 1_u64;
            for factor in factors {
                let value = evaluate_term(factor, bindings, facts, inputs)?;
                observed = observed.checked_mul(value).ok_or_else(|| {
                    let mut sides = factor_values.clone();
                    sides.push(value);
                    sides.insert(0, product_value);
                    overflow(
                        relation,
                        bindings,
                        facts,
                        &sides,
                        "factor product overflows u64",
                    )
                })?;
                factor_values.push(value);
            }
            if product_value == observed {
                Ok(())
            } else {
                let mut sides = vec![product_value];
                sides.extend_from_slice(&factor_values);
                Err(unsatisfied(
                    relation,
                    bindings,
                    facts,
                    &sides,
                    &format!("observed {product_value} != product of {factor_values:?}"),
                ))
            }
        }
    }
}

fn evaluate_term(
    term: &ExtentTerm,
    bindings: &[(ShapeSymbol, RootBinding)],
    facts: &AbiFacts,
    inputs: &[InterfaceEntryData<InputKey>],
) -> Result<u64, RetainedShapeRelationFailure> {
    match term {
        ExtentTerm::Constant(value) => Ok(*value),
        ExtentTerm::Symbol(symbol) => {
            let binding = bindings
                .iter()
                .find(|(declared, _)| declared == symbol)
                .map(|(_, binding)| binding)
                .expect("decode proved every constrained symbol is declared");
            evaluate_binding(symbol, binding, facts, inputs)
        }
    }
}

fn evaluate_binding(
    symbol: &ShapeSymbol,
    binding: &RootBinding,
    facts: &AbiFacts,
    inputs: &[InterfaceEntryData<InputKey>],
) -> Result<u64, RetainedShapeRelationFailure> {
    match binding.source() {
        BindingSource::Static(extent) => Ok(extent.get()),
        BindingSource::InputDimension { input, axis } => {
            let declared = inputs.iter().find(|entry| entry.key == *input);
            match declared {
                None => Err(RetainedShapeRelationFailure::invalid_domain(
                    symbol,
                    binding.source(),
                    &format!(
                        "input `{}` is not an interface input of this artifact",
                        input.as_str()
                    ),
                )),
                Some(entry) if usize_axis(*axis) >= entry.rank() => {
                    Err(RetainedShapeRelationFailure::invalid_domain(
                        symbol,
                        binding.source(),
                        &format!(
                            "input `{}` axis {} is outside rank {}",
                            input.as_str(),
                            axis.get(),
                            entry.rank()
                        ),
                    ))
                }
                Some(_) => facts.input_extent(input, *axis).ok_or_else(|| {
                    RetainedShapeRelationFailure::missing(
                        symbol,
                        binding.source(),
                        &format!(
                            "input `{}` axis {} is not bound",
                            input.as_str(),
                            axis.get()
                        ),
                    )
                }),
            }
        }
        BindingSource::TargetProperty { key } => facts.target_property(key).ok_or_else(|| {
            RetainedShapeRelationFailure::missing(
                symbol,
                binding.source(),
                &format!("target-property `{key}` is not bound"),
            )
        }),
        BindingSource::InterfaceParameter { key } => {
            Err(RetainedShapeRelationFailure::invalid_domain(
                symbol,
                binding.source(),
                &format!("interface-parameter `{key}` has no authoritative ABI binding"),
            ))
        }
    }
}

fn usize_axis(axis: Axis) -> usize {
    usize::try_from(axis.get()).expect("axis fits every supported host usize")
}

fn unsatisfied(
    relation: &ExtentRelation,
    bindings: &[(ShapeSymbol, RootBinding)],
    facts: &AbiFacts,
    observed: &[u64],
    detail: &str,
) -> RetainedShapeRelationFailure {
    RetainedShapeRelationFailure::with_terms(
        RetainedShapeRelationFailureClass::Unsatisfied,
        relation,
        bindings,
        facts,
        observed,
        detail,
    )
}

fn overflow(
    relation: &ExtentRelation,
    bindings: &[(ShapeSymbol, RootBinding)],
    facts: &AbiFacts,
    observed: &[u64],
    detail: &str,
) -> RetainedShapeRelationFailure {
    RetainedShapeRelationFailure::with_terms(
        RetainedShapeRelationFailureClass::ArithmeticOverflow,
        relation,
        bindings,
        facts,
        observed,
        detail,
    )
}

/// Why one retained shape relation could not be evaluated against bound facts.
///
/// Opaque: the class is the typed cause a caller matches, and [`fmt::Display`]
/// names the relation, every participating symbol and authoritative source, and
/// every observed value available at the failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedShapeRelationFailure {
    class: RetainedShapeRelationFailureClass,
    message: String,
}

impl RetainedShapeRelationFailure {
    /// Returns the typed cause of this failure.
    #[must_use]
    pub const fn class(&self) -> RetainedShapeRelationFailureClass {
        self.class
    }

    fn missing(symbol: &ShapeSymbol, source: &BindingSource, detail: &str) -> Self {
        Self {
            class: RetainedShapeRelationFailureClass::MissingBinding,
            message: format!(
                "retained shape relation missing binding for {symbol} ({}); {detail}",
                render_source(source)
            ),
        }
    }

    fn invalid_domain(symbol: &ShapeSymbol, source: &BindingSource, detail: &str) -> Self {
        Self {
            class: RetainedShapeRelationFailureClass::InvalidBindingDomain,
            message: format!(
                "retained shape relation binding for {symbol} ({}) is outside this artifact's domain; {detail}",
                render_source(source)
            ),
        }
    }

    fn with_terms(
        class: RetainedShapeRelationFailureClass,
        relation: &ExtentRelation,
        bindings: &[(ShapeSymbol, RootBinding)],
        facts: &AbiFacts,
        observed: &[u64],
        detail: &str,
    ) -> Self {
        let mut terms = Vec::new();
        relation.for_each_symbol(|symbol| {
            if terms.iter().any(|(named, _, _)| named == symbol) {
                return;
            }
            let source = bindings
                .iter()
                .find(|(declared, _)| declared == symbol)
                .map(|(_, binding)| binding.source());
            let value = source.and_then(|source| observed_source(source, facts));
            terms.push((symbol.clone(), source.cloned(), value));
        });
        let rendered_terms = terms
            .into_iter()
            .map(|(symbol, source, value)| {
                let source = source
                    .as_ref()
                    .map_or_else(|| "unbound".to_owned(), render_source);
                match value {
                    Some(value) => format!("{symbol} ({source}) = {value}"),
                    None => format!("{symbol} ({source}) unbound"),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let observed_sides = observed
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        Self {
            class,
            message: format!(
                "retained shape relation `{relation}` is {class}: {rendered_terms}; observed sides [{observed_sides}]; {detail}"
            ),
        }
    }
}

fn observed_source(source: &BindingSource, facts: &AbiFacts) -> Option<u64> {
    match source {
        BindingSource::Static(extent) => Some(extent.get()),
        BindingSource::InputDimension { input, axis } => facts.input_extent(input, *axis),
        BindingSource::TargetProperty { key } => facts.target_property(key),
        BindingSource::InterfaceParameter { .. } => None,
    }
}

fn render_source(source: &BindingSource) -> String {
    match source {
        BindingSource::Static(extent) => format!("static {}", extent.get()),
        BindingSource::InputDimension { input, axis } => {
            format!("input `{}` axis {}", input.as_str(), axis.get())
        }
        BindingSource::InterfaceParameter { key } => {
            format!("interface-parameter `{key}`")
        }
        BindingSource::TargetProperty { key } => format!("target-property `{key}`"),
    }
}

impl fmt::Display for RetainedShapeRelationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for RetainedShapeRelationFailure {}

/// Typed cause of a retained-shape-relation evaluation failure.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum RetainedShapeRelationFailureClass {
    /// An authoritative source the relation names was not bound.
    MissingBinding,
    /// A binding named a domain this artifact's interface does not declare.
    InvalidBindingDomain,
    /// Every term was bound and the relation does not hold.
    Unsatisfied,
    /// Unsigned 64-bit arithmetic overflowed while evaluating the relation.
    ArithmeticOverflow,
}

impl fmt::Display for RetainedShapeRelationFailureClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingBinding => "missing-binding",
            Self::InvalidBindingDomain => "invalid-binding-domain",
            Self::Unsatisfied => "unsatisfied",
            Self::ArithmeticOverflow => "arithmetic-overflow",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::mem::variant_count;
    use std::sync::Arc;

    use tiler_ir::program::abi::TargetPropertyKey;
    use tiler_ir::semantic::{InputKey, SemanticProgramBuilder};
    use tiler_ir::shape::{
        Axis, BindingSource, Extent, ExtentRelation, ExtentTerm, FactProvenance, RootBinding,
        SemanticInputConstraint, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
        decode_shape_env_subject, encode_shape_env_subject,
    };

    use super::{RetainedShapeEnvironment, RetainedShapeRelationFailureClass, empty_environment};
    use crate::program::facts::AbiFactBinder;
    use crate::program::tests::{
        SCALE_BITS, build_artifact, build_graph, fused_program, lowering_provider, semantic_program,
    };
    use crate::program::{
        ArtifactBuildError, ArtifactProgramBuilder, AvailabilityPhase, CompilationEnvironment,
        decode_artifact,
    };

    fn symbol(name: &str) -> ShapeSymbol {
        ShapeSymbol::new(SymbolScope::new("artifact/retained").unwrap(), name).unwrap()
    }

    fn term(name: &str) -> ExtentTerm {
        ExtentTerm::Symbol(symbol(name))
    }

    fn static_binding(value: u64) -> RootBinding {
        RootBinding::new(
            BindingSource::Static(Extent::new(value)),
            AvailabilityPhase::CompileProfile,
            FactProvenance::StaticallyProven,
        )
        .unwrap()
    }

    fn input_binding(axis: u32) -> RootBinding {
        RootBinding::new(
            BindingSource::InputDimension {
                input: InputKey::new("input").unwrap(),
                axis: Axis::new(axis),
            },
            AvailabilityPhase::LiveDevicePreflight,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    fn token_property() -> TargetPropertyKey {
        TargetPropertyKey::new("tiler.target.test.t@1").unwrap()
    }

    fn token_binding() -> RootBinding {
        RootBinding::new(
            BindingSource::TargetProperty {
                key: token_property(),
            },
            AvailabilityPhase::LiveDevicePreflight,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    fn runtime_sct() -> Arc<tiler_ir::shape::ShapeEnv> {
        let mut draft = ShapeEnvBuilder::new();
        declare_sct(&mut draft, |name| match name {
            "S" => input_binding(0),
            "C" => input_binding(1),
            _ => token_binding(),
        });
        Arc::new(draft.build().unwrap())
    }

    fn bind_sct(s: u64, c: u64, t: u64) -> crate::program::AbiFacts {
        let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
        binder
            .bind_input_extent(InputKey::new("input").unwrap(), Axis::new(0), s)
            .unwrap();
        binder
            .bind_input_extent(InputKey::new("input").unwrap(), Axis::new(1), c)
            .unwrap();
        binder
            .bind_target_property(token_property(), AvailabilityPhase::LiveDevicePreflight, t)
            .unwrap();
        binder.build()
    }

    fn declare_sct(draft: &mut ShapeEnvBuilder, bind: impl Fn(&str) -> RootBinding) {
        for name in ["S", "C", "T"] {
            let declared = symbol(name);
            draft.declare(declared.clone()).unwrap();
            draft.bind(&declared, bind(name)).unwrap();
        }
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::additive_equality(term("S"), term("C"), term("T")),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
    }

    fn program_over(
        environment: Arc<tiler_ir::shape::ShapeEnv>,
    ) -> tiler_ir::semantic::SemanticProgram {
        build_graph(
            SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap(),
        )
    }

    fn artifact_over(
        environment: Arc<tiler_ir::shape::ShapeEnv>,
    ) -> crate::program::VerifiedArtifactProgram {
        let semantic = program_over(environment);
        let program = fused_program(&semantic, SCALE_BITS);
        let provider = lowering_provider(1);
        build_artifact(&semantic, &program, provider.clone(), &[provider])
    }

    fn decoded_over(
        environment: Arc<tiler_ir::shape::ShapeEnv>,
    ) -> crate::program::DecodedArtifact {
        let artifact = artifact_over(environment);
        decode_artifact(&artifact.encode().unwrap()).unwrap()
    }

    #[test]
    fn the_failure_class_table_is_sized_from_the_type() {
        const CLASSES: [RetainedShapeRelationFailureClass;
            variant_count::<RetainedShapeRelationFailureClass>()] = [
            RetainedShapeRelationFailureClass::MissingBinding,
            RetainedShapeRelationFailureClass::InvalidBindingDomain,
            RetainedShapeRelationFailureClass::Unsatisfied,
            RetainedShapeRelationFailureClass::ArithmeticOverflow,
        ];
        assert_eq!(CLASSES.len(), 4);
    }

    #[test]
    fn a_static_neighbour_s_equals_c_plus_t_passes() {
        let mut draft = ShapeEnvBuilder::new();
        declare_sct(&mut draft, |name| match name {
            "S" => static_binding(15),
            "C" => static_binding(14),
            _ => static_binding(1),
        });
        let decoded = decoded_over(Arc::new(draft.build().unwrap()));
        let facts = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build();
        decoded
            .evaluate_retained_shape_relations(&facts)
            .expect("S = 15, C = 14, T = 1 holds statically");
    }

    #[test]
    fn a_runtime_bound_neighbour_s_equals_c_plus_t_passes() {
        let decoded = decoded_over(runtime_sct());
        decoded
            .evaluate_retained_shape_relations(&bind_sct(15, 14, 1))
            .expect("S = 15, C = 14, T = 1 holds at the invocation");
    }

    #[test]
    fn an_inconsistent_triple_names_every_term_and_observed_side() {
        let decoded = decoded_over(runtime_sct());
        let error = decoded
            .evaluate_retained_shape_relations(&bind_sct(13, 14, 1))
            .expect_err("S = 13, C = 14, T = 1 cannot hold");
        assert_eq!(
            error.class(),
            RetainedShapeRelationFailureClass::Unsatisfied
        );
        let rendered = error.to_string();
        assert!(
            rendered.contains('S') && rendered.contains('C') && rendered.contains('T'),
            "the refusal must name all three terms: {rendered}"
        );
        assert!(
            rendered.contains("13") && rendered.contains("14") && rendered.contains('1'),
            "the refusal must name the observed sides: {rendered}"
        );
        assert!(
            rendered.contains("13 != 14 + 1"),
            "the refusal must name the unsatisfied arithmetic: {rendered}"
        );
    }

    #[test]
    fn a_missing_binding_is_its_own_class() {
        let decoded = decoded_over(runtime_sct());
        let facts = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build();
        let error = decoded
            .evaluate_retained_shape_relations(&facts)
            .expect_err("an unbound input axis is a missing binding");
        assert_eq!(
            error.class(),
            RetainedShapeRelationFailureClass::MissingBinding
        );
        assert!(
            error.to_string().contains("input `input` axis"),
            "{}",
            error
        );
    }

    #[test]
    fn a_wrong_domain_binding_is_its_own_class() {
        let mut draft = ShapeEnvBuilder::new();
        let s = symbol("S");
        draft.declare(s.clone()).unwrap();
        draft
            .bind(
                &s,
                RootBinding::new(
                    BindingSource::InputDimension {
                        input: InputKey::new("absent").unwrap(),
                        axis: Axis::new(0),
                    },
                    AvailabilityPhase::LiveDevicePreflight,
                    FactProvenance::RuntimeValidated,
                )
                .unwrap(),
            )
            .unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::equal(term("S"), ExtentTerm::Constant(4)),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        let decoded = decoded_over(Arc::new(draft.build().unwrap()));
        let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
        binder
            .bind_input_extent(InputKey::new("absent").unwrap(), Axis::new(0), 4)
            .unwrap();
        let error = decoded
            .evaluate_retained_shape_relations(&binder.build())
            .expect_err("a foreign input key is the wrong domain");
        assert_eq!(
            error.class(),
            RetainedShapeRelationFailureClass::InvalidBindingDomain
        );
        assert!(error.to_string().contains("absent"), "{}", error);
    }

    #[test]
    fn an_unsupported_relation_tag_refuses_as_an_unsupported_artifact() {
        let mut draft = ShapeEnvBuilder::new();
        declare_sct(&mut draft, |name| match name {
            "S" => static_binding(15),
            "C" => static_binding(14),
            _ => static_binding(1),
        });
        let environment = draft.build().unwrap();
        let mut bytes = encode_shape_env_subject(
            &environment
                .bindings()
                .map(|(symbol, binding)| (symbol.clone(), binding.clone()))
                .collect::<Vec<_>>(),
            &environment.constraints().cloned().collect::<Vec<_>>(),
        );
        let relation_tag = bytes
            .iter()
            .rposition(|byte| *byte == 0x06)
            .expect("the encoded additive relation carries tag 0x06");
        bytes[relation_tag] = 0x07;
        let error = decode_shape_env_subject(&bytes).expect_err("0x07 is not an admitted relation");
        assert!(
            matches!(
                error,
                tiler_ir::shape::ShapeEnvSubjectError::UnknownRelation { tag: 0x07 }
            ),
            "{error:?}"
        );
        let wrapped = RetainedShapeEnvironment::from_bytes(&bytes)
            .expect_err("the artifact carrier refuses the same tag");
        assert!(
            matches!(
                wrapped,
                tiler_ir::shape::ShapeEnvSubjectError::UnknownRelation { tag: 0x07 }
            ),
            "{wrapped:?}"
        );
    }

    #[test]
    fn invocation_bindings_do_not_enter_artifact_identity() {
        let decoded = decoded_over(runtime_sct());
        let identity = decoded.identity();
        decoded
            .evaluate_retained_shape_relations(&bind_sct(15, 14, 1))
            .unwrap();
        decoded
            .evaluate_retained_shape_relations(&bind_sct(16, 15, 1))
            .unwrap();
        assert_eq!(
            decoded.identity().as_bytes(),
            identity.as_bytes(),
            "changing C, T, and S must not mint a second artifact"
        );
    }

    #[test]
    fn two_fixed_interface_programs_differ_by_an_unused_environment() {
        let mut draft = ShapeEnvBuilder::new();
        declare_sct(&mut draft, |name| match name {
            "S" => static_binding(15),
            "C" => static_binding(14),
            _ => static_binding(1),
        });
        let with_env = artifact_over(Arc::new(draft.build().unwrap()));
        let empty = artifact_over(Arc::new(empty_environment()));
        let ordinary = {
            let semantic = semantic_program();
            let program = fused_program(&semantic, SCALE_BITS);
            let provider = lowering_provider(1);
            build_artifact(&semantic, &program, provider.clone(), &[provider])
        };
        assert_ne!(
            with_env.canonical_identity(),
            empty.canonical_identity(),
            "an unused retained environment is identity-bearing"
        );
        assert_eq!(
            empty.canonical_identity(),
            ordinary.canonical_identity(),
            "no environment and the empty environment remain one subject"
        );
    }

    #[test]
    fn an_interface_parameter_binding_refuses_at_construction() {
        let mut draft = ShapeEnvBuilder::new();
        let n = symbol("n");
        draft.declare(n.clone()).unwrap();
        draft
            .bind(
                &n,
                RootBinding::new(
                    BindingSource::InterfaceParameter {
                        key: tiler_ir::shape::InterfaceParameterKey::new("n").unwrap(),
                    },
                    AvailabilityPhase::LaunchPreflight,
                    FactProvenance::RuntimeValidated,
                )
                .unwrap(),
            )
            .unwrap();
        let semantic = program_over(Arc::new(draft.build().unwrap()));
        let provider = lowering_provider(1);
        let environment = CompilationEnvironment::new([provider]).unwrap();
        assert!(
            matches!(
                ArtifactProgramBuilder::new(&semantic, environment),
                Err(ArtifactBuildError::UnsupportedRetainedBindingSource { .. })
            ),
            "InterfaceParameter has no ABI binding yet"
        );
    }

    #[test]
    fn a_target_property_binding_evaluates_from_bound_facts() {
        let mut draft = ShapeEnvBuilder::new();
        let width = symbol("width");
        draft.declare(width.clone()).unwrap();
        draft
            .bind(
                &width,
                RootBinding::new(
                    BindingSource::TargetProperty {
                        key: TargetPropertyKey::new("tiler.target.test@1").unwrap(),
                    },
                    AvailabilityPhase::LiveDevicePreflight,
                    FactProvenance::RuntimeValidated,
                )
                .unwrap(),
            )
            .unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::equal(term("width"), ExtentTerm::Constant(8)),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        let decoded = decoded_over(Arc::new(draft.build().unwrap()));
        let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
        binder
            .bind_target_property(
                TargetPropertyKey::new("tiler.target.test@1").unwrap(),
                AvailabilityPhase::LiveDevicePreflight,
                8,
            )
            .unwrap();
        decoded
            .evaluate_retained_shape_relations(&binder.build())
            .expect("a bound target property satisfies the retained equality");
    }
}
