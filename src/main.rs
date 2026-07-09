mod config;
mod logging;
mod state;
mod ui;

use std::{path::PathBuf, process};

use clap::Parser;
use gtk4::{
    Application,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
};

use crate::{
    config::{
        Config,
        io::{load_config, write_config},
    },
    state::State,
    ui::build_ui,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    readonly: bool,

    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    command: Option<String>,
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        logging::set_verbose(true);
    }

    if args.command.as_deref() == Some("config") {
        gtk4::init().expect("Failed to get GTK working.");
        write_config(args.config.as_deref(), Config::default());
        process::exit(0);
    }

    let app = Application::builder()
        .application_id(format!("ch.skew.{}", env!("CARGO_PKG_NAME")))
        .build();

    app.connect_activate(move |app| {
        let config = load_config(args.config.as_deref());
        config.validate();
        let state = State::new(
            config.dir.clone(),
            args.readonly,
            config.max_recurrence_count,
            config.max_recurrence_date,
        );
        debug!("Using directory: {:?}", config.dir);

        build_ui(app, config, state);
    });

    app.run_with_args::<String>(&[]);
}
