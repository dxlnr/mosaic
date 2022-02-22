use serde_json::json;
use serde::{Deserialize, Serialize};

use reqwest::Client;

use crate::{rest::stats::Stats, settings::JobSettings};

pub struct HttpClient {
    pub client: Client,
    pub settings: JobSettings,
}

// #[async_trait]
impl HttpClient {
    pub fn new(settings: JobSettings) -> Self {
        Self {client: Client::new(), settings }
    }

    pub async fn release_stats(&mut self, stats: &Stats) -> Result<(), String> {
        let res_json = JobStats::new(self.settings.job_id, self.settings.job_token.clone(), stats.clone());
        let _ = self.client.post(&self.settings.route).json(&json!(res_json)).send().await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobStats {
    job_id: u32,
    job_token: String,
    stats: Stats,
}

impl JobStats {
    pub fn new(job_id: u32, job_token: String, stats: Stats) -> Self {
        Self { job_id, job_token, stats }
    }
}