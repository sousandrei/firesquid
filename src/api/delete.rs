use tracing::error;
use warp::http::StatusCode;

use super::VmInput;
use crate::state;
use crate::state::StatePtr;
use crate::vm;

pub async fn handler(
    body: VmInput,
    state_ptr: StatePtr,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    if state::get_vm_pid(state_ptr.clone(), &body.vm_name)
        .await
        .is_none()
    {
        return Ok(Box::new(StatusCode::NOT_FOUND));
    };

    if let Err(e) = vm::terminate(&body.vm_name).await {
        error!("{}", e);

        return Ok(Box::new(warp::reply::with_status(
            e.to_string(),
            StatusCode::BAD_REQUEST,
        )));
    };

    Ok(Box::new(StatusCode::OK))
}
