use siege_kernel::KafkaProperties;

use crate::event::{DomainEvent, EventEmitter, TopicDeletedEvent};
use crate::kafka::KafkaBackend;
use crate::{SiegeContext, SiegeError};

impl<C: SiegeContext> std::fmt::Debug for Topic<'_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Topic")
            .field("name", &self.name)
            .field("partitions", &self.partitions)
            .field("replication_factor", &self.replication_factor)
            .finish()
    }
}

pub struct Topic<'a, C: SiegeContext> {
    ctx: &'a C,
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
}

impl<'a, C: SiegeContext> Topic<'a, C> {
    pub(crate) fn new(
        ctx: &'a C,
        name: String,
        partitions: i32,
        replication_factor: i32,
    ) -> Self {
        Self {
            ctx,
            name,
            partitions,
            replication_factor,
        }
    }

    pub async fn config(&self) -> Result<KafkaProperties, SiegeError> {
        let detail = self.ctx.kafka().get_topic(&self.name).await?;
        Ok(detail.config)
    }

    pub async fn delete(self) -> Result<(), SiegeError> {
        self.ctx.kafka().delete_topic(&self.name).await?;
        self.ctx.events().emit(&DomainEvent::TopicDeleted(
            TopicDeletedEvent {
                name: self.name.clone(),
            },
        ));
        Ok(())
    }

    pub async fn update_config(&self, config: KafkaProperties) -> Result<(), SiegeError> {
        self.ctx
            .kafka()
            .update_topic_config(&self.name, config)
            .await
    }
}
