use actix_web::{HttpResponse, web};
use siege::{ChaosBackend, SiegeContext};
use siege_api_spec::{
    ChaosPartitionsRequest, ChaosProduceRequest, ChaosResult, ChaosTopicRequest, SseEvent,
    TopicResource,
};

use crate::error::HttpError;
use crate::sse::broadcaster::Broadcaster;

fn ok(topic: String) -> HttpResponse {
    HttpResponse::Ok().json(ChaosResult {
        topic,
        result: "ok".into(),
    })
}

fn broadcast_updated(broadcaster: &Broadcaster, detail: siege::kafka::TopicDetail) {
    broadcaster.send(SseEvent::TopicUpdated {
        topic: TopicResource {
            name: detail.name,
            partitions: detail.partitions,
            replication_factor: detail.replication_factor,
            config: detail.config,
        },
    });
}

fn chaos_err(e: impl std::fmt::Display) -> HttpError {
    HttpError::bad_gateway(e.to_string())
}

pub async fn delete_topic<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    broadcaster: web::Data<Broadcaster>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().delete_topic(&req.topic).await.map_err(chaos_err)?;
    broadcaster.send(SseEvent::TopicDeleted {
        name: req.topic.clone(),
    });
    Ok(ok(req.topic))
}

pub async fn zero_retention<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    broadcaster: web::Data<Broadcaster>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().zero_retention(&req.topic).await.map_err(chaos_err)?;
    if let Ok(detail) = client.chaos().get_topic(&req.topic).await {
        broadcast_updated(&broadcaster, detail);
    }
    Ok(ok(req.topic))
}

pub async fn flip_cleanup_policy<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    broadcaster: web::Data<Broadcaster>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().flip_cleanup_policy(&req.topic).await.map_err(chaos_err)?;
    if let Ok(detail) = client.chaos().get_topic(&req.topic).await {
        broadcast_updated(&broadcaster, detail);
    }
    Ok(ok(req.topic))
}

pub async fn increase_partitions<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    broadcaster: web::Data<Broadcaster>,
    body: web::Json<ChaosPartitionsRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client
        .chaos()
        .increase_partitions(&req.topic, req.partitions)
        .await
        .map_err(chaos_err)?;
    if let Ok(detail) = client.chaos().get_topic(&req.topic).await {
        broadcast_updated(&broadcaster, detail);
    }
    Ok(ok(req.topic))
}

pub async fn poison_pills<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<ChaosProduceRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().poison_pills(&req.topic, req.count).await.map_err(chaos_err)?;
    Ok(ok(req.topic))
}

pub async fn schema_break<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<ChaosProduceRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().schema_break(&req.topic, req.count).await.map_err(chaos_err)?;
    Ok(ok(req.topic))
}
