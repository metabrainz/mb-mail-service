FROM rust:latest AS builder

# install lld
RUN apt-get update && apt-get install -y lld

# Set up Rust toolchain
WORKDIR /app
COPY ./rust-toolchain.toml .
RUN rustc --version

# convert docker target to rust target
ARG TARGETPLATFORM

# for available rust targets, see `rustup target list` or https://doc.rust-lang.org/nightly/rustc/platform-support.html
# for available docker platforms, see https://github.com/docker/cli/blob/fb2ba5d63ba4166ceeefa21c2fd866b06966874e/cli/command/manifest/util.go#L23
RUN TARGETTUPLE=$(case $TARGETPLATFORM in \
    "linux/386") echo i686-unknown-linux-gnu ;; \
    "linux/amd64") echo x86_64-unknown-linux-gnu ;; \
    "linux/arm64") echo aarch64-unknown-linux-gnu ;; \
    "linux/arm") echo arm-unknown-linux-gnueabihf ;; \
    "linux/arm/v7") echo armv7-unknown-linux-gnueabihf ;; \
    "linux/riscv64") echo riscv64gc-unknown-linux-gnu ;; \
    "linux/ppc64le") echo powerpc64le-unknown-linux-gnu ;; \
    "linux/s390x") echo s390x-unknown-linux-gnu ;; \
    *) exit 1 ;; \
    esac) && \
    echo "TARGETTUPLE=$TARGETTUPLE" >> /etc/environment

# Developer tool versions
# renovate: datasource=github-releases depName=cargo-binstall packageName=cargo-bins/cargo-binstall
ENV BINSTALL_VERSION=1.10.17
# renovate: github-releases depName=cargo-sbom packageName=psastras/sbom-rs
ENV CARGO_SBOM_VERSION=0.9.1

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm cargo-sbom --version $CARGO_SBOM_VERSION

# Get source
COPY . .

# Build binary
# We disable incremental compilation to save disk space, as it only produces a minimal speedup for this case.
ENV CARGO_INCREMENTAL=0

RUN mkdir /out
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    . /etc/environment && \
    cargo build --locked --release --target $TARGETTUPLE && \
    cp ./target/$TARGETTUPLE/release/mb-mail-service /out/app

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
