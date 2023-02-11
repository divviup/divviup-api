use divviup_api::{divviup_api, ApiConfig};

fn main() {
    env_logger::init();
    let config = ApiConfig::from_env().expect("Missing config:");
    trillium_tokio::run(divviup_api(config));
}
