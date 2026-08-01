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

/// Maximum bytes in the exact textual subject of one opaque call.
///
/// Explain subject keys are bounded to 255 bytes. The longest decimal `u32`
/// revision is ten bytes, and the two delimiters occupy two more, so
/// construction reserves those bytes and rejects an identity whose exact
/// `provider/call@revision` spelling could not be carried without hashing or
/// truncation.
pub(crate) const MAX_OPAQUE_CALL_SUBJECT_BYTES: usize = 255;

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
    /// Builds a call identity, refusing a component outside the governed
    /// delimiter-safe ASCII grammar or an identity too long for explain.
    ///
    /// An empty name would make two distinct calls render identically, which is
    /// the failure a reader of explain output cannot see.
    pub(crate) const fn new(
        provider: &'static str,
        call: &'static str,
        revision: u32,
    ) -> Option<Self> {
        if !valid_identity_component(provider) || !valid_identity_component(call) {
            return None;
        }
        if call_subject_len(provider, call, revision) > MAX_OPAQUE_CALL_SUBJECT_BYTES {
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

    /// The exact delimiter-safe explain subject for this call.
    ///
    /// Construction proves this is at most
    /// [`MAX_OPAQUE_CALL_SUBJECT_BYTES`]. No digest or truncation is needed, so
    /// a reader can recover the complete governed identity from the subject.
    pub(crate) fn subject(&self) -> String {
        format!("{self}")
    }
}

const fn valid_identity_component(component: &str) -> bool {
    if component.is_empty() {
        return false;
    }
    let bytes = component.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if !matches!(
            byte,
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'-'
        ) {
            return false;
        }
        index += 1;
    }
    true
}

const fn decimal_digits(value: u32) -> usize {
    if value < 10 {
        1
    } else if value < 100 {
        2
    } else if value < 1_000 {
        3
    } else if value < 10_000 {
        4
    } else if value < 100_000 {
        5
    } else if value < 1_000_000 {
        6
    } else if value < 10_000_000 {
        7
    } else if value < 100_000_000 {
        8
    } else if value < 1_000_000_000 {
        9
    } else {
        10
    }
}

const fn call_subject_len(provider: &str, call: &str, revision: u32) -> usize {
    provider.len() + call.len() + decimal_digits(revision) + 2
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

/// Why an opaque-call proposal could not be represented exactly in explain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OpaqueCallProposalError {
    /// A binding name is outside the delimiter-safe governed component grammar.
    InvalidBindingName {
        /// The zero-based binding position.
        index: usize,
        /// The refused name.
        name: &'static str,
    },
    /// The exact call-and-ordered-bindings subject exceeds the explain bound.
    SubjectTooLong {
        /// Bytes the exact subject requires.
        actual: usize,
        /// Bytes the explain vocabulary admits.
        maximum: usize,
    },
}

#[allow(
    dead_code,
    reason = "see the type's own allow: accessors read by the frontier admission"
)]
impl OpaqueCallProposal {
    /// Builds a proposal whose complete ordered claim is exactly reportable.
    ///
    /// Semantic validation still happens at admission: an unknown, repeated, or
    /// absent ABI parameter remains a typed frontier refusal. Construction only
    /// rejects lexical ambiguity and a proposal whose exact explain subject
    /// would require truncation or hashing.
    pub(crate) fn new(
        call: OpaqueCallIdentity,
        bindings: Vec<(&'static str, TensorRole)>,
    ) -> Result<Self, OpaqueCallProposalError> {
        let actual = proposal_subject_len(call, &bindings);
        if actual > MAX_OPAQUE_CALL_SUBJECT_BYTES {
            return Err(OpaqueCallProposalError::SubjectTooLong {
                actual,
                maximum: MAX_OPAQUE_CALL_SUBJECT_BYTES,
            });
        }
        for (index, (name, _)) in bindings.iter().enumerate() {
            if !crate::call_abi::valid_parameter_name(name) {
                return Err(OpaqueCallProposalError::InvalidBindingName { index, name });
            }
        }
        Ok(Self { call, bindings })
    }

    /// The registered call this proposal names.
    pub(crate) const fn call(&self) -> OpaqueCallIdentity {
        self.call
    }

    /// Which tensor role each parameter binds.
    pub(crate) fn bindings(&self) -> &[(&'static str, TensorRole)] {
        &self.bindings
    }

    /// Exact delimiter-safe subject covering the call and ordered bindings.
    ///
    /// Construction proves this fits [`MAX_OPAQUE_CALL_SUBJECT_BYTES`]. Binding
    /// order is rendered rather than canonicalized away because it is part of
    /// the provider's proposal identity.
    pub(crate) fn subject(&self) -> String {
        let mut subject = self.call.subject();
        subject.push('[');
        for (index, (name, role)) in self.bindings.iter().enumerate() {
            if index != 0 {
                subject.push(',');
            }
            subject.push_str(name);
            subject.push('=');
            push_tensor_role_name(&mut subject, *role);
        }
        subject.push(']');
        debug_assert_eq!(
            subject.len(),
            proposal_subject_len(self.call, &self.bindings)
        );
        subject
    }
}

fn proposal_subject_len(call: OpaqueCallIdentity, bindings: &[(&str, TensorRole)]) -> usize {
    call_subject_len(call.provider, call.call, call.revision)
        + 2
        + bindings
            .iter()
            .enumerate()
            .map(|(index, (name, role))| {
                name.len() + 1 + tensor_role_name_len(*role) + usize::from(index != 0)
            })
            .sum::<usize>()
}

/// Renders one bound tensor role into an exact opaque-call subject.
///
/// An input renders its ordinal too. Two parameters bound to two different
/// input tensors are two different proposals, and a subject spelling both
/// `input` would give one name to two things — which is exactly what a subject
/// exists to prevent.
fn push_tensor_role_name(subject: &mut String, role: TensorRole) {
    match role {
        TensorRole::Input { ordinal } => {
            subject.push_str(INPUT_ROLE_PREFIX);
            subject.push_str(&ordinal.get().to_string());
        }
        TensorRole::Intermediate => subject.push_str("intermediate"),
        TensorRole::Output => subject.push_str("output"),
    }
}

/// Mirrors [`push_tensor_role_name`] so the precomputed bound stays exact.
///
/// Computed rather than formatted: this runs once per binding inside the
/// admission check that `subject` later debug-asserts against, so it must agree
/// with the renderer without allocating a string to find out.
const fn tensor_role_name_len(role: TensorRole) -> usize {
    match role {
        TensorRole::Input { ordinal } => {
            let digits = match ordinal.get().checked_ilog10() {
                Some(log) => log as usize + 1,
                None => 1,
            };
            INPUT_ROLE_PREFIX.len() + digits
        }
        TensorRole::Intermediate => "intermediate".len(),
        TensorRole::Output => "output".len(),
    }
}

/// Opens the rendering of an input role, ahead of its decimal ordinal.
const INPUT_ROLE_PREFIX: &str = "input#";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_declaration::WorkScaling;
    use tiler_ir::schedule::InputOrdinal;
    use tiler_ir::schedule::{
        ExceptionalValueAssumption, NumericalPermission, ResourceRequirements, SubnormalMode,
    };

    /// Resources ample enough that only the fault under test can fire.
    fn resources(bindings: u32) -> ResourceRequirements {
        ResourceRequirements {
            buffer_bindings: bindings,
            threads_per_workgroup: 1,
            local_memory_bytes: 0,
            requires_device_memory: true,
            synchronization: None,
            input_subnormals: SubnormalMode::Preserve,
            result_subnormals: SubnormalMode::Preserve,
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
            permutation: NumericalPermission::Forbidden,
            signed_zero: NumericalPermission::Forbidden,
            nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
        }
    }

    use crate::boundary::{
        AdmittedMemoryDomains, ByteAlignment, ExecutionAffinity, LayoutGuarantee,
        LayoutRequirement, MemoryDomainClass, StorageEncoding, StorageScalar,
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
            alignment: ByteAlignment::natural_for(StorageScalar::F32),
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
        assert!(OpaqueCallIdentity::new("p/c", "c", 0).is_none());
        assert!(OpaqueCallIdentity::new("p", "c@1", 0).is_none());
        assert_eq!(
            OpaqueCallIdentity::new("provider", "call", u32::MAX)
                .expect("bounded")
                .subject(),
            "provider/call@4294967295"
        );
        let too_long = "x".repeat(MAX_OPAQUE_CALL_SUBJECT_BYTES);
        let leaked = Box::leak(too_long.into_boxed_str());
        assert!(OpaqueCallIdentity::new(leaked, "c", 0).is_none());
    }

    #[test]
    fn proposal_subject_is_exact_ordered_and_bounded() {
        let call = identity("c");
        let proposal = OpaqueCallProposal::new(
            call,
            vec![
                (
                    "x",
                    TensorRole::Input {
                        ordinal: InputOrdinal::FIRST,
                    },
                ),
                ("y", TensorRole::Output),
            ],
        )
        .expect("bounded");
        assert_eq!(proposal.subject(), "p/c@1[x=input#0,y=output]");
        assert_eq!(
            OpaqueCallProposal::new(
                call,
                vec![(
                    "x/y",
                    TensorRole::Input {
                        ordinal: InputOrdinal::FIRST
                    }
                )]
            ),
            Err(OpaqueCallProposalError::InvalidBindingName {
                index: 0,
                name: "x/y",
            })
        );

        let long = Box::leak("x".repeat(250).into_boxed_str());
        let actual = "p/c@1[]".len() + long.len() + "=output".len();
        assert_eq!(
            OpaqueCallProposal::new(call, vec![(long, TensorRole::Output)]),
            Err(OpaqueCallProposalError::SubjectTooLong {
                actual,
                maximum: MAX_OPAQUE_CALL_SUBJECT_BYTES,
            })
        );
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
