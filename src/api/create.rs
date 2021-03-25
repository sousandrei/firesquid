use tracing::error;
use warp::http::StatusCode;

use super::VmInput;
use crate::state::StatePtr;
use crate::vm;

pub async fn handler(
    body: VmInput,
    state_ptr: StatePtr,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    if let Err(e) = vm::spawn(&body.vm_name, state_ptr).await {
        error!("{}", e);

        return Ok(Box::new(warp::reply::with_status(
            e.to_string(),
            StatusCode::BAD_REQUEST,
        )));
    };

    Ok(Box::new(StatusCode::CREATED))
}
