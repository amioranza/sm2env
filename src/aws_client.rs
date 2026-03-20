use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::config::Region;
use aws_sdk_secretsmanager::Client;

pub async fn build_client(region: Option<&str>, profile: Option<&str>) -> Client {
    let mut config_builder = aws_config::defaults(BehaviorVersion::latest());

    if let Some(r) = region {
        config_builder = config_builder.region(Region::new(r.to_string()));
    }

    if let Some(p) = profile {
        config_builder = config_builder.profile_name(p);
    }

    let config = config_builder.load().await;
    Client::new(&config)
}
