FROM node:alpine as assets
WORKDIR /src/app
COPY app/package.json /src/app/package.json
COPY app/package-lock.json /src/app/package-lock.json
RUN npm ci
COPY app /src/app
RUN npm ci
RUN npm run build

FROM rust:1.70.0-alpine as chef
RUN apk --no-cache add libc-dev
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo install cargo-chef
WORKDIR /src

FROM chef AS planner
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY build.rs /src/build.rs
COPY migration /src/migration
COPY src /src/src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /src/recipe.json /src/recipe.json
RUN cargo chef cook --workspace --release --recipe-path recipe.json
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY build.rs /src/build.rs
COPY migration /src/migration
COPY src /src/src
COPY --from=assets /src/app/build /src/app/build
ARG RUST_FEATURES=default
RUN ASSET_DIR=/src/app/build cargo build --workspace --release --features ${RUST_FEATURES}

FROM alpine:3.18.2 AS final
ARG GIT_REVISION=unknown
LABEL revision ${GIT_REVISION}
EXPOSE 8080
ENV HOST=0.0.0.0
COPY --from=builder /src/target/release/migrate_to /migrate_to
COPY --from=builder /src/target/release/divviup_api_bin /divviup_api_bin
ENTRYPOINT ["/divviup_api_bin"]
