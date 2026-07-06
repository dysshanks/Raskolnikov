FROM rust:1.96-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src/ src/
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl ca-certificates nmap gobuster nikto sqlmap && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/raskolnikov /usr/local/bin/
RUN ln -s raskolnikov /usr/local/bin/rsk && ln -s raskolnikov /usr/local/bin/rk
ENTRYPOINT ["raskolnikov"]
CMD ["--help"]
