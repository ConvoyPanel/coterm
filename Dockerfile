FROM rust:1.74-alpine3.18 as backend-builder

WORKDIR /src

COPY src-rust/ .

