mod error;
mod kafka;
mod routes;
mod sse;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use siege_api_spec::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use kafka::rdkafka_backend::RdKafkaBackend;
use sse::broadcaster::Broadcaster;

#[derive(Parser)]
#[command(name = "siege-api", about = "Kafka topic management API")]
struct Cli {
    #[arg(long)]
    bootstrap_servers: String,

    #[arg(long, default_value = "8080")]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let backend = RdKafkaBackend::new(&cli.bootstrap_servers);
    let broadcaster = Broadcaster::new(256);

    let watcher_backend = backend.clone();
    let watcher_broadcaster = broadcaster.clone();
    tokio::spawn(async move {
        sse::watcher::watch_cluster(
            &watcher_backend,
            &watcher_broadcaster,
            std::time::Duration::from_secs(5),
        )
        .await;
    });

    let backend_data = web::Data::new(backend);
    let broadcaster_data = web::Data::new(broadcaster);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(backend_data.clone())
            .app_data(broadcaster_data.clone())
            .configure(routes::configure::<RdKafkaBackend>)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(("0.0.0.0", cli.port))?
    .run()
    .await
}
