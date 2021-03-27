use tracing::error;
use warp::http::StatusCode;

use super::VmInput;
use crate::state;
use crate::state::StatePtr;

//TODO: process kill into vm package
pub async fn handler(
    body: VmInput,
    state_ptr: StatePtr,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let pid = state::get_vm_pid(state_ptr.clone(), &body.vm_name)
        .await
        .unwrap_or(0);

    if pid == 0 {
        return Ok(Box::new(StatusCode::NOT_FOUND));
    }

    let mut child = match tokio::process::Command::new("kill")
        .arg(pid.to_string())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            error!("{}", e);

            return Ok(Box::new(warp::reply::with_status(
                e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )));
        }
    };

    if let Err(e) = child.wait().await {
        error!("{}", e);

        return Ok(Box::new(warp::reply::with_status(
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )));
    };

    Ok(Box::new(StatusCode::OK))
}
