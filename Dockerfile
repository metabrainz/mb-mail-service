
FROM rust:latest AS builder

WORKDIR /app

COPY . .

ENV RUSTFLAGS='-C target-feature=+crt-static'

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --target x86_64-unknown-linux-gnu && \
    cp ./target/x86_64-unknown-linux-gnu/release/mb-mail-service /mb-mail-service

# RUN ls -R /app

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

CMD ["/app/app"]
