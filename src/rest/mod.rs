//! Modules implements a REST API which can be used to expose data of the running aggregation process.
//!
pub mod client;
pub mod stats;

use serde_json::json;
use std::convert::Infallible;
use tracing::warn;
use warp::{reply::Reply, Filter};

use crate::service::fetch::Fetch;

/// fetching the stats
async fn fetch_stats<F: Fetch>(mut fetcher: F) -> Result<impl warp::Reply, Infallible> {
    Ok(match fetcher.fetch_stats().await {
        Err(e) => {
            warn!("fetching the process metrics failed: {:?}", e);
            warp::reply::json(&{}).into_response()
        }
        Ok(None) => {
            warn!("no stats data available that can be exposed.");
            warp::reply::json(&{}).into_response()
        }
        Ok(Some(stats)) => {
            let res = json!(&stats);
            warp::reply::json(&res).into_response()
        }
    })
}

pub async fn serve<F>(rest_api: &Option<std::net::SocketAddr>, fetcher: F) -> Result<(), Infallible>
where
    F: Fetch + Sync + Send + 'static + Clone,
{
    match rest_api {
        Some(addr) => {
            let entry = warp::path::end().map(|| "Rest API up & running.");
            let stats = warp::path!("stats")
                .and(warp::get())
                .and(with_fetcher(fetcher.clone()))
                .and_then(fetch_stats);
            let routes = entry.or(stats).with(warp::log("http"));
            run_http(routes, *addr).await
        }
        None => Ok(()),
    }
}

async fn run_http<F>(filter: F, rest_api: std::net::SocketAddr) -> Result<(), Infallible>
where
    F: Filter + Clone + Send + Sync + 'static,
    F::Extract: Reply,
{
    warp::serve(filter).run(rest_api).await;
    Ok(())
}

/// Converts a data fetcher into a [`warp`] filter.
fn with_fetcher<F: Fetch + Sync + Send + 'static + Clone>(
    fetcher: F,
) -> impl Filter<Extract = (F,), Error = Infallible> + Clone {
    warp::any().map(move || fetcher.clone())
}
