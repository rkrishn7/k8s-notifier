FROM rust:1.71.0-buster as builder
WORKDIR /usr/src

# Create a new empty shell project
RUN USER=root cargo new k8s-notifier
WORKDIR /usr/src/k8s-notifier

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# Cache dependencies
RUN cargo build --release
RUN rm src/*.rs

COPY . .

# Build for release
RUN rm ./target/release/deps/k8s_notifier*

RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app

# Install libssl (Rust links against this library)
RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates && \
    apt-get clean

# Copy the binary from the builder stage
COPY --from=builder /usr/src/k8s-notifier/target/release/k8s-notifier .

CMD ["./k8s-notifier"]
