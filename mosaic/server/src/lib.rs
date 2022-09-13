#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/python-sdk/main/docs/source/_static/mo-logo.svg",
    issue_tracker_base_url = "https://github.com/modalic/mosaic/issues"
)]
//! # Mosaic: Aggregation Server for Federated Learning
//!
//! It uses the implementation of FL with buffered asynchronous aggregation
//! that has been recently introduced in [Nguyen et al. 2021](https://arxiv.org/abs/2106.06639).
pub mod settings;