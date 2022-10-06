#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/python-sdk/main/docs/source/_static/mo-logo.svg",
    issue_tracker_base_url = "https://github.com/modalic/mosaic/issues"
)]
//! The `Aggregator`
//!
//! [FedBuff](https://arxiv.org/abs/2106.06639) for buffered asynchronous aggregation is
//! the baseline algorithm that is implemented as raw framework.
//! For avoiding a Trusted Execution Environment (TEE) and still performing secure aggregation,
//! the [BASecAgg](https://arxiv.org/abs/2110.02177) protocol is choosen.
//!
pub mod aggr;
pub mod rest;
pub mod services;
pub mod settings;
pub mod state_engine;
pub mod storage;
