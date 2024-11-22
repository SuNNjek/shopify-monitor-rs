# =========================
# =        Builder        =
# =========================
FROM rust:slim-bookworm AS builder

RUN apt update && apt install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Fetch dependencies
COPY Cargo.toml .
RUN cargo fetch

# Build release binary
COPY src ./src
RUN cargo build --release


# =========================
# =         Image         =
# =========================
FROM debian:bookworm-slim AS release

RUN apt update && apt install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy over binary from builder stage
COPY --from=builder ./target/release/shopify-monitor .

CMD [ "/app/shopify-monitor" ]