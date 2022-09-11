#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/python-sdk/main/docs/source/_static/mo-logo.svg",
    issue_tracker_base_url = "https://github.com/modalic/mosaic/issues"
)]

//! The `selector` is the only component that directly communicates with clients.
//! When necessary, it forwards client requests to other components. The Selector has two
//! main responsibilities.
//! - **Client selection**. The Selector advertises available tasks to clients,
//! and summarizes current client availability for the Coordinator.
//! - **Client participation**. The Selector routes client requests to the corresponding Aggregator.
pub mod proxy;


struct Selector {}