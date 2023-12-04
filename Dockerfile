FROM rust:1.74-alpine3.18 as backend

WORKDIR /src
COPY src-rust/ .
RUN apk add --no-cache musl-dev
RUN cargo build --release


FROM node:20.10-alpine3.18 as frontend

WORKDIR /src
COPY . .

RUN npm install && npm run build


FROM alpine:3.18

WORKDIR /var/www
COPY --from=backend /src/target/release/coterm /var/www/
COPY --from=frontend /src/build /var/www/public


CMD ["/var/www/coterm"]