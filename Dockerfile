FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /src

FROM chef AS planner
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY build.rs /src/build.rs
COPY migration /src/migration
COPY src /src/src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS api-builder
COPY --from=planner /src/recipe.json /src/recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path /src/recipe.json
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY build.rs /src/build.rs
COPY migration /src/migration
COPY src /src/src
RUN touch build.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release --target x86_64-unknown-linux-musl

FROM chef AS migration-builder
COPY --from=planner /src/recipe.json /src/recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path /src/recipe.json -p migration
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY migration /src/migration
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release --target x86_64-unknown-linux-musl -p migration

FROM alpine:3.17.3 AS runtime
ARG GIT_REVISION=unknown
LABEL revision ${GIT_REVISION}
EXPOSE 8080
ENV HOST=0.0.0.0
COPY --from=api-builder /src/target/x86_64-unknown-linux-musl/release/divviup_api_bin /divviup_api_bin
COPY --from=migration-builder /src/target/x86_64-unknown-linux-musl/release/migration /migration
ENTRYPOINT ["/divviup_api_bin"]
