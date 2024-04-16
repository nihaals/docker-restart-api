FROM rust:slim-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /src/target/release/docker-restart-api /server
ENV HOST=0.0.0.0
CMD ["/server"]
