use crate::error::{Error, Result};
use std::env;

pub struct Config {
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self> {
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
