use crate::components::ui::icon::IconName;
use siege_api_client::ChaosExt;

#[derive(Clone, Copy, PartialEq)]
pub enum ChaosAction {
    Delete,
    LowRetention,
    FlipCleanupPolicy,
    IncreasePartitions,
    PoisonPills,
    SchemaBreak,
}

impl ChaosAction {
    pub const ALL: &[Self] = &[
        Self::Delete,
        Self::LowRetention,
        Self::FlipCleanupPolicy,
        Self::IncreasePartitions,
        Self::PoisonPills,
        Self::SchemaBreak,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Delete => "Delete",
            Self::LowRetention => "Low retention",
            Self::FlipCleanupPolicy => "Flip cleanup",
            Self::IncreasePartitions => "Add partition",
            Self::PoisonPills => "Poison pills",
            Self::SchemaBreak => "Schema break",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Self::Delete => IconName::Skull,
            Self::LowRetention => IconName::Hourglass,
            Self::FlipCleanupPolicy => IconName::Swords,
            Self::IncreasePartitions => IconName::Shield,
            Self::PoisonPills => IconName::Flask,
            Self::SchemaBreak => IconName::Zap,
        }
    }

    pub fn is_destructive(self) -> bool {
        matches!(self, Self::Delete)
    }

    pub fn success_message(self, name: &str) -> String {
        match self {
            Self::Delete => format!("Deleted topic '{name}'"),
            Self::LowRetention => format!("Set retention to 1ms for '{name}'"),
            Self::FlipCleanupPolicy => format!("Flipped cleanup policy for '{name}'"),
            Self::IncreasePartitions => format!("Increased partitions for '{name}'"),
            Self::PoisonPills => format!("Sent 10 poison pills to '{name}'"),
            Self::SchemaBreak => format!("Sent 10 schema-breaking messages to '{name}'"),
        }
    }

    pub async fn execute(self, topic: &siege_api_client::Topic<'_>) -> Result<(), String> {
        match self {
            Self::Delete => topic.delete().await.map_err(|e| e.to_string()),
            Self::LowRetention => topic.low_retention().await.map(|_| ()).map_err(|e| e.to_string()),
            Self::FlipCleanupPolicy => topic.flip_cleanup_policy().await.map(|_| ()).map_err(|e| e.to_string()),
            Self::IncreasePartitions => topic.increase_partitions(1).await.map(|_| ()).map_err(|e| e.to_string()),
            Self::PoisonPills => topic.poison_pills(10).await.map(|_| ()).map_err(|e| e.to_string()),
            Self::SchemaBreak => topic.schema_break(10).await.map(|_| ()).map_err(|e| e.to_string()),
        }
    }
}
