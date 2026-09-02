FROM node:22-alpine AS web
WORKDIR /app
COPY package.json package-lock.json* tsconfig.json vite.config.ts index.html app.html ./
COPY frontend ./frontend
RUN npm ci --ignore-scripts && npm run build

FROM rust:1-alpine AS server
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
ARG BUILD_SHA=unknown
ENV BUILD_SHA=${BUILD_SHA}
RUN cargo build --release

FROM alpine:3.22
RUN addgroup -S app && adduser -S -G app app && mkdir -p /data && chown app:app /data
WORKDIR /app
COPY --from=server /app/target/release/math-circle-board /usr/local/bin/math-circle-board
COPY --from=web /app/dist ./dist
ENV DATA_DIR=/data DIST_DIR=/app/dist
USER app
EXPOSE 8080
VOLUME ["/data"]
CMD ["math-circle-board"]
