//! IR's closed exact-finite residual-domain proof algorithm.
//!
//! One authority, one derivation: every retained obligation whose access domain
//! resolves to a bounded finite product is decided by evaluating every point of
//! it exactly, in arbitrary precision, and anything outside that fragment is
//! reported unknown rather than approximated. The work is metered before it is
//! performed — a conservative preflight charges structural cells and integer
//! bytes against a ledger the whole realization shares — so a caller that funded
//! one budget cannot have it multiplied by a stage count, and exhaustion is a
//! typed unknown rather than an unbounded run.
//!
//! The cost formulas follow the locked `num-bigint` implementations and bound
//! work; they are not part of any identity. Nothing here decides what a receipt
//! retains: that vocabulary is [`super::proof`]'s.

use std::collections::{HashMap, HashSet};

use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero};

use crate::index::{
    IndexDomainPredicate, IndexDomainUnknownReason, IndexExprView, IndexExtentRef, IndexInteger,
    ProofResource, UnknownIndexDomainPredicate, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexRegion, VerifiedTensorAccessId,
};

use super::proof::{
    IndexDomainDisproof, IndexDomainProofBudget, IndexDomainProofClaim, IndexDomainProofEvidence,
};
use super::{COUNTEREXAMPLE_TAG, EXHAUSTIVE_DERIVATION};

#[derive(Clone, Copy, Debug)]
pub(super) struct IndexDomainProofExhaustion {
    pub(super) resource: ProofResource,
    pub(super) required: u128,
    pub(super) limit: u64,
}

pub(super) struct IndexDomainProofLedger {
    cell_limit: u64,
    integer_byte_limit: u64,
    used_cells: u128,
    pub(super) used_integer_bytes: u128,
    pub(super) exhaustion: Option<IndexDomainProofExhaustion>,
}

impl IndexDomainProofLedger {
    pub(super) const fn new(budget: IndexDomainProofBudget) -> Self {
        Self {
            cell_limit: budget.max_cells(),
            integer_byte_limit: budget.max_integer_bytes(),
            used_cells: 0,
            used_integer_bytes: 0,
            exhaustion: None,
        }
    }

    fn debit(&mut self, resource: ProofResource, amount: u128) -> Result<(), ProofPlanningFailure> {
        if let Some(exhaustion) = self.exhaustion {
            return Err(ProofPlanningFailure::Exhausted(exhaustion));
        }
        let (used, limit) = match resource {
            ProofResource::Cells => (&mut self.used_cells, self.cell_limit),
            ProofResource::IntegerBytes => (&mut self.used_integer_bytes, self.integer_byte_limit),
        };
        let Some(required) = used.checked_add(amount) else {
            return Err(ProofPlanningFailure::Unsupported);
        };
        if required > u128::from(limit) {
            let exhaustion = IndexDomainProofExhaustion {
                resource,
                required,
                limit,
            };
            self.exhaustion = Some(exhaustion);
            return Err(ProofPlanningFailure::Exhausted(exhaustion));
        }
        *used = required;
        Ok(())
    }

    pub(super) fn reserve_evaluation(
        &mut self,
        cells: u128,
        integer_bytes: u128,
    ) -> Result<(), ProofPlanningFailure> {
        if let Some(exhaustion) = self.exhaustion {
            return Err(ProofPlanningFailure::Exhausted(exhaustion));
        }
        let Some(required_cells) = self.used_cells.checked_add(cells) else {
            return Err(ProofPlanningFailure::Unsupported);
        };
        let Some(required_integer_bytes) = self.used_integer_bytes.checked_add(integer_bytes)
        else {
            return Err(ProofPlanningFailure::Unsupported);
        };
        let exhaustion = if required_cells > u128::from(self.cell_limit) {
            Some(IndexDomainProofExhaustion {
                resource: ProofResource::Cells,
                required: required_cells,
                limit: self.cell_limit,
            })
        } else if required_integer_bytes > u128::from(self.integer_byte_limit) {
            Some(IndexDomainProofExhaustion {
                resource: ProofResource::IntegerBytes,
                required: required_integer_bytes,
                limit: self.integer_byte_limit,
            })
        } else {
            None
        };
        if let Some(exhaustion) = exhaustion {
            self.exhaustion = Some(exhaustion);
            return Err(ProofPlanningFailure::Exhausted(exhaustion));
        }
        self.used_cells = required_cells;
        self.used_integer_bytes = required_integer_bytes;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ProofPlanningFailure {
    Unsupported,
    Exhausted(IndexDomainProofExhaustion),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct IndexDomainKey(pub(super) Vec<(VerifiedDimensionId, u64)>);

#[derive(Clone)]
struct ResolvedIndexDomain {
    key: IndexDomainKey,
    points: u64,
}

pub(super) struct PlannedDomainObligation {
    pub(super) slot: usize,
    pub(super) obligation: UnknownIndexDomainPredicate,
    pub(super) upper_bound: Option<u64>,
}

pub(super) struct IndexDomainGroup {
    pub(super) domain: IndexDomainKey,
    pub(super) points: u64,
    pub(super) obligations: Vec<PlannedDomainObligation>,
}

/// Assesses one region's obligations against a budget of its own.
///
/// The single-region spelling the exact-finite proof tests are written against.
/// Completion itself always goes through [`assess_finite_domains_with`], because
/// a realization's stages share one ledger.
#[cfg(test)]
pub(super) fn assess_finite_domains(
    region: &VerifiedIndexRegion,
    obligations: &[UnknownIndexDomainPredicate],
    budget: IndexDomainProofBudget,
) -> Vec<IndexDomainProofClaim> {
    let mut ledger = IndexDomainProofLedger::new(budget);
    assess_finite_domains_with(region, obligations, &mut ledger)
}

/// Assesses one stage's obligations against a ledger the whole realization
/// shares.
///
/// The ledger is a parameter rather than a fresh budget per stage because a
/// caller states one bound for the work it is willing to fund. Re-funding it per
/// stage would let an n-stage realization spend n times the limit its caller
/// named, which is the same fail-closed bound quietly weakened by the arrival of
/// a second stage.
pub(super) fn assess_finite_domains_with(
    region: &VerifiedIndexRegion,
    obligations: &[UnknownIndexDomainPredicate],
    ledger: &mut IndexDomainProofLedger,
) -> Vec<IndexDomainProofClaim> {
    let mut claims = vec![None; obligations.len()];
    let mut access_domains = HashMap::<VerifiedTensorAccessId, Option<ResolvedIndexDomain>>::new();
    let mut extents = HashMap::<IndexExtentRef, Option<u64>>::new();
    let mut groups = Vec::<IndexDomainGroup>::new();
    let mut group_indices = HashMap::<IndexDomainKey, usize>::new();

    for (slot, obligation) in obligations.iter().copied().enumerate() {
        if ledger.exhaustion.is_some() {
            break;
        }
        if let Err(failure) = ledger.debit(ProofResource::Cells, 1) {
            if matches!(failure, ProofPlanningFailure::Unsupported) {
                claims[slot] = Some(unsupported_proof_claim());
            }
            break;
        }
        let domain = if let Some(cached) = access_domains.get(&obligation.subject()) {
            cached.clone()
        } else {
            let resolved = match resolve_domain(region, obligation.subject(), &mut *ledger) {
                Ok(domain) => Some(domain),
                Err(ProofPlanningFailure::Unsupported) => None,
                Err(ProofPlanningFailure::Exhausted(_)) => break,
            };
            access_domains.insert(obligation.subject(), resolved.clone());
            resolved
        };
        let Some(domain) = domain else {
            claims[slot] = Some(unsupported_proof_claim());
            continue;
        };
        let upper_bound =
            if let IndexDomainPredicate::LessThanExtent { extent, .. } = obligation.predicate() {
                let resolved = if let Some(cached) = extents.get(&extent) {
                    *cached
                } else {
                    if ledger.debit(ProofResource::Cells, 1).is_err() {
                        break;
                    }
                    let resolved = resolve_extent(region, extent);
                    extents.insert(extent, resolved);
                    resolved
                };
                if resolved.is_none() {
                    claims[slot] = Some(unsupported_proof_claim());
                    continue;
                }
                resolved
            } else {
                None
            };
        let group_index = if let Some(group_index) = group_indices.get(&domain.key) {
            *group_index
        } else {
            let group_index = groups.len();
            groups.push(IndexDomainGroup {
                domain: domain.key.clone(),
                points: domain.points,
                obligations: Vec::new(),
            });
            group_indices.insert(domain.key.clone(), group_index);
            group_index
        };
        groups[group_index]
            .obligations
            .push(PlannedDomainObligation {
                slot,
                obligation,
                upper_bound,
            });
    }

    if let Some(exhaustion) = ledger.exhaustion {
        fill_unassessed(&mut claims, exhaustion);
        return claims.into_iter().map(Option::unwrap).collect();
    }

    for group in groups {
        if let Some(exhaustion) = ledger.exhaustion {
            fill_unassessed(&mut claims, exhaustion);
            break;
        }
        if group.points == 0 {
            for planned in group.obligations {
                claims[planned.slot] = Some(exhaustive_proof_claim(0));
            }
            continue;
        }
        match assess_domain_group(region, &group, &mut *ledger) {
            Ok(group_claims) => {
                for (planned, claim) in group.obligations.iter().zip(group_claims) {
                    claims[planned.slot] = Some(claim);
                }
            }
            Err(ProofPlanningFailure::Unsupported) => {
                for planned in group.obligations {
                    claims[planned.slot] = Some(unsupported_proof_claim());
                }
            }
            Err(ProofPlanningFailure::Exhausted(exhaustion)) => {
                fill_unassessed(&mut claims, exhaustion);
                break;
            }
        }
    }
    claims
        .into_iter()
        .map(|claim| claim.unwrap_or_else(unsupported_proof_claim))
        .collect()
}

fn resolve_domain(
    region: &VerifiedIndexRegion,
    subject: VerifiedTensorAccessId,
    ledger: &mut IndexDomainProofLedger,
) -> Result<ResolvedIndexDomain, ProofPlanningFailure> {
    let access = region
        .access(subject)
        .map_err(|_| ProofPlanningFailure::Unsupported)?;
    let mut dimensions = Vec::with_capacity(access.domain().len());
    for dimension in access.domain() {
        ledger.debit(ProofResource::Cells, 1)?;
        let extent = region
            .dimension(dimension)
            .ok()
            .and_then(|dimension| dimension.extent().as_static())
            .ok_or(ProofPlanningFailure::Unsupported)?;
        dimensions.push((dimension, extent.get()));
    }
    let points = finite_point_count(
        &dimensions
            .iter()
            .map(|(_, extent)| *extent)
            .collect::<Vec<_>>(),
    )
    .and_then(|points| u64::try_from(points).ok())
    .ok_or(ProofPlanningFailure::Unsupported)?;
    Ok(ResolvedIndexDomain {
        key: IndexDomainKey(dimensions),
        points,
    })
}

pub(super) fn assess_domain_group(
    region: &VerifiedIndexRegion,
    group: &IndexDomainGroup,
    ledger: &mut IndexDomainProofLedger,
) -> Result<Vec<IndexDomainProofClaim>, ProofPlanningFailure> {
    let mut reached = HashSet::new();
    let mut postorder = Vec::new();
    let mut widths = HashMap::new();
    let mut node_bytes = 0_u128;
    let mut edge_count = 0_u128;
    let mut supported = vec![true; group.obligations.len()];
    for (index, planned) in group.obligations.iter().enumerate() {
        let postorder_start = postorder.len();
        let node_bytes_start = node_bytes;
        let edge_count_start = edge_count;
        let result = plan_expression(
            region,
            predicate_expression(planned.obligation.predicate()),
            &group.domain,
            &mut reached,
            &mut postorder,
            &mut widths,
            &mut node_bytes,
            &mut edge_count,
            ledger,
        );
        match result {
            Ok(()) => {}
            Err(ProofPlanningFailure::Unsupported) => {
                for expression in postorder.drain(postorder_start..) {
                    reached.remove(&expression);
                    widths.remove(&expression);
                }
                node_bytes = node_bytes_start;
                edge_count = edge_count_start;
                supported[index] = false;
            }
            Err(exhausted @ ProofPlanningFailure::Exhausted(_)) => return Err(exhausted),
        }
    }
    if supported.iter().all(|supported| !supported) {
        return Ok(group
            .obligations
            .iter()
            .map(|_| unsupported_proof_claim())
            .collect());
    }
    let predicate_bytes = group
        .obligations
        .iter()
        .zip(&supported)
        .try_fold(0_u128, |bytes, (planned, supported)| {
            if !supported {
                return Some(bytes);
            }
            let width = *widths.get(&predicate_expression(planned.obligation.predicate()))?;
            bytes.checked_add(match planned.obligation.predicate() {
                IndexDomainPredicate::NonNegative { .. } => 8,
                IndexDomainPredicate::LessThanExtent { .. } => {
                    checked_mul(8, checked_add(byte_limbs(width).ok()?, 1).ok()?).ok()?
                }
            })
        })
        .ok_or(ProofPlanningFailure::Unsupported)?;
    let dimension_cells = (group.domain.0.len() as u128)
        .checked_mul(2)
        .ok_or(ProofPlanningFailure::Unsupported)?;
    let node_cells = (postorder.len() as u128)
        .checked_mul(2)
        .ok_or(ProofPlanningFailure::Unsupported)?;
    let cells_per_point = dimension_cells
        .checked_add(node_cells)
        .and_then(|cells| cells.checked_add(edge_count))
        .and_then(|cells| {
            cells.checked_add(supported.iter().filter(|value| **value).count() as u128)
        })
        .ok_or(ProofPlanningFailure::Unsupported)?;
    let integer_bytes_per_point = node_bytes
        .checked_add(predicate_bytes)
        .ok_or(ProofPlanningFailure::Unsupported)?;
    ledger.reserve_evaluation(
        u128::from(group.points)
            .checked_mul(cells_per_point)
            .ok_or(ProofPlanningFailure::Unsupported)?,
        u128::from(group.points)
            .checked_mul(integer_bytes_per_point)
            .ok_or(ProofPlanningFailure::Unsupported)?,
    )?;

    let mut coordinates = vec![0_u64; group.domain.0.len()];
    let mut environment = HashMap::with_capacity(group.domain.0.len());
    let mut values = HashMap::with_capacity(postorder.len());
    let mut first_counterexamples = vec![None; group.obligations.len()];
    for point_ordinal in 0..group.points {
        environment.clear();
        environment.extend(
            group
                .domain
                .0
                .iter()
                .zip(&coordinates)
                .map(|((dimension, _), coordinate)| (*dimension, *coordinate)),
        );
        values.clear();
        for expression in &postorder {
            evaluate_planned_node(region, *expression, &environment, &mut values)
                .ok_or(ProofPlanningFailure::Unsupported)?;
        }
        for (index, planned) in group.obligations.iter().enumerate() {
            if !supported[index] || first_counterexamples[index].is_some() {
                continue;
            }
            let expression = predicate_expression(planned.obligation.predicate());
            let value = values
                .get(&expression)
                .ok_or(ProofPlanningFailure::Unsupported)?;
            if !predicate_holds(planned.obligation.predicate(), planned.upper_bound, value) {
                first_counterexamples[index] = Some(point_ordinal);
            }
        }
        increment_coordinates(&mut coordinates, &group.domain.0);
    }
    group
        .obligations
        .iter()
        .zip(first_counterexamples)
        .enumerate()
        .map(|(index, (planned, counterexample))| {
            if !supported[index] {
                return Ok(unsupported_proof_claim());
            }
            if let Some(point_ordinal) = counterexample {
                let reason = match planned.obligation.predicate() {
                    IndexDomainPredicate::NonNegative { .. } => "logical-index-negative",
                    IndexDomainPredicate::LessThanExtent { .. } => {
                        "logical-index-not-less-than-extent"
                    }
                };
                let disproof =
                    IndexDomainDisproof::new(reason, encode_counterexample(point_ordinal))
                        .map_err(|_| ProofPlanningFailure::Unsupported)?;
                Ok(IndexDomainProofClaim::Disproved(
                    disproof.with_point_ordinal(point_ordinal),
                ))
            } else {
                Ok(exhaustive_proof_claim(group.points))
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn plan_expression(
    region: &VerifiedIndexRegion,
    expression: VerifiedIndexExprId,
    domain: &IndexDomainKey,
    reached: &mut HashSet<VerifiedIndexExprId>,
    postorder: &mut Vec<VerifiedIndexExprId>,
    widths: &mut HashMap<VerifiedIndexExprId, u128>,
    node_bytes: &mut u128,
    edge_count: &mut u128,
    ledger: &mut IndexDomainProofLedger,
) -> Result<(), ProofPlanningFailure> {
    if reached.contains(&expression) {
        return Ok(());
    }
    ledger.debit(ProofResource::Cells, 1)?;
    let expression_view = region
        .index_expression(expression)
        .map_err(|_| ProofPlanningFailure::Unsupported)?;
    let width = match expression_view.view() {
        IndexExprView::Constant(value) => {
            let width = integer_width(value);
            *node_bytes = checked_add(*node_bytes, copy_cost(width)?)?;
            width
        }
        IndexExprView::Dimension(dimension) => {
            if !domain
                .0
                .iter()
                .any(|(candidate, _)| *candidate == dimension)
            {
                return Err(ProofPlanningFailure::Unsupported);
            }
            *node_bytes = checked_add(*node_bytes, copy_cost(8)?)?;
            8
        }
        IndexExprView::LinearCombination { constant, terms } => {
            let mut accumulator = integer_width(constant);
            *node_bytes = checked_add(*node_bytes, copy_cost(accumulator)?)?;
            for term in terms {
                ledger.debit(ProofResource::Cells, 1)?;
                *edge_count = checked_add(*edge_count, 1)?;
                plan_expression(
                    region,
                    term.value(),
                    domain,
                    reached,
                    postorder,
                    widths,
                    node_bytes,
                    edge_count,
                    ledger,
                )?;
                // A symbolic coefficient has no width to charge and no value to
                // multiply by, so the fragment is refused by name before any
                // budget is spent on it rather than planned and then failed.
                let coefficient = integer_width(
                    term.coefficient()
                        .as_literal()
                        .ok_or(ProofPlanningFailure::Unsupported)?,
                );
                let child = *widths
                    .get(&term.value())
                    .ok_or(ProofPlanningFailure::Unsupported)?;
                let product = checked_add(coefficient, child)?;
                let next_accumulator = checked_add(accumulator.max(product), 1)?;
                *node_bytes = checked_add(
                    *node_bytes,
                    multiplication_cost(coefficient, child, product)?,
                )?;
                *node_bytes = checked_add(
                    *node_bytes,
                    addition_cost(accumulator, product, next_accumulator)?,
                )?;
                accumulator = next_accumulator;
            }
            accumulator
        }
        IndexExprView::FloorDiv { dividend, divisor } => {
            let divisor = divisor
                .as_static()
                .ok_or(ProofPlanningFailure::Unsupported)?;
            ledger.debit(ProofResource::Cells, 1)?;
            *edge_count = checked_add(*edge_count, 1)?;
            plan_expression(
                region, dividend, domain, reached, postorder, widths, node_bytes, edge_count,
                ledger,
            )?;
            let width = *widths
                .get(&dividend)
                .ok_or(ProofPlanningFailure::Unsupported)?;
            let _ = divisor;
            let result_width = checked_add(width, 1)?;
            *node_bytes = checked_add(*node_bytes, division_cost(width, result_width)?)?;
            result_width
        }
        IndexExprView::Modulo { dividend, divisor } => {
            divisor
                .as_static()
                .ok_or(ProofPlanningFailure::Unsupported)?;
            ledger.debit(ProofResource::Cells, 1)?;
            *edge_count = checked_add(*edge_count, 1)?;
            plan_expression(
                region, dividend, domain, reached, postorder, widths, node_bytes, edge_count,
                ledger,
            )?;
            let width = *widths
                .get(&dividend)
                .ok_or(ProofPlanningFailure::Unsupported)?;
            *node_bytes = checked_add(*node_bytes, division_cost(width, 8)?)?;
            8
        }
    };
    reached.insert(expression);
    widths.insert(expression, width);
    postorder.push(expression);
    Ok(())
}

pub(super) fn checked_add(left: u128, right: u128) -> Result<u128, ProofPlanningFailure> {
    left.checked_add(right)
        .ok_or(ProofPlanningFailure::Unsupported)
}

fn checked_mul(left: u128, right: u128) -> Result<u128, ProofPlanningFailure> {
    left.checked_mul(right)
        .ok_or(ProofPlanningFailure::Unsupported)
}

// These byte-work bounds follow the locked num-bigint 0.4.8 multiplication
// and division implementations. A dependency revision requires a formula
// audit. They conservatively count 8-byte limb touches, including operands,
// results, and transient work; they do not describe proof identity.
fn byte_limbs(bytes: u128) -> Result<u128, ProofPlanningFailure> {
    Ok(checked_add(bytes.max(1), 7)? / 8)
}

fn copy_cost(width: u128) -> Result<u128, ProofPlanningFailure> {
    checked_mul(16, byte_limbs(width)?)
}

fn addition_cost(left: u128, right: u128, result: u128) -> Result<u128, ProofPlanningFailure> {
    let limbs = checked_add(byte_limbs(left)?, byte_limbs(right)?)?;
    checked_mul(32, checked_add(limbs, byte_limbs(result)?)?)
}

pub(super) fn multiplication_cost(
    left: u128,
    right: u128,
    result: u128,
) -> Result<u128, ProofPlanningFailure> {
    let left = byte_limbs(left)?;
    let right = byte_limbs(right)?;
    let result = byte_limbs(result)?;
    let nonlinear = checked_mul(
        256,
        checked_mul(checked_add(left, 1)?, checked_add(right, 1)?)?,
    )?;
    checked_add(
        nonlinear,
        checked_mul(32, checked_add(checked_add(left, right)?, result)?)?,
    )
}

pub(super) fn division_cost(dividend: u128, result: u128) -> Result<u128, ProofPlanningFailure> {
    let dividend = byte_limbs(dividend)?;
    let result = byte_limbs(result)?;
    let division = checked_mul(8, checked_mul(6, checked_add(dividend, 1)?)?)?;
    checked_add(division, checked_mul(16, checked_add(result, 1)?)?)
}

fn evaluate_planned_node(
    region: &VerifiedIndexRegion,
    expression: VerifiedIndexExprId,
    environment: &HashMap<VerifiedDimensionId, u64>,
    values: &mut HashMap<VerifiedIndexExprId, BigInt>,
) -> Option<()> {
    let value = match region.index_expression(expression).ok()?.view() {
        IndexExprView::Constant(value) => decode_integer(value),
        IndexExprView::Dimension(dimension) => BigInt::from(*environment.get(&dimension)?),
        IndexExprView::LinearCombination { constant, terms } => {
            let mut total = decode_integer(constant);
            for term in terms {
                // Declines on a symbolic coefficient exactly as the arms below
                // decline on a symbolic divisor: there is no value to multiply
                // by, and `None` here refuses the evaluation rather than
                // resolving the symbol through a second authority.
                total +=
                    decode_integer(term.coefficient().as_literal()?) * values.get(&term.value())?;
            }
            total
        }
        IndexExprView::FloorDiv { dividend, divisor } => values
            .get(&dividend)?
            .div_floor(&BigInt::from(divisor.as_static()?.get())),
        IndexExprView::Modulo { dividend, divisor } => values
            .get(&dividend)?
            .mod_floor(&BigInt::from(divisor.as_static()?.get())),
    };
    values.insert(expression, value);
    Some(())
}

fn exhaustive_proof_claim(points: u64) -> IndexDomainProofClaim {
    IndexDomainProofClaim::Proved(IndexDomainProofEvidence::ExhaustiveFinite {
        points,
        derivation: EXHAUSTIVE_DERIVATION.into(),
    })
}

fn unsupported_proof_claim() -> IndexDomainProofClaim {
    IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment)
}

fn fill_unassessed(
    claims: &mut [Option<IndexDomainProofClaim>],
    exhaustion: IndexDomainProofExhaustion,
) {
    for claim in claims.iter_mut().filter(|claim| claim.is_none()) {
        *claim = Some(proof_resource_limit(exhaustion));
    }
}

pub(super) fn proof_resource_limit(
    exhaustion: IndexDomainProofExhaustion,
) -> IndexDomainProofClaim {
    IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
        resource: exhaustion.resource,
        required: exhaustion.required,
        limit: exhaustion.limit,
    })
}

pub(super) fn finite_point_count(extents: &[u64]) -> Option<u128> {
    if extents.contains(&0) {
        return Some(0);
    }
    extents.iter().try_fold(1_u128, |product, extent| {
        product.checked_mul(u128::from(*extent))
    })
}

fn integer_width(value: &IndexInteger) -> u128 {
    (value.magnitude_byte_len() as u128).max(1)
}

const fn predicate_expression(predicate: IndexDomainPredicate) -> VerifiedIndexExprId {
    match predicate {
        IndexDomainPredicate::NonNegative { expression }
        | IndexDomainPredicate::LessThanExtent { expression, .. } => expression,
    }
}

fn decode_integer(value: &IndexInteger) -> BigInt {
    value.to_bigint()
}

fn predicate_holds(
    predicate: IndexDomainPredicate,
    upper_bound: Option<u64>,
    value: &BigInt,
) -> bool {
    match predicate {
        IndexDomainPredicate::NonNegative { .. } => value >= &BigInt::zero(),
        IndexDomainPredicate::LessThanExtent { .. } => upper_bound.is_some_and(|extent| {
            value.sign() == Sign::Minus || value.to_u64().is_some_and(|value| value < extent)
        }),
    }
}

pub(super) fn resolve_extent(region: &VerifiedIndexRegion, extent: IndexExtentRef) -> Option<u64> {
    match extent {
        IndexExtentRef::Dimension(dimension) => region
            .dimension(dimension)
            .ok()?
            .extent()
            .as_static()
            .map(crate::shape::Extent::get),
        IndexExtentRef::TensorAxis { tensor, axis } => {
            let axis = usize::try_from(axis).ok()?;
            region
                .tensor(tensor)
                .ok()?
                .shape()
                .as_static()?
                .extents()
                .get(axis)
                .copied()
                .map(crate::shape::Extent::get)
        }
    }
}

fn increment_coordinates(coordinates: &mut [u64], dimensions: &[(VerifiedDimensionId, u64)]) {
    for (coordinate, (_, extent)) in coordinates.iter_mut().zip(dimensions).rev() {
        *coordinate += 1;
        if *coordinate < *extent {
            return;
        }
        *coordinate = 0;
    }
}

pub(super) fn encode_counterexample(point_ordinal: u64) -> Box<[u8]> {
    let mut output = COUNTEREXAMPLE_TAG.to_vec();
    output.extend_from_slice(&point_ordinal.to_be_bytes());
    output.into_boxed_slice()
}
