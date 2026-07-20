//! Compile-checking spike for the normalized scheduled-region contract.
//!
//! Run with:
//! `rustc --edition 2021 --test spikes/scheduling/scheduled_region_model.rs -o /tmp/tiler-schedule-spike && /tmp/tiler-schedule-spike`

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DomainId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AxisId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LocalId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum DomainKind {
    Spatial,
    Reduction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Extent {
    Static(u64),
    Symbol(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Domain {
    id: DomainId,
    name: &'static str,
    kind: DomainKind,
    extent: Extent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IndexRegion {
    identity: &'static str,
    domains: Vec<Domain>,
    scalar_program: &'static str,
    access_maps: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HardwareAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AxisBinding {
    Grid(HardwareAxis),
    Workgroup(HardwareAxis),
    Subgroup,
    Lane,
    Serial,
    VectorLane,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TailPolicy {
    Exact,
    Predicated,
    IdentityPadded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScheduleAxis {
    id: AxisId,
    name: &'static str,
    extent: Extent,
    binding: AxisBinding,
    tail: TailPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CoordExpr {
    Const(u64),
    Axis(AxisId),
    DomainExtent(DomainId),
    Add(Box<CoordExpr>, Box<CoordExpr>),
    Mul(Box<CoordExpr>, Box<CoordExpr>),
    FloorDiv(Box<CoordExpr>, Box<CoordExpr>),
    Mod(Box<CoordExpr>, Box<CoordExpr>),
}

impl CoordExpr {
    fn axis(id: u16) -> Self {
        Self::Axis(AxisId(id))
    }

    fn add(lhs: Self, rhs: Self) -> Self {
        Self::Add(Box::new(lhs), Box::new(rhs))
    }

    fn mul(lhs: Self, rhs: Self) -> Self {
        Self::Mul(Box::new(lhs), Box::new(rhs))
    }

    fn visit_refs(&self, axes: &mut BTreeSet<AxisId>, domains: &mut BTreeSet<DomainId>) {
        match self {
            Self::Axis(id) => {
                axes.insert(*id);
            }
            Self::DomainExtent(id) => {
                domains.insert(*id);
            }
            Self::Add(lhs, rhs)
            | Self::Mul(lhs, rhs)
            | Self::FloorDiv(lhs, rhs)
            | Self::Mod(lhs, rhs) => {
                lhs.visit_refs(axes, domains);
                rhs.visit_refs(axes, domains);
            }
            Self::Const(_) => {}
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LogicalCoordinate {
    domain: DomainId,
    expression: CoordExpr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ActivePredicate {
    Always,
    CoordinatesInBounds(Vec<DomainId>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkAssignment {
    logical_coordinates: Vec<LogicalCoordinate>,
    active: ActivePredicate,
    unique_output_owner: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VectorShape {
    Fixed(u16),
    Scalable { minimum_lanes: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VectorPlan {
    axis: AxisId,
    shape: VectorShape,
    masked: bool,
    required_alignment_bytes: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalAllocation {
    id: LocalId,
    name: &'static str,
    bytes: u64,
    alignment: u16,
    live_phases: (u16, u16),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StagingTransfer {
    local: LocalId,
    write_phase: u16,
    read_phase: u16,
    source_coordinates: Vec<CoordExpr>,
    active: ActivePredicate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParticipantScope {
    Subgroup,
    Workgroup,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BarrierPlan {
    after_phase: u16,
    participants: ParticipantScope,
    convergent: bool,
    fenced_local_allocations: Vec<LocalId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NumericalPermissions {
    reassociation: bool,
    operand_permutation: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ReductionTopology {
    Serial {
        axis: AxisId,
    },
    WorkgroupTree {
        contributor_axis: AxisId,
        local: LocalId,
        combine_strides: Vec<u16>,
        result_owner_axis_value: u16,
        identity_padding_proven: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReductionPlan {
    domain: DomainId,
    topology: ReductionTopology,
    permissions: NumericalPermissions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DispatchMode {
    UniformGroups,
    NonuniformGrid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LaunchPlan {
    mode: DispatchMode,
    grid: [Extent; 3],
    workgroup: [u32; 3],
    dynamic_local_bytes: u64,
    zero_work_is_no_dispatch: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KernelSchedule {
    axes: Vec<ScheduleAxis>,
    work: WorkAssignment,
    vectors: Vec<VectorPlan>,
    locals: Vec<LocalAllocation>,
    transfers: Vec<StagingTransfer>,
    barriers: Vec<BarrierPlan>,
    reductions: Vec<ReductionPlan>,
    launch: LaunchPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScheduledRegion {
    index_region: IndexRegion,
    schedule: KernelSchedule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResourceRequirements {
    threads_per_workgroup: u64,
    static_local_bytes: u64,
    dynamic_local_bytes: u64,
    barrier_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResourceEstimate {
    register_pressure_class: &'static str,
    coalescing_score: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TargetProfile {
    max_threads_per_workgroup: u64,
    max_local_bytes: u64,
    supports_nonuniform_grid: bool,
    supports_scalable_vectors: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Diagnostic {
    rule: &'static str,
    subject: String,
}

fn diagnostic(rule: &'static str, subject: impl Into<String>) -> Diagnostic {
    Diagnostic {
        rule,
        subject: subject.into(),
    }
}

fn verify_intrinsic(region: &ScheduledRegion) -> Result<ResourceRequirements, Diagnostic> {
    let domains: BTreeMap<_, _> = region
        .index_region
        .domains
        .iter()
        .map(|domain| (domain.id, domain))
        .collect();
    let axes: BTreeMap<_, _> = region
        .schedule
        .axes
        .iter()
        .map(|axis| (axis.id, axis))
        .collect();

    if axes.len() != region.schedule.axes.len()
        || region
            .schedule
            .axes
            .iter()
            .enumerate()
            .any(|(index, axis)| axis.id != AxisId(index as u16))
    {
        return Err(diagnostic("schedule.axis_ids.canonical", "schedule.axes"));
    }

    let mapped: BTreeSet<_> = region
        .schedule
        .work
        .logical_coordinates
        .iter()
        .map(|mapping| mapping.domain)
        .collect();
    let expected: BTreeSet<_> = domains.keys().copied().collect();
    if mapped != expected || mapped.len() != region.schedule.work.logical_coordinates.len() {
        return Err(diagnostic(
            "schedule.domain_mapping.complete_once",
            "schedule.work.logical_coordinates",
        ));
    }
    if !region.schedule.work.unique_output_owner {
        return Err(diagnostic("schedule.output_owner.unique", "schedule.work"));
    }

    for mapping in &region.schedule.work.logical_coordinates {
        let mut axis_refs = BTreeSet::new();
        let mut domain_refs = BTreeSet::new();
        mapping
            .expression
            .visit_refs(&mut axis_refs, &mut domain_refs);
        if axis_refs.iter().any(|id| !axes.contains_key(id))
            || domain_refs.iter().any(|id| !domains.contains_key(id))
        {
            return Err(diagnostic(
                "schedule.coordinate_refs.defined",
                format!("domain.{}", mapping.domain.0),
            ));
        }
    }

    let locals: BTreeMap<_, _> = region
        .schedule
        .locals
        .iter()
        .map(|local| (local.id, local))
        .collect();
    if locals.len() != region.schedule.locals.len() {
        return Err(diagnostic("schedule.local_ids.unique", "schedule.locals"));
    }

    if let Some(barrier) = region
        .schedule
        .barriers
        .iter()
        .find(|barrier| !barrier.convergent)
    {
        return Err(diagnostic(
            "schedule.barrier.convergent",
            format!("phase.{}", barrier.after_phase),
        ));
    }

    for transfer in &region.schedule.transfers {
        let Some(local) = locals.get(&transfer.local) else {
            return Err(diagnostic(
                "schedule.transfer.local_exists",
                "transfer.local",
            ));
        };
        if transfer.write_phase >= transfer.read_phase
            || local.live_phases.0 > transfer.write_phase
            || local.live_phases.1 < transfer.read_phase
        {
            return Err(diagnostic(
                "schedule.staging.lifetime",
                format!("local.{}", transfer.local.0),
            ));
        }
        let synchronized = region.schedule.barriers.iter().any(|barrier| {
            barrier.after_phase >= transfer.write_phase
                && barrier.after_phase < transfer.read_phase
                && barrier.convergent
                && barrier.fenced_local_allocations.contains(&transfer.local)
        });
        if !synchronized {
            return Err(diagnostic(
                "schedule.staging.synchronized",
                format!("local.{}", transfer.local.0),
            ));
        }
    }

    let reduction_domains: BTreeSet<_> = region
        .index_region
        .domains
        .iter()
        .filter(|domain| domain.kind == DomainKind::Reduction)
        .map(|domain| domain.id)
        .collect();
    let planned_reductions: BTreeSet<_> = region
        .schedule
        .reductions
        .iter()
        .map(|plan| plan.domain)
        .collect();
    if reduction_domains != planned_reductions
        || planned_reductions.len() != region.schedule.reductions.len()
    {
        return Err(diagnostic(
            "schedule.reduction.coverage",
            "schedule.reductions",
        ));
    }
    for reduction in &region.schedule.reductions {
        match &reduction.topology {
            ReductionTopology::Serial { axis } if !axes.contains_key(axis) => {
                return Err(diagnostic("schedule.reduction.axis_exists", "serial.axis"));
            }
            ReductionTopology::WorkgroupTree {
                contributor_axis,
                local,
                combine_strides,
                identity_padding_proven,
                ..
            } => {
                if !axes.contains_key(contributor_axis) || !locals.contains_key(local) {
                    return Err(diagnostic(
                        "schedule.reduction.resources_exist",
                        format!("domain.{}", reduction.domain.0),
                    ));
                }
                if !reduction.permissions.reassociation
                    || !reduction.permissions.operand_permutation
                {
                    return Err(diagnostic(
                        "schedule.reduction.numerical_order",
                        format!("domain.{}", reduction.domain.0),
                    ));
                }
                if combine_strides.is_empty()
                    || !combine_strides
                        .windows(2)
                        .all(|window| window[0] > window[1])
                    || !identity_padding_proven
                {
                    return Err(diagnostic(
                        "schedule.reduction.tree_complete",
                        format!("domain.{}", reduction.domain.0),
                    ));
                }
            }
            ReductionTopology::Serial { .. } => {}
        }
    }

    for vector in &region.schedule.vectors {
        if !axes.contains_key(&vector.axis)
            || vector.required_alignment_bytes == 0
            || matches!(
                vector.shape,
                VectorShape::Fixed(0) | VectorShape::Scalable { minimum_lanes: 0 }
            )
        {
            return Err(diagnostic(
                "schedule.vector.well_formed",
                "schedule.vectors",
            ));
        }
    }

    let threads_per_workgroup = region
        .schedule
        .launch
        .workgroup
        .iter()
        .try_fold(1_u64, |product, value| {
            product.checked_mul(u64::from(*value))
        })
        .ok_or_else(|| diagnostic("schedule.launch.product_no_overflow", "launch.workgroup"))?;
    if threads_per_workgroup == 0 {
        return Err(diagnostic(
            "schedule.launch.nonzero_workgroup",
            "launch.workgroup",
        ));
    }
    let static_local_bytes = region
        .schedule
        .locals
        .iter()
        .try_fold(0_u64, |sum, local| sum.checked_add(local.bytes))
        .ok_or_else(|| diagnostic("schedule.local_bytes.no_overflow", "schedule.locals"))?;

    Ok(ResourceRequirements {
        threads_per_workgroup,
        static_local_bytes,
        dynamic_local_bytes: region.schedule.launch.dynamic_local_bytes,
        barrier_count: region.schedule.barriers.len() as u32,
    })
}

fn assess_target(
    region: &ScheduledRegion,
    resources: &ResourceRequirements,
    target: &TargetProfile,
) -> Result<(), Diagnostic> {
    if resources.threads_per_workgroup > target.max_threads_per_workgroup {
        return Err(diagnostic(
            "target.threads_per_workgroup",
            "resources.threads_per_workgroup",
        ));
    }
    let local_total = resources
        .static_local_bytes
        .checked_add(resources.dynamic_local_bytes)
        .ok_or_else(|| diagnostic("target.local_bytes.no_overflow", "resources.local_bytes"))?;
    if local_total > target.max_local_bytes {
        return Err(diagnostic("target.local_bytes", "resources.local_bytes"));
    }
    if region.schedule.launch.mode == DispatchMode::NonuniformGrid
        && !target.supports_nonuniform_grid
    {
        return Err(diagnostic(
            "target.nonuniform_dispatch",
            "schedule.launch.mode",
        ));
    }
    if region.schedule.vectors.iter().any(|vector| {
        matches!(vector.shape, VectorShape::Scalable { .. }) && !target.supports_scalable_vectors
    }) {
        return Err(diagnostic("target.scalable_vectors", "schedule.vectors"));
    }
    Ok(())
}

fn pointwise() -> ScheduledRegion {
    ScheduledRegion {
        index_region: IndexRegion {
            identity: "z[i] = max(x[i] + y[i], 0)",
            domains: vec![Domain {
                id: DomainId(0),
                name: "i",
                kind: DomainKind::Spatial,
                extent: Extent::Symbol("N"),
            }],
            scalar_program: "max(add(load x, load y), 0)",
            access_maps: vec!["x[i]", "y[i]", "z[i]"],
        },
        schedule: KernelSchedule {
            axes: vec![
                ScheduleAxis {
                    id: AxisId(0),
                    name: "group_x",
                    extent: Extent::Symbol("ceil_div(N, 256)"),
                    binding: AxisBinding::Grid(HardwareAxis::X),
                    tail: TailPolicy::Exact,
                },
                ScheduleAxis {
                    id: AxisId(1),
                    name: "thread_x",
                    extent: Extent::Static(256),
                    binding: AxisBinding::Workgroup(HardwareAxis::X),
                    tail: TailPolicy::Predicated,
                },
            ],
            work: WorkAssignment {
                logical_coordinates: vec![LogicalCoordinate {
                    domain: DomainId(0),
                    expression: CoordExpr::add(
                        CoordExpr::mul(CoordExpr::axis(0), CoordExpr::Const(256)),
                        CoordExpr::axis(1),
                    ),
                }],
                active: ActivePredicate::CoordinatesInBounds(vec![DomainId(0)]),
                unique_output_owner: true,
            },
            vectors: vec![],
            locals: vec![],
            transfers: vec![],
            barriers: vec![],
            reductions: vec![],
            launch: LaunchPlan {
                mode: DispatchMode::UniformGroups,
                grid: [
                    Extent::Symbol("ceil_div(N, 256)"),
                    Extent::Static(1),
                    Extent::Static(1),
                ],
                workgroup: [256, 1, 1],
                dynamic_local_bytes: 0,
                zero_work_is_no_dispatch: true,
            },
        },
    }
}

fn broadcast_pointwise() -> ScheduledRegion {
    let mut region = pointwise();
    region.index_region.identity = "z[b,m,n] = x[b,m,n] + bias[n]";
    region.index_region.domains = vec![
        Domain {
            id: DomainId(0),
            name: "b",
            kind: DomainKind::Spatial,
            extent: Extent::Symbol("B"),
        },
        Domain {
            id: DomainId(1),
            name: "m",
            kind: DomainKind::Spatial,
            extent: Extent::Symbol("M"),
        },
        Domain {
            id: DomainId(2),
            name: "n",
            kind: DomainKind::Spatial,
            extent: Extent::Symbol("N"),
        },
    ];
    region.index_region.scalar_program = "add(load x[b,m,n], load bias[n])";
    region.index_region.access_maps = vec!["x[b,m,n]", "bias[n]", "z[b,m,n]"];
    let linear = CoordExpr::add(
        CoordExpr::mul(CoordExpr::axis(0), CoordExpr::Const(256)),
        CoordExpr::axis(1),
    );
    let n = CoordExpr::DomainExtent(DomainId(2));
    let m = CoordExpr::DomainExtent(DomainId(1));
    let mn = CoordExpr::mul(m.clone(), n.clone());
    region.schedule.work.logical_coordinates = vec![
        LogicalCoordinate {
            domain: DomainId(0),
            expression: CoordExpr::FloorDiv(Box::new(linear.clone()), Box::new(mn)),
        },
        LogicalCoordinate {
            domain: DomainId(1),
            expression: CoordExpr::Mod(
                Box::new(CoordExpr::FloorDiv(
                    Box::new(linear.clone()),
                    Box::new(n.clone()),
                )),
                Box::new(m),
            ),
        },
        LogicalCoordinate {
            domain: DomainId(2),
            expression: CoordExpr::Mod(Box::new(linear), Box::new(n)),
        },
    ];
    region.schedule.work.active =
        ActivePredicate::CoordinatesInBounds(vec![DomainId(0), DomainId(1), DomainId(2)]);
    region.schedule.axes[0].extent = Extent::Symbol("ceil_div(B*M*N, 256)");
    region.schedule.launch.grid[0] = Extent::Symbol("ceil_div(B*M*N, 256)");
    region
}

fn pointwise_vector4() -> ScheduledRegion {
    let mut region = pointwise();
    region.schedule.axes = vec![
        ScheduleAxis {
            id: AxisId(0),
            name: "group_x",
            extent: Extent::Symbol("ceil_div(N,256)"),
            binding: AxisBinding::Grid(HardwareAxis::X),
            tail: TailPolicy::Exact,
        },
        ScheduleAxis {
            id: AxisId(1),
            name: "thread_x",
            extent: Extent::Static(64),
            binding: AxisBinding::Workgroup(HardwareAxis::X),
            tail: TailPolicy::Predicated,
        },
        ScheduleAxis {
            id: AxisId(2),
            name: "vector_lane",
            extent: Extent::Static(4),
            binding: AxisBinding::VectorLane,
            tail: TailPolicy::Predicated,
        },
    ];
    region.schedule.work.logical_coordinates = vec![LogicalCoordinate {
        domain: DomainId(0),
        expression: CoordExpr::add(
            CoordExpr::mul(
                CoordExpr::add(
                    CoordExpr::mul(CoordExpr::axis(0), CoordExpr::Const(64)),
                    CoordExpr::axis(1),
                ),
                CoordExpr::Const(4),
            ),
            CoordExpr::axis(2),
        ),
    }];
    region.schedule.vectors = vec![VectorPlan {
        axis: AxisId(2),
        shape: VectorShape::Fixed(4),
        masked: true,
        required_alignment_bytes: 16,
    }];
    region.schedule.launch.workgroup = [64, 1, 1];
    region
}

fn tiled_transpose() -> ScheduledRegion {
    ScheduledRegion {
        index_region: IndexRegion {
            identity: "y[m,n] = x[n,m]",
            domains: vec![
                Domain {
                    id: DomainId(0),
                    name: "m",
                    kind: DomainKind::Spatial,
                    extent: Extent::Symbol("M"),
                },
                Domain {
                    id: DomainId(1),
                    name: "n",
                    kind: DomainKind::Spatial,
                    extent: Extent::Symbol("N"),
                },
            ],
            scalar_program: "load x[n,m]",
            access_maps: vec!["x[n,m]", "y[m,n]"],
        },
        schedule: KernelSchedule {
            axes: vec![
                ScheduleAxis {
                    id: AxisId(0),
                    name: "tile_m",
                    extent: Extent::Symbol("ceil_div(M,32)"),
                    binding: AxisBinding::Grid(HardwareAxis::X),
                    tail: TailPolicy::Exact,
                },
                ScheduleAxis {
                    id: AxisId(1),
                    name: "tile_n",
                    extent: Extent::Symbol("ceil_div(N,32)"),
                    binding: AxisBinding::Grid(HardwareAxis::Y),
                    tail: TailPolicy::Exact,
                },
                ScheduleAxis {
                    id: AxisId(2),
                    name: "lane_x",
                    extent: Extent::Static(32),
                    binding: AxisBinding::Workgroup(HardwareAxis::X),
                    tail: TailPolicy::Predicated,
                },
                ScheduleAxis {
                    id: AxisId(3),
                    name: "lane_y",
                    extent: Extent::Static(8),
                    binding: AxisBinding::Workgroup(HardwareAxis::Y),
                    tail: TailPolicy::Predicated,
                },
                ScheduleAxis {
                    id: AxisId(4),
                    name: "serial_q",
                    extent: Extent::Static(4),
                    binding: AxisBinding::Serial,
                    tail: TailPolicy::Predicated,
                },
            ],
            work: WorkAssignment {
                logical_coordinates: vec![
                    LogicalCoordinate {
                        domain: DomainId(0),
                        expression: CoordExpr::add(
                            CoordExpr::mul(CoordExpr::axis(0), CoordExpr::Const(32)),
                            CoordExpr::add(
                                CoordExpr::axis(3),
                                CoordExpr::mul(CoordExpr::axis(4), CoordExpr::Const(8)),
                            ),
                        ),
                    },
                    LogicalCoordinate {
                        domain: DomainId(1),
                        expression: CoordExpr::add(
                            CoordExpr::mul(CoordExpr::axis(1), CoordExpr::Const(32)),
                            CoordExpr::axis(2),
                        ),
                    },
                ],
                active: ActivePredicate::CoordinatesInBounds(vec![DomainId(0), DomainId(1)]),
                unique_output_owner: true,
            },
            vectors: vec![],
            locals: vec![LocalAllocation {
                id: LocalId(0),
                name: "tile[32][33]",
                bytes: 32 * 33 * 4,
                alignment: 16,
                live_phases: (0, 2),
            }],
            transfers: vec![StagingTransfer {
                local: LocalId(0),
                write_phase: 0,
                read_phase: 2,
                source_coordinates: vec![
                    CoordExpr::axis(0),
                    CoordExpr::axis(1),
                    CoordExpr::axis(2),
                    CoordExpr::axis(3),
                    CoordExpr::axis(4),
                ],
                active: ActivePredicate::CoordinatesInBounds(vec![DomainId(0), DomainId(1)]),
            }],
            barriers: vec![BarrierPlan {
                after_phase: 1,
                participants: ParticipantScope::Workgroup,
                convergent: true,
                fenced_local_allocations: vec![LocalId(0)],
            }],
            reductions: vec![],
            launch: LaunchPlan {
                mode: DispatchMode::UniformGroups,
                grid: [
                    Extent::Symbol("ceil_div(M,32)"),
                    Extent::Symbol("ceil_div(N,32)"),
                    Extent::Static(1),
                ],
                workgroup: [32, 8, 1],
                dynamic_local_bytes: 0,
                zero_work_is_no_dispatch: true,
            },
        },
    }
}

fn row_reduction(permissions: NumericalPermissions) -> ScheduledRegion {
    ScheduledRegion {
        index_region: IndexRegion {
            identity: "y[m] = reduce_add n x[m,n]",
            domains: vec![
                Domain {
                    id: DomainId(0),
                    name: "m",
                    kind: DomainKind::Spatial,
                    extent: Extent::Symbol("M"),
                },
                Domain {
                    id: DomainId(1),
                    name: "n",
                    kind: DomainKind::Reduction,
                    extent: Extent::Symbol("N"),
                },
            ],
            scalar_program: "add(acc, load x[m,n])",
            access_maps: vec!["x[m,n]", "y[m]"],
        },
        schedule: KernelSchedule {
            axes: vec![
                ScheduleAxis {
                    id: AxisId(0),
                    name: "row_group",
                    extent: Extent::Symbol("M"),
                    binding: AxisBinding::Grid(HardwareAxis::X),
                    tail: TailPolicy::Exact,
                },
                ScheduleAxis {
                    id: AxisId(1),
                    name: "contributor",
                    extent: Extent::Static(256),
                    binding: AxisBinding::Workgroup(HardwareAxis::X),
                    tail: TailPolicy::IdentityPadded,
                },
                ScheduleAxis {
                    id: AxisId(2),
                    name: "serial_n",
                    extent: Extent::Symbol("ceil_div(N,256)"),
                    binding: AxisBinding::Serial,
                    tail: TailPolicy::IdentityPadded,
                },
            ],
            work: WorkAssignment {
                logical_coordinates: vec![
                    LogicalCoordinate {
                        domain: DomainId(0),
                        expression: CoordExpr::axis(0),
                    },
                    LogicalCoordinate {
                        domain: DomainId(1),
                        expression: CoordExpr::add(
                            CoordExpr::axis(1),
                            CoordExpr::mul(CoordExpr::axis(2), CoordExpr::Const(256)),
                        ),
                    },
                ],
                active: ActivePredicate::CoordinatesInBounds(vec![DomainId(0), DomainId(1)]),
                unique_output_owner: true,
            },
            vectors: vec![],
            locals: vec![LocalAllocation {
                id: LocalId(0),
                name: "partials[256]",
                bytes: 256 * 4,
                alignment: 16,
                live_phases: (0, 9),
            }],
            transfers: vec![],
            barriers: (0..8)
                .map(|phase| BarrierPlan {
                    after_phase: phase,
                    participants: ParticipantScope::Workgroup,
                    convergent: true,
                    fenced_local_allocations: vec![LocalId(0)],
                })
                .collect(),
            reductions: vec![ReductionPlan {
                domain: DomainId(1),
                topology: ReductionTopology::WorkgroupTree {
                    contributor_axis: AxisId(1),
                    local: LocalId(0),
                    combine_strides: vec![128, 64, 32, 16, 8, 4, 2, 1],
                    result_owner_axis_value: 0,
                    identity_padding_proven: true,
                },
                permissions,
            }],
            launch: LaunchPlan {
                mode: DispatchMode::UniformGroups,
                grid: [Extent::Symbol("M"), Extent::Static(1), Extent::Static(1)],
                workgroup: [256, 1, 1],
                dynamic_local_bytes: 0,
                zero_work_is_no_dispatch: true,
            },
        },
    }
}

fn row_reduction_serial() -> ScheduledRegion {
    let mut region = row_reduction(NumericalPermissions {
        reassociation: false,
        operand_permutation: false,
    });
    region.schedule.axes = vec![
        ScheduleAxis {
            id: AxisId(0),
            name: "row_group",
            extent: Extent::Symbol("ceil_div(M,64)"),
            binding: AxisBinding::Grid(HardwareAxis::X),
            tail: TailPolicy::Exact,
        },
        ScheduleAxis {
            id: AxisId(1),
            name: "row_thread",
            extent: Extent::Static(64),
            binding: AxisBinding::Workgroup(HardwareAxis::X),
            tail: TailPolicy::Predicated,
        },
        ScheduleAxis {
            id: AxisId(2),
            name: "serial_n",
            extent: Extent::Symbol("N"),
            binding: AxisBinding::Serial,
            tail: TailPolicy::Exact,
        },
    ];
    region.schedule.work.logical_coordinates = vec![
        LogicalCoordinate {
            domain: DomainId(0),
            expression: CoordExpr::add(
                CoordExpr::mul(CoordExpr::axis(0), CoordExpr::Const(64)),
                CoordExpr::axis(1),
            ),
        },
        LogicalCoordinate {
            domain: DomainId(1),
            expression: CoordExpr::axis(2),
        },
    ];
    region.schedule.locals.clear();
    region.schedule.barriers.clear();
    region.schedule.reductions = vec![ReductionPlan {
        domain: DomainId(1),
        topology: ReductionTopology::Serial { axis: AxisId(2) },
        permissions: NumericalPermissions {
            reassociation: false,
            operand_permutation: false,
        },
    }];
    region.schedule.launch.grid[0] = Extent::Symbol("ceil_div(M,64)");
    region.schedule.launch.workgroup = [64, 1, 1];
    region
}

#[cfg(test)]
mod tests {
    use super::*;

    const TARGET: TargetProfile = TargetProfile {
        max_threads_per_workgroup: 512,
        max_local_bytes: 32 * 1024,
        supports_nonuniform_grid: false,
        supports_scalable_vectors: false,
    };

    #[test]
    fn required_workloads_and_alternatives_are_intrinsically_valid() {
        let permissive = NumericalPermissions {
            reassociation: true,
            operand_permutation: true,
        };
        for region in [
            pointwise(),
            pointwise_vector4(),
            broadcast_pointwise(),
            tiled_transpose(),
            row_reduction_serial(),
            row_reduction(permissive),
        ] {
            let resources = verify_intrinsic(&region).unwrap();
            assess_target(&region, &resources, &TARGET).unwrap();
        }
    }

    #[test]
    fn parallel_reduction_is_rejected_without_both_numerical_permissions() {
        let region = row_reduction(NumericalPermissions {
            reassociation: true,
            operand_permutation: false,
        });
        let error = verify_intrinsic(&region).unwrap_err();
        assert_eq!(error.rule, "schedule.reduction.numerical_order");
    }

    #[test]
    fn divergent_barrier_is_an_intrinsic_failure() {
        let mut region = tiled_transpose();
        region.schedule.barriers[0].convergent = false;
        let error = verify_intrinsic(&region).unwrap_err();
        assert_eq!(error.rule, "schedule.barrier.convergent");
    }

    #[test]
    fn incomplete_domain_mapping_is_an_intrinsic_failure() {
        let mut region = broadcast_pointwise();
        region.schedule.work.logical_coordinates.pop();
        let error = verify_intrinsic(&region).unwrap_err();
        assert_eq!(error.rule, "schedule.domain_mapping.complete_once");
    }

    #[test]
    fn hard_target_limit_is_not_an_intrinsic_or_cost_failure() {
        let region = pointwise();
        let resources = verify_intrinsic(&region).unwrap();
        let tiny_target = TargetProfile {
            max_threads_per_workgroup: 64,
            ..TARGET
        };
        let error = assess_target(&region, &resources, &tiny_target).unwrap_err();
        assert_eq!(error.rule, "target.threads_per_workgroup");

        let transpose = tiled_transpose();
        let resources = verify_intrinsic(&transpose).unwrap();
        let tiny_local_target = TargetProfile {
            max_local_bytes: 1024,
            ..TARGET
        };
        let error = assess_target(&transpose, &resources, &tiny_local_target).unwrap_err();
        assert_eq!(error.rule, "target.local_bytes");

        let deliberately_bad_cost = ResourceEstimate {
            register_pressure_class: "very-high",
            coalescing_score: 0,
        };
        assert_eq!(deliberately_bad_cost.coalescing_score, 0);
        assert!(verify_intrinsic(&region).is_ok());
    }
}
