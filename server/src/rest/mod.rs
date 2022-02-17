pub mod stats;

use std::convert::Infallible;
use warp::{Filter, Reply};

use crate::{settings::APISettings, service::fetch::Fetch};

pub async fn serve<F>(
    api_settings: &APISettings,
    _fetcher: F,
) -> Result<(), Infallible>
where
    F: Fetch + Sync + Send + 'static + Clone,
{
    let routes = warp::path::end().map(|| "Rest API up & running.");
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