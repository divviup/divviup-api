use clap::{builder::PossibleValuesParser, command, Parser, ValueEnum};
use sea_orm_migration::{
    sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait, QueryOrder},
    seaql_migrations, MigratorTrait, SchemaManager,
};
use std::cmp::Ordering;
use tracing::info;
use tracing_subscriber::EnvFilter;

use migration::Migrator;

#[derive(Copy, Clone, ValueEnum, Debug)]
enum Direction {
    Up,
    Down,
}

/// Wraps the SeaORM migration library to provide additional features. Lets
/// you bring migrations up or down to a particular version. Allows dry-run
/// of migrations.
#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    #[arg(value_enum)]
    direction: Direction,
    #[arg(value_parser = available_migrations())]
    target_version: String,
    #[arg(short, long)]
    dry_run: bool,
    #[arg(short = 'u', long, env = "DATABASE_URL")]
    database_url: String,
}

#[async_std::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db = Database::connect(ConnectOptions::new(args.database_url)).await?;
    match args.direction {
        Direction::Up => migrate_up::<Migrator>(&db, args.dry_run, &args.target_version).await?,
        Direction::Down => {
            migrate_down::<Migrator>(&db, args.dry_run, &args.target_version).await?
        }
    }
    Ok(())
}

async fn migrate_up<M: MigratorTrait>(
    db: &DatabaseConnection,
    dry_run: bool,
    target: &str,
) -> Result<(), Error> {
    let target_index =
        migration_index::<M>(target).ok_or(Error::MigrationNotFound(target.to_string()))?;

    let migrations_range;
    let num_migrations = match latest_applied_migration(db).await {
        Ok(latest_migration) => {
            let latest_index =
                migration_index::<M>(&latest_migration).ok_or(Error::DbMigrationNotFound)?;
            match target_index.cmp(&latest_index) {
                Ordering::Less => return Err(Error::VersionTooOld(target.to_string())),
                Ordering::Equal => {
                    info!("no action taken, already at desired version");
                    return Ok(());
                }
                Ordering::Greater => {
                    migrations_range = (latest_index + 1)..=target_index;
                    target_index - latest_index
                }
            }
        }
        Err(err) if matches!(err, Error::DbNotInitialized) => {
            migrations_range = 0usize..=target_index;
            // The migration API takes "number of migrations to apply". If we have an
            // uninitialized database, and we want to apply the first migration (index 0),
            // then we still have to apply at least one migration.
            target_index + 1
        }
        Err(err) => return Err(err),
    };

    info!(
        "executing {num_migrations} up migration(s) to reach {target}: {:?}",
        Migrator::migrations()[migrations_range]
            .iter()
            .map(|m| m.name())
            .collect::<Vec<_>>()
    );
    if !dry_run {
        M::up(db, Some(u32::try_from(num_migrations)?))
            .await
            .map_err(Error::from)?
    }
    Ok(())
}

async fn migrate_down<M: MigratorTrait>(
    db: &DatabaseConnection,
    dry_run: bool,
    target: &str,
) -> Result<(), Error> {
    let latest_index = migration_index::<M>(&latest_applied_migration(db).await?)
        .ok_or(Error::DbMigrationNotFound)?;
    let target_index =
        migration_index::<M>(target).ok_or(Error::MigrationNotFound(target.to_string()))?;

    let num_migrations = match latest_index.cmp(&target_index) {
        Ordering::Less => return Err(Error::VersionTooNew(target.to_string())),
        Ordering::Equal => {
            info!("no action taken, already at desired version");
            return Ok(());
        }
        Ordering::Greater => latest_index - target_index,
    };

    info!(
        "executing {num_migrations} down migration(s) to reach {target}: {:?}",
        Migrator::migrations()[(target_index + 1)..=(latest_index)]
            .iter()
            .rev()
            .map(|m| m.name())
            .collect::<Vec<_>>()
    );
    if !dry_run {
        M::down(db, Some(u32::try_from(num_migrations)?))
            .await
            .map_err(Error::from)?
    }
    Ok(())
}

fn migration_index<M: MigratorTrait>(version: &str) -> Option<usize> {
    M::migrations().iter().position(|m| m.name() == version)
}

async fn latest_applied_migration(db: &DatabaseConnection) -> Result<String, Error> {
    if !SchemaManager::new(db).has_table("seaql_migrations").await? {
        // The migrations table has not been created.
        return Err(Error::DbNotInitialized);
    }
    Ok(seaql_migrations::Entity::find()
        .order_by_desc(seaql_migrations::Column::Version)
        .one(db)
        .await?
        // The migrations table exists, but no migrations have been applied.
        .ok_or(Error::DbNotInitialized)?
        .version)
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("DB error: {0}")]
    Db(#[from] DbErr),
    #[error("DB is not initialized with migrations table")]
    DbNotInitialized,
    #[error("migration applied to DB is not found in available migrations")]
    DbMigrationNotFound,
    #[error("migration version {0} not found in avaliable migrations")]
    MigrationNotFound(String),
    #[error("migration version {0} is older than the latest applied migration")]
    VersionTooOld(String),
    #[error("migration version {0} is newer than the latest applied migration")]
    VersionTooNew(String),
    #[error("error calculating number of migrations, too many migrations?: {0}")]
    OverflowError(#[from] std::num::TryFromIntError),
}

fn available_migrations() -> PossibleValuesParser {
    PossibleValuesParser::new(
        // Leak memory to give migration names 'static lifetime, so clap can
        // use them.
        Migrator::migrations()
            .into_iter()
            .map(|m| Box::leak(Box::new(m.name().to_owned())) as &'static str)
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::prelude::*;

    macro_rules! test_migration {
        ($name:ident, $table_name:ident) => {
            #[allow(non_camel_case_types)]
            struct $name;

            impl MigrationName for $name {
                fn name(&self) -> &str {
                    stringify!($name)
                }
            }

            #[async_trait::async_trait]
            impl MigrationTrait for $name {
                async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                    manager
                        .create_table(
                            Table::create()
                                .table($table_name::Table)
                                .col(ColumnDef::new($table_name::Id).uuid().primary_key())
                                .to_owned(),
                        )
                        .await?;
                    Ok(())
                }
                async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                    manager
                        .drop_table(Table::drop().table($table_name::Table).to_owned())
                        .await
                }
            }

            #[derive(Iden)]
            enum $table_name {
                Table,
                Id,
            }
        };
    }
    test_migration!(m20230101_000000_test_migration_1, TestTable1);
    test_migration!(m20230201_000000_test_migration_2, TestTable2);
    test_migration!(m20230301_000000_test_migration_3, TestTable3);
    test_migration!(m20230401_000000_test_migration_4, TestTable4);
    test_migration!(m20230501_000000_test_migration_5, TestTable5);

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![
                Box::new(m20230101_000000_test_migration_1),
                Box::new(m20230201_000000_test_migration_2),
                Box::new(m20230301_000000_test_migration_3),
                Box::new(m20230401_000000_test_migration_4),
                Box::new(m20230501_000000_test_migration_5),
            ]
        }
    }

    fn all_migrations() -> Vec<&'static str> {
        vec![
            "m20230101_000000_test_migration_1",
            "m20230201_000000_test_migration_2",
            "m20230301_000000_test_migration_3",
            "m20230401_000000_test_migration_4",
            "m20230501_000000_test_migration_5",
        ]
    }

    async fn test_database() -> DatabaseConnection {
        Database::connect(ConnectOptions::new("sqlite::memory:".to_string()))
            .await
            .unwrap()
    }

    async fn applied_migrations(db: &DatabaseConnection) -> Vec<String> {
        seaql_migrations::Entity::find()
            .order_by_asc(seaql_migrations::Column::Version)
            .all(db)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.version)
            .collect()
    }

    #[async_std::test]
    async fn migrate_to_up_latest() {
        let db = test_database().await;

        // To latest
        migrate_up::<TestMigrator>(&db, false, "m20230501_000000_test_migration_5")
            .await
            .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());

        // Ensure no-op
        migrate_up::<TestMigrator>(&db, false, "m20230501_000000_test_migration_5")
            .await
            .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());
    }

    #[async_std::test]
    async fn migrate_to_up() {
        let db = test_database().await;

        // To first
        migrate_up::<TestMigrator>(&db, false, "m20230101_000000_test_migration_1")
            .await
            .unwrap();
        assert_eq!(
            applied_migrations(&db).await,
            vec!["m20230101_000000_test_migration_1"]
        );

        // Dry run
        migrate_up::<TestMigrator>(&db, true, "m20230101_000000_test_migration_1")
            .await
            .unwrap();
        assert_eq!(
            applied_migrations(&db).await,
            vec!["m20230101_000000_test_migration_1"]
        );

        // To third
        migrate_up::<TestMigrator>(&db, false, "m20230301_000000_test_migration_3")
            .await
            .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // To non-existent
        let result = migrate_up::<TestMigrator>(&db, false, "foobar").await;
        assert!(matches!(result, Err(Error::MigrationNotFound(_))));
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // To old version
        let result =
            migrate_up::<TestMigrator>(&db, false, "m20230101_000000_test_migration_1").await;
        assert!(matches!(result, Err(Error::VersionTooOld(_))));
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);
    }

    #[async_std::test]
    async fn migrate_to_down() {
        let db = test_database().await;

        // To latest
        migrate_up::<TestMigrator>(&db, false, "m20230501_000000_test_migration_5")
            .await
            .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());

        // Ensure no-op
        migrate_down::<TestMigrator>(&db, false, "m20230501_000000_test_migration_5")
            .await
            .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());

        // Dry-run
        migrate_down::<TestMigrator>(&db, true, "m20230301_000000_test_migration_3")
            .await
            .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());

        // To third
        migrate_down::<TestMigrator>(&db, false, "m20230301_000000_test_migration_3")
            .await
            .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // To newer version
        let result =
            migrate_down::<TestMigrator>(&db, false, "m20230401_000000_test_migration_4").await;
        assert!(matches!(result, Err(Error::VersionTooNew(_))));
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // To non-existent version
        let result = migrate_down::<TestMigrator>(&db, false, "foobar").await;
        assert!(matches!(result, Err(Error::MigrationNotFound(_))));
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // Upgrade back to fourth, ensure we can still upgrade again.
        migrate_up::<TestMigrator>(&db, false, "m20230401_000000_test_migration_4")
            .await
            .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations()[..4]);
    }

    #[test]
    fn ensure_migrations_are_sorted() {
        // Migrations in Migrator must be in lexicographic order, otherwise
        // this CLI will not work correctly.
        assert!(Migrator::migrations()
            .windows(2)
            .all(|window| window[0].name() <= window[1].name()))
    }
}
