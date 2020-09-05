use crate::activity::{Activity, ActivityId};
use crate::error::{Error, Result};
use log::*;
use regex::Regex;

pub struct Client {
    http: reqwest::blocking::Client,
}

impl Client {
    pub fn new(email: &str, password: &str) -> Result<Self> {
        let http = auth(email, password)?;
        Ok(Self { http })
    }

    pub fn list_activities(&self) -> Result<Vec<Activity>> {
        debug!("Listing  activities");
        let response = self.http
            .get("https://connect.garmin.com/modern/proxy/activitylist-service/activities/search/activities")
            .query(&[
                ("start", 0.to_string()),
                ("limit", 5.to_string())
            ])
            .send()?;

        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let activities = response.json().map_err(|_| Error::UnexpectedServerResponse);

        debug!("Activities: {:#?}", activities);
        activities
    }

    pub fn download_activity(&self, id: ActivityId) -> Result<Vec<u8>> {
        debug!("Downloading activity {}", id);
        let mut response = self
            .http
            .get(&format!(
                "https://connect.garmin.com/modern/proxy/download-service/files/activity/{}",
                id
            ))
            .send()?;

        if response.status() == reqwest::StatusCode::OK {
            let mut buf = vec![];
            response.copy_to(&mut buf)?;
            Ok(unzip(buf))
        } else {
            Err(Error::IOError("Something went wrong".to_string()))
        }
    }
}

fn auth(username: &str, password: &str) -> Result<reqwest::blocking::Client> {
    let http = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(20))
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
    let res = http.get(&ticket).send()?;

    assert_eq!(res.status(), 200);

    Ok(http)
}

fn extract_ticket_url(auth_response: &str) -> Result<String> {
    let re = Regex::new(r#"response_url\s*=\s*"(https:[^"]+)""#).unwrap();

    let matches = re
        .captures_iter(auth_response)
        .next()
        .ok_or(Error::InvalidInput(
            "Cannot extract the ticket url".to_string(),
        ))?;

    Ok(matches[1].to_string().replace("\\", ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ticket() {
        let auth_response = r#"response_url = "https:\/\/connect.garmin.com\/modern?ticket=ST-0123456-aBCDefgh1iJkLmN5opQ9R-cas";"#;
        assert_eq!(
            extract_ticket_url(auth_response),
            Ok(
                "https://connect.garmin.com/modern?ticket=ST-0123456-aBCDefgh1iJkLmN5opQ9R-cas"
                    .to_string()
            )
        )
    }
}

/// Decompress the contents of the provided buffer
fn unzip(buf: Vec<u8>) -> Vec<u8> {
    use std::io::prelude::*;

    let reader = std::io::Cursor::new(buf);
    let mut zip = zip::ZipArchive::new(reader).unwrap();

    assert_eq!(zip.len(), 1);
    let mut file = zip.by_index(0).unwrap();

    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();

    buffer
}
