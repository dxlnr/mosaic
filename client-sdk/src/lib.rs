#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/python-sdk/main/docs/source/_static/mo-logo.svg",
    issue_tracker_base_url = "https://github.com/modalic/mosaic/issues"
)]
//! SDK for implementing the protocol on the client side.
//!
pub mod client;
pub mod configs;
pub mod state_engine;

pub use self::{
    client::{Client, Task},
    configs::Conf,
};