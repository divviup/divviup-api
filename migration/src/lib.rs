pub use sea_orm_migration::prelude::*;

mod m20230211_224741_create_tasks;
mod m20230211_224853_create_sessions;
mod m20230211_233835_create_accounts;
mod m20230217_211422_create_memberships;
mod m20230322_223043_add_fields_to_task;
mod m20230427_221953_create_queue;
mod m20230512_200213_make_task_max_batch_size_a_big_integer;
mod m20230512_202411_add_two_urls_to_every_task;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230211_224741_create_tasks::Migration),
            Box::new(m20230211_224853_create_sessions::Migration),
            Box::new(m20230211_233835_create_accounts::Migration),
            Box::new(m20230217_211422_create_memberships::Migration),
            Box::new(m20230322_223043_add_fields_to_task::Migration),
            Box::new(m20230427_221953_create_queue::Migration),
            Box::new(m20230512_200213_make_task_max_batch_size_a_big_integer::Migration),
            Box::new(m20230512_202411_add_two_urls_to_every_task::Migration),
        ]
    }
}
