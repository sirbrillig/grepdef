use super::FileType;
use regex::bytes::Regex;

/// The placeholder in each pattern branch that is replaced by the query
const QUERY_PLACEHOLDER: &str = "{query}";

/// The definition patterns for each file type, as a list of alternation branches.
///
/// These are shared by searching (where the placeholder becomes the query) and indexing (where
/// the placeholder becomes a capture group) so that the index can never disagree with a search.
fn get_pattern_branches(file_type: &FileType) -> &'static [&'static str] {
    match file_type {
        FileType::JS => &[
            r"\b(?:function|var|let|const|class|interface|type)\s+{query}\b",
            r"\b{query}\([^)]*\)\s*(?::[^\{]+)?\{",
            r"\b{query}:",
            r"@typedef\s*(?:\{[^\}]+\})?\s*{query}\b",
        ],
        FileType::PHP => &[
            r"\b(?:function|class|trait|interface|enum|const) {query}\b",
            r#"\bdefine\s*\(\s*['"]{query}['"]"#,
        ],
        FileType::RS => &[r"\b(?:fn|trait|enum|struct|mod) {query}\b"],
        FileType::PY => &[r"\b(?:(?:async\s+)?def|class)\s+{query}\b"],
        FileType::RB => &[r"\b(?:def|class|module)\s+(?:self\.)?{query}\b"],
    }
}

fn join_branches(file_type: &FileType, replacement: &str) -> String {
    let branches: Vec<String> = get_pattern_branches(file_type)
        .iter()
        .map(|branch| branch.replace(QUERY_PLACEHOLDER, replacement))
        .collect();
    format!("(?:{})", branches.join("|"))
}

pub fn get_regex_for_query(query: &str, file_type: &FileType) -> Regex {
    Regex::new(&join_branches(file_type, query))
        .expect("Could not create regex for file type query")
}

/// Regexes used to extract every possible symbol definition from a file
pub struct SymbolExtractors {
    /// All branches combined; used to quickly find lines that might contain a definition
    pub combined: Regex,

    /// Each branch on its own, with the query replaced by a capture group
    pub branches: Vec<Regex>,

    /// Matches a single symbol word
    pub word: Regex,
}

/// The pattern that a symbol must match to be indexed (and so a query must match to use the index)
const SYMBOL_PATTERN: &str = r"\w+";

pub fn get_symbol_extractors(file_type: &FileType) -> SymbolExtractors {
    let capture = format!("({SYMBOL_PATTERN})");
    SymbolExtractors {
        combined: Regex::new(&join_branches(file_type, &capture))
            .expect("Could not create combined symbol regex"),
        branches: get_pattern_branches(file_type)
            .iter()
            .map(|branch| {
                Regex::new(&branch.replace(QUERY_PLACEHOLDER, &capture))
                    .expect("Could not create symbol regex")
            })
            .collect(),
        word: Regex::new(SYMBOL_PATTERN).expect("Could not create word regex"),
    }
}

/// Return true if the query is a plain symbol that can be looked up in an index
///
/// The index only lists symbols made of word characters, so a query with any other characters
/// (eg: a PHP constant like `define('foo-bar', 1)`) must be searched for without the index.
pub fn is_plain_symbol(query: &str) -> bool {
    Regex::new(&format!("^{SYMBOL_PATTERN}$"))
        .expect("Could not create symbol regex")
        .is_match(query.as_bytes())
}
