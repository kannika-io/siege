use siege_api_spec::{TopicConfigUpdateRequest, TopicDetailResource};

use crate::{ClientError, SiegeClient};

pub struct Topic<'a> {
    pub(crate) client: &'a SiegeClient,
    pub(crate) name: String,
}

impl Topic<'_> {
    pub async fn get(&self) -> Result<TopicDetailResource, ClientError> {
        let resp = self
            .client
            .http()
            .get(format!("{}/api/topics/{}", self.client.base_url(), self.name))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            Err(SiegeClient::api_error(resp).await)
        }
    }

    pub async fn delete(&self) -> Result<(), ClientError> {
        let resp = self
            .client
            .http()
            .delete(format!("{}/api/topics/{}", self.client.base_url(), self.name))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(SiegeClient::api_error(resp).await)
        }
    }

    pub async fn update_config(
        &self,
        config: &TopicConfigUpdateRequest,
    ) -> Result<(), ClientError> {
        let resp = self
            .client
            .http()
            .post(format!(
                "{}/api/topics/{}/config",
                self.client.base_url(),
                self.name
            ))
            .json(config)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(SiegeClient::api_error(resp).await)
        }
    }
}
