mod activity;
mod client;
mod config;
mod error;

use client::Client;
use config::Config;
use error::*;
use log::*;
use std::env;
use std::fs;
use std::process;

fn main() {
    let env = env_logger::Env::default().filter_or("GARMIN_EXPORT_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        error!("Problem parsing arguments: {:?}", err);
        process::exit(1);
    });

    download_activities(config).unwrap_or_else(|err| {
        error!("Couldn't download activities: {:?}", err);
        process::exit(1);
    });
}

fn download_activities(config: Config) -> Result<()> {
    let client = Client::new(&config.username, &config.password)?;

    for activity in client.list_activities()?.iter() {
        let zip = client.download_activity(activity.id())?;
        let fname = format!("{}.zip", activity.id());
        fs::write(fname, zip.as_slice())?;
    }

    Ok(())
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error.to_string())
    }
}
