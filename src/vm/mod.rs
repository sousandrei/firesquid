mod child;
mod drive;
mod http;
mod socket;

use tokio::task;
use tracing::{error, info};

use crate::error::RuntimeError;
use crate::{StatePtr, Vm};

pub async fn spawn(name: &str, state_ptr: StatePtr) -> Result<(), RuntimeError> {
    let name = String::from(name);

    {
        let state = state_ptr.lock().await;
        if let Some(_) = state.vms.iter().position(|vm| vm.name == name) {
            return Err(RuntimeError::new(&format!(
                "Vm name already used [{}]",
                name
            )));
        }

        if let Err(_) =
            drive::create_drive(&name, &state.tmp_dir, &state.assets_dir, &state.drive_name)
        {
            drive::delete_drive(&name, &state.tmp_dir)?;
            socket::delete_socket(&name, &state.tmp_dir).unwrap();
            return Err(RuntimeError::new("Error creating drive"));
        };
    }

    task::spawn(async move {
        let child = child::spawn_process(&name, state_ptr.clone())
            .await
            .unwrap();

        {
            let mut state = state_ptr.lock().await;

            state.vms.push(Vm {
                name: name.clone(),
                pid: child.id(),
            });
        }

        if let Err(_) = child.await {
            error!(
                "Failed to start machine, proceeding to teardown [{}]",
                &name
            );
        };

        {
            let mut state = state_ptr.lock().await;

            drive::delete_drive(&name, &state.tmp_dir).unwrap();
            socket::delete_socket(&name, &state.tmp_dir).unwrap();

            if let Some(index) = state.vms.iter().position(|vm| vm.name == name) {
                state.vms.remove(index);
            }
        }

        info!("Terminated [{}]", name);
    });

    Ok(())
}

pub async fn terminate(name: &str) -> Result<(), RuntimeError> {
    child::stop_machine(name).await?;
    Ok(())
}
