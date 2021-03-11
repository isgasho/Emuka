FROM rust:1 as builder

RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    gcc-mingw-w64 \
    binutils-mingw-w64 \
    git \
    bison \
    bsdmainutils

WORKDIR /rgbds
ENV RGBDB_VERSION=0.4.2

RUN wget https://github.com/gbdev/rgbds/releases/download/v$RGBDB_VERSION/rgbds-$RGBDB_VERSION.tar.gz
RUN tar -xvf * && \
    cd rgbds && \
    make && \
    make install

WORKDIR /app

# Really wish I could build only build the dependencies but, alas, ...
COPY . .

RUN rustup target add x86_64-pc-windows-gnu
RUN cargo build --release --target x86_64-pc-windows-gnu

RUN strip ./target/x86_64-pc-windows-gnu/release/emuka-server.exe
RUN strip ./target/x86_64-pc-windows-gnu/release/emuka-client.exe

ARG USER=root
RUN chown -R $USER:$USER /app



FROM alpine:latest

ARG USER=root
USER $USER

WORKDIR /in

COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/emuka-client.exe .
COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/emuka-server.exe .

CMD sh -c "cp * /out"
