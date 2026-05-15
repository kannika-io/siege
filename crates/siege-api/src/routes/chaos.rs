use actix_web::{HttpResponse, web};
use siege::SiegeContext;
use siege_api_spec::{ChaosPartitionsRequest, ChaosProduceRequest, ChaosResult, ChaosTopicRequest};

use crate::error::HttpError;

fn ok(topic: String) -> HttpResponse {
    HttpResponse::Ok().json(ChaosResult {
        topic,
        result: "ok".into(),
    })
}

pub async fn delete_topic<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().delete_topic(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn low_retention<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().low_retention(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn flip_cleanup_policy<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().flip_cleanup_policy(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn increase_partitions<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<ChaosPartitionsRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().increase_partitions(&req.topic, req.partitions).await?;
    Ok(ok(req.topic))
}

pub async fn poison_pills<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<ChaosProduceRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().poison_pills(&req.topic, req.count).await?;
    Ok(ok(req.topic))
}

pub async fn schema_break<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
    body: web::Json<ChaosProduceRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    client.chaos().schema_break(&req.topic, req.count).await?;
    Ok(ok(req.topic))
}
