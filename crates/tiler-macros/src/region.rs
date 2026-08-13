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
//! `strict_serial_sum(…)` and a scalar literal resolve the same way, to
//! [`strict_serial_sum_f32_op`] and [`constant_f32_op`]. Neither adds a rule:
//! the reduction's result shape is the registry's `without_axes`, and the
//! constant's rank-0 shape is what makes it the scalar side of the elementwise
//! rule already quoted.
//!
//! # A reduction is where the derived shape stops agreeing for free
//!
//! Every other operation this profile spells is shape-preserving, so "the shape
//! this module derived equals the shape the registry inferred" held whatever the
//! derivation did with rank. A reduction removes axes, and *which* axes is a
//! name resolution performed here — so the check below is the thing that catches
//! a name resolved to the wrong position, and it is checked before an expansion
//! can emit a result whose declared rank is not the program's.
//!
//! # Nothing here spells a plan
//!
//! A region states which axes it sums. Whether that becomes one kernel or two,
//! and whether anything is materialized between them, is decided by the
//! optimizer against a target profile; this module hands over the same program
//! either way.
//!
//! # The public logical program is constructed through the registry
//!
//! A region's declared extents — literal or symbolic — are handed to
//! [`SemanticProgramBuilder`] as [`SourcedExtent`]s, resolving in the one
//! [`ShapeEnv`] [`BoundRegion`] already verified. The builder is opened with
//! that environment rather than a second one, so the program and the binding
//! share one [`ShapeEnvIdentity`]. A wholly literal region still produces the
//! `Static` arm of [`SourcedShape`] by that type's own normalization; a
//! symbolic axis takes the same constructor.
//!
//! The registry is the authority over whether the region's operations admit
//! those operands. Governed elementwise families decide symbol-involving
//! equality through the environment's own proof. A family that still declines
//! symbolic operands — today the strict serial sum — surfaces as
//! [`RegionError::Program`], not as a silent deferral. The shape that program
//! *infers* for the result is required to equal the shape this module
//! *derived*: the authority decides, and the derivation is checked against it.
//!
//! What does not change is the runtime contract: the emitted facts check every
//! operand's rank and stored scalar, unify every symbol, and construct the
//! declared result, symbolic or not. Delivering an artifact family from a
//! symbolic program is a later ticket.
//!
//! # The compiler is not invoked *here*
//!
//! This module stops at a verified [`SemanticProgram`] and hands it on;
//! [`crate::aot`] is what carries it into `tiler_compiler::session` and the
//! offline Metal driver, and only for a region whose `deliver` statement
//! selected an artifact family. The separation is the reason the program is
//! *carried* out of [`ProgramEvidence::Verified`] rather than rebuilt there.
//!
//! It could not be invoked at all until recently, and the reason was measured at
//! `b623670`: the approved region over three `f32[4]` inputs was refused by
//! `compile_governed` under every named `NumericalContract` value with
//! `UnsupportedCapability { rule: "signature" }` before any target-qualified
//! trace, because both recognized program shapes opened with
//! `program.input_count() != 1`. That refusal is gone — the scheduled-region
//! vocabulary now names which input tensor each access and each scalar leaf
//! reads — and `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs`
//! carries the executable statement of the transition. What remains
//! contract-dependent is `RelaxedF32`, which declines any fused
//! multiply-then-add body at any input count — and a region may now state it,
//! because [`crate::numerics`] decides which contracts are *nameable* and never
//! which the target honours. Such a region is refused downstream, with the
//! compiler's own reason, rather than here.
//!
//! [`AvailabilityPhase::LiveDevicePreflight`]: tiler_ir::program::abi::AvailabilityPhase::LiveDevicePreflight
//! [`multiply_f32_op`]: tiler_ir::semantic::multiply_f32_op
//! [`add_f32_op`]: tiler_ir::semantic::add_f32_op
//! [`strict_serial_sum_f32_op`]: tiler_ir::semantic::strict_serial_sum_f32_op
//! [`constant_f32_op`]: tiler_ir::semantic::constant_f32_op
//! [`SourcedExtent`]: tiler_ir::shape::SourcedExtent
//! [`SourcedShape`]: tiler_ir::shape::SourcedShape
//! [`ShapeEnv`]: tiler_ir::shape::ShapeEnv
//! [`ShapeEnvIdentity`]: tiler_ir::shape::ShapeEnvIdentity
//! [`BoundRegion`]: crate::binding::BoundRegion

use core::fmt;
use std::sync::Arc;

use tiler_ir::program::StorageScalar;
use tiler_ir::semantic::{
    BuildError, F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum, Value,
};
use tiler_ir::shape::{Axis, Extent, ShapeEnv, SourcedExtent};

use crate::binding::{BoundRegion, DeclaredAxis, RegionBindError, RegionDeclarations};
use crate::grammar::{
    AxisExtentSyntax, AxisSyntax, Expression, Name, OperandSyntax, Operator, RegionSyntax,
    ScalarSyntax, SyntaxError,
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

/// The element type a scalar constant written in a region body has.
///
/// Stated rather than inferred from the operands it sits beside: the registry
/// registers one scalar-constant operation and it is `f32`, so `2.0` is an
/// `f32` constant and there is nothing else it could be. Widening
/// [`ELEMENT_TYPES`] would make that a real question — what `2.0` means beside a
/// non-`f32` operand — and that is a syntax decision for Tom rather than an
/// inference this module may quietly start making.
const SCALAR_CONSTANT_TYPE: StorageScalar = StorageScalar::F32;

/// Whether an expansion could construct the region's public logical program.
///
/// The verified variant *carries* the program rather than merely reporting that
/// one existed, because the program is what [`crate::aot`] compiles: an
/// expansion that had to rebuild it from the syntax a second time would be a
/// second constructor for one subject, and the artifact identity would then name
/// whichever of the two the driver happened to be handed.
#[derive(Clone, Debug)]
pub(crate) enum ProgramEvidence {
    /// The region was constructed as a [`SemanticProgram`] through the governed
    /// registry, verified, and its inferred result shape checked against the
    /// derived one. Symbolic extents are carried as sourced shapes; a family
    /// that still declines them is [`RegionError::Program`], not a second
    /// variant here.
    Verified(SemanticProgram),
}

impl ProgramEvidence {
    /// Returns the verified public logical program.
    pub(crate) const fn verified(&self) -> &SemanticProgram {
        match self {
            Self::Verified(program) => program,
        }
    }
}

/// One region, lowered to what an expansion emits.
///
/// Not `Eq`/`PartialEq`: a verified [`SemanticProgram`] is neither, and giving
/// this record an equality that skipped it would compare two expansions as
/// equal while they carried different programs.
#[derive(Clone, Debug)]
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
    /// An operation's operands cannot be combined because their stored scalars
    /// differ. Shape agreement is the registry's, and arrives as [`Self::Program`].
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
    /// A scalar literal is not a value this profile's constant can take.
    MalformedScalarConstant {
        /// The number as the region spells it.
        text: String,
        /// The literal token.
        span: S,
    },
    /// A reduction names an axis the expression it reduces does not have.
    UnknownReducedAxis {
        /// The name as written.
        name: String,
        /// The axis names that expression does have, rendered.
        available: String,
        /// The name's token.
        span: S,
    },
    /// A reduction names an axis that more than one axis answers to.
    ///
    /// `f32[n, n]` is a legal square shape, so the ambiguity is refused where it
    /// is *used* rather than where it is declared: nothing is wrong with the
    /// declaration until something has to pick one of the two axes.
    AmbiguousReducedAxis {
        /// The name as written.
        name: String,
        /// The name's token.
        span: S,
    },
    /// One reduction names one axis twice.
    RepeatedReducedAxis {
        /// The name as written.
        name: String,
        /// The second mention.
        span: S,
    },
    /// Two operands give one axis position two different names.
    ///
    /// Refused rather than resolved in favour of a side, because which operand
    /// was written first is not a fact about the computation: taking the left
    /// name would make `a + b` and `b + a` denote results whose axes reduce
    /// under different spellings.
    ConflictingAxisNames {
        /// The axis position the two operands disagree about.
        position: usize,
        /// The name the left operand gives it.
        left: String,
        /// The name the right operand gives it.
        right: String,
        /// The operator token that combined them.
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
            Self::MalformedScalarConstant { text, .. } => write!(
                formatter,
                "`{text}` is not a value an `f32` constant can take; a region's scalar constant is a \
                 finite number, and a literal that rounds to an infinity would compute something \
                 other than what it spells"
            ),
            Self::UnknownReducedAxis {
                name, available, ..
            } => write!(
                formatter,
                "`{name}` is not a named axis of the expression this reduction sums, which has \
                 {available}; an axis is named by the shape that declares it, so write `{name}` as \
                 one of that operand's axes, as in `f32[{name}: 8]`"
            ),
            Self::AmbiguousReducedAxis { name, .. } => write!(
                formatter,
                "`{name}` names more than one axis of the expression this reduction sums, so which \
                 axis to sum is not decided; give those axes distinct names in the shape that \
                 declares them, as in `f32[{name}: 8, other: 8]`"
            ),
            Self::RepeatedReducedAxis { name, .. } => write!(
                formatter,
                "`{name}` is named twice by one reduction; a reduction sums each axis once, and \
                 summing one twice is not a computation this vocabulary carries"
            ),
            Self::ConflictingAxisNames {
                position,
                left,
                right,
                ..
            } => write!(
                formatter,
                "axis {position} is named `{left}` by one operand of this operation and `{right}` \
                 by the other, so the axis its result exposes has no one name; name it the same \
                 way in both shapes, or leave it unnamed in one of them"
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
            | Self::MalformedScalarConstant { span, .. }
            | Self::UnknownReducedAxis { span, .. }
            | Self::AmbiguousReducedAxis { span, .. }
            | Self::RepeatedReducedAxis { span, .. }
            | Self::ConflictingAxisNames { span, .. }
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

/// One axis of a resolved value: its extent, and what it is called here.
///
/// The name is carried beside the extent rather than inside [`DeclaredAxis`]
/// because it reaches nothing the binding module emits: an axis name decides
/// which position a reduction removes at expansion time, and the runtime facts
/// describe extents. Putting it in the emitted vocabulary would publish a
/// spelling that nothing at run time can act on.
#[derive(Clone)]
struct ResolvedAxis<S> {
    declared: DeclaredAxis<S>,
    name: Option<Name<S>>,
}

/// One operand after its element type and axes are resolved.
struct ResolvedOperand<S> {
    key: InputKey,
    storage_scalar: StorageScalar,
    axes: Vec<ResolvedAxis<S>>,
    name: Name<S>,
}

/// One subexpression's declared element type and shape.
struct ResolvedValue<S> {
    storage_scalar: StorageScalar,
    axes: Vec<ResolvedAxis<S>>,
}

/// One subexpression with every name already resolved to what it refers to.
///
/// Built once, beside the shape derivation that needed the same resolution, and
/// then *applied*. The alternative — walking the syntax a second time to
/// construct the program — makes two resolvers for one subject, and they would
/// have to agree about which operand a name refers to and which position a
/// reduced axis sits at. Where they disagreed, the derived shape and the
/// constructed program would describe different computations.
enum ResolvedExpression<S> {
    /// A declared operand, by its position in the region's interface.
    ///
    /// The reference is retained beside the position so the one refusal this
    /// carries — a position with no value, which nothing constructible reaches —
    /// still names the token a consumer wrote.
    Operand { position: usize, name: Name<S> },
    /// A scalar constant, by its exact binary32 payload.
    Constant { bits: u32, span: S },
    /// A strict serial sum over resolved axis positions, strictly ascending.
    Reduction {
        span: S,
        operand: Box<ResolvedExpression<S>>,
        axes: Vec<usize>,
    },
    /// One registered binary operation.
    Binary {
        operator: Operator,
        span: S,
        left: Box<ResolvedExpression<S>>,
        right: Box<ResolvedExpression<S>>,
    },
}

/// One subexpression and the value it denotes.
struct Resolved<S> {
    expression: ResolvedExpression<S>,
    value: ResolvedValue<S>,
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
            declared_axes(&resolved.axes),
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
        result.value.storage_scalar,
        declared_axes(&result.value.axes),
        syntax.out,
    )?;

    let bound: BoundRegion = declarations.bind()?;
    let environment = bound.environment_arc();
    let program = verify_public_logical_program(syntax, &operands, &result, &environment)?;

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
        let DeclaredAxis::Symbol { name, span } = &axis.declared else {
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
        axes: operand.axes.iter().map(resolve_axis).collect(),
        name: operand.name.clone(),
    })
}

/// Resolves one declared axis's extent and the name it is known by.
fn resolve_axis<S: Copy>(axis: &AxisSyntax<S>) -> ResolvedAxis<S> {
    ResolvedAxis {
        declared: match &axis.extent {
            AxisExtentSyntax::Symbol(name) => DeclaredAxis::Symbol {
                name: name.text.clone(),
                span: name.span,
            },
            AxisExtentSyntax::Literal { value, .. } => DeclaredAxis::Literal(*value),
        },
        name: axis.name().cloned(),
    }
}

/// Resolves one subexpression's names, element type, and shape.
fn resolve_expression<S: Copy>(
    expression: &Expression<S>,
    operands: &[ResolvedOperand<S>],
) -> Result<Resolved<S>, RegionError<S>> {
    match expression {
        Expression::Operand(name) => operands
            .iter()
            .position(|operand| operand.name.text == name.text)
            .map(|position| Resolved {
                expression: ResolvedExpression::Operand {
                    position,
                    name: name.clone(),
                },
                value: ResolvedValue {
                    storage_scalar: operands[position].storage_scalar,
                    axes: operands[position].axes.clone(),
                },
            })
            .ok_or_else(|| RegionError::UnknownOperand {
                name: name.text.clone(),
                span: name.span,
            }),
        Expression::Scalar(scalar) => Ok(Resolved {
            expression: ResolvedExpression::Constant {
                bits: scalar_constant_bits(scalar)?,
                span: scalar.span,
            },
            value: ResolvedValue {
                storage_scalar: SCALAR_CONSTANT_TYPE,
                // Rank 0, which is what makes a constant the scalar side of the
                // registry's own elementwise rule rather than a value this
                // module broadcasts itself.
                axes: Vec::new(),
            },
        }),
        Expression::Reduction {
            span,
            operand,
            axes,
        } => {
            let reduced = resolve_expression(operand, operands)?;
            let positions = reduced_positions(&reduced.value.axes, axes)?;
            Ok(Resolved {
                value: ResolvedValue {
                    storage_scalar: reduced.value.storage_scalar,
                    axes: reduced
                        .value
                        .axes
                        .iter()
                        .enumerate()
                        .filter(|(position, _)| !positions.contains(position))
                        .map(|(_, axis)| axis.clone())
                        .collect(),
                },
                expression: ResolvedExpression::Reduction {
                    span: *span,
                    operand: Box::new(reduced.expression),
                    axes: positions,
                },
            })
        }
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
            if left.value.storage_scalar != right.value.storage_scalar {
                return Err(RegionError::IncompatibleOperandShapes {
                    operator: operator.as_str(),
                    left: rendered_axes(&left.value.axes),
                    right: rendered_axes(&right.value.axes),
                    span: *span,
                });
            }
            Ok(Resolved {
                value: ResolvedValue {
                    storage_scalar: left.value.storage_scalar,
                    axes: elementwise_axes(*operator, &left.value.axes, &right.value.axes, *span)?,
                },
                expression: ResolvedExpression::Binary {
                    operator: *operator,
                    span: *span,
                    left: Box::new(left.expression),
                    right: Box::new(right.expression),
                },
            })
        }
    }
}

/// Reads one scalar literal as the exact binary32 payload it denotes.
///
/// The rounding is Rust's own — the text is the text a consumer would have
/// written in an `f32` literal beside the region, and it reaches `f32` through
/// the same correctly-rounded parse. A value the format cannot hold is refused
/// rather than saturated, matching what rustc does with `1e40f32`: a constant
/// that silently became an infinity would decide what a kernel computes.
fn scalar_constant_bits<S: Copy>(scalar: &ScalarSyntax<S>) -> Result<u32, RegionError<S>> {
    let malformed = || RegionError::MalformedScalarConstant {
        text: scalar.text.clone(),
        span: scalar.span,
    };
    let value: f32 = scalar.text.parse().map_err(|_| malformed())?;
    if !value.is_finite() {
        return Err(malformed());
    }
    Ok(value.to_bits())
}

/// Resolves a reduction's named axes to positions, in canonical ascending order.
///
/// Ascending because *which* axes are summed is what the region means and the
/// order they were written in is not — `[cols, rows]` and `[rows, cols]` denote
/// one computation, and the registry's own canonical form is ascending, so
/// sorting here is what keeps two spellings from becoming two programs.
fn reduced_positions<S: Copy>(
    axes: &[ResolvedAxis<S>],
    named: &[Name<S>],
) -> Result<Vec<usize>, RegionError<S>> {
    let mut positions: Vec<usize> = Vec::with_capacity(named.len());
    for wanted in named {
        let mut matched = axes.iter().enumerate().filter(|(_, axis)| {
            axis.name
                .as_ref()
                .is_some_and(|name| name.text == wanted.text)
        });
        let Some((position, _)) = matched.next() else {
            return Err(RegionError::UnknownReducedAxis {
                name: wanted.text.clone(),
                available: rendered_axis_names(axes),
                span: wanted.span,
            });
        };
        if matched.next().is_some() {
            return Err(RegionError::AmbiguousReducedAxis {
                name: wanted.text.clone(),
                span: wanted.span,
            });
        }
        if positions.contains(&position) {
            return Err(RegionError::RepeatedReducedAxis {
                name: wanted.text.clone(),
                span: wanted.span,
            });
        }
        positions.push(position);
    }
    positions.sort_unstable();
    Ok(positions)
}

/// Renders the axis names a diagnostic can offer for a value.
fn rendered_axis_names<S>(axes: &[ResolvedAxis<S>]) -> String {
    let named: Vec<String> = axes
        .iter()
        .filter_map(|axis| axis.name.as_ref())
        .map(|name| format!("`{}`", name.text))
        .collect();
    if named.is_empty() {
        "no named axis".to_owned()
    } else {
        format!("the axes {}", named.join(", "))
    }
}

/// Returns the extents of a resolved shape, which is what the binding module
/// takes.
fn declared_axes<S: Clone>(axes: &[ResolvedAxis<S>]) -> Vec<DeclaredAxis<S>> {
    axes.iter().map(|axis| axis.declared.clone()).collect()
}

/// Derives named result axes for the binding facts. Shape *agreement* is not
/// decided here.
///
/// The registry's `elementwise_binary_shape` is the authority over whether two
/// operands may combine: it already decides rank, literal inequality, scalar
/// broadcast, and `proves_equal` for symbols, and it is the rule a constructed
/// program will run. This function used to restate that rule so a symbolic
/// region — which never reached the registry — was still refused. Once the
/// region is constructed through the registry, a second independent refusal
/// would be a second authority.
///
/// What remains here is only what the registry does not know: axis-name union,
/// so `a + b` and `b + a` expose the same named axes, and
/// [`RegionError::ConflictingAxisNames`]. When the operands do not agree, the
/// left axes are kept as a placeholder so binding can proceed; `verify` then
/// reports the registry's own refusal as [`RegionError::Program`].
fn elementwise_axes<S: Copy>(
    _operator: Operator,
    left: &[ResolvedAxis<S>],
    right: &[ResolvedAxis<S>],
    span: S,
) -> Result<Vec<ResolvedAxis<S>>, RegionError<S>> {
    if left.is_empty() {
        return Ok(right.to_vec());
    }
    if right.is_empty() {
        return Ok(left.to_vec());
    }
    if !axes_agree(left, right) {
        return Ok(left.to_vec());
    }

    let mut merged = Vec::with_capacity(left.len());
    for (position, (left_axis, right_axis)) in left.iter().zip(right).enumerate() {
        let name = match (&left_axis.name, &right_axis.name) {
            (Some(left_name), Some(right_name)) if left_name.text != right_name.text => {
                return Err(RegionError::ConflictingAxisNames {
                    position,
                    left: left_name.text.clone(),
                    right: right_name.text.clone(),
                    span,
                });
            }
            (Some(name), _) | (None, Some(name)) => Some(name.clone()),
            (None, None) => None,
        };
        merged.push(ResolvedAxis {
            declared: left_axis.declared.clone(),
            name,
        });
    }
    Ok(merged)
}

/// Reports whether two resolved shapes have the same extents, spans aside.
///
/// A span is where a name was written, not part of what it means, so comparing
/// [`DeclaredAxis`] values directly would make `f32[n] * f32[n]` depend on which
/// tokens spelled the two `n`s. An axis *name* is not compared either: what two
/// operands must agree about to be combined is their extents, and disagreeing
/// names are a separate refusal with a separate reason.
fn axes_agree<S>(left: &[ResolvedAxis<S>], right: &[ResolvedAxis<S>]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| match (&left.declared, &right.declared) {
                (DeclaredAxis::Literal(left), DeclaredAxis::Literal(right)) => left == right,
                (
                    DeclaredAxis::Symbol { name: left, .. },
                    DeclaredAxis::Symbol { name: right, .. },
                ) => left == right,
                _ => false,
            })
}

/// Renders one resolved shape's extents the way a region spells them.
fn rendered_axes<S>(axes: &[ResolvedAxis<S>]) -> String {
    let rendered = axes
        .iter()
        .map(|axis| match &axis.declared {
            DeclaredAxis::Literal(extent) => extent.to_string(),
            DeclaredAxis::Symbol { name, .. } => name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{rendered}]")
}

/// Builds one operand or result boundary from the region's own axes.
///
/// Literal and symbolic axes take this one path. [`SourcedShape`](tiler_ir::shape::SourcedShape)
/// normalizes an all-literal vector to the `Static` arm, so a wholly literal
/// region is unchanged by construction rather than by a branch.
fn sourced_extents<S: Copy>(
    axes: &[ResolvedAxis<S>],
    environment: &ShapeEnv,
) -> Result<Vec<SourcedExtent>, RegionError<S>> {
    axes.iter()
        .map(|axis| match &axis.declared {
            DeclaredAxis::Literal(extent) => Ok(SourcedExtent::Static(Extent::new(*extent))),
            DeclaredAxis::Symbol { name, span } => environment
                .bindings()
                .find_map(|(symbol, _)| (symbol.name() == name).then(|| symbol.clone()))
                .map(SourcedExtent::Symbol)
                .ok_or_else(|| RegionError::Program {
                    span: *span,
                    detail: format!(
                        "declared symbol `{name}` is missing from the bound environment"
                    ),
                }),
        })
        .collect()
}

/// Renders sourced extents the way a region spells them: a literal as its
/// integer, a symbol as the name the region wrote, never the scoped encoding.
fn rendered_sourced_extents(extents: &[SourcedExtent]) -> String {
    let rendered = extents
        .iter()
        .map(|extent| {
            if let Some(literal) = extent.as_static() {
                literal.get().to_string()
            } else if let Some(symbol) = extent.symbol() {
                symbol.name().to_owned()
            } else {
                extent.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{rendered}]")
}

/// Constructs and verifies the region as a public logical program.
///
/// Hands [`BoundRegion`]'s environment to the builder rather than constructing
/// a second one. Operand shapes are sourced extents built from [`DeclaredAxis`],
/// so a literal axis and a symbolic axis take one path. A family that declines
/// those operands is [`RegionError::Program`].
fn verify_public_logical_program<S: Copy>(
    syntax: &RegionSyntax<S>,
    operands: &[ResolvedOperand<S>],
    result: &Resolved<S>,
    environment: &Arc<ShapeEnv>,
) -> Result<ProgramEvidence, RegionError<S>> {
    let refused = |span: S| {
        move |source: BuildError| RegionError::Program {
            span,
            detail: source.to_string(),
        }
    };

    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(Arc::clone(environment))
            .map_err(|source| RegionError::Program {
                span: syntax.region,
                detail: source.to_string(),
            })?;

    let mut values: Vec<Value<F32>> = Vec::with_capacity(operands.len());
    for operand in operands {
        let extents = sourced_extents(&operand.axes, environment)?;
        let value = builder
            .input_sourced::<F32>(operand.key.clone(), extents)
            .map_err(refused(operand.name.span))?;
        values.push(value);
    }

    let derived = sourced_extents(&result.value.axes, environment)?;

    let root = apply_expression(&mut builder, &result.expression, &values)?;
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
    // trusted beside it. Both sides are sourced extents; a wholly literal
    // region compares as the `Static` arm on each side by normalization.
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
    let inferred_extents: Vec<SourcedExtent> = inferred.extents().collect();
    if inferred_extents != derived {
        return Err(RegionError::ResultShapeDisagreement {
            derived: rendered_axes(&result.value.axes),
            inferred: rendered_sourced_extents(&inferred_extents),
            span: syntax.out,
        });
    }

    Ok(ProgramEvidence::Verified(program))
}

/// Applies one resolved subexpression through the governed operation facades.
///
/// A translation and not a second resolver: every name was decided by
/// [`resolve_expression`], so an operand is an index into `values` and a reduced
/// axis is a position. The only refusals it can produce are the authority's own.
fn apply_expression<S: Copy>(
    builder: &mut SemanticProgramBuilder,
    expression: &ResolvedExpression<S>,
    values: &[Value<F32>],
) -> Result<Value<F32>, RegionError<S>> {
    match expression {
        // The `ok_or_else` arm is unreachable: the position came from
        // `operands.iter().position(…)` over the same list `values` was built
        // from, one value per operand. It is a refusal rather than an index
        // panic anyway, because a panic inside an expansion aborts rustc with no
        // span at all — and it is *this* refusal because "the position names no
        // operand" is what a name that did not resolve would have meant.
        ResolvedExpression::Operand { position, name } => values
            .get(*position)
            .copied()
            .ok_or_else(|| RegionError::UnknownOperand {
                name: name.text.clone(),
                span: name.span,
            }),
        ResolvedExpression::Constant { bits, span } => {
            F32Constant::apply(builder, *bits).map_err(|source| RegionError::Program {
                span: *span,
                detail: source.to_string(),
            })
        }
        ResolvedExpression::Reduction {
            span,
            operand,
            axes,
        } => {
            let operand = apply_expression(builder, operand, values)?;
            let axes: Vec<Axis> = axes
                .iter()
                .map(|position| {
                    u32::try_from(*position)
                        .map(Axis::new)
                        .map_err(|_| RegionError::Program {
                            span: *span,
                            detail: "an axis position exceeds what an axis index addresses"
                                .to_owned(),
                        })
                })
                .collect::<Result<_, _>>()?;
            StrictSerialF32Sum::apply(builder, operand, axes).map_err(|source| {
                RegionError::Program {
                    span: *span,
                    detail: source.to_string(),
                }
            })
        }
        ResolvedExpression::Binary {
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
