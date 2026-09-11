//! Request dispatch for the local daemon control protocol.

use crate::{
    error::Error,
    protocol::{Operation, Response, ResponseBody},
    state::SharedState,
};

pub async fn dispatch(state: &SharedState, request_id: u64, operation: Operation) -> Response {
    let result = match operation {
        Operation::Status => Ok(ResponseBody::Status {
            service: "firesquid".to_owned(),
        }),
        Operation::VmList => state.list_vms().await.map(ResponseBody::VmList),
        Operation::VmCreate { name } => state.create_vm(&name).await.map(ResponseBody::Vm),
        Operation::VmInspect { id } => match state.get_vm(&id).await {
            Ok(Some(vm)) => Ok(ResponseBody::Vm(vm)),
            Ok(None) => Err(Error::InvalidRequest(format!("VM not found: {id}"))),
            Err(error) => Err(error),
        },
        Operation::VmDelete { id } => match state.delete_vm(&id).await {
            Ok(true) => Ok(ResponseBody::Empty),
            Ok(false) => Err(Error::InvalidRequest(format!("VM not found: {id}"))),
            Err(error) => Err(error),
        },
    };

    match result {
        Ok(body) => Response::ok(request_id, body),
        Err(error) => Response::error(request_id, &error),
    }
}
