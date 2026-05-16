use crate::event::{DomainEvent, EventEmitter, TopicsSeededEvent};
use crate::kafka::KafkaBackend;
use crate::{SeedBackend, SeedResult, SiegeContext, SiegeError};

pub struct Seed<'a, C: SiegeContext> {
    ctx: &'a C,
}

impl<'a, C: SiegeContext> Seed<'a, C> {
    pub(crate) fn new(ctx: &'a C) -> Self {
        Self { ctx }
    }

    pub async fn seed_topics(&self) -> Result<SeedResult, SiegeError> {
        let result = self.ctx.seeder().seed_topics().await?;
        let mut topics = Vec::new();
        for name in &result.created {
            if let Ok(detail) = self.ctx.kafka().get_topic(name).await {
                topics.push(detail);
            }
        }
        self.ctx.events().emit(&DomainEvent::TopicsSeeded(TopicsSeededEvent { topics }));
        Ok(result)
    }
}
