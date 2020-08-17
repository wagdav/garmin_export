use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Activity {
    #[serde(rename(deserialize = "activityId"))]
    id: u64,

    #[serde(rename(deserialize = "activityName"))]
    name: String,

    description: Option<String>,
}
