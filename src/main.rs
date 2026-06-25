mod config;
mod logging;
mod state;
mod ui;

use std::path::PathBuf;

use clap::Parser;

use crate::{config::io::load_config, state::State};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    readonly: bool,

    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        logging::set_verbose(true);
    }

    let config = load_config(args.config.as_deref());

    debug!("Using directory: {:?}", config.dir);

    let state = State::new(config.dir, args.readonly);
}
