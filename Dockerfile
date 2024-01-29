FROM lukemathwalker/cargo-chef:latest-rust-1.74-alpine3.18 AS backend-chef

WORKDIR /src


FROM backend-chef AS backend-planner

COPY src-rust/ .
RUN apk add --no-cache musl-dev
RUN cargo chef prepare --recipe-path recipe.json


FROM backend-chef AS backend

ENV OPENSSL_DIR=/usr

COPY --from=backend-planner /src/recipe.json recipe.json
RUN apk add --no-cache openssl-dev
RUN cargo chef cook --release --recipe-path recipe.json
COPY src-rust/ .
RUN cargo build --release


FROM node:20.10-alpine3.18 as frontend

WORKDIR /src
COPY . .
RUN npm install
RUN npm run build


FROM alpine:3.18

WORKDIR /var/www
RUN apk add --no-cache gcompat
COPY --from=backend /src/target/release/coterm /var/www/
COPY --from=frontend /src/build /var/www/public

CMD ["/var/www/coterm"]