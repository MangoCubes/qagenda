mod logging;
mod state;

use clap::Parser;
use std::path::PathBuf;

use crate::state::State;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_os_t = default_dir())]
    caldir: PathBuf,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    readonly: bool,
}

fn default_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    return PathBuf::from("./events");
    #[cfg(not(debug_assertions))]
    return PathBuf::from(std::env::var("HOME").expect("No home???")).join(".calendar");
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        logging::set_verbose(true);
    }
    debug!("Verbose mode enabled");
    debug!("Using directory: {:?}", args.caldir);

    let state = State::new(args.caldir, args.readonly);
}
