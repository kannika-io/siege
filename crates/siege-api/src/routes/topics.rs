use actix_web::{web, HttpResponse};
use futures::StreamExt;
use siege::SiegeContext;
use siege_api_spec::{CreateTopicRequest, SseEvent, TopicConfigUpdateRequest, TopicResource};

use crate::error::HttpError;
use crate::mapping;
use crate::sse::broadcaster::Broadcaster;

pub async fn list_topics<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
) -> Result<HttpResponse, HttpError> {
    let topics = client.topics().list().await?;
    let resources: Vec<TopicResource> = topics.into_iter().map(mapping::topic_to_resource).collect();
    Ok(HttpResponse::Ok().json(resources))
}

pub async fn get_topic<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    path: web::Path<String>,
) -> Result<HttpResponse, HttpError> {
    let name = path.into_inner();
    let detail = client.topics().get(&name).await?;
    Ok(HttpResponse::Ok().json(mapping::detail_to_resource(detail)))
}

pub async fn create_topic<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<CreateTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client
        .topics()
        .create(&req.name, req.partitions, req.replication_factor, req.config)
        .await?;
    Ok(HttpResponse::Created().finish())
}

pub async fn delete_topic<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    path: web::Path<String>,
) -> Result<HttpResponse, HttpError> {
    let name = path.into_inner();
    client.topics().delete(&name).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_topic_config<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    path: web::Path<String>,
    body: web::Json<TopicConfigUpdateRequest>,
) -> Result<HttpResponse, HttpError> {
    let name = path.into_inner();
    client
        .topics()
        .update_config(&name, body.into_inner().config)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn events<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    broadcaster: web::Data<Broadcaster>,
) -> HttpResponse {
    let topics = client.topics().list().await.unwrap_or_default();
    let resources: Vec<TopicResource> =
        topics.into_iter().map(mapping::topic_to_resource).collect();
    let rx = broadcaster.subscribe();

    let snapshot_data =
        serde_json::to_string(&SseEvent::TopicsSnapshot { topics: resources }).unwrap();

    let initial = futures::stream::once(async move {
        Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(format!(
            "data: {snapshot_data}\n\n"
        )))
    });

    let updates = tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async {
        match result {
            Ok(event) => {
                let data = serde_json::to_string(&event).ok()?;
                Some(Ok(actix_web::web::Bytes::from(format!("data: {data}\n\n"))))
            }
            Err(_) => None,
        }
    });

    HttpResponse::Ok()
        .insert_header(("content-type", "text/event-stream"))
        .insert_header(("cache-control", "no-cache"))
        .streaming(initial.chain(updates))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::http::StatusCode;
    use actix_web::{test, App};
    use siege::{KafkaProperties, SiegeContext, TopicDetail};

    use crate::kafka::mock::MockKafkaBackend;
    use crate::routes::configure;
    use crate::sse::broadcaster::Broadcaster;

    use super::*;

    struct MockContext {
        kafka: MockKafkaBackend,
        events: Broadcaster,
    }

    impl SiegeContext for MockContext {
        type Kafka = MockKafkaBackend;
        type Events = Broadcaster;

        fn kafka(&self) -> &MockKafkaBackend {
            &self.kafka
        }

        fn events(&self) -> &Broadcaster {
            &self.events
        }
    }

    fn mock_ctx(kafka: MockKafkaBackend) -> MockContext {
        MockContext {
            kafka,
            events: Broadcaster::new(16),
        }
    }

    fn sample_detail(name: &str) -> TopicDetail {
        TopicDetail {
            name: name.into(),
            partitions: 3,
            replication_factor: 1,
            config: KafkaProperties::new(),
        }
    }

    fn test_app(
        ctx: MockContext,
    ) -> (
        web::Data<siege::client::Client<MockContext>>,
        web::Data<Broadcaster>,
    ) {
        let broadcaster = web::Data::new(ctx.events.clone());
        let client = web::Data::new(siege::client::Client::new(ctx));
        (client, broadcaster)
    }

    #[actix_web::test]
    async fn test_list_topics() {
        let (client, broadcaster) = test_app(mock_ctx(MockKafkaBackend::with_topics(vec![
            sample_detail("topic-a"),
            sample_detail("topic-b"),
        ])));
        let app = test::init_service(
            App::new()
                .app_data(client)
                .app_data(broadcaster)
                .configure(configure::<MockContext>),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/topics").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<TopicResource> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);
    }

    #[actix_web::test]
    async fn test_get_topic_found() {
        let (client, broadcaster) = test_app(mock_ctx(MockKafkaBackend::with_topics(vec![
            sample_detail("my-topic"),
        ])));
        let app = test::init_service(
            App::new()
                .app_data(client)
                .app_data(broadcaster)
                .configure(configure::<MockContext>),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/topics/my-topic")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: siege_api_spec::TopicDetailResource = test::read_body_json(resp).await;
        assert_eq!(body.name, "my-topic");
    }

    #[actix_web::test]
    async fn test_get_topic_not_found() {
        let (client, broadcaster) = test_app(mock_ctx(MockKafkaBackend::new()));
        let app = test::init_service(
            App::new()
                .app_data(client)
                .app_data(broadcaster)
                .configure(configure::<MockContext>),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/topics/nope")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_create_topic() {
        let (client, broadcaster) = test_app(mock_ctx(MockKafkaBackend::new()));
        let app = test::init_service(
            App::new()
                .app_data(client)
                .app_data(broadcaster)
                .configure(configure::<MockContext>),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/topics")
            .set_json(serde_json::json!({
                "name": "new-topic",
                "partitions": 6,
                "replication_factor": 3
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_create_topic_conflict() {
        let (client, broadcaster) = test_app(mock_ctx(MockKafkaBackend::with_topics(vec![
            sample_detail("exists"),
        ])));
        let app = test::init_service(
            App::new()
                .app_data(client)
                .app_data(broadcaster)
                .configure(configure::<MockContext>),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/topics")
            .set_json(serde_json::json!({
                "name": "exists",
                "partitions": 1,
                "replication_factor": 1
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn test_delete_topic() {
        let (client, broadcaster) = test_app(mock_ctx(MockKafkaBackend::with_topics(vec![
            sample_detail("doomed"),
        ])));
        let app = test::init_service(
            App::new()
                .app_data(client)
                .app_data(broadcaster)
                .configure(configure::<MockContext>),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri("/api/topics/doomed")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[actix_web::test]
    async fn test_update_topic_config() {
        let (client, broadcaster) = test_app(mock_ctx(MockKafkaBackend::with_topics(vec![
            sample_detail("t"),
        ])));
        let app = test::init_service(
            App::new()
                .app_data(client)
                .app_data(broadcaster)
                .configure(configure::<MockContext>),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/topics/t/config")
            .set_json(TopicConfigUpdateRequest {
                config: HashMap::from([("retention.ms".into(), "1000".into())]).into(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
