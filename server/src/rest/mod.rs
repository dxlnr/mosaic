use std::convert::Infallible;
use warp::{Filter, Reply};

use crate::settings::APISettings;

pub async fn serve(
    api_settings: APISettings
) -> Result<(), Infallible>
{
    let routes = warp::path::end().map(|| "Rest API up & running.");
    return run_http(routes, api_settings).await;
}

async fn run_http<F>(filter: F, api_settings: APISettings) -> Result<(), Infallible>
where
    F: Filter + Clone + Send + Sync + 'static,
    F::Extract: Reply,
{
    warp::serve(filter).run(api_settings.rest_api).await;
    Ok(())
}