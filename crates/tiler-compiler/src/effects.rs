//! What an opaque call may touch, and what an optimizer may therefore do around it.
//!
//! A slice of `implement-opaque-physical-call-providers`. An opaque call is one
//! whose body the compiler does not model, so every question an optimizer would
//! normally answer by inspection — may this be reordered, fused, eliminated,
//! executed twice — has to be answered from a *declaration* instead.
//!
//! # The one rule everything here follows
//!
//! An undeclared or unknown effect is the **most conservative** effect, never
//! the most convenient one. `AGENTS.md` puts it generally — extension
//! mechanisms must preserve validation and feasibility, and "extensible" does
//! not mean unknown behaviour is optimizable — and the opaque-call ticket puts
//! it sharply: an opaque call may not smuggle unknown semantics or effects into
//! logical IR.
//!
//! This is why [`CallEffects::unknown`] exists and why it is what
//! [`Default`] would give you if this type had one. It does not have one, and
//! that is deliberate: a `Default` impl reads as "the ordinary case", and the
//! ordinary case for an opaque call is *not knowing*, which a caller should
//! have to write down rather than receive by omission.

use core::fmt;

/// Whether a call may be removed when its results are unused.
#[allow(
    dead_code,
    reason = "constructed only by opaque-call providers, and no production provider exists yet: the declaration path consumes these live (matching and reading them at admission), but nothing outside tests builds a CallEffects. The compile-path constructor arrives with caller-supplied physical providers"
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Elimination {
    /// Removing the call when its results are unused is observationally
    /// equivalent.
    Removable,
    /// The call must run even if nothing reads its results.
    ///
    /// The conservative answer, and the one an undeclared call gets. A call
    /// that writes storage the compiler does not model, or that the runtime
    /// observes, is not dead merely because its return value is.
    Required,
}

/// Whether a call may be executed more than once, or moved across other work.
#[allow(
    dead_code,
    reason = "constructed only by opaque-call providers, and no production provider exists yet: the declaration path consumes these live (matching and reading them at admission), but nothing outside tests builds a CallEffects. The compile-path constructor arrives with caller-supplied physical providers"
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Motion {
    /// The call is a pure function of its inputs: reordering it against
    /// anything it does not data-depend on, or evaluating it twice, is
    /// observationally equivalent.
    Free,
    /// The call keeps its position relative to other ordered effects.
    ///
    /// The conservative answer. Two calls that both write, or one that writes
    /// and one that reads the same storage, may not be swapped even when no
    /// value flows between them.
    Ordered,
}

/// Whether a call's results may share storage with its inputs.
///
/// This is separate from [`Motion`] because the questions are independent: a
/// pure call can still return a view onto an input, and an ordered call can
/// return storage that aliases nothing. Collapsing them would make one
/// declaration answer a question it was never asked.
#[allow(
    dead_code,
    reason = "constructed only by opaque-call providers, and no production provider exists yet: the declaration path consumes these live (matching and reading them at admission), but nothing outside tests builds a CallEffects. The compile-path constructor arrives with caller-supplied physical providers"
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Aliasing {
    /// Results occupy storage distinct from every input.
    Distinct,
    /// A result may alias an input.
    ///
    /// The conservative answer. A caller must not reuse an input's storage
    /// after the call, and the boundary property model's `MaterializationForm`
    /// consequences follow from this rather than from the call's return type.
    MayAliasInputs,
}

/// The complete effect declaration of one opaque call.
///
/// Every field has a conservative value, and [`Self::unknown`] is all of them
/// at once. There is no `Default`: see the module header for why an opaque
/// call's "ordinary case" must be written down rather than received by
/// omission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallEffects {
    elimination: Elimination,
    motion: Motion,
    aliasing: Aliasing,
}

#[allow(
    dead_code,
    reason = "constructed only by opaque-call providers, and no production provider exists yet: the declaration path consumes these live (matching and reading them at admission), but nothing outside tests builds a CallEffects. The compile-path constructor arrives with caller-supplied physical providers"
)]
impl CallEffects {
    /// The declaration for a call nothing is known about.
    ///
    /// Maximally conservative on every axis. This is what an unregistered,
    /// undeclared, or partially-declared call must be treated as — never a
    /// permissive default, and never "pure until proven otherwise".
    pub(crate) const fn unknown() -> Self {
        Self {
            elimination: Elimination::Required,
            motion: Motion::Ordered,
            aliasing: Aliasing::MayAliasInputs,
        }
    }

    /// A declaration a provider states explicitly.
    pub(crate) const fn declared(
        elimination: Elimination,
        motion: Motion,
        aliasing: Aliasing,
    ) -> Self {
        Self {
            elimination,
            motion,
            aliasing,
        }
    }

    /// Whether the call may be removed when its results are unused.
    pub(crate) const fn elimination(self) -> Elimination {
        self.elimination
    }

    /// Whether the call may be reordered or re-executed.
    pub(crate) const fn motion(self) -> Motion {
        self.motion
    }

    /// Whether the call's results may share storage with its inputs.
    pub(crate) const fn aliasing(self) -> Aliasing {
        self.aliasing
    }

    /// Whether this declaration permits *any* optimization the conservative
    /// one would not.
    ///
    /// Lets a caller decide whether a declaration is worth carrying
    /// at all; a declaration equal to [`Self::unknown`] enables nothing and a
    /// provider stating it has said only that it does not know.
    pub(crate) fn permits_more_than_unknown(self) -> bool {
        self != Self::unknown()
    }

    /// The conservative meet of two declarations.
    ///
    /// Used when a region contains more than one opaque call: the region as a
    /// whole may only be optimized as far as its most restrictive member
    /// allows. Written as an explicit match per axis rather than a numeric
    /// minimum, so adding a third value to any axis is a build error here
    /// instead of silently ordering itself against the others.
    pub(crate) const fn meet(self, other: Self) -> Self {
        Self {
            elimination: match (self.elimination, other.elimination) {
                (Elimination::Removable, Elimination::Removable) => Elimination::Removable,
                (Elimination::Required, _) | (_, Elimination::Required) => Elimination::Required,
            },
            motion: match (self.motion, other.motion) {
                (Motion::Free, Motion::Free) => Motion::Free,
                (Motion::Ordered, _) | (_, Motion::Ordered) => Motion::Ordered,
            },
            aliasing: match (self.aliasing, other.aliasing) {
                (Aliasing::Distinct, Aliasing::Distinct) => Aliasing::Distinct,
                (Aliasing::MayAliasInputs, _) | (_, Aliasing::MayAliasInputs) => {
                    Aliasing::MayAliasInputs
                }
            },
        }
    }
}

impl fmt::Display for CallEffects {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elimination = match self.elimination {
            Elimination::Removable => "removable",
            Elimination::Required => "required",
        };
        let motion = match self.motion {
            Motion::Free => "free",
            Motion::Ordered => "ordered",
        };
        let aliasing = match self.aliasing {
            Aliasing::Distinct => "distinct",
            Aliasing::MayAliasInputs => "may-alias-inputs",
        };
        write!(formatter, "{elimination}/{motion}/{aliasing}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The unknown declaration is conservative on every axis.
    ///
    /// Asserted field by field rather than against a constructed twin, so a
    /// constructor that flipped one axis to a permissive value would fail here
    /// rather than agree with itself.
    #[test]
    fn the_unknown_declaration_permits_nothing() {
        let unknown = CallEffects::unknown();
        assert_eq!(unknown.elimination(), Elimination::Required);
        assert_eq!(unknown.motion(), Motion::Ordered);
        assert_eq!(unknown.aliasing(), Aliasing::MayAliasInputs);
        assert!(
            !unknown.permits_more_than_unknown(),
            "the unknown declaration reported that it enables an optimization"
        );
    }

    /// A declaration permissive on any single axis is distinguishable.
    ///
    /// Each axis is checked on its own, so a comparison that only looked at one
    /// field would fail on the other two.
    #[test]
    fn a_single_permissive_axis_is_more_than_unknown() {
        for permissive in [
            CallEffects::declared(
                Elimination::Removable,
                Motion::Ordered,
                Aliasing::MayAliasInputs,
            ),
            CallEffects::declared(
                Elimination::Required,
                Motion::Free,
                Aliasing::MayAliasInputs,
            ),
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct),
        ] {
            assert!(
                permissive.permits_more_than_unknown(),
                "{permissive} was not distinguished from the unknown declaration"
            );
        }
    }

    /// The meet of anything with the unknown declaration is unknown.
    ///
    /// This is the property that makes a region containing one undeclared call
    /// safe: the region cannot be optimized past its worst member, however
    /// permissive the others are.
    #[test]
    fn one_undeclared_call_constrains_the_whole_region() {
        let permissive =
            CallEffects::declared(Elimination::Removable, Motion::Free, Aliasing::Distinct);
        let unknown = CallEffects::unknown();

        assert_eq!(permissive.meet(unknown), unknown);
        assert_eq!(
            unknown.meet(permissive),
            unknown,
            "the meet is not symmetric, so the result depends on member order"
        );
        assert_eq!(
            permissive.meet(permissive),
            permissive,
            "two permissive declarations were needlessly constrained"
        );
    }

    /// The meet is per-axis, not all-or-nothing.
    ///
    /// Two declarations each permissive on a different axis must meet to the
    /// conservative value on both — not to either input, and not to one of them
    /// chosen by position.
    #[test]
    fn the_meet_is_taken_on_each_axis_independently() {
        let removable =
            CallEffects::declared(Elimination::Removable, Motion::Ordered, Aliasing::Distinct);
        let free = CallEffects::declared(Elimination::Required, Motion::Free, Aliasing::Distinct);
        let met = removable.meet(free);

        assert_eq!(met.elimination(), Elimination::Required);
        assert_eq!(met.motion(), Motion::Ordered);
        assert_eq!(
            met.aliasing(),
            Aliasing::Distinct,
            "an axis both declarations agreed on was needlessly constrained"
        );
    }
}
