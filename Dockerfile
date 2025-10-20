FROM rust:1.90.0-trixie AS builder 
WORKDIR /app
COPY . .
RUN cargo build --release

FROM scratch AS runtime
WORKDIR /usr/local/bin
COPY --from=builder /app/target/release/yeth .
ENTRYPOINT ["/usr/local/bin/yeth"]
