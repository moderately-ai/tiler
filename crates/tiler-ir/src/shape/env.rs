//! The `ShapeEnv` authority: scoped symbols, typed root bindings, semantic
//! constraints, and identity.
//!
//! # Where this surface is reachable from
//!
//! The accepted subset is re-exported flat from [`crate::shape`]; this module
//! itself is `pub(crate)`, so the decision procedure, its disjoint-set forest,
//! and its per-class domains stay inside it. The additive relation and its
//! corresponding conflict and fragment variants are a public draft pending
//! Tom's acceptance, not part of that accepted subset. A frontend constructs
//! an environment through [`ShapeEnvBuilder`] and reads it through
//! [`ShapeEnv`]'s queries.
//!
//! # What this module owns
//!
//! `docs/ir.md`'s constraint-and-proof-context section specifies a `ShapeEnv`
//! as scoped symbol declarations, source bindings, *and* a constraint
//! environment over extent equalities (including a fixed two-addend equality),
//! divisibility, nonnegativity, intervals, and factorization. This file owns the
//! first two and the storage, identity, and lifecycle of the third;
//! [`constraint`] owns the constraint vocabulary and the decision procedure,
//! and its module documentation states the exact arithmetic fragment that
//! procedure decides.
//!
//! # The invariants this module establishes
//!
//! **Scope is part of identity.** The contract states that "equal spelling in
//! different scopes never implies equality". A [`ShapeSymbol`] therefore pairs a
//! name with a [`SymbolScope`], and two symbols spelled `n` in different scopes
//! are different symbols at every comparison, in identity bytes, and in binding
//! lookup. Storing the name alone and comparing scopes separately would make
//! every future consumer responsible for remembering the rule.
//!
//! **Every symbol has exactly one declaration and one root binding.** A second
//! declaration is [`ShapeEnvError::DuplicateDeclaration`] and a second binding
//! is [`ShapeEnvError::AlreadyBound`]; neither is last-write-wins. A symbol left
//! unbound at [`ShapeEnvBuilder::build`] is [`ShapeEnvError::FreeSymbol`],
//! because the contract makes free symbols invalid rather than deferred.
//!
//! **A binding states where its value comes from and when it can be read.** The
//! verifier requires that "every root extent symbol has exactly one typed
//! binding whose source class and availability phase are supported by every
//! semantic factor that consumes it", so both travel on the binding rather than
//! being inferred from the source class. They are genuinely independent: two
//! target properties can be readable at different phases.
//!
//! **Availability phases are ADR 0043's, not a second set.** [`AvailabilityPhase`]
//! is the one in [`crate::program::abi`]. A shape-local copy would be the same
//! defect `relocate-abi-expressions-into-tiler-ir` closed twice.
//!
//! **Contradiction is decided at `build`, not deferred to a checked step.** The
//! contract says "contradictory semantic constraints reject the graph", so an
//! environment the contract calls invalid must never exist as a verified
//! product. ADR 0071 makes the consuming `build` the whole-object verification
//! point precisely so that holding a verified value is sufficient evidence; a
//! separate `check_constraints` step would reintroduce the second pass that
//! rule exists to remove, and every consumer would then have to prove it ran.
//! The cost is that a contradiction is found once, at construction, rather than
//! amortized — which is the correct trade for a value that is built once and
//! read by every later stage.
//!
//! **Root bindings participate in that decision.** A symbol bound to
//! [`BindingSource::Static`] enters the constraint system as a constant, so
//! a constraint contradicting a statically bound extent is rejected here rather
//! than surviving into index lowering. Bindings whose value is not known until a
//! later phase contribute no value and constrain nothing.

use core::fmt;
use std::error::Error;

use super::{Axis, Extent};
use crate::identity::{push_len, push_slice};
use crate::program::abi::{AvailabilityPhase, TargetPropertyKey};
use crate::semantic::InputKey;

pub(crate) mod constraint;

// Flat re-export rather than a published `constraint` module: the constraint
// vocabulary is part of the accepted `ShapeEnv` surface, while the decision
// procedure, its disjoint-set forest, and its per-class domains are not. A
// module published because its file exists would carry the second along with
// the first.
pub use constraint::{
    ConstraintConflict, ExtentInterval, ExtentRelation, ExtentTerm, FragmentViolation,
    GuardApplicability, SemanticInputConstraint, VariantGuard,
};

/// Domain separator of a canonical shape-environment encoding.
///
/// `v3` rather than `v2`: a [`BindingSource::TargetProperty`] no longer carries
/// a version field beside its already-versioned key, so a binding encodes to
/// different bytes than it did. Bumping states that rather than letting two
/// encodings share one domain. As with the `v1` to `v2` change, no durable
/// artifact, cache, or cross-process reader ever observed an earlier version, so
/// this records the change rather than migrating anything.
///
/// Publishing this vocabulary did not move the version. No byte an environment
/// encodes changed, and a domain that advanced for a visibility change alone
/// would make two identical subjects carry different domains — which is the
/// defect a domain separator exists to prevent.
const SHAPE_ENV_DOMAIN: &[u8] = b"tiler.shape-env.v3\0";

/// Largest number of bytes a symbol name may occupy.
const MAX_SYMBOL_NAME_BYTES: usize = 128;

/// The scope one shape symbol is declared in.
///
/// Scopes are compared by their opaque bytes and are never parsed. A scope
/// exists so that two independently constructed subjects — two program inputs,
/// two inlined frontend regions — can each declare a symbol named `n` without
/// those symbols becoming equal, which the contract requires and which a bare
/// name cannot express.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolScope(Vec<u8>);

impl SymbolScope {
    /// Creates a scope from opaque bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::EmptyScope`] for an empty scope, which would
    /// otherwise be indistinguishable from an absent one.
    pub fn new(bytes: impl AsRef<[u8]>) -> Result<Self, ShapeEnvError> {
        let bytes = bytes.as_ref();
        if bytes.is_empty() {
            return Err(ShapeEnvError::EmptyScope);
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Returns the scope's opaque bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One scoped extent symbol.
///
/// Name and scope together are the symbol. Neither is meaningful alone: the
/// contract's rule that equal spelling in different scopes never implies
/// equality is enforced by this pairing rather than by callers remembering it.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShapeSymbol {
    scope: SymbolScope,
    name: String,
}

impl ShapeSymbol {
    /// Declares a symbol in a scope.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::EmptySymbolName`] for an empty name, or
    /// [`ShapeEnvError::SymbolNameTooLong`] past the governed bound.
    pub fn new(scope: SymbolScope, name: impl Into<String>) -> Result<Self, ShapeEnvError> {
        let name = name.into();
        if name.is_empty() {
            return Err(ShapeEnvError::EmptySymbolName);
        }
        if name.len() > MAX_SYMBOL_NAME_BYTES {
            return Err(ShapeEnvError::SymbolNameTooLong {
                actual: name.len(),
                limit: MAX_SYMBOL_NAME_BYTES,
            });
        }
        Ok(Self { scope, name })
    }

    /// Returns the scope this symbol is declared in.
    #[must_use]
    pub const fn scope(&self) -> &SymbolScope {
        &self.scope
    }

    /// Returns the symbol's name within its scope.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Appends this symbol's canonical bytes.
    ///
    /// Crate-internal: an identity encoder establishes what the bytes mean, and
    /// ADR 0074 keeps that authority with the module that defines the meaning
    /// rather than exposing it to a caller who could derive an identity under
    /// different rules.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        push_slice(bytes, self.scope.as_bytes());
        push_slice(bytes, self.name.as_bytes());
    }

    /// Returns the exact byte length [`Self::encode`] appends.
    ///
    /// Derived from the same two length-prefixed runs the encoder writes, so a
    /// consumer that must size a buffer before encoding cannot disagree with it
    /// about what a symbol costs.
    pub(crate) fn encoded_len(&self) -> usize {
        const PREFIX: usize = size_of::<u64>();
        PREFIX + self.scope.as_bytes().len() + PREFIX + self.name.len()
    }
}

impl fmt::Display for ShapeSymbol {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.scope.0.escape_ascii(), self.name)
    }
}

/// Largest number of bytes an interface-parameter key may occupy.
const MAX_INTERFACE_PARAMETER_KEY_BYTES: usize = 1_024;

/// A stable key of one declared host interface parameter.
///
/// A newtype rather than a bare `String` because the accepted shape-environment
/// contract makes that a correctness requirement, not a style preference:
/// "extent values, signed shape intermediates, symbol IDs, axis indices, input
/// indices, interface-parameter indices, target property keys, binding phases,
/// and physical index widths must not be accidentally mixed merely because
/// their representations are primitive types."
///
/// This is the crate's first definition of the concept rather than a second
/// one; an input tensor's interface key is [`InputKey`] and a governed device
/// property is [`TargetPropertyKey`], and the contract keeps all three apart.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InterfaceParameterKey(String);

impl InterfaceParameterKey {
    /// Creates a nonempty stable interface-parameter key.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::EmptyInterfaceParameterKey`] for an empty key,
    /// or [`ShapeEnvError::InterfaceParameterKeyTooLong`] past the governed
    /// bound.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ShapeEnvError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ShapeEnvError::EmptyInterfaceParameterKey);
        }
        if value.len() > MAX_INTERFACE_PARAMETER_KEY_BYTES {
            return Err(ShapeEnvError::InterfaceParameterKeyTooLong {
                actual: value.len(),
                limit: MAX_INTERFACE_PARAMETER_KEY_BYTES,
            });
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the exact key text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InterfaceParameterKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Where a root binding's value comes from.
///
/// These are the four classes [ADR 0008](../../../../docs/decisions/0008-typed-root-bindings.md)
/// names, spelled as it spells them — `Static`, `InputDimension`,
/// `InterfaceParameter`, `TargetProperty`. The class is recorded rather than
/// inferred from the value, because two bindings can carry the same number for
/// different reasons and only one of them may be legal for a given consumer.
///
/// Each field is the governed type the crate already defines for that concept
/// rather than a primitive standing in for it, per the contract's newtype
/// mandate. That is what lets a consumer check an `InputDimension` binding
/// against the input a region actually declares instead of comparing strings.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingSource {
    /// An extent fixed at graph construction.
    Static(Extent),
    /// One axis extent of a bound program input's shape metadata.
    ///
    /// Metadata, never element data: the contract models a runtime integer as
    /// an explicit interface parameter rather than encoding it in a tensor's
    /// contents.
    InputDimension {
        /// Interface key of the input the extent is read from.
        input: InputKey,
        /// Axis position within that input's shape.
        axis: Axis,
    },
    /// An immutable host integer declared by the program interface.
    InterfaceParameter {
        /// Stable parameter key.
        key: InterfaceParameterKey,
    },
    /// An admitted governed target property.
    ///
    /// [`TargetPropertyKey`] is already the crate's stable *versioned* key, so
    /// the version travels inside it. A second version field beside the key
    /// would be a second authority over the same fact.
    TargetProperty {
        /// Governed property key.
        key: TargetPropertyKey,
    },
}

impl BindingSource {
    /// Returns the governed tag of this source class.
    ///
    /// Exhaustive rather than a discriminant read, so adding a source class is
    /// a build error here instead of a silent re-encoding of every shape
    /// environment ever identified (ADR 0074 convention 3).
    const fn tag(&self) -> u8 {
        match self {
            Self::Static(_) => 0x01,
            Self::InputDimension { .. } => 0x02,
            Self::InterfaceParameter { .. } => 0x03,
            Self::TargetProperty { .. } => 0x04,
        }
    }

    /// Returns the earliest phase this source can be read at.
    ///
    /// A static extent is known from the compile profile; every other class
    /// depends on something the compiler does not have until later. This is the
    /// *floor*: a binding may declare a later phase than its class requires,
    /// which [`RootBinding::new`] checks, but never an earlier one.
    const fn earliest_phase(&self) -> AvailabilityPhase {
        match self {
            Self::Static(_) => AvailabilityPhase::CompileProfile,
            // An input extent and an interface parameter are both known once a
            // concrete invocation exists, which is no earlier than a bound
            // device and context.
            Self::InputDimension { .. } | Self::InterfaceParameter { .. } => {
                AvailabilityPhase::LiveDevicePreflight
            }
            Self::TargetProperty { .. } => AvailabilityPhase::LiveDevicePreflight,
        }
    }

    /// Returns the extent this source fixes, when it fixes one statically.
    ///
    /// Only [`Self::Static`] does. Every other class names a value the compiler
    /// does not hold, which is the distinction a consumer needs before it can
    /// treat a symbol as a literal.
    #[must_use]
    pub const fn static_extent(&self) -> Option<Extent> {
        match self {
            Self::Static(extent) => Some(*extent),
            Self::InputDimension { .. }
            | Self::InterfaceParameter { .. }
            | Self::TargetProperty { .. } => None,
        }
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        match self {
            Self::Static(extent) => bytes.extend_from_slice(&extent.get().to_be_bytes()),
            Self::InputDimension { input, axis } => {
                push_slice(bytes, input.as_str().as_bytes());
                bytes.extend_from_slice(&axis.get().to_be_bytes());
            }
            Self::InterfaceParameter { key } => push_slice(bytes, key.as_str().as_bytes()),
            Self::TargetProperty { key } => push_slice(bytes, key.as_str().as_bytes()),
        }
    }
}

/// How a fact about a symbol was established.
///
/// `docs/ir.md`: "Facts record provenance: statically proven, frontend-required,
/// or runtime-validated." The distinction is load-bearing rather than
/// descriptive — the contract also forbids inferred or proven facts from
/// "silently becoming additional frontend-required semantics", which is only
/// expressible if the three are separate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FactProvenance {
    /// Established by the compiler from the program itself.
    StaticallyProven,
    /// Demanded by the frontend as a condition of the program's meaning.
    FrontendRequired,
    /// Checked at runtime before dependent work begins.
    RuntimeValidated,
}

impl FactProvenance {
    /// Returns the governed tag of this provenance, exhaustively.
    const fn tag(self) -> u8 {
        match self {
            Self::StaticallyProven => 0x01,
            Self::FrontendRequired => 0x02,
            Self::RuntimeValidated => 0x03,
        }
    }
}

/// The single typed root binding of one extent symbol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootBinding {
    source: BindingSource,
    phase: AvailabilityPhase,
    provenance: FactProvenance,
}

impl RootBinding {
    /// Binds one symbol's root value.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::PhaseTooEarly`] when the declared phase is
    /// earlier than the source class can supply. A binding that claimed a
    /// device property were readable from the compile profile would let a
    /// consumer evaluate it before any device exists, which is the failure the
    /// availability ladder is for.
    pub fn new(
        source: BindingSource,
        phase: AvailabilityPhase,
        provenance: FactProvenance,
    ) -> Result<Self, ShapeEnvError> {
        let earliest = source.earliest_phase();
        if phase < earliest {
            return Err(ShapeEnvError::PhaseTooEarly {
                declared: phase,
                earliest,
            });
        }
        Ok(Self {
            source,
            phase,
            provenance,
        })
    }

    /// Returns where this binding's value comes from.
    #[must_use]
    pub const fn source(&self) -> &BindingSource {
        &self.source
    }

    /// Returns the phase at which this binding becomes readable.
    #[must_use]
    pub const fn phase(&self) -> AvailabilityPhase {
        self.phase
    }

    /// Returns how this binding was established.
    #[must_use]
    pub const fn provenance(&self) -> FactProvenance {
        self.provenance
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        self.source.encode(bytes);
        bytes.push(self.phase.tag());
        bytes.push(self.provenance.tag());
    }
}

/// A typed failure of shape-environment construction.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ShapeEnvError {
    /// A scope was empty, which is indistinguishable from an absent scope.
    EmptyScope,
    /// A symbol name was empty.
    EmptySymbolName,
    /// A symbol name exceeded the governed bound.
    SymbolNameTooLong {
        /// Bytes the rejected name occupied.
        actual: usize,
        /// Governed limit.
        limit: usize,
    },
    /// An interface-parameter key was empty.
    EmptyInterfaceParameterKey,
    /// An interface-parameter key exceeded the governed bound.
    InterfaceParameterKeyTooLong {
        /// Bytes the rejected key occupied.
        actual: usize,
        /// Governed limit.
        limit: usize,
    },
    /// A symbol was declared twice in one scope.
    DuplicateDeclaration {
        /// The symbol whose second declaration was rejected.
        symbol: ShapeSymbol,
    },
    /// A symbol was bound twice.
    ///
    /// Not last-write-wins: the contract gives each symbol exactly one root
    /// binding, so a second one is a contradiction rather than an update.
    AlreadyBound {
        /// The symbol whose second binding was rejected.
        symbol: ShapeSymbol,
    },
    /// A binding named a symbol that was never declared.
    UndeclaredSymbol {
        /// The symbol the rejected binding named.
        symbol: ShapeSymbol,
    },
    /// A declared symbol reached `build` with no root binding.
    ///
    /// The contract makes free symbols invalid rather than deferred, so this
    /// fails construction instead of producing an environment a later stage
    /// would have to re-check.
    FreeSymbol {
        /// The symbol left unbound.
        symbol: ShapeSymbol,
    },
    /// A binding claimed a phase earlier than its source class can supply.
    PhaseTooEarly {
        /// The phase the rejected binding declared.
        declared: AvailabilityPhase,
        /// The earliest phase its source class admits.
        earliest: AvailabilityPhase,
    },
    /// An interval relation was written with its bounds inverted.
    EmptyInterval {
        /// The rejected lower bound.
        lower: u64,
        /// The rejected upper bound.
        upper: u64,
    },
    /// A factorization named fewer than two factors.
    DegenerateFactorization {
        /// How many factors the rejected relation named.
        factors: usize,
    },
    /// A relation named a symbol that was never declared.
    ConstraintOnUndeclaredSymbol {
        /// The symbol the rejected relation named.
        symbol: ShapeSymbol,
    },
    /// A relation lies outside the supported arithmetic fragment.
    ///
    /// Refused rather than admitted and under-decided: the contract's rule that
    /// contradictory constraints reject the graph is only meaningful if
    /// "no contradiction" is a decision, and an undecidable relation would make
    /// it indistinguishable from an unexamined one.
    UnsupportedRelation {
        /// The refused relation.
        relation: Box<ExtentRelation>,
        /// Which part of the fragment boundary it crossed.
        violation: FragmentViolation,
    },
    /// The semantic input constraints cannot all hold.
    ///
    /// The contract makes this reject the graph, so it fails `build` rather
    /// than producing an environment a later stage would have to re-check.
    ContradictoryConstraints {
        /// The explained reason no assignment satisfies the set.
        conflict: Box<ConstraintConflict>,
    },
}

impl fmt::Display for ShapeEnvError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyScope => formatter.write_str("shape-env.empty-scope: rejected"),
            Self::EmptySymbolName => formatter.write_str("shape-env.empty-symbol-name: rejected"),
            Self::SymbolNameTooLong { actual, limit } => write!(
                formatter,
                "shape-env.symbol-name-too-long: {actual} bytes exceeds {limit}"
            ),
            Self::EmptyInterfaceParameterKey => {
                formatter.write_str("shape-env.empty-interface-parameter-key: rejected")
            }
            Self::InterfaceParameterKeyTooLong { actual, limit } => write!(
                formatter,
                "shape-env.interface-parameter-key-too-long: {actual} bytes exceeds {limit}"
            ),
            Self::DuplicateDeclaration { symbol } => {
                write!(formatter, "shape-env.duplicate-declaration: {symbol}")
            }
            Self::AlreadyBound { symbol } => {
                write!(formatter, "shape-env.already-bound: {symbol}")
            }
            Self::UndeclaredSymbol { symbol } => {
                write!(formatter, "shape-env.undeclared-symbol: {symbol}")
            }
            Self::FreeSymbol { symbol } => write!(formatter, "shape-env.free-symbol: {symbol}"),
            Self::PhaseTooEarly { declared, earliest } => write!(
                formatter,
                "shape-env.phase-too-early: {declared} precedes {earliest}"
            ),
            Self::EmptyInterval { lower, upper } => write!(
                formatter,
                "shape-env.empty-interval: lower {lower} exceeds upper {upper}"
            ),
            Self::DegenerateFactorization { factors } => write!(
                formatter,
                "shape-env.degenerate-factorization: {factors} factors, at least two required"
            ),
            Self::ConstraintOnUndeclaredSymbol { symbol } => {
                write!(
                    formatter,
                    "shape-env.constraint-on-undeclared-symbol: {symbol}"
                )
            }
            Self::UnsupportedRelation {
                relation,
                violation,
            } => write!(
                formatter,
                "shape-env.unsupported-relation: `{relation}` is outside the supported fragment: {violation}"
            ),
            Self::ContradictoryConstraints { conflict } => {
                write!(formatter, "shape-env.contradictory-constraints: {conflict}")
            }
        }
    }
}

impl Error for ShapeEnvError {}

/// The canonical identity of one verified shape environment.
///
/// Opaque bytes with no public constructor, per ADR 0074: only the encoder that
/// establishes what the identity means produces one.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShapeEnvIdentity(Vec<u8>);

impl ShapeEnvIdentity {
    /// Returns the identity's canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An append-only draft shape environment.
///
/// Declarations, bindings, constraints, and guards are separate steps because
/// the contract separates them: a symbol exists once declared, remains invalid
/// until bound, and the constraints over it are a different kind of statement
/// than the guards that only qualify one optimization.
#[derive(Clone, Debug, Default)]
pub struct ShapeEnvBuilder {
    entries: Vec<(ShapeSymbol, Option<RootBinding>)>,
    constraints: Vec<SemanticInputConstraint>,
    guards: Vec<VariantGuard>,
}

impl ShapeEnvBuilder {
    /// Opens an empty draft.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares one scoped symbol.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::DuplicateDeclaration`] when the symbol is
    /// already declared. The check leaves the draft unchanged.
    pub fn declare(&mut self, symbol: ShapeSymbol) -> Result<(), ShapeEnvError> {
        if self.position(&symbol).is_some() {
            return Err(ShapeEnvError::DuplicateDeclaration { symbol });
        }
        self.entries.push((symbol, None));
        Ok(())
    }

    /// Binds one declared symbol's root value.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::UndeclaredSymbol`] when the symbol was never
    /// declared, or [`ShapeEnvError::AlreadyBound`] when it already has a root
    /// binding. Both leave the draft unchanged.
    pub fn bind(
        &mut self,
        symbol: &ShapeSymbol,
        binding: RootBinding,
    ) -> Result<(), ShapeEnvError> {
        let Some(position) = self.position(symbol) else {
            return Err(ShapeEnvError::UndeclaredSymbol {
                symbol: symbol.clone(),
            });
        };
        if self.entries[position].1.is_some() {
            return Err(ShapeEnvError::AlreadyBound {
                symbol: symbol.clone(),
            });
        }
        self.entries[position].1 = Some(binding);
        Ok(())
    }

    /// Records one semantic input constraint.
    ///
    /// Required for the program's expressions to be defined: a contradictory
    /// set fails [`Self::build`]. Use [`Self::guard`] for a predicate that only
    /// one optimization needs.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::ConstraintOnUndeclaredSymbol`] when the relation
    /// names a symbol this draft has not declared. The check leaves the draft
    /// unchanged.
    pub fn require(&mut self, constraint: SemanticInputConstraint) -> Result<(), ShapeEnvError> {
        let constraint = constraint.canonicalized();
        self.check_declared(constraint.relation())?;
        self.constraints.push(constraint);
        Ok(())
    }

    /// Records one variant guard.
    ///
    /// Required only for the optimization it qualifies: an unsatisfiable guard
    /// does not fail [`Self::build`], and is instead reported by
    /// [`ShapeEnv::unsatisfiable_guards`] so planning selects another variant.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::ConstraintOnUndeclaredSymbol`] when the relation
    /// names a symbol this draft has not declared. The check leaves the draft
    /// unchanged.
    pub fn guard(&mut self, guard: VariantGuard) -> Result<(), ShapeEnvError> {
        let guard = guard.canonicalized();
        self.check_declared(guard.relation())?;
        self.guards.push(guard);
        Ok(())
    }

    /// Verifies the draft and freezes it.
    ///
    /// Whole-object verification per ADR 0071: free symbols, the supported
    /// arithmetic fragment, and constraint contradiction are all decided here,
    /// so a returned [`ShapeEnv`] needs no second pass to be trustworthy.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::FreeSymbol`] naming the first declared symbol
    /// with no root binding, in canonical order so the diagnostic does not
    /// depend on declaration order; [`ShapeEnvError::UnsupportedRelation`] when
    /// a constraint or guard lies outside the supported fragment; and
    /// [`ShapeEnvError::ContradictoryConstraints`] when the semantic input
    /// constraints and root bindings cannot all hold.
    pub fn build(self) -> Result<ShapeEnv, ShapeEnvError> {
        // Canonical order first, so both the rejection and the identity are
        // functions of the environment rather than of how it was authored.
        let mut entries = self.entries;
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        let mut bound = Vec::with_capacity(entries.len());
        for (symbol, binding) in entries {
            let Some(binding) = binding else {
                return Err(ShapeEnvError::FreeSymbol { symbol });
            };
            bound.push((symbol, binding));
        }

        let mut constraints = self.constraints;
        constraints.sort();
        // Only an exact (relation, provenance) repeat is removed. Two
        // assertions of one relation under different provenance stay two
        // constraints: merging them would decide which reason survived, which
        // is how a proven fact would silently become a required one.
        constraints.dedup();

        let mut guards = self.guards;
        guards.sort();
        guards.dedup();

        let relations: Vec<&ExtentRelation> = constraints
            .iter()
            .map(SemanticInputConstraint::relation)
            .collect();
        constraint::decide(&bound, &relations)?;

        // Guards are decided separately and only for decidability. Their
        // failure is a planning outcome, not an invalid input, so a
        // contradiction here is deliberately not propagated.
        for guard in &guards {
            if let Err(error @ ShapeEnvError::UnsupportedRelation { .. }) =
                guard_verdict(&bound, &relations, guard)
            {
                return Err(error);
            }
        }

        let identity = ShapeEnvIdentity(encode_environment(&bound, &constraints));
        Ok(ShapeEnv {
            entries: bound,
            constraints,
            guards,
            identity,
        })
    }

    fn position(&self, symbol: &ShapeSymbol) -> Option<usize> {
        self.entries.iter().position(|(held, _)| held == symbol)
    }

    fn check_declared(&self, relation: &ExtentRelation) -> Result<(), ShapeEnvError> {
        let mut undeclared = None;
        relation.for_each_symbol(|symbol| {
            if undeclared.is_none() && self.position(symbol).is_none() {
                undeclared = Some(symbol.clone());
            }
        });
        match undeclared {
            Some(symbol) => Err(ShapeEnvError::ConstraintOnUndeclaredSymbol { symbol }),
            None => Ok(()),
        }
    }
}

/// Decides one guard against the environment's semantic constraints.
///
/// The semantic set is known satisfiable by the time this runs, so any
/// contradiction reported here is attributable to the guard alone.
fn guard_verdict(
    bound: &[(ShapeSymbol, RootBinding)],
    relations: &[&ExtentRelation],
    guard: &VariantGuard,
) -> Result<(), ShapeEnvError> {
    let mut with_guard = relations.to_vec();
    with_guard.push(guard.relation());
    constraint::decide(bound, &with_guard)
}

/// A verified shape environment: every symbol declared once and bound once.
///
/// Immutable and unforgeable — private fields, no unchecked constructor, and no
/// mutable access to a draft — per the ADR 0071 lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapeEnv {
    entries: Vec<(ShapeSymbol, RootBinding)>,
    constraints: Vec<SemanticInputConstraint>,
    guards: Vec<VariantGuard>,
    identity: ShapeEnvIdentity,
}

impl ShapeEnv {
    /// Returns the environment's canonical identity.
    ///
    /// Covers exactly what the contract names: "symbol declarations,
    /// root-binding provenance, and semantic constraints". Two things are
    /// deliberately outside it. Nothing derived from the constraints is stored
    /// at all, so no solver cache can leak into identity by omission. And
    /// variant guards are excluded because they are not semantic constraints:
    /// two environments describing the same program must have the same identity
    /// whether or not a planner happened to record predicates for optimizations
    /// it was considering.
    #[must_use]
    pub const fn identity(&self) -> &ShapeEnvIdentity {
        &self.identity
    }

    /// Returns every semantic input constraint, in canonical order.
    #[must_use]
    pub fn constraints(&self) -> impl ExactSizeIterator<Item = &SemanticInputConstraint> {
        self.constraints.iter()
    }

    /// Returns only the constraints the frontend required.
    ///
    /// The contract forbids inferred or proven facts from "silently becoming
    /// additional frontend-required semantics", so the set a consumer must
    /// treat as frontend-imposed is read through provenance rather than assumed
    /// from membership in the constraint list.
    pub fn frontend_required_constraints(&self) -> impl Iterator<Item = &SemanticInputConstraint> {
        self.constraints
            .iter()
            .filter(|constraint| constraint.provenance() == FactProvenance::FrontendRequired)
    }

    /// Returns every variant guard, in canonical order.
    #[must_use]
    pub fn guards(&self) -> impl ExactSizeIterator<Item = &VariantGuard> {
        self.guards.iter()
    }

    /// Returns the guards no assignment can satisfy alongside this environment.
    ///
    /// Recomputed rather than stored. The contract excludes "derived solver
    /// caches" from canonical identity, and the simplest way to hold that is to
    /// derive nothing that could be stored: the environment retains only what
    /// was asserted.
    ///
    /// A guard listed here selects another valid plan or fallback; it does not
    /// make the program invalid, which is the distinction the contract draws
    /// between a variant guard and a semantic input constraint.
    pub fn unsatisfiable_guards(&self) -> Vec<&VariantGuard> {
        let relations: Vec<&ExtentRelation> = self
            .constraints
            .iter()
            .map(SemanticInputConstraint::relation)
            .collect();
        self.guards
            .iter()
            .filter(|guard| guard_verdict(&self.entries, &relations, guard).is_err())
            .collect()
    }

    /// Returns every symbol and its root binding, in canonical order.
    #[must_use]
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = (&ShapeSymbol, &RootBinding)> {
        self.entries
            .iter()
            .map(|(symbol, binding)| (symbol, binding))
    }

    /// Resolves one symbol's root binding.
    #[must_use]
    pub fn binding(&self, symbol: &ShapeSymbol) -> Option<&RootBinding> {
        self.entries
            .iter()
            .find(|(held, _)| held == symbol)
            .map(|(_, binding)| binding)
    }

    /// Returns the latest phase any binding in this environment requires.
    ///
    /// A consumer that can only read facts through some phase compares against
    /// this rather than walking the bindings itself.
    #[must_use]
    pub fn latest_required_phase(&self) -> Option<AvailabilityPhase> {
        self.entries
            .iter()
            .map(|(_, binding)| binding.phase())
            .max()
    }

    /// Returns the interval every model of this environment confines `symbol` to.
    ///
    /// This is the query a consumer needs to prove a bound over a symbolic
    /// extent: the interval contains every admissible value, so a fact proved
    /// against it holds for every binding the environment admits. It is not a
    /// claim that every value inside it is admissible — a divisibility
    /// constraint can exclude interior values — so it may be used to prove a
    /// bound and never to enumerate a domain.
    ///
    /// Recomputed rather than stored, like [`Self::unsatisfiable_guards`]: the
    /// contract excludes derived solver caches from canonical identity, and
    /// storing nothing derived is how this module holds that.
    ///
    /// Returns `None` for an undeclared symbol, and for a class whose bound
    /// left the extent domain, which carries nothing a consumer can prove
    /// against.
    pub fn extent_interval(&self, symbol: &ShapeSymbol) -> Option<ExtentInterval> {
        let slot = self.entries.iter().position(|(held, _)| held == symbol)?;
        let relations: Vec<&ExtentRelation> = self
            .constraints
            .iter()
            .map(SemanticInputConstraint::relation)
            .collect();
        // `build` already decided this exact set, so the solve cannot fail. It
        // is still propagated as `None` rather than unwrapped: a panic here
        // would convert a future refactor's mistake into a crash instead of a
        // consumer-visible refusal.
        constraint::solve(&self.entries, &relations)
            .ok()?
            .interval(slot)
    }

    /// Returns whether this environment proves a symbol is at least one.
    ///
    /// **Why this reads semantic constraints and never variant guards.** A
    /// symbolic divisor's positivity is a condition of the expression being
    /// *defined*: `x floordiv 0` has no meaning under any plan, and the shape
    /// contract classes a zero divisor as a typed evaluation or construction
    /// error rather than a missed optimization. `env/constraint.rs` draws the
    /// discriminator — a semantic input constraint is required for the
    /// expression to be defined and its failure is an invalid-input diagnostic,
    /// while a variant guard is required only for one optimization and its
    /// failure selects another plan. Positivity is on the first side of that
    /// line, so folding guards in here would admit an expression whose
    /// definedness rests on a predicate whose failure merely picks a different
    /// plan. [`Self::extent_interval`] and [`Self::proves_equal`] read the same
    /// set for the same reason.
    ///
    /// **Proving this is not sufficient to make the expression analyzable.** A
    /// symbolic divisor crosses the affine boundary: the constraint-prover
    /// boundary classes it as nonlinear for the Presburger lane, and ADR 0046
    /// permits a pass to "conservatively decline semi-affine maps they cannot
    /// analyze". What positivity establishes is that the expression is well
    /// defined, not that interval propagation can say anything about it.
    ///
    /// Unknown symbols are not proved, so the answer is `false` rather than an
    /// error: a caller asking about a symbol this environment never declared
    /// has not been told the divisor is positive.
    pub fn proves_positive(&self, symbol: &ShapeSymbol) -> bool {
        let Some(slot) = self.entries.iter().position(|(held, _)| held == symbol) else {
            return false;
        };
        let relations: Vec<&ExtentRelation> = self
            .constraints
            .iter()
            .map(SemanticInputConstraint::relation)
            .collect();
        // As in `extent_interval`: `build` already decided this exact set, and a
        // failure is propagated as "not proved" rather than unwrapped, so a
        // future refactor's mistake becomes a refusal instead of a crash.
        constraint::solve(&self.entries, &relations)
            .ok()
            .and_then(|mut solution| solution.interval(slot))
            .is_some_and(|interval| interval.lower >= 1)
    }

    /// Returns whether this environment forces two symbols to be equal.
    ///
    /// The question [`Self::extent_interval`] cannot answer. An interval is a
    /// fact about one symbol in isolation, so two symbols confined to the same
    /// wide interval are not thereby equal, and two symbols the environment
    /// *does* force together are not thereby confined to one point. Deciding
    /// equality needs the equality classes themselves, which is what this
    /// exposes.
    ///
    /// One-sided: `true` proves equality in every model, `false` means the
    /// environment does not prove it and never that the two differ. Recomputed
    /// rather than stored, like every other query here, so no derived solver
    /// state exists that could reach canonical identity.
    ///
    /// Returns `false` for a symbol this environment does not declare, which is
    /// the fail-closed answer: an undeclared symbol has no binding here and
    /// nothing this environment says can bear on it.
    pub fn proves_equal(&self, left: &ShapeSymbol, right: &ShapeSymbol) -> bool {
        let Some(left) = self.entries.iter().position(|(held, _)| held == left) else {
            return false;
        };
        let Some(right) = self.entries.iter().position(|(held, _)| held == right) else {
            return false;
        };
        let relations: Vec<&ExtentRelation> = self
            .constraints
            .iter()
            .map(SemanticInputConstraint::relation)
            .collect();
        // As in `extent_interval`: `build` already decided this exact set, and a
        // failure is propagated as "not proved" rather than unwrapped, so a
        // future refactor's mistake becomes a refusal instead of a crash.
        constraint::solve(&self.entries, &relations)
            .is_ok_and(|mut solution| solution.same_class(left, right))
    }
}

/// Encodes one bound environment canonically.
///
/// Domain-separated and length-prefixed per ADR 0074, over the entries and
/// constraints in the canonical order `build` established, so the bytes are a
/// function of the environment rather than of authoring order. Guards are not
/// encoded and no derived state exists to encode.
fn encode_environment(
    entries: &[(ShapeSymbol, RootBinding)],
    constraints: &[SemanticInputConstraint],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, SHAPE_ENV_DOMAIN);
    push_len(&mut bytes, entries.len());
    for (symbol, binding) in entries {
        symbol.encode(&mut bytes);
        binding.encode(&mut bytes);
    }
    push_len(&mut bytes, constraints.len());
    for constraint in constraints {
        constraint.encode(&mut bytes);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::constraint::{ExtentTerm, GuardApplicability};
    use super::{
        BindingSource, ConstraintConflict, Extent, ExtentRelation, FactProvenance,
        FragmentViolation, InterfaceParameterKey, RootBinding, SemanticInputConstraint,
        ShapeEnvBuilder, ShapeEnvError, ShapeSymbol, SymbolScope, VariantGuard,
    };
    use crate::program::abi::{AvailabilityPhase, TargetPropertyKey};

    #[test]
    fn constraint_and_guard_wrapper_constructors_remain_const() {
        const RELATION: ExtentRelation =
            ExtentRelation::equal(ExtentTerm::Constant(1), ExtentTerm::Constant(1));
        const CONSTRAINT: SemanticInputConstraint =
            SemanticInputConstraint::new(RELATION, FactProvenance::FrontendRequired);
        const GUARD: VariantGuard = VariantGuard::new(RELATION, GuardApplicability::Schedule);

        assert_eq!(CONSTRAINT.relation(), &RELATION);
        assert_eq!(GUARD.relation(), &RELATION);
    }

    fn symbol(scope: &str, name: &str) -> ShapeSymbol {
        ShapeSymbol::new(SymbolScope::new(scope).unwrap(), name).unwrap()
    }

    /// A symbolic divisor's positivity comes from semantic constraints alone.
    ///
    /// **This is the discriminator the whole query exists to hold, so it is
    /// tested from both sides.** The same relation — an extent of at least one
    /// — proves positivity when it is required and does not when it is merely
    /// guarded. A `proves_positive` that folded guards in would pass the first
    /// half of this test and fail the second, which is exactly the defect: it
    /// would admit an expression whose definedness rests on a predicate whose
    /// failure only selects another plan.
    #[test]
    fn positivity_is_proved_by_a_constraint_and_not_by_a_guard() {
        let d = symbol("region/0", "d");
        let at_least_one = ExtentRelation::interval(term("d"), 1, 64).unwrap();

        let mut required_env = ShapeEnvBuilder::new();
        required_env.declare(d.clone()).unwrap();
        required_env.bind(&d, dynamic_binding("d")).unwrap();
        required_env
            .require(required(at_least_one.clone()))
            .unwrap();
        let required_env = required_env
            .build()
            .expect("a positive extent is satisfiable");
        assert!(
            required_env.proves_positive(&d),
            "a required extent of at least one proves the divisor positive",
        );

        let mut guarded = ShapeEnvBuilder::new();
        guarded.declare(d.clone()).unwrap();
        guarded.bind(&d, dynamic_binding("d")).unwrap();
        guarded
            .guard(VariantGuard::new(at_least_one, GuardApplicability::Storage))
            .unwrap();
        let guarded = guarded.build().expect("a guard is not invalid input");
        assert!(
            !guarded.proves_positive(&d),
            "a variant guard must not establish that an expression is defined; its \
             failure selects another plan rather than rejecting the program",
        );

        // A symbol this environment never declared is not proved either. The
        // answer is `false` rather than an error: nobody told us it is positive.
        let undeclared = symbol("region/0", "elsewhere");
        assert!(!required_env.proves_positive(&undeclared));
    }

    fn static_binding(value: u64) -> RootBinding {
        RootBinding::new(
            BindingSource::Static(Extent::new(value)),
            AvailabilityPhase::CompileProfile,
            FactProvenance::StaticallyProven,
        )
        .unwrap()
    }

    /// A binding whose value no compile-time reasoning can read.
    fn dynamic_binding(key: &str) -> RootBinding {
        RootBinding::new(
            BindingSource::InterfaceParameter {
                key: InterfaceParameterKey::new(key).unwrap(),
            },
            AvailabilityPhase::LaunchPreflight,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    fn term(name: &str) -> ExtentTerm {
        ExtentTerm::Symbol(symbol("region/0", name))
    }

    fn divisor(value: u64) -> NonZeroU64 {
        NonZeroU64::new(value).unwrap()
    }

    /// A draft whose named symbols are all declared and dynamically bound.
    fn draft_over(names: &[&str]) -> ShapeEnvBuilder {
        let mut draft = ShapeEnvBuilder::new();
        for name in names {
            let declared = symbol("region/0", name);
            draft.declare(declared.clone()).unwrap();
            draft.bind(&declared, dynamic_binding(name)).unwrap();
        }
        draft
    }

    fn required(relation: ExtentRelation) -> SemanticInputConstraint {
        SemanticInputConstraint::new(relation, FactProvenance::FrontendRequired)
    }

    /// Equal spelling in two scopes is two symbols, everywhere it matters.
    ///
    /// The contract states this as a rule about the environment; asserting it
    /// only through `!=` would leave open that the two collide once bound or
    /// once encoded, which is where a collision would actually do damage.
    #[test]
    fn equal_spelling_in_different_scopes_is_never_one_symbol() {
        let left = symbol("region/0", "n");
        let right = symbol("region/1", "n");
        assert_ne!(left, right);

        let mut draft = ShapeEnvBuilder::new();
        draft.declare(left.clone()).unwrap();
        // A second declaration of the *same* spelling in another scope is not a
        // duplicate, which is the half of the rule an inequality test misses.
        draft.declare(right.clone()).unwrap();
        draft.bind(&left, static_binding(2)).unwrap();
        draft.bind(&right, static_binding(3)).unwrap();
        let env = draft.build().unwrap();

        assert_eq!(env.bindings().len(), 2);
        assert_eq!(
            env.binding(&left).unwrap().source(),
            &BindingSource::Static(Extent::new(2))
        );
        assert_eq!(
            env.binding(&right).unwrap().source(),
            &BindingSource::Static(Extent::new(3))
        );
    }

    /// One declaration and one binding per symbol; neither is last-write-wins.
    #[test]
    fn a_symbol_is_declared_once_and_bound_once() {
        let n = symbol("region/0", "n");
        let mut draft = ShapeEnvBuilder::new();
        draft.declare(n.clone()).unwrap();

        assert_eq!(
            draft.declare(n.clone()),
            Err(ShapeEnvError::DuplicateDeclaration { symbol: n.clone() })
        );

        draft.bind(&n, static_binding(4)).unwrap();
        assert_eq!(
            draft.bind(&n, static_binding(5)),
            Err(ShapeEnvError::AlreadyBound { symbol: n.clone() })
        );

        // The rejected second binding left the first intact rather than
        // partially applying.
        let env = draft.build().unwrap();
        assert_eq!(
            env.binding(&n).unwrap().source(),
            &BindingSource::Static(Extent::new(4))
        );

        let undeclared = symbol("region/9", "m");
        let mut other = ShapeEnvBuilder::new();
        assert_eq!(
            other.bind(&undeclared, static_binding(1)),
            Err(ShapeEnvError::UndeclaredSymbol { symbol: undeclared })
        );
    }

    /// A free symbol fails construction rather than deferring.
    #[test]
    fn an_unbound_symbol_rejects_the_environment() {
        let n = symbol("region/0", "n");
        let mut draft = ShapeEnvBuilder::new();
        draft.declare(n.clone()).unwrap();
        assert_eq!(draft.build(), Err(ShapeEnvError::FreeSymbol { symbol: n }));
    }

    /// A binding cannot claim to be readable before its source exists.
    #[test]
    fn a_binding_may_not_precede_its_source_class() {
        let device = BindingSource::TargetProperty {
            key: TargetPropertyKey::new("tiler.target.max-threads@1").unwrap(),
        };
        assert_eq!(
            RootBinding::new(
                device.clone(),
                AvailabilityPhase::CompileProfile,
                FactProvenance::StaticallyProven,
            ),
            Err(ShapeEnvError::PhaseTooEarly {
                declared: AvailabilityPhase::CompileProfile,
                earliest: AvailabilityPhase::LiveDevicePreflight,
            })
        );

        // A later phase than the floor is admitted: a property can be deferred
        // past the earliest point it could have been read.
        RootBinding::new(
            device,
            AvailabilityPhase::LaunchPreflight,
            FactProvenance::RuntimeValidated,
        )
        .unwrap();
    }

    /// Identity is a function of the environment, not of authoring order.
    #[test]
    fn identity_is_canonical_and_separates_distinguishable_environments() {
        let a = symbol("region/0", "a");
        let b = symbol("region/0", "b");

        let build = |reversed: bool| {
            let mut draft = ShapeEnvBuilder::new();
            let order = if reversed {
                vec![(b.clone(), 7_u64), (a.clone(), 5)]
            } else {
                vec![(a.clone(), 5), (b.clone(), 7)]
            };
            for (symbol, value) in order {
                draft.declare(symbol.clone()).unwrap();
                draft.bind(&symbol, static_binding(value)).unwrap();
            }
            draft.build().unwrap()
        };

        assert_eq!(
            build(false).identity(),
            build(true).identity(),
            "declaration order is not part of the environment",
        );

        // Each field that distinguishes two environments must distinguish their
        // identities, or the identity would claim two subjects are one.
        let base = build(false);
        let mut differing_value = ShapeEnvBuilder::new();
        differing_value.declare(a.clone()).unwrap();
        differing_value.bind(&a, static_binding(5)).unwrap();
        differing_value.declare(b.clone()).unwrap();
        differing_value.bind(&b, static_binding(8)).unwrap();
        assert_ne!(differing_value.build().unwrap().identity(), base.identity());

        let mut differing_provenance = ShapeEnvBuilder::new();
        differing_provenance.declare(a.clone()).unwrap();
        differing_provenance.bind(&a, static_binding(5)).unwrap();
        differing_provenance.declare(b.clone()).unwrap();
        differing_provenance
            .bind(
                &b,
                RootBinding::new(
                    BindingSource::Static(Extent::new(7)),
                    AvailabilityPhase::CompileProfile,
                    // Same value, different reason for believing it.
                    FactProvenance::FrontendRequired,
                )
                .unwrap(),
            )
            .unwrap();
        assert_ne!(
            differing_provenance.build().unwrap().identity(),
            base.identity(),
            "provenance is part of identity, per the contract",
        );

        assert!(
            base.identity()
                .as_bytes()
                .starts_with(&(super::SHAPE_ENV_DOMAIN.len() as u64).to_be_bytes()),
            "the encoding is domain-separated and length-framed",
        );
    }

    /// The environment reports the latest phase any of its bindings needs.
    #[test]
    fn the_latest_required_phase_is_the_maximum_over_bindings() {
        let mut draft = ShapeEnvBuilder::new();
        assert_eq!(
            ShapeEnvBuilder::new()
                .build()
                .unwrap()
                .latest_required_phase(),
            None
        );

        let n = symbol("region/0", "n");
        let m = symbol("region/0", "m");
        draft.declare(n.clone()).unwrap();
        draft.bind(&n, static_binding(2)).unwrap();
        draft.declare(m.clone()).unwrap();
        draft
            .bind(
                &m,
                RootBinding::new(
                    BindingSource::InterfaceParameter {
                        key: InterfaceParameterKey::new("batch").unwrap(),
                    },
                    AvailabilityPhase::LaunchPreflight,
                    FactProvenance::RuntimeValidated,
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(
            draft.build().unwrap().latest_required_phase(),
            Some(AvailabilityPhase::LaunchPreflight),
        );
    }

    /// "Contradictory semantic constraints reject the graph" — at `build`.
    ///
    /// The timing is the substance of the clause: asserting only that some
    /// checked step reports the contradiction would leave a verified `ShapeEnv`
    /// in existence that the contract calls invalid. `build` is the only
    /// constructor, so a returned environment is never contradictory.
    #[test]
    fn contradictory_semantic_constraints_reject_the_environment_at_build() {
        let mut draft = draft_over(&["n"]);
        draft
            .require(required(ExtentRelation::equal(
                term("n"),
                ExtentTerm::Constant(4),
            )))
            .unwrap();
        draft
            .require(required(ExtentRelation::equal(
                term("n"),
                ExtentTerm::Constant(5),
            )))
            .unwrap();
        assert_eq!(
            draft.build(),
            Err(ShapeEnvError::ContradictoryConstraints {
                conflict: Box::new(ConstraintConflict::ConflictingConstants {
                    symbol: symbol("region/0", "n"),
                    first: 4,
                    second: 5,
                }),
            })
        );

        // A divisibility and an interval that no integer satisfies together is
        // the same rejection, reached by the solver rather than by two literals
        // disagreeing.
        let mut narrowed = draft_over(&["n"]);
        narrowed
            .require(required(ExtentRelation::divisible(term("n"), divisor(4))))
            .unwrap();
        narrowed
            .require(required(ExtentRelation::interval(term("n"), 3, 3).unwrap()))
            .unwrap();
        assert!(matches!(
            narrowed.build(),
            Err(ShapeEnvError::ContradictoryConstraints { .. })
        ));

        // The satisfiable neighbour is admitted, so the rejection above is a
        // decision about the constraints rather than a refusal of the kind.
        let mut widened = draft_over(&["n"]);
        widened
            .require(required(ExtentRelation::divisible(term("n"), divisor(4))))
            .unwrap();
        widened
            .require(required(ExtentRelation::interval(term("n"), 3, 8).unwrap()))
            .unwrap();
        widened.build().unwrap();
    }

    /// A root binding is part of the environment the constraints are decided against.
    ///
    /// The contract puts declarations, bindings, and constraints in one
    /// `ShapeEnv`; deciding the constraints in isolation would admit a set that
    /// contradicts a statically known extent.
    #[test]
    fn a_statically_bound_extent_participates_in_the_decision() {
        let n = symbol("region/0", "n");

        let mut contradictory = ShapeEnvBuilder::new();
        contradictory.declare(n.clone()).unwrap();
        contradictory.bind(&n, static_binding(10)).unwrap();
        contradictory
            .require(required(ExtentRelation::divisible(term("n"), divisor(4))))
            .unwrap();
        assert!(matches!(
            contradictory.build(),
            Err(ShapeEnvError::ContradictoryConstraints { .. })
        ));

        let mut consistent = ShapeEnvBuilder::new();
        consistent.declare(n.clone()).unwrap();
        consistent.bind(&n, static_binding(12)).unwrap();
        consistent
            .require(required(ExtentRelation::divisible(term("n"), divisor(4))))
            .unwrap();
        consistent.build().unwrap();
    }

    /// One decode step must not accept stale state whose observed result extent
    /// disagrees with its context and token extents.
    #[test]
    fn a_decode_shaped_additive_mismatch_refuses_and_names_all_three_terms() {
        let build = |sum: u64| {
            let mut draft = ShapeEnvBuilder::new();
            for (name, value) in [("S", sum), ("C", 14), ("T", 1)] {
                let declared = symbol("decode/layer-0", name);
                draft.declare(declared.clone()).unwrap();
                draft.bind(&declared, static_binding(value)).unwrap();
            }
            draft
                .require(required(ExtentRelation::additive_equality(
                    ExtentTerm::Symbol(symbol("decode/layer-0", "S")),
                    ExtentTerm::Symbol(symbol("decode/layer-0", "C")),
                    ExtentTerm::Symbol(symbol("decode/layer-0", "T")),
                )))
                .unwrap();
            draft.build()
        };

        let error =
            build(13).expect_err("state valid over [0, 13) cannot verify when C = 14 and T = 1");
        let ShapeEnvError::ContradictoryConstraints { conflict } = &error else {
            panic!("the mismatch is a typed contradictory constraint, not {error}");
        };
        assert_eq!(
            **conflict,
            ConstraintConflict::AdditiveEqualityMismatch {
                relation: ExtentRelation::additive_equality(
                    ExtentTerm::Symbol(symbol("decode/layer-0", "S")),
                    ExtentTerm::Symbol(symbol("decode/layer-0", "C")),
                    ExtentTerm::Symbol(symbol("decode/layer-0", "T")),
                ),
                sum: 13,
                addends: 15,
            }
        );
        let diagnostic = error.to_string();
        for term in ["S", "C", "T"] {
            assert!(
                diagnostic.contains(term),
                "the three-term diagnostic must name {term}: {diagnostic}"
            );
        }

        build(15).expect("S = 15 is consistent with C = 14 and T = 1");
    }

    /// A partially observed equality reports why the remaining extent cannot
    /// inhabit the nonnegative extent domain; it is not a fully observed mismatch.
    #[test]
    fn an_observed_addend_exceeding_the_sum_names_the_negative_remainder() {
        let build = |sum: u64| {
            let mut draft = ShapeEnvBuilder::new();
            for (name, binding) in [
                ("S", static_binding(sum)),
                ("known", static_binding(3)),
                ("remaining", dynamic_binding("remaining")),
            ] {
                let declared = symbol("partial", name);
                draft.declare(declared.clone()).unwrap();
                draft.bind(&declared, binding).unwrap();
            }
            draft
                .require(required(ExtentRelation::additive_equality(
                    ExtentTerm::Symbol(symbol("partial", "S")),
                    ExtentTerm::Symbol(symbol("partial", "remaining")),
                    ExtentTerm::Symbol(symbol("partial", "known")),
                )))
                .unwrap();
            draft.build()
        };

        let error = build(2).expect_err("2 == remaining + 3 needs a negative extent");
        let ShapeEnvError::ContradictoryConstraints { conflict } = &error else {
            panic!("the impossible remainder is a typed contradiction, not {error}");
        };
        assert_eq!(
            **conflict,
            ConstraintConflict::AddendExceedsSum {
                relation: ExtentRelation::additive_equality(
                    ExtentTerm::Symbol(symbol("partial", "S")),
                    ExtentTerm::Symbol(symbol("partial", "remaining")),
                    ExtentTerm::Symbol(symbol("partial", "known")),
                ),
                sum: 2,
                addend: 3,
                remaining: ExtentTerm::Symbol(symbol("partial", "remaining")),
            }
        );
        let diagnostic = error.to_string();
        for fact in ["sum 2", "addend 3", "remaining", "negative"] {
            assert!(
                diagnostic.contains(fact),
                "the partial-observation diagnostic must contain {fact:?}: {diagnostic}"
            );
        }

        build(3).expect("3 == remaining + 3 has the nonnegative solution remaining = 0");
    }

    /// Runtime-bound extents retain an additive requirement for preflight.
    #[test]
    fn a_runtime_bound_additive_relation_has_an_exhibited_model() {
        let mut draft = draft_over(&["S", "C", "T", "capacity"]);
        draft
            .require(required(ExtentRelation::additive_equality(
                term("S"),
                term("C"),
                term("T"),
            )))
            .unwrap();
        draft
            .require(required(ExtentRelation::non_negative_difference(
                term("capacity"),
                term("S"),
            )))
            .unwrap();
        let environment = draft
            .build()
            .expect("the all-zero model proves the runtime-bound set satisfiable");
        assert_eq!(environment.constraints().len(), 2);
    }

    /// A relation outside the supported fragment is refused, never under-decided.
    ///
    /// "The solver algorithm and exact supported arithmetic fragment remain
    /// implementation choices" — but a chosen fragment only means anything if
    /// what lies outside it fails loudly. A nonlinear factorization admitted and
    /// reported as "no contradiction found" would be indistinguishable, to every
    /// caller, from a decided satisfiable.
    #[test]
    fn a_relation_outside_the_supported_fragment_is_refused_rather_than_ignored() {
        let mut nonlinear = draft_over(&["n", "a", "b"]);
        nonlinear
            .require(required(
                ExtentRelation::factorization(term("n"), vec![term("a"), term("b")]).unwrap(),
            ))
            .unwrap();
        assert_eq!(
            nonlinear.build(),
            Err(ShapeEnvError::UnsupportedRelation {
                relation: Box::new(
                    ExtentRelation::factorization(term("n"), vec![term("a"), term("b")]).unwrap()
                ),
                violation: FragmentViolation::UnderdeterminedFactorization { undetermined: 3 },
            })
        );

        // An undecidable *guard* is refused for the same reason. Its failure
        // would select another plan, but its undecidability leaves the variant's
        // selectability unknown, which is not the same thing.
        let mut guarded = draft_over(&["n", "a", "b"]);
        guarded
            .guard(VariantGuard::new(
                ExtentRelation::factorization(term("n"), vec![term("a"), term("b")]).unwrap(),
                GuardApplicability::Schedule,
            ))
            .unwrap();
        assert!(matches!(
            guarded.build(),
            Err(ShapeEnvError::UnsupportedRelation { .. })
        ));

        // An underdetermined additive equality is admitted only when the
        // canonical lower-bound model exhibits a solution. `C >= 1` makes that
        // model `(S, C, T) = (0, 1, 0)`, so this relation is conservatively
        // refused rather than being admitted on "no contradiction found".
        let mut additive = draft_over(&["S", "C", "T"]);
        additive
            .require(required(ExtentRelation::additive_equality(
                term("S"),
                term("C"),
                term("T"),
            )))
            .unwrap();
        additive
            .require(required(
                ExtentRelation::interval(term("C"), 1, 64).unwrap(),
            ))
            .unwrap();
        assert!(matches!(
            additive.build(),
            Err(ShapeEnvError::UnsupportedRelation {
                violation: FragmentViolation::UnderdeterminedAdditiveEquality { undetermined: 3 },
                ..
            })
        ));

        // In-fragment, a factorization with one undetermined term is solved
        // rather than merely stored: `128 == 8 * outer` forces `outer == 16`,
        // which the interval then contradicts.
        let n = symbol("region/0", "n");
        let outer = symbol("region/0", "outer");
        let mut solved = ShapeEnvBuilder::new();
        solved.declare(n.clone()).unwrap();
        solved.bind(&n, static_binding(128)).unwrap();
        solved.declare(outer.clone()).unwrap();
        solved.bind(&outer, dynamic_binding("outer")).unwrap();
        solved
            .require(required(
                ExtentRelation::factorization(
                    term("n"),
                    vec![ExtentTerm::Constant(8), term("outer")],
                )
                .unwrap(),
            ))
            .unwrap();
        solved
            .require(required(
                ExtentRelation::interval(term("outer"), 0, 10).unwrap(),
            ))
            .unwrap();
        assert!(matches!(
            solved.build(),
            Err(ShapeEnvError::ContradictoryConstraints { .. })
        ));
    }

    /// The relation kinds are decided together, not checked one at a time.
    ///
    /// The contradiction here belongs to no single relation: each is
    /// individually satisfiable, and only the comparison chain carrying the
    /// pinned lower bound into a divisibility-tightened interval refutes them.
    #[test]
    fn the_decision_covers_the_whole_constraint_set() {
        let build = |ceiling: u64| {
            let mut draft = draft_over(&["a", "b", "c"]);
            draft
                .require(required(ExtentRelation::non_negative_difference(
                    term("a"),
                    term("b"),
                )))
                .unwrap();
            draft
                .require(required(ExtentRelation::non_negative_difference(
                    term("b"),
                    term("c"),
                )))
                .unwrap();
            draft
                .require(required(ExtentRelation::equal(
                    term("c"),
                    ExtentTerm::Constant(10),
                )))
                .unwrap();
            draft
                .require(required(
                    ExtentRelation::interval(term("a"), 0, ceiling).unwrap(),
                ))
                .unwrap();
            draft
                .require(required(ExtentRelation::divisible(term("a"), divisor(4))))
                .unwrap();
            draft.build()
        };

        // `a >= b >= c == 10` and `4 | a` force `a >= 12`.
        assert!(matches!(
            build(11),
            Err(ShapeEnvError::ContradictoryConstraints { .. })
        ));
        build(12).unwrap();
    }

    /// A cycle of nonnegative differences is an equality and is decided as one.
    ///
    /// `a >= b` and `b >= a` force `a == b`, so the two symbols' divisibility
    /// facts must meet. Treating the comparisons as independent bounds would
    /// admit a set with no solution.
    #[test]
    fn a_comparison_cycle_meets_the_facts_of_both_symbols() {
        let build = |ceiling: u64| {
            let mut draft = draft_over(&["a", "b"]);
            draft
                .require(required(ExtentRelation::non_negative_difference(
                    term("a"),
                    term("b"),
                )))
                .unwrap();
            draft
                .require(required(ExtentRelation::non_negative_difference(
                    term("b"),
                    term("a"),
                )))
                .unwrap();
            draft
                .require(required(ExtentRelation::divisible(term("a"), divisor(2))))
                .unwrap();
            draft
                .require(required(ExtentRelation::divisible(term("b"), divisor(3))))
                .unwrap();
            draft
                .require(required(
                    ExtentRelation::interval(term("a"), 1, ceiling).unwrap(),
                ))
                .unwrap();
            draft.build()
        };

        // The merged class must be a multiple of six, and none lies in [1, 5].
        assert!(matches!(
            build(5),
            Err(ShapeEnvError::ContradictoryConstraints { .. })
        ));
        build(7).unwrap();
    }

    /// A semantic input constraint and a variant guard are not interchangeable.
    ///
    /// The contract says so explicitly and says why: failure of the first is an
    /// invalid-input diagnostic, failure of the second selects another valid
    /// plan. One unsatisfiable relation, recorded each way, must therefore
    /// produce two different outcomes.
    #[test]
    fn a_failing_constraint_rejects_where_a_failing_guard_selects_another_plan() {
        let n = symbol("region/0", "n");
        let unsatisfiable = ExtentRelation::divisible(term("n"), divisor(16));

        let mut as_constraint = ShapeEnvBuilder::new();
        as_constraint.declare(n.clone()).unwrap();
        as_constraint.bind(&n, static_binding(24)).unwrap();
        as_constraint
            .require(required(unsatisfiable.clone()))
            .unwrap();
        assert!(matches!(
            as_constraint.build(),
            Err(ShapeEnvError::ContradictoryConstraints { .. })
        ));

        let mut as_guard = ShapeEnvBuilder::new();
        as_guard.declare(n.clone()).unwrap();
        as_guard.bind(&n, static_binding(24)).unwrap();
        as_guard
            .guard(VariantGuard::new(
                unsatisfiable,
                GuardApplicability::Storage,
            ))
            .unwrap();
        // A satisfiable guard alongside it, so the report distinguishes rather
        // than condemning every guard once one fails.
        as_guard
            .guard(VariantGuard::new(
                ExtentRelation::divisible(term("n"), divisor(8)),
                GuardApplicability::DispatchSafety,
            ))
            .unwrap();

        let env = as_guard
            .build()
            .expect("a failing guard is not invalid input");
        let unsatisfiable = env.unsatisfiable_guards();
        assert_eq!(unsatisfiable.len(), 1);
        assert_eq!(
            unsatisfiable[0].applicability(),
            GuardApplicability::Storage
        );
        assert_eq!(env.guards().len(), 2);
    }

    /// "Inferred or proven facts may not silently become additional
    /// frontend-required semantics."
    #[test]
    fn a_proven_fact_never_becomes_a_frontend_required_one() {
        let relation = ExtentRelation::divisible(term("n"), divisor(4));

        let mut proven_only = draft_over(&["n"]);
        proven_only
            .require(SemanticInputConstraint::new(
                relation.clone(),
                FactProvenance::StaticallyProven,
            ))
            .unwrap();
        let env = proven_only.build().unwrap();
        assert_eq!(env.constraints().len(), 1);
        assert_eq!(
            env.constraints().next().unwrap().provenance(),
            FactProvenance::StaticallyProven,
        );
        assert_eq!(env.frontend_required_constraints().count(), 0);

        // The same relation asserted for two different reasons stays two
        // constraints. Collapsing them would decide which reason survived, and
        // whichever way it fell would rewrite one fact's provenance.
        let mut both = draft_over(&["n"]);
        both.require(SemanticInputConstraint::new(
            relation.clone(),
            FactProvenance::StaticallyProven,
        ))
        .unwrap();
        both.require(SemanticInputConstraint::new(
            relation.clone(),
            FactProvenance::FrontendRequired,
        ))
        .unwrap();
        let env = both.build().unwrap();
        assert_eq!(env.constraints().len(), 2);
        assert_eq!(env.frontend_required_constraints().count(), 1);

        // An exact repeat is canonicalization, not a merge: nothing is lost.
        let mut repeated = draft_over(&["n"]);
        repeated
            .require(SemanticInputConstraint::new(
                relation.clone(),
                FactProvenance::StaticallyProven,
            ))
            .unwrap();
        repeated
            .require(SemanticInputConstraint::new(
                relation,
                FactProvenance::StaticallyProven,
            ))
            .unwrap();
        assert_eq!(repeated.build().unwrap().constraints().len(), 1);
    }

    /// "Canonical identity includes symbol declarations, root-binding
    /// provenance, and semantic constraints but excludes derived solver caches."
    #[test]
    fn semantic_constraints_are_identity_and_guards_are_not() {
        let interval = ExtentRelation::interval(term("n"), 1, 64).unwrap();
        let divisible = ExtentRelation::divisible(term("n"), divisor(4));

        let build = |reversed: bool, guarded: bool| {
            let mut draft = draft_over(&["n"]);
            let order = if reversed {
                vec![divisible.clone(), interval.clone()]
            } else {
                vec![interval.clone(), divisible.clone()]
            };
            for relation in order {
                draft.require(required(relation)).unwrap();
            }
            if guarded {
                draft
                    .guard(VariantGuard::new(
                        ExtentRelation::divisible(term("n"), divisor(16)),
                        GuardApplicability::TargetCompatibility,
                    ))
                    .unwrap();
            }
            draft.build().unwrap()
        };

        let base = build(false, false);
        assert_eq!(
            base.identity(),
            build(true, false).identity(),
            "the order constraints were authored in is not part of the environment",
        );
        assert_eq!(
            base.identity(),
            build(false, true).identity(),
            "a variant guard is not a semantic constraint and does not name the program",
        );

        // Every field that distinguishes two constraint sets must distinguish
        // their identities, or the identity would claim two subjects are one.
        let mut fewer = draft_over(&["n"]);
        fewer.require(required(interval.clone())).unwrap();
        assert_ne!(fewer.build().unwrap().identity(), base.identity());

        let mut reasoned_differently = draft_over(&["n"]);
        reasoned_differently.require(required(interval)).unwrap();
        reasoned_differently
            .require(SemanticInputConstraint::new(
                divisible,
                FactProvenance::StaticallyProven,
            ))
            .unwrap();
        assert_ne!(
            reasoned_differently.build().unwrap().identity(),
            base.identity(),
            "provenance is part of identity, per the contract",
        );

        let mut additive = draft_over(&["n", "left", "right"]);
        additive
            .require(required(ExtentRelation::additive_equality(
                term("n"),
                term("left"),
                term("right"),
            )))
            .unwrap();
        let additive = additive.build().unwrap();
        let mut reversed = draft_over(&["n", "left", "right"]);
        reversed
            .require(required(ExtentRelation::AdditiveEquality {
                sum: term("n"),
                left: term("right"),
                right: term("left"),
            }))
            .unwrap();
        reversed
            .require(required(ExtentRelation::additive_equality(
                term("n"),
                term("left"),
                term("right"),
            )))
            .unwrap();
        let reversed = reversed.build().unwrap();
        let empty = draft_over(&["n", "left", "right"]).build().unwrap();
        assert_ne!(
            additive.identity(),
            empty.identity(),
            "the fresh relation tag and all three terms enter identity",
        );
        assert_eq!(
            additive.identity(),
            reversed.identity(),
            "direct and helper addend spellings are canonicalized before identity",
        );
        assert_eq!(
            reversed.constraints().len(),
            1,
            "canonicalization precedes constraint sorting and deduplication",
        );

        let mut guarded = draft_over(&["n", "left", "right"]);
        guarded
            .guard(VariantGuard::new(
                ExtentRelation::AdditiveEquality {
                    sum: term("n"),
                    left: term("right"),
                    right: term("left"),
                },
                GuardApplicability::Schedule,
            ))
            .unwrap();
        guarded
            .guard(VariantGuard::new(
                ExtentRelation::additive_equality(term("n"), term("left"), term("right")),
                GuardApplicability::Schedule,
            ))
            .unwrap();
        let guarded = guarded.build().unwrap();
        assert_eq!(
            guarded.guards().len(),
            1,
            "guard canonicalization precedes sorting and deduplication",
        );
        assert_eq!(
            guarded.guards().next().unwrap().relation(),
            &ExtentRelation::additive_equality(term("n"), term("left"), term("right")),
            "the stored guard uses the same canonical relation as a constraint",
        );
    }

    /// A relation over an undeclared symbol is refused where it is written.
    ///
    /// The contract makes free symbols invalid; a constraint naming one would
    /// otherwise reach the solver as a symbol with no binding and no domain.
    #[test]
    fn a_relation_over_an_undeclared_symbol_is_refused_at_insertion() {
        let mut draft = draft_over(&["n"]);
        let stray = ExtentRelation::equal(term("n"), term("m"));
        assert_eq!(
            draft.require(required(stray.clone())),
            Err(ShapeEnvError::ConstraintOnUndeclaredSymbol {
                symbol: symbol("region/0", "m"),
            })
        );
        assert_eq!(
            draft.guard(VariantGuard::new(stray, GuardApplicability::Schedule)),
            Err(ShapeEnvError::ConstraintOnUndeclaredSymbol {
                symbol: symbol("region/0", "m"),
            })
        );

        // Both rejections left the draft unchanged rather than partly applying.
        let env = draft.build().unwrap();
        assert_eq!(env.constraints().len(), 0);
        assert_eq!(env.guards().len(), 0);
    }

    /// A malformed relation is refused where it is written, not carried to the solver.
    #[test]
    fn an_unwritable_relation_is_refused_at_construction() {
        assert_eq!(
            ExtentRelation::interval(term("n"), 5, 4),
            Err(ShapeEnvError::EmptyInterval { lower: 5, upper: 4 })
        );
        assert_eq!(
            ExtentRelation::factorization(term("n"), vec![ExtentTerm::Constant(8)]),
            Err(ShapeEnvError::DegenerateFactorization { factors: 1 })
        );
    }
}
