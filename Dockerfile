# ------------------------------------------------------------------------------
# QubitCore - Production Minimal Container
# Multi-stage build targeting Alpine Linux (<60MB RSS runtime budget)
# ------------------------------------------------------------------------------
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev sqlite-dev build-base

WORKDIR /app
COPY . .

RUN cargo build --release -p qubit-core -p qubit-cli

# Final lean runtime image
FROM alpine:3.20 AS runtime

RUN apk add --no-cache ca-certificates tzdata sqlite-libs libgcc && \
    addgroup -g 1000 qubit && \
    adduser -D -u 1000 -G qubit -h /home/qubit qubit && \
    mkdir -p /var/lib/qubit /etc/qubit && \
    chown -R qubit:qubit /var/lib/qubit /etc/qubit

WORKDIR /home/qubit

COPY --from=builder /app/target/release/qubit-core /usr/local/bin/qubit-core
COPY --from=builder /app/target/release/qubit-cli /usr/local/bin/qubit-cli

USER qubit
ENV QUBIT_PORT=4000 \
    QUBIT_HOST=0.0.0.0 \
    QUBIT_STORAGE_PATH=/var/lib/qubit/qubit_data.db \
    RUST_LOG=info

EXPOSE 4000

HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -qO- http://127.0.0.1:4000/health || exit 1

ENTRYPOINT ["/usr/local/bin/qubit-core"]
