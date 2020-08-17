use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Activity {
    #[serde(rename(deserialize = "activityId"))]
    id: u64,

    #[serde(rename(deserialize = "activityName"))]
    name: String,

    description: Option<String>,

    #[serde(rename(deserialize = "startTimeGMT"))]
    #[serde(deserialize_with = "datetime::deserialize")]
    start_time_gmt: DateTime<Utc>,
}

mod datetime {
    use super::*;
    use chrono::prelude::*;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn can_parse_json() {
        let activity_json = r#"
        {
          "activityId": 42,
          "activityName": "some name",
          "description": "some description",
          "startTimeGMT": "2020-08-15 10:03:11"
        }
        "#;

        let activity = Activity {
            id: 42,
            name: "some name".to_string(),
            description: Some("some description".to_string()),
            start_time_gmt: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(1597485791, 0),
                Utc,
            ),
        };

        assert_eq!(activity, dbg!(serde_json::from_str(activity_json).unwrap()));
    }
}
