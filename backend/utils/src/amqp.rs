use std::time::Duration;

use deadpool_lapin::{Config, PoolConfig, Timeouts};

pub fn amqp_config(num_cpus: usize) -> Config {
    let timeouts = Timeouts {
        create: Some(Duration::from_secs(5)),
        wait: Some(Duration::from_secs(5)),
        recycle: Some(Duration::from_secs(5)),
    };
    let pool_config = PoolConfig {
        max_size: num_cpus,
        timeouts,
        ..Default::default()
    };
    Config {
        url: Some("amqp://guest:guest@localhost:5672".into()),
        pool: Some(pool_config),
        ..Default::default()
    }
}
