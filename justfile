set dotenv-load

api_port := "8080"
console_port := "3000"
bootstrap_servers := "localhost:9092"

# Start Kafka in Docker
kafka:
    docker compose up kafka -d
    @echo "Waiting for Kafka to be healthy..."
    @until docker compose exec kafka kafka-topics --bootstrap-server localhost:9092 --list > /dev/null 2>&1; do sleep 1; done
    @echo "Kafka ready"

# Build and run the API server (with seed topics)
api: kafka
    cargo run -p siege-api -- --bootstrap-servers {{bootstrap_servers}} --port {{api_port}} --seed

# Build Tailwind CSS (watch mode)
css-watch:
    cd crates/siege-console && npm run css:watch

# Build Tailwind CSS (one-shot)
css-build:
    cd crates/siege-console && npm run css:build

# Run the Dioxus dev server
console: css-build
    dx serve --package siege-console --port {{console_port}}

# Run API + console in parallel (requires Kafka already running)
dev:
    just kafka
    just api & just css-watch & just console-only
    wait

# Run console without rebuilding CSS first (for use with dev)
console-only:
    dx serve --package siege-console --port {{console_port}}

# Run all tests
test:
    cargo test --workspace --exclude siege-console

# Check everything compiles
check:
    cargo check --workspace --exclude siege-console
    dx check --package siege-console

# Stop Kafka
down:
    docker compose down
