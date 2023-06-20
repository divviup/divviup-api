use clap::{builder::PossibleValuesParser, command, Parser, ValueEnum};
use migration::{
    sea_orm::{ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryOrder},
    MigratorTrait,
};
use sea_orm_migration::{prelude::*, seaql_migrations};
use tracing::{info, Level};

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
        .with_max_level(Level::DEBUG)
        .init();

    let db = Database::connect(ConnectOptions::new(args.database_url)).await?;
    migrate_to::<migration::Migrator>(&db, args.dry_run, args.direction, &args.target_version)
        .await?;
    Ok(())
}

async fn migrate_to<M: MigratorTrait>(
    db: &DatabaseConnection,
    dry_run: bool,
    dir: Direction,
    target: &str,
) -> Result<(), Error> {
    let latest_index = match latest_applied_migration(db).await {
        Ok(m) => migration_index::<M>(&m).ok_or(Error::DBMigrationNotFound)?,
        Err(_) => -1, // Empty database.
    };
    let target_index =
        migration_index::<M>(target).ok_or(Error::MigrationNotFound(target.to_string()))?;
    if target_index == latest_index {
        info!("no action taken, already at desired version");
        return Ok(());
    }

    match dir {
        Direction::Up => match target_index - latest_index {
            num_migrations if num_migrations > 0 => {
                info!("executing {num_migrations} migration(s) to {target}");
                if !dry_run {
                    M::up(db, Some(num_migrations as u32))
                        .await
                        .map_err(Error::from)?
                }
                Ok(())
            }
            _ => Err(Error::VersionTooOld(target.to_string())),
        },
        Direction::Down => match latest_index - target_index {
            num_migrations if num_migrations > 0 => {
                info!("executing {num_migrations} migration(s) to {target}");
                if !dry_run {
                    M::down(db, Some(num_migrations as u32))
                        .await
                        .map_err(Error::from)?
                }
                Ok(())
            }
            _ => Err(Error::VersionTooNew(target.to_string())),
        },
    }
}

fn migration_index<M: MigratorTrait>(version: &str) -> Option<i64> {
    M::migrations()
        .iter()
        .position(|m| m.name() == version)
        .map(|i| i as i64)
}

async fn latest_applied_migration(db: &DatabaseConnection) -> Result<String, Error> {
    Ok(seaql_migrations::Entity::find()
        .order_by_desc(seaql_migrations::Column::Version)
        .one(db)
        .await?
        .ok_or(Error::DbNotInitialized)?
        .version)
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("DB error: {0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("DB is not initialized with migrations table")]
    DbNotInitialized,
    #[error("migration applied to DB is not found in available migrations")]
    DBMigrationNotFound,
    #[error("migration version {0} not found in avaliable migrations")]
    MigrationNotFound(String),
    #[error("migration version {0} is older than the latest applied migration")]
    VersionTooOld(String),
    #[error("migration version {0} is newer than the latest applied migration")]
    VersionTooNew(String),
}

fn available_migrations() -> PossibleValuesParser {
    PossibleValuesParser::new(
        // Leak memory to give migration names 'static lifetime, so clap can
        // use them.
        migration::Migrator::migrations()
            .into_iter()
            .map(|m| Box::leak(Box::new(m.name().to_owned())) as &'static str)
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Up,
            "m20230501_000000_test_migration_5",
        )
        .await
        .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());

        // Ensure no-op
        migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Up,
            "m20230501_000000_test_migration_5",
        )
        .await
        .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());
    }

    #[async_std::test]
    async fn migrate_to_up() {
        let db = test_database().await;

        // To first
        migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Up,
            "m20230101_000000_test_migration_1",
        )
        .await
        .unwrap();
        assert_eq!(
            applied_migrations(&db).await,
            vec!["m20230101_000000_test_migration_1"]
        );

        // Dry run
        migrate_to::<TestMigrator>(
            &db,
            true,
            Direction::Up,
            "m20230101_000000_test_migration_1",
        )
        .await
        .unwrap();
        assert_eq!(
            applied_migrations(&db).await,
            vec!["m20230101_000000_test_migration_1"]
        );

        // To third
        migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Up,
            "m20230301_000000_test_migration_3",
        )
        .await
        .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // To non-existent
        let result = migrate_to::<TestMigrator>(&db, false, Direction::Up, "foobar").await;
        assert!(matches!(result, Err(Error::MigrationNotFound(_))));
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // To old version
        let result = migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Up,
            "m20230101_000000_test_migration_1",
        )
        .await;
        assert!(matches!(result, Err(Error::VersionTooOld(_))));
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);
    }

    #[async_std::test]
    async fn migrate_to_down() {
        let db = test_database().await;

        // To latest
        migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Up,
            "m20230501_000000_test_migration_5",
        )
        .await
        .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());

        // Ensure no-op
        migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Down,
            "m20230501_000000_test_migration_5",
        )
        .await
        .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());

        // Dry-run
        migrate_to::<TestMigrator>(
            &db,
            true,
            Direction::Down,
            "m20230301_000000_test_migration_3",
        )
        .await
        .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations());

        // To third
        migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Down,
            "m20230301_000000_test_migration_3",
        )
        .await
        .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // To newer version
        let result = migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Down,
            "m20230401_000000_test_migration_4",
        )
        .await;
        assert!(matches!(result, Err(Error::VersionTooNew(_))));
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // To non-existent version
        let result = migrate_to::<TestMigrator>(&db, false, Direction::Down, "foobar").await;
        assert!(matches!(result, Err(Error::MigrationNotFound(_))));
        assert_eq!(applied_migrations(&db).await, all_migrations()[..3]);

        // Upgrade back to fourth, ensure we can still upgrade again.
        migrate_to::<TestMigrator>(
            &db,
            false,
            Direction::Up,
            "m20230401_000000_test_migration_4",
        )
        .await
        .unwrap();
        assert_eq!(applied_migrations(&db).await, all_migrations()[..4]);
    }

    #[test]
    fn ensure_migrations_are_sorted() {
        // Migrations in migration::Migrator must be in lexicographic order, otherwise
        // this CLI will not work correctly.
        assert!(migration::Migrator::migrations()
            .windows(2)
            .all(|window| window[0].name() <= window[1].name()))
    }
}
