use actix_web::{web, HttpResponse};
use siege_api_spec::{ChaosPartitionsRequest, ChaosProduceRequest, ChaosResult, ChaosTopicRequest};
use siege_chaos::ChaosClient;

use crate::error::HttpError;

fn ok(topic: String) -> HttpResponse {
    HttpResponse::Ok().json(ChaosResult {
        topic,
        result: "ok".into(),
    })
}

pub async fn delete_topic(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.delete_topic(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn zero_retention(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.zero_retention(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn flip_cleanup_policy(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.flip_cleanup_policy(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn increase_partitions(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosPartitionsRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.increase_partitions(&req.topic, req.partitions).await?;
    Ok(ok(req.topic))
}

pub async fn poison_pills(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosProduceRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.poison_pills(&req.topic, req.count).await?;
    Ok(ok(req.topic))
}

pub async fn schema_break(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosProduceRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.schema_break(&req.topic, req.count).await?;
    Ok(ok(req.topic))
}
