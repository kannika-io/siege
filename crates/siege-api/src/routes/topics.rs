use actix_web::{web, HttpResponse};
use siege_core::{CreateTopicRequest, TopicConfigUpdate};

use crate::error::ApiError;
use crate::kafka::backend::KafkaBackend;

pub async fn list_topics<K: KafkaBackend>(
    backend: web::Data<K>,
) -> Result<HttpResponse, ApiError> {
    let topics = backend.list_topics().await?;
    Ok(HttpResponse::Ok().json(topics))
}

pub async fn get_topic<K: KafkaBackend>(
    backend: web::Data<K>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let name = path.into_inner();
    let detail = backend.get_topic(&name).await?;
    Ok(HttpResponse::Ok().json(detail))
}

pub async fn create_topic<K: KafkaBackend>(
    backend: web::Data<K>,
    body: web::Json<CreateTopicRequest>,
) -> Result<HttpResponse, ApiError> {
    backend.create_topic(body.into_inner()).await?;
    Ok(HttpResponse::Created().finish())
}

pub async fn delete_topic<K: KafkaBackend>(
    backend: web::Data<K>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let name = path.into_inner();
    backend.delete_topic(&name).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_topic_config<K: KafkaBackend>(
    backend: web::Data<K>,
    path: web::Path<String>,
    body: web::Json<TopicConfigUpdate>,
) -> Result<HttpResponse, ApiError> {
    let name = path.into_inner();
    backend
        .update_topic_config(&name, body.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::http::StatusCode;
    use actix_web::{test, App};
    use siege_core::TopicDetail;

    use crate::kafka::mock::MockKafkaBackend;
    use crate::routes::configure;

    use super::*;

    fn sample_detail(name: &str) -> TopicDetail {
        TopicDetail {
            name: name.into(),
            partitions: 3,
            replication_factor: 1,
            config: HashMap::new(),
        }
    }

    #[actix_web::test]
    async fn test_list_topics() {
        let backend = MockKafkaBackend::with_topics(vec![
            sample_detail("topic-a"),
            sample_detail("topic-b"),
        ]);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(backend))
                .configure(configure::<MockKafkaBackend>),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/topics").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<siege_core::Topic> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);
    }

    #[actix_web::test]
    async fn test_get_topic_found() {
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("my-topic")]);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(backend))
                .configure(configure::<MockKafkaBackend>),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/topics/my-topic")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: TopicDetail = test::read_body_json(resp).await;
        assert_eq!(body.name, "my-topic");
    }

    #[actix_web::test]
    async fn test_get_topic_not_found() {
        let backend = MockKafkaBackend::new();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(backend))
                .configure(configure::<MockKafkaBackend>),
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
        let backend = MockKafkaBackend::new();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(backend))
                .configure(configure::<MockKafkaBackend>),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/topics")
            .set_json(CreateTopicRequest {
                name: "new-topic".into(),
                partitions: 6,
                replication_factor: 3,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_create_topic_conflict() {
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("exists")]);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(backend))
                .configure(configure::<MockKafkaBackend>),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/topics")
            .set_json(CreateTopicRequest {
                name: "exists".into(),
                partitions: 1,
                replication_factor: 1,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn test_delete_topic() {
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("doomed")]);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(backend))
                .configure(configure::<MockKafkaBackend>),
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
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("t")]);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(backend))
                .configure(configure::<MockKafkaBackend>),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/topics/t/config")
            .set_json(TopicConfigUpdate {
                config: HashMap::from([("retention.ms".into(), "1000".into())]),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
