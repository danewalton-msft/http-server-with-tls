FROM rust:1.76 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:24.04
RUN apt-get update && apt-get install -y ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/rust-mtls-server .
COPY certs /certs
CMD ["./rust-mtls-server"]
