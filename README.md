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
* Docker
  * On MacOS, ensure your docker VM has sufficient resources, at least 2vCPUs and 8GB of RAM.
* [docker-compose](https://docs.docker.com/compose/) >=v2.27.0
  * On MacOS, [install this through `brew`][brew]. Notice this calls for extra modifications to
    `~/.docker/config.json`.
  * On Linux, install `docker-compose-plugin` from the `docker-ce` repository. See the OS-specific
    [instructions here][linux].
  * Alternatively, for both platforms, you can [install the binary plugin][compose].

Some Rust dependencies require additional system dependencies. These can be installed with your usual
package manager:
* C compiler (GCC or Clang)
* CMake

[brew]: https://formulae.brew.sh/formula/docker-compose
[linux]: https://docs.docker.com/engine/install/
[compose]: https://github.com/docker/compose/releases

### Local Development

This will get you up and running quickly for development purposes.

1. Clone the repository and navigate to its root.
1. Execute `echo "http://localhost:8080" >app/public/api_url`
1. Execute `docker compose up`.
1. Navigate in your browser to `http://localhost:8081/`.

Data is persisted until you `docker compose rm --volumes`.

If you want to use image versions besides the defaults, you can use environment variables
`JANUS_AGGREGATOR_IMAGE`, `JANUS_MIGRATOR_IMAGE`, `DIVVIUP_API_IMAGE` and
`DIVVIUP_API_MIGRATOR_IMAGE` when invoking `docker compose`.  For example:

```bash
DIVVIUP_API_IMAGE=divviup_api:localversion \
  JANUS_IMAGE=us-west2-docker.pkg.dev/divviup-artifacts-public/janus/janus_aggregator:0.7.18 \
  docker compose up
```

`divviup_api:localversion` will be pulled from the local Docker repository and
`us-west2-docker.pkg.dev/divviup-artifacts-public/janus/janus_aggregator:0.7.18` will be pulled from
`us-west2-docker.pkg.dev`.

We also provide `compose.dev.yaml`, which will build `divviup-api` from local sources. Try:

```bash
docker compose -f compose.dev.yaml watch
```

`docker compose` will automatically reload containers when you make changes. The `JANUS_IMAGE` and
`JANUS_MIGRATOR_IMAGE` variables are honored by `compose.dev.yaml`.

An account named `demo` is also automatically created. Two Janus aggregators will be created for
you, and are automatically paired to divviup-api. Their information is:

| Name | Aggregator API address | Aggregator API auth token | Paired with | DAP API outside compose network |
| ---- | ---------------------- | ------------------------- | ----------- | ------------------------------- |
| `leader` | `http://janus_1_aggregator:8080/aggregator-api` | `0000` | Shared, first party | `localhost:9001` |
| `helper` | `http://janus_2_aggregator:8080/aggregator-api` | `0000` | `demo` account | `localhost:9002` |

If using the divviup CLI, consider compiling with the `--features admin` option. Also, set these
environment variables.

```bash
# This token is intentionally blank, but you'll still need to set the variable.
export DIVVIUP_TOKEN=
export DIVVIUP_API_URL=http://localhost:8080

# Set this for any account-specific commands. Since divviup-api in dev mode will automatically
# authenticate you as an admin, it won't know which account to target.
export DIVVIUP_ACCOUNT_ID={account uuid}
```

PostgreSQL is exposed on port 5432 with username and password `postgres`.

If you need to iterate on database migrations, you may wish to disable the `divviup_api_migrate`
service by commenting it out in `compose.yaml`.

## Security Notes

* We do not have CSRF protections because we only accept a custom content type for non-idempotent
  request methods such as POST, and have constrained CORS rules.
