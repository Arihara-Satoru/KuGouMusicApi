FROM node:lts-bookworm-slim AS compat
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY module ./module
COPY util ./util
COPY rust-compat ./rust-compat
COPY scripts/build-rust-compat.cjs ./scripts/build-rust-compat.cjs
RUN node scripts/build-rust-compat.cjs

FROM rust:1.93-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY rust-src ./rust-src
COPY --from=compat /app/rust-assets ./rust-assets
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
