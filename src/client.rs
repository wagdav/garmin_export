use crate::activity::{Activity, ActivityId};
use crate::error::{Error, Result};
use crate::rate_limiter::RateLimiter;
use log::*;
use reqwest::{blocking, header, redirect, StatusCode};

pub struct Client {
    http: blocking::Client,
    email: String,
    password: String,
    limiter: RateLimiter,
}

const SSO_URL: &str = "https://sso.garmin.com/sso";
const MODERN_URL: &str = "https://connect.garmin.com/modern";
const SIGNIN_URL: &str = "https://sso.garmin.com/sso/signin";

impl Client {
    pub fn new(email: &str, password: &str) -> Result<Self> {
        let http = blocking::Client::builder()
            .redirect(redirect::Policy::none())
            .cookie_store(true)
            .build()?;

        let client = Self {
            http,
            email: email.to_string(),
            password: password.to_string(),
            limiter: RateLimiter::new(),
        };

        client.auth()?;

        Ok(client)
    }

    pub fn list_activities(&self) -> Result<Vec<Activity>> {
        let result = self._list_activities();
        if let Err(Error::Forbidden) = result {
            warn!("Got 403, trying to login again and repeat the query");
            self.auth()?;
            self._list_activities()
        } else {
            result
        }
    }

    fn _list_activities(&self) -> Result<Vec<Activity>> {
        debug!("Listing  activities");
        self.limiter.wait();
        let response = self.http
            .get("https://connect.garmin.com/modern/proxy/activitylist-service/activities/search/activities")
            .query(&[
                ("start", 0.to_string()),
                ("limit", 3.to_string())
            ])
            .send()?;

        if response.status() == StatusCode::OK {
            let activities = response.json().map_err(|_| Error::UnexpectedServerResponse);

            debug!("Activities: {:#?}", activities);
            activities
        } else if response.status() == StatusCode::FORBIDDEN {
            Err(Error::Forbidden)
        } else {
            Err(Error::UnexpectedServerResponse)
        }
    }

    pub fn download_activity(&self, id: ActivityId) -> Result<Vec<u8>> {
        let result = self._download_activity(id);
        if let Err(Error::Forbidden) = result {
            warn!("Got 403, trying to login again and repeat the query");
            self.auth()?;
            self._download_activity(id)
        } else {
            result
        }
    }

    fn _download_activity(&self, id: ActivityId) -> Result<Vec<u8>> {
        debug!("Downloading activity {}", id);
        self.limiter.wait();
        let mut response = self
            .http
            .get(&format!(
                "{}/proxy/download-service/files/activity/{}",
                MODERN_URL, id
            ))
            .send()?;

        if response.status() == StatusCode::OK {
            let mut buf = vec![];
            response.copy_to(&mut buf)?;
            Ok(unzip(&buf)?)
        } else if response.status() == StatusCode::FORBIDDEN {
            Err(Error::Forbidden)
        } else {
            Err(Error::IOError("Something went wrong".to_string()))
        }
    }

    fn auth(&self) -> Result<()> {
        let params = [
            ("clientId", "GarminConnect"),
            ("consumeServiceTicket", "false"),
            ("gauthHost", SSO_URL),
            ("service", MODERN_URL),
        ];

        let data = [
            ("username", self.email.as_str()),
            ("password", self.password.as_str()),
            ("embed", "true"),
            ("lt", "e1s1"),
            ("_eventId", "submit"),
            ("displayNameRequired", "false"),
        ];

        self.limiter.wait();
        let res = self.http.get(SIGNIN_URL).query(&params).send()?;
        res.error_for_status()?;

        self.limiter.wait();
        let res = self
            .http
            .post(SIGNIN_URL)
            .header(header::ORIGIN, "https://sso.garmin.com")
            .query(&params)
            .form(&data)
            .send()?;
        res.error_for_status()?;

        self.limiter.wait();
        let res = self.http.get(MODERN_URL).send()?;
        assert!(res.status().is_redirection());

        let mut next_url = res.headers().get(header::LOCATION).unwrap().clone();
        loop {
            debug!("Redirecting to {:?}", next_url);

            self.limiter.wait();
            let response = self.http.get(next_url.to_str().unwrap()).send()?;

            if response.status().is_success() {
                break;
            }

            if response.status().is_redirection() {
                next_url = response.headers().get(header::LOCATION).unwrap().clone();
            }
        }

        Ok(())
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
