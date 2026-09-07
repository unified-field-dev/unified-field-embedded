# Prefer digest-pinned bases so rebuilds do not silently pick a newer floating tag.
FROM rust:1-bookworm@sha256:82150a52ec202c1b14d7817e14516c392bb7f5cfebd88f1ed531cb37ebd39922 AS builder
WORKDIR /app
COPY . .
RUN cargo build -p server --release --features server-embedded --locked

FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 uf \
    && useradd --system --uid 10001 --gid uf --home-dir /var/lib/uf --create-home uf
COPY --from=builder /app/target/release/server /usr/local/bin/server
USER uf
EXPOSE 3000
CMD ["server"]
