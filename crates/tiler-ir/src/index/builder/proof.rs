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
        // One pass over each root's stored value, because the two ways a value
        // can name a dimension its root cannot supply are the same defect seen
        // from two roles. The role decides which is reported, and it decides
        // exclusively: a reduction dimension is never in a write's domain, so
        // testing membership first would rename every unreduced value.
        for output in &self.outputs {
            let domain = self.accesses[output.access as usize].domain();
            for dimension in &self.values[output.value as usize].free_dimensions {
                let value = ScalarValueId {
                    owner: self.owner,
                    index: output.value,
                };
                let named = DimensionId {
                    owner: self.owner,
                    index: *dimension,
                };
                if self.dimensions[*dimension as usize].role == DomainRole::Reduction {
                    diagnostics.push(IndexRegionDiagnostic::FreeReductionDimension {
                        value,
                        dimension: named,
                    });
                } else if !domain.contains(dimension) {
                    // Only reachable since a write may iterate a subset of the
                    // parallel dimensions. The value has no single value at a
                    // point this root visits, so there is nothing to store.
                    diagnostics.push(IndexRegionDiagnostic::ValueDimensionOutsideWriteDomain {
                        access: TensorAccessId {
                            owner: self.owner,
                            index: output.access,
                        },
                        value,
                        dimension: named,
                    });
                }
            }
        }
        // A parallel dimension is used by being iterated or by being one a
        // reachable value varies along; a reduction dimension is used only by
        // being bound by a reachable reduction, which is a stronger demand and
        // stays exactly as it was — appearing in a read's domain is not what
        // makes a reduction dimension reduced.
        //
        // The parallel half exists because the subset write domain is what
        // first lets a parallel dimension go unmentioned. Compaction retains
        // every declared dimension, so an unmentioned one would sit in the
        // canonical identity of a region whose meaning does not include it, and
        // two regions that mean the same thing would not share an identity.
        let used_parallel: BTreeSet<u32> = reachable_accesses
            .iter()
            .flat_map(|access| self.accesses[*access as usize].domain().iter().copied())
            .chain(
                reachable_values
                    .iter()
                    .flat_map(|value| self.values[*value as usize].free_dimensions.iter().copied()),
            )
            .collect();
        for (i, dimension) in self.dimensions.iter().enumerate() {
            let index = bounded_index(i);
            let used = match dimension.role {
                DomainRole::Parallel => used_parallel.contains(&index),
                DomainRole::Reduction => used_reductions.contains(&index),
            };
            if !used {
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
                .entry(
                    self.accesses[output.access as usize]
                        .direct()
                        .expect("an output root is a direct write")
                        .tensor,
                )
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
                .any(|a| self.accesses[*a as usize].touches_tensor(index))
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
            // Only a direct access reaches the machinery below. A gather read's
            // bounds obligation is total and is discharged at compaction by
            // `derive_gather_index_bounds`; the interval, permutation, and
            // enumeration arguments here reason about one tensor and one
            // authored coordinate run, neither of which describes a gather.
            // Its own obligations are revalidated whole instead.
            let Some(access) = self.accesses[*access_index as usize].direct() else {
                self.verify_gather_access(*access_index, diagnostics);
                continue;
            };
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
                    partition_proofs.insert(
                        *tensor,
                        JointPartitionProof::Interval {
                            facts: self.partition_fact_source(*tensor, roots),
                        },
                    );
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
                        let access = self.accesses[*root as usize]
                            .direct()
                            .expect("a partition root is a direct write");
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
                        partition_proofs.insert(
                            *tensor,
                            JointPartitionProof::Exhaustive {
                                points,
                                facts: self.partition_fact_source(*tensor, &partitioned[tensor]),
                            },
                        );
                    }
                }
                for access_index in &exhaustive_accesses {
                    let access = self.accesses[*access_index as usize]
                        .direct()
                        .expect("only a direct access is scheduled for enumeration");
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
                let access = self.accesses[*access_index as usize]
                    .direct()
                    .expect("only a direct access is scheduled for enumeration");
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

    /// Revalidates one gather access against every obligation `gather_read` enforces.
    ///
    /// This is the *later* owner, not a second copy of the caller's diagnostics:
    /// the builder's structured errors win for caller input, and this exists so
    /// that corruption and any future internal construction cannot install a
    /// gather the authoring path would have refused. Reporting under
    /// [`IndexRegionDiagnostic::GatherAccess`] rather than
    /// `CoordinateOutOfBounds` is deliberate — invocation-required data is not
    /// an observed bad coordinate, and collapsing the two would let a reader
    /// conclude that some index value was seen out of range.
    ///
    /// **Obligations, not a rule-for-rule mirror.** `gather_read` refuses an
    /// aliased pair under its own [`IndexBuildError::GatherAliasedTensors`], and
    /// [`GatherAccessRule`] has no alias member — yet the obligation is still
    /// total here, discharged by the two type rules instead of by a rule of its
    /// own. An aliased access names **one** tensor, and one tensor carries
    /// **one** value type: either that type is not `tiler::f32@1`, and
    /// [`GatherAccessRule::SourceType`] fires, or it is — and then it is not
    /// `tiler::u32@1` either, so [`GatherAccessRule::IndexType`] fires. There is
    /// no third case. That is also why no alias member is added: it could never
    /// be raised, and a published rule nothing can raise states an obligation
    /// the vocabulary cannot honour. The test module below drives *both*
    /// aliasings and asserts the premise the argument rests on: that the two
    /// value types differ.
    ///
    /// The literal-shape rules are checked before `DomainShape`, mirroring the
    /// builder boundary exactly: a sourced boundary must never be reported as a
    /// shape disagreement derived from an environment this surface refuses to
    /// consult.
    ///
    /// **The domain obligation is one predicate, not two readings of one
    /// field.** This arrives at the committed `domain` — the ascending-ordinal
    /// collection of a `BTreeSet` — while `gather_read` sees the caller's
    /// slice, so the two ends disagree about order for the same region whenever
    /// a caller declared its result dimensions in any order but ascending.
    /// [`gather_domain_carries_result_extents`] is therefore called from both
    /// sides rather than spelled twice: it compares extents as a multiset, the
    /// strongest statement a set at rest can support. Writing the comparison
    /// here instead would restore a rule that answers differently at each end,
    /// which is what made the "every obligation" claim above false.
    fn verify_gather_access(
        &self,
        access_index: u32,
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) {
        let gather = self.accesses[access_index as usize]
            .gather_read()
            .expect("the caller selected a gather access");
        let access = TensorAccessId {
            owner: self.owner,
            index: access_index,
        };
        let mut refuse = |rule| {
            diagnostics.push(IndexRegionDiagnostic::GatherAccess { access, rule });
        };
        let source = &self.tensors[gather.source as usize];
        let index = &self.tensors[gather.index as usize];
        if source.role != TensorRole::Input {
            refuse(GatherAccessRule::SourceRole);
            return;
        }
        if index.role != TensorRole::Input {
            refuse(GatherAccessRule::IndexRole);
            return;
        }
        if source.value_type != F32::resolved_type().clone() {
            refuse(GatherAccessRule::SourceType);
            return;
        }
        if index.value_type != gather_index_resolved_type() {
            refuse(GatherAccessRule::IndexType);
            return;
        }
        let Some(source_shape) = source.shape.as_static() else {
            refuse(GatherAccessRule::SourceShapeLiteral);
            return;
        };
        let Some(index_shape) = index.shape.as_static() else {
            refuse(GatherAccessRule::IndexShapeLiteral);
            return;
        };
        if source_shape.rank() == 0 {
            refuse(GatherAccessRule::SourceRank);
            return;
        }
        if gather.axis as usize >= source_shape.rank() {
            refuse(GatherAccessRule::Axis);
            return;
        }
        if gather.source_coordinates.len() != source_shape.rank().saturating_sub(1) {
            refuse(GatherAccessRule::SourceCoordinateRank);
            return;
        }
        if gather.index_coordinates.len() != index_shape.rank() {
            refuse(GatherAccessRule::IndexCoordinateRank);
            return;
        }
        let mut declared = Vec::with_capacity(gather.domain.len());
        for dimension in &gather.domain {
            let Some(extent) = self.dimensions[*dimension as usize].extent.as_static() else {
                refuse(GatherAccessRule::DomainExtentLiteral);
                return;
            };
            declared.push(extent);
        }
        let (Ok(declared_shape), Ok((_, derived))) = (
            Shape::try_new(declared),
            gather_result_shape(Axis::new(gather.axis), source_shape, index_shape),
        ) else {
            refuse(GatherAccessRule::DomainShape);
            return;
        };
        // The same predicate the authoring path applies, rather than a second
        // spelling of it: this reads the committed sorted run while
        // `gather_read` reads the caller's slice, so two spellings of one
        // comparison are two chances to disagree about one region.
        if !gather_domain_carries_result_extents(&declared_shape, &derived) {
            refuse(GatherAccessRule::DomainShape);
            return;
        }
        let domain: BTreeSet<u32> = gather.domain.iter().copied().collect();
        if gather.source_coordinates.iter().any(|coordinate| {
            !self.expressions[*coordinate as usize]
                .dimensions
                .is_subset(&domain)
        }) {
            refuse(GatherAccessRule::SourceCoordinateScope);
            return;
        }
        if gather.index_coordinates.iter().any(|coordinate| {
            !self.expressions[*coordinate as usize]
                .dimensions
                .is_subset(&domain)
        }) {
            refuse(GatherAccessRule::IndexCoordinateScope);
        }
    }

    fn cheap_index_domain_predicates(
        &self,
        access_index: u32,
        access: &DirectAccessData,
        shape: &SourcedShape,
        points: Option<u64>,
    ) -> Vec<PendingIndexDomainPredicate> {
        let mut predicates = Vec::with_capacity(access.coordinates.len().saturating_mul(2));
        // One answer for the whole access, so its two atoms per axis and the
        // enumeration that may later rewrite them all describe the same
        // permitted premises.
        let facts = self.access_fact_source(access, shape);
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
                    facts,
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
        access: &DirectAccessData,
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
        access: &DirectAccessData,
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
        access: &DirectAccessData,
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
        access: &DirectAccessData,
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
                // A symbolic coefficient is bounded by the largest magnitude
                // its declared extent admits, which is what `interval_linear`
                // multiplied by and is therefore the same bound the propagated
                // interval rests on. `None` is unreachable for the same reason
                // the `expect` below is: this expression has an interval, and
                // `interval_linear` states one only when every child had one
                // and every symbolic coefficient was bounded. It still fails
                // closed at the widest budget rather than asserting.
                let coefficient_bound = match &term.coefficient {
                    SourcedIndexInteger::Literal(value) => value.0.abs(),
                    SourcedIndexInteger::Symbol(symbol) => {
                        let Some(bound) = self
                            .extent_interval(&SourcedExtent::Symbol(symbol.clone()))
                            .map(|interval| BigInt::from(interval.upper))
                        else {
                            return u128::MAX;
                        };
                        bound
                    }
                };
                let (minimum, maximum) = self.expressions[term.value as usize]
                    .interval
                    .as_ref()
                    .expect("a linear interval requires every child interval");
                let child_bound = minimum.abs().max(maximum.abs());
                let Ok(product) = checked_index_product(&coefficient_bound, &child_bound) else {
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
        let access = self.accesses[access_index as usize]
            .direct()
            .expect("only a direct access is scheduled for enumeration");
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
        // already excluded an undetermined divisor or coefficient, so reaching
        // this returns "not proved" with no diagnostic — a write's ownership
        // requirement is refused separately by that same gate, and inventing an
        // out-of-bounds refutation here would be a claim nothing established.
        let Some(scalars) = self.plan_scalars(expression_plan) else {
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
            let evaluated = self.evaluate_expressions(expression_plan, &assignments, &scalars);
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
    /// It fails on a symbolic divisor or a symbolic coefficient the environment
    /// does not pin to one value: with no divisor there is no quotient and with
    /// no coefficient there is no product, and a walk that produced no value for
    /// a coordinate would be indistinguishable, to
    /// [`Self::verify_access_exhaustively`], from a coordinate that landed out
    /// of bounds. Deciding it before the walk is what keeps a missing proof from
    /// being reported as a refutation.
    ///
    /// **A pinned symbol is read here, in both positions.** An earlier draft
    /// read the divisor and declined the coefficient, on the ground that
    /// resolving a coefficient would make an enumeration's result a function of
    /// the binding while a divisor's value "was already required to make the
    /// expression defined at all". That carve-out does not hold: what admission
    /// required of a divisor is [`ExtentSources::proves_positive`], and an
    /// environment stating `d` in `[1, 64]` satisfies it while determining
    /// nothing, so [`ExtentSources::determined`] reads strictly more than
    /// definedness ever demanded. The two halves are on one rule instead, the
    /// one [`interval_linear`] states: a proof may read this region's shape
    /// environment, because the region's identity names that environment; a
    /// rewrite may not, which is why normalization still refuses to fold `S * x`
    /// at `S == 1`.
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
                IndexNode::LinearCombination { terms, .. } => terms.iter().all(|term| {
                    term.coefficient.as_literal().is_some()
                        || self.determined_scalar(&term.coefficient).is_some()
                }),
                IndexNode::Constant(_) | IndexNode::Dimension(_) => true,
            },
        )
    }

    /// Resolves every environment-sourced scalar an enumeration plan will need.
    ///
    /// `None` when one of them is undetermined, which
    /// [`Self::coordinates_are_evaluable`] already excluded before any budget
    /// was taken. Resolving them once, up front, is what lets the point loop
    /// below be total arithmetic rather than a per-point lookup that could fail
    /// halfway through a walk.
    ///
    /// Divisors are keyed by their owning expression because each quotient has
    /// its own; coefficients are keyed by *symbol* because one symbol scaling
    /// two terms is one value, and resolving it per term would ask the
    /// environment the same question twice and leave two answers that a future
    /// change could let diverge.
    fn plan_scalars(&self, plan: &[u32]) -> Option<PlanScalars> {
        let mut scalars = PlanScalars {
            divisors: BTreeMap::new(),
            coefficients: BTreeMap::new(),
        };
        for index in plan {
            match &*self.expressions[*index as usize].node {
                IndexNode::FloorDiv { divisor, .. } | IndexNode::Modulo { divisor, .. } => {
                    scalars.divisors.insert(*index, self.determined(divisor)?);
                }
                IndexNode::LinearCombination { terms, .. } => {
                    for term in terms {
                        if let Some(symbol) = term.coefficient.symbol() {
                            scalars
                                .coefficients
                                .insert(symbol.clone(), self.determined_scalar(&term.coefficient)?);
                        }
                    }
                }
                IndexNode::Constant(_) | IndexNode::Dimension(_) => {}
            }
        }
        Some(scalars)
    }

    pub(super) fn evaluate_expressions(
        &self,
        plan: &[u32],
        dimensions: &BTreeMap<u32, u64>,
        scalars: &PlanScalars,
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
                        // Total arithmetic: `plan_scalars` resolved every
                        // symbolic coefficient in this plan before any budget
                        // was taken, so both arms are present by construction.
                        let coefficient = match &term.coefficient {
                            SourcedIndexInteger::Literal(value) => value.0.clone(),
                            SourcedIndexInteger::Symbol(symbol) => {
                                BigInt::from(scalars.coefficients[symbol])
                            }
                        };
                        sum + coefficient * &values[&term.value]
                    })
                }
                IndexNode::FloorDiv { dividend, .. } => {
                    values[dividend].div_floor(&BigInt::from(scalars.divisors[index]))
                }
                IndexNode::Modulo { dividend, .. } => {
                    values[dividend].mod_floor(&BigInt::from(scalars.divisors[index]))
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
    pub(super) fn write_is_permutation(
        &self,
        access: &DirectAccessData,
        shape: &SourcedShape,
    ) -> bool {
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
    /// coefficient and a non-negative displacement. The unit may be written
    /// literally or be a symbol this region's environment determines as one;
    /// that is exactly the form
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
                // The rectangle vocabulary requires an exact unit. A literal
                // states that directly; a symbol is admitted only when this
                // region's environment determines it to be exactly one. The
                // expression still names the symbol, so proof and canonical
                // normalization remain separate decisions.
                let is_unit = match term.coefficient.as_literal() {
                    Some(value) => value.0 == BigInt::from(1_u8),
                    None => self.determined_scalar(&term.coefficient) == Some(1),
                };
                if !is_unit {
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
    ///
    /// **Both halves of that argument are root-local, and re-deriving them is
    /// what admits roots over unequal sub-domains.** Every quantifier above
    /// ranges over `access.domain` — the dimensions *this* root iterates — and
    /// none over the region's parallel set. Injectivity needs the two distinct
    /// points to differ somewhere in `consumed`, which the equal-cardinality
    /// check makes exactly `access.domain`; and the volume is the product of
    /// each axis's span, which is `extent(d)` for a consumed `d` and `1` for a
    /// constant axis, hence the product of the extents of `access.domain` —
    /// which is that domain's point count. The premise the write contract used
    /// to carry, that every root iterates every parallel dimension, appears in
    /// neither derivation. What it bought was the *global* corollary that all
    /// roots have one point count and so own equal shares; dropping it drops
    /// only that corollary.
    ///
    /// A zero-extent dimension is the degenerate case rather than an excluded
    /// one: its axis's span is zero, the rectangle is empty, its volume is zero,
    /// and the root writes no element — which is what the injective map over an
    /// empty domain says. The emptiness has to be carried into the disjointness
    /// test rather than left to the ranges, and
    /// [`Self::decide_partition_by_interval`] states why.
    fn write_partition_box(
        &self,
        access: &DirectAccessData,
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
    /// **An empty rectangle is separated from everything, and it is checked
    /// before the axes rather than left to them.** `[a, b)` and `[c, d)` meet
    /// exactly when `max(a, c) < min(b, d)`, which needs both ranges nonempty;
    /// the per-axis test below decides `b > c && d > a`, which is that
    /// condition only once `a < b` and `c < d` hold. A root over a zero-extent
    /// dimension has a range with `a == b`, and such a root writes nothing, so
    /// reading the axes alone would refuse a legal partition whenever its empty
    /// member sits strictly inside a sibling's range rather than at its edge.
    /// The volume identity is untouched by the case: an empty rectangle
    /// contributes zero, which is exactly the element count of a root that
    /// visits no point.
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
            let Some(placed) = self.write_partition_box(
                self.accesses[*root as usize]
                    .direct()
                    .expect("a partition root is a direct write"),
                shape,
            ) else {
                return PartitionVerdict::Enumerate;
            };
            boxes.push(placed);
        }
        let empty = |placed: &[(u64, u64)]| placed.iter().any(|(start, end)| start == end);
        for (position, left) in boxes.iter().enumerate() {
            for right in &boxes[position.saturating_add(1)..] {
                let separated = empty(left)
                    || empty(right)
                    || left.iter().zip(right).any(
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
            let access = self.accesses[*root as usize]
                .direct()
                .expect("a partition root is a direct write");
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
            let access = self.accesses[*root as usize]
                .direct()
                .expect("a partition root is a direct write");
            let mut reached = BTreeSet::new();
            for coordinate in &access.coordinates {
                self.mark_expr(*coordinate, &mut reached);
            }
            let plan = reached.into_iter().collect::<Vec<_>>();
            let (Some(extents), Some(scalars)) = (
                self.domain_extents(&access.domain),
                self.plan_scalars(&plan),
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
                let evaluated = self.evaluate_expressions(&plan, &assignments, &scalars);
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

/// Every environment-sourced scalar one enumeration plan needs, resolved once.
///
/// Held together rather than as two returns so that a caller cannot resolve the
/// divisors of a plan and forget its coefficients: the walk needs both, and
/// both are refused before any budget is charged.
pub(super) struct PlanScalars {
    /// One resolved divisor per quotient or remainder expression in the plan.
    divisors: BTreeMap<u32, u64>,
    /// One resolved value per declared symbol a plan coefficient names.
    coefficients: BTreeMap<ShapeSymbol, u64>,
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

/// Whole-region gather revalidation, driven from corrupted draft state.
///
/// These are the only tests that reach [`IndexRegionBuilder::verify_gather_access`]
/// with a gather it can refuse. Every gather that arrives through `gather_read`
/// was already checked by `prepare_gather_access` against the same obligations,
/// so no admitted region can make an arm fire — which is the design, and also
/// why the arms need a test that corrupts the draft directly. Without one,
/// "nothing ran" and "every rule holds" are the same green.
///
/// That claim rests on the two sides sharing a predicate wherever they read a
/// field from opposite ends, and it is **not** self-evidently true: the domain
/// obligation is one such field, and an order-sensitive comparison over it
/// admits a region here that `verify` then refuses. It is also not checked by
/// anything in this module, because every gather below
/// is built by `admitted_gather`, which declares its result dimensions in
/// ascending order and so cannot separate a caller's order from an ordinal one.
/// `every_declaration_order_of_one_gather_is_admitted_by_both_validators`, in
/// `tests/index_gather.rs`, is what holds the claim; a reader repairing an arm
/// here should treat this paragraph as a pointer to that test rather than as
/// evidence on its own.
///
/// Each case perturbs exactly one field of the committed [`AccessData`], so a
/// reddening perturbation names which rule is load-bearing.
#[cfg(test)]
mod tests {
    use super::*;

    /// An admitted `[4, 4]` gather on axis 0 by a `[2]` index, and its ordinal.
    ///
    /// Authored through the public builder so the starting point is a gather the
    /// authoring path *accepted*; the corruption is then the only difference
    /// between a diagnostic and none.
    fn admitted_gather() -> (IndexRegionBuilder, u32) {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let mut builder = IndexRegionBuilder::new(registry).expect("a builder identity remains");
        let rows = builder
            .dimension(DomainRole::Parallel, Extent::new(2))
            .expect("a parallel dimension is admitted");
        let columns = builder
            .dimension(DomainRole::Parallel, Extent::new(4))
            .expect("a parallel dimension is admitted");
        let source = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type().clone(),
                Shape::from_dims([4, 4]),
            )
            .expect("a literal f32 input is admitted");
        let index = builder
            .tensor(
                TensorRole::Input,
                gather_index_resolved_type(),
                Shape::from_dims([2]),
            )
            .expect("a literal u32 input is admitted");
        let row = builder
            .dimension_expr(rows)
            .expect("a dimension expression");
        let column = builder
            .dimension_expr(columns)
            .expect("a dimension expression");
        builder
            .gather_read(
                source,
                index,
                &[rows, columns],
                &[column],
                &[row],
                Axis::new(0),
            )
            .expect("the fixture gather is admitted");
        let ordinal = builder
            .accesses
            .iter()
            .position(|access| access.gather_read().is_some())
            .expect("the fixture authors exactly one gather");
        (builder, bounded_index(ordinal))
    }

    /// Corrupts the committed gather, revalidates it, and returns the diagnostics.
    fn revalidate(
        corrupt: impl FnOnce(&mut IndexRegionBuilder, u32),
    ) -> Vec<IndexRegionDiagnostic> {
        let (mut builder, access) = admitted_gather();
        corrupt(&mut builder, access);
        let mut diagnostics = Vec::new();
        builder.verify_gather_access(access, &mut diagnostics);
        diagnostics
    }

    /// The one rule the revalidation reported, or `None` for any other shape.
    fn refused(diagnostics: &[IndexRegionDiagnostic]) -> Option<GatherAccessRule> {
        match diagnostics {
            [IndexRegionDiagnostic::GatherAccess { rule, .. }] => Some(*rule),
            _ => None,
        }
    }

    fn gather_mut(builder: &mut IndexRegionBuilder, access: u32) -> &mut GatherReadAccessData {
        match &mut builder.accesses[access as usize] {
            AccessData::GatherRead(gather) => gather,
            AccessData::Direct(_) => panic!("the fixture committed a gather"),
        }
    }

    /// The negative control: an uncorrupted gather is revalidated silently.
    ///
    /// Without this, every case below would also pass against a
    /// `verify_gather_access` that refused unconditionally, and the suite would
    /// be evidence that the arms *fire* rather than that they *discriminate*.
    #[test]
    fn an_admitted_gather_revalidates_without_a_diagnostic() {
        assert_eq!(revalidate(|_, _| {}), Vec::new());
    }

    /// An aliased pair is refused under a **type** rule, from either side.
    ///
    /// The whole of why `GatherAccessRule` needs no alias member. One tensor
    /// carries one value type, so aliasing onto the f32 source leaves the index
    /// role holding f32 and aliasing onto the u32 index leaves the source role
    /// holding u32 — the two type rules between them cover every aliasing there
    /// is. The premise is asserted rather than assumed: were the two value types
    /// ever made equal, both arms would stop firing and this would say so here
    /// instead of admitting a corrupted region.
    #[test]
    fn an_alias_onto_either_operand_is_refused_by_a_type_rule() {
        assert_ne!(
            F32::resolved_type().clone(),
            gather_index_resolved_type(),
            "the alias argument rests on the two operand types being distinct",
        );
        assert_eq!(
            refused(&revalidate(|builder, access| {
                let gather = gather_mut(builder, access);
                gather.index = gather.source;
            })),
            Some(GatherAccessRule::IndexType),
            "a pair aliased onto the f32 source has an index operand that is not u32",
        );
        assert_eq!(
            refused(&revalidate(|builder, access| {
                let gather = gather_mut(builder, access);
                gather.source = gather.index;
            })),
            Some(GatherAccessRule::SourceType),
            "a pair aliased onto the u32 index has a source operand that is not f32",
        );
    }

    /// A boundary demoted out of the input role is refused in its own role.
    ///
    /// Driven separately per operand, because one shared role check would report
    /// whichever ordinal it happened to read first and could not tell a caller
    /// which of the two boundaries moved.
    #[test]
    fn a_gather_operand_outside_the_input_role_is_refused_by_role() {
        assert_eq!(
            refused(&revalidate(|builder, access| {
                let source = gather_mut(builder, access).source;
                builder.tensors[source as usize].role = TensorRole::Output;
            })),
            Some(GatherAccessRule::SourceRole),
        );
        assert_eq!(
            refused(&revalidate(|builder, access| {
                let index = gather_mut(builder, access).index;
                builder.tensors[index as usize].role = TensorRole::Output;
            })),
            Some(GatherAccessRule::IndexRole),
        );
    }

    /// An axis outside the source rank is refused, and rank zero before it.
    ///
    /// The two are checked in this order by the builder boundary, so a rank-zero
    /// source must be reported as `SourceRank` and never as an axis that happens
    /// to exceed a rank of nothing.
    #[test]
    fn a_corrupted_axis_and_a_rank_zero_source_are_refused_in_that_precedence() {
        assert_eq!(
            refused(&revalidate(|builder, access| {
                gather_mut(builder, access).axis = 7;
            })),
            Some(GatherAccessRule::Axis),
        );
        assert_eq!(
            refused(&revalidate(|builder, access| {
                let source = gather_mut(builder, access).source;
                let scalar = Shape::try_new([]).expect("a rank-zero shape is admitted");
                builder.tensors[source as usize].shape = scalar.into();
                // Left at 7 so the rank rule must win on precedence rather than
                // because no other rule could have fired.
                gather_mut(builder, access).axis = 7;
            })),
            Some(GatherAccessRule::SourceRank),
            "a rank-zero source is named as such, not as an out-of-range axis",
        );
    }

    /// A coordinate run of the wrong arity is refused in its own run's name.
    #[test]
    fn a_coordinate_run_of_the_wrong_arity_is_refused_by_run() {
        assert_eq!(
            refused(&revalidate(|builder, access| {
                gather_mut(builder, access).source_coordinates.clear();
            })),
            Some(GatherAccessRule::SourceCoordinateRank),
        );
        assert_eq!(
            refused(&revalidate(|builder, access| {
                gather_mut(builder, access).index_coordinates.clear();
            })),
            Some(GatherAccessRule::IndexCoordinateRank),
        );
    }

    /// A declared domain that no longer derives from the operands is refused.
    #[test]
    fn a_domain_disagreeing_with_the_derived_result_shape_is_refused() {
        assert_eq!(
            refused(&revalidate(|builder, access| {
                gather_mut(builder, access).domain.pop();
            })),
            Some(GatherAccessRule::DomainShape),
        );
    }

    /// A coordinate reaching outside the access domain is refused per run.
    ///
    /// The substituted coordinate keeps its run's arity and leaves the declared
    /// domain untouched, so every earlier rule still passes and only the scope
    /// rule can be what fired. Both runs are driven, because a scope check
    /// written over one run would leave the other unguarded while still turning
    /// this test green for the run it did cover.
    #[test]
    fn a_coordinate_leaving_the_access_domain_is_refused_by_run() {
        /// Replaces one coordinate run with an expression over a fresh
        /// dimension the gather's domain does not contain.
        fn outside(builder: &mut IndexRegionBuilder) -> u32 {
            let spare = builder
                .dimension(DomainRole::Parallel, Extent::new(2))
                .expect("a parallel dimension is admitted");
            builder
                .dimension_expr(spare)
                .expect("a dimension expression")
                .index
        }
        assert_eq!(
            refused(&revalidate(|builder, access| {
                let coordinate = outside(builder);
                gather_mut(builder, access).source_coordinates = vec![coordinate];
            })),
            Some(GatherAccessRule::SourceCoordinateScope),
        );
        assert_eq!(
            refused(&revalidate(|builder, access| {
                let coordinate = outside(builder);
                gather_mut(builder, access).index_coordinates = vec![coordinate];
            })),
            Some(GatherAccessRule::IndexCoordinateScope),
        );
    }
}
