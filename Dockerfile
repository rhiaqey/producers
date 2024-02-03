FROM rust:1.75-slim-bullseye as builder

ARG BINARY
ARG FEATURES

ENV RUST_BACKTRACE=1

RUN apt-get update \
    && apt-get install -y \
      cmake \
      pkg-config \
      libssl-dev \
      protobuf-compiler

WORKDIR /usr/src/

COPY . .

RUN cargo install --bin ${BINARY} --features ${FEATURES} --path .

FROM debian:bullseye-slim

ARG BINARY
ARG USER=1001

ENV BINARY=$BINARY
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_BACKTRACE=1
ENV RUST_LOG=trace
ENV USER=$USER

LABEL org.opencontainers.image.description="Rhiaqey Producer ${BINARY}"

RUN apt-get update \
    && apt-get install -y \
      ca-certificates \
      net-tools \
      libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates
RUN useradd -ms /bin/bash $USER

COPY --from=builder --chown=$USER:$USER /usr/local/cargo/bin/${BINARY} /usr/local/bin/${BINARY}

USER $USER

CMD [ "sh", "-c", "${BINARY}" ]
