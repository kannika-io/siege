use siege::kafka::BoxFuture;
use siege::schema_registry::{SchemaId, SchemaRegistryBackend};
use siege::SiegeError;

pub struct SchemaRegistryClient {
    base_url: String,
    client: reqwest::Client,
}

impl SchemaRegistryClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }
}

impl SchemaRegistryBackend for SchemaRegistryClient {
    fn register_schema(
        &self,
        subject: &str,
        schema: &str,
    ) -> BoxFuture<'_, Result<SchemaId, SiegeError>> {
        let url = format!("{}/subjects/{}/versions", self.base_url, subject);
        let body = serde_json::json!({
            "schemaType": "AVRO",
            "schema": schema,
        });
        Box::pin(async move {
            let resp = self
                .client
                .post(&url)
                .header("Content-Type", "application/vnd.schemaregistry.v1+json")
                .json(&body)
                .send()
                .await
                .map_err(|e| SiegeError::SchemaRegistry(e.to_string()))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(SiegeError::SchemaRegistry(
                    format!("HTTP {status}: {body}"),
                ));
            }

            let parsed: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| SiegeError::SchemaRegistry(e.to_string()))?;

            let id = parsed["id"]
                .as_i64()
                .ok_or_else(|| SiegeError::SchemaRegistry("missing 'id' in response".into()))?;

            Ok(SchemaId(id as i32))
        })
    }

    fn delete_subject(
        &self,
        subject: &str,
    ) -> BoxFuture<'_, Result<(), SiegeError>> {
        let url = format!(
            "{}/subjects/{}?permanent=true",
            self.base_url, subject
        );
        Box::pin(async move {
            let resp = self
                .client
                .delete(&url)
                .send()
                .await
                .map_err(|e| SiegeError::SchemaRegistry(e.to_string()))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(SiegeError::SchemaRegistry(
                    format!("HTTP {status}: {body}"),
                ));
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_schema_sends_post() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/subjects/test-topic-value/versions")
            .match_header("content-type", "application/vnd.schemaregistry.v1+json")
            .with_status(200)
            .with_body(r#"{"id":42}"#)
            .create_async()
            .await;

        let client = SchemaRegistryClient::new(&server.url());
        let schema = r#"{"type":"record","name":"Test","fields":[]}"#;
        let result = client.register_schema("test-topic-value", schema).await;

        let id = result.expect("register_schema should succeed");
        assert_eq!(id, SchemaId(42));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn delete_subject_sends_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/subjects/test-topic-value?permanent=true")
            .with_status(200)
            .with_body("[1]")
            .create_async()
            .await;

        let client = SchemaRegistryClient::new(&server.url());
        let result = client.delete_subject("test-topic-value").await;

        result.expect("delete_subject should succeed");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn register_schema_maps_http_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/subjects/bad-value/versions")
            .with_status(500)
            .with_body(r#"{"error_code":50001,"message":"Error in the backend"}"#)
            .create_async()
            .await;

        let client = SchemaRegistryClient::new(&server.url());
        let result = client.register_schema("bad-value", "{}").await;

        assert!(result.is_err());
    }
}
