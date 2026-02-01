pub mod parse_versions {
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum VersionError {
        #[error("HTTP request failed: {0}")]
        HttpError(#[from] reqwest::Error),

        #[error("JSON parsing failed: {0}")]
        JsonError(#[from] serde_json::Error),

        #[error("Version not found: {0}")]
        VersionNotFound(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LatestVersions {
        pub release: String,
        pub snapshot: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VersionInfo {
        pub id: String,
        #[serde(rename = "type")]
        pub version_type: String,
        pub url: String,
        pub time: String,
        pub release_time: String,
        pub sha1: String,
        pub compliance_level: i32,
    }
}
