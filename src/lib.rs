#![warn(missing_docs)]
//! Quick search for symbol definitions in various programming languages
//!
//! Currently this supports Rust, JS (or TypeScript), Python, PHP, and Ruby.
//!
//! This can be used like "Go to definition" in an IDE, except that instead of using a language
//! server, it just searches for the definition using text parsing. This is less accurate but often
//! faster in projects with lots of files or where a language server won't work or hasn't yet
//! started.
//!
//! grepdef since 3.0 is written in Rust and is designed to be extremely fast.
//!
//! This can also be used as a library crate for other Rust programs.
//!
//! # Example
//!
//! The syntax of the CLI is similar to that of `grep` or `ripgrep`: first put the symbol you want
//! to search for (eg: a function name, class name, etc.) and then list the file(s) or directories
//! over which you want to search.
//!
//! ```text
//! $ grepdef parseQuery ./src
//! ./src/queries.js:function parseQuery {
//! ```
//!
//! Just like `grep`, you can add the `-n` option to include line numbers.
//!
//! ```text
//! $ grepdef -n parseQuery ./src
//! ./src/queries.js:17:function parseQuery {
//! ```
//!
//! The search will be faster if you specify what type of file you are searching for using the
//! `--type` option.
//!
//! ```text
//! $ grepdef --type js -n parseQuery ./src
//! ./src/queries.js:17:function parseQuery {
//! ```
//!
//! To use the crate from other Rust code, use [Searcher].
//!
//! ```
//! use grepdef::{Args, Searcher};
//!
//! for result in Searcher::new(Args::from_query("parseQuery")).unwrap().search().unwrap() {
//!     println!("{}", result.to_grep());
//! }
//! ```

use clap::Parser;
use colored::Colorize;
use ignore::{WalkBuilder, WalkState};
use regex::Regex;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Seek};
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time;
use strum_macros::Display;
use strum_macros::EnumString;

mod file_type;
mod query_regex;

/// The command-line arguments to be used by [Searcher]
///
/// Can be passed to [Searcher::new].
///
/// The only required property is [Args::query].
///
/// # Example
///
/// ```
/// use grepdef::Args;
/// let config = Args::from_query("parseQuery");
/// assert_eq!(config.query, String::from("parseQuery"));
/// assert_eq!(config.file_path, None); // The current directory
/// assert_eq!(config.file_type, None); // Auto-detect the file type
/// assert_eq!(config.line_number, false); // Do not print line numbers
/// ```
#[derive(Parser, Debug, Default)]
#[command(
    version,
    arg_required_else_help = true,
    about = "Quick search for symbol definitions in various programming languages",
    long_about = "Quick search for symbol definitions in various programming languages"
)]
pub struct Args {
    /// (Required) The symbol name (function, class, etc.) to search for
    pub query: String,

    /// The file path(s) to search; recursively searches directories and respects .gitignore
    pub file_path: Option<Vec<String>>,

    /// The file type to search (js, php, rs, ts, py, rb); will guess if not set
    #[arg(short = 't', long = "type")]
    pub file_type: Option<String>,

    /// Show line numbers of matches if set
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// Control color output ("never", "always", "auto"); default "auto"
    #[arg(long = "color")]
    pub color: Option<String>,

    /// Disable color (also supports NO_COLOR env)
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Limit the number of results
    #[arg(short = 'l', long = "limit")]
    pub limit: Option<usize>,

    /// (Advanced) Print debugging information
    #[arg(long = "debug")]
    pub debug: bool,

    /// (Advanced) The searching method
    #[arg(long = "search-method")]
    pub search_method: Option<SearchMethod>,

    /// (Advanced) The number of threads to use
    #[arg(short = 'j', long = "threads")]
    pub threads: Option<NonZero<usize>>,

    /// The output format; defaults to 'grep'
    #[arg(long = "format")]
    pub format: Option<SearchResultFormat>,
}

impl Args {
    /// Create a new set of arguments for [Searcher] with the minimal configuration
    pub fn from_query(query: &str) -> Args {
        Args {
            query: query.into(),
            ..Args::default()
        }
    }

    /// Create a new set of arguments for [Searcher]
    pub fn new(
        query: String,
        file_type: Option<String>,
        file_path: Option<Vec<String>>,
        line_number: bool,
    ) -> Args {
        Args {
            query,
            file_type,
            file_path,
            line_number,
            ..Args::default()
        }
    }
}

/// (Advanced) The type of underlying search algorithm to use
///
/// In general, a pre-scan is a good idea to quickly skip files that don't have a match, which
/// should be most files. You shouldn't need to change this from the default.
#[derive(clap::ValueEnum, Clone, Default, Debug, EnumString, PartialEq, Display)]
pub enum SearchMethod {
    /// Pre-scan each file by reading fully into memory and using a Regex
    #[default]
    PrescanRegex,

    /// Pre-scan each file by reading bytes until the query is found using memmem
    PrescanMemmem,

    /// Don't pre-scan files.
    NoPrescan,
}

/// The configuration used by a [Searcher]
///
/// Created by passing [Args] to [Config::new].
#[derive(Clone, Debug)]
struct Config {
    /// The symbol name (function, class, etc.) being searched for
    query: String,

    /// The list of file paths to search, ignoring invisible or gitignored files
    file_paths: Vec<String>,

    /// The type of files to scan (JS or PHP or RS)
    file_type: FileType,

    /// Include line numbers in results if true
    line_number: bool,

    /// Output debugging info during search if true
    debug: bool,

    /// Limit the number of results
    limit: Option<usize>,

    /// Explicitly disable color output if true
    no_color: bool,

    /// Explicitly control color output ("never", "always", "auto")
    color: ColorOption,

    /// The [SearchMethod] to use
    search_method: SearchMethod,

    /// The number of threads to use for searching files
    num_threads: NonZero<usize>,

    /// The output format
    format: SearchResultFormat,
}

impl Config {
    /// Create a new Config using an [Args]
    pub fn new(args: Args) -> Result<Config, String> {
        if args.debug {
            let args_formatted = format!("Creating config with args {:?}", args);
            println!("{}", args_formatted.yellow());
        }
        let file_paths = match args.file_path {
            Some(file_path) => file_path,
            None => vec![".".into()],
        };
        let file_type = match args.file_type {
            Some(file_type_string) => FileType::from_string(file_type_string.as_str())?,
            None => FileType::from_file_paths(&file_paths)?,
        };
        let color = match args.color {
            Some(color_option_string) => ColorOption::from_string(color_option_string.as_str())?,
            None => ColorOption::AUTO,
        };

        let num_threads = match args.threads {
            Some(threads) => threads,
            None => std::thread::available_parallelism().unwrap_or(NonZero::new(5).unwrap()),
        };

        let config = Config {
            query: args.query,
            file_paths,
            file_type,
            line_number: args.line_number,
            debug: args.debug,
            no_color: args.no_color,
            color,
            search_method: args.search_method.unwrap_or_default(),
            limit: args.limit,
            num_threads,
            format: args.format.unwrap_or_default(),
        };
        debug(&config, format!("Created config {:?}", config).as_str());
        Ok(config)
    }
}

/// The supported file types to search
///
/// You can turn a string into a [FileType] using [FileType::from_string] which also supports
/// type aliases like `javascript`, `javascriptreact`, or `typescript.tsx`.
#[derive(Clone, Debug)]
pub enum FileType {
    /// The JS (or TS) file type
    JS,

    /// The PHP file type
    PHP,

    /// The Rust file type
    RS,

    /// The Python file type
    PY,

    /// The Ruby file type
    RB,
}

impl FileType {
    /// Turn a string into a [FileType]
    ///
    /// You can turn a string into a [FileType] using [FileType::from_string] which also supports
    /// type aliases like `javascript`, `javascriptreact`, or `typescript.tsx`.
    pub fn from_string(file_type_string: &str) -> Result<FileType, String> {
        match file_type_string {
            "js" => Ok(FileType::JS),
            "ts" => Ok(FileType::JS),
            "jsx" => Ok(FileType::JS),
            "tsx" => Ok(FileType::JS),
            "javascript" => Ok(FileType::JS),
            "javascript.jsx" => Ok(FileType::JS),
            "javascriptreact" => Ok(FileType::JS),
            "typescript" => Ok(FileType::JS),
            "typescript.tsx" => Ok(FileType::JS),
            "typescriptreact" => Ok(FileType::JS),
            "php" => Ok(FileType::PHP),
            "rs" => Ok(FileType::RS),
            "rust" => Ok(FileType::RS),
            "py" => Ok(FileType::PY),
            "python" => Ok(FileType::PY),
            "rb" => Ok(FileType::RB),
            "ruby" => Ok(FileType::RB),
            _ => Err(format!("Invalid file type '{}'", file_type_string)),
        }
    }

    /// Get the textual representation of a [FileType]
    pub fn to_string(&self) -> String {
        match self {
            Self::JS => String::from("js"),
            Self::PHP => String::from("php"),
            Self::RS => String::from("rs"),
            Self::PY => String::from("py"),
            Self::RB => String::from("rb"),
        }
    }

    /// Try to guess a [FileType] based on a list of file paths
    ///
    /// This can examine files or recursive directories and try to determine the [FileType] to
    /// search for. It will return the file type of the first file it finds that matches one of the
    /// file type patterns the crate supports.
    ///
    /// If a directory includes multiple supported file types, this could be incorrect, so it's
    /// more reliable (and faster) to specify a file type explicitly.
    pub fn from_file_paths(file_paths: &Vec<String>) -> Result<FileType, &'static str> {
        for file_path in file_paths {
            let guess = file_type::guess_file_type_from_file_path(file_path);
            if let Some(value) = guess {
                return Ok(value);
            }
        }
        Err("Unable to guess file type from file paths")
    }
}

/// The supported arguments to the color option
///
/// You can turn a string into a [ColorOption] using [ColorOption::from_string].
#[derive(Clone, Debug)]
pub enum ColorOption {
    /// Always colorize
    ALWAYS,

    /// Never colorize
    NEVER,

    /// Auto-detect colorize
    AUTO,
}

impl ColorOption {
    /// Convert string to ColorOption
    pub fn from_string(color_option_string: &str) -> Result<ColorOption, String> {
        match color_option_string {
            "always" => Ok(ColorOption::ALWAYS),
            "never" => Ok(ColorOption::NEVER),
            "auto" => Ok(ColorOption::AUTO),
            _ => Err(format!("Invalid color option '{}'", color_option_string)),
        }
    }
}

/// The output format of [SearchResult]
#[derive(clap::ValueEnum, Clone, Default, Debug, EnumString, PartialEq, Display, Copy)]
pub enum SearchResultFormat {
    /// grep-like output; colon-separated path, line number, and text
    #[default]
    Grep,

    /// JSON output; one document per match
    JsonPerMatch,

    /// JSON output; one array document with all results
    JsonList,
}

/// The type of a search event
///
/// Search events are a concept that really only exists for [SearchResultFormat::JsonList].
///
/// When that flag is not enabled, [SearchResult] will have an `event_type` property that is always
/// [SearchEventType::NONE] and serialized instances of [SearchResult] will exclude the property
/// entirely.
///
/// When [SearchResultFormat::JsonList] is enabled, the list of results will include a
/// [SearchEventResult] at the start and end of its list with [SearchEventType::START] and
/// [SearchEventType::END] respectively. Also, each [SearchResult] match will have the `event_type`
/// property [SearchEventType::MATCH].
///
/// Most search events are of type [SearchEventType::MATCH], but [SearchResultFormat::JsonList]
/// includes results for the start and end of a search as well.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum SearchEventType {
    /// The start of a search. Only present in [SearchResultFormat::JsonList].
    START,

    /// The end of a search. Only present in [SearchResultFormat::JsonList].
    END,

    /// A match. Each result will use this in [SearchResultFormat::JsonList].
    MATCH,

    /// An empty type. Primarily used when [SearchResultFormat::JsonList] is not set.
    NONE,
}

impl SearchEventType {
    fn is_empty(&self) -> bool {
        match self {
            SearchEventType::NONE => true,
            _ => false,
        }
    }
}

/// A search event that is not a result but rather just an informative marker
///
/// This is required because the [SearchResultFormat::JsonList] format needs "start" and "end" markers.
#[derive(serde::Serialize)]
struct SearchEventResult {
    pub event_type: SearchEventType,
}

impl SearchEventResult {
    pub fn to_json_in_list(&self) -> String {
        match self.event_type {
            SearchEventType::START => serde_json::to_string(self).unwrap_or_default() + ",",
            SearchEventType::END => serde_json::to_string(self).unwrap_or_default(),
            SearchEventType::MATCH => serde_json::to_string(self).unwrap_or_default() + ",",
            SearchEventType::NONE => String::from(""),
        }
    }
}

/// A result from calling [Searcher::search] or [Searcher::search_and_format]
///
/// Note that `line_number` will be set only if [Args::line_number] is true when searching.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct SearchResult {
    /// The event type. This will always be [SearchEventType::MATCH].
    #[serde(skip_serializing_if = "SearchEventType::is_empty")]
    pub event_type: SearchEventType,

    /// The path to the file containing the symbol definition
    pub file_path: String,

    /// The line number of the symbol definition in the file
    pub line_number: Option<usize>,

    /// The symbol definition line
    pub text: String,
}

impl SearchResult {
    /// Return a formatted string for output in the "grep" format
    ///
    /// That is, either `file path:text on line` or, if [Args::line_number] is true,
    /// `file path:line number:text on line`.
    ///
    /// # Example
    ///
    /// If [Args::line_number] is true,
    ///
    /// ```text
    /// ./src/queries.js:17:function parseQuery {
    /// ```
    pub fn to_grep(&self) -> String {
        match self.line_number {
            Some(line_number) => format!(
                "{}:{}:{}",
                self.file_path.magenta(),
                line_number.to_string().green(),
                self.text
            ),
            None => format!("{}:{}", self.file_path.magenta(), self.text),
        }
    }

    /// Return a formatted string for output in the [SearchResultFormat::JsonPerMatch] format
    pub fn to_json_per_match(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Return a formatted string for output in the [SearchResultFormat::JsonList] format
    pub fn to_json_in_list(&self) -> String {
        serde_json::to_string(self).unwrap_or_default() + ","
    }
}

/// A struct that can perform a search
///
/// This is the main API of this crate.
///
/// # Example
///
/// ```
/// use grepdef::{Args, Searcher};
/// let searcher = Searcher::new(Args::new(
///     String::from("parseQuery"),
///     None,
///     None,
///     true
/// ))
/// .unwrap();
///
/// for result in searcher.search_and_format().unwrap() {
///     println!("{}", result);
/// }
///
/// searcher.search_and_format_callback(|line| println!("{}", line));
/// ```
pub struct Searcher {
    config: Config,
}

impl Searcher {
    /// Create a new Config using an [Args]
    pub fn new(args: Args) -> Result<Searcher, String> {
        let config = Config::new(args)?;
        Ok(Searcher { config })
    }

    /// Perform the search and return formatted strings
    pub fn search_and_format(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut results: Vec<String> = vec![];
        let error = self.search_and_format_callback(|result| results.push(result));
        match error {
            Ok(_) => Ok(results),
            Err(err) => Err(err),
        }
    }

    /// Perform the search and run a callback for each formatted string
    pub fn search_and_format_callback<F>(&self, mut callback: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(String),
    {
        if self.config.format == SearchResultFormat::JsonList {
            callback(String::from("["));
            let event = SearchEventResult {
                event_type: SearchEventType::START,
            };
            callback(event.to_json_in_list());
        }
        let error = self.search_callback(|result| match self.config.format {
            SearchResultFormat::Grep => callback(result.to_grep()),
            SearchResultFormat::JsonPerMatch => callback(result.to_json_per_match()),
            SearchResultFormat::JsonList => callback(result.to_json_in_list()),
        });
        if self.config.format == SearchResultFormat::JsonList {
            let event = SearchEventResult {
                event_type: SearchEventType::END,
            };
            callback(event.to_json_in_list());
            callback(String::from("]"));
        }
        error
    }

    /// Perform the search and call the callback for each result
    pub fn search_callback<F>(&self, mut callback: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(SearchResult),
    {
        // Don't try to even calculate elapsed time if we are not going to print it
        let start: Option<time::Instant> = if self.config.debug {
            Some(time::Instant::now())
        } else {
            None
        };
        let re = query_regex::get_regex_for_query(&self.config.query, &self.config.file_type);
        let config = Arc::new(self.config.clone());

        match config.color {
            ColorOption::ALWAYS => colored::control::set_override(true),
            ColorOption::NEVER => colored::control::set_override(false),
            ColorOption::AUTO => (),
        }
        if config.no_color {
            colored::control::set_override(false);
        }

        self.debug("Starting searchers");
        let should_quit = Arc::new(AtomicBool::new(false));
        let searched_file_count = Arc::new(AtomicUsize::new(0));

        // Spawn a thread to run the parallel walk. When it finishes (or all tx clones are
        // dropped), the rx iterator will end naturally.
        let rx = {
            let (tx, rx) = mpsc::channel();
            let should_quit_walk = Arc::clone(&should_quit);
            let searched_file_count_walk = Arc::clone(&searched_file_count);
            let config_walk = Arc::clone(&config);
            let re_walk = re.clone();

            std::thread::spawn(move || {
                if config_walk.file_paths.is_empty() {
                    return;
                }
                // WalkBuilder::new() requires an initial path, so seed it with
                // the first and add the rest.
                let mut builder = WalkBuilder::new(&config_walk.file_paths[0]);
                for fp in config_walk.file_paths.iter().skip(1) {
                    builder.add(fp);
                }
                builder.threads(config_walk.num_threads.get());

                // run() uses a two-level closure: the outer closure is called once
                // per walk thread to set up per-thread state (cloning shared values),
                // and the returned Box<dyn FnMut> is the visitor called for each entry
                // on that thread.
                builder.build_parallel().run(|| {
                    let tx = tx.clone();
                    let re = re_walk.clone();
                    let config = Arc::clone(&config_walk);
                    let should_quit = Arc::clone(&should_quit_walk);
                    let searched_file_count = Arc::clone(&searched_file_count_walk);

                    Box::new(move |entry| {
                        if should_quit.load(Ordering::Relaxed) {
                            return WalkState::Quit;
                        }
                        let entry = match entry {
                            Ok(e) => e,
                            Err(_) => return WalkState::Continue,
                        };
                        if entry.path().is_dir() {
                            return WalkState::Continue;
                        }
                        let path_str = match entry.path().to_str() {
                            Some(p) => p.to_string(),
                            None => return WalkState::Continue,
                        };
                        if !file_type::path_matches_file_type(&path_str, &config.file_type) {
                            return WalkState::Continue;
                        }
                        searched_file_count.fetch_add(1, Ordering::Relaxed);
                        let tx_file = tx.clone();
                        search_file(&re, &path_str, &config, move |results| {
                            let _ = tx_file.send(results);
                        });
                        WalkState::Continue
                    })
                });
            });

            rx
        };

        self.debug("Listening to searcher results");
        let mut result_counter: usize = 0;
        'all_results: for received_results in rx {
            for received_result in received_results {
                result_counter += 1;
                callback(received_result);
                // Don't try to even calculate elapsed time if we are not going to print it
                if let (true, Some(start)) = (config.debug, start) {
                    self.debug(
                        format!("Found a result in {} ms", start.elapsed().as_millis()).as_str(),
                    );
                }
                if let Some(i) = config.limit {
                    self.debug(format!("This is result {}; limit {}", result_counter, i).as_str());
                    if result_counter >= i {
                        self.debug("Limit reached");
                        should_quit.store(true, Ordering::Relaxed);
                        break 'all_results;
                    }
                }
            }
        }

        // Don't try to even calculate elapsed time if we are not going to print it
        if let (true, Some(start)) = (config.debug, start) {
            self.debug(
                format!(
                    "Scanned {} files in {} ms",
                    searched_file_count.load(Ordering::Relaxed),
                    start.elapsed().as_millis()
                )
                .as_str(),
            );
        }
        Ok(())
    }

    /// Perform the search and return [SearchResult] structs
    pub fn search(&self) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let mut results: Vec<SearchResult> = vec![];
        let search_result = self.search_callback(|result| results.push(result));
        match search_result {
            Ok(_) => Ok(results),
            Err(err) => Err(err),
        }
    }

    fn debug(&self, output: &str) {
        if self.config.debug {
            println!("{}", output.yellow());
        }
    }
}

fn debug(config: &Config, output: &str) {
    if config.debug {
        println!("{}", output.yellow());
    }
}

fn search_file<F>(re: &Regex, file_path: &str, config: &Config, callback: F)
where
    F: FnOnce(Vec<SearchResult>) + Send + 'static,
{
    debug(config, format!("Scanning file {}", file_path).as_str());
    let file = fs::File::open(file_path);

    match file {
        Ok(mut file) => {
            // Scan the file in big chunks to see if it has what we are looking for. This is more efficient
            // than going line-by-line on every file since matches should be quite rare.
            if match config.search_method {
                SearchMethod::PrescanRegex => !file_type::does_file_match_regexp(&file, re),
                SearchMethod::PrescanMemmem => {
                    !file_type::does_file_match_query(&file, &config.query)
                }
                SearchMethod::NoPrescan => false,
            } {
                debug(
                    config,
                    format!("Presearch of {} found no match; skipping", &file_path).as_str(),
                );
                callback(vec![]);
                return;
            }

            let rewind_result = file.rewind();
            if rewind_result.is_err() {
                callback(vec![]);
                return;
            }
            debug(
                config,
                format!(
                    "Presearch of {} was successful; searching for line",
                    &file_path
                )
                .as_str(),
            );
            callback(search_file_line_by_line(re, file_path, &file, config));
        }
        Err(_) => {
            callback(vec![]);
        }
    }
}

fn search_file_line_by_line(
    re: &Regex,
    file_path: &str,
    file: &fs::File,
    config: &Config,
) -> Vec<SearchResult> {
    let lines = io::BufReader::new(file).lines();
    let mut line_counter = 0;

    lines
        .filter_map(|line| {
            line_counter += 1;
            if !match &line {
                Ok(line) => re.is_match(line),
                Err(_) => false,
            } {
                return None;
            }

            let text = match line {
                Ok(line) => line,
                // If reading the line causes an error (eg: invalid UTF), then skip it by treating
                // it as empty.
                Err(_err) => String::from(""),
            };

            Some(SearchResult {
                event_type: match config.format {
                    SearchResultFormat::JsonList => SearchEventType::MATCH,
                    _ => SearchEventType::NONE,
                },
                file_path: String::from(file_path),
                line_number: if config.line_number {
                    Some(line_counter)
                } else {
                    None
                },
                text: text.trim().into(),
            })
        })
        .collect()
}
