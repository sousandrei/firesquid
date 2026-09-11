//! Request dispatch for the local daemon control protocol.

use crate::{
    error::Error,
    protocol::{Operation, Response, ResponseBody},
    state::SharedState,
};

use super::operations;

pub async fn dispatch(state: &SharedState, request_id: u64, operation: Operation) -> Response {
    let result: Result<ResponseBody, Error> = operations::handle(state, operation).await;

    match result {
        Ok(body) => Response::ok(request_id, body),
        Err(error) => Response::error(request_id, &error),
    }
}
