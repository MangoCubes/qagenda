mod config;
mod logging;
mod state;
mod ui;

use clap::Parser;

use crate::state::State;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    readonly: bool,
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
