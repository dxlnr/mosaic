// use std::convert::Infallible;
// use tracing::warn;
// use warp::{Filter, Reply};


// /// Communicates running stats about the aggregation process.
// async fn process_stats() -> Result<impl warp::Reply, Infallible> {
//     let _ = handler.handle_stats().await.map_err(|err| {
//         warn!("processing stats data failed: {:?}", err);
//     });
//     Ok(warp::reply())
// }

/// process statistics update event.
pub type StatsUpdate = Option<Stats>;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Stats {
    pub loss: Vec<f32>,
}