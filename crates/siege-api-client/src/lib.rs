pub use siege_api_spec::{
    CreateTopicRequest, KafkaProperties, SiegeError, SseEvent, Topic, TopicConfigUpdate,
    TopicDetail,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(SiegeError),
}

#[derive(Clone)]
pub struct SiegeClient {
    base_url: String,
    client: reqwest::Client,
}

impl SiegeClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn list_topics(&self) -> Result<Vec<Topic>, ClientError> {
        let resp = self
            .client
            .get(format!("{}/api/topics", self.base_url))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            Err(ClientError::Api(resp.json().await?))
        }
    }

    pub async fn get_topic(&self, name: &str) -> Result<TopicDetail, ClientError> {
        let resp = self
            .client
            .get(format!("{}/api/topics/{name}", self.base_url))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            Err(ClientError::Api(resp.json().await?))
        }
    }

    pub async fn create_topic(&self, req: &CreateTopicRequest) -> Result<(), ClientError> {
        let resp = self
            .client
            .post(format!("{}/api/topics", self.base_url))
            .json(req)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ClientError::Api(resp.json().await?))
        }
    }

    pub async fn delete_topic(&self, name: &str) -> Result<(), ClientError> {
        let resp = self
            .client
            .delete(format!("{}/api/topics/{name}", self.base_url))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ClientError::Api(resp.json().await?))
        }
    }

    pub async fn update_topic_config(
        &self,
        name: &str,
        config: &TopicConfigUpdate,
    ) -> Result<(), ClientError> {
        let resp = self
            .client
            .post(format!("{}/api/topics/{name}/config", self.base_url))
            .json(config)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ClientError::Api(resp.json().await?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let client = SiegeClient::new("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn client_strips_trailing_slash() {
        let client = SiegeClient::new("http://localhost:8080/");
        assert_eq!(client.base_url, "http://localhost:8080");
    }
}
