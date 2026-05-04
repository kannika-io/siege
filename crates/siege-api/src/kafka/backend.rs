use std::future::Future;

use siege_core::{CreateTopicRequest, SiegeError, Topic, TopicConfigUpdate, TopicDetail};

pub trait KafkaBackend: Send + Sync + 'static {
    fn list_topics(&self) -> impl Future<Output = Result<Vec<Topic>, SiegeError>> + Send;
    fn get_topic(&self, name: &str) -> impl Future<Output = Result<TopicDetail, SiegeError>> + Send;
    fn create_topic(
        &self,
        req: CreateTopicRequest,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send;
    fn delete_topic(&self, name: &str) -> impl Future<Output = Result<(), SiegeError>> + Send;
    fn update_topic_config(
        &self,
        name: &str,
        config: TopicConfigUpdate,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send;
}
