pub use sea_orm_migration::prelude::*;

mod m20230211_224741_create_tasks;
mod m20230211_224853_create_sessions;
mod m20230211_233835_create_accounts;
mod m20230217_211422_create_memberships;
mod m20230322_223043_add_fields_to_task;
mod m20230427_221953_create_queue;
mod m20230512_200213_make_task_max_batch_size_a_big_integer;
mod m20230512_202411_add_two_urls_to_every_task;
mod m20230616_223923_create_aggregators;
mod m20230620_195535_add_aggregators_to_tasks;
mod m20230622_232534_make_aggregator_api_url_mandatory;
mod m20230626_183248_add_is_first_party_to_aggregators;
mod m20230630_175314_create_api_tokens;
mod m20230703_201332_add_additional_fields_to_api_tokens;
mod m20230725_220134_add_vdafs_and_query_types_to_aggregators;
mod m20230731_181722_rename_aggregator_bearer_token;
mod m20230808_204859_create_hpke_config;

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
            Box::new(m20230616_223923_create_aggregators::Migration),
            Box::new(m20230620_195535_add_aggregators_to_tasks::Migration),
            Box::new(m20230622_232534_make_aggregator_api_url_mandatory::Migration),
            Box::new(m20230626_183248_add_is_first_party_to_aggregators::Migration),
            Box::new(m20230630_175314_create_api_tokens::Migration),
            Box::new(m20230703_201332_add_additional_fields_to_api_tokens::Migration),
            Box::new(m20230725_220134_add_vdafs_and_query_types_to_aggregators::Migration),
            Box::new(m20230731_181722_rename_aggregator_bearer_token::Migration),
            Box::new(m20230808_204859_create_hpke_config::Migration),
        ]
    }
}
