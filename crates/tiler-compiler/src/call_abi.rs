//! The typed ABI an opaque call declares: what it binds, named rather than positional.
//!
//! A slice of `implement-opaque-physical-call-providers`.
//!
//! # Why parameters are named and not merely ordered
//!
//! This is settled by a mistake already recorded in `AGENTS.md`, in the section
//! on eliminating options that do not survive: an artifact "binding buffers by
//! slot position could not verify the position meant what it assumed", and the
//! consequence named there is a silently wrong result rather than a trade-off.
//!
//! A positional ABI is checkable only for *arity*. Two calls that both take
//! three buffers agree positionally whatever those buffers are for, so swapping
//! an input for an output passes every check a position can support and fails at
//! runtime, or worse, does not fail. A name plus a typed role makes the
//! mismatch a rejection at declaration time.
//!
//! Positions still exist — a binding table is ordered, and
//! [`CallParameter::slot`] is that order — but the slot is *derived from* the
//! declaration rather than being the declaration. Nothing here matches two
//! parameters by comparing slots.

use crate::boundary::{ByteAlignment, LayoutGuarantee, LayoutRequirement, StorageEncoding};
use core::fmt;
use tiler_ir::schedule::{AccessMode, AccessOrdinal};

/// Maximum bytes in one exactly reportable ABI parameter name.
pub(crate) const MAX_PARAMETER_NAME_BYTES: usize = 255;

/// Whether a parameter name is an unambiguous governed identity component.
pub(crate) const fn valid_parameter_name(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_PARAMETER_NAME_BYTES {
        return false;
    }
    let bytes = name.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !matches!(
            bytes[index],
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'-'
        ) {
            return false;
        }
        index += 1;
    }
    true
}

/// What a parameter is for.
///
/// Closed, and the distinctions are the ones a binding must not blur. An
/// `In` and an `Out` of identical shape and dtype are not interchangeable, and
/// a positional ABI cannot say so.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(
    dead_code,
    reason = "no production opaque-call provider declares an ABI; frontier admission consumes these roles from test providers until caller-supplied physical providers reach the compile path"
)]
pub(crate) enum ParameterRole {
    /// Read by the call, never written.
    In,
    /// Written by the call, never read.
    Out,
    /// Both read and written.
    ///
    /// Distinct from declaring an `In` and an `Out` over the same storage: that
    /// pair says two things may alias, while this says one thing is updated in
    /// place, and only the second tells a caller its prior contents are gone.
    InOut,
}

impl ParameterRole {
    /// The governed canonical key naming this role.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
            Self::InOut => "inout",
        }
    }

    /// Whether the call reads this parameter's prior contents.
    pub(crate) const fn reads(self) -> bool {
        match self {
            Self::In | Self::InOut => true,
            Self::Out => false,
        }
    }

    /// Whether the call writes this parameter.
    pub(crate) const fn writes(self) -> bool {
        match self {
            Self::Out | Self::InOut => true,
            Self::In => false,
        }
    }
}

impl fmt::Display for ParameterRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

/// The layout a parameter states, typed by the direction it states it in.
///
/// `crate::boundary` types the two directions differently and the asymmetry is
/// load-bearing: `LayoutGuarantee` has one variant, the only layout the bounded
/// profile produces, while `LayoutRequirement` adds `UnitStrideOnAxis` because a
/// consumer can ask for something a producer does not volunteer.
///
/// So an `In` parameter *requires* a layout and an `Out` parameter *guarantees*
/// one, and a single type for both would silently forbid one direction. A sum
/// rather than a pair of `Option`s because a pair admits four combinations of
/// which three are malformed, leaving the constructor to re-check what the type
/// should have made unrepresentable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "no production opaque-call provider constructs a parameter layout; frontier admission consumes it from test providers until caller-supplied physical providers reach the compile path"
)]
pub(crate) enum ParameterLayout {
    /// The call requires this of the tensor bound to the parameter.
    Required(LayoutRequirement),
    /// The call guarantees this of what it writes.
    Guaranteed(LayoutGuarantee),
    /// An in-place parameter, which does both.
    Both {
        /// What the call requires of the incoming value.
        requires: LayoutRequirement,
        /// What it guarantees of the value it leaves behind.
        guarantees: LayoutGuarantee,
    },
}

impl ParameterLayout {
    /// Whether this layout states the directions `role` needs.
    ///
    /// An exhaustive match rather than a pair of boolean tests, so a fourth
    /// role or a fourth layout shape is a build error here instead of silently
    /// admitting a combination nobody considered.
    const fn matches(self, role: ParameterRole) -> bool {
        match (self, role) {
            (Self::Required(_), ParameterRole::In)
            | (Self::Guaranteed(_), ParameterRole::Out)
            | (Self::Both { .. }, ParameterRole::InOut) => true,
            (Self::Required(_), ParameterRole::Out | ParameterRole::InOut)
            | (Self::Guaranteed(_), ParameterRole::In | ParameterRole::InOut)
            | (Self::Both { .. }, ParameterRole::In | ParameterRole::Out) => false,
        }
    }
}

/// What a provider states about one parameter.
///
/// Layout, encoding, and alignment are here rather than on the declaration as a
/// whole because they are properties of *that binding*: a two-parameter call may
/// want a dense row-major input and a differently-laid-out output, and a
/// declaration-level answer could not say so.
///
/// None of the three has a default. A boundary contract must state all three,
/// and a guess would be a claim the provider never made — the same reasoning
/// that keeps `CallEffects` from having a `Default`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ParameterSpec {
    /// The parameter's name, unique within its ABI.
    pub(crate) name: &'static str,
    /// What the parameter is for.
    pub(crate) role: ParameterRole,
    /// The storage layout this binding states, in the direction its role needs.
    pub(crate) layout: ParameterLayout,
    /// How one element is represented at the position layout assigns it.
    pub(crate) encoding: StorageEncoding,
    /// Byte alignment of this binding's first element.
    pub(crate) alignment: ByteAlignment,
}

/// One declared parameter of an opaque call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallParameter {
    spec: ParameterSpec,
    slot: u32,
}

#[allow(
    dead_code,
    reason = "the derived slot is a test-facing declaration-order witness; production frontier admission matches parameters by name and role rather than reading slots"
)]
impl CallParameter {
    /// The parameter's declared name, unique within its ABI.
    pub(crate) const fn name(&self) -> &'static str {
        self.spec.name
    }

    /// What the parameter is for.
    pub(crate) const fn role(&self) -> ParameterRole {
        self.spec.role
    }

    /// Everything the provider stated about this binding.
    pub(crate) const fn spec(&self) -> &ParameterSpec {
        &self.spec
    }

    /// The binding-table position this parameter occupies.
    ///
    /// Derived from declaration order, not supplied by the provider. A provider
    /// that could choose its own slots could produce two parameters claiming
    /// one, and the check for that would be a second authority over something
    /// the ABI already knows.
    pub(crate) const fn slot(&self) -> u32 {
        self.slot
    }
}

impl fmt::Display for CallParameter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}@{}",
            self.spec.name, self.spec.role, self.slot
        )
    }
}

/// Why an ABI declaration was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "only provider-side ABI construction returns this error, and production installs no opaque-call provider until caller-supplied physical providers reach the compile path"
)]
pub(crate) enum AbiError {
    /// Two parameters share a name.
    DuplicateName(&'static str),
    /// A parameter has no name.
    ///
    /// An unnamed parameter can only be matched positionally, which is the
    /// whole thing this ABI exists to avoid.
    UnnamedParameter(u32),
    /// A parameter name is not a delimiter-safe governed identity component.
    InvalidParameterName {
        /// The parameter's derived slot.
        slot: u32,
        /// The refused name.
        name: &'static str,
    },
    /// A parameter name cannot fit exactly in an explain identity value.
    ParameterNameTooLong {
        /// The parameter's derived slot.
        slot: u32,
        /// Bytes the exact name requires.
        actual: usize,
        /// Bytes the explain vocabulary admits.
        maximum: usize,
    },
    /// A parameter's layout states the wrong direction for its role.
    ///
    /// An `In` parameter that guarantees a layout, or an `Out` that requires
    /// one, has said something about a direction it does not have.
    LayoutDirectionMismatch(&'static str),
    /// The call declares no parameter it writes.
    ///
    /// A call that writes nothing produces nothing observable, so admitting one
    /// would mean carrying a call whose elimination is never wrong — and if
    /// that is genuinely intended, the effect declaration is where it is said,
    /// not by omitting every output.
    NoWrittenParameter,
}

impl fmt::Display for AbiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateName(name) => {
                write!(formatter, "abi.duplicate-name: {name} is declared twice")
            }
            Self::UnnamedParameter(slot) => {
                write!(formatter, "abi.unnamed-parameter: slot {slot} has no name")
            }
            Self::InvalidParameterName { slot, name } => write!(
                formatter,
                "abi.invalid-parameter-name: slot {slot} name {name:?} is not governed"
            ),
            Self::ParameterNameTooLong {
                slot,
                actual,
                maximum,
            } => write!(
                formatter,
                "abi.parameter-name-too-long: slot {slot} requires {actual} bytes, maximum {maximum}"
            ),
            Self::LayoutDirectionMismatch(name) => write!(
                formatter,
                "abi.layout-direction-mismatch: {name} states a layout its role does not have"
            ),
            Self::NoWrittenParameter => {
                formatter.write_str("abi.no-written-parameter: the call writes nothing")
            }
        }
    }
}

/// The complete declared ABI of one opaque call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallAbi {
    parameters: Vec<CallParameter>,
}

#[allow(
    dead_code,
    reason = "ABI declaration and compatibility are exercised by tests; production frontier admission consumes an already-checked ABI until caller-supplied physical providers construct and compare one"
)]
impl CallAbi {
    /// Declares an ABI from `(name, role)` pairs, in binding-table order.
    ///
    /// Slots are assigned from the iteration order rather than taken from the
    /// caller; see [`CallParameter::slot`].
    pub(crate) fn declare(
        parameters: impl IntoIterator<Item = ParameterSpec>,
    ) -> Result<Self, AbiError> {
        let mut declared: Vec<CallParameter> = Vec::new();
        for (slot, spec) in parameters.into_iter().enumerate() {
            let slot = u32::try_from(slot).unwrap_or(u32::MAX);
            if spec.name.is_empty() {
                return Err(AbiError::UnnamedParameter(slot));
            }
            if spec.name.len() > MAX_PARAMETER_NAME_BYTES {
                return Err(AbiError::ParameterNameTooLong {
                    slot,
                    actual: spec.name.len(),
                    maximum: MAX_PARAMETER_NAME_BYTES,
                });
            }
            if !valid_parameter_name(spec.name) {
                return Err(AbiError::InvalidParameterName {
                    slot,
                    name: spec.name,
                });
            }
            if declared
                .iter()
                .any(|existing| existing.spec.name == spec.name)
            {
                return Err(AbiError::DuplicateName(spec.name));
            }
            if !spec.layout.matches(spec.role) {
                return Err(AbiError::LayoutDirectionMismatch(spec.name));
            }
            declared.push(CallParameter { spec, slot });
        }
        if !declared
            .iter()
            .any(|parameter| parameter.spec.role.writes())
        {
            return Err(AbiError::NoWrittenParameter);
        }
        Ok(Self {
            parameters: declared,
        })
    }

    /// The parameters, in binding-table order.
    pub(crate) fn parameters(&self) -> &[CallParameter] {
        &self.parameters
    }

    /// The parameter with this name.
    ///
    /// **The only supported way to find a parameter.** There is deliberately no
    /// lookup by slot: a caller that had a slot and wanted a parameter would be
    /// reintroducing exactly the positional matching this ABI exists to
    /// prevent.
    pub(crate) fn parameter(&self, name: &str) -> Option<&CallParameter> {
        self.parameters
            .iter()
            .find(|parameter| parameter.spec.name == name)
    }

    /// Whether this ABI can be bound where `other` is expected.
    ///
    /// Matching is by **name and role together**. Same names in a different
    /// declaration order compose; same order with different names does not.
    /// That asymmetry is the point: a binding table is ordered, but the order
    /// is a consequence of the declaration rather than the identity of it.
    pub(crate) fn is_compatible_with(&self, other: &Self) -> bool {
        self.parameters.len() == other.parameters.len()
            && self.parameters.iter().all(|mine| {
                other
                    .parameter(mine.spec.name)
                    .is_some_and(|theirs| theirs.spec == mine.spec)
            })
    }
}

/// Why a parameter-to-tensor binding was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BindingError {
    /// A parameter the ABI declares is not bound.
    UnboundParameter(&'static str),
    /// A binding names a parameter the ABI does not declare.
    UnknownParameter(&'static str),
    /// A parameter is bound more than once.
    ParameterBoundTwice(&'static str),
    AccessOutOfRange {
        parameter: &'static str,
        access: AccessOrdinal,
    },
    InOutRegionUnsupported {
        parameter: &'static str,
        access: AccessOrdinal,
    },
    AccessModeMismatch {
        parameter: &'static str,
        access: AccessOrdinal,
        parameter_role: ParameterRole,
        access_mode: AccessMode,
    },
    UnboundAccess(AccessOrdinal),
    AccessStorageDisagreement {
        access: AccessOrdinal,
        first: &'static str,
        second: &'static str,
    },
}

impl fmt::Display for BindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnboundParameter(name) => {
                write!(formatter, "binding.unbound-parameter: {name} is not bound")
            }
            Self::UnknownParameter(name) => {
                write!(
                    formatter,
                    "binding.unknown-parameter: {name} is not declared"
                )
            }
            Self::ParameterBoundTwice(name) => {
                write!(
                    formatter,
                    "binding.bound-twice: {name} is bound more than once"
                )
            }
            Self::AccessOutOfRange { parameter, access } => write!(
                formatter,
                "binding.access-out-of-range: {parameter} names access {} outside the region",
                access.get()
            ),
            Self::InOutRegionUnsupported { parameter, access } => write!(
                formatter,
                "binding.inout-region-unsupported: {parameter} names regional access {}",
                access.get()
            ),
            Self::AccessModeMismatch {
                parameter,
                access,
                parameter_role,
                access_mode,
            } => write!(
                formatter,
                "binding.access-mode-mismatch: {parameter} ({parameter_role:?}) cannot bind access {} ({access_mode:?})",
                access.get()
            ),
            Self::UnboundAccess(access) => write!(
                formatter,
                "binding.unbound-access: access {} is not bound",
                access.get()
            ),
            Self::AccessStorageDisagreement {
                access,
                first,
                second,
            } => write!(
                formatter,
                "binding.access-storage-disagreement: {first} and {second} share access {} \
                 but declare different storage",
                access.get()
            ),
        }
    }
}

/// Checks a parameter-to-role binding against the ABI that declares the
/// parameters.
///
/// # Why this is not inferred
///
/// A boundary contract is keyed by tensor role, and an ABI names parameters.
/// Nothing connects them but the provider's claim that *this* call implements
/// *that* region with *this* parameter on that tensor. Inferring it from a
/// parameter's role or slot would reintroduce exactly what this module's named
/// parameters exist to prevent — `In` does not tell you which input.
pub(crate) fn check_bindings<Role: Copy + Eq>(
    abi: &CallAbi,
    bindings: &[(&'static str, Role)],
) -> Result<(), BindingError> {
    let mut seen: Vec<&'static str> = Vec::with_capacity(bindings.len());
    for (name, _) in bindings {
        if abi.parameter(name).is_none() {
            return Err(BindingError::UnknownParameter(name));
        }
        if seen.contains(name) {
            return Err(BindingError::ParameterBoundTwice(name));
        }
        seen.push(name);
    }
    for parameter in abi.parameters() {
        if !seen.contains(&parameter.name()) {
            return Err(BindingError::UnboundParameter(parameter.name()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::StorageScalar;

    /// A spec with the bounded profile's storage answers, so tests that care
    /// about names and roles do not have to restate the other three.
    fn layout_for(role: ParameterRole) -> ParameterLayout {
        match role {
            ParameterRole::In => ParameterLayout::Required(LayoutRequirement::DenseRowMajor),
            ParameterRole::Out => ParameterLayout::Guaranteed(LayoutGuarantee::DenseRowMajor),
            ParameterRole::InOut => ParameterLayout::Both {
                requires: LayoutRequirement::DenseRowMajor,
                guarantees: LayoutGuarantee::DenseRowMajor,
            },
        }
    }

    fn spec(name: &'static str, role: ParameterRole) -> ParameterSpec {
        ParameterSpec {
            name,
            role,
            layout: layout_for(role),
            encoding: StorageEncoding::Unpacked,
            alignment: ByteAlignment::natural_for(StorageScalar::F32),
        }
    }

    fn abi(parameters: impl IntoIterator<Item = (&'static str, ParameterRole)>) -> CallAbi {
        CallAbi::declare(parameters.into_iter().map(|(name, role)| spec(name, role)))
            .expect("a well-formed abi")
    }

    /// A duplicate or unnamed parameter is refused; a well-formed one is not.
    #[test]
    fn a_malformed_declaration_is_refused() {
        assert!(
            CallAbi::declare(
                [("x", ParameterRole::In), ("y", ParameterRole::Out)]
                    .map(|(name, role)| spec(name, role))
            )
            .is_ok(),
            "a well-formed abi was refused"
        );
        assert_eq!(
            CallAbi::declare(
                [("x", ParameterRole::In), ("x", ParameterRole::Out)]
                    .map(|(name, role)| spec(name, role))
            ),
            Err(AbiError::DuplicateName("x")),
        );
        assert_eq!(
            CallAbi::declare(
                [("x", ParameterRole::Out), ("", ParameterRole::In)]
                    .map(|(name, role)| spec(name, role))
            ),
            Err(AbiError::UnnamedParameter(1)),
        );
        assert_eq!(
            CallAbi::declare([("x/y", ParameterRole::Out)].map(|(name, role)| spec(name, role))),
            Err(AbiError::InvalidParameterName {
                slot: 0,
                name: "x/y",
            }),
        );
        let too_long = Box::leak("x".repeat(MAX_PARAMETER_NAME_BYTES + 1).into_boxed_str());
        assert_eq!(
            CallAbi::declare([spec(too_long, ParameterRole::Out)]),
            Err(AbiError::ParameterNameTooLong {
                slot: 0,
                actual: MAX_PARAMETER_NAME_BYTES + 1,
                maximum: MAX_PARAMETER_NAME_BYTES,
            }),
        );
        assert_eq!(
            CallAbi::declare([("x", ParameterRole::In)].map(|(name, role)| spec(name, role))),
            Err(AbiError::NoWrittenParameter),
        );
    }

    /// Declaration order sets the slots and does not decide compatibility.
    ///
    /// This is the property that separates this ABI from a positional one: the
    /// same parameters declared in either order are the same ABI.
    #[test]
    fn the_same_names_in_a_different_order_are_compatible() {
        let forward = abi([("input", ParameterRole::In), ("output", ParameterRole::Out)]);
        let reverse = abi([("output", ParameterRole::Out), ("input", ParameterRole::In)]);

        assert!(forward.is_compatible_with(&reverse));
        assert!(reverse.is_compatible_with(&forward));
        assert_eq!(
            forward.parameter("input").expect("declared").slot(),
            0,
            "slots do not follow declaration order"
        );
        assert_eq!(reverse.parameter("input").expect("declared").slot(), 1);
    }

    /// A swapped role at the same position is *not* compatible.
    ///
    /// The failure a positional ABI cannot see: two calls agreeing on arity and
    /// on every slot, differing only in what each slot is for.
    #[test]
    fn the_same_positions_with_swapped_roles_are_not_compatible() {
        let left = abi([("a", ParameterRole::In), ("b", ParameterRole::Out)]);
        let right = abi([("a", ParameterRole::Out), ("b", ParameterRole::In)]);

        assert_eq!(left.parameters().len(), right.parameters().len());
        assert!(
            !left.is_compatible_with(&right),
            "an input was accepted where an output was expected"
        );
    }

    /// Differing names at matching positions and roles are not compatible.
    #[test]
    fn different_names_are_not_compatible() {
        let left = abi([("a", ParameterRole::In), ("b", ParameterRole::Out)]);
        let right = abi([("c", ParameterRole::In), ("d", ParameterRole::Out)]);
        assert!(!left.is_compatible_with(&right));
    }

    /// `InOut` reads and writes; the other two do exactly one.
    #[test]
    fn roles_report_their_access() {
        assert!(ParameterRole::In.reads() && !ParameterRole::In.writes());
        assert!(!ParameterRole::Out.reads() && ParameterRole::Out.writes());
        assert!(ParameterRole::InOut.reads() && ParameterRole::InOut.writes());
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Role {
        In,
        Out,
    }

    /// A complete, agreeing binding is accepted.
    ///
    /// Without this the rejection tests below would pass against a check that
    /// refused everything.
    #[test]
    fn a_complete_binding_is_accepted() {
        let abi = abi([("a", ParameterRole::In), ("b", ParameterRole::Out)]);
        assert_eq!(
            check_bindings(&abi, &[("a", Role::In), ("b", Role::Out)]),
            Ok(())
        );
    }

    /// Every declared parameter must be bound, and only declared ones may be.
    #[test]
    fn an_incomplete_or_unknown_binding_is_refused() {
        let abi = abi([("a", ParameterRole::In), ("b", ParameterRole::Out)]);
        assert_eq!(
            check_bindings(&abi, &[("a", Role::In)]),
            Err(BindingError::UnboundParameter("b"))
        );
        assert_eq!(
            check_bindings(&abi, &[("a", Role::In), ("b", Role::Out), ("c", Role::In)]),
            Err(BindingError::UnknownParameter("c"))
        );
        assert_eq!(
            check_bindings(&abi, &[("a", Role::In), ("a", Role::Out), ("b", Role::Out)]),
            Err(BindingError::ParameterBoundTwice("a"))
        );
    }

    /// A layout stating the wrong direction for its role is refused.
    ///
    /// The accepting cases are driven for all three roles, so a check that
    /// refused everything — or that only understood `In` — fails here.
    #[test]
    fn a_layout_must_state_the_direction_its_role_has() {
        for role in [ParameterRole::In, ParameterRole::Out, ParameterRole::InOut] {
            assert!(
                CallAbi::declare([spec("a", role), spec("w", ParameterRole::Out)]).is_ok(),
                "a correctly-directed layout was refused for {role}"
            );
        }

        let guaranteeing_input = ParameterSpec {
            name: "a",
            role: ParameterRole::In,
            layout: ParameterLayout::Guaranteed(LayoutGuarantee::DenseRowMajor),
            encoding: StorageEncoding::Unpacked,
            alignment: ByteAlignment::natural_for(StorageScalar::F32),
        };
        assert_eq!(
            CallAbi::declare([guaranteeing_input, spec("w", ParameterRole::Out)]),
            Err(AbiError::LayoutDirectionMismatch("a")),
            "an input guaranteeing a layout was admitted"
        );

        let requiring_output = ParameterSpec {
            name: "w",
            role: ParameterRole::Out,
            layout: ParameterLayout::Required(LayoutRequirement::DenseRowMajor),
            encoding: StorageEncoding::Unpacked,
            alignment: ByteAlignment::natural_for(StorageScalar::F32),
        };
        assert_eq!(
            CallAbi::declare([requiring_output]),
            Err(AbiError::LayoutDirectionMismatch("w")),
            "an output requiring a layout was admitted"
        );
    }

    /// An input may require unit stride — the reason the two types differ.
    ///
    /// This is what the single-guarantee-typed field silently forbade, so it is
    /// pinned rather than left implied by the enum's existence.
    #[test]
    fn an_input_may_require_a_strided_layout() {
        let strided = ParameterSpec {
            name: "a",
            role: ParameterRole::In,
            layout: ParameterLayout::Required(LayoutRequirement::UnitStrideOnAxis {
                axis: tiler_ir::shape::Axis::new(0),
                rank: 2,
            }),
            encoding: StorageEncoding::Unpacked,
            alignment: ByteAlignment::natural_for(StorageScalar::F32),
        };
        assert!(
            CallAbi::declare([strided, spec("w", ParameterRole::Out)]).is_ok(),
            "an input requiring unit stride was refused"
        );
    }
}
