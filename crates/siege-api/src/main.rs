mod error;
mod kafka;
mod mapping;
mod routes;
mod sse;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use clap::Parser;
use siege::{SeedBackend, SiegeContext};
use siege_api_spec::ApiDoc;
use siege_schema_registry::SchemaRegistryClient;
use siege_seed::avsc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use siege_chaos::ChaosClient;
use siege_kafka::RdKafkaBackend;
use siege_seed::{Seeder, TopicSeed};
use sse::broadcaster::Broadcaster;
use tokio_util::sync::CancellationToken;

pub(crate) struct Siege {
    kafka: RdKafkaBackend,
    events: Broadcaster,
    chaos: ChaosClient,
    seeder: Seeder,
    schema_registry: Option<SchemaRegistryClient>,
}

impl SiegeContext for Siege {
    type Kafka = RdKafkaBackend;
    type Events = Broadcaster;
    type Chaos = ChaosClient;
    type Seeder = Seeder;
    type SchemaRegistry = SchemaRegistryClient;

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

    fn schema_registry(&self) -> Option<&SchemaRegistryClient> {
        self.schema_registry.as_ref()
    }
}

#[derive(Parser)]
#[command(name = "siege-api", about = "Kafka topic management API")]
struct Cli {
    #[arg(long)]
    bootstrap_servers: String,

    #[arg(long, default_value = "51363")]
    port: u16,

    #[arg(long, default_value = "false")]
    seed: bool,

    #[arg(long)]
    schema_registry_url: Option<String>,

    #[arg(long)]
    post_seed_hook: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let backend = RdKafkaBackend::new(&cli.bootstrap_servers);
    let broadcaster = Arc::new(Broadcaster::new(256));
    let chaos = ChaosClient::new(backend.clone());

    let schema_registry = cli
        .schema_registry_url
        .as_deref()
        .map(SchemaRegistryClient::new);

    let mut seeder = Seeder::new(backend.clone())
        .idempotent()
        .events(broadcaster.clone())
        .topic(
            TopicSeed::new("kings-landing", 6)
                .schema(avsc!("../../schemas/kings-landing.avsc"))
                .records(100_000_000),
        )
        .topic(
            TopicSeed::new("winterfell", 3)
                .schema(avsc!("../../schemas/winterfell.avsc"))
                .records(1_000_000),
        )
        .topic(
            TopicSeed::new("the-wall", 1)
                .schema(avsc!("../../schemas/the-wall.avsc"))
                .records(500_000),
        )
        .topic(
            TopicSeed::new("iron-islands", 3)
                .schema(avsc!("../../schemas/iron-islands.avsc"))
                .records(1_000_000),
        )
        .topic(
            TopicSeed::new("dragonstone", 3)
                .schema(avsc!("../../schemas/dragonstone.avsc"))
                .records(1_000_000),
        )
        .topic(
            TopicSeed::new("the-citadel", 1)
                .config("cleanup.policy", "compact")
                .schema(avsc!("../../schemas/the-citadel.avsc"))
                .records(500_000),
        );

    if let Some(ref hook_path) = cli.post_seed_hook {
        if cli.seed {
            seeder = seeder.on_complete(tokio::process::Command::new(hook_path));
        } else {
            eprintln!("warning: --post-seed-hook has no effect without --seed");
        }
    }

    if let Some(ref url) = cli.schema_registry_url {
        seeder = seeder.schema_registry(SchemaRegistryClient::new(url));
    }

    if cli.seed {
        if let Err(e) = seeder.seed_topics().await {
            eprintln!("seed error: {e}");
        }
    }

    let cancel = CancellationToken::new();

    let watcher_backend = backend.clone();
    let watcher_broadcaster = broadcaster.clone();
    let watcher_cancel = cancel.clone();
    let watcher_handle = tokio::spawn(async move {
        sse::watcher::watch_cluster(
            &watcher_backend,
            &watcher_broadcaster,
            std::time::Duration::from_secs(5),
            watcher_cancel,
        )
        .await;
    });

    let broadcaster_data = web::Data::from(broadcaster.clone());

    let client = web::Data::new(siege::client::Client::new(Siege {
        kafka: backend,
        events: (*broadcaster).clone(),
        chaos,
        seeder,
        schema_registry,
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
    let server = server.disable_signals().run();
    let server_handle = server.handle();

    let shutdown_cancel = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            eprintln!("\nshutting down…");
            shutdown_cancel.cancel();
            server_handle.stop(true).await;
        }
    });

    let result = server.await;
    cancel.cancel();
    let _ = watcher_handle.await;
    result
}
