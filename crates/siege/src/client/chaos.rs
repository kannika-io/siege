use crate::event::*;
use crate::kafka::KafkaBackend;
use crate::{ChaosBackend, SiegeContext, SiegeError};

pub struct Chaos<'a, C: SiegeContext> {
    ctx: &'a C,
}

impl<'a, C: SiegeContext> Chaos<'a, C> {
    pub(crate) fn new(ctx: &'a C) -> Self {
        Self { ctx }
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<(), SiegeError> {
        self.ctx.chaos().delete_topic(topic).await.map_err(|e| SiegeError::Chaos(e.to_string()))?;
        self.ctx.events().emit(&DomainEvent::ChaosTopicDeleted(ChaosTopicDeletedEvent {
            topic: topic.to_owned(),
        }));
        Ok(())
    }

    pub async fn low_retention(&self, topic: &str) -> Result<(), SiegeError> {
        self.ctx.chaos().low_retention(topic).await.map_err(|e| SiegeError::Chaos(e.to_string()))?;
        let detail = self.ctx.kafka().get_topic(topic).await?;
        self.ctx.events().emit(&DomainEvent::ChaosRetentionLowered(ChaosRetentionLoweredEvent { detail }));
        Ok(())
    }

    pub async fn flip_cleanup_policy(&self, topic: &str) -> Result<(), SiegeError> {
        self.ctx.chaos().flip_cleanup_policy(topic).await.map_err(|e| SiegeError::Chaos(e.to_string()))?;
        let detail = self.ctx.kafka().get_topic(topic).await?;
        self.ctx.events().emit(&DomainEvent::ChaosCleanupPolicyFlipped(ChaosCleanupPolicyFlippedEvent { detail }));
        Ok(())
    }

    pub async fn increase_partitions(&self, topic: &str, extra: i32) -> Result<(), SiegeError> {
        self.ctx.chaos().increase_partitions(topic, extra).await.map_err(|e| SiegeError::Chaos(e.to_string()))?;
        let detail = self.ctx.kafka().get_topic(topic).await?;
        self.ctx.events().emit(&DomainEvent::ChaosPartitionsIncreased(ChaosPartitionsIncreasedEvent { detail }));
        Ok(())
    }

    pub async fn poison_pills(&self, topic: &str, count: u32) -> Result<(), SiegeError> {
        self.ctx.chaos().poison_pills(topic, count).await.map_err(|e| SiegeError::Chaos(e.to_string()))?;
        self.ctx.events().emit(&DomainEvent::ChaosPoisonPillsSent(ChaosPoisonPillsSentEvent {
            topic: topic.to_owned(),
            count,
        }));
        Ok(())
    }

    pub async fn schema_break(&self, topic: &str, count: u32) -> Result<(), SiegeError> {
        self.ctx.chaos().schema_break(topic, count).await.map_err(|e| SiegeError::Chaos(e.to_string()))?;
        self.ctx.events().emit(&DomainEvent::ChaosSchemaBreakSent(ChaosSchemaBreakSentEvent {
            topic: topic.to_owned(),
            count,
        }));
        Ok(())
    }
}
