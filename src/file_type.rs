use super::FileType;
use ignore::Walk;
use memchr::memmem;
use regex::Regex;
use std::fs;
use std::io::Read;

pub fn get_regexp_for_file_type(file_type: &FileType) -> Regex {
    let regexp_string = match file_type {
        FileType::JS => &r"\.(js|jsx|ts|tsx|mjs|cjs)$".to_string(),
        FileType::PHP => &r"\.php$".to_string(),
        FileType::RS => &r"\.rs$".to_string(),
    };
    Regex::new(regexp_string).expect("Could not create regex for file extension")
}

pub fn guess_file_type_from_file_path(file_path: &str) -> Option<FileType> {
    let js_regex = get_regexp_for_file_type(&FileType::JS);
    let php_regex = get_regexp_for_file_type(&FileType::PHP);
    let rs_regex = get_regexp_for_file_type(&FileType::RS);
    for entry in Walk::new(file_path) {
        let path = match entry {
            Ok(path) => path.into_path(),
            Err(_) => continue,
        };
        if path.is_dir() {
            continue;
        }
        let path = match path.to_str() {
            Some(p) => p.to_string(),
            None => String::from(""),
        };
        if js_regex.is_match(&path) {
            return Some(FileType::JS);
        }
        if php_regex.is_match(&path) {
            return Some(FileType::PHP);
        }
        if rs_regex.is_match(&path) {
            return Some(FileType::RS);
        }
    }
    None
}

pub fn does_file_match_regexp(mut file: &fs::File, re: &Regex) -> bool {
    let mut buf = String::new();
    let bytes = file.read_to_string(&mut buf);
    if bytes.unwrap_or(0) == 0 {
        return false;
    }
    re.is_match(&buf)
}

pub fn does_file_match_query(mut file: &fs::File, query: &str) -> bool {
    let mut full: Vec<u8> = vec![];
    let mut buf = [0u8; 2048];
    let finder = memmem::Finder::new(query);
    loop {
        let bytes = file.read(&mut buf);
        if bytes.unwrap_or(0) == 0 {
            break false;
        }
        if full.contains(&0xA) {
            let mut split_full = full.rsplit(|&b| b == b'\n');
            full = split_full.next().unwrap_or(&[0u8, 1]).to_vec();
        }
        full.extend(buf);
        if finder.find(&full).is_some() {
            break true;
        }
    }
}
