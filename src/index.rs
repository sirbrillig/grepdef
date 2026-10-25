//! An optional on-disk index of which files define which symbols
//!
//! The index never gives a wrong answer because it is only used to skip files that have not changed
//! since the index was built. Every search still walks the file tree and compares each file's size
//! and timestamps against the index; any file that is new or changed is searched directly, as is
//! every unchanged file that the index lists for the query. Files the index lists are still
//! searched normally, so a stale entry can only cost time, never produce a result.

use super::{debug, file_type, query_regex, Config};
use ignore::{DirEntry, WalkState};
use memchr::{memchr, memmem, memrchr};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{self, SystemTime, UNIX_EPOCH};

/// The first line of every index file; change the version when the format or the meaning of its
/// contents changes (eg: if the definition patterns change) so that old indexes are rebuilt.
const HEADER: &[u8] = b"grepdef-index 1\n";

/// Files changed within this many seconds of the index being built are left out of the index
///
/// Some filesystems have coarse timestamps, so a file changed just after being indexed could keep
/// the same timestamps as when it was read. Leaving recently changed files out of the index means
/// they will always be searched directly. Two seconds covers the coarsest common filesystem (FAT).
const RACY_WINDOW_SECONDS: i64 = 2;

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

    fn newest_seconds(&self) -> i64 {
        self.mtime.0.max(self.ctime.0)
    }
}

/// An index loaded for a single query
pub struct Index {
    /// Every indexed file, by path, with its id and the stamp it had when indexed
    files: HashMap<String, (u32, FileStamp)>,

    /// The ids of the files which the index lists as defining the query
    candidates: HashSet<u32>,
}

impl Index {
    /// Return true if a file found while walking needs to be searched
    pub fn should_search(&self, path: &str, entry: &DirEntry) -> bool {
        let Some((id, stamp)) = self.files.get(path) else {
            // New (or never indexable) file
            return true;
        };
        match entry.metadata() {
            Ok(metadata) if FileStamp::from_metadata(&metadata) == *stamp => {
                self.candidates.contains(id)
            }
            _ => true,
        }
    }

    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }
}

/// Load the index for a search, building it first if needed
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
            return Some(index);
        }
        Err(err) => debug(
            config,
            format!("No usable index at {}: {}", path.display(), err).as_str(),
        ),
    }
    if let Err(err) = build(config, &path) {
        debug(config, format!("Could not build index: {}", err).as_str());
        return None;
    }
    debug(
        config,
        format!(
            "Built index {} in {} ms",
            path.display(),
            start.elapsed().as_millis()
        )
        .as_str(),
    );
    load(&path, &config.query).ok()
}

/// The index file for a search
///
/// Paths in the index are stored exactly as the walk produces them, which depends on the working
/// directory and on how the search paths were written, so all of those are part of the key.
fn get_index_path(config: &Config) -> Option<PathBuf> {
    let dir = match &config.index_dir {
        Some(dir) => PathBuf::from(dir),
        None => get_default_index_dir()?,
    };
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

/// Read an index file
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
fn load(path: &Path, query: &str) -> io::Result<Index> {
    let data = fs::read(path)?;
    if !data.starts_with(HEADER) {
        return Err(invalid_data("unknown index format"));
    }

    let mut files = HashMap::new();
    let mut pos = HEADER.len();
    while pos < data.len() {
        let end = memchr(b'\n', &data[pos..]).map_or(data.len(), |i| pos + i);
        let line = &data[pos..end];
        if !line.starts_with(b"F ") {
            break;
        }
        let (file_path, stamp) = parse_file_line(line).ok_or_else(|| invalid_data("bad file"))?;
        let id = files.len() as u32;
        files.insert(file_path, (id, stamp));
        pos = end + 1;
    }

    // Every symbol line follows a newline (the header ends with one), so searching for the
    // newline too guarantees that only a whole symbol at the start of a line can match.
    let symbols = &data[pos - 1..];
    let needle = format!("\nS {query} ");
    let mut candidates = HashSet::new();
    if let Some(found) = memmem::find(symbols, needle.as_bytes()) {
        let ids_start = found + needle.len();
        let ids_end = memchr(b'\n', &symbols[ids_start..]).map_or(symbols.len(), |i| ids_start + i);
        let ids = std::str::from_utf8(&symbols[ids_start..ids_end])
            .map_err(|_| invalid_data("bad symbol"))?;
        for id in ids.split(' ') {
            candidates.insert(id.parse().map_err(|_| invalid_data("bad file id"))?);
        }
    }

    Ok(Index { files, candidates })
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

/// Walk all the files for a search, record which symbols each one defines, and write the index
fn build(config: &Config, dest: &Path) -> io::Result<()> {
    let start_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let extractors = query_regex::get_symbol_extractors(&config.file_type);
    let builder =
        super::make_walk_builder(config).ok_or_else(|| invalid_data("no paths to index"))?;

    let (tx, rx) = mpsc::channel::<(String, FileStamp, HashSet<String>)>();
    builder.build_parallel().run(|| {
        let tx = tx.clone();
        let extractors = &extractors;
        Box::new(move |entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => return WalkState::Continue,
            };
            // Only regular files can be trusted by their stamp; others (eg: symlinks, whose
            // stamp is the link's and not the target's) are left out so they are always searched.
            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                return WalkState::Continue;
            }
            let path = match entry.path().to_str() {
                Some(path) if !path.contains('\n') => path,
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
            if stamp.newest_seconds() >= start_seconds - RACY_WINDOW_SECONDS {
                return WalkState::Continue;
            }
            let bytes = match fs::read(entry.path()) {
                Ok(bytes) => bytes,
                Err(_) => return WalkState::Continue,
            };
            let _ = tx.send((path.to_string(), stamp, extract_symbols(&bytes, extractors)));
            WalkState::Continue
        })
    });
    drop(tx);

    let mut files: Vec<(String, FileStamp)> = vec![];
    let mut symbols: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for (path, stamp, file_symbols) in rx {
        let id = files.len() as u32;
        for symbol in file_symbols {
            symbols.entry(symbol).or_default().push(id);
        }
        files.push((path, stamp));
    }
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
    files: &[(String, FileStamp)],
    symbols: &BTreeMap<String, Vec<u32>>,
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
    use crate::FileType;

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
}
