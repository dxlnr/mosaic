#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/python-sdk/main/docs/source/_static/mo-logo.svg",
    issue_tracker_base_url = "https://github.com/modalic/mosaic/issues"
)]

//! The `core` provides basic building blocks for implementing the aggregation procedure.
//! It serves also as a common crate.
pub mod crypto;
pub mod message;
pub mod model;

pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/mosaic.rs"));
}