# Build stage
FROM rust:alpine AS builder
WORKDIR /usr/src/app

# Install required tools and OpenSSL development packages
# Using the -dev packages for development and static libraries
RUN apk add --no-cache \
    curl \
    jq \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    musl-dev \
    build-base \
    git \
    linux-headers  
# Set environment variables for static linking of OpenSSL
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr

ARG GHPAT
ENV GHPAT=${GHPAT}

RUN git config --global url."https://${GHPAT}@github.com/".insteadOf "https://github.com/"


# Configure cargo to use git CLI for fetching
RUN mkdir -p ~/.cargo && \
    echo -e '[net]\ngit-fetch-with-cli = true' > ~/.cargo/config

# Copy over manifests
COPY Cargo.toml Cargo.lock* ./

# Extract package (binary) name dynamically from Cargo.toml
RUN export BIN_NAME=$(grep -m 1 "name" Cargo.toml | sed 's/.*"\(.*\)".*/\1/') && \
    echo "Detected binary: $BIN_NAME"

# Cache dependencies - this is the key step for optimizing build times
RUN mkdir -p src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the actual source code
COPY . .

RUN ls -la

# Build the actual application, leveraging cached dependencies
RUN touch src/main.rs && \
    export BIN_NAME=$(grep -m 1 "name" Cargo.toml | sed 's/.*"\(.*\)".*/\1/') && \
    cargo build --release && \
    mv target/release/$BIN_NAME /usr/src/app/binary

# Runtime stage
FROM alpine:latest
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/binary /app/binary
COPY --from=builder usr/src/app/erc20_abi.json /app/erc20_abi.json

# Set the command to run the application
CMD ["/app/binary"]
