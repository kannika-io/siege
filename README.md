# Siege

Siege is a chaos tool for Kafka, built in Rust.
It lets you wreak havoc on Kafka topics for demo and testing purposes: delete topics, corrupt schemas, send poison pills, flip cleanup policies, and more.
It ships with a web console featuring a "Wheel of Chaos" and a REST API.

The backend is an Actix-web server that talks to Kafka and streams live updates to the frontend via SSE.
The frontend is a Dioxus/WASM app styled with Tailwind CSS.
A built-in seeder can bulk-generate deterministic Avro test data against a Schema Registry.

## Quick start

**Prerequisites:** [Rust](https://rustup.rs/), [Docker](https://docs.docker.com/get-docker/), [just](https://github.com/casey/just), [dioxus-cli](https://dioxuslabs.com/), [Node.js](https://nodejs.org/)

```sh
just cf        # Start Confluent Platform (Kafka + Schema Registry)
just api       # Start the API (seeds test data on first run)
just console   # Start the web console (in another terminal)
```

Console at `http://localhost:3000`, API at `http://localhost:51363`, Swagger UI at `http://localhost:51363/swagger-ui/`.


## License

[Business Source License 1.1](LICENSE)
