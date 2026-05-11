use siege_kernel::KafkaProperties;

use crate::event::{DomainEvent, EventEmitter, TopicCreatedEvent, TopicDeletedEvent};
use crate::kafka::KafkaBackend;
use crate::{SiegeContext, SiegeError, Topic};

pub struct Topics<'a, C: SiegeContext> {
    ctx: &'a C,
}

impl<'a, C: SiegeContext> Topics<'a, C> {
    pub(crate) fn new(ctx: &'a C) -> Self {
        Self { ctx }
    }

    pub async fn list(&self) -> Result<Vec<Topic<'a, C>>, SiegeError> {
        let metas = self.ctx.kafka().list_topics().await?;
        Ok(metas
            .into_iter()
            .map(|m| Topic::new(self.ctx, m.name, m.partitions, m.replication_factor))
            .collect())
    }

    pub async fn get(&self, name: &str) -> Result<Topic<'a, C>, SiegeError> {
        let detail = self.ctx.kafka().get_topic(name).await?;
        Ok(Topic::new(
            self.ctx,
            detail.name,
            detail.partitions,
            detail.replication_factor,
        ))
    }

    pub async fn create(
        &self,
        name: &str,
        partitions: i32,
        replication_factor: i32,
        config: KafkaProperties,
    ) -> Result<Topic<'a, C>, SiegeError> {
        self.ctx
            .kafka()
            .create_topic(name, partitions, replication_factor, config)
            .await?;
        self.ctx.events().emit(&DomainEvent::TopicCreated(
            TopicCreatedEvent {
                name: name.to_owned(),
                partitions,
                replication_factor,
            },
        ));
        Ok(Topic::new(
            self.ctx,
            name.to_owned(),
            partitions,
            replication_factor,
        ))
    }

    pub async fn delete(&self, name: &str) -> Result<(), SiegeError> {
        self.ctx.kafka().delete_topic(name).await?;
        self.ctx.events().emit(&DomainEvent::TopicDeleted(
            TopicDeletedEvent {
                name: name.to_owned(),
            },
        ));
        Ok(())
    }

    pub async fn update_config(
        &self,
        name: &str,
        config: KafkaProperties,
    ) -> Result<(), SiegeError> {
        self.ctx
            .kafka()
            .update_topic_config(name, config)
            .await
    }
}
