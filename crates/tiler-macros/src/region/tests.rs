//! What a parsed region means, decided from both sides.
//!
//! The syntax is built directly rather than parsed, so a failure here is about
//! meaning rather than about the grammar; `crate::grammar::tests` is where the
//! text is decided. Spans are integers, for the reason stated there.

use crate::grammar::{
    AxisExtentSyntax, AxisSyntax, Expression, Name, OperandSyntax, Operator, RegionSyntax,
    ScalarSyntax,
};

use super::{RegionError, lower};

/// A span a test can construct and assert on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct At(u32);

/// The span a refusal naming no single token is reported at.
const REGION: At = At(0);

fn name(text: &str, at: u32) -> Name<At> {
    Name {
        text: text.to_owned(),
        span: At(at),
    }
}

fn symbol_axis(text: &str, at: u32) -> AxisSyntax<At> {
    AxisSyntax {
        label: None,
        extent: AxisExtentSyntax::Symbol(name(text, at)),
    }
}

fn literal_axis(value: u64, at: u32) -> AxisSyntax<At> {
    AxisSyntax {
        label: None,
        extent: AxisExtentSyntax::Literal {
            value,
            span: At(at),
        },
    }
}

/// A literal-extent axis under a name, which is what makes it reducible.
fn named_axis(label: &str, value: u64, at: u32) -> AxisSyntax<At> {
    AxisSyntax {
        label: Some(name(label, at)),
        ..literal_axis(value, at + 1)
    }
}

fn scalar(text: &str, at: u32) -> Expression<At> {
    Expression::Scalar(ScalarSyntax {
        text: text.to_owned(),
        span: At(at),
    })
}

fn reduction(at: u32, operand: Expression<At>, axes: &[(&str, u32)]) -> Expression<At> {
    Expression::Reduction {
        span: At(at),
        operand: Box::new(operand),
        axes: axes.iter().map(|(text, at)| name(text, *at)).collect(),
    }
}

fn operand(key: &str, at: u32, dtype: &str, axes: Vec<AxisSyntax<At>>) -> OperandSyntax<At> {
    OperandSyntax {
        name: name(key, at),
        dtype: name(dtype, at + 1),
        axes,
    }
}

fn reference(key: &str, at: u32) -> Expression<At> {
    Expression::Operand(name(key, at))
}

fn binary(
    operator: Operator,
    at: u32,
    left: Expression<At>,
    right: Expression<At>,
) -> Expression<At> {
    Expression::Binary {
        operator,
        span: At(at),
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// `(a * b) + c` over three like-shaped operands.
fn approved_body() -> Expression<At> {
    binary(
        Operator::Add,
        50,
        binary(
            Operator::Multiply,
            51,
            reference("a", 52),
            reference("b", 53),
        ),
        reference("c", 54),
    )
}

/// The approved region, with each operand's axes supplied by the caller.
fn approved_region(axes: impl Fn() -> Vec<AxisSyntax<At>>) -> RegionSyntax<At> {
    RegionSyntax {
        region: REGION,
        symbols: vec![name("n", 1)],
        operands: vec![
            operand("a", 10, "f32", axes()),
            operand("b", 20, "f32", axes()),
            operand("c", 30, "f32", axes()),
        ],
        // A region's delivery policy decides nothing about what it means, which
        // is why every case in this file states none.
        delivery: None,
        out: At(40),
        body: approved_body(),
    }
}

/// The approved example as Tom approved it: one symbolic extent.
fn symbolic_region() -> RegionSyntax<At> {
    approved_region(|| vec![symbol_axis("n", 12)])
}

/// The same region with its extent fixed, which is what makes the public
/// logical program representable.
fn static_region() -> RegionSyntax<At> {
    RegionSyntax {
        symbols: Vec::new(),
        ..approved_region(|| vec![literal_axis(4, 12)])
    }
}

/// The approved region lowers to facts that bind, and defers its program
/// because a symbolic extent has no fixed shape to give the semantic layer.
#[test]
fn the_approved_region_lowers_and_defers_its_public_logical_program() {
    let expansion = lower(&symbolic_region()).expect("the approved region lowers");

    assert!(expansion.program.verified().is_none());
    assert_eq!(
        expansion
            .operands
            .iter()
            .map(|operand| operand.text.as_str())
            .collect::<Vec<_>>(),
        ["a", "b", "c"],
        "generated code supplies operands in the interface order the facts declare",
    );
    assert_eq!(
        expansion.operands[0].span,
        At(10),
        "each operand reference carries the token that named it",
    );

    // `n` sources from `a` axis 0 and obliges the other two, and the result is
    // `f32[n]`.
    assert!(
        expansion.facts.contains(
            "symbols: &[::tiler::__private::SymbolFacts { name: \"n\", source: \
             ::tiler::__private::AxisRef { operand: 0usize, axis: 0usize }, obligations: \
             &[::tiler::__private::AxisRef { operand: 1usize, axis: 0usize }, \
             ::tiler::__private::AxisRef { operand: 2usize, axis: 0usize }] }]"
        ),
        "{}",
        expansion.facts,
    );
    assert!(
        expansion.facts.contains(
            "result: ::tiler::__private::ResultFacts { key: \"out\", storage_scalar: \
             ::tiler::value::StorageScalar::F32, axes: \
             &[::tiler::__private::ResultAxis::Symbol(0usize)] }"
        ),
        "{}",
        expansion.facts,
    );
}

/// A region whose every extent is a literal is constructed and verified as a
/// public logical program through the governed registry.
#[test]
fn a_static_region_is_constructed_as_a_public_logical_program() {
    let expansion = lower(&static_region()).expect("the static region lowers");
    assert!(expansion.program.verified().is_some());
    assert!(
        expansion
            .facts
            .contains("axes: &[::tiler::__private::ResultAxis::Literal(4u64)]"),
        "{}",
        expansion.facts,
    );

    // The operand's literal extent reaches the facts too, and it has to: no
    // symbol names this axis, so a runtime check has nothing else to compare the
    // supplied value against.
    assert!(
        expansion.facts.contains(
            "::tiler::__private::OperandFacts { key: \"a\", storage_scalar: \
             ::tiler::value::StorageScalar::F32, extents: \
             &[::tiler::__private::OperandExtent::Literal(4u64)] }"
        ),
        "{}",
        expansion.facts,
    );

    // The paired neighbour: one symbolic axis is enough to defer it, so
    // `Verified` is a claim about representability rather than about this
    // fixture happening to be simple.
    assert!(
        lower(&approved_region(|| vec![symbol_axis("n", 12)]))
            .expect("the region lowers")
            .program
            .verified()
            .is_none(),
    );
}

/// An element type this profile does not register is refused at the element
/// type's own token — the granularity Tom's decision turned on.
#[test]
fn an_unregistered_element_type_is_refused_at_its_own_token() {
    for spelling in ["u8", "f64", "f16", "i32", "float"] {
        let mut region = static_region();
        region.operands[1].dtype = name(spelling, 21);
        assert_eq!(
            lower(&region).expect_err("the element type is not registered"),
            RegionError::UnknownElementType {
                name: spelling.to_owned(),
                span: At(21),
            },
            "`{spelling}` must be refused at its own token",
        );
    }

    // The accepting neighbour differs only in the element type.
    lower(&static_region()).expect("`f32` is registered");
}

/// A body reference to a name no `in` statement declares is refused at the
/// reference, not at the region.
#[test]
fn an_undeclared_operand_reference_is_refused_at_the_reference() {
    let mut region = static_region();
    region.body = binary(
        Operator::Add,
        50,
        reference("a", 52),
        reference("missing", 53),
    );
    assert_eq!(
        lower(&region).expect_err("`missing` is not declared"),
        RegionError::UnknownOperand {
            name: "missing".to_owned(),
            span: At(53),
        },
    );
}

/// Operand shapes must match or one operand must be scalar — the registry's own
/// rule, refused at the operator that would have combined them.
#[test]
fn incompatible_operand_shapes_are_refused_at_the_operator() {
    // Different literal extents.
    let mut region = static_region();
    region.operands[1].axes = vec![literal_axis(5, 22)];
    assert_eq!(
        lower(&region).expect_err("`4` and `5` are not one shape"),
        RegionError::IncompatibleOperandShapes {
            operator: "*",
            left: "[4]".to_owned(),
            right: "[5]".to_owned(),
            span: At(51),
        },
    );

    // Different ranks.
    let mut region = static_region();
    region.operands[1].axes = vec![literal_axis(4, 22), literal_axis(4, 23)];
    assert!(matches!(
        lower(&region).expect_err("rank 1 and rank 2 are not one shape"),
        RegionError::IncompatibleOperandShapes { span: At(51), .. }
    ));

    // Two *different symbols* are not one shape either: nothing at expansion
    // time proves `n` and `m` take one value, and treating them as compatible
    // would defer a shape error into a wrong result.
    let mut region = symbolic_region();
    region.symbols.push(name("m", 2));
    region.operands[1].axes = vec![symbol_axis("m", 22)];
    assert_eq!(
        lower(&region).expect_err("`n` and `m` are not proved equal"),
        RegionError::IncompatibleOperandShapes {
            operator: "*",
            left: "[n]".to_owned(),
            right: "[m]".to_owned(),
            span: At(51),
        },
    );

    // The accepting neighbour: one symbol named by both.
    lower(&symbolic_region()).expect("one symbol names both axes");
}

/// A scalar operand broadcasts, and the result takes the shaped side.
///
/// The registry's rule again, and the reason the shape check is not simply
/// "equal": `F32Multiply` admits a rank-0 operand against any shape.
#[test]
fn a_scalar_operand_broadcasts_and_the_result_takes_the_shaped_side() {
    let mut region = static_region();
    region.operands[1].axes = Vec::new();
    let expansion = lower(&region).expect("a scalar operand broadcasts");
    assert!(expansion.program.verified().is_some());
    assert!(
        expansion
            .facts
            .contains("axes: &[::tiler::__private::ResultAxis::Literal(4u64)]"),
        "{}",
        expansion.facts,
    );
    assert!(
        expansion.facts.contains(
            "::tiler::__private::OperandFacts { key: \"b\", storage_scalar: \
                      ::tiler::value::StorageScalar::F32, extents: &[] }"
        ),
        "{}",
        expansion.facts,
    );

    // A scalar on the left is the mirror case, and the result is still shaped.
    let mut region = static_region();
    region.operands[0].axes = Vec::new();
    assert!(
        lower(&region)
            .expect("a scalar left operand broadcasts")
            .facts
            .contains("axes: &[::tiler::__private::ResultAxis::Literal(4u64)]"),
    );

    // Every operand scalar is a rank-0 region, which is admitted too.
    let mut region = static_region();
    for operand in &mut region.operands {
        operand.axes = Vec::new();
    }
    let expansion = lower(&region).expect("a rank-0 region is admitted");
    assert!(expansion.program.verified().is_some());
    assert!(expansion.facts.contains("axes: &[]"), "{}", expansion.facts);
}

/// A region declaring no operand is refused: nothing sources a symbol, and no
/// context exists to construct a result from.
#[test]
fn a_region_without_operands_is_refused() {
    let region = RegionSyntax {
        region: REGION,
        symbols: Vec::new(),
        operands: Vec::new(),
        delivery: None,
        out: At(40),
        body: reference("a", 52),
    };
    // The body is resolved first, so the missing operand is what is named.
    assert_eq!(
        lower(&region).expect_err("nothing supplies a value"),
        RegionError::UnknownOperand {
            name: "a".to_owned(),
            span: At(52),
        },
    );
}

/// A declared symbol no operand axis names is refused at its declaration, by
/// the binding module rather than by a second rule here.
#[test]
fn an_unsourced_symbol_is_refused_by_the_binding_authority() {
    let mut region = symbolic_region();
    region.symbols.push(name("m", 2));
    assert!(matches!(
        lower(&region).expect_err("`m` has no source"),
        RegionError::Binding(crate::binding::RegionBindError::UnboundSymbol { span: At(2), .. })
    ));
}

/// An axis naming a symbol no `sym` declared is refused at that axis.
#[test]
fn an_undeclared_symbol_is_refused_at_the_axis_that_names_it() {
    let mut region = symbolic_region();
    region.operands[2].axes = vec![symbol_axis("k", 32)];
    assert!(matches!(
        lower(&region).expect_err("`k` was never declared"),
        RegionError::Binding(crate::binding::RegionBindError::UndeclaredSymbol {
            span: At(32),
            ..
        })
    ));
}

/// Every path a generated fact names resolves through the one crate the
/// consumer declared.
///
/// The accepted inline developer experience forbids a source scan, a consumer
/// `build.rs`, a registry, a prepare step, and runtime source JIT. This checks
/// the emitted text against that list for a region built from real syntax, and
/// names its population — every `::`-rooted path — rather than reporting "no
/// offending token found" for text it failed to scan.
#[test]
fn generated_facts_name_only_the_facade() {
    let facts = lower(&symbolic_region()).expect("the region lowers").facts;

    for forbidden in [
        "include_bytes!",
        "include_str!",
        "include!",
        "std::fs",
        "std::process",
        "std::env",
        "extern crate",
        "::tiler_",
        "compile",
    ] {
        assert!(
            !facts.contains(forbidden),
            "the emitted facts contain `{forbidden}`: {facts}",
        );
    }

    let roots: Vec<&str> = facts
        .match_indices("::")
        .filter(|(at, _)| {
            facts[..*at]
                .chars()
                .next_back()
                .is_none_or(|before| !(before.is_alphanumeric() || before == '_'))
        })
        .map(|(at, _)| {
            let rest = &facts[at.saturating_add(2)..];
            let end = rest
                .find(|character: char| !(character.is_alphanumeric() || character == '_'))
                .unwrap_or(rest.len());
            &rest[..end]
        })
        .collect();
    assert!(
        roots.len() >= 12,
        "the scan found {} rooted paths, too few to have walked these facts: {facts}",
        roots.len(),
    );
    for root in roots {
        assert_eq!(
            root, "tiler",
            "an absolute path in the emitted facts names `{root}` rather than the one crate the \
             consumer declared: {facts}",
        );
    }
}

/// Writing the `in` list in another order moves the emitted operand table and
/// leaves the canonical source where it was.
///
/// The half that makes the symbol binding independent of declaration order is
/// `binding`'s; what this asserts is that lowering from real syntax preserves
/// it, and that the interface order generated code supplies follows the text.
#[test]
fn declaration_order_moves_the_interface_and_not_the_canonical_source() {
    let forward = lower(&symbolic_region()).expect("the region lowers");
    let mut reversed_syntax = symbolic_region();
    reversed_syntax.operands.reverse();
    let reversed = lower(&reversed_syntax).expect("the region lowers");

    assert_eq!(
        reversed
            .operands
            .iter()
            .map(|operand| operand.text.as_str())
            .collect::<Vec<_>>(),
        ["c", "b", "a"],
    );
    assert_ne!(forward.facts, reversed.facts);
    // `a` still sources `n`, and in the reversed interface it is operand 2.
    assert!(
        reversed.facts.contains(
            "source: ::tiler::__private::AxisRef { operand: 2usize, axis: 0usize }, obligations: \
             &[::tiler::__private::AxisRef { operand: 1usize, axis: 0usize }, \
             ::tiler::__private::AxisRef { operand: 0usize, axis: 0usize }]"
        ),
        "{}",
        reversed.facts,
    );
}

/// An operand declared but not used in the body is still part of the interface.
///
/// It has to be: it is supplied at the call site, its rank and stored scalar are
/// checked, and its axes may be what sources a symbol. Dropping it would change
/// the region's interface from the one its text declares.
#[test]
fn an_operand_the_body_does_not_use_stays_in_the_interface() {
    let mut region = symbolic_region();
    region.body = reference("a", 52);
    let expansion = lower(&region).expect("an unused operand is admitted");
    assert_eq!(expansion.operands.len(), 3);
    assert!(
        expansion.facts.contains("key: \"c\""),
        "{}",
        expansion.facts,
    );
}

/// `in x: f32[rows: 2, cols: 2]; out strict_serial_sum(x * 2.0 + 1.0, [cols])`.
///
/// The reduction fixture every case below perturbs, and the whole-program shape
/// the compiler recognizes besides a pointwise chain. Both extents are 2 because
/// the bound declaration's measured grid-axis capacity is four threads, so a
/// wider region would make a reachability test into a capacity test.
fn serial_sum_region() -> RegionSyntax<At> {
    RegionSyntax {
        region: REGION,
        symbols: Vec::new(),
        operands: vec![operand(
            "x",
            10,
            "f32",
            vec![named_axis("rows", 2, 12), named_axis("cols", 2, 14)],
        )],
        delivery: None,
        out: At(40),
        body: reduction(
            50,
            binary(
                Operator::Add,
                51,
                binary(
                    Operator::Multiply,
                    52,
                    reference("x", 53),
                    scalar("2.0", 54),
                ),
                scalar("1.0", 55),
            ),
            &[("cols", 56)],
        ),
    }
}

/// A reduction lowers, verifies, and its result loses exactly the reduced axis.
#[test]
fn a_reduction_region_lowers_and_its_result_loses_the_reduced_axis() {
    let expansion = lower(&serial_sum_region()).expect("the reduction region lowers");
    assert!(expansion.program.verified().is_some());

    // Rank 1 rather than rank 2, and the surviving extent is `rows`.
    assert!(
        expansion
            .facts
            .contains("axes: &[::tiler::__private::ResultAxis::Literal(2u64)] }"),
        "{}",
        expansion.facts,
    );
    // The operand is still rank 2: a reduction changes the result, not the
    // interface the invocation is handed.
    assert!(
        expansion.facts.contains(
            "::tiler::__private::OperandFacts { key: \"x\", storage_scalar: \
             ::tiler::value::StorageScalar::F32, extents: \
             &[::tiler::__private::OperandExtent::Literal(2u64), \
             ::tiler::__private::OperandExtent::Literal(2u64)] }"
        ),
        "{}",
        expansion.facts,
    );

    // Reducing both axes leaves a rank-0 result, which is the registry's own
    // `without_axes` and not a floor this module imposes.
    let mut region = serial_sum_region();
    region.body = reduction(
        50,
        binary(
            Operator::Add,
            51,
            binary(
                Operator::Multiply,
                52,
                reference("x", 53),
                scalar("2.0", 54),
            ),
            scalar("1.0", 55),
        ),
        &[("cols", 56), ("rows", 57)],
    );
    let expansion = lower(&region).expect("reducing every axis is admitted");
    assert!(expansion.program.verified().is_some());
    assert!(
        expansion.facts.contains("axes: &[] }"),
        "{}",
        expansion.facts
    );
}

/// The registry decides the result's shape, and a derivation that disagrees is a
/// typed refusal rather than an expansion that emits a rank the program does not
/// have.
///
/// The perturbation is the one a defect here would actually produce: a reduction
/// resolves an axis *name* to a position, and a name resolved to the wrong
/// position — or to none — leaves the derived result with axes the program
/// removed. It is applied to the derivation directly because that is the value
/// under test; every other case in this file reaches the same function through
/// `lower`, which is what makes the agreeing half below evidence that the check
/// is not simply always failing.
#[test]
fn a_wrong_result_derivation_is_refused_against_the_registry() {
    let syntax = serial_sum_region();
    let operands: Vec<_> = syntax
        .operands
        .iter()
        .map(|operand| super::resolve_operand(operand).expect("the operand resolves"))
        .collect();
    let mut resolved =
        super::resolve_expression(&syntax.body, &operands).expect("the body resolves");

    assert!(
        super::verify_public_logical_program(&syntax, &operands, &resolved)
            .expect("the agreeing derivation verifies")
            .verified()
            .is_some(),
        "the correct derivation must pass, or the refusal below is about nothing",
    );

    // The wrong derivation: the reduced axis was kept.
    resolved.value.axes.clone_from(&operands[0].axes);
    let refusal = super::verify_public_logical_program(&syntax, &operands, &resolved)
        .expect_err("a derivation the registry did not infer is refused");
    let RegionError::ResultShapeDisagreement {
        derived,
        inferred,
        span,
    } = &refusal
    else {
        panic!("unexpected refusal: {refusal:?}");
    };
    assert_eq!(derived, "[2, 2]");
    assert_ne!(derived, inferred);
    assert_eq!(*span, At(40), "the disagreement is reported at `out`");
    assert!(
        refusal.to_string().contains("defect in `tiler-macros`"),
        "the diagnostic must attribute the defect to this crate: {refusal}",
    );
}

/// Every way a reduction can name an axis wrongly is refused at the name.
#[test]
fn a_reduction_axis_name_must_resolve_to_one_axis() {
    let reducing = |axes: &[(&str, u32)]| {
        let mut region = serial_sum_region();
        region.body = reduction(
            50,
            binary(
                Operator::Add,
                51,
                binary(
                    Operator::Multiply,
                    52,
                    reference("x", 53),
                    scalar("2.0", 54),
                ),
                scalar("1.0", 55),
            ),
            axes,
        );
        region
    };

    // A name no axis answers to, and the refusal offers the ones that exist.
    let refusal = lower(&reducing(&[("depth", 56)])).expect_err("`depth` names no axis");
    assert_eq!(
        refusal,
        RegionError::UnknownReducedAxis {
            name: "depth".to_owned(),
            available: "the axes `rows`, `cols`".to_owned(),
            span: At(56),
        },
    );

    // An unnamed axis is not reachable by position: a literal extent with no
    // name has no name.
    let mut unnamed = reducing(&[("cols", 56)]);
    unnamed.operands[0].axes = vec![literal_axis(2, 12), literal_axis(2, 14)];
    assert!(matches!(
        lower(&unnamed).expect_err("an unnamed axis cannot be reduced"),
        RegionError::UnknownReducedAxis { span: At(56), .. }
    ));

    // A name two axes answer to, which `f32[n, n]` is: the shape stays legal and
    // the *use* is what is refused.
    let mut ambiguous = reducing(&[("n", 56)]);
    ambiguous.symbols = vec![name("n", 1)];
    ambiguous.operands[0].axes = vec![symbol_axis("n", 12), symbol_axis("n", 14)];
    assert_eq!(
        lower(&ambiguous).expect_err("`n` names two axes"),
        RegionError::AmbiguousReducedAxis {
            name: "n".to_owned(),
            span: At(56),
        },
    );

    // One axis named twice by one reduction.
    assert_eq!(
        lower(&reducing(&[("cols", 56), ("cols", 57)])).expect_err("`cols` is named twice"),
        RegionError::RepeatedReducedAxis {
            name: "cols".to_owned(),
            span: At(57),
        },
    );

    // The accepting neighbours: one axis, and both axes in either written order,
    // which denote one program because *which* axes are summed is the meaning
    // and the order they were written in is not.
    let one = lower(&reducing(&[("cols", 56)])).expect("one named axis reduces");
    let forward = lower(&reducing(&[("rows", 56), ("cols", 57)])).expect("both axes reduce");
    let reversed = lower(&reducing(&[("cols", 56), ("rows", 57)])).expect("both axes reduce");
    assert!(one.program.verified().is_some());
    assert_eq!(
        forward
            .program
            .verified()
            .map(|program| program.semantic_identity().clone()),
        reversed
            .program
            .verified()
            .map(|program| program.semantic_identity().clone()),
        "two orders of one axis set must denote one program",
    );
}

/// A scalar constant is rank 0, so it broadcasts by the registry's own rule, and
/// a value the format cannot hold is refused rather than saturated.
#[test]
fn a_scalar_constant_is_rank_zero_and_must_be_finite() {
    // Rank 0 against a rank-2 operand: the result takes the shaped side.
    let mut region = serial_sum_region();
    region.body = binary(
        Operator::Multiply,
        52,
        reference("x", 53),
        scalar("2.5", 54),
    );
    let expansion = lower(&region).expect("a constant broadcasts");
    assert!(expansion.program.verified().is_some());
    assert!(
        expansion.facts.contains(
            "axes: &[::tiler::__private::ResultAxis::Literal(2u64), \
             ::tiler::__private::ResultAxis::Literal(2u64)] }"
        ),
        "{}",
        expansion.facts,
    );

    // A literal past the format's range is refused at the literal, matching what
    // rustc does with `1e40f32` rather than silently becoming an infinity.
    let mut region = serial_sum_region();
    region.body = binary(
        Operator::Multiply,
        52,
        reference("x", 53),
        scalar("1e40", 54),
    );
    assert_eq!(
        lower(&region).expect_err("`1e40` is not an `f32`"),
        RegionError::MalformedScalarConstant {
            text: "1e40".to_owned(),
            span: At(54),
        },
    );

    // The accepting neighbour differs only in the exponent.
    let mut region = serial_sum_region();
    region.body = binary(
        Operator::Multiply,
        52,
        reference("x", 53),
        scalar("1e30", 54),
    );
    assert!(
        lower(&region)
            .expect("`1e30` is an `f32`")
            .program
            .verified()
            .is_some(),
    );
}

/// Two operands naming one axis position differently is refused at the operator,
/// rather than resolved in favour of whichever was written first.
#[test]
fn conflicting_axis_names_are_refused_at_the_operator() {
    let mut region = serial_sum_region();
    region.operands.push(operand(
        "y",
        20,
        "f32",
        vec![named_axis("rows", 2, 22), named_axis("depth", 2, 24)],
    ));
    region.body = binary(
        Operator::Multiply,
        52,
        reference("x", 53),
        reference("y", 54),
    );
    assert_eq!(
        lower(&region).expect_err("axis 1 has two names"),
        RegionError::ConflictingAxisNames {
            position: 1,
            left: "cols".to_owned(),
            right: "depth".to_owned(),
            span: At(52),
        },
    );

    // An axis one side leaves unnamed is not a conflict: the name is unioned, so
    // the result exposes it whichever operand was written first.
    let unnamed = |left_first: bool| {
        let mut region = serial_sum_region();
        region.operands.push(operand(
            "y",
            20,
            "f32",
            vec![literal_axis(2, 22), literal_axis(2, 24)],
        ));
        let (left, right) = if left_first {
            (reference("x", 53), reference("y", 54))
        } else {
            (reference("y", 53), reference("x", 54))
        };
        region.body = reduction(
            50,
            binary(
                Operator::Add,
                51,
                binary(Operator::Multiply, 52, left, right),
                scalar("1.0", 55),
            ),
            &[("cols", 56)],
        );
        lower(&region)
            .expect("an unnamed axis takes the other operand's name")
            .facts
    };
    assert_eq!(unnamed(true), unnamed(false));
}

/// A reduction over a symbolic extent lowers and defers its program, for the
/// same reason every other symbolic region does.
#[test]
fn a_symbolic_reduction_defers_its_public_logical_program() {
    let mut region = serial_sum_region();
    region.symbols = vec![name("n", 1)];
    region.operands[0].axes = vec![symbol_axis("n", 12), named_axis("cols", 2, 14)];
    let expansion = lower(&region).expect("a symbolic reduction lowers");
    assert!(expansion.program.verified().is_none());
    // The result is still derived, and it is `f32[n]`: reducing `cols` removes
    // the literal axis and leaves the symbol sourcing one.
    assert!(
        expansion
            .facts
            .contains("axes: &[::tiler::__private::ResultAxis::Symbol(0usize)] }"),
        "{}",
        expansion.facts,
    );
}

/// Compiles one region's public logical program against the bound macOS
/// declaration and returns whether a plan was selected.
///
/// The compiler is invoked from *this* file rather than only from
/// `crate::aot`'s tests, and the reason is what those tests say about
/// themselves: they build their programs by hand so that a grammar change
/// cannot fail them for an unrelated reason. The claim below is exactly the one
/// they therefore cannot make — that a region a consumer can *write* denotes a
/// whole program this build recognizes — and that is a claim about the grammar,
/// so it is checked beside the grammar.
///
/// No toolchain runs and no file is written: `compile` stops at a selected plan,
/// which is the whole of "the compiler admits this region for the declared
/// target". Producing the bytes is `crate::aot`'s, and the end-to-end path is
/// `crates/tiler/tests/facade/pass/deliver_compiles_a_reduction.rs`.
fn plans_for_the_bound_declaration(region: &RegionSyntax<At>) -> bool {
    use tiler_build::BoundMetalCompileDeclaration;
    use tiler_compiler::session::{CompileRequest, compile};
    use tiler_compiler::target::TargetRequest;

    let expansion = lower(region).expect("the region lowers");
    let program = expansion
        .program
        .verified()
        .expect("a literal-extent region has a verified program");
    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().expect("the declaration assembles");
    let targets =
        TargetRequest::new([declaration.profile().clone()]).expect("a singleton target request");
    compile(CompileRequest::new(program, crate::aot::CONTRACT, targets))
        .ok()
        .and_then(|batch| batch.into_targets().pop())
        .and_then(|outcome| outcome.into_parts().1.ok())
        .is_some_and(|compilation| compilation.selected().is_some())
}

/// The reduction this grammar spells reaches a whole program the compiler
/// recognizes, which is the point of admitting it at all.
///
/// The pointwise region beside it is the control: without it, a compiler that
/// admitted nothing and one that admits both would be indistinguishable here.
#[test]
fn the_recognized_serial_sum_shape_is_reachable_from_a_region() {
    assert!(
        plans_for_the_bound_declaration(&serial_sum_region()),
        "`strict_serial_sum(x * 2.0 + 1.0, [cols])` must reach a recognized whole program",
    );
    assert!(
        plans_for_the_bound_declaration(&static_region()),
        "the approved pointwise region must still plan, or the assertion above proves nothing",
    );
}

/// A region the compiler does not recognize is refused with a diagnostic naming
/// what a consumer would change, rather than with the capability rule alone.
///
/// Both cases are semantically well-formed regions: each lowers to a verified
/// public logical program, and each is refused only because *this build's*
/// recognizer does not cover its whole-program shape. That is what makes the
/// diagnostic the deliverable — the consumer did nothing wrong that a rule name
/// like `input-arity` would tell them about.
///
/// The stated cache root does not exist and could not be created, so a case that
/// got past the compiler would fail differently and this test would say so.
#[test]
fn an_unrecognized_region_names_what_a_consumer_would_change() {
    use tiler_metal_aot::driver::Toolchain;
    use tiler_metal_aot::family::ArtifactFamilySelection;

    let deliver = |region: &RegionSyntax<At>| {
        let expansion = lower(region).expect("the region lowers");
        assert!(
            expansion.program.verified().is_some(),
            "the region must be a valid public logical program, or this tests the wrong refusal",
        );
        crate::aot::deliver(
            expansion.program.verified(),
            ArtifactFamilySelection::new(crate::delivery::NamedProfile::MacOs.policy())
                .expect("the accepted macOS profile is a valid selection"),
            &crate::cache_root::RootEnvironment::new(
                Some(std::ffi::OsString::from("/tiler-no-such-cache-root")),
                None,
            ),
            &Toolchain::system(),
        )
        .expect_err("an unrecognized whole program has no plan")
        .to_string()
    };

    // A bare reduction: no scalar arithmetic between the input and the sum.
    let mut bare = serial_sum_region();
    bare.body = reduction(50, reference("x", 53), &[("cols", 56)]);

    // A reduction over two inputs.
    let mut multi_input = serial_sum_region();
    multi_input.operands.push(operand(
        "y",
        20,
        "f32",
        vec![named_axis("rows", 2, 22), named_axis("cols", 2, 24)],
    ));
    multi_input.body = reduction(
        50,
        binary(
            Operator::Add,
            51,
            binary(
                Operator::Multiply,
                52,
                reference("x", 53),
                reference("y", 54),
            ),
            scalar("1.0", 55),
        ),
        &[("cols", 56)],
    );

    // A pointwise chain deeper than one scale and one bias.
    let mut deeper = serial_sum_region();
    deeper.body = reduction(
        50,
        binary(
            Operator::Add,
            51,
            binary(
                Operator::Multiply,
                52,
                binary(
                    Operator::Multiply,
                    57,
                    reference("x", 53),
                    scalar("2.0", 54),
                ),
                scalar("3.0", 58),
            ),
            scalar("1.0", 55),
        ),
        &[("cols", 56)],
    );

    // A reduction whose operand is itself a reduction.
    let mut nested = serial_sum_region();
    nested.body = reduction(
        57,
        reduction(
            50,
            binary(
                Operator::Add,
                51,
                binary(
                    Operator::Multiply,
                    52,
                    reference("x", 53),
                    scalar("2.0", 54),
                ),
                scalar("1.0", 55),
            ),
            &[("cols", 56)],
        ),
        &[("rows", 58)],
    );

    let cases = [
        ("a bare reduction", bare),
        ("a two-input reduction", multi_input),
        ("a deeper pointwise chain under a reduction", deeper),
        ("a reduction of a reduction", nested),
    ];
    assert_eq!(
        cases.len(),
        4,
        "the population is every grammar-expressible shape this build does not recognize that the \
         ticket names, counted",
    );
    for (label, region) in cases {
        let diagnostic = deliver(&region);
        for named in [
            "strict_serial_sum(x * 2.0 + 1.0, [cols])",
            "(a * b) + c",
            "fallback-only",
        ] {
            assert!(
                diagnostic.contains(named),
                "case `{label}` must name `{named}`: {diagnostic}",
            );
        }
        assert!(
            !diagnostic.contains("UnsupportedCapability {"),
            "case `{label}` must not leak the raw capability refusal: {diagnostic}",
        );
    }

    // The accepting neighbour, differing from the first case by the scalar
    // arithmetic alone: it is not refused at all.
    assert!(plans_for_the_bound_declaration(&serial_sum_region()));
}
