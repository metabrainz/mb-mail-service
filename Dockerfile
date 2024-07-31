
FROM rust:latest AS builder

WORKDIR /app
COPY ./rust-toolchain.toml .
RUN rustc --version

COPY . .

ENV RUSTFLAGS='-C target-feature=+crt-static'
ENV CARGO_INCREMENTAL=0

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release --target x86_64-unknown-linux-gnu && \
    cp ./target/x86_64-unknown-linux-gnu/release/mb-mail-service /mb-mail-service



# serve

FROM scratch

# Import from builder.

WORKDIR /app

# Copy our build
COPY --from=builder /mb-mail-service ./app 

ENV APP_LISTEN_MODE=tcp_listener
ENV APP_LISTEN_PORT=3000
ENV APP_LISTEN_HOST=0.0.0.0
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=30s --start-period=5s --retries=4 CMD ["/app/app", "healthcheck"]

LABEL org.opencontainers.image.source=https://github.com/metabrainz/mb-mail-service
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.licenses=GPL-2.0-or-later

CMD ["/app/app"]
