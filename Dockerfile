FROM --platform=$BUILDPLATFORM docker.io/tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1-slim-bookworm AS builder

# Don't delete the apt cache
RUN rm -f /etc/apt/apt.conf.d/docker-clean

# Install repo tools
# Line one: compiler tools
# Line two: curl, for downloading binaries
# Line three: for xx-verify
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
    clang lld pkg-config make jq \
    curl \
    file

# Developer tool versions
# renovate: datasource=github-releases depName=cargo-bins/cargo-binstall
ENV BINSTALL_VERSION=1.14.4
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
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    xx-apt-get install -y \
    xx-c-essentials xx-cxx-essentials \
    libssl-dev

# Set up Rust toolchain
WORKDIR /app
COPY ./rust-toolchain.toml .
RUN rustc --version
RUN rustup target add $(xx-cargo --print-target-triple)

# Build binary
# We disable incremental compilation to save disk space, as it only produces a minimal speedup for this case.
RUN echo "CARGO_INCREMENTAL=0" >> /etc/environment

# Configure pkg-config
RUN <<EOF
    echo "PKG_CONFIG_LIBDIR=/usr/lib/$(xx-info)/pkgconfig" >> /etc/environment
    echo "PKG_CONFIG=/usr/bin/$(xx-info)-pkg-config" >> /etc/environment
    echo "PKG_CONFIG_ALLOW_CROSS=true" >> /etc/environment
EOF

# Prepare output directories
RUN mkdir /out


# Verify environment configuration
RUN cat /etc/environment
RUN xx-cargo --print-target-triple

# Get source
COPY . .

# Build the binary
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    bash <<'EOF'
    set -o allexport
    . /etc/environment
    TARGET_DIR=($(cargo metadata --no-deps --format-version 1 | \
            jq -r ".target_directory"))
    mkdir /out/sbin
    PACKAGE=mb-mail-service
    xx-cargo build --locked --release \
        -p $PACKAGE;
    BINARIES=($(cargo metadata --no-deps --format-version 1 | \
        jq -r ".packages[] | select(.name == \"$PACKAGE\") | .targets[] | select( .kind | map(. == \"bin\") | any ) | .name"))
    for BINARY in "${BINARIES[@]}"; do
        echo $BINARY
        xx-verify $TARGET_DIR/$(xx-cargo   --print-target-triple)/release/$BINARY
        cp $TARGET_DIR/$(xx-cargo --print-target-triple)/release/$BINARY /out/sbin/$BINARY
    done
EOF

# Generate Software Bill of Materials (SBOM)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    bash <<'EOF'
    mkdir /out/sbom
    typeset -A PACKAGES
    for BINARY in /out/sbin/*; do
        BINARY_BASE=$(basename ${BINARY})
        package=$(cargo metadata --no-deps --format-version 1 | jq -r ".packages[] | select(.targets[] | select( .kind | map(. == \"bin\") | any ) | .name == \"$BINARY_BASE\") | .name")
        if [ -z "$package" ]; then
            continue
        fi
        PACKAGES[$package]=1
    done
    for PACKAGE in $(echo ${!PACKAGES[@]}); do
        echo $PACKAGE
        cargo sbom --cargo-package $PACKAGE > /out/sbom/$PACKAGE.spdx.json
    done
EOF

# Extract dynamically linked dependencies
RUN <<EOF
    mkdir /out/libs
    mkdir /out/libs-root
    for BINARY in /out/sbin/*; do
        lddtree "$BINARY" | awk '{print $(NF-0) " " $1}' | sort -u -k 1,1 | awk '{print "install", "-D", $1, (($2 ~ /^\//) ? "/out/libs-root" $2 : "/out/libs/" $2)}' | xargs -I {} sh -c {}
    done
EOF

FROM scratch

WORKDIR /

# Copy root certs for tls into image
# You can also mount the certs from the host
# --volume /etc/ssl/certs:/etc/ssl/certs:ro
COPY --from=rust:1-slim-bookworm /etc/ssl/certs /etc/ssl/certs

# Copy our build
COPY --from=builder /out/sbin/ /sbin/
# Copy SBOM
COPY --from=builder /out/sbom/ /sbom/

# Copy dynamic libraries to root
COPY --from=builder /out/libs-root/ /
COPY --from=builder /out/libs/ /usr/lib/

# Inform linker where to find libraries
ENV LD_LIBRARY_PATH=/usr/lib

# Default configuration to expose the server
ENV APP_LISTEN_MODE=tcp_listener
ENV APP_LISTEN_PORT=3000
ENV APP_LISTEN_HOST=0.0.0.0
EXPOSE 3000

# Basic healthcheck to ensure the server is running
HEALTHCHECK --interval=15s --timeout=30s --start-period=5s --retries=4 CMD ["mb-mail-service", "healthcheck"]

CMD ["mb-mail-service"]
