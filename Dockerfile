FROM rust:alpine AS builder

# Install dependencies for OpenSSL vendored build
RUN apk add --no-cache perl make pkgconf openssl-dev musl-dev

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy source code and build with OpenSSL vendoring enabled
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage with minimal runtime using scratch
FROM scratch

# Copy CA certificates for SSL verification
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/speedtest_mqtt /usr/local/bin/speedtest_mqtt

CMD ["speedtest_mqtt"]
