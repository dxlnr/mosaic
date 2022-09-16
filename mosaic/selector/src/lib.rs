#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/python-sdk/main/docs/source/_static/mo-logo.svg",
    issue_tracker_base_url = "https://github.com/modalic/mosaic/issues"
)]

//! The `selector` is responsible for accepting and forwarding device connections.
//! It periodically receives information from the Coordinator about how many devices are needed, 
//! which they use to make local decisions about whether or not to accept each device.
//! 
//! It is the only component that directly communicates with clients.
//! 
//! The Selector has two main responsibilities:
//! - **Client selection**. The Selector advertises available tasks to clients,
//! and summarizes current client availability for the Coordinator.
//! - **Client participation**. The Selector routes client requests to the corresponding Aggregator.
//! 
//! Therefore, the `selector` can be interpreted as a proxy server.
//! 
pub mod proxy;


struct Selector {}