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

## System requirements
* [NodeJS](https://nodejs.org/) and [npm](https://www.npmjs.com/)
* [Rust (current stable or nightly)](https://www.rust-lang.org/tools/install)
* [PostgreSQL](https://www.postgresql.org/)

### Required environment variables

An example `.envrc` is provided for optional but recommended use with [`direnv`](https://direnv.net).

* `AUTH_URL` -- The Auth0-hosted base url that we use for identity
* `API_URL` -- The public-facing base url for this application
* `AUTH_CLIENT_ID` -- Auth0-provided client id
* `AUTH_CLIENT_SECRET` -- Auth0-provided client secret
* `SESSION_SECRETS` -- Comma-joined base64url-encoded, without padding,
  cryptographically-randomly secrets that are each at least 32 bytes long
  after base64url decoding. The first one will be used for new sessions.
* `AUTH_AUDIENCE` -- The OAuth 2 audience, for use when authenticating users via Auth0
* `APP_URL` -- The public-facing url for the associated browser client application
* `DATABASE_URL` -- A [libpq-compatible postgres uri](https://www.postgresql.org/docs/current/libpq-connect.html#id-1.7.3.8.3.6)
* `POSTMARK_TOKEN` -- the token from the transactional stream from a [postmark](https://postmarkapp.com) account
* `EMAIL_ADDRESS` -- the address this deployment should send from
* `DATABASE_ENCRYPTION_KEYS` -- Comma-joined url-safe-no-pad base64'ed 16 byte cryptographically-random keys. The first one will be used to encrypt aggregator API authentication tokens at rest in the database

### Optional binding environment variables

* `HOST` -- default `"localhost"`, on unix-like systems, the server can also be configured to bind to bsd/berkeley sockets by setting `HOST` to a filesystem path, in which case `PORT` is ignored
* `PORT` -- default `8080`
* `LISTEN_FD` -- if supplied on unix-like systems, if this is set to an open file descriptor number, the server will listen to that fd
* `OTEL_EXPORTER_PROMETHEUS_HOST` -- default `"localhost"`
* `OTEL_EXPORTER_PROMETHEUS_PORT` -- default `9464`
* `ASSET_DIR` -- set this to skip building the react app and include react app assets from a different directory

## Initial setup

### Migrating the database

* First, create the database referred to by `DATABASE_URL` in your environment. This may an invocation of [`createdb`](https://www.postgresql.org/docs/current/app-createdb.html) if running locally.
* `cargo run -p migration -- up` will bring the application up to the current schema
* For more options, execute `cargo run -p migration -- --help` emits this:

<details>
  <summary><code>cargo run -p migration -- --help</code></summary>
  
```
sea-orm-migration 0.11.0

USAGE:
    migration [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -h, --help
            Print help information

    -s, --database-schema <DATABASE_SCHEMA>
            Database schema
             - For MySQL and SQLite, this argument is ignored.
             - For PostgreSQL, this argument is optional with default value 'public'.
            [env: DATABASE_SCHEMA=]

    -u, --database-url <DATABASE_URL>
            Database URL
            
            [env: DATABASE_URL=postgres://localhost/divviup_dev]

    -v, --verbose
            Show debug messages

    -V, --version
            Print version information

SUBCOMMANDS:
    init
            Initialize migration directory
    generate
            Generate a new, empty migration
    fresh
            Drop all tables from the database, then reapply all migrations
    refresh
            Rollback all applied migrations, then reapply all migrations
    reset
            Rollback all applied migrations
    status
            Check the status of all migrations
    up
            Apply pending migrations
    down
            Rollback applied migrations
    help
            Print this message or the help of the given subcommand(s)
```
</details>

### Installing npm dependencies

```bash
$ cd app && npm ci && cd -
```

## Running the server

This service has dependencies on several external services. In order to support development and testing, there is an `api-mocks` cargo feature to stub out all external services including aggregator apis, auth0, and postmark.

As such, to run a standalone development server,

```bash
$ cargo run --features api-mocks
```

### Embedded React App

If an environment variable `ASSET_DIR` is available, all files in that directory will be served as a virtual host on `APP_URL`.

```bash
$ cd app && npm ci && npm run build && cd - && env ASSET_DIR=app/build cargo run
```

### Running the React development server

Configure the rust app environment to point `APP_URL` to whatever port you're using here, eg `env APP_URL=http://localhost:8082 cargo run --features api-mocks`

```bash
$ cd app && PORT=8082 npm start
```


## Security Notes

* We do not have CSRF protections because we only accept a custom content type for non-idempotent request methods such as POST, and have constrained CORS rules.


