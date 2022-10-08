mod client;
mod settings;

pub use self::{
    client::{Event, Events, InitError, Notifier, Client, Task},
    settings::{Settings, SettingsError},
};

mod reqwest_client;
pub(crate) use reqwest_client::new_client;
pub use reqwest_client::ClientError;
