# Build stage
FROM rust:1.75 AS build
RUN apt-get -y update
RUN apt-get install musl-tools -y
WORKDIR /app/src/
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new main
WORKDIR /app/src/main
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
COPY migrations ./migrations
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
MAINTAINER xpakx.github.io
COPY --from=build /usr/local/cargo/bin/checkers /usr/local/bin/main

EXPOSE 8080
CMD [ "main"]
