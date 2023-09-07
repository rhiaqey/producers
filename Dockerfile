FROM rust:1.72-slim-bullseye as builder
ARG BINARY
ARG FEATURES
RUN apt-get update \
    && apt-get install -y \
      cmake \
      pkg-config \
      libssl-dev \
      protobuf-compiler
WORKDIR /usr/src/
COPY . .
ENV RUST_BACKTRACE=1
RUN cargo install --bin ${BINARY} --features ${FEATURES} --path .

FROM debian:bullseye-slim
ARG BINARY
ENV BINARY=$BINARY
ENV DEBIAN_FRONTEND=noninteractive
LABEL org.opencontainers.image.description="Rhiaqey producer"
RUN apt-get update \
    && apt-get install -y \
      ca-certificates \
      net-tools \
      libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates
COPY --from=builder /usr/local/cargo/bin/${BINARY} /usr/local/bin/${BINARY}
ENV RUST_BACKTRACE=1
ENV RUST_LOG=trace
CMD [ "sh", "-c", "${BINARY}" ]
