use log::{debug, info};
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::process;

struct Client {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Activity {
    #[serde(rename(deserialize = "activityId"))]
    id: u64,

    #[serde(rename(deserialize = "activityName"))]
    name: String,

    description: Option<String>,
}

// move this to errors.rs
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidInput(String),
    RequestFailed(String),
    UnexpectedServerResponse,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RequestFailed(error.to_string())
    }
}

impl Client {
    pub fn new(email: &str, password: &str) -> Self {
        Client {
            email: email.to_string(),
            password: password.to_string(),
        }
    }

    pub fn list_activities(&self) -> Result<Vec<Activity>> {
        self.auth()
    }

    fn auth(&self) -> Result<Vec<Activity>> {
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .build()?;

        let res = client
            .post("https://sso.garmin.com/sso/signin")
            .header("origin", "https://sso.garmin.com")
            .query(&[("service", "https://connect.garmin.com/modern")])
            .form(&[
                ("username", &self.email),
                ("password", &self.password),
                ("embed", &false.to_string()),
            ])
            .send()?;

        debug!("Claiming the authentication toket");
        let ticket = extract_ticket_url(&res.text()?)?;
        let res = client
            .get("https://connect.garmin.com/modern")
            .query(&[("ticket", ticket)])
            .send()?;

        assert_eq!(res.status(), 200);

        debug!("Pinging legacy endpoint");
        client
            .get("https://connect.garmin.com/legacy/session")
            .send()?;

        let res = client.get("https://connect.garmin.com/modern/proxy/activitylist-service/activities/search/activities")
            .query(&[
                ("start", 0.to_string()),
                ("limit", 10.to_string())
            ])
            .send()?
            .json()?;

        Ok(res)
    }
}

fn extract_ticket_url(auth_response: &str) -> Result<String> {
    let re = Regex::new(r#"response_url\s*=\s*"(https:[^"]+)""#).unwrap();

    let matches = re
        .captures_iter(auth_response)
        .next()
        .ok_or(Error::UnexpectedServerResponse)?;
    let first_match = matches[1].to_string();
    let v: Vec<&str> = first_match.split("?ticket=").collect();
    Ok(v[1].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let client = Client::new("john.doe@example.com", "password");
        assert_eq!(client.email, "john.doe@example.com");
        assert_eq!(client.password, "password");
    }

    #[test]
    fn parse_ticket() {
        let auth_response = r#"response_url = "https:\/\/connect.garmin.com\/modern?ticket=ST-0123456-aBCDefgh1iJkLmN5opQ9R-cas";"#;
        assert_eq!(
            extract_ticket_url(auth_response),
            Ok("ST-0123456-aBCDefgh1iJkLmN5opQ9R-cas".to_string())
        )
    }
}

struct Config {
    username: String,
    password: String,
}

impl Config {
    fn new(mut args: env::Args) -> Result<Self> {
        args.next();

        let username = args
            .next()
            .ok_or(Error::InvalidInput("Username is missing".to_string()))?;
        let password = args
            .next()
            .ok_or(Error::InvalidInput("Password is missing".to_string()))?;

        Ok(Self { username, password })
    }
}

fn main() {
    let env = env_logger::Env::default().filter_or("GARMIN_CONNECT_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {:?}", err);
        process::exit(1);
    });

    let client = Client::new(&config.username.to_string(), &config.password.to_string());

    let activities = client.list_activities().unwrap_or_else(|err| {
        eprintln!("Error listing the activities: {:?}", err);
        process::exit(1);
    });

    info!("Activities: {:#?}", activities);
}
