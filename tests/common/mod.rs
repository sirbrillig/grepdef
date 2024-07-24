use grepdef::{Args, SearchResult, Searcher};

pub fn make_args(
    query: String,
    file_path: Option<String>,
    file_type_string: Option<String>,
) -> Args {
    Args {
        query,
        file_path: match file_path {
            Some(file_path) => Some(file_path.split_whitespace().map(String::from).collect()),
            None => None,
        },
        file_type: file_type_string,
        line_number: true,
        search_method: None,
        debug: false,
        no_color: false,
        threads: None,
    }
}

pub fn do_search(args: Args) -> Vec<SearchResult> {
    let searcher = Searcher::new(args).unwrap();
    searcher.search().expect("Search failed for test")
}

pub fn get_default_fixture_for_file_type_string(file_type_string: &str) -> Result<String, String> {
    match file_type_string {
        "js" => Ok(String::from("./tests/fixtures/by-language/js-fixture.js")),
        "ts" => Ok(String::from("./tests/fixtures/by-language/ts-fixture.ts")),
        "jsx" => Ok(String::from("./tests/fixtures/by-language/jsx-fixture.jsx")),
        "tsx" => Ok(String::from("./tests/fixtures/by-language/tsx-fixture.tsx")),
        "php" => Ok(String::from("./tests/fixtures/by-language/php-fixture.php")),
        "rs" => Ok(String::from("./tests/fixtures/by-language/rs-fixture.rs")),
        _ => {
            return Err(format!(
                "No fixture found for file type '{}'",
                file_type_string
            ))
        }
    }
}

pub fn get_expected_text_line_for_test_search(
    file_type_string: &str,
) -> Result<(String, usize), String> {
    match file_type_string {
        "js" => Ok((String::from("function parseQuery() {"), 7)),
        "ts" => Ok((String::from("function parseQueryTS(): string {"), 7)),
        "jsx" => Ok((String::from("function parseQuery() {"), 7)),
        "tsx" => Ok((String::from("function parseQueryTS(): string {"), 7)),
        "php" => Ok((String::from("function parseQuery() {"), 6)),
        "rs" => Ok((String::from("pub fn query_db() -> bool {}"), 1)),
        _ => {
            return Err(format!(
                "No expected text found for file type '{}'",
                file_type_string
            ))
        }
    }
}

pub fn get_expected_search_result_for_file_type(file_type_string: &str) -> SearchResult {
    let (text, line_number) = get_expected_text_line_for_test_search(file_type_string).unwrap();
    SearchResult {
        file_path: get_default_fixture_for_file_type_string(file_type_string).unwrap(),
        line_number: Some(line_number),
        text,
    }
}
