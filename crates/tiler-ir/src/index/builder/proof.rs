#![allow(
    clippy::wildcard_imports,
    reason = "a private child of `builder`, not a separate concept: every name it uses is \
defined in that parent, and enumerating them would restate the parent's imports and have to \
be restated on every change"
)]

//! The proof obligations a draft region must discharge before it is verified.
//!
//! Reachability, access bounds, write permutation, and the exhaustive
//! enumeration fallback. The invariant is that **every diagnostic is
//! collected rather than returned on the first failure**: a caller fixing one
//! unproved access should not have to rebuild to discover the next, so
//! `verify` accumulates and reports the set.

use super::*;

impl IndexRegionBuilder {
    pub(super) fn verify(&self) -> Result<VerifiedIndexRegion, Vec<IndexRegionDiagnostic>> {
        let mut diagnostics = Vec::new();
        if self.outputs.is_empty() {
            diagnostics.push(IndexRegionDiagnostic::NoOutputs);
        }
        self.verify_output_tensors(&mut diagnostics);
        let reachable_values = self.reachable_values();
        let reachable_accesses: BTreeSet<_> = reachable_values
            .iter()
            .filter_map(|i| match self.values[*i as usize].definition {
                ScalarValueDefinition::AccessRead { access } => Some(access),
                ScalarValueDefinition::OperationResult { .. } => None,
            })
            .chain(self.outputs.iter().map(|o| o.access))
            .collect();
        self.verify_inputs_are_reachable(&reachable_accesses, &mut diagnostics);
        let reachable_operations: BTreeSet<_> = reachable_values
            .iter()
            .filter_map(|value| match self.values[*value as usize].definition {
                ScalarValueDefinition::OperationResult { operation, .. } => Some(operation),
                ScalarValueDefinition::AccessRead { .. } => None,
            })
            .collect();
        let used_reductions: BTreeSet<_> = reachable_operations
            .iter()
            .filter_map(
                |operation| match &self.operations[*operation as usize].kind {
                    ScalarOperationKindData::Reduce { dimensions, .. } => {
                        Some(dimensions.iter().copied())
                    }
                    ScalarOperationKindData::Apply { .. } => None,
                },
            )
            .flatten()
            .collect();
        for output in &self.outputs {
            for dimension in &self.values[output.value as usize].free_dimensions {
                if self.dimensions[*dimension as usize].role == DomainRole::Reduction {
                    diagnostics.push(IndexRegionDiagnostic::FreeReductionDimension {
                        value: ScalarValueId {
                            owner: self.owner,
                            index: output.value,
                        },
                        dimension: DimensionId {
                            owner: self.owner,
                            index: *dimension,
                        },
                    });
                }
            }
        }
        for (i, _dimension) in self
            .dimensions
            .iter()
            .enumerate()
            .filter(|(_, d)| d.role == DomainRole::Reduction)
        {
            let index = bounded_index(i);
            if !used_reductions.contains(&index) {
                diagnostics.push(IndexRegionDiagnostic::UnusedDomainDimension {
                    dimension: DimensionId {
                        owner: self.owner,
                        index,
                    },
                });
            }
        }
        let (index_domain_predicates, partition_proofs) =
            self.verify_accesses(&reachable_accesses, &mut diagnostics);
        if !diagnostics.is_empty() {
            diagnostics.sort_by_key(|d| format!("{d:?}"));
            diagnostics.dedup();
            return Err(diagnostics);
        }
        self.compact(
            reachable_values,
            reachable_accesses,
            index_domain_predicates,
            &partition_proofs,
        )
        .map_err(|diagnostic| vec![diagnostic])
    }

    /// Groups the write accesses of every output whose roots partition it.
    ///
    /// A tensor appears only when more than one root names it, so a region
    /// nothing partitions produces an empty map and every path below it is the
    /// one that existed before partitions did. Roots are **not** deduplicated:
    /// two roots naming one access are two members occupying one rectangle,
    /// which the joint obligation refuses as an overlap. Collapsing them here
    /// would instead admit a region whose second root writes a different value
    /// to the elements the first already owns.
    pub(super) fn partitioned_outputs(&self) -> BTreeMap<u32, Vec<u32>> {
        let mut roots: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for output in &self.outputs {
            roots
                .entry(self.accesses[output.access as usize].tensor)
                .or_default()
                .push(output.access);
        }
        roots.retain(|_, accesses| accesses.len() > 1);
        roots
    }

    pub(super) fn verify_output_tensors(&self, diagnostics: &mut Vec<IndexRegionDiagnostic>) {
        for (i, _tensor) in self
            .tensors
            .iter()
            .enumerate()
            .filter(|(_, t)| t.role == TensorRole::Output)
        {
            let index = bounded_index(i);
            if !self.output_tensors.contains(&index) {
                diagnostics.push(IndexRegionDiagnostic::MissingOutputTensor {
                    tensor: TensorId {
                        owner: self.owner,
                        index,
                    },
                });
            }
        }
    }

    pub(super) fn verify_inputs_are_reachable(
        &self,
        reachable_accesses: &BTreeSet<u32>,
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) {
        for (i, _tensor) in self
            .tensors
            .iter()
            .enumerate()
            .filter(|(_, t)| t.role == TensorRole::Input)
        {
            let index = bounded_index(i);
            if !reachable_accesses
                .iter()
                .any(|a| self.accesses[*a as usize].tensor == index)
            {
                diagnostics.push(IndexRegionDiagnostic::UnusedInputTensor {
                    tensor: TensorId {
                        owner: self.owner,
                        index,
                    },
                });
            }
        }
    }
    pub(super) fn verify_accesses(
        &self,
        accesses: &BTreeSet<u32>,
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) -> (
        Vec<PendingIndexDomainPredicate>,
        BTreeMap<u32, JointPartitionProof>,
    ) {
        let mut cells = 0_u128;
        let mut integer_bytes = 0_u128;
        let mut predicates = Vec::new();
        let mut exhaustive_accesses = Vec::new();
        let partitioned = self.partitioned_outputs();
        for access_index in accesses {
            let access = &self.accesses[*access_index as usize];
            let shape = &self.tensors[access.tensor as usize].shape;
            // A partition member owes totality over its own rectangle, not over
            // the boundary, so every per-access ownership demand below is asked
            // only of a root that owns its output alone. Reading the condition
            // once here keeps the four places that consume it from drifting
            // into disagreement about which roots the joint obligation covers.
            let owns_alone =
                access.mode == AccessMode::Write && !partitioned.contains_key(&access.tensor);
            let points = self.domain_points(&access.domain);
            let predicate_start = predicates.len();
            predicates.extend(self.cheap_index_domain_predicates(
                *access_index,
                access,
                shape,
                points,
            ));
            if points == Some(0) {
                if owns_alone && self.boundary_element_count(shape) != Some(0) {
                    diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                        access: TensorAccessId {
                            owner: self.owner,
                            index: *access_index,
                        },
                    });
                }
                continue;
            }
            let IntervalVerdict {
                definitely_outside, ..
            } = self.interval_verdict(access, shape);
            // A coordinate outside every axis is only a refutation over a
            // domain that is visited. A symbolic extent whose environment
            // admits zero may not be, so the stronger claim needs the stronger
            // premise; a static domain reaching here always satisfies it,
            // because an empty one was answered above.
            if definitely_outside && self.domain_is_nonempty(&access.domain) {
                diagnostics.push(IndexRegionDiagnostic::CoordinateOutOfBounds {
                    access: TensorAccessId {
                        owner: self.owner,
                        index: *access_index,
                    },
                });
                continue;
            }
            let unresolved_bounds = predicates[predicate_start..].iter().any(|predicate| {
                matches!(
                    predicate.disposition,
                    PendingIndexDomainDisposition::Unknown(_)
                )
            });
            let unresolved_ownership = owns_alone && !self.write_is_permutation(access, shape);
            if unresolved_bounds || unresolved_ownership {
                // The finite fallback walks the domain point by point and checks
                // each coordinate against a boundary axis, so it needs an exact
                // size on both sides. An environment that determines neither
                // leaves no enumeration to budget for and no interval that
                // closed the question. A read retains its exact residual
                // predicates; a write still refuses unresolved ownership. This
                // is deliberately not a proof-resource outcome because no
                // finite enumeration was available to stop.
                //
                // A divisor the environment does not fix blocks the walk for the
                // same reason and is refused in the same place: there is no
                // arithmetic to perform at a point, and charging a budget for a
                // walk that cannot happen would report an absent proof as a
                // resource limit.
                let enumerable = points.is_some()
                    && self.boundary_extents(shape).is_some()
                    && self.coordinates_are_evaluable(&access.coordinates);
                let Some(points) = points.filter(|_| enumerable) else {
                    if unresolved_ownership {
                        diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                            access: TensorAccessId {
                                owner: self.owner,
                                index: *access_index,
                            },
                        });
                    }
                    continue;
                };
                exhaustive_accesses.push(*access_index);
                let (plan_len, bytes_per_point) = self.proof_plan_size(&access.coordinates);
                cells = cells.saturating_add(u128::from(points).saturating_mul(plan_len as u128));
                let coordinate_bytes = u128::from(points).saturating_mul(bytes_per_point.max(1));
                let dense_bytes = if owns_alone {
                    // Every axis is determined here, so a `None` is an element
                    // count this host cannot represent — the pre-existing
                    // unbounded-proof case, kept on its pre-existing path.
                    self.boundary_element_count(shape)
                        .map_or(u128::MAX, |elements| {
                            elements.div_ceil(64).saturating_mul(8) as u128
                        })
                } else {
                    0
                };
                integer_bytes =
                    integer_bytes.saturating_add(coordinate_bytes.saturating_add(dense_bytes));
            }
        }
        // Interval reasoning runs before the budget because it takes none: it
        // closes over each root's rectangle rather than over the boundary's
        // elements. Only the outputs it cannot place are scheduled for the
        // joint walk, and their cost joins the same accumulator the per-access
        // enumerations use — two independent budgets would each admit a proof
        // the pair of them exceeds.
        let mut partition_proofs = BTreeMap::new();
        let mut partition_walks = Vec::new();
        for (tensor, roots) in &partitioned {
            match self.decide_partition_by_interval(*tensor, roots, diagnostics) {
                PartitionVerdict::Interval => {
                    partition_proofs.insert(*tensor, JointPartitionProof::Interval);
                }
                PartitionVerdict::Refuted => {}
                PartitionVerdict::Enumerate => {
                    let Some(elements) = self.partition_walk_elements(*tensor, roots) else {
                        // No enumeration exists, and interval reasoning already
                        // declined. Nothing proved these roots own the output,
                        // which is the absent-proof outcome rather than a
                        // resource one — no walk was available to stop.
                        for root in roots {
                            diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                                access: TensorAccessId {
                                    owner: self.owner,
                                    index: *root,
                                },
                            });
                        }
                        continue;
                    };
                    for root in roots {
                        let access = &self.accesses[*root as usize];
                        let points = u128::from(
                            self.domain_points(&access.domain)
                                .expect("a scheduled walk has determined domain extents"),
                        );
                        let (plan_len, bytes_per_point) = self.proof_plan_size(&access.coordinates);
                        cells = cells.saturating_add(points.saturating_mul(plan_len as u128));
                        integer_bytes = integer_bytes
                            .saturating_add(points.saturating_mul(bytes_per_point.max(1)));
                    }
                    integer_bytes = integer_bytes
                        .saturating_add(elements.div_ceil(64).saturating_mul(8) as u128);
                    partition_walks.push(*tensor);
                }
            }
        }
        let admitted = with_admitted_proof_budget(
            cells,
            integer_bytes,
            MAX_EXHAUSTIVE_PROOF_CELLS,
            MAX_EXHAUSTIVE_PROOF_BYTES,
            || {
                for tensor in &partition_walks {
                    if let Some(points) = self.verify_partition_exhaustively(
                        *tensor,
                        &partitioned[tensor],
                        diagnostics,
                    ) {
                        partition_proofs
                            .insert(*tensor, JointPartitionProof::Exhaustive { points });
                    }
                }
                for access_index in &exhaustive_accesses {
                    let access = &self.accesses[*access_index as usize];
                    let shape = &self.tensors[access.tensor as usize].shape;
                    // An undetermined domain or boundary has no extent vector
                    // to walk. Its exact read predicates remain unknown; a
                    // write still fails the separate ownership requirement.
                    let Some(extents) = self.domain_extents(&access.domain) else {
                        continue;
                    };
                    let Some(axes) = self.boundary_extents(shape) else {
                        continue;
                    };
                    if extents.contains(&0) || !self.access_needs_exhaustive_proof(access, shape) {
                        continue;
                    }
                    let mut reached = BTreeSet::new();
                    for coordinate in &access.coordinates {
                        self.mark_expr(*coordinate, &mut reached);
                    }
                    let plan = reached.into_iter().collect::<Vec<_>>();
                    let discharged = self.verify_access_exhaustively(
                        *access_index,
                        &plan,
                        &extents,
                        &axes,
                        access.mode == AccessMode::Write
                            && !partitioned.contains_key(&access.tensor),
                        diagnostics,
                    );
                    if discharged {
                        let evidence = IndexDomainEvidence::ExhaustiveFinite {
                            points: enumerated_points(self.domain_points(&access.domain)),
                        };
                        for predicate in predicates
                            .iter_mut()
                            .filter(|predicate| predicate.access == *access_index)
                        {
                            if matches!(
                                predicate.disposition,
                                PendingIndexDomainDisposition::Unknown(_)
                            ) {
                                predicate.disposition =
                                    PendingIndexDomainDisposition::Discharged(evidence);
                            }
                        }
                    }
                }
            },
        );
        if let Err(excess) = admitted {
            for predicate in predicates.iter_mut().filter(|predicate| {
                exhaustive_accesses.contains(&predicate.access)
                    && matches!(
                        predicate.disposition,
                        PendingIndexDomainDisposition::Unknown(_)
                    )
            }) {
                predicate.disposition =
                    PendingIndexDomainDisposition::Unknown(excess.unknown_reason());
            }
            let mut blocked_write = false;
            for access_index in &exhaustive_accesses {
                let access = &self.accesses[*access_index as usize];
                if access.mode == AccessMode::Write
                    && !partitioned.contains_key(&access.tensor)
                    && !self
                        .write_is_permutation(access, &self.tensors[access.tensor as usize].shape)
                {
                    blocked_write = true;
                    diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                        access: TensorAccessId {
                            owner: self.owner,
                            index: *access_index,
                        },
                    });
                }
            }
            // A scheduled joint walk that the budget stopped leaves its output
            // owned by nothing. Reported per root rather than per output,
            // because the root is the entity the caller can act on and the
            // resource diagnostic beside it already names the boundary that was
            // too large to walk.
            for tensor in &partition_walks {
                blocked_write = true;
                for root in &partitioned[tensor] {
                    diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                        access: TensorAccessId {
                            owner: self.owner,
                            index: *root,
                        },
                    });
                }
            }
            if blocked_write || !diagnostics.is_empty() {
                diagnostics.push(excess.diagnostic());
            }
        }
        (predicates, partition_proofs)
    }

    fn cheap_index_domain_predicates(
        &self,
        access_index: u32,
        access: &AccessData,
        shape: &SourcedShape,
        points: Option<u64>,
    ) -> Vec<PendingIndexDomainPredicate> {
        let mut predicates = Vec::with_capacity(access.coordinates.len().saturating_mul(2));
        for (axis, (coordinate, extent)) in access
            .coordinates
            .iter()
            .copied()
            .zip(shape.extents())
            .enumerate()
        {
            let vacuous = points == Some(0);
            let interval = self.expressions[coordinate as usize].interval.as_ref();
            let axis_interval = self.extent_interval(&extent);
            let domain_dimension = match *self.expressions[coordinate as usize].node {
                IndexNode::Dimension(dimension) if access.domain.contains(&dimension) => {
                    Some(dimension)
                }
                IndexNode::Constant(_)
                | IndexNode::Dimension(_)
                | IndexNode::LinearCombination { .. }
                | IndexNode::FloorDiv { .. }
                | IndexNode::Modulo { .. } => None,
            };
            let structural = domain_dimension.is_some_and(|dimension| {
                self.extents_proved_equal(&self.dimensions[dimension as usize].extent, &extent)
            });
            let nonnegative = if vacuous {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::VacuousEmptyDomain)
            } else if interval.is_some_and(|(minimum, _)| minimum >= &BigInt::zero()) {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval)
            } else if structural {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::ProvedExtentEquality)
            } else {
                IndexDomainEvidence::Unknown
            };
            let less_than = if vacuous {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::VacuousEmptyDomain)
            } else if interval
                .zip(axis_interval)
                .is_some_and(|((_, maximum), axis)| maximum < &BigInt::from(axis.lower))
            {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval)
            } else if structural {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::ProvedExtentEquality)
            } else {
                IndexDomainEvidence::Unknown
            };
            for (bound, evidence) in [
                (PendingIndexDomainBound::NonNegative, nonnegative),
                (PendingIndexDomainBound::LessThanAxis, less_than),
            ] {
                predicates.push(PendingIndexDomainPredicate {
                    access: access_index,
                    axis: bounded_index(axis),
                    bound,
                    disposition: match evidence {
                        IndexDomainEvidence::Unknown => PendingIndexDomainDisposition::Unknown(
                            IndexDomainUnknownReason::InsufficientFacts,
                        ),
                        evidence => PendingIndexDomainDisposition::Discharged(evidence),
                    },
                });
            }
        }
        predicates
    }

    /// Reads one access's coordinate intervals against its tensor's axes.
    ///
    /// The two answers are independent and neither implies the other's
    /// negation: an interval that overlaps a boundary proves nothing either
    /// way, while one lying wholly outside refutes the access.
    ///
    /// A symbolic axis is compared against the *side of its own interval that
    /// makes each answer sound*, and the two sides are different. Proving a
    /// coordinate in bounds needs it below the axis in **every** model, so it is
    /// compared against the axis's lower bound. Refuting one needs it at or
    /// above the axis in every model, so that is compared against the upper
    /// bound. A static axis has a one-point interval and both comparisons
    /// collapse to the literal, which is why this reads as one rule rather than
    /// a symbolic special case.
    pub(super) fn interval_verdict(
        &self,
        access: &AccessData,
        shape: &SourcedShape,
    ) -> IntervalVerdict {
        let mut verdict = IntervalVerdict {
            interval_proved: true,
            definitely_outside: false,
        };
        for (coordinate, extent) in access.coordinates.iter().zip(shape.extents()) {
            let Some((min, max)) = &self.expressions[*coordinate as usize].interval else {
                verdict.interval_proved = false;
                continue;
            };
            let Some(axis) = self.extent_interval(&extent) else {
                // The environment bounds this axis nowhere, so nothing about it
                // is provable and nothing about it is refutable either.
                verdict.interval_proved = false;
                continue;
            };
            if max < &BigInt::zero() || min >= &BigInt::from(axis.upper) {
                verdict.definitely_outside = true;
            }
            if min < &BigInt::zero() || max >= &BigInt::from(axis.lower) {
                verdict.interval_proved = false;
            }
        }
        verdict
    }

    /// Returns whether every coordinate is a domain dimension whose extent the
    /// environment proves equal to the axis it indexes.
    ///
    /// The sound argument interval propagation cannot express. A coordinate
    /// that *is* `IndexNode::Dimension(d)`, with `d` iterated by this access,
    /// ranges over `[0, extent(d))` by construction; when the environment
    /// proves `extent(d)` and the axis are one extent, the coordinate is below
    /// the axis in every model. Nothing about either interval is needed, which
    /// is exactly why `[n] -> [n]` with nothing determined is provable here and
    /// nowhere else: `n`'s interval is the whole extent domain, so `max(i)` is
    /// never below the axis's floor.
    ///
    /// Deliberately *not* a permutation check. Two axes may name the same
    /// dimension and still each be in bounds; covering a boundary exactly once
    /// is a separate obligation that [`Self::write_is_permutation`] owns, and
    /// conflating them would either refuse a legal read or let a write claim
    /// ownership it has not shown.
    pub(super) fn coordinates_are_bounded_dimensions(
        &self,
        access: &AccessData,
        shape: &SourcedShape,
    ) -> bool {
        if access.coordinates.len() != shape.rank() {
            return false;
        }
        access
            .coordinates
            .iter()
            .zip(shape.extents())
            .all(|(coordinate, extent)| {
                let IndexNode::Dimension(d) = *self.expressions[*coordinate as usize].node else {
                    return false;
                };
                access.domain.contains(&d)
                    && self.extents_proved_equal(&self.dimensions[d as usize].extent, &extent)
            })
    }

    /// Returns whether this access's coordinates are proved in bounds without
    /// enumerating its domain.
    ///
    /// Interval propagation first and the structural equality argument second.
    /// The order is load-bearing for the *evidence*, not for soundness: an
    /// access interval propagation already proved must keep recording
    /// [`BoundsProof::Interval`], or the retained evidence of existing regions
    /// silently changes meaning.
    pub(super) fn bounds_proved_without_enumeration(
        &self,
        access: &AccessData,
        shape: &SourcedShape,
    ) -> bool {
        self.interval_verdict(access, shape).interval_proved
            || self.coordinates_are_bounded_dimensions(access, shape)
    }

    /// Returns whether the cheap proofs alone leave this access unproved.
    ///
    /// Exactly the condition the verifier falls through on, read from
    /// [`Self::bounds_proved_without_enumeration`] rather than recomputed, so
    /// the enumeration pass cannot come to disagree with the pass that decided
    /// one was needed.
    pub(super) fn access_needs_exhaustive_proof(
        &self,
        access: &AccessData,
        shape: &SourcedShape,
    ) -> bool {
        !self.bounds_proved_without_enumeration(access, shape)
            || (access.mode == AccessMode::Write && !self.write_is_permutation(access, shape))
    }

    pub(super) fn proof_plan_size(&self, coordinates: &[u32]) -> (usize, u128) {
        let mut visited = vec![false; self.expressions.len()];
        let mut pending = coordinates.to_vec();
        let mut count = 0_usize;
        let mut integer_bytes = 0_u128;
        while let Some(expression) = pending.pop() {
            if std::mem::replace(&mut visited[expression as usize], true) {
                continue;
            }
            count = count.saturating_add(1);
            integer_bytes = integer_bytes.saturating_add(self.expression_integer_bytes(expression));
            match &*self.expressions[expression as usize].node {
                IndexNode::LinearCombination { terms, .. } => {
                    pending.extend(terms.iter().map(|term| term.value));
                }
                IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => {
                    pending.push(*dividend);
                }
                IndexNode::Constant(_) | IndexNode::Dimension(_) => {}
            }
        }
        (count, integer_bytes)
    }

    /// Returns the exact number of points a domain visits, when that is known.
    ///
    /// `None` for a domain with a symbolic extent the environment does not
    /// determine. That is not zero and not "very many": it is the case where no
    /// enumeration exists, and collapsing it into either would be the silent
    /// approximation the contract forbids.
    /// Returns each domain dimension's exact extent, when all of them are known.
    ///
    /// This is what an enumeration walks. It exists separately from
    /// [`Self::domain_points`] because a point count that saturated would still
    /// let a caller enumerate, and an extent vector that is missing one entry
    /// must not.
    pub(super) fn domain_extents(&self, domain: &[u32]) -> Option<Vec<u64>> {
        domain
            .iter()
            .map(|dimension| self.determined_extent(*dimension))
            .collect()
    }

    pub(super) fn domain_points(&self, domain: &[u32]) -> Option<u64> {
        // One empty dimension makes the domain empty whatever the others are,
        // so it is answered before every extent is required to be determined.
        if domain
            .iter()
            .any(|dimension| self.determined_extent(*dimension) == Some(0))
        {
            return Some(0);
        }
        domain.iter().try_fold(1_u64, |points, dimension| {
            Some(points.saturating_mul(self.determined_extent(*dimension)?))
        })
    }

    pub(super) fn expression_integer_bytes(&self, expression: u32) -> u128 {
        let data = &self.expressions[expression as usize];
        if data.interval.is_none() {
            return u128::MAX;
        }
        let magnitude_bound = if let IndexNode::LinearCombination { constant, terms } = &*data.node
        {
            let mut bound = constant.0.abs();
            for term in terms {
                let (minimum, maximum) = self.expressions[term.value as usize]
                    .interval
                    .as_ref()
                    .expect("a linear interval requires every child interval");
                let child_bound = minimum.abs().max(maximum.abs());
                let Ok(product) = checked_index_product(&term.coefficient.0.abs(), &child_bound)
                else {
                    return u128::MAX;
                };
                if checked_index_add_assign(&mut bound, &product).is_err() {
                    return u128::MAX;
                }
            }
            bound
        } else {
            let Some((minimum, maximum)) = &data.interval else {
                return u128::MAX;
            };
            minimum.abs().max(maximum.abs())
        };
        u128::try_from(magnitude_bound.to_signed_bytes_be().len().max(1)).unwrap_or(u128::MAX)
    }

    /// Walks one access's domain, proving bounds and — when `owns_alone` —
    /// that this write covers its whole boundary exactly once.
    ///
    /// `owns_alone` is false for a root whose output several roots partition.
    /// Such a root still needs its coordinates walked for bounds, but a
    /// per-access coverage bitset would demand it cover the whole boundary,
    /// which is precisely what a partition member does not do and must not be
    /// refused for. Its ownership is decided once for the whole root set by
    /// [`Self::verify_partition_exhaustively`].
    pub(super) fn verify_access_exhaustively(
        &self,
        access_index: u32,
        expression_plan: &[u32],
        extents: &[u64],
        axes: &[u64],
        owns_alone: bool,
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) -> bool {
        let access = &self.accesses[access_index as usize];
        let shape = &self.tensors[access.tensor as usize].shape;
        let Some(elements) = self.boundary_element_count(shape) else {
            if owns_alone {
                diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                    access: TensorAccessId {
                        owner: self.owner,
                        index: access_index,
                    },
                });
            }
            return false;
        };
        // Fail closed rather than walking: the caller's `enumerable` gate
        // already excluded an undetermined divisor, so reaching this returns
        // "not proved" with no diagnostic — a write's ownership requirement is
        // refused separately by that same gate, and inventing an
        // out-of-bounds refutation here would be a claim nothing established.
        let Some(divisors) = self.plan_divisors(expression_plan) else {
            return false;
        };
        let mut seen = owns_alone.then(|| vec![0_u64; elements.div_ceil(64)]);
        let mut point = vec![0_u64; extents.len()];
        loop {
            let assignments: BTreeMap<_, _> = access
                .domain
                .iter()
                .copied()
                .zip(point.iter().copied())
                .collect();
            let evaluated = self.evaluate_expressions(expression_plan, &assignments, &divisors);
            let mut linear = 0_usize;
            let mut in_bounds = true;
            for (coordinate, extent) in access.coordinates.iter().zip(axes) {
                let Some(value) = evaluated.get(coordinate).and_then(ToPrimitive::to_usize) else {
                    in_bounds = false;
                    break;
                };
                let Ok(axis_extent) = usize::try_from(*extent) else {
                    in_bounds = false;
                    break;
                };
                if value >= axis_extent {
                    in_bounds = false;
                    break;
                }
                let Some(next) = linear
                    .checked_mul(axis_extent)
                    .and_then(|base| base.checked_add(value))
                else {
                    in_bounds = false;
                    break;
                };
                linear = next;
            }
            if !in_bounds {
                diagnostics.push(IndexRegionDiagnostic::CoordinateOutOfBounds {
                    access: TensorAccessId {
                        owner: self.owner,
                        index: access_index,
                    },
                });
                return false;
            }
            if let Some(bits) = &mut seen {
                let word = linear / 64;
                let mask = 1_u64 << (linear % 64);
                if bits.get(word).is_none_or(|bits| bits & mask != 0) {
                    diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                        access: TensorAccessId {
                            owner: self.owner,
                            index: access_index,
                        },
                    });
                    return false;
                }
                bits[word] |= mask;
            }
            if !advance_point(&mut point, extents) {
                break;
            }
        }
        if let Some(bits) = seen
            && (0..elements).any(|index| bits[index / 64] & (1_u64 << (index % 64)) == 0)
        {
            diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                access: TensorAccessId {
                    owner: self.owner,
                    index: access_index,
                },
            });
            return false;
        }
        true
    }

    /// Returns whether every expression reachable from these coordinates has a
    /// value at a domain point.
    ///
    /// The only way it can fail is a semi-affine divisor the environment does
    /// not pin to one value: with no divisor there is no quotient, and a walk
    /// that produced no value for a coordinate would be indistinguishable, to
    /// [`Self::verify_access_exhaustively`], from a coordinate that landed out
    /// of bounds. Deciding it before the walk is what keeps a missing proof from
    /// being reported as a refutation.
    pub(super) fn coordinates_are_evaluable(&self, coordinates: &[u32]) -> bool {
        let mut reached = BTreeSet::new();
        for coordinate in coordinates {
            self.mark_expr(*coordinate, &mut reached);
        }
        reached.into_iter().all(
            |expression| match &*self.expressions[expression as usize].node {
                IndexNode::FloorDiv { divisor, .. } | IndexNode::Modulo { divisor, .. } => {
                    self.determined(divisor).is_some()
                }
                IndexNode::Constant(_)
                | IndexNode::Dimension(_)
                | IndexNode::LinearCombination { .. } => true,
            },
        )
    }

    /// Resolves every divisor an enumeration plan will need.
    ///
    /// `None` when one of them is undetermined, which
    /// [`Self::coordinates_are_evaluable`] already excluded before any budget
    /// was taken. Resolving them once, up front, is what lets the point loop
    /// below be total arithmetic rather than a per-point lookup that could fail
    /// halfway through a walk.
    fn plan_divisors(&self, plan: &[u32]) -> Option<BTreeMap<u32, u64>> {
        let mut divisors = BTreeMap::new();
        for index in plan {
            match &*self.expressions[*index as usize].node {
                IndexNode::FloorDiv { divisor, .. } | IndexNode::Modulo { divisor, .. } => {
                    divisors.insert(*index, self.determined(divisor)?);
                }
                IndexNode::Constant(_)
                | IndexNode::Dimension(_)
                | IndexNode::LinearCombination { .. } => {}
            }
        }
        Some(divisors)
    }

    pub(super) fn evaluate_expressions(
        &self,
        plan: &[u32],
        dimensions: &BTreeMap<u32, u64>,
        divisors: &BTreeMap<u32, u64>,
    ) -> BTreeMap<u32, BigInt> {
        let mut values: BTreeMap<u32, BigInt> = BTreeMap::new();
        for index in plan {
            let expression = &self.expressions[*index as usize];
            let value = match &*expression.node {
                IndexNode::Constant(value) => value.0.clone(),
                IndexNode::Dimension(dimension) => {
                    BigInt::from(dimensions.get(dimension).copied().unwrap_or(0))
                }
                IndexNode::LinearCombination { constant, terms } => {
                    terms.iter().fold(constant.0.clone(), |sum, term| {
                        sum + &term.coefficient.0 * &values[&term.value]
                    })
                }
                IndexNode::FloorDiv { dividend, .. } => {
                    values[dividend].div_floor(&BigInt::from(divisors[index]))
                }
                IndexNode::Modulo { dividend, .. } => {
                    values[dividend].mod_floor(&BigInt::from(divisors[index]))
                }
            };
            values.insert(*index, value);
        }
        values
    }
    /// Returns whether this write's coordinates are a dimension permutation
    /// that covers its boundary exactly once.
    ///
    /// The per-axis obligation is that the dimension written along and the axis
    /// written into are the *same size*. With both static that is a literal
    /// comparison; with either symbolic it is a question only the constraint
    /// environment can answer, and [`Self::extents_proved_equal`] answers it
    /// one-sidedly — an unproved equality is not a permutation, never a proved
    /// disequality. That is what lets a dynamically shaped output be written:
    /// `y[i] = f(i)` over a domain sized `n` covers a boundary sized `n`
    /// whatever value `n` takes, and the environment proves that from the
    /// symbol alone.
    pub(super) fn write_is_permutation(&self, access: &AccessData, shape: &SourcedShape) -> bool {
        if access.coordinates.len() != access.domain.len() || shape.rank() != access.domain.len() {
            return false;
        }
        let mut seen = BTreeSet::new();
        for (coordinate, extent) in access.coordinates.iter().zip(shape.extents()) {
            let IndexNode::Dimension(d) = *self.expressions[*coordinate as usize].node else {
                return false;
            };
            if !access.domain.contains(&d)
                || !self.extents_proved_equal(&self.dimensions[d as usize].extent, &extent)
                || !seen.insert(d)
            {
                return false;
            }
        }
        true
    }

    /// Reads one coordinate as a domain dimension displaced by a literal.
    ///
    /// The whole vocabulary interval reasoning admits for a partition, and it
    /// is deliberately narrow: `d`, `d + c`, and a bare `c`, each with a unit
    /// coefficient and a non-negative displacement. That is exactly the form
    /// whose image over a dimension is a *contiguous* range, which is what
    /// makes a rectangle the right description of the root's partition. A
    /// coordinate outside it — a scaled dimension, a quotient, a sum of two
    /// dimensions — may still be a legal partition member, and it is not
    /// refused here: it returns `None`, and the joint enumeration decides it.
    ///
    /// A zero displacement arrives as [`IndexNode::Dimension`] rather than as a
    /// one-term combination, because `linear_combination` normalizes `1 * d`
    /// back to `d` before interning; the combination arm therefore never has to
    /// consider a zero constant.
    fn coordinate_offset_dimension(&self, expression: u32) -> Option<(u64, Option<u32>)> {
        match &*self.expressions[expression as usize].node {
            IndexNode::Constant(value) => value.0.to_u64().map(|offset| (offset, None)),
            IndexNode::Dimension(dimension) => Some((0, Some(*dimension))),
            IndexNode::LinearCombination { constant, terms } => {
                let [term] = terms.as_slice() else {
                    return None;
                };
                if term.coefficient.0 != BigInt::from(1_u8) {
                    return None;
                }
                let IndexNode::Dimension(dimension) = *self.expressions[term.value as usize].node
                else {
                    return None;
                };
                constant.0.to_u64().map(|offset| (offset, Some(dimension)))
            }
            IndexNode::FloorDiv { .. } | IndexNode::Modulo { .. } => None,
        }
    }

    /// Places one write root as a rectangle of half-open coordinate ranges.
    ///
    /// `None` means "interval reasoning cannot place this root", never "this
    /// root is unsound": an undetermined extent, a coordinate outside the
    /// displaced-dimension vocabulary, a rectangle the boundary does not
    /// contain, or a dimension used twice all decline rather than refute. Every
    /// one of them is a case the joint enumeration can still decide, and
    /// turning a declined placement into a refusal would refuse legal regions
    /// for the mechanism's convenience.
    ///
    /// The root's own injectivity is established here rather than assumed: each
    /// axis consumes at most one domain dimension, and the consumed set is the
    /// whole domain. Two distinct domain points then differ in some dimension,
    /// that dimension appears in exactly one axis, and the unit coefficient
    /// carries the difference into that coordinate — so the point-to-coordinate
    /// map is injective and the rectangle's volume *is* the number of distinct
    /// elements this root writes. Coverage arithmetic downstream depends on
    /// that equality, so it cannot be left to the enumeration that does not run.
    fn write_partition_box(
        &self,
        access: &AccessData,
        shape: &SourcedShape,
    ) -> Option<Vec<(u64, u64)>> {
        if access.coordinates.len() != shape.rank() {
            return None;
        }
        let axes = self.boundary_extents(shape)?;
        let mut consumed = BTreeSet::new();
        let mut ranges = Vec::with_capacity(access.coordinates.len());
        for (coordinate, axis) in access.coordinates.iter().zip(&axes) {
            let (offset, dimension) = self.coordinate_offset_dimension(*coordinate)?;
            let span = match dimension {
                Some(dimension) => {
                    if !access.domain.contains(&dimension) || !consumed.insert(dimension) {
                        return None;
                    }
                    self.determined_extent(dimension)?
                }
                None => 1,
            };
            let end = offset.checked_add(span)?;
            if end > *axis {
                return None;
            }
            ranges.push((offset, end));
        }
        // `consumed` is a subset of the domain by the containment check above,
        // so equal cardinality is equal sets.
        if consumed.len() != access.domain.len() {
            return None;
        }
        Some(ranges)
    }

    /// Decides one output's joint partition obligation by interval reasoning.
    ///
    /// Disjointness first, then coverage, and the order is not an optimization.
    /// Two axis-aligned rectangles intersect exactly when their ranges overlap
    /// on *every* axis, so one separating axis refutes an intersection and the
    /// absence of one establishes it — the test is exact in both directions
    /// rather than conservative in either. Coverage is then the volume
    /// identity: rectangles that are pairwise disjoint and contained in the
    /// boundary have a union of exactly the summed volume, so a sum equal to
    /// the boundary's element count means the union *is* the boundary. Applied
    /// without the disjointness premise the same sum would admit a set that
    /// double-covers one element and leaves another bare, which is why
    /// coverage is derived from disjointness rather than checked beside it.
    ///
    /// Containment also makes the summed volume an upper bound, so an inequality
    /// can only be a shortfall — which is what lets the mismatch be reported as
    /// an uncovered boundary rather than as an unexplained arithmetic
    /// disagreement.
    fn decide_partition_by_interval(
        &self,
        tensor: u32,
        roots: &[u32],
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) -> PartitionVerdict {
        let shape = &self.tensors[tensor as usize].shape;
        let Some(elements) = self.boundary_element_count(shape) else {
            return PartitionVerdict::Enumerate;
        };
        let mut boxes = Vec::with_capacity(roots.len());
        for root in roots {
            let Some(placed) = self.write_partition_box(&self.accesses[*root as usize], shape)
            else {
                return PartitionVerdict::Enumerate;
            };
            boxes.push(placed);
        }
        for (position, left) in boxes.iter().enumerate() {
            for right in &boxes[position.saturating_add(1)..] {
                let separated = left.iter().zip(right).any(
                    |((left_start, left_end), (right_start, right_end))| {
                        left_end <= right_start || right_end <= left_start
                    },
                );
                if !separated {
                    diagnostics.push(IndexRegionDiagnostic::OutputPartitionRangesOverlap {
                        tensor: TensorId {
                            owner: self.owner,
                            index: tensor,
                        },
                    });
                    return PartitionVerdict::Refuted;
                }
            }
        }
        let covered = boxes.iter().fold(0_u128, |total, placed| {
            total.saturating_add(placed.iter().fold(1_u128, |volume, (start, end)| {
                volume.saturating_mul(u128::from(end.saturating_sub(*start)))
            }))
        });
        if covered != elements as u128 {
            diagnostics.push(IndexRegionDiagnostic::OutputPartitionUncovered {
                tensor: TensorId {
                    owner: self.owner,
                    index: tensor,
                },
            });
            return PartitionVerdict::Refuted;
        }
        PartitionVerdict::Interval
    }

    /// Returns the boundary element count a joint walk would need, when every
    /// part of that walk is available.
    ///
    /// `None` when the boundary or any root's domain is undetermined, or when a
    /// divisor the environment does not fix leaves a coordinate with no value
    /// at a point. Decided before any budget is taken, for the reason the
    /// per-access gate states: charging a budget for a walk that cannot happen
    /// would report an absent proof as a resource limit.
    fn partition_walk_elements(&self, tensor: u32, roots: &[u32]) -> Option<usize> {
        let shape = &self.tensors[tensor as usize].shape;
        self.boundary_extents(shape)?;
        let elements = self.boundary_element_count(shape)?;
        for root in roots {
            let access = &self.accesses[*root as usize];
            self.domain_extents(&access.domain)?;
            self.domain_points(&access.domain)?;
            if !self.coordinates_are_evaluable(&access.coordinates) {
                return None;
            }
        }
        Some(elements)
    }

    /// Decides one output's joint partition obligation by enumeration.
    ///
    /// One bitset shared across every root, which is what makes this a *joint*
    /// proof rather than a sequence of per-root ones: a bit already set is a
    /// second root writing an element the first owns, and a bit still clear
    /// after every root has walked is an element nothing wrote. Returns the
    /// enumerated point count, or `None` after pushing the diagnostic that
    /// names which of those two happened — every `None` path here records a
    /// refusal, because a silent one would let the region build with an
    /// output no proof covers.
    fn verify_partition_exhaustively(
        &self,
        tensor: u32,
        roots: &[u32],
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) -> Option<u64> {
        let tensor_id = TensorId {
            owner: self.owner,
            index: tensor,
        };
        let shape = &self.tensors[tensor as usize].shape;
        // The scheduler proved each of these available before budgeting the
        // walk, so a `None` here is a fail-closed floor rather than a reachable
        // path; it still refuses rather than assuming.
        let (Some(elements), Some(axes)) = (
            self.boundary_element_count(shape),
            self.boundary_extents(shape),
        ) else {
            return self.refuse_unproved_partition(roots, diagnostics);
        };
        let mut seen = vec![0_u64; elements.div_ceil(64)];
        let mut walked = 0_u64;
        for root in roots {
            let access = &self.accesses[*root as usize];
            let mut reached = BTreeSet::new();
            for coordinate in &access.coordinates {
                self.mark_expr(*coordinate, &mut reached);
            }
            let plan = reached.into_iter().collect::<Vec<_>>();
            let (Some(extents), Some(divisors)) = (
                self.domain_extents(&access.domain),
                self.plan_divisors(&plan),
            ) else {
                return self.refuse_unproved_partition(roots, diagnostics);
            };
            // An empty root visits no point and therefore owns nothing. That is
            // not a refusal on its own: the elements it would have covered are
            // left clear, and the coverage scan below reports them.
            if extents.contains(&0) {
                continue;
            }
            let mut point = vec![0_u64; extents.len()];
            loop {
                let assignments: BTreeMap<_, _> = access
                    .domain
                    .iter()
                    .copied()
                    .zip(point.iter().copied())
                    .collect();
                let evaluated = self.evaluate_expressions(&plan, &assignments, &divisors);
                let mut linear = 0_usize;
                for (coordinate, extent) in access.coordinates.iter().zip(&axes) {
                    let Some(value) = evaluated.get(coordinate).and_then(ToPrimitive::to_usize)
                    else {
                        return self.refuse_out_of_bounds(*root, diagnostics);
                    };
                    let Ok(axis_extent) = usize::try_from(*extent) else {
                        return self.refuse_out_of_bounds(*root, diagnostics);
                    };
                    if value >= axis_extent {
                        return self.refuse_out_of_bounds(*root, diagnostics);
                    }
                    let Some(next) = linear
                        .checked_mul(axis_extent)
                        .and_then(|base| base.checked_add(value))
                    else {
                        return self.refuse_out_of_bounds(*root, diagnostics);
                    };
                    linear = next;
                }
                let word = linear / 64;
                let mask = 1_u64 << (linear % 64);
                if seen.get(word).is_none_or(|bits| bits & mask != 0) {
                    diagnostics.push(IndexRegionDiagnostic::OutputPartitionDoubleWritten {
                        tensor: tensor_id,
                    });
                    return None;
                }
                seen[word] |= mask;
                walked = walked.saturating_add(1);
                if !advance_point(&mut point, &extents) {
                    break;
                }
            }
        }
        if (0..elements).any(|index| seen[index / 64] & (1_u64 << (index % 64)) == 0) {
            diagnostics.push(IndexRegionDiagnostic::OutputPartitionUncovered { tensor: tensor_id });
            return None;
        }
        Some(walked)
    }

    fn refuse_unproved_partition(
        &self,
        roots: &[u32],
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) -> Option<u64> {
        for root in roots {
            diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                access: TensorAccessId {
                    owner: self.owner,
                    index: *root,
                },
            });
        }
        None
    }

    fn refuse_out_of_bounds(
        &self,
        root: u32,
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) -> Option<u64> {
        diagnostics.push(IndexRegionDiagnostic::CoordinateOutOfBounds {
            access: TensorAccessId {
                owner: self.owner,
                index: root,
            },
        });
        None
    }
}

/// What interval reasoning concluded about one output's write roots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PartitionVerdict {
    /// The rectangles are pairwise disjoint and cover the boundary exactly.
    Interval,
    /// Interval reasoning declined; the joint enumeration must decide.
    Enumerate,
    /// Interval reasoning refuted the set and recorded its diagnostic.
    Refuted,
}
