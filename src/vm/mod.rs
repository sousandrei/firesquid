mod child;
mod drive;
mod http;
mod socket;

use tokio::task;
use tracing::{error, info};

use crate::error::RuntimeError;
use crate::state::{StatePtr, Vm};

pub async fn spawn(name: &str, state_ptr: StatePtr) -> Result<(), RuntimeError> {
    let name = String::from(name);

    {
        let vms = state_ptr.vms.clone();
        let vms = vms.lock().await;
        if let Some(_) = vms.iter().position(|vm| vm.name == name) {
            return Err(RuntimeError::new(&format!(
                "Vm name already used [{}]",
                name
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
    }

    task::spawn(async move {
        let child = match child::spawn_process(&name, state_ptr.clone()).await {
            Ok(i) => i,
            Err(e) => {
                return {
                    error!(
                        "Failed to start machine, proceeding to teardown [{}, {}]",
                        &name,
                        e.to_string()
                    );

                    drive::delete_drive(&name, &state_ptr.tmp_dir).unwrap();
                    socket::delete_socket(&name, &state_ptr.tmp_dir).unwrap();
                }
            }
        };

        {
            let mut vms = state_ptr.vms.lock().await;

            vms.push(Vm {
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

        drive::delete_drive(&name, &state_ptr.tmp_dir).unwrap();
        socket::delete_socket(&name, &state_ptr.tmp_dir).unwrap();

        //TODO: vms no copy
        // {
        //     let mut vms = state.vms.lock().await;
        //     if let Some(index) = vms.iter().position(|vm| vm.name == name) {
        //         vms.remove(index);
        //     }
        // }

        info!("Terminated [{}]", name);
    });

    Ok(())
}

pub async fn terminate(name: &str) -> Result<(), RuntimeError> {
    child::stop_machine(name).await?;
    Ok(())
}
