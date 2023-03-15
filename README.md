# DivviUp API Server

## API Specification:
<b style="color:red">TODO: This will either be an inline description of the API or a link to another markdown document or swagger definition</b>

## Configuring and running

## System requirements

* [Rust (current stable or nightly)](https://www.rust-lang.org/tools/install)
* [PostgreSQL](https://www.postgresql.org/)

### [Setting up Auth0](#auth0)

<b style="color:red">TODO: This section will be updated with step-by-step instructions to configure auth0 for this application from scratch.</b>

### Required environment variables

An example `.envrc` is provided for optional but recommended use with [`direnv`](https://direnv.net).

* `AUTH_URL` -- The auth0-hosted base url that we use for identity (see [auth0 config section](#auth0))
* `API_URL` -- The public-facing base url for this application
* `AUTH_CLIENT_ID` -- auth0-provided client id (see [auth0 config section](#auth0))
* `AUTH_CLIENT_SECRET` -- auth0-provided client secret (see [auth0 config section](#auth0))
* `SESSION_SECRET` -- A cryptographically-randomly secret that is at least 32 bytes long. Future note: trillium sessions support [secret rotation](https://docs.trillium.rs/trillium_sessions/struct.sessionhandler#method.with_older_secrets), but divviup-api does not yet use this
* `AUTH_AUDIENCE` -- this is not currently used for anything important and probably will go away, but for now you should set it to `https://api.divviup.org`
* `APP_URL` -- The public-facing url for the associated browser client application
* `DATABASE_URL` -- A [libpq-compatible postgres uri](https://www.postgresql.org/docs/current/libpq-connect.html#id-1.7.3.8.3.6)

### Optional binding environment variables

* `HOST` -- default `"localhost"`, on unix-like systems, the server can also be configured to bind to bsd/berkeley sockets by setting `HOST` to a filesystem path, in which case `PORT` is ignored
* `PORT` -- default `8080`
* `LISTEN_FD` -- if supplied on unix-like systems, if this is set to an open file descriptor number, the server will listen to that fd

## Migrating the database

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

## Running the server

```bash
$ cargo run
```

<details>
    <summary>
<h3>Optional: cargo-devserver</h3>
</summary>
For development purposes, you may want to install and use [`cargo devserver`](https://github.com/jbr/cargo-devserver), which binds to the environment's `PORT` and `HOST`, runs the server under a shared FD, recompiles the application in the background on source change and restarts the server on successful compilation.
</details>

## Security Notes

* We do not have CSRF protections because we only accept a custom content type for non-idempotent request methods such as POST, and have constrained CORS rules.


