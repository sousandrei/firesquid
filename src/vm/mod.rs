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

    if state::get_vm_pid(state_ptr.clone(), &name).await.is_some() {
        return Err(RuntimeError::new(&format!(
            "Vm name already used [{}]",
            &name
        )));
    }

    if drive::create_drive(
        &name,
        &state_ptr.tmp_dir,
        &state_ptr.assets_dir,
        &state_ptr.drive_name,
    )
    .is_err()
    {
        drive::delete_drive(&name, &state_ptr.tmp_dir)?;
        socket::delete_socket(&name, &state_ptr.tmp_dir)?;
        return Err(RuntimeError::new("Error creating drive"));
    };

    task::spawn(async move {
        let mut child = match child::spawn_process(&name, state_ptr.clone()).await {
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

        match child.id() {
            Some(id) => state::add_vm(state_ptr.clone(), &name, id).await,
            None => error!("Failed to start machine, proceeding to teardown [{}]", name),
        };

        if child.wait().await.is_err() {
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
