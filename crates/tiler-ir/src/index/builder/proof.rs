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
        self.verify_accesses(&reachable_accesses, &mut diagnostics);
        if !diagnostics.is_empty() {
            diagnostics.sort_by_key(|d| format!("{d:?}"));
            diagnostics.dedup();
            return Err(diagnostics);
        }
        self.compact(reachable_values, reachable_accesses)
            .map_err(|diagnostic| vec![diagnostic])
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
    /// Returns the refusal that fits this access's mode.
    ///
    /// A write left unproved has not been shown to own its output; a read left
    /// unproved has not been shown to stay in bounds. Both are refusals in
    /// `docs/ir.md`'s taxonomy, and neither is the proof-resource diagnostic,
    /// which the same contract defines as meaning an enumeration stopped.
    ///
    /// Matched exhaustively rather than tested against `Write`, so a third
    /// access mode is a build error here instead of silently inheriting the
    /// read's refusal.
    pub(super) fn unproved(&self, access_index: u32, mode: AccessMode) -> IndexRegionDiagnostic {
        let access = TensorAccessId {
            owner: self.owner,
            index: access_index,
        };
        match mode {
            AccessMode::Write => IndexRegionDiagnostic::WriteOwnershipNotProven { access },
            AccessMode::Read => IndexRegionDiagnostic::BoundsNotProven { access },
        }
    }

    /// Returns the first extent this access depends on that the environment
    /// bounds nowhere, rendered as its symbol.
    ///
    /// **The discriminator is the upper bound, not determinacy, and not a
    /// missing interval.** An extent the environment confines to `[2, 8]` is
    /// not *determined*, so no enumeration can walk it — but it is bounded, and
    /// the refusal that follows is a proof that did not close rather than a
    /// fact nobody stated. An extent nothing constrains is the second case, and
    /// only that one is fixed by adding a constraint.
    ///
    /// Testing for a *missing* interval would never fire: the constraint solver
    /// seeds every symbol at the whole extent domain and narrows from there, so
    /// an unconstrained symbol has an interval reaching the domain ceiling
    /// rather than no interval at all. `ExtentInterval::states_no_upper_bound`
    /// owns that condition, beside the constant that defines the ceiling.
    ///
    /// Both the boundary axes and the iterated domain's extents are consulted,
    /// because either can be the unbounded one and a caller told about the
    /// wrong half would constrain the wrong symbol. Boundary first, matching
    /// the order the interval verdict walks them in.
    pub(super) fn unbounded_extent_symbol(
        &self,
        access: &AccessData,
        shape: &SourcedShape,
    ) -> Option<String> {
        let boundary = shape.extents().collect::<Vec<_>>();
        let domain = access
            .domain
            .iter()
            .map(|dimension| self.dimensions[*dimension as usize].extent.clone());
        boundary
            .into_iter()
            .chain(domain)
            .filter(|extent| extent.symbol().is_some())
            .find(|extent| {
                self.extent_interval(extent)
                    .is_none_or(|interval| interval.states_no_upper_bound())
            })
            .and_then(|extent| extent.symbol().map(ToString::to_string))
    }

    pub(super) fn verify_accesses(
        &self,
        accesses: &BTreeSet<u32>,
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) {
        let mut cells = 0_u128;
        let mut integer_bytes = 0_u128;
        for access_index in accesses {
            let access = &self.accesses[*access_index as usize];
            let shape = &self.tensors[access.tensor as usize].shape;
            let points = self.domain_points(&access.domain);
            if points == Some(0) {
                if access.mode == AccessMode::Write && self.boundary_element_count(shape) != Some(0)
                {
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
                interval_proved,
                definitely_outside,
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
            // A proven permutation implies the structural bounds argument —
            // both require every coordinate to be a domain dimension whose
            // extent the environment proves equal to its axis, and the
            // permutation additionally requires those dimensions distinct — so
            // a write that owns its boundary no longer has to satisfy the
            // interval conjunct separately. That is a subsumption rather than a
            // relaxation: the bounds obligation is still discharged, by the
            // argument that actually holds.
            if !(interval_proved || self.coordinates_are_bounded_dimensions(access, shape))
                || (access.mode == AccessMode::Write && !self.write_is_permutation(access, shape))
            {
                // The finite fallback walks the domain point by point and checks
                // each coordinate against a boundary axis, so it needs an exact
                // size on both sides. An environment that determines neither
                // leaves no enumeration to budget for and no interval that
                // closed the question. Refusing here is what keeps a symbolic
                // extent from becoming an unproved escape hatch; it is a
                // refusal, deliberately not the proof-resource diagnostic,
                // which `docs/ir.md` defines as meaning an enumeration stopped.
                let enumerable = points.is_some() && self.boundary_extents(shape).is_some();
                let Some(points) = points.filter(|_| enumerable) else {
                    // An extent the environment bounds nowhere says why the
                    // proof was unavailable rather than only that it failed,
                    // and it is the one case a frontend can act on. An extent
                    // that is bounded but undetermined falls through to the
                    // generic refusal, because there the region is as stated
                    // and it is the proof that did not reach it.
                    diagnostics.push(self.unbounded_extent_symbol(access, shape).map_or_else(
                        || self.unproved(*access_index, access.mode),
                        |symbol| IndexRegionDiagnostic::ExtentBoundNotStated {
                            access: TensorAccessId {
                                owner: self.owner,
                                index: *access_index,
                            },
                            symbol,
                        },
                    ));
                    continue;
                };
                let (plan_len, bytes_per_point) = self.proof_plan_size(&access.coordinates);
                cells = cells.saturating_add(u128::from(points).saturating_mul(plan_len as u128));
                let coordinate_bytes = u128::from(points).saturating_mul(bytes_per_point.max(1));
                let dense_bytes = if access.mode == AccessMode::Write {
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
        let admitted = with_admitted_proof_budget(
            cells,
            integer_bytes,
            MAX_EXHAUSTIVE_PROOF_CELLS,
            MAX_EXHAUSTIVE_PROOF_BYTES,
            || {
                for access_index in accesses {
                    let access = &self.accesses[*access_index as usize];
                    let shape = &self.tensors[access.tensor as usize].shape;
                    // An undetermined domain or boundary was already refused
                    // above; neither has an extent vector to walk, so both are
                    // skipped rather than enumerated with a substituted size.
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
                    self.verify_access_exhaustively(
                        *access_index,
                        &plan,
                        &extents,
                        &axes,
                        diagnostics,
                    );
                }
            },
        );
        if let Err(excess) = admitted {
            diagnostics.push(excess.diagnostic());
        }
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

    pub(super) fn verify_access_exhaustively(
        &self,
        access_index: u32,
        expression_plan: &[u32],
        extents: &[u64],
        axes: &[u64],
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) {
        let access = &self.accesses[access_index as usize];
        let shape = &self.tensors[access.tensor as usize].shape;
        let Some(elements) = self.boundary_element_count(shape) else {
            diagnostics.push(self.unproved(access_index, access.mode));
            return;
        };
        let mut seen =
            (access.mode == AccessMode::Write).then(|| vec![0_u64; elements.div_ceil(64)]);
        let mut point = vec![0_u64; extents.len()];
        loop {
            let assignments: BTreeMap<_, _> = access
                .domain
                .iter()
                .copied()
                .zip(point.iter().copied())
                .collect();
            let evaluated = self.evaluate_expressions(expression_plan, &assignments);
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
                return;
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
                    return;
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
        }
    }

    pub(super) fn evaluate_expressions(
        &self,
        plan: &[u32],
        dimensions: &BTreeMap<u32, u64>,
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
                IndexNode::FloorDiv { dividend, divisor } => {
                    values[dividend].div_floor(&BigInt::from(*divisor))
                }
                IndexNode::Modulo { dividend, divisor } => {
                    values[dividend].mod_floor(&BigInt::from(*divisor))
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
}
