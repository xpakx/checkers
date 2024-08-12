# Build stage
FROM rust:1.75 AS build
RUN apt-get -y update
RUN apt-get install musl-tools -y
WORKDIR /app/src/
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new game-engine
WORKDIR /app/src/game-engine
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
MAINTAINER xpakx.github.io
COPY --from=build /usr/local/cargo/bin/checkers /usr/local/bin/game-engine

EXPOSE 8080
CMD [ "game-engine"]
