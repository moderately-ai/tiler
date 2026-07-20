#![forbid(unsafe_code)]

use std::any::TypeId;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeKey(pub &'static str);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QuantSchemeKey(pub &'static str);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalValue {
    Bool(bool),
    Unsigned(u64),
    Type(ResolvedValueType),
    Record(Vec<(u32, CanonicalValue)>),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeArguments(pub Vec<CanonicalValue>);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EncodedNumericContract(pub Vec<(u32, CanonicalValue)>);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeDefinitionFacts(pub CanonicalValue);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OperationAttributes(pub Vec<(u32, CanonicalValue)>);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ResolvedValueType {
    Nominal(TypeKey),
    Parameterized {
        constructor: TypeKey,
        arguments: TypeArguments,
    },
    EncodedNumeric {
        scheme: QuantSchemeKey,
        contract: EncodedNumericContract,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormativeDefinitionRef(pub &'static str);

pub trait TypeAuthority: Send + Sync + 'static {
    fn validate(
        &self,
        value: &ResolvedValueType,
        registry: &FrozenSemanticRegistry,
    ) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct TypeDefinition {
    pub key: TypeDefinitionKey,
    pub normative: NormativeDefinitionRef,
    pub facts: TypeDefinitionFacts,
    pub authority: Arc<dyn TypeAuthority>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TypeDefinitionKey {
    Nominal(TypeKey),
    Constructor(TypeKey),
    EncodedScheme(QuantSchemeKey),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OpKey(pub &'static str);

pub trait OperationDefinition: Send + Sync + 'static {
    fn key(&self) -> OpKey;
    fn infer(
        &self,
        operands: &[ResolvedValueType],
        attributes: &OperationAttributes,
    ) -> Result<Vec<ResolvedValueType>, Error>;
}

#[derive(Clone)]
struct OperationRegistration {
    definition: Arc<dyn OperationDefinition>,
}

pub trait ValueTypeMarker: 'static {}

pub trait SemanticProvider: Send + Sync + 'static {
    fn register(&self, registrar: &mut SemanticRegistrar<'_>) -> Result<(), Error>;
}

#[derive(Default)]
pub struct SemanticRegistryBuilder {
    types: BTreeMap<TypeDefinitionKey, TypeDefinition>,
    operations: BTreeMap<OpKey, OperationRegistration>,
    markers: BTreeMap<TypeId, ResolvedValueType>,
}

#[derive(Default)]
struct RegistrationBatch {
    types: Vec<TypeDefinition>,
    operations: Vec<Arc<dyn OperationDefinition>>,
    markers: Vec<(TypeId, ResolvedValueType)>,
}

pub struct SemanticRegistrar<'a> {
    batch: &'a mut RegistrationBatch,
}

impl SemanticRegistrar<'_> {
    pub fn register_type(&mut self, definition: TypeDefinition) {
        self.batch.types.push(definition);
    }

    pub fn register_operation(&mut self, definition: Arc<dyn OperationDefinition>) {
        self.batch.operations.push(definition);
    }

    pub fn bind_marker<T: ValueTypeMarker>(&mut self, value: ResolvedValueType) {
        self.batch.markers.push((TypeId::of::<T>(), value));
    }
}

impl SemanticRegistryBuilder {
    pub fn register_provider(&mut self, provider: &dyn SemanticProvider) -> Result<(), Error> {
        let mut batch = RegistrationBatch::default();
        provider.register(&mut SemanticRegistrar { batch: &mut batch })?;
        let mut candidate = Self {
            types: self.types.clone(),
            operations: self.operations.clone(),
            markers: self.markers.clone(),
        };
        for definition in batch.types {
            if candidate
                .types
                .insert(definition.key.clone(), definition)
                .is_some()
            {
                return Err(Error::DuplicateAuthority);
            }
        }
        for definition in batch.operations {
            let key = definition.key();
            if candidate
                .operations
                .insert(key, OperationRegistration { definition })
                .is_some()
            {
                return Err(Error::DuplicateAuthority);
            }
        }
        for (marker, value) in batch.markers {
            if candidate.markers.insert(marker, value).is_some() {
                return Err(Error::DuplicateMarker);
            }
        }
        *self = candidate;
        Ok(())
    }

    pub fn freeze(self) -> Result<FrozenSemanticRegistry, Error> {
        let registry = FrozenSemanticRegistry(Arc::new(FrozenRegistryData {
            types: self.types,
            operations: self.operations,
            markers: self.markers,
        }));
        for value in registry.0.markers.values() {
            registry.validate_type(value)?;
        }
        Ok(registry)
    }
}

#[derive(Clone)]
pub struct FrozenSemanticRegistry(Arc<FrozenRegistryData>);

struct FrozenRegistryData {
    types: BTreeMap<TypeDefinitionKey, TypeDefinition>,
    operations: BTreeMap<OpKey, OperationRegistration>,
    markers: BTreeMap<TypeId, ResolvedValueType>,
}

impl FrozenSemanticRegistry {
    pub fn resolve_marker<T: ValueTypeMarker>(&self) -> Result<ResolvedValueType, Error> {
        self.0
            .markers
            .get(&TypeId::of::<T>())
            .cloned()
            .ok_or(Error::UnregisteredMarker)
    }

    pub fn validate_type(&self, value: &ResolvedValueType) -> Result<(), Error> {
        let key = match value {
            ResolvedValueType::Nominal(key) => TypeDefinitionKey::Nominal(key.clone()),
            ResolvedValueType::Parameterized { constructor, .. } => {
                TypeDefinitionKey::Constructor(constructor.clone())
            }
            ResolvedValueType::EncodedNumeric { scheme, .. } => {
                TypeDefinitionKey::EncodedScheme(scheme.clone())
            }
        };
        self.0
            .types
            .get(&key)
            .ok_or(Error::UnknownTypeAuthority)?
            .authority
            .validate(value, self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ValueId(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Value<T> {
    id: ValueId,
    marker: PhantomData<fn() -> T>,
}

impl<T> Value<T> {
    pub fn erase(self) -> ValueId {
        self.id
    }
}

pub struct SemanticProgramBuilder {
    registry: FrozenSemanticRegistry,
    values: Vec<ResolvedValueType>,
}

impl SemanticProgramBuilder {
    pub fn new(registry: FrozenSemanticRegistry) -> Self {
        Self {
            registry,
            values: Vec::new(),
        }
    }

    pub fn input<T: ValueTypeMarker>(&mut self) -> Result<Value<T>, Error> {
        let value_type = self.registry.resolve_marker::<T>()?;
        let id = self.input_resolved(value_type)?;
        Ok(Value {
            id,
            marker: PhantomData,
        })
    }

    pub fn input_resolved(&mut self, value_type: ResolvedValueType) -> Result<ValueId, Error> {
        self.registry.validate_type(&value_type)?;
        let id = ValueId(self.values.len());
        self.values.push(value_type);
        Ok(id)
    }

    pub fn apply(
        &mut self,
        key: &OpKey,
        operands: &[ValueId],
        attributes: &OperationAttributes,
    ) -> Result<Vec<ValueId>, Error> {
        let definition = Arc::clone(
            &self
                .registry
                .0
                .operations
                .get(key)
                .ok_or(Error::UnknownOperation)?
                .definition,
        );
        let operand_types = operands
            .iter()
            .map(|value| self.values.get(value.0).cloned().ok_or(Error::ForeignValue))
            .collect::<Result<Vec<_>, _>>()?;
        let result_types = definition.infer(&operand_types, attributes)?;
        let mut results = Vec::with_capacity(result_types.len());
        for result_type in result_types {
            self.registry.validate_type(&result_type)?;
            let id = ValueId(self.values.len());
            self.values.push(result_type);
            results.push(id);
        }
        Ok(results)
    }

    pub fn reify<T: ValueTypeMarker>(&self, id: ValueId) -> Result<Value<T>, Error> {
        let expected = self.registry.resolve_marker::<T>()?;
        if self.values.get(id.0) != Some(&expected) {
            return Err(Error::TypeMismatch);
        }
        Ok(Value {
            id,
            marker: PhantomData,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    DuplicateAuthority,
    DuplicateMarker,
    ForeignValue,
    InvalidInstance,
    TypeMismatch,
    UnknownOperation,
    UnknownTypeAuthority,
    UnregisteredMarker,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for Error {}
