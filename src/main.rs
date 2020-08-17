mod activity;
mod client;
mod config;
mod error;

use client::Client;
use config::Config;
use log::info;
use std::env;
use std::process;

fn main() {
    let env = env_logger::Env::default().filter_or("GARMIN_EXPORT_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {:?}", err);
        process::exit(1);
    });

    let client = Client::new(&config.username, &config.password).unwrap_or_else(|err| {
        eprintln!("Cannot connect to connect.garmin.com {:?}", err);
        process::exit(1);
    });

    let activities = client.list_activities().unwrap_or_else(|err| {
        eprintln!("Error listing the activities: {:?}", err);
        process::exit(1);
    });

    info!("Activities: {:#?}", activities);
}
