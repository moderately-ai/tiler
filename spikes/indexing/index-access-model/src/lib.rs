//! Executable contract spike for ADR 0046.
//!
//! This is deliberately small and dependency-free. It exercises the boundary
//! between logical tensor coordinates and physical strided storage; it is not
//! the implementation chosen for Tiler.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IterVar(pub u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ParamId(pub u16);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IndexExpr {
    Const(i128),
    Iter(IterVar),
    Param(ParamId),
    /// Reserved solely to prove that tensor-derived indices are rejected.
    TensorValue(u16),
    Add(Vec<IndexExpr>),
    Mul(Vec<IndexExpr>),
    FloorDiv(Box<IndexExpr>, Box<IndexExpr>),
    Mod(Box<IndexExpr>, Box<IndexExpr>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExprClass {
    Affine,
    QuasiAffine,
    SemiAffine,
    DataDependent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyError {
    Arity,
    FreeIterationVariable(IterVar),
    FreeParameter(ParamId),
    DataDependentIndex,
    NonlinearIterationProduct,
    IterationDependentDivisor,
    NonPositiveDivisor,
    EvaluationOverflow,
    CoordinateOutOfBounds,
    DuplicateWrite,
    IncompleteWrite,
    LayoutRank,
    NegativeStrideUnsupported,
    StorageOutOfBounds,
}

impl IndexExpr {
    pub fn canonicalize(self) -> Self {
        match self {
            Self::Add(xs) => {
                let mut terms = Vec::new();
                for x in xs.into_iter().map(Self::canonicalize) {
                    match x {
                        Self::Add(inner) => terms.extend(inner),
                        other => terms.push(other),
                    }
                }
                terms.sort();
                let constants: Vec<_> = terms
                    .iter()
                    .filter_map(|x| match x {
                        Self::Const(v) => Some(*v),
                        _ => None,
                    })
                    .collect();
                if let Some(constant) = constants
                    .iter()
                    .try_fold(0_i128, |acc, &v| acc.checked_add(v))
                {
                    terms.retain(|x| !matches!(x, Self::Const(_)));
                    if constant != 0 {
                        terms.push(Self::Const(constant));
                    }
                }
                terms.retain(|x| *x != Self::Const(0));
                terms.sort();
                match terms.len() {
                    0 => Self::Const(0),
                    1 => terms.pop().expect("length checked"),
                    _ => Self::Add(terms),
                }
            }
            Self::Mul(xs) => {
                let mut factors = Vec::new();
                for x in xs.into_iter().map(Self::canonicalize) {
                    match x {
                        Self::Mul(inner) => factors.extend(inner),
                        other => factors.push(other),
                    }
                }
                if factors.contains(&Self::Const(0)) {
                    return Self::Const(0);
                }
                factors.sort();
                let constants: Vec<_> = factors
                    .iter()
                    .filter_map(|x| match x {
                        Self::Const(v) => Some(*v),
                        _ => None,
                    })
                    .collect();
                if let Some(constant) = constants
                    .iter()
                    .try_fold(1_i128, |acc, &v| acc.checked_mul(v))
                {
                    factors.retain(|x| !matches!(x, Self::Const(_)));
                    if constant != 1 || factors.is_empty() {
                        factors.push(Self::Const(constant));
                    }
                }
                factors.retain(|x| *x != Self::Const(1));
                factors.sort();
                match factors.len() {
                    0 => Self::Const(1),
                    1 => factors.pop().expect("length checked"),
                    _ => Self::Mul(factors),
                }
            }
            Self::FloorDiv(a, b) => {
                let a = a.canonicalize();
                let b = b.canonicalize();
                if let (Self::Const(x), Self::Const(y)) = (&a, &b)
                    && *y > 0
                {
                    return Self::Const(x.div_euclid(*y));
                }
                Self::FloorDiv(Box::new(a), Box::new(b))
            }
            Self::Mod(a, b) => {
                let a = a.canonicalize();
                let b = b.canonicalize();
                if let (Self::Const(x), Self::Const(y)) = (&a, &b)
                    && *y > 0
                {
                    return Self::Const(x.rem_euclid(*y));
                }
                Self::Mod(Box::new(a), Box::new(b))
            }
            atom => atom,
        }
    }

    fn depends_on_iteration(&self) -> bool {
        match self {
            Self::Iter(_) | Self::TensorValue(_) => true,
            Self::Const(_) | Self::Param(_) => false,
            Self::Add(xs) | Self::Mul(xs) => xs.iter().any(Self::depends_on_iteration),
            Self::FloorDiv(a, b) | Self::Mod(a, b) => {
                a.depends_on_iteration() || b.depends_on_iteration()
            }
        }
    }

    pub fn class(&self) -> ExprClass {
        match self {
            Self::TensorValue(_) => ExprClass::DataDependent,
            Self::Const(_) | Self::Iter(_) | Self::Param(_) => ExprClass::Affine,
            Self::Add(xs) => xs
                .iter()
                .map(Self::class)
                .max_by_key(|c| match c {
                    ExprClass::Affine => 0,
                    ExprClass::QuasiAffine => 1,
                    ExprClass::SemiAffine => 2,
                    ExprClass::DataDependent => 3,
                })
                .unwrap_or(ExprClass::Affine),
            Self::Mul(xs) => {
                if xs
                    .iter()
                    .any(|x| matches!(x.class(), ExprClass::DataDependent))
                {
                    ExprClass::DataDependent
                } else if xs.iter().filter(|x| !matches!(x, Self::Const(_))).count() > 1 {
                    ExprClass::SemiAffine
                } else {
                    xs.iter()
                        .map(Self::class)
                        .find(|c| !matches!(c, ExprClass::Affine))
                        .unwrap_or(ExprClass::Affine)
                }
            }
            Self::FloorDiv(a, b) | Self::Mod(a, b) => {
                if matches!(a.class(), ExprClass::DataDependent)
                    || matches!(b.class(), ExprClass::DataDependent)
                {
                    ExprClass::DataDependent
                } else if matches!(**b, Self::Const(_))
                    && !matches!(a.class(), ExprClass::SemiAffine)
                {
                    ExprClass::QuasiAffine
                } else {
                    ExprClass::SemiAffine
                }
            }
        }
    }

    fn verify_shape(
        &self,
        dims: usize,
        params: &BTreeMap<ParamId, i128>,
    ) -> Result<(), VerifyError> {
        match self {
            Self::Const(_) => Ok(()),
            Self::Iter(v) if usize::from(v.0) < dims => Ok(()),
            Self::Iter(v) => Err(VerifyError::FreeIterationVariable(*v)),
            Self::Param(p) if params.contains_key(p) => Ok(()),
            Self::Param(p) => Err(VerifyError::FreeParameter(*p)),
            Self::TensorValue(_) => Err(VerifyError::DataDependentIndex),
            Self::Add(xs) => xs.iter().try_for_each(|x| x.verify_shape(dims, params)),
            Self::Mul(xs) => {
                xs.iter().try_for_each(|x| x.verify_shape(dims, params))?;
                if xs.iter().filter(|x| x.depends_on_iteration()).count() > 1 {
                    Err(VerifyError::NonlinearIterationProduct)
                } else {
                    Ok(())
                }
            }
            Self::FloorDiv(a, b) | Self::Mod(a, b) => {
                a.verify_shape(dims, params)?;
                b.verify_shape(dims, params)?;
                if b.depends_on_iteration() {
                    return Err(VerifyError::IterationDependentDivisor);
                }
                if b.eval(&vec![0; dims], params)? <= 0 {
                    return Err(VerifyError::NonPositiveDivisor);
                }
                Ok(())
            }
        }
    }

    pub fn eval(
        &self,
        iteration: &[i128],
        params: &BTreeMap<ParamId, i128>,
    ) -> Result<i128, VerifyError> {
        match self {
            Self::Const(v) => Ok(*v),
            Self::Iter(v) => iteration
                .get(usize::from(v.0))
                .copied()
                .ok_or(VerifyError::FreeIterationVariable(*v)),
            Self::Param(p) => params.get(p).copied().ok_or(VerifyError::FreeParameter(*p)),
            Self::TensorValue(_) => Err(VerifyError::DataDependentIndex),
            Self::Add(xs) => xs.iter().try_fold(0_i128, |acc, x| {
                acc.checked_add(x.eval(iteration, params)?)
                    .ok_or(VerifyError::EvaluationOverflow)
            }),
            Self::Mul(xs) => xs.iter().try_fold(1_i128, |acc, x| {
                acc.checked_mul(x.eval(iteration, params)?)
                    .ok_or(VerifyError::EvaluationOverflow)
            }),
            Self::FloorDiv(a, b) => {
                let divisor = b.eval(iteration, params)?;
                if divisor <= 0 {
                    return Err(VerifyError::NonPositiveDivisor);
                }
                Ok(a.eval(iteration, params)?.div_euclid(divisor))
            }
            Self::Mod(a, b) => {
                let divisor = b.eval(iteration, params)?;
                if divisor <= 0 {
                    return Err(VerifyError::NonPositiveDivisor);
                }
                Ok(a.eval(iteration, params)?.rem_euclid(divisor))
            }
        }
    }

    fn substitute_iterations(&self, replacements: &[IndexExpr]) -> Result<Self, VerifyError> {
        Ok(match self {
            Self::Iter(v) => replacements
                .get(usize::from(v.0))
                .cloned()
                .ok_or(VerifyError::FreeIterationVariable(*v))?,
            Self::Add(xs) => Self::Add(
                xs.iter()
                    .map(|x| x.substitute_iterations(replacements))
                    .collect::<Result<_, _>>()?,
            ),
            Self::Mul(xs) => Self::Mul(
                xs.iter()
                    .map(|x| x.substitute_iterations(replacements))
                    .collect::<Result<_, _>>()?,
            ),
            Self::FloorDiv(a, b) => Self::FloorDiv(
                Box::new(a.substitute_iterations(replacements)?),
                Box::new(b.substitute_iterations(replacements)?),
            ),
            Self::Mod(a, b) => Self::Mod(
                Box::new(a.substitute_iterations(replacements)?),
                Box::new(b.substitute_iterations(replacements)?),
            ),
            atom => atom.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IterationDomain {
    pub extents: Vec<u64>,
}

impl IterationDomain {
    pub fn points(&self) -> Vec<Vec<i128>> {
        let mut points = vec![Vec::new()];
        for &extent in &self.extents {
            let mut next = Vec::new();
            for prefix in points {
                for i in 0..extent {
                    let mut point = prefix.clone();
                    point.push(i128::from(i));
                    next.push(point);
                }
            }
            points = next;
        }
        points
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TensorAccessMap {
    pub domain_rank: usize,
    pub coordinates: Vec<IndexExpr>,
}

impl TensorAccessMap {
    pub fn canonicalize(self) -> Self {
        Self {
            domain_rank: self.domain_rank,
            coordinates: self
                .coordinates
                .into_iter()
                .map(IndexExpr::canonicalize)
                .collect(),
        }
    }

    pub fn verify(
        &self,
        domain: &IterationDomain,
        tensor_shape: &[u64],
        params: &BTreeMap<ParamId, i128>,
        ordinary_write: bool,
    ) -> Result<(), VerifyError> {
        if self.domain_rank != domain.extents.len() || self.coordinates.len() != tensor_shape.len()
        {
            return Err(VerifyError::Arity);
        }
        for expr in &self.coordinates {
            expr.verify_shape(self.domain_rank, params)?;
        }

        let mut visited = BTreeSet::new();
        for point in domain.points() {
            let coordinate: Vec<_> = self
                .coordinates
                .iter()
                .map(|x| x.eval(&point, params))
                .collect::<Result<_, _>>()?;
            if coordinate
                .iter()
                .zip(tensor_shape)
                .any(|(&i, &extent)| i < 0 || i >= i128::from(extent))
            {
                return Err(VerifyError::CoordinateOutOfBounds);
            }
            if ordinary_write && !visited.insert(coordinate) {
                return Err(VerifyError::DuplicateWrite);
            }
        }
        if ordinary_write {
            let expected = tensor_shape.iter().try_fold(1_u128, |acc, &x| {
                acc.checked_mul(u128::from(x))
                    .ok_or(VerifyError::EvaluationOverflow)
            })?;
            if u128::try_from(visited.len()).expect("usize always fits u128") != expected {
                return Err(VerifyError::IncompleteWrite);
            }
        }
        Ok(())
    }

    pub fn evaluate(
        &self,
        point: &[i128],
        params: &BTreeMap<ParamId, i128>,
    ) -> Result<Vec<i128>, VerifyError> {
        self.coordinates
            .iter()
            .map(|x| x.eval(point, params))
            .collect()
    }

    /// Returns `next(self(iteration))`.
    pub fn then(&self, next: &Self) -> Result<Self, VerifyError> {
        if next.domain_rank != self.coordinates.len() {
            return Err(VerifyError::Arity);
        }
        Ok(Self {
            domain_rank: self.domain_rank,
            coordinates: next
                .coordinates
                .iter()
                .map(|x| x.substitute_iterations(&self.coordinates))
                .collect::<Result<_, _>>()?,
        }
        .canonicalize())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BufferView {
    pub shape: Vec<u64>,
    pub allocation_relative_start: u64,
    pub element_strides: Vec<i64>,
    pub accessible_elements: u64,
}

impl BufferView {
    pub fn verify(&self) -> Result<(), VerifyError> {
        if self.shape.len() != self.element_strides.len() {
            return Err(VerifyError::LayoutRank);
        }
        if self.element_strides.iter().any(|&x| x < 0) {
            return Err(VerifyError::NegativeStrideUnsupported);
        }
        if self.shape.contains(&0) {
            return Ok(());
        }
        let max = self.shape.iter().zip(&self.element_strides).try_fold(
            u128::from(self.allocation_relative_start),
            |acc, (&extent, &stride)| {
                let contribution = u128::from(extent - 1)
                    .checked_mul(u128::try_from(stride).expect("nonnegative checked"))
                    .ok_or(VerifyError::EvaluationOverflow)?;
                acc.checked_add(contribution)
                    .ok_or(VerifyError::EvaluationOverflow)
            },
        )?;
        if max >= u128::from(self.accessible_elements) {
            return Err(VerifyError::StorageOutOfBounds);
        }
        Ok(())
    }

    pub fn element_offset(&self, coordinate: &[i128]) -> Result<u64, VerifyError> {
        self.verify()?;
        if coordinate.len() != self.shape.len() {
            return Err(VerifyError::LayoutRank);
        }
        let mut offset = u128::from(self.allocation_relative_start);
        for ((&index, &extent), &stride) in coordinate
            .iter()
            .zip(&self.shape)
            .zip(&self.element_strides)
        {
            if index < 0 || index >= i128::from(extent) {
                return Err(VerifyError::CoordinateOutOfBounds);
            }
            offset = offset
                .checked_add(
                    u128::try_from(index)
                        .expect("nonnegative checked")
                        .checked_mul(u128::try_from(stride).expect("nonnegative checked"))
                        .ok_or(VerifyError::EvaluationOverflow)?,
                )
                .ok_or(VerifyError::EvaluationOverflow)?;
        }
        u64::try_from(offset).map_err(|_| VerifyError::EvaluationOverflow)
    }
}

/// Exhaustive proof helper for the bounded spike. Production uses symbolic
/// range proofs/guards but must cover the same intermediates.
pub fn all_offsets_fit_u32(
    domain: &IterationDomain,
    map: &TensorAccessMap,
    view: &BufferView,
    params: &BTreeMap<ParamId, i128>,
) -> Result<bool, VerifyError> {
    view.verify()?;
    for point in domain.points() {
        if point.iter().any(|&x| u32::try_from(x).is_err()) {
            return Ok(false);
        }
        let coordinate = map.evaluate(&point, params)?;
        if coordinate.iter().any(|&x| u32::try_from(x).is_err()) {
            return Ok(false);
        }
        if u32::try_from(view.element_offset(&coordinate)?).is_err() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(v: i128) -> IndexExpr {
        IndexExpr::Const(v)
    }
    fn d(v: u16) -> IndexExpr {
        IndexExpr::Iter(IterVar(v))
    }
    fn add(xs: impl IntoIterator<Item = IndexExpr>) -> IndexExpr {
        IndexExpr::Add(xs.into_iter().collect())
    }
    fn mul(xs: impl IntoIterator<Item = IndexExpr>) -> IndexExpr {
        IndexExpr::Mul(xs.into_iter().collect())
    }

    #[test]
    fn canonical_add_is_order_independent() {
        let a = add([d(1), c(2), d(0)]).canonicalize();
        let b = add([d(0), d(1), c(2)]).canonicalize();
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalization_does_not_wrap_large_constants() {
        let once = add([c(i128::MAX), c(1), d(0)]).canonicalize();
        let twice = once.clone().canonicalize();
        assert_eq!(once, twice);
        assert!(matches!(once, IndexExpr::Add(_)));
    }

    #[test]
    fn permutation_is_total_and_uniquely_writes() {
        let domain = IterationDomain {
            extents: vec![2, 3],
        };
        let map = TensorAccessMap {
            domain_rank: 2,
            coordinates: vec![d(1), d(0)],
        };
        assert_eq!(map.verify(&domain, &[3, 2], &BTreeMap::new(), true), Ok(()));
    }

    #[test]
    fn composed_inverse_permutations_canonicalize_to_identity() {
        let permutation = TensorAccessMap {
            domain_rank: 2,
            coordinates: vec![d(1), d(0)],
        };
        let composed = permutation.then(&permutation).expect("compatible maps");
        assert_eq!(composed.coordinates, vec![d(0), d(1)]);
    }

    #[test]
    fn broadcast_aliases_reads_but_not_ordinary_writes() {
        let domain = IterationDomain {
            extents: vec![4, 3],
        };
        let map = TensorAccessMap {
            domain_rank: 2,
            coordinates: vec![d(1)],
        };
        assert_eq!(map.verify(&domain, &[3], &BTreeMap::new(), false), Ok(()));
        assert_eq!(
            map.verify(&domain, &[3], &BTreeMap::new(), true),
            Err(VerifyError::DuplicateWrite)
        );
    }

    #[test]
    fn reshape_linearizes_then_delinearizes() {
        let linear = add([mul([d(0), c(2)]), d(1)]);
        let map = TensorAccessMap {
            domain_rank: 2,
            coordinates: vec![
                IndexExpr::FloorDiv(Box::new(linear.clone()), Box::new(c(3))),
                IndexExpr::Mod(Box::new(linear), Box::new(c(3))),
            ],
        }
        .canonicalize();
        let domain = IterationDomain {
            extents: vec![3, 2],
        };
        assert_eq!(map.verify(&domain, &[2, 3], &BTreeMap::new(), true), Ok(()));
    }

    #[test]
    fn split_and_merge_maps_cover_the_same_six_elements() {
        let split = TensorAccessMap {
            domain_rank: 1,
            coordinates: vec![
                IndexExpr::FloorDiv(Box::new(d(0)), Box::new(c(3))),
                IndexExpr::Mod(Box::new(d(0)), Box::new(c(3))),
            ],
        };
        assert_eq!(
            split.verify(
                &IterationDomain { extents: vec![6] },
                &[2, 3],
                &BTreeMap::new(),
                true
            ),
            Ok(())
        );

        let merge = TensorAccessMap {
            domain_rank: 2,
            coordinates: vec![add([mul([d(0), c(3)]), d(1)])],
        };
        assert_eq!(
            merge.verify(
                &IterationDomain {
                    extents: vec![2, 3]
                },
                &[6],
                &BTreeMap::new(),
                true
            ),
            Ok(())
        );
    }

    #[test]
    fn zero_extent_and_rank_zero_domains_have_explicit_coverage() {
        let empty = TensorAccessMap {
            domain_rank: 1,
            coordinates: vec![d(0)],
        };
        assert_eq!(
            empty.verify(
                &IterationDomain { extents: vec![0] },
                &[0],
                &BTreeMap::new(),
                true
            ),
            Ok(())
        );

        let scalar = TensorAccessMap {
            domain_rank: 0,
            coordinates: vec![],
        };
        assert_eq!(
            scalar.verify(
                &IterationDomain { extents: vec![] },
                &[],
                &BTreeMap::new(),
                true
            ),
            Ok(())
        );
    }

    #[test]
    fn bounds_and_complete_write_coverage_are_independent() {
        let identity = TensorAccessMap {
            domain_rank: 1,
            coordinates: vec![d(0)],
        };
        assert_eq!(
            identity.verify(
                &IterationDomain { extents: vec![2] },
                &[3],
                &BTreeMap::new(),
                true
            ),
            Err(VerifyError::IncompleteWrite)
        );
        assert_eq!(
            identity.verify(
                &IterationDomain { extents: vec![3] },
                &[2],
                &BTreeMap::new(),
                false
            ),
            Err(VerifyError::CoordinateOutOfBounds)
        );
    }

    #[test]
    fn symbolic_reshape_is_semi_affine_and_requires_positive_divisor() {
        let width = IndexExpr::Param(ParamId(0));
        let divisor = IndexExpr::Param(ParamId(1));
        let linear = add([mul([d(0), width]), d(1)]);
        let map = TensorAccessMap {
            domain_rank: 2,
            coordinates: vec![
                IndexExpr::FloorDiv(Box::new(linear.clone()), Box::new(divisor.clone())),
                IndexExpr::Mod(Box::new(linear), Box::new(divisor)),
            ],
        };
        assert_eq!(map.coordinates[0].class(), ExprClass::SemiAffine);
        let domain = IterationDomain {
            extents: vec![3, 2],
        };
        let params = BTreeMap::from([(ParamId(0), 2), (ParamId(1), 3)]);
        assert_eq!(map.verify(&domain, &[2, 3], &params, true), Ok(()));

        let bad_params = BTreeMap::from([(ParamId(0), 2), (ParamId(1), 0)]);
        assert_eq!(
            map.verify(&domain, &[2, 3], &bad_params, false),
            Err(VerifyError::NonPositiveDivisor)
        );
    }

    #[test]
    fn noncontiguous_view_uses_allocation_relative_element_offsets() {
        let view = BufferView {
            shape: vec![2, 3],
            allocation_relative_start: 2,
            element_strides: vec![5, 2],
            accessible_elements: 12,
        };
        assert_eq!(view.element_offset(&[1, 2]), Ok(11));
    }

    #[test]
    fn storage_range_and_negative_stride_are_rejected() {
        let too_short = BufferView {
            shape: vec![2, 3],
            allocation_relative_start: 2,
            element_strides: vec![5, 2],
            accessible_elements: 11,
        };
        assert_eq!(too_short.verify(), Err(VerifyError::StorageOutOfBounds));
        let negative = BufferView {
            shape: vec![2],
            allocation_relative_start: 1,
            element_strides: vec![-1],
            accessible_elements: 2,
        };
        assert_eq!(
            negative.verify(),
            Err(VerifyError::NegativeStrideUnsupported)
        );
    }

    #[test]
    fn tensor_derived_and_iteration_nonlinear_indices_are_rejected() {
        let data_map = TensorAccessMap {
            domain_rank: 1,
            coordinates: vec![IndexExpr::TensorValue(0)],
        };
        assert_eq!(
            data_map.verify(
                &IterationDomain { extents: vec![1] },
                &[1],
                &BTreeMap::new(),
                false
            ),
            Err(VerifyError::DataDependentIndex)
        );
        let nonlinear = TensorAccessMap {
            domain_rank: 2,
            coordinates: vec![mul([d(0), d(1)])],
        };
        assert_eq!(
            nonlinear.verify(
                &IterationDomain {
                    extents: vec![1, 1]
                },
                &[1],
                &BTreeMap::new(),
                false
            ),
            Err(VerifyError::NonlinearIterationProduct)
        );
    }

    #[test]
    fn extents_fitting_u32_do_not_prove_offsets_fit_u32() {
        let domain = IterationDomain { extents: vec![2] };
        let map = TensorAccessMap {
            domain_rank: 1,
            coordinates: vec![d(0)],
        };
        let view = BufferView {
            shape: vec![2],
            allocation_relative_start: 0,
            element_strides: vec![i64::from(u32::MAX) + 1],
            accessible_elements: u64::from(u32::MAX) + 2,
        };
        assert_eq!(
            all_offsets_fit_u32(&domain, &map, &view, &BTreeMap::new()),
            Ok(false)
        );
        assert_eq!(view.element_offset(&[1]), Ok(u64::from(u32::MAX) + 1));
    }

    #[test]
    fn tail_mask_prevents_out_of_domain_access() {
        for logical_extent in [7_u64, 8, 9] {
            let launched = logical_extent.div_ceil(8) * 8;
            let view = BufferView {
                shape: vec![logical_extent],
                allocation_relative_start: 0,
                element_strides: vec![1],
                accessible_elements: logical_extent,
            };
            for lane in 0..launched {
                let active = lane < logical_extent;
                if active {
                    assert_eq!(view.element_offset(&[i128::from(lane)]), Ok(lane));
                }
            }
            assert_eq!(
                view.element_offset(&[i128::from(logical_extent)]),
                Err(VerifyError::CoordinateOutOfBounds)
            );
        }
    }
}
