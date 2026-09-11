//! Request dispatch for the local daemon control protocol.

use crate::{
    error::Error,
    protocol::{Operation, Response, ResponseBody},
    runtime,
    state::{SharedState, VmSpec, VmStatus},
};

pub async fn dispatch(state: &SharedState, request_id: u64, operation: Operation) -> Response {
    let result: Result<ResponseBody, Error> = async {
        match operation {
            Operation::Status => Ok(ResponseBody::Status {
                service: "firesquid".to_owned(),
            }),
            Operation::VmList => state.list_vms().await.map(ResponseBody::VmList),
            Operation::VmCreate {
                name,
                kernel_path,
                rootfs_path,
                vcpus,
                memory_mib,
            } => state
                .create_vm_with_spec(&VmSpec {
                    name,
                    kernel_path,
                    rootfs_path,
                    vcpus,
                    memory_mib,
                })
                .await
                .map(ResponseBody::Vm),
            Operation::VmStart { id } => runtime::start(state, &id)
                .await
                .map(|_| ResponseBody::Empty),
            Operation::VmStop { id } => {
                runtime::stop(state, &id).await.map(|_| ResponseBody::Empty)
            }
            Operation::VmKill { id } => {
                runtime::kill(state, &id).await.map(|_| ResponseBody::Empty)
            }
            Operation::VmInspect { id } => match state.get_vm(&id).await {
                Ok(Some(vm)) => Ok(ResponseBody::Vm(vm)),
                Ok(None) => Err(Error::InvalidRequest(format!("VM not found: {id}"))),
                Err(error) => Err(error),
            },
            Operation::VmDelete { id } => match state.get_vm(&id).await {
                Ok(Some(vm))
                    if matches!(
                        vm.status,
                        VmStatus::Created | VmStatus::Stopped | VmStatus::Failed
                    ) =>
                {
                    if state.delete_vm(&id).await? {
                        Ok(ResponseBody::Empty)
                    } else {
                        Err(Error::InvalidRequest(format!("VM not found: {id}")))
                    }
                }
                Ok(Some(_)) => Err(Error::InvalidRequest(format!("VM is active: {id}"))),
                Ok(None) => Err(Error::InvalidRequest(format!("VM not found: {id}"))),
                Err(error) => Err(error),
            },
            Operation::VmLogs { id } => {
                state
                    .get_vm(&id)
                    .await?
                    .ok_or_else(|| Error::InvalidRequest(format!("VM not found: {id}")))?;
                match tokio::fs::read_to_string(state.config.vm_log(&id)).await {
                    Ok(logs) => Ok(ResponseBody::Logs(logs)),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        Ok(ResponseBody::Logs(String::new()))
                    }
                    Err(error) => Err(error.into()),
                }
            }
        }
    }
    .await;

    match result {
        Ok(body) => Response::ok(request_id, body),
        Err(error) => Response::error(request_id, &error),
    }
}
