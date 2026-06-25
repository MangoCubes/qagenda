mod config;
mod logging;
mod state;
mod ui;

use std::path::PathBuf;

use clap::Parser;
use gtk4::{
    Application,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
};

use crate::{config::io::load_config, state::State, ui::build_ui};

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

    let app = Application::builder()
        .application_id("ch.skew.qcal")
        .build();

    app.connect_activate(move |app| {
        let config = load_config(args.config.as_deref());
        let state = State::new(config.dir.clone(), args.readonly);
        debug!("Using directory: {:?}", config.dir);

        build_ui(app, config, state);
    });

    app.run_with_args::<String>(&[]);
}
