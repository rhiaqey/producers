use rhiaqey_common::env::RedisSettings;
use rustis::client::Client;

pub async fn connect(settings: RedisSettings) -> Option<Client> {
    let connect_uri = match settings.redis_address {
        None => format!(
            "redis+sentinel://:{}@{}/{}?sentinel_password={}",
            settings.redis_password,
            settings.redis_sentinel_addresses.unwrap(),
            settings.redis_sentinel_master,
            settings.redis_password
        ),
        Some(_) => format!(
            "redis://:{}@{}",
            settings.redis_password,
            settings.redis_address.unwrap()
        ),
    };

    let result = Client::connect(connect_uri).await.unwrap();
    Some(result)
}
