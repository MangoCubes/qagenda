mod logging;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_os_t = default_dir())]
    caldir: PathBuf,

    #[arg(short, long)]
    verbose: bool,
}

fn default_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    return PathBuf::from("./events");
    #[cfg(not(debug_assertions))]
    return PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "~".to_string()))
        .join(".calendar");
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        logging::set_verbose(true);
    }
    debug!("Verbose mode enabled");
    debug!("Using directory: {:?}", args.caldir);
}
