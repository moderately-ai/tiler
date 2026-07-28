//! Canonical identity encoding for the frozen reference registry.
//!
//! The exact length is computed before a byte is written, so the encoders
//! and the length functions here are one pair: changing either without the
//! other trips the `debug_assert` at the end of
//! [`compute_reference_identity`] rather than silently moving an identity.

use std::collections::BTreeMap;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::semantic::{
    FrozenSemanticRegistry, OpKey, ProviderIdentity, ResolvedValueType, SemanticCapabilityAuthority,
};

use super::MAX_REFERENCE_REGISTRY_IDENTITY_BYTES;
use super::error::{ReferenceRegistryError, ReferenceRegistryResource};
use super::registry::{
    CanonicalReferenceRegistryIdentity, ReferenceCapabilityKey, ReferenceCapabilityRevision,
    ReferenceSignature, RegisteredReferenceCapability, RegisteredReferenceValueValidator,
};

pub(crate) fn compute_reference_identity(
    semantic_registry: &FrozenSemanticRegistry,
    capabilities: &BTreeMap<ReferenceCapabilityKey, RegisteredReferenceCapability>,
    value_validators: &BTreeMap<ResolvedValueType, RegisteredReferenceValueValidator>,
    exact_len: usize,
) -> CanonicalReferenceRegistryIdentity {
    let mut bytes = Vec::with_capacity(exact_len);
    bytes.extend_from_slice(b"tiler.reference-registry.v2\0");
    push_slice(&mut bytes, semantic_registry.snapshot_identity().as_bytes());
    push_len(&mut bytes, value_validators.len());
    for (resolved_type, validator) in value_validators {
        push_slice(&mut bytes, resolved_type.canonical_encoding().as_bytes());
        encode_reference_authority(&mut bytes, &validator.semantic_authority);
        encode_provider_capability(&mut bytes, &validator.provider, validator.revision);
    }
    push_len(&mut bytes, capabilities.len());
    for (key, capability) in capabilities {
        encode_op_key(&mut bytes, &key.operation);
        encode_signature(&mut bytes, &key.signature);
        encode_reference_authority(&mut bytes, &capability.semantic_authority);
        encode_provider_capability(&mut bytes, &capability.provider, capability.revision);
    }
    debug_assert_eq!(bytes.len(), exact_len);
    CanonicalReferenceRegistryIdentity(bytes)
}

fn encode_reference_authority(output: &mut Vec<u8>, authority: &SemanticCapabilityAuthority) {
    push_slice(output, authority.reached_definitions().as_bytes());
    push_slice(output, authority.admission_provenance().as_bytes());
    push_slice(output, authority.registry_snapshot().as_bytes());
}

pub(crate) fn encode_provider_capability(
    output: &mut Vec<u8>,
    provider: &ProviderIdentity,
    revision: ReferenceCapabilityRevision,
) {
    push_slice(output, provider.namespace().as_bytes());
    push_slice(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
    output.extend_from_slice(&revision.get().to_be_bytes());
}

fn encode_op_key(output: &mut Vec<u8>, key: &OpKey) {
    push_slice(output, key.namespace().as_bytes());
    push_slice(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

pub(crate) fn encode_signature(output: &mut Vec<u8>, signature: &ReferenceSignature) {
    for values in [signature.operands(), signature.results()] {
        push_len(output, values.len());
        for value in values {
            let canonical = value.canonical_encoding();
            push_slice(output, canonical.as_bytes());
        }
    }
}

pub(crate) const fn encoded_bytes_len(payload_len: usize) -> usize {
    std::mem::size_of::<u64>().saturating_add(payload_len)
}

pub(crate) fn reference_identity_base_len(semantic_registry: &FrozenSemanticRegistry) -> usize {
    b"tiler.reference-registry.v2\0"
        .len()
        .saturating_add(encoded_bytes_len(
            semantic_registry.snapshot_identity().as_bytes().len(),
        ))
        .saturating_add(2 * std::mem::size_of::<u64>())
}

fn reference_authority_identity_len(authority: &SemanticCapabilityAuthority) -> usize {
    [
        authority.reached_definitions().as_bytes().len(),
        authority.admission_provenance().as_bytes().len(),
        authority.registry_snapshot().as_bytes().len(),
    ]
    .into_iter()
    .map(encoded_bytes_len)
    .fold(0_usize, usize::saturating_add)
}

pub(crate) fn reference_provider_identity_len(provider: &ProviderIdentity) -> usize {
    encoded_bytes_len(provider.namespace().len())
        .saturating_add(encoded_bytes_len(provider.name().len()))
        .saturating_add(2 * std::mem::size_of::<u32>())
}

pub(crate) fn reference_value_identity_len(
    resolved_type: &ResolvedValueType,
    authority: &SemanticCapabilityAuthority,
    provider: &ProviderIdentity,
    _revision: ReferenceCapabilityRevision,
) -> usize {
    encoded_bytes_len(resolved_type.canonical_encoding().as_bytes().len())
        .saturating_add(reference_authority_identity_len(authority))
        .saturating_add(reference_provider_identity_len(provider))
}

pub(crate) fn reference_signature_identity_len(signature: &ReferenceSignature) -> usize {
    [signature.operands(), signature.results()]
        .into_iter()
        .map(|values| {
            values
                .iter()
                .map(|value| encoded_bytes_len(value.canonical_encoding().as_bytes().len()))
                .fold(std::mem::size_of::<u64>(), usize::saturating_add)
        })
        .fold(0_usize, usize::saturating_add)
}

pub(crate) fn reference_capability_identity_len(
    key: &ReferenceCapabilityKey,
    authority: &SemanticCapabilityAuthority,
    provider: &ProviderIdentity,
    _revision: ReferenceCapabilityRevision,
) -> usize {
    encoded_bytes_len(key.operation.namespace().len())
        .saturating_add(encoded_bytes_len(key.operation.name().len()))
        .saturating_add(std::mem::size_of::<u32>())
        .saturating_add(reference_signature_identity_len(&key.signature))
        .saturating_add(reference_authority_identity_len(authority))
        .saturating_add(reference_provider_identity_len(provider))
}

pub(crate) fn collect_signature_types(
    values: impl IntoIterator<Item = ResolvedValueType>,
    resource: ReferenceRegistryResource,
    limit: usize,
) -> Result<Vec<ResolvedValueType>, ReferenceRegistryError> {
    let mut retained = Vec::new();
    let mut retained_bytes = 0_usize;
    for value in values.into_iter().take(limit.saturating_add(1)) {
        if retained.len() == limit {
            return Err(ReferenceRegistryError::ResourceExceeded {
                resource,
                limit,
                actual: limit.saturating_add(1),
            });
        }
        retained_bytes = retained_bytes.saturating_add(value.canonical_encoding().as_bytes().len());
        if retained_bytes > MAX_REFERENCE_REGISTRY_IDENTITY_BYTES {
            return Err(ReferenceRegistryError::ResourceExceeded {
                resource: ReferenceRegistryResource::CanonicalIdentityBytes,
                limit: MAX_REFERENCE_REGISTRY_IDENTITY_BYTES,
                actual: retained_bytes,
            });
        }
        retained.push(value);
    }
    Ok(retained)
}
