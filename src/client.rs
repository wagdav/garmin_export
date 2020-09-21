use crate::activity::{Activity, ActivityId};
use crate::error::{Error, Result};
use crate::rate_limiter::RateLimiter;
use log::*;
use reqwest::{blocking, header, redirect};

pub struct Client {
    http: blocking::Client,
    email: String,
    password: String,
    limiter: RateLimiter,
}

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
        self.retry(|| {
            debug!("Listing  activities");
            self.limiter.wait();
            let response = self.http
            .get("https://connect.garmin.com/modern/proxy/activitylist-service/activities/search/activities")
            .query(&[
                ("start", 0.to_string()),
                ("limit", 3.to_string())
            ])
            .send()?
            .error_for_status()?;

            let activities = response.json().map_err(|error| {
                Error::APIError(format!(
                    "Cannot decode the activity list response: {}",
                    error
                ))
            });

            debug!("Activities: {:#?}", activities);
            activities
        })
    }

    pub fn download_activity(&self, id: ActivityId) -> Result<Vec<u8>> {
        self.retry(|| {
            debug!("Downloading activity {}", id);
            self.limiter.wait();
            let mut response = self
                .http
                .get(&format!(
                    "{}/proxy/download-service/files/activity/{}",
                    MODERN_URL, id
                ))
                .send()?
                .error_for_status()?;

            let mut buf = vec![];
            response.copy_to(&mut buf)?;

            Ok(unzip(&buf)?)
        })
    }

    fn auth(&self) -> Result<()> {
        let params = [("service", MODERN_URL)];

        let data = [
            ("username", self.email.as_str()),
            ("password", self.password.as_str()),
            ("embed", "false"),
        ];

        self.limiter.wait();
        self.http
            .get(SIGNIN_URL)
            .query(&params)
            .send()?
            .error_for_status()?;

        self.limiter.wait();
        self.http
            .post(SIGNIN_URL)
            .header(header::ORIGIN, "https://sso.garmin.com")
            .query(&params)
            .form(&data)
            .send()?
            .error_for_status()?;

        self.limiter.wait();
        let res = self.http.get(MODERN_URL).send()?.error_for_status()?;
        assert!(res.status().is_redirection());

        let mut next_url = res.headers().get(header::LOCATION).unwrap().clone();
        loop {
            debug!("Redirecting to {:?}", next_url);

            self.limiter.wait();
            let response = self
                .http
                .get(next_url.to_str().unwrap())
                .send()?
                .error_for_status()?;

            if response.status().is_success() {
                break;
            }

            if response.status().is_redirection() {
                next_url = response.headers().get(header::LOCATION).unwrap().clone();
            }
        }

        Ok(())
    }

    fn retry<F, R>(&self, action: F) -> Result<R>
    where
        F: Fn() -> Result<R>,
    {
        let mut trials = 3;
        loop {
            let result = action();
            trials -= 1;

            match result {
                Err(Error::Forbidden) if trials > 0 => {
                    warn!("Got 403, trying to login again and repeat the query");
                    self.auth()?;
                }
                _ => return result,
            }
        }
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
