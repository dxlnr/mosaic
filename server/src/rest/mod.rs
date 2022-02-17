pub mod stats;

use serde_json::json;
use std::convert::Infallible;
use tracing::warn;
use warp::{
    http::{Response, StatusCode},
    reply::Reply,
    Filter,
};

use crate::{service::fetch::Fetch, settings::APISettings};

/// fetching the stats
async fn fetch_stats<F: Fetch>(mut fetcher: F) -> Result<impl warp::Reply, Infallible> {
    Ok(match fetcher.fetch_stats().await {
        Err(e) => {
            warn!("fetching the process metrics failed: {:?}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(json!(&{}).to_string())
                .unwrap()
        }
        Ok(None) => Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(json!(&{}).to_string())
            .unwrap(),
        Ok(Some(stats)) => {
            let res = serde_json::to_string(&stats).unwrap();
            Response::builder()
                .status(StatusCode::OK)
                .body(json!(&res).to_string())
                .unwrap()
        }
    })
}

pub async fn serve<F>(api_settings: &APISettings, fetcher: F) -> Result<(), Infallible>
where
    F: Fetch + Sync + Send + 'static + Clone,
{
    let entry = warp::path::end().map(|| "Rest API up & running.");
    let stats = warp::path!("stats")
        .and(warp::get())
        .and(with_fetcher(fetcher.clone()))
        .and_then(fetch_stats);
    let routes = entry.or(stats).with(warp::log("http"));
    return run_http(routes, api_settings).await;
}

async fn run_http<F>(filter: F, api_settings: &APISettings) -> Result<(), Infallible>
where
    F: Filter + Clone + Send + Sync + 'static,
    F::Extract: Reply,
{
    warp::serve(filter).run(api_settings.rest_api).await;
    Ok(())
}

/// Converts a data fetcher into a [`warp`] filter.
fn with_fetcher<F: Fetch + Sync + Send + 'static + Clone>(
    fetcher: F,
) -> impl Filter<Extract = (F,), Error = Infallible> + Clone {
    warp::any().map(move || fetcher.clone())
}
