FROM node:alpine as assets
WORKDIR /src/app
COPY app/package.json /src/app/package.json
COPY app/package-lock.json /src/app/package-lock.json
RUN npm ci
COPY app /src/app
RUN npm ci
RUN npm run build

FROM rust:1.69.0-alpine as builder
RUN apk add libc-dev
WORKDIR /src
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY build.rs /src/build.rs
COPY migration /src/migration
COPY src /src/src
COPY --from=assets /src/app/build /src/app/build
ARG RUST_FEATURES=default
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    ASSET_DIR=/src/app/build \
    cargo build --profile release -p migration && \
    cp /src/target/release/migration /migration
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    ASSET_DIR=/src/app/build \
    cargo build --profile release --features ${RUST_FEATURES} && \
    cp /src/target/release/divviup_api_bin /divviup_api_bin

FROM alpine:3.17.3
ARG GIT_REVISION=unknown
LABEL revision ${GIT_REVISION}
EXPOSE 8080
ENV HOST=0.0.0.0
COPY --from=builder /migration /migration
COPY --from=builder /divviup_api_bin /divviup_api_bin
ENTRYPOINT ["/divviup_api_bin"]
