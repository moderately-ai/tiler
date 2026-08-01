//! What a parsed region means, decided from both sides.
//!
//! The syntax is built directly rather than parsed, so a failure here is about
//! meaning rather than about the grammar; `crate::grammar::tests` is where the
//! text is decided. Spans are integers, for the reason stated there.

use crate::grammar::{AxisSyntax, Expression, Name, OperandSyntax, Operator, RegionSyntax};

use super::{ProgramEvidence, RegionError, lower};

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
    AxisSyntax::Symbol(name(text, at))
}

fn literal_axis(value: u64, at: u32) -> AxisSyntax<At> {
    AxisSyntax::Literal {
        value,
        span: At(at),
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

    assert_eq!(expansion.program, ProgramEvidence::DeferredSymbolicExtent);
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
    assert_eq!(expansion.program, ProgramEvidence::Verified);
    assert!(
        expansion
            .facts
            .contains("axes: &[::tiler::__private::ResultAxis::Literal(4u64)]"),
        "{}",
        expansion.facts,
    );

    // The paired neighbour: one symbolic axis is enough to defer it, so
    // `Verified` is a claim about representability rather than about this
    // fixture happening to be simple.
    assert_eq!(
        lower(&approved_region(|| vec![symbol_axis("n", 12)]))
            .expect("the region lowers")
            .program,
        ProgramEvidence::DeferredSymbolicExtent,
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
    assert_eq!(expansion.program, ProgramEvidence::Verified);
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
                      ::tiler::value::StorageScalar::F32, rank: 0usize }"
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
    assert_eq!(expansion.program, ProgramEvidence::Verified);
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
