FROM rust:1.68.2-alpine as builder
RUN apk add libc-dev
WORKDIR /src
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY migration /src/migration
COPY src /src/src
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/src/target cargo build --profile release -p migration && cp /src/target/release/migration /migration
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/src/target cargo build --profile release && cp /src/target/release/divviup_api_bin /divviup_api_bin

FROM alpine:3.17.2
COPY --from=builder /migration /migration
COPY --from=builder /divviup_api_bin /divviup_api_bin
ENTRYPOINT ["/divviup_api_bin"]
