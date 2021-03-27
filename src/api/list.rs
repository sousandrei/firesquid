use warp::http::StatusCode;

use crate::state;
use crate::state::StatePtr;

pub async fn handler(state_ptr: StatePtr) -> Result<impl warp::Reply, warp::Rejection> {
    let vms = state::get_vms(state_ptr).await;

    Ok(warp::reply::with_status(
        warp::reply::json(&vms),
        StatusCode::OK,
    ))
}
