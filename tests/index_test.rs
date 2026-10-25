use grepdef::{Args, Searcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::{thread, time};

fn make_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("grepdef-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn search_with_index(query: &str, dir: &Path, index_dir: &Path) -> Vec<String> {
    let mut args = Args::new(
        query.into(),
        Some("php".into()),
        Some(vec![dir.to_str().unwrap().into()]),
        false,
    );
    args.index = true;
    args.index_dir = Some(index_dir.to_str().unwrap().into());
    let mut paths: Vec<String> = Searcher::new(args)
        .unwrap()
        .search()
        .unwrap()
        .into_iter()
        .map(|result| {
            Path::new(&result.file_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();
    paths.sort();
    paths
}

#[test]
fn search_with_index_finds_definitions_in_changed_new_and_unchanged_files() {
    let dir = make_temp_dir("index-src");
    let index_dir = make_temp_dir("index-cache");
    fs::write(dir.join("a.php"), "<?php\nfunction foo() {}\n").unwrap();
    fs::write(dir.join("b.php"), "<?php\nfunction bar() {}\n").unwrap();
    fs::write(dir.join("c.php"), "<?php\n// nothing\n").unwrap();
    fs::write(dir.join("e.php"), "<?php\ndefine('foo-bar', 1);\n").unwrap();

    // Files changed just before indexing are not trusted by the index, so wait until they can be.
    thread::sleep(time::Duration::from_millis(2100));

    // Builds the index, then uses it.
    assert_eq!(vec!["a.php"], search_with_index("foo", &dir, &index_dir));
    assert!(fs::read_dir(&index_dir).unwrap().count() > 0);
    assert_eq!(vec!["a.php"], search_with_index("foo", &dir, &index_dir));

    // A changed file that the index does not list.
    fs::write(dir.join("c.php"), "<?php\nfunction foo() {}\n").unwrap();
    assert_eq!(
        vec!["a.php", "c.php"],
        search_with_index("foo", &dir, &index_dir)
    );

    // A new file.
    fs::write(dir.join("d.php"), "<?php\nfunction foo() {}\n").unwrap();
    assert_eq!(
        vec!["a.php", "c.php", "d.php"],
        search_with_index("foo", &dir, &index_dir)
    );

    // A changed file that the index lists but no longer defines the symbol.
    fs::write(dir.join("a.php"), "<?php\nfunction other() {}\n").unwrap();
    assert_eq!(
        vec!["c.php", "d.php"],
        search_with_index("foo", &dir, &index_dir)
    );

    // A deleted file.
    fs::remove_file(dir.join("b.php")).unwrap();
    assert_eq!(
        Vec::<String>::new(),
        search_with_index("bar", &dir, &index_dir)
    );

    // A query that is not a plain symbol does not use the index, which only lists plain symbols.
    assert_eq!(
        vec!["e.php"],
        search_with_index("foo-bar", &dir, &index_dir)
    );

    fs::remove_dir_all(&dir).unwrap();
    fs::remove_dir_all(&index_dir).unwrap();
}
