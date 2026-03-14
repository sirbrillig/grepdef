use super::FileType;
use regex::Regex;

pub fn get_regex_for_query(query: &str, file_type: &FileType) -> Regex {
    let regexp_string = match file_type {
        FileType::JS => &format!(
            r"(?:\b(?:function|var|let|const|class|interface|type)\s+{query}\b|\b{query}\([^)]*\)\s*(?::[^\{{]+)?\{{|\b{query}:|@typedef\s*(?:\{{[^\}}]+\}})?\s*{query}\b)"
        ),
        FileType::PHP => &format!(
            r#"\b(?:function|class|trait|interface|enum|const) {query}\b|\bdefine\s*\(\s*['"]{query}['"]"#
        ),
        FileType::RS => &format!(r"\b(?:fn|trait|enum|struct|mod) {query}\b"),
        FileType::PY => &format!(r"\b(?:(?:async\s+)?def|class)\s+{query}\b"),
        FileType::RB => &format!(r"\b(?:def|class|module)\s+(?:self\.)?{query}\b"),
    };
    Regex::new(regexp_string).expect("Could not create regex for file type query")
}
