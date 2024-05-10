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
1. Execute `docker compose watch`.
1. Navigate in your browser to `http://localhost:8081/`.

`docker compose` will automatically reload containers when you make changes. Data is persisted
until you `docker compose rm --volumes`.

Two Janus aggregators will be created for you, but are not automatically paired to divviup-api.
Their information is:
1. Address: `http://janus_1_aggregator:8080/aggregator-api`, Token: `0000`
1. Address: `http://janus_2_aggregator:8080/aggregator-api`, Token: `0000`

If you need to talk to these aggregators from outside compose's network namespace, e.g. with a
testing client, they are mapped to `localhost:9001` and `localhost:9002`, respectively.

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
