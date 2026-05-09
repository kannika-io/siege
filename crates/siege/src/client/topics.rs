use siege_kernel::KafkaProperties;

use crate::event::EventEmitter;
use crate::kafka::KafkaBackend;
use crate::{SiegeContext, SiegeError, Topic, TopicCreated, TopicDeleted, TopicDetail};

pub struct Topics<'a, C: SiegeContext> {
    ctx: &'a C,
}

impl<'a, C: SiegeContext> Topics<'a, C> {
    pub(crate) fn new(ctx: &'a C) -> Self {
        Self { ctx }
    }

    pub async fn list(&self) -> Result<Vec<Topic>, SiegeError> {
        self.ctx.kafka().list_topics().await
    }

    pub async fn get(&self, name: &str) -> Result<TopicDetail, SiegeError> {
        self.ctx.kafka().get_topic(name).await
    }

    pub async fn create(
        &self,
        name: &str,
        partitions: i32,
        replication_factor: i32,
    ) -> Result<TopicCreated, SiegeError> {
        self.ctx
            .kafka()
            .create_topic(name, partitions, replication_factor)
            .await?;
        let event = TopicCreated {
            topic: Topic {
                name: name.to_owned(),
                partitions,
                replication_factor,
            },
        };
        self.ctx.events().emit(&event);
        Ok(event)
    }

    pub async fn delete(&self, name: &str) -> Result<TopicDeleted, SiegeError> {
        self.ctx.kafka().delete_topic(name).await?;
        let event = TopicDeleted {
            name: name.to_owned(),
        };
        self.ctx.events().emit(&event);
        Ok(event)
    }

    pub async fn update_config(
        &self,
        name: &str,
        config: KafkaProperties,
    ) -> Result<(), SiegeError> {
        self.ctx.kafka().update_topic_config(name, config).await
    }
}
