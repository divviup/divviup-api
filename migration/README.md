# Running Migrator CLI

- Generate a new migration file
    ```sh
    cargo run -- migrate generate MIGRATION_NAME
    ```
- Apply all pending migrations
    ```sh
    cargo run
    ```
    ```sh
    cargo run -- up
    ```
- Apply first 10 pending migrations
    ```sh
    cargo run -- up -n 10
    ```
- Rollback last applied migrations
    ```sh
    cargo run -- down
    ```
- Rollback last 10 applied migrations
    ```sh
    cargo run -- down -n 10
    ```
- Drop all tables from the database, then reapply all migrations
    ```sh
    cargo run -- fresh
    ```
- Rollback all applied migrations, then reapply all migrations
    ```sh
    cargo run -- refresh
    ```
- Rollback all applied migrations
    ```sh
    cargo run -- reset
    ```
- Check the status of all migrations
    ```sh
    cargo run -- status
    ```

# Running extended migrator CLI

There is a CLI at `bin/migrate_to.rs` that covers some gaps in the standard SeaORM
migrator CLI. It's most useful for running in automation.

- To apply all migrations up to the given migration
    ```sh
    cargo run --bin migrate_to -- up MIGRATION_NAME
    ```
- To dry-run all migrations up to the given migration
    ```sh
    cargo run --bin migrate_to -- --dry-run up MIGRATION_NAME
    ```
- To downgrade migrations down to the given migration
    ```sh
    cargo run --bin migrate_to -- down MIGRATION_NAME
    ```
