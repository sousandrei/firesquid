use crate::{
    error::Error,
    protocol::{Operation, ResponseBody, VmInfo},
    runtime,
    state::{SharedState, VmSpec, VmStatus},
};
pub async fn handle(state: &SharedState, operation: Operation) -> Result<ResponseBody, Error> {
    match operation {
        Operation::Status => status(state).await,
        Operation::VmList => list(state).await,
        Operation::VmCreate {
            name,
            kernel_path,
            rootfs_path,
            vcpus,
            memory_mib,
        } => {
            create(
                state,
                VmSpec {
                    name,
                    kernel_path,
                    rootfs_path,
                    vcpus,
                    memory_mib,
                },
            )
            .await
        }
        Operation::VmStart { id } => start(state, &id).await,
        Operation::VmStop { id } => stop(state, &id).await,
        Operation::VmKill { id } => kill(state, &id).await,
        Operation::VmInspect { id } => inspect(state, &id).await,
        Operation::VmDelete { id } => delete(state, &id).await,
        Operation::VmLogs { id } => logs(state, &id).await,
    }
}

async fn list(state: &SharedState) -> Result<ResponseBody, Error> {
    state.list_vms().await.map(ResponseBody::VmList)
}

async fn status(state: &SharedState) -> Result<ResponseBody, Error> {
    let vms = state
        .list_vms()
        .await?
        .into_iter()
        .map(info)
        .collect::<Vec<_>>();
    let mut enriched = Vec::with_capacity(vms.len());
    for vm in vms {
        enriched.push(vm.await?);
    }
    Ok(ResponseBody::Status {
        service: "firesquid".to_owned(),
        vms: enriched,
    })
}

async fn info(vm: crate::state::Vm) -> Result<VmInfo, Error> {
    let metrics = crate::runtime::collect_metrics(&vm).await?;
    Ok(VmInfo { vm, metrics })
}

async fn create(state: &SharedState, spec: VmSpec) -> Result<ResponseBody, Error> {
    let vm = state.create_vm_with_spec(&spec).await?;
    Ok(ResponseBody::Vm(info(vm).await?))
}

async fn start(state: &SharedState, id: &str) -> Result<ResponseBody, Error> {
    runtime::start(state, id).await.map(|_| ResponseBody::Empty)
}

async fn stop(state: &SharedState, id: &str) -> Result<ResponseBody, Error> {
    runtime::stop(state, id).await.map(|_| ResponseBody::Empty)
}

async fn kill(state: &SharedState, id: &str) -> Result<ResponseBody, Error> {
    runtime::kill(state, id).await.map(|_| ResponseBody::Empty)
}

async fn inspect(state: &SharedState, id: &str) -> Result<ResponseBody, Error> {
    let vm = state
        .get_vm(id)
        .await?
        .ok_or_else(|| Error::InvalidRequest(format!("VM not found: {id}")))?;
    Ok(ResponseBody::Vm(info(vm).await?))
}

async fn delete(state: &SharedState, id: &str) -> Result<ResponseBody, Error> {
    let vm = state
        .get_vm(id)
        .await?
        .ok_or_else(|| Error::InvalidRequest(format!("VM not found: {id}")))?;
    if !matches!(
        vm.status,
        VmStatus::Created | VmStatus::Stopped | VmStatus::Failed
    ) {
        return Err(Error::InvalidRequest(format!("VM is active: {id}")));
    }
    if state.delete_vm(id).await? {
        Ok(ResponseBody::Empty)
    } else {
        Err(Error::InvalidRequest(format!("VM not found: {id}")))
    }
}

async fn logs(state: &SharedState, id: &str) -> Result<ResponseBody, Error> {
    state
        .get_vm(id)
        .await?
        .ok_or_else(|| Error::InvalidRequest(format!("VM not found: {id}")))?;
    match tokio::fs::read_to_string(state.config.vm_log(id)).await {
        Ok(logs) => Ok(ResponseBody::Logs(logs)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(ResponseBody::Logs(String::new()))
        }
        Err(error) => Err(error.into()),
    }
}
