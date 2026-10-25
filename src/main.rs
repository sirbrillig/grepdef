use clap::Parser;
use grepdef::Args;
use grepdef::Searcher;
use std::process;

fn main() {
    let mut args = Args::parse();
    args.index_refresh_command = std::env::current_exe().ok();
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
