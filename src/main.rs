mod activity;
mod client;
mod error;
mod rate_limiter;

use client::Client;
use error::*;
use log::*;
use std::fs;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "garmin_export",
    about = "Export FIT files from connect.garmin.com"
)]
struct Config {
    username: String,
    password: String,
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(about = "Export a given activity")]
    Activity {
        #[structopt(help = "Activity ID")]
        id: activity::ActivityId,
    },
    Fit {
        path: std::path::PathBuf,
    },
}

fn main() {
    let env = env_logger::Env::default().filter_or("GARMIN_EXPORT_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let config = Config::from_args();
    debug!("{:?}", config);

    let result = match config.cmd {
        None => {
            let client = Client::new(&config.username, &config.password).unwrap();
            download_activities(&client)
        }
        Some(Command::Activity { id }) => {
            let client = Client::new(&config.username, &config.password).unwrap();
            download_activity(&client, id)
        }
        Some(Command::Fit { path }) => show_fit(&path),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(err) => {
            error!("Couldn't download the specified activity: {:?}", err);
            process::exit(1);
        }
    }
}

fn download_activity(client: &Client, id: activity::ActivityId) -> Result<()> {
    let zip = client.download_activity(id)?;
    let fname = format!("{}.fit", id);
    fs::write(fname, zip.as_slice())?;
    Ok(())
}

fn download_activities(client: &Client) -> Result<()> {
    for activity in client.list_activities()?.iter() {
        download_activity(client, activity.id())?;
    }
    Ok(())
}

fn show_fit(path: &std::path::PathBuf) -> Result<()> {
    let mut fp = std::fs::File::open(&path)?;
    for data in fitparser::from_reader(&mut fp).unwrap() {
        println!("{:#?}", data);
    }
    Ok(())
}
