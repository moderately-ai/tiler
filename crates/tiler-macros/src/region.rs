//! What a parsed region *means*: names resolved, operations looked up in the
//! governed semantic profile, and the tokens one invocation expands to.
//!
//! [`crate::grammar`] decides the shape of the text and nothing else. This
//! module is where every name acquires a referent — an element type, a declared
//! operand, a registered operation — and where the two things an expansion
//! produces come from: the region facts generated code carries, and the public
//! logical program the region denotes.
//!
//! # One operation vocabulary, and it is the registry's
//!
//! `*` and `+` are not spellings this frontend defines. They resolve to
//! [`multiply_f32_op`] and [`add_f32_op`], the operation keys the governed
//! semantic profile registers, and the elementwise rule below is the rule that
//! profile's own inferencer applies — *"operand shapes must match or one operand
//! must be scalar"*, quoted from the rejection it returns. An operator with no
//! registered operation is refused at its token rather than given a meaning
//! here, because a second operation vocabulary disconnected from the public
//! logical program is precisely what Tom's syntax decision does not authorize.
//!
//! # The public logical program is constructed exactly when it is representable
//!
//! `tiler_ir::shape::Shape` is a **fixed**-extent vocabulary: an
//! [`Extent`](tiler_ir::shape::Extent) is a `u64`, and no symbol reaches it. A
//! region's `sym n` is bound from operand metadata at
//! [`AvailabilityPhase::LiveDevicePreflight`] — at run time, from the values the
//! invocation is handed — so at expansion time there is no extent to give the
//! semantic layer.
//!
//! So a region whose every declared extent is a literal is constructed and
//! verified as a real [`SemanticProgram`], and the shape that program's
//! registry *infers* for the result is required to equal the shape this module
//! *derived*: the authority decides, and the derivation is checked against it
//! wherever it can be. A region carrying a symbolic extent cannot be, and this
//! module says so in [`ProgramEvidence`] rather than substituting a
//! representative extent and calling the result verified — a program built over
//! invented extents would be a different program, and its identity would name
//! something no consumer wrote.
//!
//! That gap is the frontend's half of the workspace's open symbolic profile, not
//! a shortcut taken here; `carry-symbolic-extents-into-the-semantic-program`
//! owns it. What is *not* deferred for either kind of region is the runtime contract:
//! the emitted facts check every operand's rank and stored scalar, unify every
//! symbol, and construct the declared result, symbolic or not.
//!
//! # The compiler is not invoked here
//!
//! Construction stops at a verified [`SemanticProgram`]. It does not continue
//! into `tiler_compiler::session`, and this crate holds no edge to that crate.
//!
//! The reason it could not was measured at `b623670`: the approved region over
//! three `f32[4]` inputs was refused by `compile_governed` under all four
//! `NumericalContract` values with `UnsupportedCapability { rule: "signature" }`
//! before any target-qualified trace, because both recognized program shapes
//! opened with `program.input_count() != 1`. `docs/integration/frontends.md`
//! requires target-neutral optimizer and verifier failures to become
//! *unconditional* `compile_error!` diagnostics, so wiring the compiler in then
//! would have made the region Tom approved a compile error at every call site.
//!
//! That refusal is gone. The scheduled-region vocabulary names which input
//! tensor each access and each scalar leaf reads, the combined `signature` rule
//! is split so each gate names its own refusal, and the approved region now
//! reaches a complete verified plan on the governed profile. It still refuses
//! under `RelaxedF32` alone — the contract that permits arithmetic contraction
//! declines any fused multiply-then-add body, at any input count — so the
//! diagnostic a wired-in compiler would raise is now contract-dependent rather
//! than unconditional. Whether that is a `compile_error!`, a narrowed contract,
//! or a deferred edge is what
//! `admit-multi-input-elementwise-programs-at-the-compiler-boundary` decides;
//! `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` carries the
//! executable statement of both halves.
//!
//! [`AvailabilityPhase::LiveDevicePreflight`]: tiler_ir::program::abi::AvailabilityPhase::LiveDevicePreflight
//! [`multiply_f32_op`]: tiler_ir::semantic::multiply_f32_op
//! [`add_f32_op`]: tiler_ir::semantic::add_f32_op

use core::fmt;

use tiler_ir::program::StorageScalar;
use tiler_ir::semantic::{
    BuildError, F32, F32Add, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder, Value,
};
use tiler_ir::shape::Shape;

use crate::binding::{BoundRegion, DeclaredAxis, RegionBindError, RegionDeclarations};
use crate::grammar::{
    AxisSyntax, Expression, Name, OperandSyntax, Operator, RegionSyntax, SyntaxError,
};

/// The interface key the region's single result is declared under.
///
/// `out` names the result expression and gives it no name of its own, so the
/// keyword is the key. It is stable rather than derived from the surrounding
/// `let`: a region's identity is a function of what it declares, and the Rust
/// binding a caller happens to assign it to is not part of the region.
const RESULT_KEY: &str = "out";

/// The element types a region may declare.
///
/// One entry, and the reason is the registry rather than a bounded ambition:
/// [`F32`] is the value-type marker the governed semantic profile registers for
/// plain tensor arithmetic, and `multiply-f32` and `add-f32` are the operations
/// registered over it. A region declaring another element type would have no
/// operation to apply to it.
const ELEMENT_TYPES: [(&str, StorageScalar); 1] = [("f32", StorageScalar::F32)];

/// Whether an expansion could construct the region's public logical program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgramEvidence {
    /// Every declared extent is a literal, so the region was constructed as a
    /// [`SemanticProgram`] through the governed registry, verified, and its
    /// inferred result shape checked against the derived one.
    ///
    /// [`SemanticProgram`]: tiler_ir::semantic::SemanticProgram
    Verified,
    /// A declared extent is symbolic, which the fixed-extent semantic shape
    /// vocabulary cannot carry. The operations were still resolved through the
    /// governed registry's keys and the runtime contract is unchanged; what is
    /// absent is the specialized program and everything derived from it.
    DeferredSymbolicExtent,
}

/// One region, lowered to what an expansion emits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Expansion<S> {
    /// The `RegionFacts` literal generated code carries, as Rust source text.
    pub(crate) facts: String,
    /// The operand names, in the interface order the facts declare, each
    /// carrying the token that names the Rust value to be supplied.
    pub(crate) operands: Vec<Name<S>>,
    /// Whether the public logical program was constructible.
    pub(crate) program: ProgramEvidence,
}

/// Why a syntactically well-formed region has no meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionError<S> {
    /// The tokens are not a region.
    Syntax(SyntaxError<S>),
    /// The declarations cannot be bound.
    Binding(RegionBindError<S>),
    /// An operand declares an element type this profile does not register.
    UnknownElementType {
        /// The name as written.
        name: String,
        /// The element-type token.
        span: S,
    },
    /// A name is not usable as a stable interface key.
    InvalidInterfaceKey {
        /// The name as written.
        name: String,
        /// The token that names it.
        span: S,
        /// The IR's own refusal.
        source: BuildError,
    },
    /// The body references a name no `in` statement declares.
    UnknownOperand {
        /// The name as written.
        name: String,
        /// The reference.
        span: S,
    },
    /// An operation's operands are neither equally shaped nor scalar.
    ///
    /// The registry's own rule, applied here so a region carrying a symbolic
    /// extent — which it cannot hand to the registry — is refused by the same
    /// rule as one that can.
    IncompatibleOperandShapes {
        /// The operator as written.
        operator: &'static str,
        /// The left operand's declared axes.
        left: String,
        /// The right operand's declared axes.
        right: String,
        /// The operator token.
        span: S,
    },
    /// Constructing the region as a public logical program was refused.
    ///
    /// Carried rather than flattened (ADR 0074 convention 1): the semantic
    /// authority's own reason is what tells a consumer whether the region, the
    /// operation, or the shape was rejected.
    Program {
        /// The token the refusal is reported at.
        span: S,
        /// The authority's own message.
        detail: String,
    },
    /// The derived result shape is not the one the registry inferred.
    ///
    /// A defect in this module rather than in the invocation, and a typed
    /// refusal rather than a panic, because a panic inside an expansion aborts
    /// rustc with no span at all.
    ResultShapeDisagreement {
        /// What this module derived.
        derived: String,
        /// What the semantic registry inferred.
        inferred: String,
        /// The `out` keyword.
        span: S,
    },
}

impl<S> fmt::Display for RegionError<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(source) => source.fmt(formatter),
            Self::Binding(source) => source.fmt(formatter),
            Self::UnknownElementType { name, .. } => write!(
                formatter,
                "`{name}` is not an element type a region may declare; this profile registers \
                 {}, because those are the element types the governed semantic operation profile \
                 has operations over",
                rendered_element_types(),
            ),
            Self::InvalidInterfaceKey { name, source, .. } => write!(
                formatter,
                "`{name}` cannot be a region's interface key: {source}"
            ),
            Self::UnknownOperand { name, .. } => write!(
                formatter,
                "`{name}` is used in the region's result but no `in` statement declares it; a \
                 region reads only the operands it declares, never a value from the surrounding \
                 Rust"
            ),
            Self::IncompatibleOperandShapes {
                operator,
                left,
                right,
                ..
            } => write!(
                formatter,
                "`{operator}` cannot combine a `{left}` operand with a `{right}` one; operand \
                 shapes must match or one operand must be scalar"
            ),
            Self::Program { detail, .. } => write!(
                formatter,
                "this region is not a valid public logical program: {detail}"
            ),
            Self::ResultShapeDisagreement {
                derived, inferred, ..
            } => write!(
                formatter,
                "this region's result was derived as `{derived}` and the semantic registry \
                 inferred `{inferred}`; this is a defect in `tiler-macros`, not in the invocation"
            ),
        }
    }
}

impl<S> RegionError<S> {
    /// Returns the span this refusal must be reported at.
    pub(crate) const fn span(&self) -> &S {
        match self {
            Self::Syntax(source) => source.span(),
            Self::Binding(source) => source.span(),
            Self::UnknownElementType { span, .. }
            | Self::InvalidInterfaceKey { span, .. }
            | Self::UnknownOperand { span, .. }
            | Self::IncompatibleOperandShapes { span, .. }
            | Self::Program { span, .. }
            | Self::ResultShapeDisagreement { span, .. } => span,
        }
    }
}

impl<S> From<SyntaxError<S>> for RegionError<S> {
    fn from(source: SyntaxError<S>) -> Self {
        Self::Syntax(source)
    }
}

impl<S> From<RegionBindError<S>> for RegionError<S> {
    fn from(source: RegionBindError<S>) -> Self {
        Self::Binding(source)
    }
}

/// Renders the element-type vocabulary a diagnostic offers.
fn rendered_element_types() -> String {
    ELEMENT_TYPES
        .iter()
        .map(|(name, _)| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// One operand after its element type and axes are resolved.
struct ResolvedOperand<S> {
    key: InputKey,
    storage_scalar: StorageScalar,
    axes: Vec<DeclaredAxis<S>>,
    name: Name<S>,
}

/// One subexpression's declared element type and shape.
struct ResolvedValue<S> {
    storage_scalar: StorageScalar,
    axes: Vec<DeclaredAxis<S>>,
}

/// Lowers one parsed region to the expansion it denotes.
///
/// # Errors
///
/// Returns the first [`RegionError`], carrying the span of the token that
/// caused it.
pub(crate) fn lower<S: Copy>(syntax: &RegionSyntax<S>) -> Result<Expansion<S>, RegionError<S>> {
    let mut declarations = RegionDeclarations::new(syntax.region);
    for symbol in &syntax.symbols {
        declarations.declare_symbol(symbol.text.clone(), symbol.span)?;
    }

    let mut operands = Vec::with_capacity(syntax.operands.len());
    for operand in &syntax.operands {
        let resolved = resolve_operand(operand)?;
        declarations.operand(
            resolved.key.clone(),
            resolved.storage_scalar,
            resolved.axes.clone(),
            resolved.name.span,
        )?;
        operands.push(resolved);
    }
    refuse_undeclared_symbols(syntax, &operands)?;

    let result = resolve_expression(&syntax.body, &operands)?;
    let result_key =
        OutputKey::new(RESULT_KEY).map_err(|source| RegionError::InvalidInterfaceKey {
            name: RESULT_KEY.to_owned(),
            span: syntax.out,
            source,
        })?;
    declarations.result(
        result_key,
        result.storage_scalar,
        result.axes.clone(),
        syntax.out,
    )?;

    let bound: BoundRegion = declarations.bind()?;
    let program = verify_public_logical_program(syntax, &operands, &result)?;

    Ok(Expansion {
        facts: bound.facts_source(),
        operands: operands.into_iter().map(|operand| operand.name).collect(),
        program,
    })
}

/// Refuses an operand axis naming a symbol no `sym` statement declares.
///
/// [`RegionDeclarations::bind`] applies this same rule and remains its
/// authority — it covers the result's axes as well, and it is what any other
/// caller of that module gets. The rule is stated once more here for *ordering*
/// rather than for a second opinion: an axis naming an undeclared symbol makes
/// every shape comparison involving it meaningless, so
/// `in a: f32[n], b: f32[k]; out a * b` would otherwise be refused as "`*`
/// cannot combine a `[n]` operand with a `[k]` one" — a message that names a
/// consequence and leaves the cause, an undeclared `k`, for the reader to spot.
///
/// # Errors
///
/// Returns [`RegionBindError::UndeclaredSymbol`], the binding module's own
/// variant, so one refusal vocabulary covers the subject however it is reached.
fn refuse_undeclared_symbols<S: Copy>(
    syntax: &RegionSyntax<S>,
    operands: &[ResolvedOperand<S>],
) -> Result<(), RegionError<S>> {
    for axis in operands.iter().flat_map(|operand| &operand.axes) {
        let DeclaredAxis::Symbol { name, span } = axis else {
            continue;
        };
        if !syntax
            .symbols
            .iter()
            .any(|declared| declared.text.as_str() == name)
        {
            return Err(RegionBindError::UndeclaredSymbol {
                name: name.clone(),
                span: *span,
            }
            .into());
        }
    }
    Ok(())
}

/// Resolves one operand's element type and axes.
fn resolve_operand<S: Copy>(
    operand: &OperandSyntax<S>,
) -> Result<ResolvedOperand<S>, RegionError<S>> {
    let storage_scalar = ELEMENT_TYPES
        .iter()
        .find(|(name, _)| *name == operand.dtype.text)
        .map(|(_, scalar)| *scalar)
        .ok_or_else(|| RegionError::UnknownElementType {
            name: operand.dtype.text.clone(),
            span: operand.dtype.span,
        })?;

    let key =
        InputKey::new(&operand.name.text).map_err(|source| RegionError::InvalidInterfaceKey {
            name: operand.name.text.clone(),
            span: operand.name.span,
            source,
        })?;

    Ok(ResolvedOperand {
        key,
        storage_scalar,
        axes: operand
            .axes
            .iter()
            .map(|axis| match axis {
                AxisSyntax::Symbol(name) => DeclaredAxis::Symbol {
                    name: name.text.clone(),
                    span: name.span,
                },
                AxisSyntax::Literal { value, .. } => DeclaredAxis::Literal(*value),
            })
            .collect(),
        name: operand.name.clone(),
    })
}

/// Resolves one subexpression's element type and shape.
fn resolve_expression<S: Copy>(
    expression: &Expression<S>,
    operands: &[ResolvedOperand<S>],
) -> Result<ResolvedValue<S>, RegionError<S>> {
    match expression {
        Expression::Operand(name) => operands
            .iter()
            .find(|operand| operand.name.text == name.text)
            .map(|operand| ResolvedValue {
                storage_scalar: operand.storage_scalar,
                axes: operand.axes.clone(),
            })
            .ok_or_else(|| RegionError::UnknownOperand {
                name: name.text.clone(),
                span: name.span,
            }),
        Expression::Binary {
            operator,
            span,
            left,
            right,
        } => {
            let left = resolve_expression(left, operands)?;
            let right = resolve_expression(right, operands)?;
            // Every registered operation is `f32`-only and every admitted
            // element type is `f32`, so the two sides cannot differ. The check
            // is written anyway because widening `ELEMENT_TYPES` must be a
            // refusal here rather than a silently mixed operation.
            if left.storage_scalar != right.storage_scalar {
                return Err(RegionError::IncompatibleOperandShapes {
                    operator: operator.as_str(),
                    left: rendered_axes(&left.axes),
                    right: rendered_axes(&right.axes),
                    span: *span,
                });
            }
            Ok(ResolvedValue {
                storage_scalar: left.storage_scalar,
                axes: elementwise_axes(*operator, &left.axes, &right.axes, *span)?,
            })
        }
    }
}

/// Applies the registry's elementwise shape rule to two declared shapes.
///
/// Equal shapes, or one side scalar. Two axes naming different symbols are
/// *not* equal: nothing at expansion time proves `n` and `m` take one value, and
/// treating them as compatible would defer a shape error into a wrong result.
fn elementwise_axes<S: Copy>(
    operator: Operator,
    left: &[DeclaredAxis<S>],
    right: &[DeclaredAxis<S>],
    span: S,
) -> Result<Vec<DeclaredAxis<S>>, RegionError<S>> {
    if left.is_empty() {
        return Ok(right.to_vec());
    }
    if right.is_empty() || axes_agree(left, right) {
        return Ok(left.to_vec());
    }
    Err(RegionError::IncompatibleOperandShapes {
        operator: operator.as_str(),
        left: rendered_axes(left),
        right: rendered_axes(right),
        span,
    })
}

/// Reports whether two declared shapes name the same axes, spans aside.
///
/// A span is where a name was written, not part of what it means, so comparing
/// [`DeclaredAxis`] values directly would make `f32[n] * f32[n]` depend on which
/// tokens spelled the two `n`s.
fn axes_agree<S>(left: &[DeclaredAxis<S>], right: &[DeclaredAxis<S>]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| match (left, right) {
                (DeclaredAxis::Literal(left), DeclaredAxis::Literal(right)) => left == right,
                (
                    DeclaredAxis::Symbol { name: left, .. },
                    DeclaredAxis::Symbol { name: right, .. },
                ) => left == right,
                _ => false,
            })
}

/// Renders one declared shape the way a region spells it.
fn rendered_axes<S>(axes: &[DeclaredAxis<S>]) -> String {
    let rendered = axes
        .iter()
        .map(|axis| match axis {
            DeclaredAxis::Literal(extent) => extent.to_string(),
            DeclaredAxis::Symbol { name, .. } => name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{rendered}]")
}

/// Returns the literal extents of a declared shape, or `None` if any is symbolic.
fn literal_extents<S>(axes: &[DeclaredAxis<S>]) -> Option<Vec<u64>> {
    axes.iter()
        .map(|axis| match axis {
            DeclaredAxis::Literal(extent) => Some(*extent),
            DeclaredAxis::Symbol { .. } => None,
        })
        .collect()
}

/// Constructs and verifies the region as a public logical program.
///
/// Returns [`ProgramEvidence::DeferredSymbolicExtent`] without constructing
/// anything when a declared extent is symbolic, because the semantic shape
/// vocabulary is fixed-extent and no value for the symbol exists yet.
fn verify_public_logical_program<S: Copy>(
    syntax: &RegionSyntax<S>,
    operands: &[ResolvedOperand<S>],
    result: &ResolvedValue<S>,
) -> Result<ProgramEvidence, RegionError<S>> {
    let mut extents = Vec::with_capacity(operands.len());
    for operand in operands {
        let Some(literal) = literal_extents(&operand.axes) else {
            return Ok(ProgramEvidence::DeferredSymbolicExtent);
        };
        extents.push(literal);
    }
    let Some(derived) = literal_extents(&result.axes) else {
        return Ok(ProgramEvidence::DeferredSymbolicExtent);
    };

    let refused = |span: S| {
        move |source: BuildError| RegionError::Program {
            span,
            detail: source.to_string(),
        }
    };

    let mut builder =
        SemanticProgramBuilder::try_standard().map_err(|source| RegionError::Program {
            span: syntax.region,
            detail: source.to_string(),
        })?;

    let mut values: Vec<(&str, Value<F32>)> = Vec::with_capacity(operands.len());
    for (operand, extents) in operands.iter().zip(&extents) {
        let shape = Shape::try_from_dims(extents.iter().copied()).map_err(|source| {
            RegionError::Program {
                span: operand.name.span,
                detail: source.to_string(),
            }
        })?;
        let value = builder
            .input::<F32>(operand.key.clone(), shape)
            .map_err(refused(operand.name.span))?;
        values.push((operand.name.text.as_str(), value));
    }

    let root = apply_expression(&mut builder, &syntax.body, &values)?;
    let output_key =
        OutputKey::new(RESULT_KEY).map_err(|source| RegionError::InvalidInterfaceKey {
            name: RESULT_KEY.to_owned(),
            span: syntax.out,
            source,
        })?;
    builder
        .output(output_key, root)
        .map_err(refused(syntax.out))?;
    let program = builder.build().map_err(|source| RegionError::Program {
        span: syntax.region,
        detail: source.to_string(),
    })?;

    // The registry is the authority over a result's shape, so what this module
    // derived is compared against what the registry inferred rather than
    // trusted beside it.
    let output = program
        .outputs()
        .next()
        .ok_or_else(|| RegionError::Program {
            span: syntax.out,
            detail: "the verified program declares no output".to_owned(),
        })?;
    let inferred = program
        .shape(output.value())
        .map_err(|source| RegionError::Program {
            span: syntax.out,
            detail: source.to_string(),
        })?;
    let inferred_extents: Vec<u64> = inferred
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    if inferred_extents != derived {
        return Err(RegionError::ResultShapeDisagreement {
            derived: rendered_axes(&result.axes),
            inferred: inferred.to_string(),
            span: syntax.out,
        });
    }

    Ok(ProgramEvidence::Verified)
}

/// Applies one subexpression through the governed operation facades.
fn apply_expression<S: Copy>(
    builder: &mut SemanticProgramBuilder,
    expression: &Expression<S>,
    values: &[(&str, Value<F32>)],
) -> Result<Value<F32>, RegionError<S>> {
    match expression {
        Expression::Operand(name) => values
            .iter()
            .find(|(declared, _)| *declared == name.text)
            .map(|(_, value)| *value)
            .ok_or_else(|| RegionError::UnknownOperand {
                name: name.text.clone(),
                span: name.span,
            }),
        Expression::Binary {
            operator,
            span,
            left,
            right,
        } => {
            let left = apply_expression(builder, left, values)?;
            let right = apply_expression(builder, right, values)?;
            let applied = match operator {
                Operator::Multiply => F32Multiply::apply(builder, left, right),
                Operator::Add => F32Add::apply(builder, left, right),
            };
            applied.map_err(|source| RegionError::Program {
                span: *span,
                detail: source.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests;
