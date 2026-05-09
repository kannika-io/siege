pub mod error;
pub mod events;
pub mod types;

pub use error::*;
pub use events::SseEvent;
pub use siege_kernel::KafkaProperties;
pub use types::*;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Siege API",
        version = "0.1.0",
        description = "Kafka topic management API"
    ),
    paths(list_topics, get_topic, create_topic, delete_topic, update_topic_config),
    components(schemas(
        TopicResource, TopicDetailResource, CreateTopicRequest, TopicConfigUpdateRequest,
        ListTopicsError, GetTopicError, CreateTopicError, DeleteTopicError,
        UpdateTopicConfigError, SseEvent,
    ))
)]
pub struct ApiDoc;

/// List all topics on the cluster
#[utoipa::path(
    get,
    path = "/api/topics",
    responses(
        (status = 200, description = "List all topics", body = Vec<TopicResource>),
        (status = 502, description = "Kafka error", body = ListTopicsError)
    )
)]
async fn list_topics() {}

/// Get topic detail
#[utoipa::path(
    get,
    path = "/api/topics/{name}",
    params(("name" = String, Path, description = "Topic name")),
    responses(
        (status = 200, description = "Topic detail", body = TopicDetailResource),
        (status = 404, description = "Topic not found", body = GetTopicError)
    )
)]
async fn get_topic() {}

/// Create a topic
#[utoipa::path(
    post,
    path = "/api/topics",
    request_body = CreateTopicRequest,
    responses(
        (status = 201, description = "Topic created"),
        (status = 409, description = "Topic already exists", body = CreateTopicError)
    )
)]
async fn create_topic() {}

/// Delete a topic
#[utoipa::path(
    delete,
    path = "/api/topics/{name}",
    params(("name" = String, Path, description = "Topic name")),
    responses(
        (status = 204, description = "Topic deleted"),
        (status = 404, description = "Topic not found", body = DeleteTopicError)
    )
)]
async fn delete_topic() {}

/// Update topic configuration
#[utoipa::path(
    post,
    path = "/api/topics/{name}/config",
    params(("name" = String, Path, description = "Topic name")),
    request_body = TopicConfigUpdateRequest,
    responses(
        (status = 200, description = "Config updated"),
        (status = 404, description = "Topic not found", body = UpdateTopicConfigError)
    )
)]
async fn update_topic_config() {}

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn spec_generates_valid_json() {
        let doc = ApiDoc::openapi();
        let json = doc.to_pretty_json().unwrap();
        assert!(json.contains("\"title\": \"Siege API\""));
        assert!(json.contains("/api/topics"));
        assert!(json.contains("/api/topics/{name}"));
        assert!(json.contains("/api/topics/{name}/config"));
    }
}
