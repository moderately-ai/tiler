//! Process-level fault harness for Tiler's proposed expansion cache protocol.
//!
//! This is deliberately a dependency-free spike, not production cache code.
//! Build and exercise it with:
//!
//! ```text
//! rustc --edition 2021 spikes/cache/cache_harness.rs -o /tmp/tiler-cache-harness
//! /tmp/tiler-cache-harness selftest
//! /tmp/tiler-cache-harness selftest --stress 32
//! ```

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAGIC: &[u8; 8] = b"TLRCCH01";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 92;
const MAX_PAYLOAD: usize = 16 * 1024 * 1024;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Durability {
    ProcessCrash,
    Fsync,
}

impl Durability {
    fn parse(value: &str) -> AnyResult<Self> {
        match value {
            "process" => Ok(Self::ProcessCrash),
            "fsync" => Ok(Self::Fsync),
            _ => Err(format!("unknown durability mode {value:?}").into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Outcome {
    Hit,
    Published,
    Uncached,
}

impl Outcome {
    fn label(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Published => "published",
            Self::Uncached => "uncached",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    AfterLock,
    AfterRecheck,
    AfterTempCreate,
    MidWrite,
    AfterWrite,
    AfterTempValidation,
    AfterFileSync,
    AfterRename,
    AfterDirectorySync,
}

impl Phase {
    const KILL_POINTS: [Self; 9] = [
        Self::AfterLock,
        Self::AfterRecheck,
        Self::AfterTempCreate,
        Self::MidWrite,
        Self::AfterWrite,
        Self::AfterTempValidation,
        Self::AfterFileSync,
        Self::AfterRename,
        Self::AfterDirectorySync,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::AfterLock => "after-lock",
            Self::AfterRecheck => "after-recheck",
            Self::AfterTempCreate => "after-temp-create",
            Self::MidWrite => "mid-write",
            Self::AfterWrite => "after-write",
            Self::AfterTempValidation => "after-temp-validation",
            Self::AfterFileSync => "after-file-sync",
            Self::AfterRename => "after-rename",
            Self::AfterDirectorySync => "after-directory-sync",
        }
    }

    fn parse(value: &str) -> AnyResult<Self> {
        Self::KILL_POINTS
            .iter()
            .copied()
            .find(|phase| phase.label() == value)
            .ok_or_else(|| format!("unknown phase {value:?}").into())
    }
}

#[derive(Clone, Debug)]
struct Fault {
    pause_at: Option<Phase>,
    marker: PathBuf,
}

impl Fault {
    fn none() -> Self {
        Self {
            pause_at: None,
            marker: PathBuf::new(),
        }
    }

    fn reach(&self, phase: Phase) -> io::Result<()> {
        if self.pause_at == Some(phase) {
            fs::write(&self.marker, phase.label())?;
            // The parent sends SIGKILL/TerminateProcess. This process must not
            // unwind: the test is specifically that the OS releases resources.
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Cache {
    root: PathBuf,
}

#[derive(Debug)]
struct Paths {
    entry: PathBuf,
    lock: PathBuf,
    temp_dir: PathBuf,
}

impl Cache {
    fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn paths(&self, key: &str) -> AnyResult<Paths> {
        validate_key_text(key)?;
        let shard = &key[..2];
        Ok(Paths {
            entry: self
                .root
                .join("v1/entries")
                .join(shard)
                .join(format!("{key}.bundle")),
            lock: self
                .root
                .join("v1/locks")
                .join(shard)
                .join(format!("{key}.lock")),
            temp_dir: self.root.join("v1/tmp").join(shard),
        })
    }

    fn read(&self, key: &str) -> AnyResult<Option<Vec<u8>>> {
        let path = self.paths(key)?.entry;
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        match decode_entry(&bytes, key) {
            Ok(payload) => Ok(Some(payload.to_vec())),
            Err(_) => Ok(None),
        }
    }

    fn get_or_build(
        &self,
        key: &str,
        payload: &[u8],
        compile_log: &Path,
        durability: Durability,
        fault: &Fault,
    ) -> AnyResult<Outcome> {
        match self.read(key) {
            Ok(Some(_)) => return Ok(Outcome::Hit),
            Ok(None) => {}
            Err(cache_error) => {
                return compile_uncached(key, payload, compile_log, &cache_error.to_string())
            }
        }

        let paths = self.paths(key)?;
        if let Err(cache_error) = prepare_directories(&paths) {
            return compile_uncached(key, payload, compile_log, &cache_error.to_string());
        }

        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&paths.lock)?;
        lock_file.lock()?;
        fault.reach(Phase::AfterLock)?;

        // The recheck is essential: a process may have published while this
        // process waited for the advisory lock.
        if self.read(key)?.is_some() {
            return Ok(Outcome::Hit);
        }
        fault.reach(Phase::AfterRecheck)?;

        record_compile(compile_log, key)?;
        let encoded = encode_entry(key, payload)?;
        let temp_path = unique_temp_path(&paths.temp_dir, key);
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        fault.reach(Phase::AfterTempCreate)?;

        let middle = encoded.len() / 2;
        temp.write_all(&encoded[..middle])?;
        fault.reach(Phase::MidWrite)?;
        temp.write_all(&encoded[middle..])?;
        fault.reach(Phase::AfterWrite)?;

        // Validate through a separate descriptor before publication. The final
        // path is never populated by a partially written or unchecked bundle.
        let mut verify = File::open(&temp_path)?;
        let mut verify_bytes = Vec::new();
        verify.read_to_end(&mut verify_bytes)?;
        decode_entry(&verify_bytes, key)
            .map_err(|error| format!("temporary entry validation failed: {error}"))?;
        fault.reach(Phase::AfterTempValidation)?;

        if durability == Durability::Fsync {
            temp.sync_all()?;
        }
        fault.reach(Phase::AfterFileSync)?;
        drop(verify);
        drop(temp);

        // Temp and final are deliberately under the same cache root. rename is
        // the only publication operation; replacement of a corrupt old entry is
        // atomic on the Unix/Darwin contract exercised by this spike.
        fs::rename(&temp_path, &paths.entry)?;
        fault.reach(Phase::AfterRename)?;
        if durability == Durability::Fsync {
            File::open(paths.entry.parent().unwrap())?.sync_all()?;
        }
        fault.reach(Phase::AfterDirectorySync)?;
        Ok(Outcome::Published)
    }

    fn evict_key(&self, key: &str) -> AnyResult<()> {
        let paths = self.paths(key)?;
        prepare_directories(&paths)?;
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&paths.lock)?;
        lock_file.lock()?;
        match fs::remove_file(&paths.entry) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        // Stable lock paths are intentionally retained. Unlinking a lock file
        // can split contenders between old and newly-created inodes.
        Ok(())
    }
}

fn compile_uncached(
    key: &str,
    payload: &[u8],
    compile_log: &Path,
    cache_error: &str,
) -> AnyResult<Outcome> {
    // Cache availability is not output correctness. A macro expansion can
    // still compile and embed a validated artifact without publishing.
    record_compile(compile_log, key)?;
    let bytes = encode_entry(key, payload)?;
    decode_entry(&bytes, key).map_err(|error| format!("uncached artifact invalid: {error}"))?;
    eprintln!("cache unavailable, compiling without publication: {cache_error}");
    Ok(Outcome::Uncached)
}

fn prepare_directories(paths: &Paths) -> io::Result<()> {
    fs::create_dir_all(paths.entry.parent().unwrap())?;
    fs::create_dir_all(paths.lock.parent().unwrap())?;
    fs::create_dir_all(&paths.temp_dir)?;
    Ok(())
}

fn unique_temp_path(directory: &Path, key: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    directory.join(format!("{key}.{}.{}.tmp", process::id(), nonce))
}

fn record_compile(path: &Path, key: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{key}")
}

fn encode_entry(key: &str, payload: &[u8]) -> AnyResult<Vec<u8>> {
    let key_bytes = parse_hex_digest(key)?;
    if payload.len() > MAX_PAYLOAD {
        return Err("payload exceeds spike bound".into());
    }
    let payload_digest = domain_hash(b"tiler.cache.payload.v1\0", payload);
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&((HEADER_LEN + payload.len()) as u64).to_le_bytes());
    out.extend_from_slice(&key_bytes);
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(&payload_digest);
    debug_assert_eq!(out.len(), HEADER_LEN);
    out.extend_from_slice(payload);
    Ok(out)
}

fn decode_entry<'a>(bytes: &'a [u8], requested_key: &str) -> Result<&'a [u8], &'static str> {
    if bytes.len() < HEADER_LEN {
        return Err("truncated header");
    }
    if &bytes[..8] != MAGIC {
        return Err("bad magic");
    }
    if u16::from_le_bytes(bytes[8..10].try_into().unwrap()) != VERSION {
        return Err("unsupported version");
    }
    if bytes[10..12] != [0, 0] {
        return Err("nonzero reserved field");
    }
    let total = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
    if total != bytes.len() as u64 {
        return Err("total length mismatch");
    }
    let requested = parse_hex_digest(requested_key).map_err(|_| "invalid requested key")?;
    if bytes[20..52] != requested {
        return Err("embedded key mismatch");
    }
    let payload_len = u64::from_le_bytes(bytes[52..60].try_into().unwrap());
    if payload_len > MAX_PAYLOAD as u64 || payload_len != (bytes.len() - HEADER_LEN) as u64 {
        return Err("payload length mismatch");
    }
    let payload = &bytes[HEADER_LEN..];
    let actual = domain_hash(b"tiler.cache.payload.v1\0", payload);
    if bytes[60..92] != actual {
        return Err("payload digest mismatch");
    }
    Ok(payload)
}

fn cache_key(label: &str) -> String {
    to_hex(&domain_hash(b"tiler.cache.key.v1\0", label.as_bytes()))
}

fn validate_key_text(value: &str) -> AnyResult<()> {
    parse_hex_digest(value).map(|_| ())
}

fn parse_hex_digest(value: &str) -> AnyResult<[u8; 32]> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("cache key must be 64 hexadecimal characters".into());
    }
    let mut out = [0u8; 32];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(out)
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut out, "{byte:02x}").unwrap();
    }
    out
}

fn domain_hash(domain: &[u8], value: &[u8]) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(domain.len() + value.len());
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(value);
    sha256(&bytes)
}

fn worker(args: &[String]) -> AnyResult<()> {
    let root = PathBuf::from(required(args, "--root")?);
    let key = required(args, "--key")?;
    let payload = required(args, "--payload")?.as_bytes();
    let compile_log = PathBuf::from(required(args, "--compile-log")?);
    let result = PathBuf::from(required(args, "--result")?);
    let durability = Durability::parse(required(args, "--durability")?)?;
    if let Some(go) = optional(args, "--go") {
        wait_for(Path::new(go), Duration::from_secs(20))?;
    }
    let fault = match optional(args, "--pause") {
        Some(phase) => Fault {
            pause_at: Some(Phase::parse(phase)?),
            marker: PathBuf::from(required(args, "--marker")?),
        },
        None => Fault::none(),
    };
    let outcome = Cache::new(root).get_or_build(key, payload, &compile_log, durability, &fault)?;
    fs::write(result, outcome.label())?;
    Ok(())
}

fn held_reader(args: &[String]) -> AnyResult<()> {
    let cache = Cache::new(PathBuf::from(required(args, "--root")?));
    let key = required(args, "--key")?;
    let marker = PathBuf::from(required(args, "--marker")?);
    let resume = PathBuf::from(required(args, "--resume")?);
    let result = PathBuf::from(required(args, "--result")?);
    let path = cache.paths(key)?.entry;
    let mut file = File::open(path)?;
    fs::write(marker, "opened")?;
    wait_for(&resume, Duration::from_secs(20))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    decode_entry(&bytes, key).map_err(|error| format!("held reader failed: {error}"))?;
    fs::write(result, "valid")?;
    Ok(())
}

fn required<'a>(args: &'a [String], flag: &str) -> AnyResult<&'a str> {
    optional(args, flag).ok_or_else(|| format!("missing {flag}").into())
}

fn optional<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|argument| argument == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn wait_for(path: &Path, timeout: Duration) -> AnyResult<()> {
    let deadline = Instant::now() + timeout;
    while !path.exists() {
        if Instant::now() >= deadline {
            return Err(format!("timed out waiting for {}", path.display()).into());
        }
        thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}

fn spawn_worker(
    root: &Path,
    key: &str,
    payload: &str,
    compile_log: &Path,
    result: &Path,
    go: Option<&Path>,
    pause: Option<(Phase, &Path)>,
    durability: Durability,
) -> AnyResult<Child> {
    let mut command = Command::new(env::current_exe()?);
    command
        .arg("worker")
        .arg("--root")
        .arg(root)
        .arg("--key")
        .arg(key)
        .arg("--payload")
        .arg(payload)
        .arg("--compile-log")
        .arg(compile_log)
        .arg("--result")
        .arg(result)
        .arg("--durability")
        .arg(match durability {
            Durability::ProcessCrash => "process",
            Durability::Fsync => "fsync",
        });
    if let Some(go) = go {
        command.arg("--go").arg(go);
    }
    if let Some((phase, marker)) = pause {
        command
            .arg("--pause")
            .arg(phase.label())
            .arg("--marker")
            .arg(marker);
    }
    Ok(command.spawn()?)
}

fn wait_success(child: &mut Child) -> AnyResult<ExitStatus> {
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("child exited with {status}").into());
    }
    Ok(status)
}

fn line_count(path: &Path) -> AnyResult<usize> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text.lines().count()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error.into()),
    }
}

fn fresh_case(root: &Path, name: &str) -> AnyResult<PathBuf> {
    let path = root.join(name);
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn test_identical(root: &Path, process_count: usize) -> AnyResult<()> {
    let case = fresh_case(root, "identical")?;
    let cache_root = case.join("cache");
    let control = case.join("control");
    fs::create_dir_all(&control)?;
    let compile_log = control.join("compiles");
    let go = control.join("go");
    let key = cache_key("identical");
    let mut children = Vec::new();
    for index in 0..process_count {
        children.push(spawn_worker(
            &cache_root,
            &key,
            "same-payload",
            &compile_log,
            &control.join(format!("result-{index}")),
            Some(&go),
            None,
            Durability::ProcessCrash,
        )?);
    }
    fs::write(&go, "go")?;
    for child in &mut children {
        wait_success(child)?;
    }
    assert_eq!(
        line_count(&compile_log)?,
        1,
        "identical key compiled more than once"
    );
    assert_eq!(
        Cache::new(cache_root).read(&key)?.as_deref(),
        Some(&b"same-payload"[..])
    );
    Ok(())
}

fn test_distinct(root: &Path, process_count: usize) -> AnyResult<()> {
    let case = fresh_case(root, "distinct")?;
    let cache_root = case.join("cache");
    let control = case.join("control");
    fs::create_dir_all(&control)?;
    let compile_log = control.join("compiles");
    let go = control.join("go");
    let mut children = Vec::new();
    for index in 0..process_count {
        children.push(spawn_worker(
            &cache_root,
            &cache_key(&format!("distinct-{index}")),
            &format!("payload-{index}"),
            &compile_log,
            &control.join(format!("result-{index}")),
            Some(&go),
            None,
            Durability::ProcessCrash,
        )?);
    }
    fs::write(&go, "go")?;
    for child in &mut children {
        wait_success(child)?;
    }
    assert_eq!(line_count(&compile_log)?, process_count);
    Ok(())
}

fn test_killed_writers(root: &Path) -> AnyResult<()> {
    for phase in Phase::KILL_POINTS {
        let case = fresh_case(root, &format!("kill-{}", phase.label()))?;
        let cache_root = case.join("cache");
        let compile_log = case.join("compiles");
        let marker = case.join("paused");
        let key = cache_key(phase.label());
        let mut killed = spawn_worker(
            &cache_root,
            &key,
            "recoverable",
            &compile_log,
            &case.join("killed-result"),
            None,
            Some((phase, &marker)),
            Durability::Fsync,
        )?;
        wait_for(&marker, Duration::from_secs(20))?;
        killed.kill()?;
        let _ = killed.wait()?;

        let mut recovery = spawn_worker(
            &cache_root,
            &key,
            "recoverable",
            &compile_log,
            &case.join("recovery-result"),
            None,
            None,
            Durability::Fsync,
        )?;
        wait_success(&mut recovery)?;
        assert_eq!(
            Cache::new(cache_root).read(&key)?.as_deref(),
            Some(&b"recoverable"[..])
        );
        let expected_compiles = if matches!(
            phase,
            Phase::AfterLock | Phase::AfterRecheck | Phase::AfterRename | Phase::AfterDirectorySync
        ) {
            1
        } else {
            2
        };
        assert_eq!(
            line_count(&compile_log)?,
            expected_compiles,
            "phase {}",
            phase.label()
        );
    }
    Ok(())
}

fn test_corruption(root: &Path) -> AnyResult<()> {
    for (name, corrupt) in [
        ("truncated", vec![0u8; 17]),
        ("bad-digest", {
            let key = cache_key("bad-digest");
            let mut bytes = encode_entry(&key, b"old")?;
            *bytes.last_mut().unwrap() ^= 1;
            bytes
        }),
    ] {
        let case = fresh_case(root, &format!("corrupt-{name}"))?;
        let cache_root = case.join("cache");
        let cache = Cache::new(cache_root.clone());
        let key = cache_key(name);
        let paths = cache.paths(&key)?;
        prepare_directories(&paths)?;
        fs::write(&paths.entry, corrupt)?;
        let compile_log = case.join("compiles");
        let mut child = spawn_worker(
            &cache_root,
            &key,
            "rebuilt",
            &compile_log,
            &case.join("result"),
            None,
            None,
            Durability::ProcessCrash,
        )?;
        wait_success(&mut child)?;
        assert_eq!(line_count(&compile_log)?, 1);
        assert_eq!(cache.read(&key)?.as_deref(), Some(&b"rebuilt"[..]));
    }
    Ok(())
}

fn test_deletion(root: &Path) -> AnyResult<()> {
    let case = fresh_case(root, "deletion")?;
    let cache_root = case.join("cache");
    let cache = Cache::new(cache_root.clone());
    let key = cache_key("deletion");
    let compile_log = case.join("compiles");
    for generation in 0..3 {
        let mut child = spawn_worker(
            &cache_root,
            &key,
            "replaceable",
            &compile_log,
            &case.join(format!("result-{generation}")),
            None,
            None,
            Durability::ProcessCrash,
        )?;
        wait_success(&mut child)?;
        if generation == 0 {
            fs::remove_file(cache.paths(&key)?.entry)?;
        } else if generation == 1 {
            fs::remove_dir_all(&cache_root)?;
        }
    }
    assert_eq!(line_count(&compile_log)?, 3);
    assert_eq!(cache.read(&key)?.as_deref(), Some(&b"replaceable"[..]));
    Ok(())
}

fn test_active_whole_cache_deletion(root: &Path) -> AnyResult<()> {
    let case = fresh_case(root, "active-whole-cache-deletion")?;
    let cache_root = case.join("cache");
    let key = cache_key("active-whole-cache-deletion");
    let compile_log = case.join("compiles");
    let marker = case.join("first-paused");
    let mut first = spawn_worker(
        &cache_root,
        &key,
        "same-correct-output",
        &compile_log,
        &case.join("first-result"),
        None,
        Some((Phase::AfterTempCreate, &marker)),
        Durability::ProcessCrash,
    )?;
    wait_for(&marker, Duration::from_secs(20))?;

    // This models an external recursive deletion, not coordinated GC. It
    // unlinks the stable lock inode, so duplicate work suppression is lost.
    // Correctness still comes from validation plus immutable publication.
    fs::remove_dir_all(&cache_root)?;
    let mut second = spawn_worker(
        &cache_root,
        &key,
        "same-correct-output",
        &compile_log,
        &case.join("second-result"),
        None,
        None,
        Durability::ProcessCrash,
    )?;
    wait_success(&mut second)?;
    first.kill()?;
    let _ = first.wait()?;

    assert_eq!(line_count(&compile_log)?, 2);
    assert_eq!(
        Cache::new(cache_root).read(&key)?.as_deref(),
        Some(&b"same-correct-output"[..])
    );
    Ok(())
}

fn test_unwritable(root: &Path) -> AnyResult<()> {
    let case = fresh_case(root, "unwritable")?;
    let unusable_root = case.join("not-a-directory");
    fs::write(&unusable_root, "occupied by a regular file")?;
    let compile_log = case.join("compiles");
    let result = case.join("result");
    let mut child = spawn_worker(
        &unusable_root,
        &cache_key("unwritable"),
        "still-correct",
        &compile_log,
        &result,
        None,
        None,
        Durability::ProcessCrash,
    )?;
    wait_success(&mut child)?;
    assert_eq!(fs::read_to_string(result)?, "uncached");
    assert_eq!(line_count(&compile_log)?, 1);
    Ok(())
}

fn test_eviction_reader(root: &Path) -> AnyResult<()> {
    let case = fresh_case(root, "eviction-reader")?;
    let cache_root = case.join("cache");
    let key = cache_key("eviction-reader");
    let compile_log = case.join("compiles");
    let mut writer = spawn_worker(
        &cache_root,
        &key,
        "reader-keeps-open-inode",
        &compile_log,
        &case.join("writer-result"),
        None,
        None,
        Durability::ProcessCrash,
    )?;
    wait_success(&mut writer)?;

    let marker = case.join("reader-opened");
    let resume = case.join("reader-resume");
    let result = case.join("reader-result");
    let mut reader = Command::new(env::current_exe()?)
        .arg("held-reader")
        .arg("--root")
        .arg(&cache_root)
        .arg("--key")
        .arg(&key)
        .arg("--marker")
        .arg(&marker)
        .arg("--resume")
        .arg(&resume)
        .arg("--result")
        .arg(&result)
        .spawn()?;
    wait_for(&marker, Duration::from_secs(20))?;
    Cache::new(cache_root.clone()).evict_key(&key)?;
    fs::write(&resume, "resume")?;
    wait_success(&mut reader)?;
    assert_eq!(fs::read_to_string(result)?, "valid");
    assert!(Cache::new(cache_root).read(&key)?.is_none());
    Ok(())
}

fn selftest(args: &[String]) -> AnyResult<()> {
    let stress = optional(args, "--stress")
        .map(str::parse)
        .transpose()?
        .unwrap_or(12usize);
    if stress == 0 || stress > 256 {
        return Err("--stress must be in 1..=256".into());
    }
    let root = env::temp_dir().join(format!(
        "tiler-cache-harness-{}-{}",
        process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir_all(&root)?;
    let started = Instant::now();

    assert_eq!(
        to_hex(&sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    test_identical(&root, stress)?;
    println!("ok concurrent-identical processes={stress}");
    test_distinct(&root, stress)?;
    println!("ok concurrent-distinct processes={stress}");
    test_killed_writers(&root)?;
    println!("ok killed-writers phases={}", Phase::KILL_POINTS.len());
    test_corruption(&root)?;
    println!("ok corrupt-and-truncated-recovery");
    test_deletion(&root)?;
    println!("ok entry-and-whole-cache-deletion");
    test_active_whole_cache_deletion(&root)?;
    println!("ok active-whole-cache-deletion-remains-correct");
    test_unwritable(&root)?;
    println!("ok unavailable-root-uncached-fallback");
    test_eviction_reader(&root)?;
    println!("ok eviction-racing-open-reader");

    fs::remove_dir_all(&root)?;
    println!(
        "all cache harness cases passed in {:.2?}",
        started.elapsed()
    );
    Ok(())
}

fn usage() {
    eprintln!("usage: cache_harness selftest [--stress N]");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("selftest") => selftest(&args[1..]),
        Some("worker") => worker(&args[1..]),
        Some("held-reader") => held_reader(&args[1..]),
        _ => {
            usage();
            process::exit(2);
        }
    };
    if let Err(error) = result {
        eprintln!("cache harness error: {error}");
        process::exit(1);
    }
}

// Dependency-free SHA-256 for an exact-byte integrity check in the spike.
fn sha256(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut data = input.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (index, word) in chunk.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes(word.try_into().unwrap());
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut out = [0u8; 32];
    for (chunk, value) in out.chunks_exact_mut(4).zip(h) {
        chunk.copy_from_slice(&value.to_be_bytes());
    }
    out
}
