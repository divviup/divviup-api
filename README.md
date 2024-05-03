# Divvi Up API Server and web app

## Badges

* [![Coverage Status](https://coveralls.io/repos/github/divviup/divviup-api/badge.svg?branch=main)](https://coveralls.io/github/divviup/divviup-api?branch=main)
* ![Rust CI](https://github.com/divviup/divviup-api/actions/workflows/rust.yml/badge.svg?branch=main)
* ![TypeScript CI](https://github.com/divviup/divviup-api/actions/workflows/ts.yml/badge.svg?branch=main)
* ![Docker Build](https://github.com/divviup/divviup-api/actions/workflows/docker.yml/badge.svg?branch=main)

## API Specification:
* [production ui](https://app.divviup.org/swagger-ui)
* [staging ui](https://app.staging.divviup.org/swagger-ui)
* [main (yml)](https://github.com/divviup/divviup-api/blob/main/documentation/openapi.yml)

## Configuring and running

### System requirements
* [NodeJS](https://nodejs.org/) and [npm](https://www.npmjs.com/)
* [Rust (current stable or nightly)](https://www.rust-lang.org/tools/install)
* [PostgreSQL](https://www.postgresql.org/)
* [docker-compose](https://docs.docker.com/compose/)

Some Rust dependencies require additional system dependencies. These can be installed with your usual
package manager:
* C compiler (GCC or Clang)
* CMake

## Local Development

### Quick Start

This will get you up and running quickly for development purposes.

1. Modify your `/etc/hosts` to contain:
    ```
    127.0.0.1   app.divviup.local api.divviup.local
    ::1         app.divviup.local api.divviup.local
    ```
1. Clone the repository and navigate to its root.
1. Execute `docker-compose watch`.
1. Navigate in your browser to `http://app.divviup.local:8080/`.

`docker-compose` will automatically reload containers when you make changes. Data is persisted
until you `docker-compose rm --volumes`.

Two Janus aggregators will be created for you, but are not automatically paired to divviup-api.
Their information is:
1. Address: `http://janus_1_aggregator:8080/aggregator-api`, Token: `0000`
1. Address: `http://janus_2_aggregator:8080/aggregator-api`, Token: `0000`

If you need to talk to these aggregators from outside docker-compose's network namespace, e.g.
with a testing client, they are mapped to `localhost:9001` and `localhost:9002`, respectively.

PostgreSQL is exposed on port 5432 with username and password `postgres`.

If you need to iterate on database migrations, you may wish to disable the `divviup_api_migrate`
service by commenting it out in `compose.yaml`.

### Faster frontend iteration

For iterating on the frontend, docker-compose can be slow since the divviup-api backend needs to
be recompiled for every frontend change. Instead, run the frontend separately from the backend to
take advantage of the speed of [Vite](https://vitejs.dev/).

1. Create the `api_url` file so the app knows where to hit the API.
    ```bash
    echo "http://api.divviup.local:8080/" >app/public/api_url
    ```
2. Run the backend with the `no-app` override:
    ```bash
    docker-compose -f compose.yaml -f compose.no-app.yaml watch
    ```
3. In a separate terminal, run the frontend:
    ```bash
    cd app
    npm i && npm start
    ```
4. Navigate to the app at `http://localhost:8081`.

This should still retain automatic reload for backend and frontend, but frontend changes will be
reflected in your browser much faster.

## Security Notes

* We do not have CSRF protections because we only accept a custom content type for non-idempotent request methods such as POST, and have constrained CORS rules.
