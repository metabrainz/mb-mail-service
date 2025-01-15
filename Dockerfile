FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1-slim-bookworm AS builder

# Install repo tools
# Line one: compiler tools
# Line two: curl, for downloading binaries
# Line three: for xx-verify
RUN apt-get update && apt-get install -y \
    clang lld pkg-config \
    curl \
    libmagic-mgc libmagic1 file

# Developer tool versions
# renovate: datasource=github-releases depName=cargo-binstall packageName=cargo-bins/cargo-binstall
ENV BINSTALL_VERSION=1.10.17
# renovate: datasource=github-releases depName=cargo-sbom packageName=psastras/sbom-rs
ENV CARGO_SBOM_VERSION=0.9.1

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm cargo-sbom --version $CARGO_SBOM_VERSION

# Set up xx (cross-compilation scripts)
COPY --from=xx / /
ARG TARGETPLATFORM

# install libraries linked by the binary
# xx-* are xx-specific meta-packages
# c is needed for ring
# cxx is needed for openssl
RUN xx-apt-get install -y xx-c-essentials xx-cxx-essentials \
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
     && ldd /out/app | grep '=>' | awk '{print $(NF-1)}' | xargs -I {} cp {} /out/libs/
# libraries with a hardcoded path, like ld
# (see for example https://github.com/vlang/v/issues/8682)
# Excluding linux-vdso.so, as it is a part of the kernel
RUN mkdir /out/libs-root \
    && ldd /out/app | grep -v '=>' | grep -v 'linux-vdso.so' | awk '{print $(NF-1)}' | xargs -I {} install -D {} /out/libs-root{}
# RUN ldd /out/app
# ldd /out/app | grep -v 'linux-vdso.so' | awk '{print $(NF-1)}'
# RUN ls /libs

FROM scratch

WORKDIR /

# Copy our build
COPY --from=builder /out/app ./app 
# Copy SBOM
COPY --from=builder /out/sbom.spdx.json ./sbom.spdx.json

# Copy hardcoded dynamic libraries
COPY --from=builder /out/libs-root /
# Copy dynamic libraries
COPY --from=builder /out/libs /libs
# Tell Linux where to find our libraries
ENV LD_LIBRARY_PATH=/libs

ENV APP_LISTEN_MODE=tcp_listener
ENV APP_LISTEN_PORT=3000
ENV APP_LISTEN_HOST=0.0.0.0
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=30s --start-period=5s --retries=4 CMD ["/app", "healthcheck"]

CMD ["/app"]
