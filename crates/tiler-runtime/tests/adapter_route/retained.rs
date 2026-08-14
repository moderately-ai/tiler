//! Retained shape relations refuse before routing commit.

use std::sync::Arc;

use tiler_artifact::program::{
    AbiFactBinder, AvailabilityPhase, RetainedShapeRelationFailureClass, TargetPropertyKey,
};
use tiler_ir::semantic::SemanticProgramBuilder;
use tiler_ir::shape::{
    Axis, BindingSource, ExtentRelation, ExtentTerm, FactProvenance, RootBinding,
    SemanticInputConstraint, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
};
use tiler_runtime::load::{DecodedProgram, LoadRejection};

use super::SOLE_DELIVERY;
use super::fixture::{FixtureSpec, assemble_portfolio_over, input_key};

fn symbol(name: &str) -> ShapeSymbol {
    ShapeSymbol::new(SymbolScope::new("runtime/retained").unwrap(), name).unwrap()
}

fn term(name: &str) -> ExtentTerm {
    ExtentTerm::Symbol(symbol(name))
}

fn additive_environment(bind: impl Fn(&str) -> RootBinding) -> Arc<tiler_ir::shape::ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
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
    Arc::new(draft.build().unwrap())
}

fn program_over(
    environment: Arc<tiler_ir::shape::ShapeEnv>,
) -> tiler_ir::semantic::SemanticProgram {
    let mut draft =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let input = draft
        .input::<tiler_ir::semantic::F32>(
            input_key(),
            tiler_ir::shape::Shape::from_dims([super::fixture::ROWS, super::fixture::COLUMNS]),
        )
        .unwrap();
    let scale =
        tiler_ir::semantic::F32Constant::apply(&mut draft, super::fixture::SCALE_BITS).unwrap();
    let bias =
        tiler_ir::semantic::F32Constant::apply(&mut draft, super::fixture::BIAS_BITS).unwrap();
    let product = tiler_ir::semantic::F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = tiler_ir::semantic::F32Add::apply(&mut draft, product, bias).unwrap();
    let sum = tiler_ir::semantic::StrictSerialF32Sum::apply(
        &mut draft,
        mapped,
        [tiler_ir::shape::Axis::new(1)],
    )
    .unwrap();
    draft
        .output(tiler_ir::semantic::OutputKey::new("result").unwrap(), sum)
        .unwrap();
    draft.build().unwrap()
}

fn decode_retained(
    environment: Arc<tiler_ir::shape::ShapeEnv>,
) -> (
    DecodedProgram,
    tiler_artifact::program::RecordedArtifactProgramIdentity,
) {
    let semantic = program_over(environment);
    let spec = FixtureSpec {
        route_requirements: Vec::new(),
        deferred_predicates: Vec::new(),
        ..FixtureSpec::default()
    };
    let built = assemble_portfolio_over(&[spec], &semantic);
    let program = DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).unwrap();
    (program, built.expected)
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

fn bind_sct(s: u64, c: u64, t: u64) -> tiler_artifact::program::AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_extent(input_key(), Axis::new(0), s)
        .unwrap();
    binder
        .bind_input_extent(input_key(), Axis::new(1), c)
        .unwrap();
    binder
        .bind_target_property(token_property(), AvailabilityPhase::LiveDevicePreflight, t)
        .unwrap();
    binder.build()
}

fn input_binding(axis: u32) -> RootBinding {
    RootBinding::new(
        BindingSource::InputDimension {
            input: input_key(),
            axis: Axis::new(axis),
        },
        AvailabilityPhase::LiveDevicePreflight,
        FactProvenance::RuntimeValidated,
    )
    .unwrap()
}

fn host() -> tiler_runtime::load::ExecutionEnvironment {
    super::fixture::scalar_host()
}

#[test]
fn an_inconsistent_triple_refuses_before_routing_commit() {
    let environment = additive_environment(|name| match name {
        "S" => input_binding(0),
        "C" => input_binding(1),
        _ => token_binding(),
    });
    let (mut program, expected) = decode_retained(environment);
    let facts = bind_sct(13, 14, 1);
    let rejection = program
        .preflight(&host(), &expected, &facts)
        .expect_err("S = 13, C = 14, T = 1 must refuse before routing commit");
    let LoadRejection::RetainedShapeRelation(failure) = rejection else {
        panic!("expected a retained-shape refusal before any route work, got {rejection}");
    };
    assert_eq!(
        failure.class(),
        RetainedShapeRelationFailureClass::Unsatisfied
    );
    let rendered = failure.to_string();
    assert!(
        rendered.contains('S') && rendered.contains('C') && rendered.contains('T'),
        "{rendered}"
    );
    assert!(
        rendered.contains("13") && rendered.contains("14") && rendered.contains("13 != 14 + 1"),
        "{rendered}"
    );
}

#[test]
fn prepare_refuses_the_same_inconsistent_triple_before_qualification() {
    let environment = additive_environment(|name| match name {
        "S" => input_binding(0),
        "C" => input_binding(1),
        _ => token_binding(),
    });
    let (mut program, expected) = decode_retained(environment);
    let rejection = program
        .prepare(&host(), &expected, &bind_sct(13, 14, 1))
        .expect_err("prepare must refuse before route qualification");
    assert!(
        matches!(rejection, LoadRejection::RetainedShapeRelation(_)),
        "{rejection}"
    );
}

#[test]
fn a_consistent_runtime_binding_reaches_preflight() {
    let environment = additive_environment(|name| match name {
        "S" => input_binding(0),
        "C" => input_binding(1),
        _ => token_binding(),
    });
    let (mut program, expected) = decode_retained(environment);
    let _preflight = program
        .preflight(&host(), &expected, &bind_sct(15, 14, 1))
        .expect("S = 15, C = 14, T = 1 is a legal invocation");
}

#[test]
fn the_ordinary_fixture_still_decodes_with_the_empty_environment() {
    let built = super::fixture::assemble(&FixtureSpec::default());
    DecodedProgram::decode(&built.bytes, SOLE_DELIVERY)
        .expect("the ordinary fixture still decodes after the identity step");
}
