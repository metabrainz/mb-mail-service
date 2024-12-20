FROM rust:latest AS builder

# install lld
RUN apt-get update && apt-get install -y lld
# install cargo-auditable
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rust-secure-code/cargo-auditable/releases/download/v0.6.6/cargo-auditable-installer.sh | sh

WORKDIR /app
COPY ./rust-toolchain.toml .
RUN rustc --version

# Get source
COPY . .

# Build binary
# We disable incremental compilation to save disk space, as it only produces a minimal speedup for this case.
ENV CARGO_INCREMENTAL=0

RUN mkdir /out
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo auditable build --locked --release --target x86_64-unknown-linux-gnu && \
    cp ./target/x86_64-unknown-linux-gnu/release/mb-mail-service /out/app

# find dynamically linked dependencies
RUN mkdir /libs \
     && ldd /out/app | grep '=>' | awk '{print $3}' | xargs -I {} cp {} /libs/
# RUN ldd /out/app
# RUN ldd /out/app | grep '=>' | awk '{print $3}'
# RUN ls /libs

FROM scratch

WORKDIR /

# Copy ld (see for example https://github.com/vlang/v/issues/8682)
COPY --from=rust:latest /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2

# Copy our build
COPY --from=builder /out/app ./app 

# Copy dynamic libraries
COPY --from=builder /libs /libs
# Tell Linux where to find our libraries
ENV LD_LIBRARY_PATH=/libs

ENV APP_LISTEN_MODE=tcp_listener
ENV APP_LISTEN_PORT=3000
ENV APP_LISTEN_HOST=0.0.0.0
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=30s --start-period=5s --retries=4 CMD ["/app", "healthcheck"]

CMD ["/app"]
