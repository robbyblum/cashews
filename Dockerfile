FROM lukemathwalker/cargo-chef:latest-rust-1.87 AS chef
WORKDIR /app

FROM chef AS planner
COPY chron-api /app/chron-api
COPY chron-base /app/chron-base
COPY chron-db /app/chron-db
COPY chron-ingest /app/chron-ingest
COPY Cargo.toml Cargo.lock /app
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY chron-api /app/chron-api
COPY chron-base /app/chron-base
COPY chron-db /app/chron-db
COPY chron-ingest /app/chron-ingest
COPY Cargo.toml Cargo.lock /app
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM archlinux:latest AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/chron-ingest /app/target/release/chron-api /app/
ENTRYPOINT ["/app/chron-ingest"]
