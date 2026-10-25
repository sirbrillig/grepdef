//! An optional on-disk index of which files define which symbols
//!
//! The index never gives a wrong answer because it is only used to skip files that have not changed
//! since the index was built. Every search still walks the file tree and compares each file's size
//! and timestamps against the index; any file that is new or changed is searched directly, as is
//! every unchanged file that the index lists for the query. Files the index lists are still
//! searched normally, so a stale entry can only cost time, never produce a result.
//!
//! After a search finds that files have changed (or that there is no index), the index is
//! refreshed: unchanged files keep their indexed symbols and only changed or new files are read.
//! When running as the CLI this happens in a separate low-priority process so that the search can
//! return immediately.
//!
//! Only one refresh should run at a time, but a stuck refresh must never block future ones, so
//! there is no lock. Instead, each refresh writes its id to an owner file and periodically checks
//! in by reading the file and rewriting it. A refresh that finds another id in the file has been
//! replaced and stops without writing anything. A search only starts a refresh if the owner file
//! has not been checked in recently, so a refresh that has crashed or hung is simply replaced.

use super::{debug, file_type, query_regex, Config};
use ignore::{DirEntry, WalkState};
use memchr::{memchr, memmem, memrchr};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{self, Duration, SystemTime, UNIX_EPOCH};

/// The first line of every index file; change the version when the format or the meaning of its
/// contents changes (eg: if the definition patterns change) so that old indexes are rebuilt.
const HEADER: &[u8] = b"grepdef-index 1\n";

/// Files changed within this many seconds of the index being built are left out of the index
///
/// Some filesystems have coarse timestamps, so a file changed just after being indexed could keep
/// the same timestamps as when it was read. Leaving recently changed files out of the index means
/// they will always be searched directly. Two seconds covers the coarsest common filesystem (FAT).
const RACY_WINDOW_NANOS: i128 = 2_000_000_000;

/// A refresh checks in with its owner file after visiting this many entries
const CHECK_IN_INTERVAL: usize = 256;

/// A refresh that has not checked in for this long is assumed to have crashed or hung
const CHECK_IN_TIMEOUT: Duration = Duration::from_secs(30);

/// A background refresh exits after this long no matter what, in case it is stuck
const BACKGROUND_REFRESH_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// The size and timestamps of a file, used to detect if it has changed since it was indexed
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct FileStamp {
    size: u64,
    mtime: (i64, i64),
    ctime: (i64, i64),
}

impl FileStamp {
    /// Note that ctime cannot be set by tools that preserve modification times (eg: `rsync -a`),
    /// so it catches changes that mtime alone would miss.
    #[cfg(unix)]
    fn from_metadata(metadata: &fs::Metadata) -> FileStamp {
        use std::os::unix::fs::MetadataExt;
        FileStamp {
            size: metadata.len(),
            mtime: (metadata.mtime(), metadata.mtime_nsec()),
            ctime: (metadata.ctime(), metadata.ctime_nsec()),
        }
    }

    #[cfg(not(unix))]
    fn from_metadata(metadata: &fs::Metadata) -> FileStamp {
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map_or((0, 0), |d| (d.as_secs() as i64, d.subsec_nanos() as i64));
        FileStamp {
            size: metadata.len(),
            mtime,
            ctime: (0, 0),
        }
    }

    fn newest_nanos(&self) -> i128 {
        let to_nanos = |(secs, nanos): (i64, i64)| secs as i128 * 1_000_000_000 + nanos as i128;
        to_nanos(self.mtime).max(to_nanos(self.ctime))
    }
}

/// Return true if a walked entry can be stored in the index
///
/// Only regular files can be trusted by their stamp; others (eg: symlinks, whose stamp is the
/// link's and not the target's) are never indexed so they are always searched.
fn is_indexable(path: &str, entry: &DirEntry) -> bool {
    entry.file_type().is_some_and(|ft| ft.is_file()) && !path.contains('\n')
}

/// An index loaded for a single query
pub struct Index {
    /// Every indexed file, by path, with its id and the stamp it had when indexed
    files: HashMap<String, (u32, FileStamp)>,

    /// The ids of the files which the index lists as defining the query
    candidates: HashSet<u32>,

    /// True if the search found a file that is new or has changed since the index was built
    found_changed_file: AtomicBool,

    /// The number of indexed files that the search found unchanged
    unchanged_file_count: AtomicUsize,
}

impl Index {
    /// Return true if a file found while walking needs to be searched
    pub fn should_search(&self, path: &str, entry: &DirEntry) -> bool {
        if !is_indexable(path, entry) {
            return true;
        }
        let Some((id, stamp)) = self.files.get(path) else {
            self.found_changed_file.store(true, Ordering::Relaxed);
            return true;
        };
        match entry.metadata() {
            Ok(metadata) if FileStamp::from_metadata(&metadata) == *stamp => {
                self.unchanged_file_count.fetch_add(1, Ordering::Relaxed);
                self.candidates.contains(id)
            }
            _ => {
                self.found_changed_file.store(true, Ordering::Relaxed);
                true
            }
        }
    }

    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    /// Return true if the search found that the index is out of date
    ///
    /// If the search walked every file, then any indexed file it did not find unchanged must have
    /// been deleted (or changed). If the search stopped early (eg: because of a limit), only the
    /// files it found can be considered.
    fn is_stale(&self, walked_all_files: bool) -> bool {
        self.found_changed_file.load(Ordering::Relaxed)
            || (walked_all_files
                && self.unchanged_file_count.load(Ordering::Relaxed) < self.files.len())
    }
}

/// Load the index for a search
///
/// Returns None if the index cannot be used for this search, in which case every file should be
/// searched.
pub fn prepare(config: &Config) -> Option<Index> {
    if !query_regex::is_plain_symbol(&config.query) {
        debug(config, "Query is not a plain symbol; not using index");
        return None;
    }
    let path = get_index_path(config)?;
    let start = time::Instant::now();
    match load(&path, &config.query) {
        Ok(index) => {
            debug(
                config,
                format!(
                    "Loaded index {} in {} ms",
                    path.display(),
                    start.elapsed().as_millis()
                )
                .as_str(),
            );
            Some(index)
        }
        Err(err) => {
            debug(
                config,
                format!("No usable index at {}: {}", path.display(), err).as_str(),
            );
            None
        }
    }
}

/// Refresh the index after a search if it is missing or out of date
///
/// If [Config::index_refresh_command] is set, this starts the refresh in a separate process and
/// returns immediately; otherwise it refreshes the index before returning.
pub fn refresh_if_needed(config: &Config, index: Option<&Index>, walked_all_files: bool) {
    if !query_regex::is_plain_symbol(&config.query) {
        return;
    }
    if index.is_some_and(|index| !index.is_stale(walked_all_files)) {
        return;
    }
    let Some(dest) = get_index_path(config) else {
        return;
    };
    if is_refresh_running(&dest) {
        debug(
            config,
            "Index is out of date but is already being refreshed",
        );
        return;
    }
    match &config.index_refresh_command {
        Some(command) => {
            debug(config, "Index is out of date; refreshing in background");
            if let Err(err) = spawn_background_refresh(config, command) {
                debug(config, format!("Could not start refresh: {}", err).as_str());
            }
        }
        None => {
            debug(config, "Index is out of date; refreshing");
            let start = time::Instant::now();
            let result = Owner::claim(&dest).and_then(|owner| {
                let result = refresh(config, &dest, &owner);
                owner.release();
                result
            });
            match result {
                Ok(()) => debug(
                    config,
                    format!("Refreshed index in {} ms", start.elapsed().as_millis()).as_str(),
                ),
                Err(err) => debug(config, format!("Could not refresh index: {}", err).as_str()),
            }
        }
    }
}

/// Start a process that runs [run_background_refresh] with the same search config
fn spawn_background_refresh(config: &Config, command_path: &Path) -> io::Result<()> {
    let mut command = Command::new(command_path);
    command
        .arg("--background-index-refresh")
        .arg("--index")
        .arg("--type")
        .arg(config.file_type.to_string())
        .arg("--threads")
        .arg(config.num_threads.to_string());
    if let Some(index_dir) = &config.index_dir {
        command.arg("--index-dir").arg(index_dir);
    }
    command
        .arg("--")
        .arg(&config.query)
        .args(&config.file_paths);

    // Detach the process so that nothing waits for it: an editor running a search will wait for
    // its output to close, and Ctrl-C in a terminal goes to the whole foreground process group.
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x8;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x200;
        const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x4000;
        command.creation_flags(
            DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | BELOW_NORMAL_PRIORITY_CLASS,
        );
    }
    let mut child = command.spawn()?;

    // Reap the process if this one is still running when it finishes (eg: in a long-running
    // program using this crate); this thread just ends if this process exits first.
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Refresh the index as a background process started by [spawn_background_refresh]
pub fn run_background_refresh(config: &Config) -> io::Result<()> {
    lower_priority();
    thread::spawn(|| {
        thread::sleep(BACKGROUND_REFRESH_TIMEOUT);
        std::process::exit(1);
    });
    let dest = get_index_path(config).ok_or_else(|| invalid_data("no index path"))?;
    let owner = Owner::claim(&dest)?;
    let result = refresh(config, &dest, &owner);
    owner.release();
    result
}

/// Ask the OS to give this process the lowest CPU priority
///
/// Disk access is deliberately not throttled (eg: with macOS's `PRIO_DARWIN_BG`), which made a
/// full build of a large index take about 15 times longer while only sparing other processes a
/// few seconds of disk contention.
fn lower_priority() {
    #[cfg(unix)]
    unsafe {
        libc::setpriority(libc::PRIO_PROCESS, 0, 19);
    }
}

/// The file through which refreshes of an index make sure only one of them is running
struct Owner {
    path: PathBuf,
    id: String,
}

impl Owner {
    /// Become the owner of an index, replacing any other refresh
    fn claim(dest: &Path) -> io::Result<Owner> {
        static CLAIM_COUNT: AtomicUsize = AtomicUsize::new(0);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let owner = Owner {
            path: get_owner_path(dest),
            id: format!(
                "{} {} {}",
                std::process::id(),
                nanos,
                CLAIM_COUNT.fetch_add(1, Ordering::Relaxed)
            ),
        };
        fs::write(&owner.path, &owner.id)?;
        Ok(owner)
    }

    /// Return false if another refresh has taken over; otherwise record that this one is alive
    ///
    /// Only the file's modification time is updated, never its contents, so that check-ins on
    /// other threads never see a partially written id.
    fn check_in(&self) -> bool {
        if !fs::read_to_string(&self.path).is_ok_and(|id| id == self.id) {
            return false;
        }
        let _ = fs::File::options()
            .write(true)
            .open(&self.path)
            .and_then(|file| file.set_modified(SystemTime::now()));
        true
    }

    fn release(&self) {
        if fs::read_to_string(&self.path).is_ok_and(|id| id == self.id) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn get_owner_path(dest: &Path) -> PathBuf {
    dest.with_extension("owner")
}

/// Return true if a refresh of an index has checked in recently
fn is_refresh_running(dest: &Path) -> bool {
    fs::metadata(get_owner_path(dest))
        .and_then(|metadata| metadata.modified())
        .is_ok_and(|modified| {
            // A time in the future (eg: from clock changes) counts as running; the timeout on
            // background refreshes means that cannot block refreshes for long.
            modified
                .elapsed()
                .map_or(true, |elapsed| elapsed < CHECK_IN_TIMEOUT)
        })
}

/// The index file for a search
///
/// Paths in the index are stored exactly as the walk produces them, which depends on the working
/// directory and on how the search paths were written, so all of those are part of the key.
fn get_index_path(config: &Config) -> Option<PathBuf> {
    let dir = get_index_dir(config.index_dir.as_deref())?;
    let cwd = std::env::current_dir().ok()?.canonicalize().ok()?;
    let mut key = Vec::from(HEADER);
    key.extend_from_slice(cwd.to_string_lossy().as_bytes());
    for file_path in &config.file_paths {
        key.push(0);
        key.extend_from_slice(file_path.as_bytes());
    }
    key.push(0);
    key.extend_from_slice(config.file_type.to_string().as_bytes());
    Some(dir.join(format!("{:016x}.idx", fnv1a(&key))))
}

fn get_index_dir(index_dir: Option<&str>) -> Option<PathBuf> {
    match index_dir {
        Some(dir) => Some(PathBuf::from(dir)),
        None => get_default_index_dir(),
    }
}

/// Remove every index so that each is rebuilt by the next search that uses it
///
/// Pass the same `index_dir` as [crate::Args::index_dir], or None for the default directory.
/// Returns the directory and the number of indexes removed.
///
/// This also stops any refresh that is in progress, since it will no longer find its claim in the
/// owner file; otherwise it could write back an index built from the old one.
pub fn remove_all_indexes(index_dir: Option<&str>) -> io::Result<(PathBuf, usize)> {
    let dir = get_index_dir(index_dir)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no index directory"))?;
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok((dir, 0)),
        Err(err) => return Err(err),
    };
    let mut removed_count = 0;
    for entry in entries {
        let path = entry?.path();
        let Some(kind) = path
            .file_name()
            .and_then(|name| get_index_file_kind(name.to_str()?))
        else {
            continue;
        };
        fs::remove_file(&path)?;
        if kind == "idx" {
            removed_count += 1;
        }
    }
    Ok((dir, removed_count))
}

/// Return the kind (`idx`, `owner`, or `tmp`) of a file name created by this module, if it is one
///
/// The index directory could be shared with other files, so only files with these exact names are
/// ever removed: `<hash>.idx`, `<hash>.owner`, or `<hash>.<pid>.tmp`.
fn get_index_file_kind(name: &str) -> Option<&str> {
    let (hash, rest) = name.split_once('.')?;
    if hash.len() != 16 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    match rest.split_once('.') {
        None if rest == "idx" || rest == "owner" => Some(rest),
        Some((pid, "tmp")) if !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit()) => {
            Some("tmp")
        }
        _ => None,
    }
}

fn get_default_index_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CACHE_HOME") {
        return Some(PathBuf::from(dir).join("grepdef"));
    }
    if cfg!(windows) {
        return std::env::var_os("LOCALAPPDATA").map(|dir| PathBuf::from(dir).join("grepdef"));
    }
    let home = PathBuf::from(std::env::var_os("HOME")?);
    if cfg!(target_os = "macos") {
        Some(home.join("Library/Caches/grepdef"))
    } else {
        Some(home.join(".cache/grepdef"))
    }
}

/// A stable hash (unlike std's DefaultHasher) so index file names survive grepdef upgrades
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, &byte| {
        (hash ^ byte as u64).wrapping_mul(0x100000001b3)
    })
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Read the file lines of an index file
///
/// The format is a header line, then one line per file:
///
/// ```text
/// F <size> <mtime secs> <mtime nsecs> <ctime secs> <ctime nsecs> <path>
/// ```
///
/// where the line number (starting at 0) is the file id, then one line per symbol:
///
/// ```text
/// S <symbol> <file id> <file id>...
/// ```
///
/// Returns the files and the rest of the data, starting with the newline before the first symbol.
fn parse_files(data: &[u8]) -> io::Result<(Vec<StoredFile>, &[u8])> {
    if !data.starts_with(HEADER) {
        return Err(invalid_data("unknown index format"));
    }
    let mut files = vec![];
    let mut pos = HEADER.len();
    while pos < data.len() {
        let end = memchr(b'\n', &data[pos..]).map_or(data.len(), |i| pos + i);
        let line = &data[pos..end];
        if !line.starts_with(b"F ") {
            break;
        }
        files.push(parse_file_line(line).ok_or_else(|| invalid_data("bad file"))?);
        pos = end + 1;
    }
    Ok((files, &data[(pos - 1).min(data.len())..]))
}

fn parse_file_line(line: &[u8]) -> Option<(String, FileStamp)> {
    let line = std::str::from_utf8(line).ok()?;
    let mut fields = line.splitn(7, ' ').skip(1);
    let mut next_number = || fields.next()?.parse::<i64>().ok();
    let stamp = FileStamp {
        size: next_number()? as u64,
        mtime: (next_number()?, next_number()?),
        ctime: (next_number()?, next_number()?),
    };
    Some((fields.next()?.to_string(), stamp))
}

fn parse_file_ids(ids: &[u8]) -> io::Result<Vec<u32>> {
    let ids = std::str::from_utf8(ids).map_err(|_| invalid_data("bad symbol"))?;
    ids.split(' ')
        .map(|id| id.parse().map_err(|_| invalid_data("bad file id")))
        .collect()
}

/// Read an index file for a search, which only needs the symbols for the query
fn load(path: &Path, query: &str) -> io::Result<Index> {
    let data = fs::read(path)?;
    let (files, symbols) = parse_files(&data)?;

    // Every symbol line follows a newline, so searching for the newline too guarantees that only
    // a whole symbol at the start of a line can match.
    let needle = format!("\nS {query} ");
    let mut candidates = HashSet::new();
    if let Some(found) = memmem::find(symbols, needle.as_bytes()) {
        let ids_start = found + needle.len();
        let ids_end = memchr(b'\n', &symbols[ids_start..]).map_or(symbols.len(), |i| ids_start + i);
        candidates.extend(parse_file_ids(&symbols[ids_start..ids_end])?);
    }

    Ok(Index {
        files: files
            .into_iter()
            .enumerate()
            .map(|(id, (path, stamp))| (path, (id as u32, stamp)))
            .collect(),
        candidates,
        found_changed_file: AtomicBool::new(false),
        unchanged_file_count: AtomicUsize::new(0),
    })
}

/// A file's path and stamp as stored in an index
type StoredFile = (String, FileStamp);

/// The full contents of an index file, used to refresh it
#[derive(Default)]
struct StoredIndex {
    files: Vec<StoredFile>,
    symbol_names: Vec<String>,

    /// For each file id, the positions in `symbol_names` of the file's symbols
    symbols_by_file: Vec<Vec<u32>>,
}

fn load_stored(path: &Path) -> io::Result<StoredIndex> {
    let data = fs::read(path)?;
    let (files, symbols) = parse_files(&data)?;
    let mut stored = StoredIndex {
        symbols_by_file: vec![vec![]; files.len()],
        files,
        symbol_names: vec![],
    };
    for line in symbols.split(|&b| b == b'\n') {
        let Some(line) = line.strip_prefix(b"S ") else {
            continue;
        };
        let name_end = memchr(b' ', line).ok_or_else(|| invalid_data("bad symbol"))?;
        let name =
            std::str::from_utf8(&line[..name_end]).map_err(|_| invalid_data("bad symbol"))?;
        let symbol_id = stored.symbol_names.len() as u32;
        stored.symbol_names.push(name.to_string());
        for file_id in parse_file_ids(&line[name_end + 1..])? {
            stored
                .symbols_by_file
                .get_mut(file_id as usize)
                .ok_or_else(|| invalid_data("bad file id"))?
                .push(symbol_id);
        }
    }
    Ok(stored)
}

/// The symbols of a file found while refreshing an index
enum FileSymbols {
    /// The file has not changed, so its symbols are those of this file id in the old index
    Unchanged(u32),

    /// The file is new or changed and these are its symbols
    Found(HashSet<String>),
}

/// Walk all the files for a search and write an index of the symbols they define
///
/// Files which are unchanged in the existing index (if there is one) keep their symbols; the rest
/// are read. Files which no longer exist are dropped.
fn refresh(config: &Config, dest: &Path, owner: &Owner) -> io::Result<()> {
    let start_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as i128);
    let old = load_stored(dest).unwrap_or_default();
    let old_ids: HashMap<&str, u32> = old
        .files
        .iter()
        .enumerate()
        .map(|(id, (path, _))| (path.as_str(), id as u32))
        .collect();
    let extractors = query_regex::get_symbol_extractors(&config.file_type);
    let builder =
        super::make_walk_builder(config).ok_or_else(|| invalid_data("no paths to index"))?;
    let visited_count = AtomicUsize::new(0);
    let replaced = AtomicBool::new(false);

    let (tx, rx) = mpsc::channel::<(String, FileStamp, FileSymbols)>();
    builder.build_parallel().run(|| {
        let tx = tx.clone();
        let (old, old_ids, extractors) = (&old, &old_ids, &extractors);
        let (visited_count, replaced) = (&visited_count, &replaced);
        Box::new(move |entry| {
            if visited_count.fetch_add(1, Ordering::Relaxed) % CHECK_IN_INTERVAL == 0
                && !owner.check_in()
            {
                replaced.store(true, Ordering::Relaxed);
            }
            if replaced.load(Ordering::Relaxed) {
                return WalkState::Quit;
            }
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => return WalkState::Continue,
            };
            let path = match entry.path().to_str() {
                Some(path) if is_indexable(path, &entry) => path,
                _ => return WalkState::Continue,
            };
            if !file_type::path_matches_file_type(path, &config.file_type) {
                return WalkState::Continue;
            }
            // Stamp the file before reading it so that a change made while reading is detected.
            let stamp = match entry.metadata() {
                Ok(metadata) => FileStamp::from_metadata(&metadata),
                Err(_) => return WalkState::Continue,
            };
            if let Some(&old_id) = old_ids.get(path) {
                if old.files[old_id as usize].1 == stamp {
                    let _ = tx.send((path.to_string(), stamp, FileSymbols::Unchanged(old_id)));
                    return WalkState::Continue;
                }
            }
            if stamp.newest_nanos() >= start_nanos - RACY_WINDOW_NANOS {
                return WalkState::Continue;
            }
            let bytes = match fs::read(entry.path()) {
                Ok(bytes) => bytes,
                Err(_) => return WalkState::Continue,
            };
            let symbols = FileSymbols::Found(extract_symbols(&bytes, extractors));
            let _ = tx.send((path.to_string(), stamp, symbols));
            WalkState::Continue
        })
    });
    drop(tx);
    let mut found_files: Vec<(String, FileStamp, FileSymbols)> = rx.into_iter().collect();

    if replaced.load(Ordering::Relaxed) || !owner.check_in() {
        return Err(io::Error::other("replaced by another refresh"));
    }

    // Overlapping search paths could find the same file twice.
    let mut seen_paths = HashSet::new();
    found_files.retain(|(path, _, _)| seen_paths.insert(path.clone()));

    let mut symbols: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    for (id, (_, _, file_symbols)) in found_files.iter().enumerate() {
        let id = id as u32;
        match file_symbols {
            FileSymbols::Unchanged(old_id) => {
                for &symbol_id in &old.symbols_by_file[*old_id as usize] {
                    let name = old.symbol_names[symbol_id as usize].as_str();
                    symbols.entry(name).or_default().push(id);
                }
            }
            FileSymbols::Found(names) => {
                for name in names {
                    symbols.entry(name.as_str()).or_default().push(id);
                }
            }
        }
    }
    let files: Vec<(&str, &FileStamp)> = found_files
        .iter()
        .map(|(path, stamp, _)| (path.as_str(), stamp))
        .collect();
    write_index(dest, &files, &symbols)
}

/// Find every symbol that the definition patterns could match in a file
///
/// This must find at least every symbol that a search could find, so it mirrors how a search
/// works: each line is matched separately. It also finds more than a search might (eg: keywords
/// like `function`), which only costs a little index space.
///
/// - The combined regex is run over the whole file only to quickly find the lines worth looking
///   at; any line on which a branch matches will contain a match of the combined regex.
/// - Each branch is run separately, and restarted after the start of each match rather than its
///   end, because a regex only reports one non-overlapping match per position and a query's match
///   could overlap with (or be in a different branch than) the one reported.
/// - A match's captured symbol is recorded along with every word in the match, because a regex
///   with optional parts (eg: Ruby's `(?:self\.)?`) may capture a different word than the one a
///   query matches (eg: for `def self.foo`, the capture is `foo` but a query for `self` matches).
fn extract_symbols(bytes: &[u8], extractors: &query_regex::SymbolExtractors) -> HashSet<String> {
    let mut symbols = HashSet::new();
    let mut add_symbol = |symbol: &[u8]| {
        if let Ok(symbol) = std::str::from_utf8(symbol) {
            if !symbols.contains(symbol) {
                symbols.insert(symbol.to_string());
            }
        }
    };
    let mut pos = 0;
    while pos < bytes.len() {
        let Some(found) = extractors.combined.find_at(bytes, pos) else {
            break;
        };
        let line_start = memrchr(b'\n', &bytes[..found.start()]).map_or(0, |i| i + 1);
        let line_end =
            memchr(b'\n', &bytes[found.start()..]).map_or(bytes.len(), |i| found.start() + i);
        let line = &bytes[line_start..line_end];
        for branch in &extractors.branches {
            let mut line_pos = 0;
            while line_pos < line.len() {
                let Some(captures) = branch.captures_at(line, line_pos) else {
                    break;
                };
                let whole = captures.get(0).expect("Match must have a group 0");
                for word in extractors.word.find_iter(whole.as_bytes()) {
                    add_symbol(word.as_bytes());
                }
                for group in captures.iter().skip(1).flatten() {
                    add_symbol(group.as_bytes());
                }
                line_pos = whole.start() + 1;
            }
        }
        pos = line_end + 1;
    }
    symbols
}

/// Write the index to a temporary file and then move it into place, so that a search never reads
/// a partially written index.
fn write_index(
    dest: &Path,
    files: &[(&str, &FileStamp)],
    symbols: &BTreeMap<&str, Vec<u32>>,
) -> io::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension(format!("{}.tmp", std::process::id()));
    let result = (|| {
        let mut out = BufWriter::new(fs::File::create(&tmp)?);
        out.write_all(HEADER)?;
        for (path, stamp) in files {
            writeln!(
                out,
                "F {} {} {} {} {} {}",
                stamp.size, stamp.mtime.0, stamp.mtime.1, stamp.ctime.0, stamp.ctime.1, path
            )?;
        }
        for (symbol, ids) in symbols {
            write!(out, "S {symbol}")?;
            for id in ids {
                write!(out, " {id}")?;
            }
            out.write_all(b"\n")?;
        }
        out.flush()?;
        fs::rename(&tmp, dest)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Args, FileType};

    fn extract(text: &str, file_type: &FileType) -> HashSet<String> {
        extract_symbols(
            text.as_bytes(),
            &query_regex::get_symbol_extractors(file_type),
        )
    }

    #[test]
    fn extract_symbols_includes_words_a_query_can_match_besides_the_capture() {
        // A query for `self` matches this line by skipping the optional `self.`
        let symbols = extract("def self.foo\nend\n", &FileType::RB);
        assert!(symbols.contains("foo"));
        assert!(symbols.contains("self"));
    }

    #[test]
    fn extract_symbols_includes_overlapping_matches() {
        // The first match (`function function`) consumes the start of the second one.
        let symbols = extract("function function foo() {}\n", &FileType::JS);
        assert!(symbols.contains("foo"));
    }

    /// Every word that a search would find in a fixture must be in the index for that fixture.
    #[test]
    fn extract_symbols_finds_every_symbol_a_search_finds_in_fixtures() {
        let word = regex::bytes::Regex::new(r"\w+").unwrap();
        for (path, file_type) in [
            ("js-fixture.js", FileType::JS),
            ("ts-fixture.ts", FileType::JS),
            ("jsx-fixture.jsx", FileType::JS),
            ("tsx-fixture.tsx", FileType::JS),
            ("php-fixture.php", FileType::PHP),
            ("rs-fixture.rs", FileType::RS),
            ("py-fixture.py", FileType::PY),
            ("rb-fixture.rb", FileType::RB),
        ] {
            let bytes = fs::read(format!("tests/fixtures/by-language/{path}")).unwrap();
            let symbols = extract_symbols(&bytes, &query_regex::get_symbol_extractors(&file_type));
            let words: HashSet<&[u8]> = word.find_iter(&bytes).map(|m| m.as_bytes()).collect();
            for query in words {
                let query = std::str::from_utf8(query).unwrap();
                let re = query_regex::get_regex_for_query(query, &file_type);
                if bytes.split(|&b| b == b'\n').any(|line| re.is_match(line)) {
                    assert!(symbols.contains(query), "{path} is missing symbol {query}");
                }
            }
        }
    }

    fn make_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("grepdef-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_config(query: &str, dir: &Path, index_dir: &Path) -> Config {
        Config::new(Args {
            query: query.into(),
            file_type: Some("php".into()),
            file_path: Some(vec![dir.to_str().unwrap().into()]),
            index: true,
            index_dir: Some(index_dir.to_str().unwrap().into()),
            ..Args::default()
        })
        .unwrap()
    }

    fn get_indexed_files_for(config: &Config) -> Vec<String> {
        let index = load(&get_index_path(config).unwrap(), &config.query).unwrap();
        let mut paths: Vec<String> = index
            .files
            .iter()
            .filter(|(_, (id, _))| index.candidates.contains(id))
            .map(|(path, _)| {
                Path::new(path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into()
            })
            .collect();
        paths.sort();
        paths
    }

    fn wait_until_files_can_be_indexed() {
        thread::sleep(Duration::from_millis(2100));
    }

    #[test]
    fn refresh_keeps_unchanged_files_and_updates_changed_ones() {
        let dir = make_temp_dir("refresh-src");
        let index_dir = make_temp_dir("refresh-cache");
        fs::write(dir.join("a.php"), "<?php\nfunction foo() {}\n").unwrap();
        fs::write(dir.join("b.php"), "<?php\nfunction foo() {}\n").unwrap();
        fs::write(dir.join("c.php"), "<?php\nfunction bar() {}\n").unwrap();
        wait_until_files_can_be_indexed();
        let config = make_config("foo", &dir, &index_dir);
        let dest = get_index_path(&config).unwrap();
        let refresh_now = || {
            let owner = Owner::claim(&dest).unwrap();
            refresh(&config, &dest, &owner).unwrap();
            owner.release();
        };

        refresh_now();
        assert_eq!(vec!["a.php", "b.php"], get_indexed_files_for(&config));

        fs::remove_file(dir.join("a.php")).unwrap();
        fs::write(dir.join("c.php"), "<?php\nfunction foo() {}\n").unwrap();
        fs::write(dir.join("d.php"), "<?php\nfunction foo() {}\n").unwrap();
        wait_until_files_can_be_indexed();
        refresh_now();
        assert_eq!(
            vec!["b.php", "c.php", "d.php"],
            get_indexed_files_for(&config)
        );
        assert!(!get_owner_path(&dest).exists());

        fs::remove_dir_all(&dir).unwrap();
        fs::remove_dir_all(&index_dir).unwrap();
    }

    #[test]
    fn refresh_leaves_out_recently_changed_files() {
        let dir = make_temp_dir("racy-src");
        let index_dir = make_temp_dir("racy-cache");
        fs::write(dir.join("a.php"), "<?php\nfunction foo() {}\n").unwrap();
        let config = make_config("foo", &dir, &index_dir);
        let dest = get_index_path(&config).unwrap();
        let owner = Owner::claim(&dest).unwrap();
        refresh(&config, &dest, &owner).unwrap();
        let index = load(&dest, "foo").unwrap();
        assert!(index.files.is_empty());

        fs::remove_dir_all(&dir).unwrap();
        fs::remove_dir_all(&index_dir).unwrap();
    }

    #[test]
    fn refresh_stops_without_writing_if_replaced() {
        let dir = make_temp_dir("replaced-src");
        let index_dir = make_temp_dir("replaced-cache");
        fs::write(dir.join("a.php"), "<?php\nfunction foo() {}\n").unwrap();
        let config = make_config("foo", &dir, &index_dir);
        let dest = get_index_path(&config).unwrap();
        let owner = Owner::claim(&dest).unwrap();
        let other_owner = Owner::claim(&dest).unwrap();
        assert!(refresh(&config, &dest, &owner).is_err());
        assert!(!dest.exists());

        // Releasing the replaced owner must not remove the new owner's claim.
        owner.release();
        assert!(other_owner.check_in());

        fs::remove_dir_all(&dir).unwrap();
        fs::remove_dir_all(&index_dir).unwrap();
    }

    #[test]
    fn remove_all_indexes_removes_only_index_files_and_stops_refreshes() {
        let index_dir = make_temp_dir("remove-cache");
        let dest = index_dir.join("0123456789abcdef.idx");
        fs::write(&dest, HEADER).unwrap();
        fs::write(index_dir.join("fedcba9876543210.idx"), HEADER).unwrap();
        fs::write(index_dir.join("0123456789abcdef.123.tmp"), "").unwrap();
        fs::write(index_dir.join("notes.txt"), "").unwrap();
        fs::write(index_dir.join("0123456789abcdef.txt"), "").unwrap();
        let owner = Owner::claim(&dest).unwrap();

        let (dir, removed_count) = remove_all_indexes(Some(index_dir.to_str().unwrap())).unwrap();
        assert_eq!(index_dir, dir);
        assert_eq!(2, removed_count);
        let mut remaining: Vec<String> = fs::read_dir(&index_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into())
            .collect();
        remaining.sort();
        assert_eq!(vec!["0123456789abcdef.txt", "notes.txt"], remaining);
        assert!(!owner.check_in());

        fs::remove_dir_all(&index_dir).unwrap();
    }

    #[test]
    fn remove_all_indexes_succeeds_if_there_is_no_index_dir() {
        let index_dir = std::env::temp_dir().join("grepdef-index-dir-that-does-not-exist");
        let (_, removed_count) = remove_all_indexes(Some(index_dir.to_str().unwrap())).unwrap();
        assert_eq!(0, removed_count);
    }

    #[test]
    fn is_refresh_running_is_false_once_owner_stops_checking_in() {
        let index_dir = make_temp_dir("running-cache");
        let dest = index_dir.join("test.idx");
        assert!(!is_refresh_running(&dest));

        let owner = Owner::claim(&dest).unwrap();
        assert!(is_refresh_running(&dest));

        let long_ago = SystemTime::now() - CHECK_IN_TIMEOUT - Duration::from_secs(1);
        fs::File::options()
            .write(true)
            .open(get_owner_path(&dest))
            .unwrap()
            .set_modified(long_ago)
            .unwrap();
        assert!(!is_refresh_running(&dest));

        owner.release();
        assert!(!is_refresh_running(&dest));
        fs::remove_dir_all(&index_dir).unwrap();
    }
}
