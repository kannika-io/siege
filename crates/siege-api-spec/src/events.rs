use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::TopicResource;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    TopicsSnapshot { topics: Vec<TopicResource> },
    TopicCreated { topic: TopicResource },
    TopicUpdated { topic: TopicResource },
    TopicDeleted { name: String },
    TopicsSeeded { topics: Vec<TopicResource> },
    ChaosTopicDeleted { topic: String },
    ChaosRetentionZeroed { topic: TopicResource },
    ChaosCleanupPolicyFlipped { topic: TopicResource },
    ChaosPartitionsIncreased { topic: TopicResource },
    ChaosPoisonPillsSent { topic: String, count: u32 },
    ChaosSchemaBreakSent { topic: String, count: u32 },
}
