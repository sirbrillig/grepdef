use clap::Parser;
use grepdef::Args;
use grepdef::Searcher;
use std::process;
use std::time::Instant;

fn main() {
    let mut args = Args::parse();
    args.index_refresh_command = std::env::current_exe().ok();
    if args.clear_indexes {
        match grepdef::remove_all_indexes(args.index_dir.as_deref()) {
            // Print to stderr so that this never mixes with search results.
            Ok((dir, count)) => eprintln!(
                "Removed {count} index(es) from {}; each will be rebuilt by the next search that uses it",
                dir.display()
            ),
            Err(err) => {
                eprintln!("Could not remove indexes: {err}");
                process::exit(exitcode::IOERR);
            }
        }
        if args.query.is_empty() && !args.update_index {
            return;
        }
    }
    let update_index = args.update_index;
    let searcher = Searcher::new(args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    });
    if update_index {
        let start = Instant::now();
        match searcher.update_index() {
            Ok(update) => {
                eprintln!(
                    "Updated index {}: {} files indexed ({} read, {} removed) in {:.2}s",
                    update.index_path.display(),
                    update.indexed_file_count,
                    update.read_file_count,
                    update.removed_file_count,
                    start.elapsed().as_secs_f64()
                );
                if update.recently_changed_file_count > 0 {
                    eprintln!(
                        "{} files kept changing and were not indexed; they will be searched directly",
                        update.recently_changed_file_count
                    );
                }
            }
            Err(err) => {
                eprintln!("Could not update index: {err}");
                process::exit(exitcode::IOERR);
            }
        }
        return;
    }
    let search_result = searcher.search_and_format_callback(|line| {
        println!("{}", line);
    });
    match search_result {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{err}");
            process::exit(exitcode::USAGE);
        }
    }
}
