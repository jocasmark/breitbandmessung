FROM rust:1.82 AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

COPY . .
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y \
    libssl1.1 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/speedtest_mqtt /usr/local/bin/speedtest_mqtt

# Set the default command to run your application
CMD ["speedtest_mqtt"]
