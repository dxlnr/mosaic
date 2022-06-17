#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/mosaic/main/public/mo-logo.svg?token=GHSAT0AAAAAABRDIVC2CE73DYQ74XURKGUIYVMMY6Q",
    issue_tracker_base_url = "https://github.com/modalic/mosaic/issues"
)]
//! # Mosaic Aggregation Server for Federated Learning

//! The Mosaic crate serves as the backbone of a MLOps platform designed for enabling Federated Learning.
//! All the aggregation of local Machine Learning models converge at Mosaic server
//! which aims for safety, reliability and performance.
//!
//! ## Using the Mosaic Server
//!
pub mod core;
pub mod db;
pub mod engine;
pub mod proxy;
pub mod rest;
pub mod service;
pub mod settings;
