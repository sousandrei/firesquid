mod child;
mod drive;
mod http;
mod socket;

use tokio::task;
use tracing::{error, info};

use crate::error::RuntimeError;
use crate::state;
use crate::state::StatePtr;

pub async fn spawn(name: &str, state_ptr: StatePtr) -> Result<(), RuntimeError> {
    let name = name.to_owned();

    if let Some(_) = state::get_vm_pid(state_ptr.clone(), &name).await {
        return Err(RuntimeError::new(&format!(
            "Vm name already used [{}]",
            &name
        )));
    }

    if let Err(_) = drive::create_drive(
        &name,
        &state_ptr.tmp_dir,
        &state_ptr.assets_dir,
        &state_ptr.drive_name,
    ) {
        drive::delete_drive(&name, &state_ptr.tmp_dir)?;
        socket::delete_socket(&name, &state_ptr.tmp_dir)?;
        return Err(RuntimeError::new("Error creating drive"));
    };

    task::spawn(async move {
        let child = match child::spawn_process(&name, state_ptr.clone()).await {
            Ok(i) => i,
            Err(e) => {
                return {
                    error!(
                        "Failed to start machine, proceeding to teardown [{}, {}]",
                        name,
                        e.to_string()
                    );

                    drive::delete_drive(&name, &state_ptr.tmp_dir).unwrap();
                    socket::delete_socket(&name, &state_ptr.tmp_dir).unwrap();
                }
            }
        };

        state::add_vm(state_ptr.clone(), &name, child.id()).await;

        if let Err(_) = child.await {
            error!("Failed to start machine, proceeding to teardown [{}]", name);
        };

        drive::delete_drive(&name, &state_ptr.tmp_dir).unwrap();
        socket::delete_socket(&name, &state_ptr.tmp_dir).unwrap();

        state::remove_vm(state_ptr.clone(), &name).await;

        info!("Terminated [{}]", &name);
    });

    Ok(())
}

pub async fn terminate(name: &str) -> Result<(), RuntimeError> {
    child::stop_machine(name).await?;
    Ok(())
}
