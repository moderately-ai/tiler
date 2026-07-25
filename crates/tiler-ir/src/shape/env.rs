#![allow(
    dead_code,
    reason = "ADR 0074 convention 7 draft: this is the shape-symbol half of the ShapeEnv authority, and its consumers do not exist yet. `implement-shapeenv-index-bindings` is the ticket that makes index lowering read it, and until then nothing on the compile path can construct one — the bounded profile's shapes are static literals with no symbols at all. Wiring a premature consumer to satisfy the lint would make the authority look adopted while proving nothing"
)]

//! The scoped shape-symbol authority: declarations, typed root bindings, identity.
//!
//! # Draft status
//!
//! This module is `pub(crate)` under ADR 0074 convention 7. `implement-shapeenv-core`
//! states that "any consequential public or cross-crate boundary remains a draft
//! until Tom reviews and accepts the exact implementation commit", so nothing
//! here is reachable outside `tiler-ir` yet and promoting it is a separate
//! reviewed step rather than a consequence of it compiling.
//!
//! # What this half owns
//!
//! `docs/ir.md`'s constraint-and-proof-context section specifies a `ShapeEnv`
//! as scoped symbol declarations, source bindings, *and* a constraint
//! environment over extent equalities, divisibility, nonnegativity, intervals,
//! and factorization. This module implements the first two. The constraint
//! environment is `implement-shapeenv-constraints`, split out rather than
//! stubbed, because a constraint set without a contradiction check would be a
//! type-system reservation wearing the name of an implemented authority — and
//! the contract's rule that "contradictory semantic constraints reject the
//! graph" is the substance of that half, not a later refinement of it.
//!
//! # The invariants this half does establish
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

use core::fmt;
use std::error::Error;

use crate::identity::{push_len, push_slice};
use crate::program::abi::AvailabilityPhase;

/// Domain separator of a canonical shape-environment encoding.
const SHAPE_ENV_DOMAIN: &[u8] = b"tiler.shape-env.v1\0";

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
pub(crate) struct SymbolScope(Vec<u8>);

impl SymbolScope {
    /// Creates a scope from opaque bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::EmptyScope`] for an empty scope, which would
    /// otherwise be indistinguishable from an absent one.
    pub(crate) fn new(bytes: impl AsRef<[u8]>) -> Result<Self, ShapeEnvError> {
        let bytes = bytes.as_ref();
        if bytes.is_empty() {
            return Err(ShapeEnvError::EmptyScope);
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Returns the scope's opaque bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One scoped extent symbol.
///
/// Name and scope together are the symbol. Neither is meaningful alone: the
/// contract's rule that equal spelling in different scopes never implies
/// equality is enforced by this pairing rather than by callers remembering it.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ShapeSymbol {
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
    pub(crate) fn new(scope: SymbolScope, name: impl Into<String>) -> Result<Self, ShapeEnvError> {
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
    pub(crate) const fn scope(&self) -> &SymbolScope {
        &self.scope
    }

    /// Returns the symbol's name within its scope.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        push_slice(bytes, self.scope.as_bytes());
        push_slice(bytes, self.name.as_bytes());
    }
}

impl fmt::Display for ShapeSymbol {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.scope.0.escape_ascii(), self.name)
    }
}

/// Where a root binding's value comes from.
///
/// `docs/ir.md` names the admitted sources: "static values, input metadata,
/// caller parameters, or admitted versioned target properties". The class is
/// recorded rather than inferred from the value, because two bindings can carry
/// the same number for different reasons and only one of them may be legal for
/// a given consumer.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum BindingSource {
    /// A value fixed at graph construction.
    StaticValue(u64),
    /// An extent read from a bound program input's metadata.
    InputMetadata {
        /// Interface key of the input the extent is read from.
        input: String,
        /// Axis position within that input's shape.
        axis: u32,
    },
    /// A value supplied by the caller at compilation or launch.
    CallerParameter {
        /// Stable parameter key.
        key: String,
    },
    /// An admitted versioned target property.
    ///
    /// The contract requires that these "use stable versioned keys and cannot
    /// depend on a selected or prepared physical pipeline in the initial
    /// execution model", which is why the version travels with the key.
    TargetProperty {
        /// Stable governed property key.
        key: String,
        /// Version of the property's definition.
        version: u32,
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
            Self::StaticValue(_) => 0x01,
            Self::InputMetadata { .. } => 0x02,
            Self::CallerParameter { .. } => 0x03,
            Self::TargetProperty { .. } => 0x04,
        }
    }

    /// Returns the earliest phase this source can be read at.
    ///
    /// A static value is known from the compile profile; every other class
    /// depends on something the compiler does not have until later. This is the
    /// *floor*: a binding may declare a later phase than its class requires,
    /// which [`RootBinding::new`] checks, but never an earlier one.
    const fn earliest_phase(&self) -> AvailabilityPhase {
        match self {
            Self::StaticValue(_) => AvailabilityPhase::CompileProfile,
            // An interface extent and a caller parameter are both known once a
            // concrete invocation exists, which is no earlier than a bound
            // device and context.
            Self::InputMetadata { .. } | Self::CallerParameter { .. } => {
                AvailabilityPhase::LiveDevicePreflight
            }
            Self::TargetProperty { .. } => AvailabilityPhase::LiveDevicePreflight,
        }
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        match self {
            Self::StaticValue(value) => bytes.extend_from_slice(&value.to_be_bytes()),
            Self::InputMetadata { input, axis } => {
                push_slice(bytes, input.as_bytes());
                bytes.extend_from_slice(&axis.to_be_bytes());
            }
            Self::CallerParameter { key } => push_slice(bytes, key.as_bytes()),
            Self::TargetProperty { key, version } => {
                push_slice(bytes, key.as_bytes());
                bytes.extend_from_slice(&version.to_be_bytes());
            }
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
pub(crate) enum FactProvenance {
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
pub(crate) struct RootBinding {
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
    pub(crate) fn new(
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
    pub(crate) const fn source(&self) -> &BindingSource {
        &self.source
    }

    /// Returns the phase at which this binding becomes readable.
    pub(crate) const fn phase(&self) -> AvailabilityPhase {
        self.phase
    }

    /// Returns how this binding was established.
    pub(crate) const fn provenance(&self) -> FactProvenance {
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
pub(crate) enum ShapeEnvError {
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
        }
    }
}

impl Error for ShapeEnvError {}

/// The canonical identity of one verified shape environment.
///
/// Opaque bytes with no public constructor, per ADR 0074: only the encoder that
/// establishes what the identity means produces one.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ShapeEnvIdentity(Vec<u8>);

impl ShapeEnvIdentity {
    /// Returns the identity's canonical bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An append-only draft shape environment.
///
/// Declarations and bindings are separate steps because the contract separates
/// them: a symbol exists once declared, and remains invalid until bound.
#[derive(Clone, Debug, Default)]
pub(crate) struct ShapeEnvBuilder {
    entries: Vec<(ShapeSymbol, Option<RootBinding>)>,
}

impl ShapeEnvBuilder {
    /// Opens an empty draft.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Declares one scoped symbol.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::DuplicateDeclaration`] when the symbol is
    /// already declared. The check leaves the draft unchanged.
    pub(crate) fn declare(&mut self, symbol: ShapeSymbol) -> Result<(), ShapeEnvError> {
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
    pub(crate) fn bind(
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

    /// Verifies the draft and freezes it.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::FreeSymbol`] naming the first declared symbol
    /// with no root binding, in canonical order so the diagnostic does not
    /// depend on declaration order.
    pub(crate) fn build(self) -> Result<ShapeEnv, ShapeEnvError> {
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

        let identity = ShapeEnvIdentity(encode_environment(&bound));
        Ok(ShapeEnv {
            entries: bound,
            identity,
        })
    }

    fn position(&self, symbol: &ShapeSymbol) -> Option<usize> {
        self.entries.iter().position(|(held, _)| held == symbol)
    }
}

/// A verified shape environment: every symbol declared once and bound once.
///
/// Immutable and unforgeable — private fields, no unchecked constructor, and no
/// mutable access to a draft — per the ADR 0071 lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShapeEnv {
    entries: Vec<(ShapeSymbol, RootBinding)>,
    identity: ShapeEnvIdentity,
}

impl ShapeEnv {
    /// Returns the environment's canonical identity.
    ///
    /// Derived from symbol declarations and root-binding provenance alone. The
    /// contract excludes "derived solver caches" from identity, and this half
    /// holds none — the constraint environment that would is
    /// `implement-shapeenv-constraints`, which must fold its constraints in
    /// here without folding anything derived from them.
    pub(crate) const fn identity(&self) -> &ShapeEnvIdentity {
        &self.identity
    }

    /// Returns every symbol and its root binding, in canonical order.
    pub(crate) fn bindings(&self) -> impl ExactSizeIterator<Item = (&ShapeSymbol, &RootBinding)> {
        self.entries
            .iter()
            .map(|(symbol, binding)| (symbol, binding))
    }

    /// Resolves one symbol's root binding.
    pub(crate) fn binding(&self, symbol: &ShapeSymbol) -> Option<&RootBinding> {
        self.entries
            .iter()
            .find(|(held, _)| held == symbol)
            .map(|(_, binding)| binding)
    }

    /// Returns the latest phase any binding in this environment requires.
    ///
    /// A consumer that can only read facts through some phase compares against
    /// this rather than walking the bindings itself.
    pub(crate) fn latest_required_phase(&self) -> Option<AvailabilityPhase> {
        self.entries
            .iter()
            .map(|(_, binding)| binding.phase())
            .max()
    }
}

/// Encodes one bound environment canonically.
///
/// Domain-separated and length-prefixed per ADR 0074, over the entries in the
/// canonical order `build` established, so the bytes are a function of the
/// environment rather than of authoring order.
fn encode_environment(entries: &[(ShapeSymbol, RootBinding)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, SHAPE_ENV_DOMAIN);
    push_len(&mut bytes, entries.len());
    for (symbol, binding) in entries {
        symbol.encode(&mut bytes);
        binding.encode(&mut bytes);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::{
        BindingSource, FactProvenance, RootBinding, ShapeEnvBuilder, ShapeEnvError, ShapeSymbol,
        SymbolScope,
    };
    use crate::program::abi::AvailabilityPhase;

    fn symbol(scope: &str, name: &str) -> ShapeSymbol {
        ShapeSymbol::new(SymbolScope::new(scope).unwrap(), name).unwrap()
    }

    fn static_binding(value: u64) -> RootBinding {
        RootBinding::new(
            BindingSource::StaticValue(value),
            AvailabilityPhase::CompileProfile,
            FactProvenance::StaticallyProven,
        )
        .unwrap()
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
            &BindingSource::StaticValue(2)
        );
        assert_eq!(
            env.binding(&right).unwrap().source(),
            &BindingSource::StaticValue(3)
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
            &BindingSource::StaticValue(4)
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
            key: "tiler.target.max-threads".to_owned(),
            version: 1,
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
                    BindingSource::StaticValue(7),
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
                    BindingSource::CallerParameter {
                        key: "batch".to_owned(),
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
}
