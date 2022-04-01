#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, forbid(warnings))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/modalic/mosaic/public/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/modalic/mosaic/public/logo.svg"
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
