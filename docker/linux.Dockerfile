FROM rust:1 as rust-builder

WORKDIR /app

RUN cargo install cargo-build-deps

RUN USER=root cargo new emuka

WORKDIR /app/emuka

COPY Cargo.toml Cargo.lock ./
COPY emuka-server/Cargo.toml ./emuka-server/Cargo.toml
COPY emuka-client/Cargo.toml ./emuka-client/Cargo.toml

RUN cargo build-deps --release

COPY . .

RUN cargo build  --release
