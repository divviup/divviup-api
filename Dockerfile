FROM rust:1.68.2-alpine as builder
RUN apk add libc-dev
RUN apk add --update npm
WORKDIR /src
COPY app /src/app
COPY Cargo.toml /src/Cargo.toml
COPY Cargo.lock /src/Cargo.lock
COPY build.rs /src/build.rs
COPY migration /src/migration
COPY src /src/src
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/src/target cargo build --profile release -p migration && cp /src/target/release/migration /migration
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/src/target cargo build --profile release && cp /src/target/release/divviup_api_bin /divviup_api_bin

FROM alpine:3.17.3
ARG GIT_REVISION=unknown
LABEL revision ${GIT_REVISION}
EXPOSE 8080
ENV HOST=0.0.0.0
COPY --from=builder /migration /migration
COPY --from=builder /divviup_api_bin /divviup_api_bin
ENTRYPOINT ["/divviup_api_bin"]
