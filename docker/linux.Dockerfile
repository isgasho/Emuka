FROM rust:1 as builder

RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    libasound2-dev \
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

RUN cargo build --release

ARG USER=root
RUN chown -R $USER:$USER /app



FROM alpine:latest

ARG USER=root
USER $USER

WORKDIR /in

COPY --from=builder /app/target/release/emuka-client .
COPY --from=builder /app/target/release/emuka-server .

CMD sh -c "cp * /out"
