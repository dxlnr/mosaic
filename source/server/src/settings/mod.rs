use tracing_subscriber::filter::EnvFilter;

pub struct Settings {
    pub log: LoggingSettings,
    pub api: APISettings,
}

pub struct LoggingSettings {
    pub filter: EnvFilter,
}

pub struct APISettings {
    pub address: std::net::SocketAddr,
}
