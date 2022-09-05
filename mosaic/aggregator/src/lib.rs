#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/python-sdk/main/docs/source/_static/mo-logo.svg",
    issue_tracker_base_url = "https://github.com/modalic/mosaic/issues"
)]
//! The `Aggregator`
//!
//! [FedBuff](https://arxiv.org/abs/2106.06639) for buffered asynchronous aggregation is
//! implemented as aggregation algorithm.
pub mod aggr;
// pub mod buffer;
