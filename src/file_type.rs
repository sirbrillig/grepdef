use super::FileType;
use ignore::Walk;
use memchr::memmem;
use regex::Regex;
use std::fs;
use std::io::Read;

pub fn path_matches_file_type(path: &str, file_type: &FileType) -> bool {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match file_type {
        FileType::JS => matches!(ext, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs"),
        FileType::PHP => ext == "php",
        FileType::RS => ext == "rs",
        FileType::PY => ext == "py",
        FileType::RB => ext == "rb",
    }
}

pub fn guess_file_type_from_file_path(file_path: &str) -> Option<FileType> {
    for entry in Walk::new(file_path) {
        let path = match entry {
            Ok(path) => path.into_path(),
            Err(_) => continue,
        };
        if path.is_dir() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        match ext {
            "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => return Some(FileType::JS),
            "php" => return Some(FileType::PHP),
            "rs" => return Some(FileType::RS),
            "py" => return Some(FileType::PY),
            "rb" => return Some(FileType::RB),
            _ => continue,
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
    // Keep query.len()-1 bytes of overlap between chunks so matches that
    // span a chunk boundary are not missed.
    let overlap = query.len().saturating_sub(1);
    loop {
        let n = file.read(&mut buf).unwrap_or(0);
        if n == 0 {
            break false;
        }
        full.extend_from_slice(&buf[..n]);
        if finder.find(&full).is_some() {
            break true;
        }
        if full.len() > overlap {
            full.drain(..full.len() - overlap);
        }
    }
}
