//! This backend's consumer-selected runtime adapter, and the route that uses it.
//!
//! Nothing here registers anything. A caller links this adapter, constructs it,
//! and hands it to `route_with_adapter`; that is the whole mechanism ADR 0090
//! leaves for row 12, and the adapter's own identity never travels in an
//! artifact.
//!
//! # The execution host is a real resource with a real terminal use
//!
//! This adapter does not evaluate on the calling thread. It acquires a worker
//! thread before the routing commit, **moves** the routed storage into that
//! worker, and gets the storage back only when the worker has finished writing
//! through it. So the storage is owned by the adapter across a boundary the
//! adapter itself has to close, and a `dispatch` that returned before closing
//! it would be returning while device-side work was still outstanding.
//!
//! That property is checked rather than described. Every submitted entry must
//! come back carrying a [`TerminalUse`], a value only the worker loop can
//! construct and only after the entry has finished, and `dispatch` refuses
//! unless it has witnessed one per submission. It is not a second completion
//! authority: the token is minted by the same worker whose write it attests,
//! the loader is not consulted, and nothing outside this adapter can see it.
//!
//! # Acquiring the host can genuinely fail, and the policy is the caller's
//!
//! [`HostRequest`] carries the stack this backend's worker needs. A host that
//! cannot spawn a thread with it is reported as [`HostUnavailable`], a typed
//! outcome that has no path to a pass: [`ExecutionOutcome`] does not implement
//! equality, holds no default, and exposes completed bits only through
//! [`ExecutionOutcome::completed`], which answers `None` for an unavailable
//! host. What unavailability *means* is decided by [`HostPolicy`] at the call
//! site, and no ambient environment variable is read anywhere in this file.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{Builder, JoinHandle};

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArithmeticType, AvailabilityPhase, BackendKey, BindingTarget,
    RecordedArtifactProgramIdentity, RepresentationKey, TargetProfileRef,
};
use tiler_runtime::adapter::{
    AdapterRouteFailure, LiveExecutionContext, RuntimeAdapter, route_with_adapter,
};
use tiler_runtime::load::{
    DTypeDispatch, DecodedProgram, ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest,
    LoadRejection, Preflight, PreparedEntryObservation, RoutedDispatch, RoutedEntry,
    TargetEnvironmentObservation, TargetEnvironmentSupport, TargetPropertyRequest,
};

use crate::backend::{
    BACKEND_KEY, F32_BYTES, REPRESENTATION_KEY, SOLE_DELIVERY, WORKGROUP_THREADS,
};
use crate::graph::{Graph, GraphEntry, GraphRefusal, Node, NodeKind, decode};

/// What a caller does when this backend's execution host is unavailable.
///
/// The adapter always refuses an unavailable host; this decides what the
/// refusal *means* to the run, which is a caller's judgement and not a
/// backend's. Neither value is read from the environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostPolicy {
    /// Unavailability fails the run.
    Require,
    /// Unavailability is an outcome the run reports and does not pass.
    Report,
}

/// What a caller asks of the host before this backend will execute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HostRequest {
    /// What unavailability means to this caller.
    pub(crate) policy: HostPolicy,
    /// Stack this backend's worker needs, in bytes.
    pub(crate) stack_bytes: usize,
}

impl HostRequest {
    /// The stack one nodefold worker needs for the graphs this fixture builds.
    pub(crate) const ORDINARY_STACK: usize = 256 * 1024;

    /// A request an ordinary host satisfies.
    pub(crate) const fn ordinary(policy: HostPolicy) -> Self {
        Self {
            policy,
            stack_bytes: Self::ORDINARY_STACK,
        }
    }

    /// A request no host satisfies, used to reach the unavailable outcome.
    pub(crate) const fn unsatisfiable(policy: HostPolicy) -> Self {
        Self {
            policy,
            stack_bytes: usize::MAX,
        }
    }
}

/// This host could not supply the execution resource this backend needs.
///
/// Carries what was missing. It is never converted into a pass: see the module
/// header for the three properties that make that structural.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostUnavailable {
    /// What the host could not supply, in the host's own words.
    pub(crate) reason: String,
}

impl fmt::Display for HostUnavailable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "nodefold.host-unavailable: {reason}",
            reason = self.reason
        )
    }
}

/// What this adapter reports when the loader asks it to bind a context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Binding {
    /// The three identities this adapter can actually execute.
    Observed,
    /// The identities it would prefer to execute, which is not the same claim.
    Preferred,
}

/// What this adapter's worker does with a submitted entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Evaluation {
    /// Evaluate the decoded graph and write the result.
    Fold,
    /// Report terminal success without evaluating anything.
    Certify,
}

/// How this adapter treats its worker's terminal use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Lifetime {
    /// Retain the routed storage until every submission is witnessed complete.
    AwaitTerminalUse,
    /// Return from `dispatch` while submissions are still outstanding.
    ReturnBeforeTerminalUse,
}

/// The complete behaviour one constructed adapter takes.
///
/// Every field but [`Self::host`] has exactly one sound value; the others exist
/// so a test can perturb the adapter itself rather than an assertion about it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Behaviour {
    /// What this caller asks of the host, and what unavailability means to it.
    pub(crate) host: HostRequest,
    /// What the adapter reports when asked to bind a context.
    pub(crate) binding: Binding,
    /// What its worker does with a submitted entry.
    pub(crate) evaluation: Evaluation,
    /// How it treats its worker's terminal use.
    pub(crate) lifetime: Lifetime,
}

impl Behaviour {
    /// The sound behaviour, with the caller's host policy.
    pub(crate) const fn sound(policy: HostPolicy) -> Self {
        Self {
            host: HostRequest::ordinary(policy),
            binding: Binding::Observed,
            evaluation: Evaluation::Fold,
            lifetime: Lifetime::AwaitTerminalUse,
        }
    }
}

/// Why this adapter refused before the routing commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdapterRefusal {
    /// The host could not supply this backend's execution resource.
    HostUnavailable(HostUnavailable),
    /// The carried bytes are not a nodefold graph this build executes.
    Payload(GraphRefusal),
    /// A mapped symbol names no entry of the carried graph.
    UnmappedSymbol(String),
    /// The route asks for more threads per workgroup than this backend admits.
    Capacity {
        /// Position of the entry in execution order.
        entry: usize,
        /// Threads per workgroup the route asks for.
        requested: u64,
        /// Threads per workgroup this backend admits.
        admitted: u64,
    },
    /// The route addresses more input bytes than the caller supplied.
    SuppliedInput {
        /// Position of the entry in execution order.
        entry: usize,
        /// The ABI slot whose range exceeds the supply.
        slot: usize,
        /// Bytes the range addresses.
        needed: u64,
        /// Bytes the caller supplied.
        supplied: u64,
    },
    /// The adapter's own bookkeeping disagreed with the route.
    Bookkeeping(String),
}

impl fmt::Display for AdapterRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HostUnavailable(cause) => write!(formatter, "{cause}"),
            Self::Payload(cause) => write!(formatter, "nodefold.payload: {cause}"),
            Self::UnmappedSymbol(symbol) => write!(
                formatter,
                "nodefold.payload: symbol `{symbol}` names no graph entry",
            ),
            Self::Capacity {
                entry,
                requested,
                admitted,
            } => write!(
                formatter,
                "nodefold.plan: entry {entry} launches {requested} thread(s) per workgroup and this backend admits {admitted}",
            ),
            Self::SuppliedInput {
                entry,
                slot,
                needed,
                supplied,
            } => write!(
                formatter,
                "nodefold.plan: entry {entry} slot {slot} addresses {needed} input byte(s) and the caller supplied {supplied}",
            ),
            Self::Bookkeeping(message) => write!(formatter, "nodefold.plan: {message}"),
        }
    }
}

impl Error for AdapterRefusal {}

/// Why this adapter failed after the routing commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdapterFailure {
    /// `dispatch` reached its end with submissions still outstanding.
    ///
    /// The exact defect ADR 0051 and ADR 0090 item 12 are about: a route that
    /// reports success while its resources are still in use has reported
    /// something it did not observe.
    TerminalUseUnwitnessed {
        /// Entries submitted to the worker.
        submitted: usize,
        /// Terminal uses witnessed coming back.
        witnessed: usize,
    },
    /// The worker stopped before answering.
    WorkerLost,
    /// The committed evaluation refused.
    Evaluate(String),
    /// The committed route published no program output to read back.
    NoProgramOutput,
}

impl fmt::Display for AdapterFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TerminalUseUnwitnessed {
                submitted,
                witnessed,
            } => write!(
                formatter,
                "nodefold.dispatch: {submitted} entr(y/ies) were submitted and {witnessed} terminal use(s) were witnessed; the routed storage is still outstanding",
            ),
            Self::WorkerLost => {
                formatter.write_str("nodefold.dispatch: the execution worker stopped")
            }
            Self::Evaluate(message) => write!(formatter, "nodefold.dispatch: {message}"),
            Self::NoProgramOutput => {
                formatter.write_str("nodefold.dispatch: the route published no program output")
            }
        }
    }
}

impl Error for AdapterFailure {}

/// What one completed route produced, or why this host could not run it.
///
/// Deliberately without `PartialEq`, `Default`, or any accessor that yields
/// bits for an unavailable host. An unavailable outcome cannot be compared
/// equal to a completed one because the type offers no comparison at all, and
/// the oracle comparison below takes bits rather than an outcome, so reaching
/// it requires destructuring a completion.
#[derive(Clone, Debug)]
pub(crate) enum ExecutionOutcome {
    /// The route completed and produced these output bit patterns.
    Completed(Vec<u32>),
    /// This host could not supply the execution resource.
    Unavailable(HostUnavailable),
}

impl ExecutionOutcome {
    /// Returns the completed bits, and `None` for an unavailable host.
    pub(crate) fn completed(&self) -> Option<&[u32]> {
        match self {
            Self::Completed(bits) => Some(bits),
            Self::Unavailable(_) => None,
        }
    }
}

/// Where one routed binding's bytes live in this adapter's storage.
#[derive(Clone, Copy, Debug)]
struct Placement {
    allocation: usize,
    offset: u64,
    bytes: u64,
}

/// Proof that one submitted entry reached its terminal use.
///
/// Constructed only inside [`worker_loop`], only after the entry has finished
/// writing, and moved back to the adapter with the storage it held.
#[derive(Debug)]
struct TerminalUse {
    entry: usize,
}

struct Job {
    entry: usize,
    graph_entry: GraphEntry,
    invocations: u64,
    evaluation: Evaluation,
    storage: Vec<Vec<u8>>,
    placements: Vec<Placement>,
}

struct Done {
    storage: Vec<Vec<u8>>,
    outcome: Result<TerminalUse, String>,
}

/// The acquired execution host: one worker thread and the channels to it.
struct Worker {
    jobs: Option<Sender<Job>>,
    done: Receiver<Done>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn acquire(request: HostRequest) -> Result<Self, HostUnavailable> {
        let (job_sender, job_receiver) = channel::<Job>();
        let (done_sender, done_receiver) = channel::<Done>();
        let handle = Builder::new()
            .name("nodefold-worker".to_owned())
            .stack_size(request.stack_bytes)
            .spawn(move || worker_loop(&job_receiver, &done_sender))
            .map_err(|error| HostUnavailable {
                reason: format!(
                    "this host cannot supply an execution thread with a {stack}-byte stack: {error}",
                    stack = request.stack_bytes,
                ),
            })?;
        Ok(Self {
            jobs: Some(job_sender),
            done: done_receiver,
            handle: Some(handle),
        })
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Closing the job channel is what ends the loop; joining is what makes
        // the thread's exit part of this adapter's own lifetime rather than
        // something the process is left holding.
        self.jobs = None;
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn worker_loop(jobs: &Receiver<Job>, done: &Sender<Done>) {
    while let Ok(job) = jobs.recv() {
        let mut storage = job.storage;
        let outcome = match job.evaluation {
            Evaluation::Fold => run_entry(
                &job.graph_entry,
                job.invocations,
                &mut storage,
                &job.placements,
            ),
            // Reports terminal success having evaluated nothing. The storage
            // comes back exactly as it was allocated.
            Evaluation::Certify => Ok(()),
        }
        .map(|()| TerminalUse { entry: job.entry });
        if done.send(Done { storage, outcome }).is_err() {
            break;
        }
    }
}

/// One evaluated value of the node table.
#[derive(Clone, Copy, Debug)]
enum Value {
    Index(u64),
    F32(u32),
    Bool(bool),
}

impl Value {
    fn kind(self) -> NodeKind {
        match self {
            Self::Index(_) => NodeKind::Index,
            Self::F32(_) => NodeKind::F32,
            Self::Bool(_) => NodeKind::Bool,
        }
    }
}

/// Evaluates one entry over every invocation of its launch.
fn run_entry(
    entry: &GraphEntry,
    invocations: u64,
    storage: &mut [Vec<u8>],
    placements: &[Placement],
) -> Result<(), String> {
    // The two demand sets, computed once for the whole launch because the node
    // table does not change between invocations. Only the values do.
    let guarded = closure(&entry.nodes, entry.store.guard.into_iter());
    let stored = closure(
        &entry.nodes,
        [entry.store.offset, entry.store.value].into_iter(),
    );
    for invocation in 0..invocations {
        let mut values: Vec<Option<Value>> = vec![None; entry.nodes.len()];
        evaluate(entry, invocation, storage, placements, &guarded, &mut values)?;
        let permitted = match entry.store.guard {
            None => true,
            Some(guard) => match read(&values, guard)? {
                Value::Bool(verdict) => verdict,
                other => {
                    return Err(format!(
                        "the store plan's guard node {guard} evaluated to a {kind:?}",
                        kind = other.kind(),
                    ));
                }
            },
        };
        if !permitted {
            continue;
        }
        evaluate(entry, invocation, storage, placements, &stored, &mut values)?;
        let offset = match read(&values, entry.store.offset)? {
            Value::Index(offset) => offset,
            other => {
                return Err(format!(
                    "the store plan's offset evaluated to a {kind:?}",
                    kind = other.kind(),
                ));
            }
        };
        let bits = match read(&values, entry.store.value)? {
            Value::F32(bits) => bits,
            other => {
                return Err(format!(
                    "the store plan's value evaluated to a {kind:?}",
                    kind = other.kind(),
                ));
            }
        };
        access(
            entry,
            storage,
            placements,
            entry.store.buffer,
            offset,
            Some(bits),
        )?;
    }
    Ok(())
}

/// Returns which nodes the given roots transitively need.
///
/// One reverse pass suffices because the decoder proved every operand is an
/// earlier ordinal, which is the property that makes this representation a
/// table rather than a program.
fn closure(nodes: &[Node], roots: impl Iterator<Item = u32>) -> Vec<bool> {
    let mut needed = vec![false; nodes.len()];
    for root in roots {
        if let Some(slot) = needed.get_mut(root as usize) {
            *slot = true;
        }
    }
    for ordinal in (0..nodes.len()).rev() {
        if !needed[ordinal] {
            continue;
        }
        for operand in operands(nodes[ordinal]) {
            if let Some(slot) = needed.get_mut(operand as usize) {
                *slot = true;
            }
        }
    }
    needed
}

fn operands(node: Node) -> Vec<u32> {
    match node {
        Node::InvocationIndex | Node::IndexConstant(_) | Node::F32Constant(_) => Vec::new(),
        Node::CanonicalizeF32Nan(source) => vec![source],
        Node::IndexAdd(lhs, rhs)
        | Node::IndexMultiply(lhs, rhs)
        | Node::IndexLessThan(lhs, rhs)
        | Node::F32Multiply(lhs, rhs)
        | Node::F32Add(lhs, rhs) => vec![lhs, rhs],
        Node::Load { offset, .. } => vec![offset],
    }
}

/// Evaluates every needed node not already evaluated, in table order.
fn evaluate(
    entry: &GraphEntry,
    invocation: u64,
    storage: &mut [Vec<u8>],
    placements: &[Placement],
    needed: &[bool],
    values: &mut [Option<Value>],
) -> Result<(), String> {
    for (ordinal, node) in entry.nodes.iter().enumerate() {
        if !needed[ordinal] || values[ordinal].is_some() {
            continue;
        }
        let value = match *node {
            Node::InvocationIndex => Value::Index(invocation),
            Node::IndexConstant(literal) => Value::Index(literal),
            Node::F32Constant(bits) => Value::F32(bits),
            Node::IndexAdd(lhs, rhs) => Value::Index(
                index(values, lhs)?
                    .checked_add(index(values, rhs)?)
                    .ok_or_else(|| "an index sum overflowed".to_owned())?,
            ),
            Node::IndexMultiply(lhs, rhs) => Value::Index(
                index(values, lhs)?
                    .checked_mul(index(values, rhs)?)
                    .ok_or_else(|| "an index product overflowed".to_owned())?,
            ),
            Node::IndexLessThan(lhs, rhs) => {
                Value::Bool(index(values, lhs)? < index(values, rhs)?)
            }
            Node::F32Multiply(lhs, rhs) => Value::F32(canonicalize(
                f32::from_bits(float(values, lhs)?) * f32::from_bits(float(values, rhs)?),
                entry.canonical_nan,
            )),
            Node::F32Add(lhs, rhs) => Value::F32(canonicalize(
                f32::from_bits(float(values, lhs)?) + f32::from_bits(float(values, rhs)?),
                entry.canonical_nan,
            )),
            Node::CanonicalizeF32Nan(source) => Value::F32(canonicalize(
                f32::from_bits(float(values, source)?),
                entry.canonical_nan,
            )),
            Node::Load { buffer, offset } => Value::F32(access(
                entry,
                storage,
                placements,
                buffer,
                index(values, offset)?,
                None,
            )?),
        };
        values[ordinal] = Some(value);
    }
    Ok(())
}

fn canonicalize(value: f32, canonical_nan: u32) -> u32 {
    if value.is_nan() {
        canonical_nan
    } else {
        value.to_bits()
    }
}

fn read(values: &[Option<Value>], ordinal: u32) -> Result<Value, String> {
    values
        .get(ordinal as usize)
        .copied()
        .flatten()
        .ok_or_else(|| format!("node {ordinal} was read before it was evaluated"))
}

fn index(values: &[Option<Value>], ordinal: u32) -> Result<u64, String> {
    match read(values, ordinal)? {
        Value::Index(value) => Ok(value),
        other => Err(format!(
            "node {ordinal} was read as an index and holds a {kind:?}",
            kind = other.kind(),
        )),
    }
}

fn float(values: &[Option<Value>], ordinal: u32) -> Result<u32, String> {
    match read(values, ordinal)? {
        Value::F32(bits) => Ok(bits),
        other => Err(format!(
            "node {ordinal} was read as an f32 and holds a {kind:?}",
            kind = other.kind(),
        )),
    }
}

/// Reads or writes one element through a declared buffer's routed placement.
fn access(
    entry: &GraphEntry,
    storage: &mut [Vec<u8>],
    placements: &[Placement],
    buffer: u32,
    element: u64,
    write: Option<u32>,
) -> Result<u32, String> {
    let declared = entry
        .buffers
        .get(buffer as usize)
        .ok_or_else(|| format!("buffer {buffer} is undeclared"))?;
    if element >= declared.element_count {
        return Err(format!(
            "element {element} is outside buffer {buffer}'s declared {count}",
            count = declared.element_count,
        ));
    }
    let placement = *placements
        .get(buffer as usize)
        .ok_or_else(|| format!("buffer {buffer} has no routed placement"))?;
    let at = element
        .checked_mul(F32_BYTES)
        .ok_or_else(|| "an element byte offset overflowed".to_owned())?;
    let end = at
        .checked_add(F32_BYTES)
        .ok_or_else(|| "an element span overflowed".to_owned())?;
    if end > placement.bytes {
        return Err(format!(
            "element {element} is outside the routed range of {bytes} byte(s)",
            bytes = placement.bytes,
        ));
    }
    let allocation = storage
        .get_mut(placement.allocation)
        .ok_or_else(|| format!("allocation {index} is unbound", index = placement.allocation))?;
    let start = usize::try_from(
        placement
            .offset
            .checked_add(at)
            .ok_or_else(|| "a placement offset overflowed".to_owned())?,
    )
    .map_err(|_| "a placement offset exceeds this host's address width".to_owned())?;
    let stop = start + 4;
    if stop > allocation.len() {
        return Err(format!(
            "allocation {index} holds {held} byte(s) and the access needs {stop}",
            index = placement.allocation,
            held = allocation.len(),
        ));
    }
    if let Some(bits) = write {
        allocation[start..stop].copy_from_slice(&bits.to_le_bytes());
        Ok(bits)
    } else {
        Ok(u32::from_le_bytes(
            <[u8; 4]>::try_from(&allocation[start..stop]).expect("a four-byte element"),
        ))
    }
}

/// One consumer's statically linked nodefold adapter. Nothing registers it.
pub(crate) struct NodefoldAdapter {
    profile: TargetProfileRef,
    dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
    input: Vec<u8>,
    behaviour: Behaviour,
    prepared: Vec<GraphEntry>,
    worker: Option<Worker>,
    storage: Vec<Vec<u8>>,
    placements: Vec<Vec<Placement>>,
    readback: Option<(usize, u64, usize)>,
    witnessed: usize,
}

impl NodefoldAdapter {
    /// Builds an adapter over the caller's input element bits.
    pub(crate) fn new(
        profile: TargetProfileRef,
        dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
        input_bits: &[u32],
        behaviour: Behaviour,
    ) -> Self {
        let mut input = Vec::with_capacity(input_bits.len() * 4);
        for bits in input_bits {
            input.extend_from_slice(&bits.to_le_bytes());
        }
        Self {
            profile,
            dtype_dispatch,
            input,
            behaviour,
            prepared: Vec::new(),
            worker: None,
            storage: Vec::new(),
            placements: Vec::new(),
            readback: None,
            witnessed: 0,
        }
    }
}

impl RuntimeAdapter for NodefoldAdapter {
    type Refusal = AdapterRefusal;
    type Failure = AdapterFailure;
    type Completion = Vec<u32>;

    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Self::Refusal> {
        let representation = match self.behaviour.binding {
            Binding::Observed => REPRESENTATION_KEY,
            // A representation this adapter cannot decode. The loader compares
            // what an adapter reports against what the artifact carries; an
            // adapter that could decide that comparison on its way to an answer
            // would be certifying its own eligibility.
            Binding::Preferred => "tiler.test.nodefold-graph-v2",
        };
        Ok(ExecutionEnvironment {
            target_profile: self.profile.clone(),
            backend: BackendKey::new(BACKEND_KEY).expect("a governed backend key"),
            representation: RepresentationKey::new(representation)
                .expect("a governed representation key"),
            dtype_dispatch: self.dtype_dispatch.clone(),
        })
    }

    /// Registers no target-environment descriptor schema.
    ///
    /// `Unsupported` filters every claimed `Plan` cell while leaving
    /// `Unclaimed` routes routable. There is no permissive default here: an
    /// adapter that cannot stand behind an accepted provider schema has nothing
    /// to attest, and saying so is the fail-closed answer.
    fn target_environment_support(&self) -> TargetEnvironmentSupport<'_> {
        TargetEnvironmentSupport::Unsupported
    }

    fn observe_target_environment(
        &mut self,
        _context: &LiveExecutionContext,
    ) -> TargetEnvironmentObservation {
        TargetEnvironmentObservation::Unavailable {
            reason: "the nodefold backend registers no target-environment descriptor schema"
                .to_owned(),
        }
    }

    /// Validates the carried bytes, which is this backend's own obligation.
    ///
    /// ADR 0090 item 8: the artifact envelope proves framing, digests, schema,
    /// canonical order, arena closure, and identity, and none of that says
    /// whether these bytes decode into something this backend can evaluate.
    fn validate_payload(
        &mut self,
        _context: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), Self::Refusal> {
        let graph: Graph = decode(entry.object()).map_err(AdapterRefusal::Payload)?;
        let prepared = graph
            .entry_for(entry.entry_symbol())
            .cloned()
            .ok_or_else(|| AdapterRefusal::UnmappedSymbol(entry.entry_symbol().to_owned()))?;
        self.prepared.push(prepared);
        Ok(())
    }

    fn observe_live_device(
        &mut self,
        _context: &LiveExecutionContext,
        _request: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        LiveDeviceObservation::Unrecognized
    }

    fn prepare_entries(
        &mut self,
        _context: &LiveExecutionContext,
        entries: &[RoutedEntry<'_>],
    ) -> Result<(), Self::Refusal> {
        if self.prepared.len() == entries.len() {
            Ok(())
        } else {
            Err(AdapterRefusal::Bookkeeping(format!(
                "{prepared} validated graph entr(y/ies) against a route of {routed}",
                prepared = self.prepared.len(),
                routed = entries.len(),
            )))
        }
    }

    /// Answers no prepared-entry property, because this backend mints none.
    ///
    /// Its workgroup capacity is a compile-time fact of a scalar execution
    /// model and is already in the target profile, so the compiler defers no
    /// predicate for it. An adapter that answered a property it had not
    /// measured would be inventing the fact the query exists to obtain.
    fn observe_prepared_entry(
        &mut self,
        _context: &LiveExecutionContext,
        _request: TargetPropertyRequest<'_>,
    ) -> PreparedEntryObservation {
        PreparedEntryObservation::Unrecognized
    }

    /// The last stage before the commit, and where the execution host is acquired.
    ///
    /// Acquiring here rather than in `allocate_dispatch` is deliberate: a host
    /// that cannot supply the worker must be a refusal a caller may still take
    /// a fallback from, and ADR 0051 places anything after the commit past that
    /// point. Nothing is acquired that a refusal here would strand.
    fn plan_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        preflight: &Preflight<'_>,
    ) -> Result<(), Self::Refusal> {
        for (position, entry) in preflight.entries().iter().enumerate() {
            if self.prepared.get(position).is_none() {
                return Err(AdapterRefusal::Bookkeeping(format!(
                    "entry {position} has no validated graph entry"
                )));
            }
            let requested = entry.launch().threads_per_workgroup();
            if requested > u64::from(WORKGROUP_THREADS) {
                return Err(AdapterRefusal::Capacity {
                    entry: position,
                    requested,
                    admitted: u64::from(WORKGROUP_THREADS),
                });
            }
            for binding in entry.bindings() {
                if !matches!(binding.binding().target(), BindingTarget::ProgramInput(_)) {
                    continue;
                }
                let needed = binding.accessible_offset() + binding.accessible_bytes();
                let supplied = u64::try_from(self.input.len()).expect("an input length fits u64");
                if needed > supplied {
                    return Err(AdapterRefusal::SuppliedInput {
                        entry: position,
                        slot: binding.slot(),
                        needed,
                        supplied,
                    });
                }
            }
        }
        self.worker = Some(Worker::acquire(self.behaviour.host).map_err(|unavailable| {
            AdapterRefusal::HostUnavailable(unavailable)
        })?);
        Ok(())
    }

    /// Acquires the routed storage. Past the commit, per ADR 0051.
    fn allocate_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<(), Self::Failure> {
        self.storage.clear();
        self.placements.clear();
        self.readback = None;
        for entry in routed.entries() {
            let mut placements = vec![
                Placement {
                    allocation: 0,
                    offset: 0,
                    bytes: 0,
                };
                entry.bindings().len()
            ];
            for binding in entry.bindings() {
                let span = binding.accessible_offset() + binding.accessible_bytes();
                let allocation = self.storage.len();
                match binding.binding().target() {
                    BindingTarget::ProgramInput(_) => self.storage.push(self.input.clone()),
                    BindingTarget::ProgramOutput(_) => {
                        self.storage.push(vec![
                            0_u8;
                            usize::try_from(span)
                                .expect("a routed range fits this host")
                        ]);
                        self.readback = Some((
                            allocation,
                            binding.accessible_offset(),
                            usize::try_from(binding.accessible_bytes() / F32_BYTES)
                                .expect("an element count fits this host"),
                        ));
                    }
                    BindingTarget::Internal => self.storage.push(vec![
                        0_u8;
                        usize::try_from(span)
                            .expect("a routed range fits this host")
                    ]),
                }
                // Placed by the *transport* the loader derived from this
                // backend's own mapping, never by the ABI slot number. A
                // backend whose mapping is not the identity and which indexed
                // by slot would bind the read where the write goes.
                let transport = binding.transport_slot() as usize;
                let Some(slot) = placements.get_mut(transport) else {
                    return Err(AdapterFailure::Evaluate(format!(
                        "transport {transport} is outside this entry's {count} binding(s)",
                        count = placements.len(),
                    )));
                };
                *slot = Placement {
                    allocation,
                    offset: binding.accessible_offset(),
                    bytes: binding.accessible_bytes(),
                };
            }
            self.placements.push(placements);
        }
        Ok(())
    }

    /// Submits every entry to the worker and returns only once each is terminal.
    fn dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<Self::Completion, Self::Failure> {
        let worker = self.worker.as_ref().ok_or(AdapterFailure::WorkerLost)?;
        let jobs = worker.jobs.as_ref().ok_or(AdapterFailure::WorkerLost)?;
        let mut submitted = 0_usize;
        for (position, entry) in routed.entries().iter().enumerate() {
            let launch = entry.launch();
            if launch.grid_threads() == 0 && launch.zero_work_skips_dispatch() {
                continue;
            }
            let graph_entry = self.prepared[position].clone();
            let placements = self.placements[position].clone();
            // The storage leaves this adapter here and does not come back until
            // the worker has finished writing through it.
            let storage = std::mem::take(&mut self.storage);
            jobs.send(Job {
                entry: position,
                graph_entry,
                invocations: launch.grid_threads(),
                evaluation: self.behaviour.evaluation,
                storage,
                placements,
            })
            .map_err(|_| AdapterFailure::WorkerLost)?;
            submitted += 1;
            if self.behaviour.lifetime == Lifetime::AwaitTerminalUse {
                let done = worker.done.recv().map_err(|_| AdapterFailure::WorkerLost)?;
                self.storage = done.storage;
                let witness = done.outcome.map_err(AdapterFailure::Evaluate)?;
                if witness.entry != position {
                    return Err(AdapterFailure::Evaluate(format!(
                        "the worker witnessed entry {witnessed} for submission {position}",
                        witnessed = witness.entry,
                    )));
                }
                self.witnessed += 1;
            }
        }
        if self.witnessed != submitted {
            return Err(AdapterFailure::TerminalUseUnwitnessed {
                submitted,
                witnessed: self.witnessed,
            });
        }
        let (allocation, offset, count) = self.readback.ok_or(AdapterFailure::NoProgramOutput)?;
        let storage = self
            .storage
            .get(allocation)
            .ok_or(AdapterFailure::NoProgramOutput)?;
        let start = usize::try_from(offset).expect("a routed offset fits this host");
        let mut bits = Vec::with_capacity(count);
        for element in 0..count {
            let at = start + element * 4;
            let run = storage
                .get(at..at + 4)
                .ok_or(AdapterFailure::NoProgramOutput)?;
            bits.push(u32::from_le_bytes(
                <[u8; 4]>::try_from(run).expect("a four-byte element"),
            ));
        }
        Ok(bits)
    }
}

/// Binds the ABI facts a route evaluates its formulas against.
///
/// The *literal* axes only. An interface axis may name a shape-environment
/// symbol whose value is the caller's bound buffer rather than anything the
/// artifact declares, so binding one here would state a fact this artifact does
/// not know; leaving it unbound makes every expression over it fail closed
/// instead of evaluating against an invented extent.
pub(crate) fn bind_facts(program: &DecodedProgram) -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    for input in program.inputs() {
        binder
            .bind_declared_extents(input.key(), input.extents())
            .expect("the declared interface binds");
    }
    binder.build()
}

/// How one routed run ended, in this fixture's own vocabulary.
#[derive(Debug)]
pub(crate) enum RouteEnd {
    /// The route completed and produced these bits.
    Completed(Vec<u32>),
    /// This host could not supply the execution resource.
    HostUnavailable(HostUnavailable),
    /// The loader refused, with its own classification carried whole.
    Refused(LoadRejection),
    /// The adapter refused before the commit.
    AdapterRefused(AdapterRefusal),
    /// The committed route failed.
    Failed(AdapterFailure),
}

/// Routes one encoded artifact through a nodefold adapter with the given behaviour.
pub(crate) fn route(
    bytes: &[u8],
    expected: &RecordedArtifactProgramIdentity,
    profile: TargetProfileRef,
    dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
    input_bits: &[u32],
    behaviour: Behaviour,
) -> RouteEnd {
    let mut program = match DecodedProgram::decode(bytes, SOLE_DELIVERY) {
        Ok(program) => program,
        Err(rejection) => return RouteEnd::Refused(rejection),
    };
    let facts = bind_facts(&program);
    let mut adapter = NodefoldAdapter::new(profile, dtype_dispatch, input_bits, behaviour);
    match route_with_adapter(&mut program, &mut adapter, expected, &facts) {
        Ok(bits) => RouteEnd::Completed(bits),
        Err(AdapterRouteFailure::Load(rejection)) => RouteEnd::Refused(rejection),
        Err(
            AdapterRouteFailure::Context(refusal)
            | AdapterRouteFailure::Payload { refusal, .. }
            | AdapterRouteFailure::Preparation(refusal)
            | AdapterRouteFailure::Plan(refusal),
        ) => match refusal {
            AdapterRefusal::HostUnavailable(unavailable) => {
                RouteEnd::HostUnavailable(unavailable)
            }
            other => RouteEnd::AdapterRefused(other),
        },
        Err(AdapterRouteFailure::Allocation(failure) | AdapterRouteFailure::Dispatch(failure)) => {
            RouteEnd::Failed(failure)
        }
        Err(other) => RouteEnd::AdapterRefused(AdapterRefusal::Bookkeeping(format!("{other:?}"))),
    }
}

/// Applies the caller's host policy to one route's end.
///
/// The whole of the policy, and it is here rather than in the adapter: the
/// adapter reports what the host could not supply, and the caller decides what
/// that means to its run. Nothing below reads an environment variable, and a
/// [`RouteEnd::HostUnavailable`] under [`HostPolicy::Require`] is an error
/// rather than an outcome — which is the difference between a host that may
/// skip and a host that may not.
pub(crate) fn apply_policy(policy: HostPolicy, end: RouteEnd) -> Result<ExecutionOutcome, String> {
    match end {
        RouteEnd::Completed(bits) => Ok(ExecutionOutcome::Completed(bits)),
        RouteEnd::HostUnavailable(unavailable) => match policy {
            HostPolicy::Require => Err(format!(
                "this caller required the execution host and {unavailable}"
            )),
            HostPolicy::Report => Ok(ExecutionOutcome::Unavailable(unavailable)),
        },
        RouteEnd::Refused(rejection) => Err(format!("the loader refused: {rejection}")),
        RouteEnd::AdapterRefused(refusal) => Err(format!("the adapter refused: {refusal}")),
        RouteEnd::Failed(failure) => Err(format!("the committed route failed: {failure}")),
    }
}

/// Where a routed result first disagrees with the oracle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Disagreement {
    /// Index of the first disagreeing element.
    pub(crate) index: usize,
    /// Bits the route produced.
    pub(crate) produced: u32,
    /// Bits `tiler-reference` requires.
    pub(crate) required: u32,
}

impl fmt::Display for Disagreement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "element {index} is 0x{produced:08x} and tiler-reference requires 0x{required:08x}",
            index = self.index,
            produced = self.produced,
            required = self.required,
        )
    }
}

/// Compares routed bits against the oracle's, element by element.
///
/// Takes bits rather than an [`ExecutionOutcome`] on purpose: reaching this
/// function requires having destructured a completion, so an unavailable host
/// has no path into a comparison at all.
pub(crate) fn agrees_with_reference(
    produced: &[u32],
    reference: &[u32],
) -> Result<usize, Disagreement> {
    if produced.len() != reference.len() {
        return Err(Disagreement {
            index: produced.len().min(reference.len()),
            produced: u32::try_from(produced.len()).unwrap_or(u32::MAX),
            required: u32::try_from(reference.len()).unwrap_or(u32::MAX),
        });
    }
    for (index, (produced, required)) in produced.iter().zip(reference).enumerate() {
        if produced != required {
            return Err(Disagreement {
                index,
                produced: *produced,
                required: *required,
            });
        }
    }
    Ok(produced.len())
}
