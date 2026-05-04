FROM rust:1.94 AS builder

RUN apt-get update && apt-get install -y cmake pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/*
RUN cargo install dioxus-cli@0.7.5
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && apt-get install -y nodejs

WORKDIR /build
COPY . .

# Build API server
RUN cargo build --release -p siege-api

# Build console: Tailwind CSS then Dioxus WASM
RUN cd crates/siege-console && npm install && npm run css:build && dx build --release

FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/siege-api /usr/local/bin/siege-api
ENTRYPOINT ["siege-api"]

FROM nginx:alpine AS console
COPY --from=builder /build/target/dx/siege-console/release/web/public/ /usr/share/nginx/html/
COPY <<'EOF' /etc/nginx/conf.d/default.conf
server {
    listen 80;
    root /usr/share/nginx/html;
    location / {
        try_files $uri $uri/ /index.html;
    }
}
EOF
