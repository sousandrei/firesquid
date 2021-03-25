mod child;
mod drive;
mod http;
mod socket;

use tokio::task;
use tracing::{error, info};

use crate::{error::RuntimeError, state, state::StatePtr};

pub async fn spawn(name: &str, state_ptr: StatePtr) -> Result<(), RuntimeError> {
    let name = name.to_owned();

    if state::get_vm_pid(state_ptr.clone(), &name).await.is_some() {
        return Err(RuntimeError::new(&format!(
            "Vm name already used [{}]",
            &name
        )));
    }

    if let Err(e) = drive::create_drive(&name) {
        drive::delete_drive(&name)?;
        socket::delete_socket(&name)?;

        let err_str = format!("Error creating drive: {}", e);
        return Err(RuntimeError::new(&err_str));
    };

    task::spawn(async move {
        let mut child = match child::spawn_process(&name).await {
            Ok(i) => i,
            Err(e) => {
                return {
                    error!(
                        "Failed to start machine, proceeding to teardown [{}, {}]",
                        name,
                        e.to_string()
                    );

                    drive::delete_drive(&name).unwrap();
                    socket::delete_socket(&name).unwrap();
                }
            }
        };

        match child.id() {
            Some(id) => state::add_vm(state_ptr.clone(), &name, id).await,
            None => error!("Failed to start machine, proceeding to teardown [{}]", name),
        };

        if child.wait().await.is_err() {
            error!("Failed to start machine, proceeding to teardown [{}]", name);
        };

        drive::delete_drive(&name).unwrap();
        socket::delete_socket(&name).unwrap();

        state::remove_vm(state_ptr.clone(), &name).await;

        info!("Terminated [{}]", &name);
    });

    Ok(())
}

pub async fn terminate(name: &str) -> Result<(), RuntimeError> {
    child::stop_machine(name).await?;
    Ok(())
}
