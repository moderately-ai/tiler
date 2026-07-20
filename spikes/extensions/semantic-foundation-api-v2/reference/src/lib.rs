#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use semantic_api_ir::{OpKey, OperationAttributes};

pub trait ReferenceOperation: Send + Sync + 'static {
    fn evaluate(
        &self,
        inputs: &[Vec<u8>],
        attributes: &OperationAttributes,
    ) -> Result<Vec<Vec<u8>>, ReferenceError>;
}

#[derive(Default)]
pub struct ReferenceRegistryBuilder {
    operations: BTreeMap<OpKey, Arc<dyn ReferenceOperation>>,
}

impl ReferenceRegistryBuilder {
    pub fn register(
        &mut self,
        key: OpKey,
        operation: Arc<dyn ReferenceOperation>,
    ) -> Result<(), ReferenceError> {
        if self.operations.insert(key, operation).is_some() {
            return Err(ReferenceError::DuplicateCapability);
        }
        Ok(())
    }

    pub fn freeze(self) -> FrozenReferenceRegistry {
        FrozenReferenceRegistry(Arc::new(self.operations))
    }
}

#[derive(Clone)]
pub struct FrozenReferenceRegistry(Arc<BTreeMap<OpKey, Arc<dyn ReferenceOperation>>>);

impl FrozenReferenceRegistry {
    pub fn evaluate(
        &self,
        key: &OpKey,
        inputs: &[Vec<u8>],
        attributes: &OperationAttributes,
    ) -> Result<Vec<Vec<u8>>, ReferenceError> {
        self.0
            .get(key)
            .ok_or(ReferenceError::MissingCapability)?
            .evaluate(inputs, attributes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceError {
    DuplicateCapability,
    MissingCapability,
}

impl std::fmt::Display for ReferenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ReferenceError {}
