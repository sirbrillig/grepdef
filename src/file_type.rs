use super::FileType;
use ignore::Walk;

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
