//! Registration of opaque physical calls.
//!
//! A slice of `implement-opaque-physical-call-providers`.
//!
//! # Why this is not `rewrite::RuleRegistry` with a different payload
//!
//! The two registries have the same *shape* — refuse a duplicate identity,
//! iterate in canonical order — and they are not the same concept.
//! `AGENTS.md` is explicit that two types with the same shape are not the same
//! concept, and names the construction site as the evidence rather than the
//! declaration. Here the sites differ in what registration *means*: a rewrite
//! provider offers an optional transformation, and a registry entry that never
//! fires costs nothing; an opaque call provider offers the only implementation
//! of a call a program may already reference, and an entry that never fires is
//! a program that cannot be built. Sharing one type would make that difference
//! invisible, and the first person to add a "skip providers that propose
//! nothing" convenience would silently apply it to both.
//!
//! What *is* shared is the reasoning, and it is not restated here: see
//! `crate::rewrite::RuleRegistry` for why a duplicate identity is refused
//! rather than shadowed, and why iteration follows canonical identity order
//! rather than registration order.

use crate::call_declaration::OpaqueCallDeclaration;
use core::fmt;
use tiler_ir::schedule::TensorRole;

/// The governed identity of one opaque call.
///
/// Provider, call name, and an output-affecting revision — the revision changes
/// when the call's observable behaviour does, not when its implementation is
/// merely rewritten.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(
    dead_code,
    reason = "the call identity; lands with the registry that keys on it, ahead of the frontier integration"
)]
pub(crate) struct OpaqueCallIdentity {
    provider: &'static str,
    call: &'static str,
    revision: u32,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft accessors whose consumer is the not-yet-written frontier integration"
)]
impl OpaqueCallIdentity {
    /// Builds a call identity, refusing an unnamed provider or call.
    ///
    /// An empty name would make two distinct calls render identically, which is
    /// the failure a reader of explain output cannot see.
    pub(crate) const fn new(
        provider: &'static str,
        call: &'static str,
        revision: u32,
    ) -> Option<Self> {
        if provider.is_empty() || call.is_empty() {
            return None;
        }
        Some(Self {
            provider,
            call,
            revision,
        })
    }

    /// The provider that owns this call.
    pub(crate) const fn provider(&self) -> &'static str {
        self.provider
    }

    /// The call's own name, unique within its provider.
    pub(crate) const fn call(&self) -> &'static str {
        self.call
    }

    /// The output-affecting revision.
    pub(crate) const fn revision(&self) -> u32 {
        self.revision
    }
}

impl fmt::Display for OpaqueCallIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/{}@{}",
            self.provider, self.call, self.revision
        )
    }
}

/// Why a call could not be registered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "registration outcome for the seam this module provides"
)]
pub(crate) enum CallRegistrationError {
    /// Another entry already claims this identity.
    DuplicateCall(OpaqueCallIdentity),
}

impl fmt::Display for CallRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCall(identity) => write!(
                formatter,
                "call.duplicate: {identity} is already registered"
            ),
        }
    }
}

/// One registered opaque call: its identity and its checked declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the registry entry; lands ahead of the frontier integration that reads it"
)]
pub(crate) struct RegisteredCall {
    identity: OpaqueCallIdentity,
    declaration: OpaqueCallDeclaration,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft accessors whose consumer is the not-yet-written frontier integration"
)]
impl RegisteredCall {
    /// The call's governed identity.
    pub(crate) const fn identity(&self) -> OpaqueCallIdentity {
        self.identity
    }

    /// The declaration, already checked for internal coherence.
    ///
    /// Registration takes an [`OpaqueCallDeclaration`] rather than its parts,
    /// so an incoherent set cannot be registered — the coherence check is not
    /// something registration repeats or could skip.
    pub(crate) const fn declaration(&self) -> &OpaqueCallDeclaration {
        &self.declaration
    }
}

/// The opaque calls available to one compilation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the registry; lands ahead of the frontier integration that will drive it"
)]
pub(crate) struct OpaqueCallRegistry {
    calls: Vec<RegisteredCall>,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft accessors whose consumer is the not-yet-written frontier integration"
)]
impl OpaqueCallRegistry {
    /// An empty registry.
    ///
    /// Unlike an empty rewrite registry, this is not merely legitimate — it is
    /// the ordinary case. A program referencing no opaque call needs none.
    pub(crate) const fn new() -> Self {
        Self { calls: Vec::new() }
    }

    /// Registers a call, refusing an identity already claimed.
    pub(crate) fn register(
        &mut self,
        identity: OpaqueCallIdentity,
        declaration: OpaqueCallDeclaration,
    ) -> Result<(), CallRegistrationError> {
        if self.calls.iter().any(|entry| entry.identity == identity) {
            return Err(CallRegistrationError::DuplicateCall(identity));
        }
        let position = self
            .calls
            .partition_point(|entry| entry.identity < identity);
        self.calls.insert(
            position,
            RegisteredCall {
                identity,
                declaration,
            },
        );
        Ok(())
    }

    /// The registered calls, in canonical identity order.
    pub(crate) fn calls(&self) -> &[RegisteredCall] {
        &self.calls
    }

    /// The call registered under this identity.
    pub(crate) fn get(&self, identity: OpaqueCallIdentity) -> Option<&RegisteredCall> {
        self.calls.iter().find(|entry| entry.identity == identity)
    }
}

/// What a provider proposes when it offers an opaque call for a region.
///
/// The identity names *which* registered call, and the bindings say which of the
/// region's tensor roles each of the call's parameters is bound to. Both are the
/// provider's claim, and both are validated at admission — the identity against
/// the registry, the bindings against the call's own ABI.
///
/// The bindings cannot be inferred. A boundary contract is keyed by tensor role
/// and an ABI names parameters, and a parameter's `ParameterRole` says whether it
/// is read or written, never *which* tensor it reads. Inferring would reintroduce
/// exactly what `crate::call_abi`'s named parameters exist to prevent.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the opaque proposal payload; lands with the frontier check that validates it"
)]
pub(crate) struct OpaqueCallProposal {
    call: OpaqueCallIdentity,
    bindings: Vec<(&'static str, TensorRole)>,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: accessors read by the frontier admission"
)]
impl OpaqueCallProposal {
    /// Builds a proposal. Validation happens at admission, not here: a provider
    /// may construct a proposal the registry will reject, and the rejection is
    /// what the caller needs to see.
    pub(crate) const fn new(
        call: OpaqueCallIdentity,
        bindings: Vec<(&'static str, TensorRole)>,
    ) -> Self {
        Self { call, bindings }
    }

    /// The registered call this proposal names.
    pub(crate) const fn call(&self) -> OpaqueCallIdentity {
        self.call
    }

    /// Which tensor role each parameter binds.
    pub(crate) fn bindings(&self) -> &[(&'static str, TensorRole)] {
        &self.bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_declaration::WorkScaling;
    use tiler_ir::schedule::{NumericalPermission, ResourceRequirements, SubnormalMode};

    /// Resources ample enough that only the fault under test can fire.
    fn resources(bindings: u32) -> ResourceRequirements {
        ResourceRequirements {
            buffer_bindings: bindings,
            threads_per_workgroup: 1,
            local_memory_bytes: 0,
            barriers: 0,
            requires_device_memory: true,
            input_subnormals: SubnormalMode::Preserve,
            result_subnormals: SubnormalMode::Preserve,
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
        }
    }

    use crate::boundary::{
        AdmittedMemoryDomains, ByteAlignment, ExecutionAffinity, LayoutGuarantee,
        LayoutRequirement, MemoryDomainClass, StorageEncoding,
    };
    use crate::call_abi::{ParameterLayout, ParameterSpec};
    /// A spec carrying the bounded profile's storage answers.
    fn spec(name: &'static str, role: ParameterRole) -> ParameterSpec {
        ParameterSpec {
            name,
            role,
            layout: match role {
                ParameterRole::In => ParameterLayout::Required(LayoutRequirement::DenseRowMajor),
                ParameterRole::Out => ParameterLayout::Guaranteed(LayoutGuarantee::DenseRowMajor),
                ParameterRole::InOut => ParameterLayout::Both {
                    requires: LayoutRequirement::DenseRowMajor,
                    guarantees: LayoutGuarantee::DenseRowMajor,
                },
            },
            encoding: StorageEncoding::Unpacked,
            alignment: ByteAlignment::F32_NATURAL,
        }
    }

    use crate::call_abi::{CallAbi, ParameterRole};
    use crate::call_placement::CallPlacement;
    use crate::effects::{Aliasing, CallEffects, Elimination, Motion};

    fn declaration() -> OpaqueCallDeclaration {
        OpaqueCallDeclaration::check(
            CallAbi::declare(
                [("input", ParameterRole::In), ("output", ParameterRole::Out)]
                    .map(|(name, role)| spec(name, role)),
            )
            .expect("well formed"),
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct),
            CallPlacement::declare(
                ExecutionAffinity::PRIMARY,
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).expect("non-empty"),
                &[MemoryDomainClass::Device],
            )
            .expect("supported"),
            resources(8),
            WorkScaling::Fixed(1),
        )
        .expect("coherent")
    }

    fn identity(call: &'static str) -> OpaqueCallIdentity {
        OpaqueCallIdentity::new("p", call, 1).expect("named")
    }

    /// An unnamed provider or call is refused.
    #[test]
    fn an_unnamed_call_is_refused() {
        assert!(OpaqueCallIdentity::new("p", "c", 0).is_some());
        assert!(OpaqueCallIdentity::new("", "c", 0).is_none());
        assert!(OpaqueCallIdentity::new("p", "", 0).is_none());
    }

    /// A duplicate identity is refused; a distinct one is not.
    #[test]
    fn a_duplicate_call_is_refused() {
        let mut registry = OpaqueCallRegistry::new();
        assert!(registry.register(identity("a"), declaration()).is_ok());
        assert!(
            registry.register(identity("b"), declaration()).is_ok(),
            "a distinct call was refused"
        );
        assert_eq!(
            registry.register(identity("a"), declaration()),
            Err(CallRegistrationError::DuplicateCall(identity("a"))),
        );
        assert_eq!(registry.calls().len(), 2);
    }

    /// A revision change is a different call, not a duplicate.
    ///
    /// Two revisions of one call must coexist: a program pinned to the old
    /// behaviour and one built against the new both have to resolve, and
    /// treating the second registration as a duplicate would make that
    /// impossible.
    #[test]
    fn a_revision_change_registers_alongside_its_predecessor() {
        let mut registry = OpaqueCallRegistry::new();
        let old = OpaqueCallIdentity::new("p", "c", 1).expect("named");
        let new = OpaqueCallIdentity::new("p", "c", 2).expect("named");

        registry.register(old, declaration()).expect("first");
        registry
            .register(new, declaration())
            .expect("a new revision is not a duplicate");
        assert_eq!(registry.calls().len(), 2);
        assert_eq!(registry.get(old).expect("registered").identity(), old);
        assert_eq!(registry.get(new).expect("registered").identity(), new);
    }

    /// Registration order does not reach iteration order.
    #[test]
    fn iteration_is_in_canonical_identity_order() {
        let mut forward = OpaqueCallRegistry::new();
        for call in ["c", "a", "b"] {
            forward
                .register(identity(call), declaration())
                .expect("distinct");
        }
        let mut reverse = OpaqueCallRegistry::new();
        for call in ["b", "a", "c"] {
            reverse
                .register(identity(call), declaration())
                .expect("distinct");
        }

        let names = |registry: &OpaqueCallRegistry| -> Vec<&'static str> {
            registry
                .calls()
                .iter()
                .map(|entry| entry.identity().call())
                .collect()
        };
        assert_eq!(names(&forward), ["a", "b", "c"]);
        assert_eq!(
            names(&forward),
            names(&reverse),
            "two registration orders produced different iteration orders"
        );
    }
}
