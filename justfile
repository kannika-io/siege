set dotenv-load

api_port := "51363"
console_port := "3000"
bootstrap_servers := "localhost:9092"

# Start Kafka in Docker
kafka:
    docker compose up kafka -d
    @echo "Waiting for Kafka to be healthy..."
    @until docker compose exec kafka kafka-topics --bootstrap-server localhost:9092 --list > /dev/null 2>&1; do sleep 1; done
    @echo "Kafka ready"

# Start Redpanda in Docker
alias redpanda := rp
rp:
    docker compose -f compose.redpanda.yaml up -d
    @echo "Waiting for Redpanda to be healthy..."
    @until docker compose -f compose.redpanda.yaml exec redpanda rpk cluster info --brokers localhost:9092 > /dev/null 2>&1; do sleep 1; done
    @echo "Redpanda ready (Kafka: localhost:19092, Schema Registry: localhost:18081, Console: localhost:9080)"

# Start Confluent Platform in Docker
alias confluent := cf
cf:
    docker compose -f compose.confluent.yaml up -d
    @echo "Waiting for Kafka to be healthy..."
    @until docker compose -f compose.confluent.yaml exec kafka kafka-topics --bootstrap-server localhost:9092 --list > /dev/null 2>&1; do sleep 1; done
    @echo "Kafka ready"
    @echo "Waiting for Schema Registry to be healthy..."
    @until curl -sf http://localhost:8081/subjects > /dev/null 2>&1; do sleep 1; done
    @echo "Confluent ready (Kafka: localhost:9092, Schema Registry: localhost:8081, UI: localhost:9080)"

# Build and run the API server (with seed topics)
api: cf
    cargo run -p siege-api -- --bootstrap-servers localhost:9092 --schema-registry-url http://localhost:8081 --port {{api_port}} --seed --post-seed-hook ./scripts/reset-armory/init.sh

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
