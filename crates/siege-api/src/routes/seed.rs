use actix_web::{web, HttpResponse};
use siege::{SeedBackend, SiegeContext};

use crate::error::HttpError;

pub async fn run_seed<C: SiegeContext>(
    client: web::Data<siege::client::Client<C>>,
) -> Result<HttpResponse, HttpError> {
    client
        .seeder()
        .seed_topics()
        .await
        .map_err(|e| HttpError::bad_gateway(e.to_string()))?;
    Ok(HttpResponse::Ok().finish())
}
