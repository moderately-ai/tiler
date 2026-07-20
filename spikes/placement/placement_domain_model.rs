//! Bounded model of symbolic placement and memory-domain enforcement.
//!
//! This spike validates representation boundaries and graph search; it does not
//! call a device API or predict transfer performance.
//!
//! Run with:
//! `rustc --edition 2021 --test spikes/placement/placement_domain_model.rs -o /tmp/tiler-placement-tests && /tmp/tiler-placement-tests`

#![allow(dead_code)]

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AffinityId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MemoryDomainId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EncodingId(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeviceClass {
    Cpu,
    Cuda,
    Metal,
}

/// A compile/plan-time requirement. It is not a runtime device ordinal.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SymbolicAffinity {
    id: AffinityId,
    class: DeviceClass,
    selector: &'static str,
}

/// Runtime-scoped identity: equal ordinals in different sessions are unequal.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LiveDeviceKey {
    provider: &'static str,
    runtime_session: u64,
    stable_device_token: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AffinityBinding {
    symbolic: AffinityId,
    live_device: LiveDeviceKey,
}

/// Semantic target properties intentionally use a disjoint type and namespace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SemanticTargetPropertyKey(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

impl AccessMode {
    fn admits(self, required: Self) -> bool {
        self == AccessMode::ReadWrite || self == required
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Coherence {
    ImplicitAfterDependency,
    ExplicitTransitionRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AccessGrant {
    affinity: AffinityId,
    mode: AccessMode,
    coherence: Coherence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MemoryDomain {
    id: MemoryDomainId,
    name: &'static str,
    grants: Vec<AccessGrant>,
    allocators: Vec<AffinityId>,
    capacity_bytes: Option<u64>,
    minimum_alignment: u64,
}

impl MemoryDomain {
    fn can_access(&self, affinity: AffinityId, mode: AccessMode) -> bool {
        self.grants
            .iter()
            .any(|grant| grant.affinity == affinity && grant.mode.admits(mode))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MovementKind {
    Copy,
    PeerCopy,
    Migrate,
    Import,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MovementEdge {
    from: MemoryDomainId,
    to: MemoryDomainId,
    kind: MovementKind,
    preserves_encoding: bool,
    fixed_cost_units: u64,
    cost_units_per_byte: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DomainGraph {
    domains: BTreeMap<MemoryDomainId, MemoryDomain>,
    movements: Vec<MovementEdge>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DeliveredPlacement {
    domain: MemoryDomainId,
    encoding: EncodingId,
    available_after: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlacementRequirement {
    affinity: AffinityId,
    required_domain: Option<MemoryDomainId>,
    access: AccessMode,
    encoding: EncodingId,
    bytes: u64,
    alignment: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Enforcer {
    Direct,
    Movement(MovementEdge),
    Repack {
        in_domain: MemoryDomainId,
        from: EncodingId,
        to: EncodingId,
    },
    Recompute {
        at: AffinityId,
        encoding: EncodingId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EnforcementPath {
    cost_units: u64,
    steps: Vec<Enforcer>,
    delivered_domain: MemoryDomainId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PlacementError {
    UnknownDomain,
    CapacityExceeded,
    AlignmentUnsupported,
    NoLegalPath,
}

impl DomainGraph {
    fn verify_storage(
        &self,
        domain: &MemoryDomain,
        bytes: u64,
        alignment: u64,
    ) -> Result<(), PlacementError> {
        if let Some(capacity) = domain.capacity_bytes {
            if bytes > capacity {
                return Err(PlacementError::CapacityExceeded);
            }
        }
        if alignment < domain.minimum_alignment
            || !alignment.is_multiple_of(domain.minimum_alignment)
        {
            return Err(PlacementError::AlignmentUnsupported);
        }
        Ok(())
    }

    fn verify_destination(
        &self,
        domain: &MemoryDomain,
        requirement: PlacementRequirement,
    ) -> Result<(), PlacementError> {
        if requirement
            .required_domain
            .is_some_and(|id| id != domain.id)
        {
            return Err(PlacementError::NoLegalPath);
        }
        if !domain.can_access(requirement.affinity, requirement.access) {
            return Err(PlacementError::NoLegalPath);
        }
        self.verify_storage(domain, requirement.bytes, requirement.alignment)
    }

    fn enforce(
        &self,
        delivered: DeliveredPlacement,
        requirement: PlacementRequirement,
    ) -> Result<EnforcementPath, PlacementError> {
        if let Some(required_domain) = requirement.required_domain {
            let destination = self
                .domains
                .get(&required_domain)
                .ok_or(PlacementError::UnknownDomain)?;
            self.verify_destination(destination, requirement)?;
        }
        let source = self
            .domains
            .get(&delivered.domain)
            .ok_or(PlacementError::UnknownDomain)?;
        if delivered.encoding == requirement.encoding
            && self.verify_destination(source, requirement).is_ok()
        {
            return Ok(EnforcementPath {
                cost_units: 0,
                steps: vec![Enforcer::Direct],
                delivered_domain: delivered.domain,
            });
        }

        let mut best = BTreeMap::<MemoryDomainId, u64>::new();
        let mut prior = BTreeMap::<MemoryDomainId, MovementEdge>::new();
        let mut frontier = BinaryHeap::from([(Reverse(0_u64), delivered.domain)]);
        best.insert(delivered.domain, 0);

        while let Some((Reverse(cost), current)) = frontier.pop() {
            if best.get(&current).copied() != Some(cost) {
                continue;
            }
            for edge in self.movements.iter().filter(|edge| edge.from == current) {
                if !edge.preserves_encoding || delivered.encoding != requirement.encoding {
                    continue;
                }
                let Some(destination) = self.domains.get(&edge.to) else {
                    continue;
                };
                if self
                    .verify_storage(destination, requirement.bytes, requirement.alignment)
                    .is_err()
                {
                    continue;
                }
                if self.verify_destination(destination, requirement).is_err()
                    && !self.movements.iter().any(|next| next.from == edge.to)
                {
                    continue;
                }
                let candidate = cost
                    .saturating_add(edge.fixed_cost_units)
                    .saturating_add(edge.cost_units_per_byte.saturating_mul(requirement.bytes));
                if candidate < best.get(&edge.to).copied().unwrap_or(u64::MAX) {
                    best.insert(edge.to, candidate);
                    prior.insert(edge.to, *edge);
                    frontier.push((Reverse(candidate), edge.to));
                }
            }
        }

        let mut candidates: Vec<_> = self
            .domains
            .values()
            .filter(|domain| self.verify_destination(domain, requirement).is_ok())
            .filter_map(|domain| best.get(&domain.id).map(|cost| (*cost, domain.id)))
            .collect();
        candidates.sort_unstable();
        let Some((cost_units, destination)) = candidates.first().copied() else {
            return Err(PlacementError::NoLegalPath);
        };

        let mut reversed = Vec::new();
        let mut cursor = destination;
        while cursor != delivered.domain {
            let edge = *prior.get(&cursor).ok_or(PlacementError::NoLegalPath)?;
            reversed.push(Enforcer::Movement(edge));
            cursor = edge.from;
        }
        reversed.reverse();
        Ok(EnforcementPath {
            cost_units,
            steps: reversed,
            delivered_domain: destination,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InitialExecutionProfile {
    affinity: AffinityId,
    command_streams: u8,
    allows_movement_stages: bool,
}

impl InitialExecutionProfile {
    fn verify_program(
        self,
        stage_affinities: &[AffinityId],
        enforcers: &[Enforcer],
    ) -> Result<(), &'static str> {
        if self.command_streams != 1 {
            return Err("initial profile requires one ordered command stream");
        }
        if stage_affinities
            .iter()
            .any(|affinity| *affinity != self.affinity)
        {
            return Err("initial profile requires one symbolic device affinity");
        }
        if !self.allows_movement_stages
            && enforcers
                .iter()
                .any(|step| matches!(step, Enforcer::Movement(_)))
        {
            return Err("initial profile requires inputs already accessible");
        }
        Ok(())
    }
}

fn example_graph(peer_edge: bool) -> DomainGraph {
    let cpu = AffinityId(0);
    let gpu0 = AffinityId(1);
    let gpu1 = AffinityId(2);
    let host = MemoryDomainId(0);
    let gpu0_private = MemoryDomainId(1);
    let gpu1_private = MemoryDomainId(2);
    let staging = MemoryDomainId(3);

    let mut domains = BTreeMap::new();
    domains.insert(
        host,
        MemoryDomain {
            id: host,
            name: "host",
            grants: vec![AccessGrant {
                affinity: cpu,
                mode: AccessMode::ReadWrite,
                coherence: Coherence::ImplicitAfterDependency,
            }],
            allocators: vec![cpu],
            capacity_bytes: None,
            minimum_alignment: 8,
        },
    );
    domains.insert(
        gpu0_private,
        MemoryDomain {
            id: gpu0_private,
            name: "gpu0-private",
            grants: vec![AccessGrant {
                affinity: gpu0,
                mode: AccessMode::ReadWrite,
                coherence: Coherence::ImplicitAfterDependency,
            }],
            allocators: vec![gpu0],
            capacity_bytes: Some(1 << 30),
            minimum_alignment: 256,
        },
    );
    domains.insert(
        gpu1_private,
        MemoryDomain {
            id: gpu1_private,
            name: "gpu1-private",
            grants: vec![AccessGrant {
                affinity: gpu1,
                mode: AccessMode::ReadWrite,
                coherence: Coherence::ImplicitAfterDependency,
            }],
            allocators: vec![gpu1],
            capacity_bytes: Some(1 << 30),
            minimum_alignment: 256,
        },
    );
    domains.insert(
        staging,
        MemoryDomain {
            id: staging,
            name: "mapped-staging",
            grants: vec![
                AccessGrant {
                    affinity: cpu,
                    mode: AccessMode::ReadWrite,
                    coherence: Coherence::ExplicitTransitionRequired,
                },
                AccessGrant {
                    affinity: gpu0,
                    mode: AccessMode::ReadWrite,
                    coherence: Coherence::ExplicitTransitionRequired,
                },
                AccessGrant {
                    affinity: gpu1,
                    mode: AccessMode::ReadWrite,
                    coherence: Coherence::ExplicitTransitionRequired,
                },
            ],
            allocators: vec![cpu],
            capacity_bytes: Some(1 << 28),
            minimum_alignment: 64,
        },
    );

    let mut movements = vec![
        MovementEdge {
            from: gpu0_private,
            to: staging,
            kind: MovementKind::Copy,
            preserves_encoding: true,
            fixed_cost_units: 20,
            cost_units_per_byte: 2,
        },
        MovementEdge {
            from: staging,
            to: gpu1_private,
            kind: MovementKind::Copy,
            preserves_encoding: true,
            fixed_cost_units: 20,
            cost_units_per_byte: 2,
        },
    ];
    if peer_edge {
        movements.push(MovementEdge {
            from: gpu0_private,
            to: gpu1_private,
            kind: MovementKind::PeerCopy,
            preserves_encoding: true,
            fixed_cost_units: 10,
            cost_units_per_byte: 1,
        });
    }
    DomainGraph { domains, movements }
}

fn main() {
    let graph = example_graph(true);
    let path = graph
        .enforce(
            DeliveredPlacement {
                domain: MemoryDomainId(1),
                encoding: EncodingId(0),
                available_after: 4,
            },
            PlacementRequirement {
                affinity: AffinityId(2),
                required_domain: Some(MemoryDomainId(2)),
                access: AccessMode::Read,
                encoding: EncodingId(0),
                bytes: 1 << 20,
                alignment: 256,
            },
        )
        .unwrap();
    println!("selected={:?} cost_units={}", path.steps, path.cost_units);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirement(affinity: u16) -> PlacementRequirement {
        PlacementRequirement {
            affinity: AffinityId(affinity),
            required_domain: (affinity == 2).then_some(MemoryDomainId(2)),
            access: AccessMode::Read,
            encoding: EncodingId(0),
            bytes: 4096,
            alignment: 256,
        }
    }

    #[test]
    fn runtime_identity_is_not_an_ordinal() {
        let first = LiveDeviceKey {
            provider: "cuda",
            runtime_session: 1,
            stable_device_token: "ordinal-0",
        };
        let second = LiveDeviceKey {
            provider: "cuda",
            runtime_session: 2,
            stable_device_token: "ordinal-0",
        };
        assert_ne!(first, second);
    }

    #[test]
    fn direct_access_is_a_domain_affinity_relation() {
        let graph = example_graph(false);
        let delivered = DeliveredPlacement {
            domain: MemoryDomainId(1),
            encoding: EncodingId(0),
            available_after: 1,
        };
        assert_eq!(
            graph.enforce(delivered, requirement(1)).unwrap().steps,
            vec![Enforcer::Direct]
        );
        assert_ne!(
            graph.enforce(delivered, requirement(2)).unwrap().steps,
            vec![Enforcer::Direct]
        );
    }

    #[test]
    fn topology_selects_peer_or_staged_transfer() {
        let delivered = DeliveredPlacement {
            domain: MemoryDomainId(1),
            encoding: EncodingId(0),
            available_after: 1,
        };
        let direct = example_graph(true)
            .enforce(delivered, requirement(2))
            .unwrap();
        let staged = example_graph(false)
            .enforce(delivered, requirement(2))
            .unwrap();
        assert_eq!(direct.steps.len(), 1);
        assert_eq!(staged.steps.len(), 2);
        assert!(direct.cost_units < staged.cost_units);
    }

    #[test]
    fn intermediate_domains_must_satisfy_storage_constraints() {
        let mut graph = example_graph(false);
        graph
            .domains
            .get_mut(&MemoryDomainId(3))
            .unwrap()
            .capacity_bytes = Some(1024);
        let delivered = DeliveredPlacement {
            domain: MemoryDomainId(1),
            encoding: EncodingId(0),
            available_after: 1,
        };
        assert_eq!(
            graph.enforce(delivered, requirement(2)),
            Err(PlacementError::NoLegalPath)
        );
    }

    #[test]
    fn capacity_and_alignment_are_hard_not_costs() {
        let graph = example_graph(true);
        let delivered = DeliveredPlacement {
            domain: MemoryDomainId(1),
            encoding: EncodingId(0),
            available_after: 1,
        };
        let mut oversized = requirement(2);
        oversized.bytes = (1 << 30) + 1;
        assert_eq!(
            graph.enforce(delivered, oversized),
            Err(PlacementError::CapacityExceeded)
        );
        let mut misaligned = requirement(2);
        misaligned.alignment = 128;
        assert_eq!(
            graph.enforce(delivered, misaligned),
            Err(PlacementError::AlignmentUnsupported)
        );
    }

    #[test]
    fn encoding_change_is_not_silently_a_transfer() {
        let graph = example_graph(true);
        let delivered = DeliveredPlacement {
            domain: MemoryDomainId(1),
            encoding: EncodingId(7),
            available_after: 1,
        };
        assert_eq!(
            graph.enforce(delivered, requirement(2)),
            Err(PlacementError::NoLegalPath)
        );
    }

    #[test]
    fn initial_program_profile_stays_single_device_and_single_queue() {
        let profile = InitialExecutionProfile {
            affinity: AffinityId(1),
            command_streams: 1,
            allows_movement_stages: false,
        };
        assert_eq!(
            profile.verify_program(&[AffinityId(1), AffinityId(1)], &[]),
            Ok(())
        );
        assert!(profile
            .verify_program(&[AffinityId(1), AffinityId(2)], &[])
            .is_err());
        let transfer = Enforcer::Movement(example_graph(true).movements[0]);
        assert!(profile
            .verify_program(&[AffinityId(1)], &[transfer])
            .is_err());
    }

    #[test]
    fn semantic_target_property_and_affinity_have_disjoint_types() {
        let semantic = SemanticTargetPropertyKey(3);
        let physical = AffinityId(3);
        assert_eq!(semantic.0, physical.0);
        // Equal payloads do not create a conversion or shared namespace.
        let _: SemanticTargetPropertyKey = semantic;
        let _: AffinityId = physical;
    }
}
