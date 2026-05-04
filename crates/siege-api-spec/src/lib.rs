use siege_core::{CreateTopicRequest, SiegeError, SseEvent, Topic, TopicConfigUpdate, TopicDetail};
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
        Topic, TopicDetail, CreateTopicRequest, TopicConfigUpdate, SiegeError, SseEvent,
    ))
)]
pub struct ApiDoc;

/// List all topics on the cluster
#[utoipa::path(
    get,
    path = "/api/topics",
    responses(
        (status = 200, description = "List all topics", body = Vec<Topic>)
    )
)]
async fn list_topics() {}

/// Get topic detail
#[utoipa::path(
    get,
    path = "/api/topics/{name}",
    params(("name" = String, Path, description = "Topic name")),
    responses(
        (status = 200, description = "Topic detail", body = TopicDetail),
        (status = 404, description = "Topic not found", body = SiegeError)
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
        (status = 409, description = "Topic already exists", body = SiegeError)
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
        (status = 404, description = "Topic not found", body = SiegeError)
    )
)]
async fn delete_topic() {}

/// Update topic configuration
#[utoipa::path(
    post,
    path = "/api/topics/{name}/config",
    params(("name" = String, Path, description = "Topic name")),
    request_body = TopicConfigUpdate,
    responses(
        (status = 200, description = "Config updated"),
        (status = 404, description = "Topic not found", body = SiegeError)
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

    #[test]
    fn spec_includes_all_schemas() {
        let doc = ApiDoc::openapi();
        let json = doc.to_pretty_json().unwrap();
        for schema in [
            "Topic",
            "TopicDetail",
            "CreateTopicRequest",
            "TopicConfigUpdate",
            "SiegeError",
            "SseEvent",
        ] {
            assert!(json.contains(schema), "Missing schema: {schema}");
        }
    }
}
