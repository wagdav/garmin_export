use crate::activity::{Activity, ActivityId};
use crate::error::{Error, Result};
use log::*;
use regex::Regex;

pub struct Client {
    http: reqwest::blocking::Client,
}

const BASE_URL: &str = "https://connect.garmin.com";
const SSO_URL: &str = "https://sso.garmin.com/sso";
const MODERN_URL: &str = "https://connect.garmin.com/modern";
const SIGNIN_URL: &str = "https://sso.garmin.com/sso/signin";

impl Client {
    pub fn new(email: &str, password: &str) -> Result<Self> {
        use reqwest::{blocking, header, redirect};

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ORIGIN,
            header::HeaderValue::from_static("https://sso.garmin.com"),
        );

        let http = blocking::Client::builder()
            .redirect(redirect::Policy::limited(20))
            .default_headers(headers)
            .cookie_store(true)
            .build()?;

        let client = Self { http };

        client.auth(email, password)?;

        Ok(client)
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
            Ok(unzip(&buf)?)
        } else {
            Err(Error::IOError("Something went wrong".to_string()))
        }
    }

    fn auth(&self, username: &str, password: &str) -> Result<()> {
        let params = [
            ("service", MODERN_URL),
            ("webhost", BASE_URL),
            ("source", SIGNIN_URL),
            ("redirectAfterAccountLoginUrl", MODERN_URL),
            ("redirectAfterAccountCreationUrl", MODERN_URL),
            ("gauthHost", SSO_URL),
            ("locale", "en_US"),
            ("id", "gauth-widget"),
            (
                "cssUrl",
                "https://static.garmincdn.com/com.garmin.connect/ui/css/gauth-custom-v1.2-min.css",
            ),
            ("clientId", "GarminConnect"),
            ("rememberMeShown", "true"),
            ("rememberMeChecked", "false"),
            ("createAccountShown", "true"),
            ("openCreateAccount", "false"),
            ("usernameShown", "false"),
            ("displayNameShown", "false"),
            ("consumeServiceTicket", "false"),
            ("initialFocus", "true"),
            ("embedWidget", "false"),
            ("generateExtraServiceTicket", "false"),
        ];

        let res = self.http
            .post(SIGNIN_URL)
            .query(&params)
            .form(&[
                ("username", username),
                ("password", password),
                ("embed", &true.to_string()),
                ("lt", "e1s1"),
                ("_eventid", "submit"),
                ("_displayNameRequired", &false.to_string()),
            ])
            .send()?;

        assert_eq!(res.status(), 200);

        let ticket = extract_ticket_url(&res.text()?)?;

        debug!("Claiming the authentication token at {}", ticket);
        let res = self.http.get(&ticket).send()?;

        assert_eq!(res.status(), 200);

        Ok(())
    }
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
fn unzip(buf: &Vec<u8>) -> Result<Vec<u8>> {
    use std::io::prelude::*;

    let reader = std::io::Cursor::new(buf);
    let mut zip = zip::ZipArchive::new(reader)?;

    assert_eq!(zip.len(), 1, "Exactly one file is expected in the archive");
    let mut file = zip.by_index(0)?;

    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}
