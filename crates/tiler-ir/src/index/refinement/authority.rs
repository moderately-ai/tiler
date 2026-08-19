//! The lowering authority one realization family is admitted under.
//!
//! An authority is the dependency-neutral statement of what a provider is
//! allowed to emit for one operation and signature: the projected semantic
//! capability, the deduplicated scalar operations its region may reach, and the
//! definitions those resolve to. It is admitted once against frozen registries
//! and then compared, so a lowering that reaches outside it is refused by the
//! verifier rather than by whatever the region happened to contain.

use core::fmt;
use std::sync::Arc;

use crate::index::{CanonicalScalarDefinitionProjection, FrozenScalarRegistry, ScalarOpKey};
use crate::semantic::{FrozenSemanticRegistry, OpKey, SemanticCapabilityAuthority};

use super::MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS;
use super::error::IndexRefinementVerificationError;
use super::identity::encode_authority_identity;
use super::registry::semantic_authorities_cohere;
use super::subject::IndexRefinementSignature;

/// Dependency-neutral admitted authority for one lowering realization family.
#[derive(Clone)]
pub struct IndexRealizationAuthority {
    pub(super) operation: OpKey,
    pub(super) signature: IndexRefinementSignature,
    semantic: SemanticCapabilityAuthority,
    pub(super) emitted_scalar_operations: Vec<ScalarOpKey>,
    emitted_scalar_definitions: CanonicalScalarDefinitionProjection,
    pub(super) semantic_registry: FrozenSemanticRegistry,
    pub(super) scalar_registry: FrozenScalarRegistry,
    pub(super) realization_law_row: Option<Box<[u8]>>,
    identity: Box<[u8]>,
}

impl fmt::Debug for IndexRealizationAuthority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IndexRealizationAuthority")
            .field("operation", &self.operation)
            .field("signature", &self.signature)
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

impl PartialEq for IndexRealizationAuthority {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for IndexRealizationAuthority {}

impl IndexRealizationAuthority {
    /// Admits one exact operation/signature and scalar-emission ceiling.
    ///
    /// # Errors
    ///
    /// Returns a typed authority error when the operation/signature projection
    /// or an emitted scalar operation is absent from the supplied registries.
    pub fn admit(
        semantic: &crate::semantic::FrozenSemanticRegistry,
        scalars: &FrozenScalarRegistry,
        operation: OpKey,
        signature: IndexRefinementSignature,
        emitted: &[ScalarOpKey],
    ) -> Result<Self, IndexRefinementVerificationError> {
        if !semantic_authorities_cohere(semantic, scalars.semantic_authority()) {
            return Err(IndexRefinementVerificationError::ScalarSemanticAuthorityMismatch);
        }
        if emitted.len() > MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS {
            return Err(
                IndexRefinementVerificationError::EmittedScalarOperationsTooLarge {
                    actual: emitted.len(),
                    limit: MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS,
                },
            );
        }
        let operation_authority = semantic
            .project_operation_authority(
                &operation,
                signature.operands.iter(),
                signature.results.iter(),
            )
            .map_err(|source| {
                IndexRefinementVerificationError::SemanticAuthority(Arc::new(source))
            })?;
        let realization_law_row = semantic.encode_index_realization_law_row_for(&operation);
        let mut emitted_scalar_operations = emitted.to_vec();
        emitted_scalar_operations.sort_unstable();
        emitted_scalar_operations.dedup();
        let emitted_scalar_definitions = scalars
            .project_reached(emitted_scalar_operations.iter())
            .map_err(|source| {
                IndexRefinementVerificationError::ScalarAuthority(Arc::new(source))
            })?;
        let identity = encode_authority_identity(
            &operation,
            &signature,
            &operation_authority,
            &emitted_scalar_definitions,
            scalars.snapshot_identity().as_bytes(),
            realization_law_row.as_deref(),
        )
        .into_boxed_slice();
        Ok(Self {
            operation,
            signature,
            semantic: operation_authority,
            emitted_scalar_operations,
            emitted_scalar_definitions,
            semantic_registry: semantic.clone(),
            scalar_registry: scalars.clone(),
            realization_law_row,
            identity,
        })
    }
    /// Returns the admitted operation.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }
    /// Returns the admitted signature.
    #[must_use]
    pub const fn signature(&self) -> &IndexRefinementSignature {
        &self.signature
    }
    /// Returns semantic authority.
    #[must_use]
    pub const fn semantic_authority(&self) -> &SemanticCapabilityAuthority {
        &self.semantic
    }
    /// Returns permitted emitted scalar operations.
    #[must_use]
    pub fn emitted_scalar_operations(&self) -> &[ScalarOpKey] {
        &self.emitted_scalar_operations
    }
    /// Returns provider-independent emitted definitions.
    #[must_use]
    pub const fn emitted_scalar_definitions(&self) -> &CanonicalScalarDefinitionProjection {
        &self.emitted_scalar_definitions
    }
}
