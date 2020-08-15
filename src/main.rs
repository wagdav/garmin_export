use log::{info, warn};
use regex::Regex;
use std::env;
use std::process;

struct Client {
    email: String,
    password: String,
}

#[derive(Debug)]
struct Activity;

impl Client {
    pub fn new(email: &str, password: &str) -> Self {
        Client {
            email: email.to_string(),
            password: password.to_string(),
        }
    }

    pub fn list_activities(&self) -> Vec<Activity> {
        self.auth();
        vec![Activity, Activity]
    }

    fn auth(&self) {
        let form_params = [
            ("username", &self.email),
            ("password", &self.password),
            ("embed", &false.to_string()),
        ];
        let query_params = [("service", "https://connect.garmin.com/modern")];

        let client = reqwest::blocking::Client::new();
        let res = client
            .post("https://sso.garmin.com/sso/signin")
            .header("origin", "https://sso.garmin.com")
            .form(&form_params)
            .query(&query_params)
            .send()
            .unwrap(); // FIXME: should return Result

        info!("Extracting the ticket url");
        let ticket_url = extract_ticket_url(&res.text().unwrap());
        info!("ticket url={}", ticket_url);
        let res = client.get(&ticket_url).send().unwrap();
        println!("status={:#?}", res.status());

        info!("Pinging legacy endpoint");
        client
            .get("https://connect.garmin.com/legacy/session")
            .send()
            .unwrap();

        let query_params = [("start", 0.to_string()), ("limit", 10.to_string())];
        let res = client.get("https://connect.garmin.com/modern/proxy/activitylist-service/activities/search/activities").query(&query_params).send().unwrap();

        println!("status={:#?}", res.status());
        println!("text={:#?}", res.text());
    }
}

fn extract_ticket_url(auth_response: &str) -> String {
    let re = Regex::new(r#"response_url\s*=\s*"(https:[^"]+)""#).unwrap();
    re.captures_iter(auth_response).next().unwrap()[1].to_string().replace(r"\/", "/")
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
            r#"https://connect.garmin.com/modern?ticket=ST-0123456-aBCDefgh1iJkLmN5opQ9R-cas"#
        );
    }
}

struct Config {
    username: String,
    password: String,
}

impl Config {
    fn new(mut args: env::Args) -> Result<Self, &'static str> {
        args.next();

        let username = args.next().ok_or("Username is missing")?;
        let password = args.next().ok_or("Password is missing")?;

        Ok(Self { username, password })
    }
}

fn main() {
    let env = env_logger::Env::default().filter_or("GARMIN_CONNECT_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let client = Client::new(&config.username.to_string(), &config.password.to_string());

    let activities = client.list_activities();

    info!("Activities: {:#?}", activities);
}
