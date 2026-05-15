mod error;
mod kafka;
mod mapping;
mod routes;
mod sse;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use siege::{NoopSchemaRegistry, SeedBackend, SiegeContext};
use siege_api_spec::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use siege_chaos::ChaosClient;
use siege_kafka::RdKafkaBackend;
use siege_seed::{Seeder, TopicSeed};
use sse::broadcaster::Broadcaster;

pub(crate) struct Siege {
    kafka: RdKafkaBackend,
    events: Broadcaster,
    chaos: ChaosClient,
    seeder: Seeder,
    schema_registry: NoopSchemaRegistry,
}

impl SiegeContext for Siege {
    type Kafka = RdKafkaBackend;
    type Events = Broadcaster;
    type Chaos = ChaosClient;
    type Seeder = Seeder;
    type SchemaRegistry = NoopSchemaRegistry;

    fn kafka(&self) -> &RdKafkaBackend {
        &self.kafka
    }

    fn events(&self) -> &Broadcaster {
        &self.events
    }

    fn chaos(&self) -> &ChaosClient {
        &self.chaos
    }

    fn seeder(&self) -> &Seeder {
        &self.seeder
    }

    fn schema_registry(&self) -> &NoopSchemaRegistry {
        &self.schema_registry
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
    let broadcaster = Broadcaster::new(256);
    let chaos = ChaosClient::new(backend.clone());
    let seeder = Seeder::new(backend.clone())
        .topic(TopicSeed::new("kings-landing", 6))
        .topic(TopicSeed::new("winterfell", 3))
        .topic(TopicSeed::new("the-wall", 1))
        .topic(TopicSeed::new("iron-islands", 3))
        .topic(TopicSeed::new("dragonstone", 3))
        .topic(TopicSeed::new("the-citadel", 1).config("cleanup.policy", "compact"));

    if cli.seed {
        if let Err(e) = seeder.seed_topics().await {
            eprintln!("seed error: {e}");
        }
    }

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
        chaos,
        seeder,
        schema_registry: NoopSchemaRegistry,
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
