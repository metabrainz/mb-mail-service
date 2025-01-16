FROM --platform=$BUILDPLATFORM docker.io/tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1-slim-bookworm AS builder

# Install repo tools
# Line one: compiler tools
# Line two: curl, for downloading binaries
# Line three: for xx-verify
RUN apt-get update && apt-get install -y \
    clang lld pkg-config \
    curl \
    file

# Developer tool versions
# renovate: datasource=github-releases depName=cargo-bins/cargo-binstall
ENV BINSTALL_VERSION=1.10.21
# renovate: datasource=github-releases depName=psastras/sbom-rs
ENV CARGO_SBOM_VERSION=0.9.1
# renovate: datasource=crate depName=lddtree
ENV LDDTREE_VERSION=0.3.7

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm cargo-sbom --version $CARGO_SBOM_VERSION
RUN cargo binstall --no-confirm lddtree --version $LDDTREE_VERSION

# Set up xx (cross-compilation scripts)
COPY --from=xx / /
ARG TARGETPLATFORM

# install libraries linked by the binary
# xx-* are xx-specific meta-packages
# c is needed for ring
# cxx is needed for openssl
RUN xx-apt-get install -y \
    xx-c-essentials xx-cxx-essentials \
    libssl-dev

# Set up Rust toolchain
WORKDIR /app
COPY ./rust-toolchain.toml .
RUN rustc --version
RUN rustup target add $(xx-cargo --print-target-triple)

# Get source
COPY . .

# Build binary
# We disable incremental compilation to save disk space, as it only produces a minimal speedup for this case.
ENV CARGO_INCREMENTAL=0

RUN echo "PKG_CONFIG_LIBDIR=/usr/lib/$(xx-info)/pkgconfig" >> /etc/environment
RUN echo "PKG_CONFIG=/usr/bin/$(xx-info)-pkg-config"
RUN echo "PKG_CONFIG_ALLOW_CROSS=true" >> /etc/environment

RUN cat /etc/environment

RUN mkdir /out
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    set -o allexport && \
    . /etc/environment && \
    xx-cargo build --locked --release && \
    xx-verify ./target/$(xx-cargo --print-target-triple)/release/mb-mail-service && \
    cp ./target/$(xx-cargo --print-target-triple)/release/mb-mail-service /out/app

RUN cargo sbom > /out/sbom.spdx.json

# find dynamically linked dependencies
RUN mkdir /out/libs \
    && lddtree /out/app | awk '{print $(NF-0) " " $1}' | sort -u -k 1,1 | awk '{print "install", "-D", $1, "/out/libs" (($2 ~ /^\//) ? $2 : $1)}' | xargs -I {} sh -c {}

FROM scratch

WORKDIR /

# Copy our build
COPY --from=builder /out/app ./app 
# Copy SBOM
COPY --from=builder /out/sbom.spdx.json ./sbom.spdx.json

# Copy dynamic libraries to root
COPY --from=builder /out/libs /

ENV APP_LISTEN_MODE=tcp_listener
ENV APP_LISTEN_PORT=3000
ENV APP_LISTEN_HOST=0.0.0.0
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=30s --start-period=5s --retries=4 CMD ["/app", "healthcheck"]

CMD ["/app"]
