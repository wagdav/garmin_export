use crate::activity::Activity;
use crate::error::{Error, Result};
use log::debug;
use regex::Regex;

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RequestFailed(error.to_string())
    }
}

pub struct Client {
    http: reqwest::blocking::Client,
}

impl Client {
    pub fn new(email: &str, password: &str) -> Result<Self> {
        let http = auth(email, password)?;
        Ok(Self { http })
    }

    pub fn list_activities(&self) -> Result<Vec<Activity>> {
        self.http
            .get("https://connect.garmin.com/modern/proxy/activitylist-service/activities/search/activities")
            .query(&[
                ("start", 0.to_string()),
                ("limit", 5.to_string())
            ])
            .send()?
            .json()
            .map_err(|_| Error::UnexpectedServerResponse)
    }
}

fn auth(username: &str, password: &str) -> Result<reqwest::blocking::Client> {
    let http = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .build()?;

    let res = http
        .post("https://sso.garmin.com/sso/signin")
        .header("origin", "https://sso.garmin.com")
        .query(&[("service", "https://connect.garmin.com/modern")])
        .form(&[
            ("username", username),
            ("password", password),
            ("embed", &false.to_string()),
        ])
        .send()?;

    debug!("Claiming the authentication toket");
    let ticket = extract_ticket_url(&res.text()?)?;
    let res = http
        .get("https://connect.garmin.com/modern")
        .query(&[("ticket", ticket)])
        .send()?;

    assert_eq!(res.status(), 200);

    debug!("Pinging legacy endpoint");
    http.get("https://connect.garmin.com/legacy/session")
        .send()?;

    Ok(http)
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
    fn parse_ticket() {
        let auth_response = r#"response_url = "https:\/\/connect.garmin.com\/modern?ticket=ST-0123456-aBCDefgh1iJkLmN5opQ9R-cas";"#;
        assert_eq!(
            extract_ticket_url(auth_response),
            Ok("ST-0123456-aBCDefgh1iJkLmN5opQ9R-cas".to_string())
        )
    }
}
