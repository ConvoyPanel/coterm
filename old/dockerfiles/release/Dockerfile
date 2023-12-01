FROM lukemathwalker/cargo-chef:latest-rust-1.69-alpine AS chef
RUN apk add --no-cache musl-dev
WORKDIR /var/www/coterm

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build-backend
COPY --from=planner /var/www/coterm/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

FROM node:18-alpine as build-frontend
WORKDIR /var/www/coterm
COPY . .
RUN npm install
RUN npm run build

FROM caddy:2.6-alpine
LABEL org.opencontainers.image.source="https://github.com/convoypanel/coterm"
WORKDIR /var/www/coterm
COPY --from=build-backend /var/www/coterm/target/release/coterm .
RUN chmod +x ./coterm
COPY --from=build-frontend /var/www/coterm/dist ./dist
COPY ./dockerfiles/release/Caddyfile /etc/caddy/Caddyfile
COPY ./dockerfiles/release/startup.sh ./startup.sh
RUN chmod +x ./startup.sh
RUN cat ./startup.sh
CMD ["sh", "./startup.sh"]