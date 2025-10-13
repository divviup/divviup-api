FROM node:alpine AS assets
WORKDIR /src/app
COPY app/package.json /src/app/package.json
COPY app/package-lock.json /src/app/package-lock.json
COPY documentation /src/documentation
RUN npm ci
COPY app /src/app
RUN npm ci
RUN npm run build

FROM rust:1.90.0-alpine AS chef
RUN apk --no-cache add libc-dev cmake make
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo install cargo-chef
WORKDIR /src

FROM chef AS planner
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY build.rs /src/build.rs
COPY migration /src/migration
COPY src /src/src
COPY test-support /src/test-support
COPY client /src/client
COPY cli /src/cli
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG RUST_FEATURES=default
ARG RUST_PROFILE=release
COPY --from=planner /src/recipe.json /src/recipe.json
RUN cargo chef cook --workspace --profile ${RUST_PROFILE} --recipe-path recipe.json
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY build.rs /src/build.rs
COPY migration /src/migration
COPY src /src/src
COPY test-support /src/test-support
COPY client /src/client
COPY cli /src/cli
COPY --from=assets /src/app/build /src/app/build
RUN ASSET_DIR=/src/app/build cargo build --workspace --bins --profile ${RUST_PROFILE} --features ${RUST_FEATURES}
# Hack: --profile=dev outputs to target/debug, so to interpolate $RUST_PROFILE in the next stage,
# we need to create this symlink.
RUN ln -s debug target/dev

FROM alpine:3.22.2 AS final
ARG GIT_REVISION=unknown
ARG RUST_PROFILE=release
LABEL revision=${GIT_REVISION}
EXPOSE 8080
ENV HOST=0.0.0.0
COPY --from=builder /src/target/${RUST_PROFILE}/migration /migration
COPY --from=builder /src/target/${RUST_PROFILE}/migrate_to /migrate_to
COPY --from=builder /src/target/${RUST_PROFILE}/divviup_api_bin /divviup_api_bin
COPY --from=builder /src/target/${RUST_PROFILE}/divviup /divviup
ENTRYPOINT ["/divviup_api_bin"]
