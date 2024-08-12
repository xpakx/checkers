# Build stage
FROM rust:1.75 AS build
RUN apt-get -y update
RUN apt-get install musl-tools -y
RUN apt-get install libssl-dev make perl musl-dev -y
WORKDIR /app/src/
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new gateway
WORKDIR /app/src/gateway
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

RUN cargo add openssl --features vendored

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
MAINTAINER xpakx.github.io
COPY --from=build /usr/local/cargo/bin/checkers /usr/local/bin/gateway

EXPOSE 8000
CMD [ "gateway"]
