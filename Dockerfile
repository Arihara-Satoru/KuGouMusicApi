FROM rust:1.93-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY rust-src ./rust-src
COPY rust-native.json ./
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/kugoumusicapi /usr/local/bin/kugoumusicapi
COPY public ./public
COPY docs ./docs
EXPOSE 3000
ENTRYPOINT ["kugoumusicapi"]
