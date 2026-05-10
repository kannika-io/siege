mod error;
mod kafka;
mod mapping;
mod routes;
mod seed;
mod sse;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use siege::SiegeContext;
use siege_api_spec::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use kafka::rdkafka_backend::RdKafkaBackend;
use sse::broadcaster::Broadcaster;

struct Siege {
    kafka: RdKafkaBackend,
    events: Broadcaster,
}

impl SiegeContext for Siege {
    type Kafka = RdKafkaBackend;
    type Events = Broadcaster;

    fn kafka(&self) -> &RdKafkaBackend {
        &self.kafka
    }

    fn events(&self) -> &Broadcaster {
        &self.events
    }
}

#[derive(Parser)]
#[command(name = "siege-api", about = "Kafka topic management API")]
struct Cli {
    #[arg(long)]
    bootstrap_servers: String,

    #[arg(long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "false")]
    seed: bool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let backend = RdKafkaBackend::new(&cli.bootstrap_servers);

    if cli.seed {
        seed::seed_topics(&backend).await;
    }

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

    let broadcaster_data = web::Data::new(broadcaster.clone());

    let client = web::Data::new(siege::client::Client::new(Siege {
        kafka: backend,
        events: broadcaster,
    }));

    let addr = ("0.0.0.0", cli.port);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(client.clone())
            .app_data(broadcaster_data.clone())
            .configure(routes::configure::<Siege>)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    });

    let server = match server.bind(addr) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            eprintln!(
                "error: port {} is already in use — try --port <other>",
                addr.1
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: failed to bind to {}:{} — {e}", addr.0, addr.1);
            std::process::exit(1);
        }
    };

    eprintln!("listening on {}:{}", addr.0, addr.1);
    server.run().await
}


