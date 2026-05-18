use std::future::Future;

use crate::SiegeError;

#[derive(Clone)]
pub struct SeedResult {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}

pub trait SeedBackend: Send + Sync + 'static {
    fn seed_topics(&self) -> impl Future<Output = Result<SeedResult, SiegeError>> + Send;
}
