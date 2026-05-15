use siege_api_spec::ChaosResult;
use thiserror::Error;

use crate::topic::Topic;

#[derive(Debug, Error)]
pub enum ChaosError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("chaos API error ({status}): {body}")]
    Api { status: u16, body: String },
}

#[allow(async_fn_in_trait)] // intentional: WASM futures are not Send
pub trait ChaosExt {
    async fn low_retention(&self) -> Result<ChaosResult, ChaosError>;
    async fn flip_cleanup_policy(&self) -> Result<ChaosResult, ChaosError>;
    async fn increase_partitions(&self, partitions: i32) -> Result<ChaosResult, ChaosError>;
    async fn poison_pills(&self, count: u32) -> Result<ChaosResult, ChaosError>;
    async fn schema_break(&self, count: u32) -> Result<ChaosResult, ChaosError>;
}

impl ChaosExt for Topic<'_> {
    async fn low_retention(&self) -> Result<ChaosResult, ChaosError> {
        self.chaos_post("low-retention", serde_json::json!({ "topic": self.name }))
            .await
    }

    async fn flip_cleanup_policy(&self) -> Result<ChaosResult, ChaosError> {
        self.chaos_post(
            "flip-cleanup-policy",
            serde_json::json!({ "topic": self.name }),
        )
        .await
    }

    async fn increase_partitions(&self, partitions: i32) -> Result<ChaosResult, ChaosError> {
        self.chaos_post(
            "increase-partitions",
            serde_json::json!({ "topic": self.name, "partitions": partitions }),
        )
        .await
    }

    async fn poison_pills(&self, count: u32) -> Result<ChaosResult, ChaosError> {
        self.chaos_post(
            "poison-pills",
            serde_json::json!({ "topic": self.name, "count": count }),
        )
        .await
    }

    async fn schema_break(&self, count: u32) -> Result<ChaosResult, ChaosError> {
        self.chaos_post(
            "schema-break",
            serde_json::json!({ "topic": self.name, "count": count }),
        )
        .await
    }
}

impl Topic<'_> {
    pub(crate) async fn chaos_post(
        &self,
        action: &str,
        body: serde_json::Value,
    ) -> Result<ChaosResult, ChaosError> {
        let resp = self
            .client
            .http()
            .post(format!("{}/api/chaos/{action}", self.client.base_url()))
            .json(&body)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            Err(ChaosError::Api { status, body })
        }
    }
}
