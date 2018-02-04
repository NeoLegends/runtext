#![feature(conservative_impl_trait)]

#[macro_use] extern crate clap;
#[macro_use] extern crate futures;
extern crate futures_stream_select_all;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_yaml;
extern crate tokio_core;

mod actions;
mod context;
mod driver;
mod multi;
mod triggers;

use std::env;
use std::fs;

use clap::{Arg, AppSettings};
use futures::future::Executor;
use tokio_core::reactor::Core;

use context::Config;
use driver::drive;

const CONFIG_FILE_PARAM: &'static str = "CONFIG_FILE";
const NO_DAEMON_PARAM: &'static str = "NO_DAEMON";
const PID_FILE_PARAM: &'static str = "PID_FILE";

fn main() {
    let default_cfg_file = "~/.config/runtext.yml".to_owned();
    let pid_path = env::temp_dir().join("runtext.pid");
    let default_pid_file = pid_path.to_string_lossy().to_owned();

    let matches = app_from_crate!()
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::GlobalVersion)
        .arg(
            Arg::with_name(CONFIG_FILE_PARAM)
                .help("The path to the configuration file. Can be either json or yaml.")
                .short("c")
                .long("config")
                .value_name("FILE")
                .index(1)
                .default_value(&default_cfg_file)
                .takes_value(true)
                .global(true)
        )
        .arg(
            Arg::with_name(NO_DAEMON_PARAM)
                .help("Don't daemonize during startup.")
                .long("no-daemon")
                .global(true),
        )
        .arg(
            Arg::with_name(PID_FILE_PARAM)
                .help("The path to the pid file for daemonization.")
                .short("p")
                .long("pid")
                .required(true)
                .default_value(&default_pid_file)
                .takes_value(true)
                .global(true)
        )
        .get_matches();

    let cfg = {
        let path = matches.value_of(CONFIG_FILE_PARAM).unwrap();
        let rdr = fs::File::open(path)
            .expect(&format!("Could not open config file '{}'. Does it exist?", path));

        let cfg: Config = serde_yaml::from_reader(rdr)
            .expect("Failed to parse config. Please ensure it is valid yaml or json and the structure is valid.");

        if let Err(err) = cfg.validate() {
            panic!("Config is invalid, {}", err);
        }

        cfg
    };

    start(cfg);
}

fn start(config: Config) {
    let mut core = Core::new().unwrap();

    let handle = core.handle();
    let drivers = config.into_iter()
        .map(|ctx| drive(ctx, handle.clone()))
        .map(|d| d.unwrap());

    for driver in drivers {
        core.execute(driver).unwrap();
    }

    core.run(futures::empty::<(), ()>()).unwrap();
}
