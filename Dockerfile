FROM rust:1.90.0-trixie AS builder 
WORKDIR /app
COPY . .
RUN cargo build --release

FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/yeth /usr/local/bin
ENTRYPOINT ["/usr/local/bin/yeth"]
