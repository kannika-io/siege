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
    SeedProgress {
        topic: String,
        topic_index: u32,
        total_topics: u32,
        records_generated: u32,
        total_records: u32,
    },
    ChaosTopicDeleted { topic: String },
    ChaosRetentionLowered { topic: TopicResource },
    ChaosCleanupPolicyFlipped { topic: TopicResource },
    ChaosPartitionsIncreased { topic: TopicResource },
    ChaosPoisonPillsSent { topic: String, count: u32 },
    ChaosSchemaBreakSent { topic: String, count: u32 },
}
