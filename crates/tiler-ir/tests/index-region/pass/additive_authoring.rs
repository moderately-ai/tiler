use tiler_ir::index::{
    BoundsProofView, IndexExprView, ReducerBodyValueDefinitionView, ScalarArity,
    ScalarAttributeSchema, ScalarEffect, ScalarOperationContract, ScalarOperationKindRef,
    ScalarValueDefinitionView, WriteOwnershipProofView,
};
use tiler_ir::semantic::CanonicalValue;

fn inspect_expression(expression: IndexExprView<'_>) {
    match expression {
        IndexExprView::Constant(_) => {}
        IndexExprView::Dimension(_) => {}
        IndexExprView::LinearCombination { .. } => {}
        IndexExprView::FloorDiv { .. } => {}
        IndexExprView::Modulo { .. } => {}
        _ => {}
    }
}

fn inspect_proofs(bounds: BoundsProofView, ownership: WriteOwnershipProofView) {
    let _ = match bounds {
        BoundsProofView::VacuousEmptyDomain => 0,
        BoundsProofView::Interval => 1,
        BoundsProofView::Exhaustive { .. } => 2,
        _ => 3,
    };
    let _ = match ownership {
        WriteOwnershipProofView::CoordinatePermutation => 0,
        WriteOwnershipProofView::Exhaustive { .. } => 1,
        _ => 2,
    };
}

fn inspect_scalar(
    value: ScalarValueDefinitionView,
    operation: ScalarOperationKindRef<'_>,
    body_value: ReducerBodyValueDefinitionView,
) {
    match value {
        ScalarValueDefinitionView::AccessRead(_) => {}
        ScalarValueDefinitionView::OperationResult { .. } => {}
        _ => {}
    }
    match operation {
        ScalarOperationKindRef::Apply { .. } => {}
        ScalarOperationKindRef::Reduce(_) => {}
        _ => {}
    }
    match body_value {
        ReducerBodyValueDefinitionView::StateParameter(_) => {}
        ReducerBodyValueDefinitionView::ContributorParameter(_) => {}
        ReducerBodyValueDefinitionView::OperationResult { .. } => {}
        _ => {}
    }
}

fn main() {
    let empty = CanonicalValue::record([]).unwrap();
    let contract = ScalarOperationContract::new(
        ScalarAttributeSchema::empty(),
        ScalarArity::exact(1).unwrap(),
        ScalarArity::exact(1).unwrap(),
        ScalarEffect::Pure,
        empty.clone(),
        empty,
    );
    assert_eq!(contract.operands().min(), 1);

    let _ = inspect_expression;
    let _ = inspect_proofs;
    let _ = inspect_scalar;
}
