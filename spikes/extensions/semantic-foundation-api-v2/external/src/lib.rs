#![forbid(unsafe_code)]

use std::sync::Arc;

use semantic_api_ir::*;
use semantic_api_reference::{ReferenceError, ReferenceOperation, ReferenceRegistryBuilder};

pub enum F32 {}
impl ValueTypeMarker for F32 {}

pub enum ExternalF8 {}
impl ValueTypeMarker for ExternalF8 {}

pub fn f32_type() -> ResolvedValueType {
    ResolvedValueType::Nominal(TypeKey("tiler::f32@1"))
}

pub fn external_f8_type() -> ResolvedValueType {
    ResolvedValueType::Nominal(TypeKey("acme::f8@1"))
}

struct ExactNominal(TypeKey);
impl TypeAuthority for ExactNominal {
    fn validate(&self, value: &ResolvedValueType, _: &FrozenSemanticRegistry) -> Result<(), Error> {
        (value == &ResolvedValueType::Nominal(self.0.clone()))
            .then_some(())
            .ok_or(Error::InvalidInstance)
    }
}

struct ComplexAuthority;
impl TypeAuthority for ComplexAuthority {
    fn validate(
        &self,
        value: &ResolvedValueType,
        registry: &FrozenSemanticRegistry,
    ) -> Result<(), Error> {
        let ResolvedValueType::Parameterized { arguments, .. } = value else {
            return Err(Error::InvalidInstance);
        };
        let [CanonicalValue::Type(component)] = arguments.0.as_slice() else {
            return Err(Error::InvalidInstance);
        };
        registry.validate_type(component)
    }
}

struct EncodedAuthority;
impl TypeAuthority for EncodedAuthority {
    fn validate(
        &self,
        value: &ResolvedValueType,
        registry: &FrozenSemanticRegistry,
    ) -> Result<(), Error> {
        let ResolvedValueType::EncodedNumeric { contract, .. } = value else {
            return Err(Error::InvalidInstance);
        };
        let Some((_, CanonicalValue::Type(storage))) = contract.0.first() else {
            return Err(Error::InvalidInstance);
        };
        registry.validate_type(storage)
    }
}

pub const ADD: OpKey = OpKey("tiler::add-rne@1");

struct Add;
impl OperationDefinition for Add {
    fn key(&self) -> OpKey {
        ADD
    }

    fn infer(
        &self,
        operands: &[ResolvedValueType],
        _: &OperationAttributes,
    ) -> Result<Vec<ResolvedValueType>, Error> {
        let [left, right] = operands else {
            return Err(Error::InvalidInstance);
        };
        (left == right)
            .then(|| vec![left.clone()])
            .ok_or(Error::TypeMismatch)
    }
}

pub struct StandardProvider;
impl SemanticProvider for StandardProvider {
    fn register(&self, registrar: &mut SemanticRegistrar<'_>) -> Result<(), Error> {
        for (key, authority) in [
            (
                TypeKey("tiler::f32@1"),
                Arc::new(ExactNominal(TypeKey("tiler::f32@1"))) as Arc<dyn TypeAuthority>,
            ),
            (
                TypeKey("tiler::i8@1"),
                Arc::new(ExactNominal(TypeKey("tiler::i8@1"))) as Arc<dyn TypeAuthority>,
            ),
        ] {
            registrar.register_type(TypeDefinition {
                key: TypeDefinitionKey::Nominal(key),
                normative: NormativeDefinitionRef("governed primitive"),
                facts: TypeDefinitionFacts(CanonicalValue::Bool(true)),
                authority,
            });
        }
        registrar.register_type(TypeDefinition {
            key: TypeDefinitionKey::Constructor(TypeKey("tiler::complex@1")),
            normative: NormativeDefinitionRef("governed complex constructor"),
            facts: TypeDefinitionFacts(CanonicalValue::Bool(true)),
            authority: Arc::new(ComplexAuthority),
        });
        registrar.register_type(TypeDefinition {
            key: TypeDefinitionKey::EncodedScheme(QuantSchemeKey("tiler::affine@1")),
            normative: NormativeDefinitionRef("governed affine scheme"),
            facts: TypeDefinitionFacts(CanonicalValue::Bool(true)),
            authority: Arc::new(EncodedAuthority),
        });
        registrar.bind_marker::<F32>(f32_type());
        registrar.register_operation(Arc::new(Add));
        Ok(())
    }
}

pub struct ExternalProvider;
impl SemanticProvider for ExternalProvider {
    fn register(&self, registrar: &mut SemanticRegistrar<'_>) -> Result<(), Error> {
        registrar.register_type(TypeDefinition {
            key: TypeDefinitionKey::Nominal(TypeKey("acme::f8@1")),
            normative: NormativeDefinitionRef("acme f8 v1"),
            facts: TypeDefinitionFacts(CanonicalValue::Unsigned(8)),
            authority: Arc::new(ExactNominal(TypeKey("acme::f8@1"))),
        });
        registrar.bind_marker::<ExternalF8>(external_f8_type());
        Ok(())
    }
}

struct AddReference;
impl ReferenceOperation for AddReference {
    fn evaluate(
        &self,
        inputs: &[Vec<u8>],
        _: &OperationAttributes,
    ) -> Result<Vec<Vec<u8>>, ReferenceError> {
        Ok(inputs.to_vec())
    }
}

pub fn register_reference(builder: &mut ReferenceRegistryBuilder) -> Result<(), ReferenceError> {
    builder.register(ADD, Arc::new(AddReference))
}

pub fn add<T: ValueTypeMarker>(
    graph: &mut SemanticProgramBuilder,
    left: Value<T>,
    right: Value<T>,
) -> Result<Value<T>, Error> {
    let result = graph.apply(
        &ADD,
        &[left.erase(), right.erase()],
        &OperationAttributes(vec![]),
    )?;
    graph.reify(result[0])
}
