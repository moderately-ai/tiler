//! Compile-checking model for Tiler's experimental operation-extension boundary.
//! This is deliberately not production code or a stable API.

use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OpKey {
    pub namespace: String,
    pub name: String,
    pub semantic_version: u32,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProviderKey {
    pub namespace: String,
    pub name: String,
    pub api_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalAttr {
    Bool(bool),
    Signed { width: u8, bits: u64 },
    Unsigned { width: u8, bits: u64 },
    FloatBits { format: String, bits: Vec<u8> },
    Utf8(String),
    Bytes(Vec<u8>),
    Sequence(Vec<CanonicalAttr>),
    Map(BTreeMap<String, CanonicalAttr>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttributeLimits {
    pub maximum_depth: usize,
    pub maximum_items: usize,
    pub maximum_bytes: usize,
}

pub fn validate_canonical_attributes(
    attributes: &BTreeMap<String, CanonicalAttr>,
    limits: AttributeLimits,
) -> Result<(), &'static str> {
    fn visit(
        value: &CanonicalAttr,
        depth: usize,
        items: &mut usize,
        bytes: &mut usize,
        limits: AttributeLimits,
    ) -> Result<(), &'static str> {
        if depth > limits.maximum_depth {
            return Err("attribute-depth-limit");
        }
        *items = items.checked_add(1).ok_or("attribute-size-overflow")?;
        if *items > limits.maximum_items {
            return Err("attribute-item-limit");
        }
        match value {
            CanonicalAttr::Utf8(value) => {
                *bytes = bytes
                    .checked_add(value.len())
                    .ok_or("attribute-size-overflow")?
            }
            CanonicalAttr::Bytes(value) | CanonicalAttr::FloatBits { bits: value, .. } => {
                *bytes = bytes
                    .checked_add(value.len())
                    .ok_or("attribute-size-overflow")?
            }
            CanonicalAttr::Sequence(values) => {
                for value in values {
                    visit(value, depth + 1, items, bytes, limits)?;
                }
            }
            CanonicalAttr::Map(values) => {
                for (key, value) in values {
                    *bytes = bytes
                        .checked_add(key.len())
                        .ok_or("attribute-size-overflow")?;
                    visit(value, depth + 1, items, bytes, limits)?;
                }
            }
            CanonicalAttr::Bool(_)
            | CanonicalAttr::Signed { .. }
            | CanonicalAttr::Unsigned { .. } => {}
        }
        if *bytes > limits.maximum_bytes {
            return Err("attribute-byte-limit");
        }
        Ok(())
    }

    let mut items: usize = 0;
    let mut bytes: usize = 0;
    for (key, value) in attributes {
        bytes = bytes
            .checked_add(key.len())
            .ok_or("attribute-size-overflow")?;
        visit(value, 1, &mut items, &mut bytes, limits)?;
    }
    if bytes > limits.maximum_bytes {
        return Err("attribute-byte-limit");
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueKind {
    Tensor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueType {
    pub kind: ValueKind,
    pub dtype: String,
    pub shape: Vec<Option<u64>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Arity {
    pub minimum: usize,
    pub maximum: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationSchema {
    pub operands: Arity,
    pub results: Arity,
    pub attribute_schema_version: u32,
    pub pure: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticContract {
    /// Stable identity of the normative specification, not a provider revision.
    pub specification: String,
    pub conformance_vectors_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceInput<'a> {
    pub operands: &'a [ValueType],
    pub attributes: &'a BTreeMap<String, CanonicalAttr>,
}

pub trait SemanticOperation: Send + Sync + 'static {
    fn key(&self) -> &OpKey;
    fn schema(&self) -> &OperationSchema;
    fn semantics(&self) -> &SemanticContract;
    fn infer_and_validate(&self, input: &InferenceInput<'_>) -> Result<Vec<ValueType>, Diagnostic>;
}

pub trait ReferenceEvaluator: Send + Sync + 'static {
    fn evaluate(
        &self,
        inputs: &[Vec<u8>],
        attributes: &BTreeMap<String, CanonicalAttr>,
    ) -> Result<Vec<Vec<u8>>, Diagnostic>;
}

pub trait DecompositionProvider: Send + Sync + 'static {
    fn decompose(&self, operation: &VerifiedOperation) -> Result<ProposedGraph, Diagnostic>;
}

pub trait AccessLoweringProvider: Send + Sync + 'static {
    fn lower_access(
        &self,
        operation: &VerifiedOperation,
    ) -> Result<ProposedAccessModel, Diagnostic>;
}

#[derive(Clone)]
pub struct SemanticRegistration {
    pub provider: ProviderKey,
    pub revision: String,
    pub operation: Arc<dyn SemanticOperation>,
    pub reference: Option<Arc<dyn ReferenceEvaluator>>,
}

#[derive(Clone)]
pub enum OptionalCapability {
    Decomposition(Arc<dyn DecompositionProvider>),
    AccessLowering(Arc<dyn AccessLoweringProvider>),
}

impl fmt::Debug for OptionalCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Decomposition(_) => "Decomposition(..)",
            Self::AccessLowering(_) => "AccessLowering(..)",
        })
    }
}

impl OptionalCapability {
    fn kind(&self) -> CapabilityKind {
        match self {
            Self::Decomposition(_) => CapabilityKind::Decomposition,
            Self::AccessLowering(_) => CapabilityKind::AccessLowering,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CapabilityKind {
    Decomposition,
    AccessLowering,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapabilitySlot {
    pub operation: OpKey,
    pub kind: CapabilityKind,
    pub provider: ProviderKey,
}

#[derive(Clone)]
pub struct CapabilityRegistration {
    pub slot: CapabilitySlot,
    pub revision: String,
    pub implementation: OptionalCapability,
}

#[derive(Default)]
pub struct RegistryBuilder {
    semantics: BTreeMap<OpKey, SemanticRegistration>,
    capabilities: BTreeMap<CapabilitySlot, CapabilityRegistration>,
}

impl RegistryBuilder {
    pub fn register_semantics(
        &mut self,
        registration: SemanticRegistration,
    ) -> Result<(), Diagnostic> {
        let key = registration.operation.key().clone();
        match self.semantics.entry(key.clone()) {
            Entry::Vacant(slot) => {
                slot.insert(registration);
            }
            Entry::Occupied(_) => {
                return Err(Diagnostic::new("duplicate-semantic-authority", key));
            }
        }
        Ok(())
    }

    pub fn register_capability(
        &mut self,
        registration: CapabilityRegistration,
    ) -> Result<(), Diagnostic> {
        let slot = registration.slot.clone();
        match self.capabilities.entry(slot.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(registration);
            }
            Entry::Occupied(_) => {
                return Err(Diagnostic::new(
                    "duplicate-capability-provider",
                    slot.operation,
                ));
            }
        }
        Ok(())
    }

    pub fn freeze(self) -> Result<RegistrySnapshot, Diagnostic> {
        for (key, registration) in &self.semantics {
            if registration.operation.key() != key {
                return Err(Diagnostic::new(
                    "semantic-key-provider-mismatch",
                    key.clone(),
                ));
            }
            validate_revision(&registration.revision, key)?;
        }
        for (slot, registration) in &self.capabilities {
            validate_revision(&registration.revision, &slot.operation)?;
            if !self.semantics.contains_key(&slot.operation) {
                return Err(Diagnostic::new(
                    "capability-without-semantics",
                    slot.operation.clone(),
                ));
            }
            if registration.implementation.kind() != slot.kind {
                return Err(Diagnostic::new(
                    "capability-kind-provider-mismatch",
                    slot.operation.clone(),
                ));
            }
        }
        Ok(RegistrySnapshot {
            semantics: self.semantics,
            capabilities: self.capabilities,
        })
    }
}

pub struct RegistrySnapshot {
    semantics: BTreeMap<OpKey, SemanticRegistration>,
    capabilities: BTreeMap<CapabilitySlot, CapabilityRegistration>,
}

impl RegistrySnapshot {
    pub fn semantic(&self, key: &OpKey) -> Option<&SemanticRegistration> {
        self.semantics.get(key)
    }

    /// Full request provenance. Artifact identity should project only providers
    /// reached or selected by the compilation, not every unused registration.
    pub fn request_provenance(&self) -> Vec<String> {
        let semantic = self.semantics.iter().map(|(key, value)| {
            format!(
                "semantic:{}::{}@{}:{}::{}@{}={}",
                key.namespace,
                key.name,
                key.semantic_version,
                value.provider.namespace,
                value.provider.name,
                value.provider.api_version,
                value.revision
            )
        });
        let capabilities = self.capabilities.iter().map(|(slot, value)| {
            format!(
                "capability:{:?}:{}::{}@{}={}",
                slot.kind,
                slot.provider.namespace,
                slot.provider.name,
                slot.provider.api_version,
                value.revision
            )
        });
        semantic.chain(capabilities).collect()
    }

    pub fn selected_provenance(
        &self,
        reachable_operations: &BTreeSet<OpKey>,
        selected_capabilities: &BTreeSet<CapabilitySlot>,
    ) -> Vec<String> {
        let semantic = reachable_operations.iter().filter_map(|key| {
            self.semantics.get(key).map(|value| {
                format!(
                    "semantic:{}::{}@{}:{}::{}@{}={}",
                    key.namespace,
                    key.name,
                    key.semantic_version,
                    value.provider.namespace,
                    value.provider.name,
                    value.provider.api_version,
                    value.revision
                )
            })
        });
        let capabilities = selected_capabilities.iter().filter_map(|slot| {
            self.capabilities.get(slot).map(|value| {
                format!(
                    "capability:{:?}:{}::{}@{}={}",
                    slot.kind,
                    slot.provider.namespace,
                    slot.provider.name,
                    slot.provider.api_version,
                    value.revision
                )
            })
        });
        semantic.chain(capabilities).collect()
    }
}

pub fn invoke_inference(
    operation: &dyn SemanticOperation,
    input: &InferenceInput<'_>,
) -> Result<Vec<ValueType>, Diagnostic> {
    catch_unwind(AssertUnwindSafe(|| operation.infer_and_validate(input)))
        .map_err(|_| Diagnostic::new("provider-panicked", operation.key().clone()))?
}

pub fn check_inference_determinism(
    operation: &dyn SemanticOperation,
    input: &InferenceInput<'_>,
) -> Result<Vec<ValueType>, Diagnostic> {
    let first = invoke_inference(operation, input)?;
    let second = invoke_inference(operation, input)?;
    if first != second {
        return Err(Diagnostic::new(
            "nondeterministic-provider",
            operation.key().clone(),
        ));
    }
    Ok(first)
}

fn validate_revision(revision: &str, operation: &OpKey) -> Result<(), Diagnostic> {
    if revision.is_empty() || revision.len() > 128 || !revision.is_ascii() {
        return Err(Diagnostic::new(
            "invalid-provider-revision",
            operation.clone(),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub operation: OpKey,
}

impl Diagnostic {
    fn new(code: &'static str, operation: OpKey) -> Self {
        Self { code, operation }
    }
}

#[derive(Clone, Debug)]
pub struct VerifiedOperation;

#[derive(Clone, Debug)]
pub struct ProposedGraph;

#[derive(Clone, Debug)]
pub struct ProposedAccessModel;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct TestOp {
        key: OpKey,
        schema: OperationSchema,
        semantics: SemanticContract,
        flip: AtomicBool,
        panic: bool,
    }

    impl TestOp {
        fn new(name: &str) -> Self {
            Self {
                key: OpKey {
                    namespace: "test".into(),
                    name: name.into(),
                    semantic_version: 1,
                },
                schema: OperationSchema {
                    operands: Arity {
                        minimum: 1,
                        maximum: Some(1),
                    },
                    results: Arity {
                        minimum: 1,
                        maximum: Some(1),
                    },
                    attribute_schema_version: 1,
                    pure: true,
                },
                semantics: SemanticContract {
                    specification: format!("test://{name}/1"),
                    conformance_vectors_digest: "sha256:vectors".into(),
                },
                flip: AtomicBool::new(false),
                panic: false,
            }
        }
    }

    impl SemanticOperation for TestOp {
        fn key(&self) -> &OpKey {
            &self.key
        }
        fn schema(&self) -> &OperationSchema {
            &self.schema
        }
        fn semantics(&self) -> &SemanticContract {
            &self.semantics
        }
        fn infer_and_validate(
            &self,
            input: &InferenceInput<'_>,
        ) -> Result<Vec<ValueType>, Diagnostic> {
            assert!(!self.panic, "intentional provider panic");
            let mut result = input.operands.to_vec();
            if self
                .flip
                .swap(!self.flip.load(Ordering::Relaxed), Ordering::Relaxed)
            {
                result.clear();
            }
            Ok(result)
        }
    }

    fn registration(name: &str, revision: &str) -> SemanticRegistration {
        SemanticRegistration {
            provider: ProviderKey {
                namespace: "test".into(),
                name: format!("{name}-semantics"),
                api_version: 1,
            },
            revision: revision.into(),
            operation: Arc::new(TestOp::new(name)),
            reference: None,
        }
    }

    struct TestDecomposition;

    impl DecompositionProvider for TestDecomposition {
        fn decompose(&self, _operation: &VerifiedOperation) -> Result<ProposedGraph, Diagnostic> {
            Ok(ProposedGraph)
        }
    }

    #[test]
    fn duplicate_semantic_authority_is_rejected() {
        let mut builder = RegistryBuilder::default();
        builder
            .register_semantics(registration("add", "r1"))
            .unwrap();
        let error = builder
            .register_semantics(registration("add", "r2"))
            .unwrap_err();
        assert_eq!(error.code, "duplicate-semantic-authority");
        let snapshot = builder.freeze().unwrap();
        assert!(snapshot.request_provenance()[0].ends_with("=r1"));
    }

    #[test]
    fn oversized_attributes_are_rejected_before_provider_calls() {
        let attributes = BTreeMap::from([("payload".into(), CanonicalAttr::Bytes(vec![0; 9]))]);
        let error = validate_canonical_attributes(
            &attributes,
            AttributeLimits {
                maximum_depth: 4,
                maximum_items: 8,
                maximum_bytes: 8,
            },
        )
        .unwrap_err();
        assert_eq!(error, "attribute-byte-limit");
    }

    #[test]
    fn snapshot_order_does_not_depend_on_registration_order() {
        let mut left = RegistryBuilder::default();
        left.register_semantics(registration("z", "r1")).unwrap();
        left.register_semantics(registration("a", "r1")).unwrap();
        let mut right = RegistryBuilder::default();
        right.register_semantics(registration("a", "r1")).unwrap();
        right.register_semantics(registration("z", "r1")).unwrap();
        assert_eq!(
            left.freeze().unwrap().request_provenance(),
            right.freeze().unwrap().request_provenance()
        );
    }

    #[test]
    fn unused_registrations_do_not_poison_selected_identity() {
        let mut builder = RegistryBuilder::default();
        builder
            .register_semantics(registration("used", "r1"))
            .unwrap();
        builder
            .register_semantics(registration("unused", "r9"))
            .unwrap();
        let snapshot = builder.freeze().unwrap();
        let used = OpKey {
            namespace: "test".into(),
            name: "used".into(),
            semantic_version: 1,
        };
        let selected = snapshot.selected_provenance(&BTreeSet::from([used]), &BTreeSet::new());
        assert_eq!(selected.len(), 1);
        assert!(!selected[0].contains("unused"));
    }

    #[test]
    fn invalid_provider_revision_is_rejected_at_freeze() {
        let mut builder = RegistryBuilder::default();
        builder.register_semantics(registration("add", "")).unwrap();
        let error = match builder.freeze() {
            Ok(_) => panic!("invalid revision unexpectedly froze"),
            Err(error) => error,
        };
        assert_eq!(error.code, "invalid-provider-revision");
    }

    #[test]
    fn capability_implementation_must_match_its_typed_slot() {
        let mut builder = RegistryBuilder::default();
        builder
            .register_semantics(registration("add", "r1"))
            .unwrap();
        let operation = OpKey {
            namespace: "test".into(),
            name: "add".into(),
            semantic_version: 1,
        };
        builder
            .register_capability(CapabilityRegistration {
                slot: CapabilitySlot {
                    operation,
                    kind: CapabilityKind::AccessLowering,
                    provider: ProviderKey {
                        namespace: "test".into(),
                        name: "wrong-kind".into(),
                        api_version: 1,
                    },
                },
                revision: "r1".into(),
                implementation: OptionalCapability::Decomposition(Arc::new(TestDecomposition)),
            })
            .unwrap();
        let error = match builder.freeze() {
            Ok(_) => panic!("mismatched capability unexpectedly froze"),
            Err(error) => error,
        };
        assert_eq!(error.code, "capability-kind-provider-mismatch");
    }

    #[test]
    fn provider_panic_is_attributed() {
        let mut operation = TestOp::new("panic");
        operation.panic = true;
        let error = invoke_inference(
            &operation,
            &InferenceInput {
                operands: &[],
                attributes: &BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "provider-panicked");
    }

    #[test]
    fn nondeterministic_inference_is_detected() {
        let operation = TestOp::new("flip");
        let operand = ValueType {
            kind: ValueKind::Tensor,
            dtype: "f32".into(),
            shape: vec![Some(4)],
        };
        let error = check_inference_determinism(
            &operation,
            &InferenceInput {
                operands: &[operand],
                attributes: &BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "nondeterministic-provider");
    }
}
