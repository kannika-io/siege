use std::fmt;
use std::future::Future;

pub struct SeedResult {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}

pub trait SeedBackend: Send + Sync + 'static {
    type Error: fmt::Display + fmt::Debug + Send + Sync + 'static;

    fn seed_topics(&self) -> impl Future<Output = Result<SeedResult, Self::Error>> + Send;
}
