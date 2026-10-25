use clap::Parser;
use grepdef::Args;
use grepdef::Searcher;
use std::process;

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
        if args.query.is_empty() {
            return;
        }
    }
    let searcher = Searcher::new(args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    });
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
