FROM rust:1.84-slim AS builder
RUN apt-get update && apt-get install -y --no-install-recommends git build-essential pkg-config openssl libssl-dev ca-certificates curl unzip && rm -rf /var/lib/apt/lists/*
ARG GIT_REPO_URL
ARG GIT_REF=main
RUN test -n "$GIT_REPO_URL"
WORKDIR /src
RUN git clone --depth 1 --branch "$GIT_REF" "$GIT_REPO_URL" /src
WORKDIR /src/slam_server
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN useradd -m appuser && apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/slam_server/target/release/slam_server /app/slam_server
RUN mkdir -p /data && chown -R appuser:appuser /app /data
USER appuser
EXPOSE 3000
CMD ["/app/slam_server"]
