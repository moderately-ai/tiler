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

use core::fmt;

/// What a parameter is for.
///
/// Closed, and the distinctions are the ones a binding must not blur. An
/// `In` and an `Out` of identical shape and dtype are not interchangeable, and
/// a positional ABI cannot say so.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(
    dead_code,
    reason = "slice of implement-opaque-physical-call-providers: the ABI vocabulary lands before the providers that declare one"
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

#[allow(
    dead_code,
    reason = "see the module header: the role vocabulary and its access predicates land ahead of the seam that consumes them"
)]
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

/// One declared parameter of an opaque call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "see the module header: the parameter type lands with the ABI it belongs to"
)]
pub(crate) struct CallParameter {
    name: &'static str,
    role: ParameterRole,
    slot: u32,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft accessors whose consumer is the not-yet-written opaque-call seam"
)]
impl CallParameter {
    /// The parameter's declared name, unique within its ABI.
    pub(crate) const fn name(&self) -> &'static str {
        self.name
    }

    /// What the parameter is for.
    pub(crate) const fn role(&self) -> ParameterRole {
        self.role
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
        write!(formatter, "{}:{}@{}", self.name, self.role, self.slot)
    }
}

/// Why an ABI declaration was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "declaration outcome for the seam the engine will build on"
)]
pub(crate) enum AbiError {
    /// Two parameters share a name.
    DuplicateName(&'static str),
    /// A parameter has no name.
    ///
    /// An unnamed parameter can only be matched positionally, which is the
    /// whole thing this ABI exists to avoid.
    UnnamedParameter(u32),
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
            Self::NoWrittenParameter => {
                formatter.write_str("abi.no-written-parameter: the call writes nothing")
            }
        }
    }
}

/// The complete declared ABI of one opaque call.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "see the module header: the ABI lands ahead of the provider seam that carries it"
)]
pub(crate) struct CallAbi {
    parameters: Vec<CallParameter>,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft accessors whose consumer is the not-yet-written opaque-call seam"
)]
impl CallAbi {
    /// Declares an ABI from `(name, role)` pairs, in binding-table order.
    ///
    /// Slots are assigned from the iteration order rather than taken from the
    /// caller; see [`CallParameter::slot`].
    pub(crate) fn declare(
        parameters: impl IntoIterator<Item = (&'static str, ParameterRole)>,
    ) -> Result<Self, AbiError> {
        let mut declared: Vec<CallParameter> = Vec::new();
        for (slot, (name, role)) in parameters.into_iter().enumerate() {
            let slot = u32::try_from(slot).unwrap_or(u32::MAX);
            if name.is_empty() {
                return Err(AbiError::UnnamedParameter(slot));
            }
            if declared.iter().any(|existing| existing.name == name) {
                return Err(AbiError::DuplicateName(name));
            }
            declared.push(CallParameter { name, role, slot });
        }
        if !declared.iter().any(|parameter| parameter.role.writes()) {
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
            .find(|parameter| parameter.name == name)
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
                    .parameter(mine.name)
                    .is_some_and(|theirs| theirs.role == mine.role)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abi(parameters: impl IntoIterator<Item = (&'static str, ParameterRole)>) -> CallAbi {
        CallAbi::declare(parameters).expect("a well-formed abi")
    }

    /// A duplicate or unnamed parameter is refused; a well-formed one is not.
    #[test]
    fn a_malformed_declaration_is_refused() {
        assert!(
            CallAbi::declare([("x", ParameterRole::In), ("y", ParameterRole::Out)]).is_ok(),
            "a well-formed abi was refused"
        );
        assert_eq!(
            CallAbi::declare([("x", ParameterRole::In), ("x", ParameterRole::Out)]),
            Err(AbiError::DuplicateName("x")),
        );
        assert_eq!(
            CallAbi::declare([("x", ParameterRole::Out), ("", ParameterRole::In)]),
            Err(AbiError::UnnamedParameter(1)),
        );
        assert_eq!(
            CallAbi::declare([("x", ParameterRole::In)]),
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
}
