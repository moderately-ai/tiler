//! Expansion-time binding of a region's declared symbols, operands, and result.
//!
//! # What `sym n;` means, and why it is decided here
//!
//! Tom ratified operand unification as the default meaning on 2026-07-30. One
//! `sym n;` declares one logical extent variable; its runtime value is unified
//! from every operand dimension that names `n`; at least one occurrence must
//! source it; and every additional occurrence owes an equality against the first
//! checked value.
//!
//! Three properties follow, and each of them is a rule this module holds rather
//! than an accident of how it happens to iterate:
//!
//! - **The canonical source does not depend on declaration order.** It is the
//!   least occurrence in the canonical order of interface key and then axis, not
//!   the first one written. Reordering the `in` list, or the `sym` lines, leaves
//!   the same axis sourcing the symbol.
//! - **The environment names keys, not positions.** A binding is
//!   [`BindingSource::InputDimension`] over an [`InputKey`] and an [`Axis`], so
//!   graph identity is a function of the interface a region declares and not of
//!   the order it was written in. [`BoundRegion::environment_identity`] is what a
//!   test can compare to see that.
//! - **An additional occurrence is an obligation, not a second binding.** ADR
//!   0008 gives every symbol exactly one root binding, and [`ShapeEnv`] enforces
//!   it: a second `bind` is [`ShapeEnvError::AlreadyBound`] rather than an
//!   update. So "`b` axis 1 also has extent `n`" cannot be a binding at all. It
//!   is a runtime interface validation, and it is carried beside the environment
//!   rather than folded into it — which is why this module composes with the
//!   promoted `ShapeEnv` profile instead of restating any part of it.
//!
//! # Why the errors are generic over a span
//!
//! Every refusal here names a token the consumer wrote, and a refusal that
//! could only be reported at the invocation as a whole would be worth less than
//! the check that produced it. `proc_macro::Span` cannot be constructed outside
//! an expanding proc macro, though, so an error type hard-wired to it would be
//! untestable — the module would compile and its diagnostics would be evidence
//! of nothing. The span is therefore a type parameter: the expansion supplies
//! `proc_macro::Span`, and the tests below supply a marker they can assert on.
//!
//! [`ShapeEnv`]: tiler_ir::shape::ShapeEnv
//! [`ShapeEnvError::AlreadyBound`]: tiler_ir::shape::ShapeEnvError::AlreadyBound

use core::fmt;
use std::sync::Arc;

use tiler_ir::program::StorageScalar;
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{InputKey, OutputKey};
use tiler_ir::shape::{
    Axis, BindingSource, FactProvenance, RootBinding, ShapeEnv, ShapeEnvBuilder, ShapeEnvError,
    ShapeEnvIdentity, ShapeSymbol, SymbolScope,
};

/// The scope every inline region's symbols are declared in.
///
/// One constant rather than a per-invocation value, and that is deliberate: the
/// scope participates in [`ShapeEnvIdentity`], so a unique scope per expansion
/// would give two textually identical regions two identities and defeat the
/// expansion cache that identity exists to key. Scopes keep two symbols apart
/// *within* one environment; two regions already have two environments, so they
/// need no help staying apart.
const REGION_SCOPE: &[u8] = b"tiler.inline-region.v1";

/// The number of results one inline region may declare today.
///
/// A bounded profile, not a limit of the model: the semantic graph carries
/// ordered named outputs and multi-result operations. What is bounded is this
/// frontend's *runtime value* boundary, which returns one constructed value; a
/// second result needs a decided tuple or record shape at the call site, and
/// inventing one here would publish a boundary no ticket reviewed.
const MAX_REGION_RESULTS: usize = 1;

/// The capabilities every region of this profile requires of an adapter.
///
/// Stated once here rather than derived per region: nothing in the profile makes
/// a region need less, and a per-region set would suggest a variability the
/// bounded profile does not have.
///
/// Variant *names* rather than the facade's `AdapterCapability` values, because
/// this crate cannot name anything in `tiler` — the facade depends on it, so the
/// edge cannot run back. What keeps the spellings honest is the pair of fixture
/// tests below: the compile-pass fixtures are compiled, so a name that is not a
/// variant fails there, and the emitted text is compared against those same
/// files byte for byte, so an emitter that spelled a different variant would not
/// match. Neither check alone is sufficient and together they close it.
const REQUIRED_CAPABILITIES: [&str; 2] = ["DenseRowMajorStorage", "ResultConstruction"];

/// One extent of a declared operand or result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DeclaredAxis<S> {
    /// An extent the region fixed literally.
    Literal(u64),
    /// An extent naming a declared symbol.
    Symbol {
        /// The symbol's name as the region spelled it.
        name: String,
        /// The token the name was written at.
        span: S,
    },
}

/// One `sym` declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DeclaredSymbol<S> {
    name: String,
    span: S,
}

/// One declared operand: its interface key, stored scalar, and axes.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DeclaredOperand<S> {
    key: InputKey,
    storage_scalar: StorageScalar,
    axes: Vec<DeclaredAxis<S>>,
}

/// The region's declared result.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DeclaredResult<S> {
    key: OutputKey,
    storage_scalar: StorageScalar,
    axes: Vec<DeclaredAxis<S>>,
}

/// Why a region's declarations cannot be bound.
///
/// Every variant carries the span of the token that caused it, so a diagnostic
/// lands on the offending declaration rather than on the invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionBindError<S> {
    /// Two `sym` statements declared one name.
    DuplicateSymbol {
        /// The repeated name.
        name: String,
        /// The rejected second declaration.
        span: S,
        /// The declaration already accepted.
        first: S,
    },
    /// Two operands declared one interface key.
    DuplicateOperand {
        /// The repeated key.
        key: InputKey,
        /// The rejected second declaration.
        span: S,
        /// The declaration already accepted.
        first: S,
    },
    /// The region declared more results than this bounded profile returns.
    UnsupportedResultCardinality {
        /// Results the region declared.
        declared: usize,
        /// Results this profile admits.
        limit: usize,
        /// The rejected declaration.
        span: S,
    },
    /// An axis named a symbol no `sym` statement declared.
    UndeclaredSymbol {
        /// The name the axis spelled.
        name: String,
        /// The token it was spelled at.
        span: S,
    },
    /// A declared symbol is named by no operand axis, so nothing sources it.
    ///
    /// A result axis does not count. A result's extent is computed from the
    /// region's inputs; reading it back out of a value that does not exist yet
    /// would be circular.
    UnboundSymbol {
        /// The symbol left unsourced.
        name: String,
        /// Its `sym` declaration.
        span: S,
    },
    /// The region declared no operands, so no symbol could be sourced and no
    /// context exists to construct a result from.
    NoOperands {
        /// The region as a whole.
        span: S,
    },
    /// An operand's rank exceeds what an [`Axis`] can address.
    OperandRankTooLarge {
        /// The operand's key.
        key: InputKey,
        /// The rejected rank.
        rank: usize,
        /// The governed limit.
        limit: usize,
        /// The operand's declaration.
        span: S,
    },
    /// The promoted shape environment refused a derived declaration or binding.
    ///
    /// Carried rather than flattened (ADR 0074 convention 1): the environment is
    /// the authority over symbol names, scopes, and binding phases, and its
    /// reason survives to the diagnostic.
    Environment {
        /// The symbol whose declaration or binding was refused.
        name: String,
        /// The declaration it came from.
        span: S,
        /// The environment's own refusal.
        source: ShapeEnvError,
    },
    /// The shape environment refused the region as a whole at verification.
    ///
    /// Separate from [`Self::Environment`] because it names no one declaration:
    /// whole-object verification decides free symbols, the supported arithmetic
    /// fragment, and constraint contradiction, and reporting it against an
    /// arbitrary symbol would attribute a region-level refusal to a token that
    /// may be innocent.
    EnvironmentVerification {
        /// The region as a whole.
        span: S,
        /// The environment's own refusal.
        source: ShapeEnvError,
    },
}

impl<S> fmt::Display for RegionBindError<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSymbol { name, .. } => write!(
                formatter,
                "`sym {name}` is declared twice; one `sym` statement declares one extent variable, \
                 and every operand axis naming it is unified against that one declaration"
            ),
            Self::DuplicateOperand { key, .. } => write!(
                formatter,
                "operand `{}` is declared twice; an interface key names one operand",
                key.as_str()
            ),
            Self::UnsupportedResultCardinality {
                declared, limit, ..
            } => write!(
                formatter,
                "this region declares {declared} results and the inline profile returns {limit}; \
                 a multi-result region needs a decided call-site shape, which \
                 `define-inline-symbol-binding-and-runtime-value-adaptation` reserved rather than \
                 invented"
            ),
            Self::UndeclaredSymbol { name, .. } => write!(
                formatter,
                "`{name}` is used as an extent but never declared; add `sym {name};` to the \
                 region's declaration block"
            ),
            Self::UnboundSymbol { name, .. } => write!(
                formatter,
                "`sym {name};` is declared but no operand axis names it, so nothing supplies its \
                 value; give an operand an axis of extent `{name}`, or remove the declaration"
            ),
            Self::NoOperands { .. } => formatter.write_str(
                "this region declares no operands; an inline region binds its extents from operand \
                 metadata and constructs its result through the adapter that supplied them",
            ),
            Self::OperandRankTooLarge {
                key, rank, limit, ..
            } => write!(
                formatter,
                "operand `{}` declares rank {rank}, past the governed limit {limit}",
                key.as_str()
            ),
            Self::Environment { name, source, .. } => write!(
                formatter,
                "the shape environment refused `{name}`: {source}"
            ),
            Self::EnvironmentVerification { source, .. } => write!(
                formatter,
                "the shape environment refused this region: {source}"
            ),
        }
    }
}

impl<S> RegionBindError<S> {
    /// Returns the span this refusal must be reported at.
    pub(crate) const fn span(&self) -> &S {
        match self {
            Self::DuplicateSymbol { span, .. }
            | Self::DuplicateOperand { span, .. }
            | Self::UnsupportedResultCardinality { span, .. }
            | Self::UndeclaredSymbol { span, .. }
            | Self::UnboundSymbol { span, .. }
            | Self::NoOperands { span }
            | Self::OperandRankTooLarge { span, .. }
            | Self::Environment { span, .. }
            | Self::EnvironmentVerification { span, .. } => span,
        }
    }
}

/// An append-only draft of one region's declarations.
///
/// Local invariants are checked as each declaration arrives and leave the draft
/// unchanged when they fail; whole-region questions — whether every symbol is
/// sourced, whether every named symbol is declared — are decided by the
/// consuming [`Self::bind`], per ADR 0074 convention 4.
#[derive(Clone, Debug)]
pub(crate) struct RegionDeclarations<S> {
    region: S,
    symbols: Vec<DeclaredSymbol<S>>,
    operands: Vec<(DeclaredOperand<S>, S)>,
    results: Vec<DeclaredResult<S>>,
}

impl<S: Copy> RegionDeclarations<S> {
    /// Opens a draft for one region.
    pub(crate) const fn new(region: S) -> Self {
        Self {
            region,
            symbols: Vec::new(),
            operands: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Records one `sym` declaration.
    ///
    /// # Errors
    ///
    /// Returns [`RegionBindError::DuplicateSymbol`] naming both declarations.
    pub(crate) fn declare_symbol(
        &mut self,
        name: impl Into<String>,
        span: S,
    ) -> Result<(), RegionBindError<S>> {
        let name = name.into();
        if let Some(first) = self.symbols.iter().find(|held| held.name == name) {
            return Err(RegionBindError::DuplicateSymbol {
                name,
                span,
                first: first.span,
            });
        }
        self.symbols.push(DeclaredSymbol { name, span });
        Ok(())
    }

    /// Records one operand.
    ///
    /// # Errors
    ///
    /// Returns [`RegionBindError::DuplicateOperand`] naming both declarations.
    pub(crate) fn operand(
        &mut self,
        key: InputKey,
        storage_scalar: StorageScalar,
        axes: Vec<DeclaredAxis<S>>,
        span: S,
    ) -> Result<(), RegionBindError<S>> {
        if let Some((_, first)) = self
            .operands
            .iter()
            .find(|(held, _)| held.key.as_str() == key.as_str())
        {
            return Err(RegionBindError::DuplicateOperand {
                key,
                span,
                first: *first,
            });
        }
        self.operands.push((
            DeclaredOperand {
                key,
                storage_scalar,
                axes,
            },
            span,
        ));
        Ok(())
    }

    /// Records the region's result.
    ///
    /// # Errors
    ///
    /// Returns [`RegionBindError::UnsupportedResultCardinality`] for a result
    /// past the bounded profile, which is where "multiple outputs" is refused —
    /// at the declaration that crosses the bound, so the diagnostic names it.
    pub(crate) fn result(
        &mut self,
        key: OutputKey,
        storage_scalar: StorageScalar,
        axes: Vec<DeclaredAxis<S>>,
        span: S,
    ) -> Result<(), RegionBindError<S>> {
        if self.results.len() >= MAX_REGION_RESULTS {
            return Err(RegionBindError::UnsupportedResultCardinality {
                declared: self.results.len().saturating_add(1),
                limit: MAX_REGION_RESULTS,
                span,
            });
        }
        self.results.push(DeclaredResult {
            key,
            storage_scalar,
            axes,
        });
        Ok(())
    }

    /// Unifies every symbol and freezes the region.
    ///
    /// # Errors
    ///
    /// Returns [`RegionBindError::NoOperands`], [`RegionBindError::UndeclaredSymbol`],
    /// [`RegionBindError::UnboundSymbol`], [`RegionBindError::OperandRankTooLarge`],
    /// [`RegionBindError::UnsupportedResultCardinality`] when no result was
    /// declared at all, or [`RegionBindError::Environment`] carrying the shape
    /// environment's own refusal.
    pub(crate) fn bind(self) -> Result<BoundRegion, RegionBindError<S>> {
        if self.operands.is_empty() {
            return Err(RegionBindError::NoOperands { span: self.region });
        }
        let Some(result) = self.results.first() else {
            return Err(RegionBindError::UnsupportedResultCardinality {
                declared: 0,
                limit: MAX_REGION_RESULTS,
                span: self.region,
            });
        };

        for (operand, span) in &self.operands {
            if u32::try_from(operand.axes.len()).is_err() {
                return Err(RegionBindError::OperandRankTooLarge {
                    key: operand.key.clone(),
                    rank: operand.axes.len(),
                    limit: u32::MAX as usize,
                    span: *span,
                });
            }
        }

        // Every symbol named anywhere must have been declared. Operands and the
        // result are both checked, so a result axis naming an undeclared symbol
        // is refused here rather than becoming an unresolvable extent later.
        let declared_names: Vec<&str> = self
            .symbols
            .iter()
            .map(|symbol| symbol.name.as_str())
            .collect();
        for axis in self
            .operands
            .iter()
            .flat_map(|(operand, _)| &operand.axes)
            .chain(&result.axes)
        {
            if let DeclaredAxis::Symbol { name, span } = axis
                && !declared_names.contains(&name.as_str())
            {
                return Err(RegionBindError::UndeclaredSymbol {
                    name: name.clone(),
                    span: *span,
                });
            }
        }

        let scope =
            SymbolScope::new(REGION_SCOPE).map_err(|source| RegionBindError::Environment {
                name: String::from_utf8_lossy(REGION_SCOPE).into_owned(),
                span: self.region,
                source,
            })?;

        let mut environment = ShapeEnvBuilder::new();
        let mut bound = Vec::with_capacity(self.symbols.len());
        for declared in &self.symbols {
            let occurrences = self.occurrences_of(&declared.name);
            let Some((source, obligations)) = occurrences.split_first() else {
                return Err(RegionBindError::UnboundSymbol {
                    name: declared.name.clone(),
                    span: declared.span,
                });
            };

            let symbol =
                ShapeSymbol::new(scope.clone(), declared.name.clone()).map_err(|source| {
                    RegionBindError::Environment {
                        name: declared.name.clone(),
                        span: declared.span,
                        source,
                    }
                })?;
            let refused = |source: ShapeEnvError| RegionBindError::Environment {
                name: declared.name.clone(),
                span: declared.span,
                source,
            };
            environment.declare(symbol.clone()).map_err(refused)?;
            // `InputDimension` floors at `LiveDevicePreflight` and the index
            // layer's `EXTENT_PHASE_CEILING` is the same phase, so this is the
            // one admissible phase rather than a choice among several. The
            // provenance is `RuntimeValidated` because that is what this module
            // arranges: the extent is read from a supplied value and checked,
            // never demanded of the caller as a semantic precondition.
            let binding = RootBinding::new(
                BindingSource::InputDimension {
                    input: self.operands[source.operand].0.key.clone(),
                    axis: source.axis,
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .map_err(refused)?;
            environment.bind(&symbol, binding).map_err(refused)?;

            bound.push(BoundSymbol {
                symbol,
                source: *source,
                obligations: obligations.to_vec(),
            });
        }

        let environment =
            environment
                .build()
                .map_err(|source| RegionBindError::EnvironmentVerification {
                    span: self.region,
                    source,
                })?;

        // The environment's canonical order, so the emitted table and the
        // environment agree on what "symbol 0" means.
        bound.sort_by(|left, right| left.symbol.cmp(&right.symbol));

        let mut result_axes = Vec::with_capacity(result.axes.len());
        for axis in &result.axes {
            result_axes.push(match axis {
                DeclaredAxis::Literal(extent) => BoundResultAxis::Literal(*extent),
                DeclaredAxis::Symbol { name, span } => BoundResultAxis::Symbol(
                    bound
                        .iter()
                        .position(|held| held.symbol.name() == name)
                        // Unreachable: the loop above refused every undeclared
                        // name, and every declared symbol either reached `bound`
                        // or returned `UnboundSymbol`. Routed to a refusal
                        // rather than a panic anyway, because a panic inside an
                        // expansion aborts rustc with no span at all.
                        .ok_or_else(|| RegionBindError::UnboundSymbol {
                            name: name.clone(),
                            span: *span,
                        })?,
                ),
            });
        }
        let result = BoundResult {
            key: result.key.clone(),
            storage_scalar: result.storage_scalar,
            axes: result_axes,
        };

        Ok(BoundRegion {
            environment: Arc::new(environment),
            operands: self
                .operands
                .into_iter()
                .map(|(operand, _)| BoundOperand {
                    key: operand.key,
                    storage_scalar: operand.storage_scalar,
                    extents: operand
                        .axes
                        .iter()
                        .map(|axis| match axis {
                            DeclaredAxis::Literal(extent) => BoundOperandExtent::Literal(*extent),
                            DeclaredAxis::Symbol { .. } => BoundOperandExtent::Symbolic,
                        })
                        .collect(),
                })
                .collect(),
            symbols: bound,
            result,
        })
    }

    /// Returns every operand axis naming one symbol, in canonical order.
    ///
    /// Canonical means by interface key and then axis position — **not** by the
    /// order the operands were written in. That is the whole of "the canonical
    /// source does not depend on declaration order": the head of this vector is
    /// the source and the tail is the obligations, so reordering the `in` list
    /// cannot move which axis a symbol is read from.
    fn occurrences_of(&self, name: &str) -> Vec<OperandAxis> {
        let mut found: Vec<(&str, OperandAxis)> = Vec::new();
        for (position, (operand, _)) in self.operands.iter().enumerate() {
            for (axis, declared) in operand.axes.iter().enumerate() {
                let DeclaredAxis::Symbol { name: spelled, .. } = declared else {
                    continue;
                };
                if spelled != name {
                    continue;
                }
                // `axes.len()` was proved to fit in a `u32` above, so this
                // conversion cannot fail for an axis of a checked operand.
                let Ok(index) = u32::try_from(axis) else {
                    continue;
                };
                found.push((
                    operand.key.as_str(),
                    OperandAxis {
                        operand: position,
                        axis: Axis::new(index),
                    },
                ));
            }
        }
        found.sort_by(|left, right| {
            left.0
                .cmp(right.0)
                .then_with(|| left.1.axis.get().cmp(&right.1.axis.get()))
        });
        found.into_iter().map(|(_, axis)| axis).collect()
    }
}

/// One operand axis of a region under binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OperandAxis {
    /// Index into the region's declared operands.
    operand: usize,
    /// Axis position within that operand's shape.
    axis: Axis,
}

/// One symbol, the axis it is read from, and the axes that owe it an equality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundSymbol {
    symbol: ShapeSymbol,
    source: OperandAxis,
    obligations: Vec<OperandAxis>,
}

/// One operand as the emitted facts describe it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundOperand {
    key: InputKey,
    storage_scalar: StorageScalar,
    extents: Vec<BoundOperandExtent>,
}

/// Where one axis of a declared operand gets the extent it must report.
///
/// A literal extent has to reach the emitted facts, because nothing else states
/// it: the environment binds symbols, and an axis naming no symbol owes no
/// obligation, so a region that dropped its literals would hand a runtime check
/// nothing to compare a supplied value against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundOperandExtent {
    /// An extent the region fixed literally.
    Literal(u64),
    /// An extent naming a declared symbol.
    ///
    /// Which symbol is deliberately not carried: the emitted symbol table is the
    /// single authority for that, and a second index here could disagree with
    /// it.
    Symbolic,
}

/// Where one result axis gets its extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundResultAxis {
    /// An extent fixed literally.
    Literal(u64),
    /// An extent equal to a bound symbol, by index into [`BoundRegion::symbols`].
    Symbol(usize),
}

/// The region's result as the emitted facts describe it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundResult {
    key: OutputKey,
    storage_scalar: StorageScalar,
    axes: Vec<BoundResultAxis>,
}

/// One region whose symbols are all declared, sourced, and unified.
///
/// Immutable and produced only by [`RegionDeclarations::bind`], so holding one
/// is evidence the whole region was decided rather than partly checked.
#[derive(Clone, Debug)]
pub(crate) struct BoundRegion {
    /// The verified environment, retained rather than dropped after it has
    /// decided. Building it is what *makes* the binding decision — `declare`
    /// and `bind` are where ADR 0008's one-root-binding rule is enforced and
    /// where a second occurrence proves to be an obligation rather than a
    /// binding — and the value itself is what the semantic builder is handed,
    /// so one region has one `ShapeEnvIdentity`.
    environment: Arc<ShapeEnv>,
    operands: Vec<BoundOperand>,
    symbols: Vec<BoundSymbol>,
    result: BoundResult,
}

impl BoundRegion {
    /// Returns the verified shape environment this region's symbols resolve in.
    ///
    /// This is the value a frontend hands to
    /// `SemanticProgramBuilder::try_standard_with_shape_environment` (and, later,
    /// to `IndexRegionBuilder::new_with_shape_environment`); nothing here
    /// duplicates what the environment decides.
    #[allow(
        dead_code,
        reason = "read by this module's tests; the expansion path clones the Arc via \
                  `environment_arc` rather than borrowing here"
    )]
    pub(crate) fn environment(&self) -> &ShapeEnv {
        &self.environment
    }

    /// Returns a clone of the `Arc` holding this region's environment.
    ///
    /// The clone is of the handle, not of the environment: two holders name
    /// one `ShapeEnvIdentity`.
    pub(crate) fn environment_arc(&self) -> Arc<ShapeEnv> {
        Arc::clone(&self.environment)
    }

    /// Returns the environment's canonical identity.
    ///
    /// Equal for two regions declaring one interface, whatever order their `in`
    /// list or their `sym` lines were written in, because a binding names an
    /// [`InputKey`] and an [`Axis`] and never a position.
    #[allow(
        dead_code,
        reason = "the identity a region's cache subject will be a function of; nothing composes a \
                  cache subject yet, and this module's tests are what keep the order-independence \
                  it exists to state from regressing"
    )]
    pub(crate) fn environment_identity(&self) -> &ShapeEnvIdentity {
        self.environment.identity()
    }

    /// Returns the declared symbols in the environment's canonical order.
    #[allow(
        dead_code,
        reason = "read by this module's tests, which assert the canonical source and the equality \
                  obligations directly rather than only through the rendered facts"
    )]
    pub(crate) fn symbols(&self) -> &[BoundSymbol] {
        &self.symbols
    }

    /// Renders the facade facts this region expands to, as Rust source text.
    ///
    /// Text rather than a `TokenStream` because the caller parses it once, the
    /// way `expand_region` already parses the anchor path, and because a string
    /// is what a test can compare against the compile-pass fixtures that stand
    /// in for generated code.
    ///
    /// Every absolute path it writes is under `::tiler::`. Nothing it writes
    /// reads a file, spawns a process, names another crate, or defers work to
    /// runtime compilation; `generated_facts_name_only_the_facade` is what holds
    /// that rather than this sentence.
    pub(crate) fn facts_source(&self) -> String {
        let operands = self
            .operands
            .iter()
            .map(|operand| {
                let extents = operand
                    .extents
                    .iter()
                    .map(|extent| match extent {
                        BoundOperandExtent::Literal(extent) => {
                            format!("::tiler::__private::OperandExtent::Literal({extent}u64)")
                        }
                        BoundOperandExtent::Symbolic => {
                            "::tiler::__private::OperandExtent::Symbolic".to_owned()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "::tiler::__private::OperandFacts {{ key: {}, storage_scalar: {}, extents: &[{extents}] }}",
                    rust_string(operand.key.as_str()),
                    storage_scalar_path(operand.storage_scalar),
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let symbols = self
            .symbols
            .iter()
            .map(|symbol| {
                let obligations = symbol
                    .obligations
                    .iter()
                    .map(|axis| axis_ref(*axis))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "::tiler::__private::SymbolFacts {{ name: {}, source: {}, obligations: &[{obligations}] }}",
                    rust_string(symbol.symbol.name()),
                    axis_ref(symbol.source),
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let capabilities = REQUIRED_CAPABILITIES
            .iter()
            .map(|capability| format!("::tiler::value::AdapterCapability::{capability}"))
            .collect::<Vec<_>>()
            .join(", ");

        let result_axes = self
            .result
            .axes
            .iter()
            .map(|axis| match axis {
                BoundResultAxis::Literal(extent) => {
                    format!("::tiler::__private::ResultAxis::Literal({extent}u64)")
                }
                BoundResultAxis::Symbol(index) => {
                    format!("::tiler::__private::ResultAxis::Symbol({index}usize)")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "::tiler::__private::RegionFacts {{ operands: &[{operands}], symbols: &[{symbols}], \
             capabilities: &[{capabilities}], result: ::tiler::__private::ResultFacts {{ key: {}, \
             storage_scalar: {}, axes: &[{result_axes}] }} }}",
            rust_string(self.result.key.as_str()),
            storage_scalar_path(self.result.storage_scalar),
        )
    }
}

/// Renders one axis reference.
fn axis_ref(axis: OperandAxis) -> String {
    format!(
        "::tiler::__private::AxisRef {{ operand: {}usize, axis: {}usize }}",
        axis.operand,
        axis.axis.get(),
    )
}

/// Renders one interface key as a Rust string literal.
///
/// `{:?}` on a `str` is Rust's own string-literal escaping, so a key containing
/// a quote or a backslash cannot close the literal early. Deriving the literal
/// from the validated key rather than from raw tokens is what keeps that true.
fn rust_string(value: &str) -> String {
    format!("{value:?}")
}

/// Renders one storage scalar as the path generated code names it.
///
/// Exhaustive rather than a lookup or a `Debug` rendering, per ADR 0074
/// convention 3's reasoning applied to token emission: widening the storage
/// vocabulary must be a build error here, not a variant this frontend silently
/// cannot spell.
const fn storage_scalar_path(scalar: StorageScalar) -> &'static str {
    match scalar {
        StorageScalar::U8 => "::tiler::value::StorageScalar::U8",
        StorageScalar::F32 => "::tiler::value::StorageScalar::F32",
        StorageScalar::Bf16 => "::tiler::value::StorageScalar::Bf16",
        StorageScalar::U32 => "::tiler::value::StorageScalar::U32",
    }
}

#[cfg(test)]
mod tests;
